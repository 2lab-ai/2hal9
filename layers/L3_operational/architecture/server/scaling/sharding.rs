//! Database sharding for horizontal scaling

use anyhow::Result;
use sqlx::{PgPool, Postgres};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use uuid::Uuid;
use async_trait::async_trait;

/// Sharding configuration
#[derive(Debug, Clone)]
pub struct ShardingConfig {
    pub shards: Vec<ShardConfig>,
    pub replication_factor: u32,
    pub read_preference: ReadPreference,
}

/// Individual shard configuration
#[derive(Debug, Clone)]
pub struct ShardConfig {
    pub id: u32,
    pub name: String,
    pub primary_url: String,
    pub replica_urls: Vec<String>,
    pub key_range: KeyRange,
}

/// Key range for a shard
#[derive(Debug, Clone)]
pub struct KeyRange {
    pub start: u64,
    pub end: u64,
}

/// Read preference for queries
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReadPreference {
    Primary,
    SecondaryPreferred,
    Secondary,
    Nearest,
}

/// Sharding strategy implementation
pub struct ShardingStrategy {
    config: ShardingConfig,
    shard_pools: HashMap<u32, ShardPool>,
}

/// Pool for a single shard
struct ShardPool {
    primary: PgPool,
    replicas: Vec<PgPool>,
}

impl ShardingStrategy {
    /// Create new sharding strategy
    pub async fn new(config: ShardingConfig) -> Result<Self> {
        let mut shard_pools = HashMap::new();
        
        for shard in &config.shards {
            let primary = PgPool::connect(&shard.primary_url).await?;
            
            let mut replicas = Vec::new();
            for replica_url in &shard.replica_urls {
                let pool = PgPool::connect(replica_url).await?;
                replicas.push(pool);
            }
            
            shard_pools.insert(shard.id, ShardPool { primary, replicas });
        }
        
        Ok(Self {
            config,
            shard_pools,
        })
    }
    
    /// Get shard ID for a user
    pub fn get_shard_for_user(&self, user_id: Uuid) -> u32 {
        let hash = self.hash_uuid(user_id);
        let shard_count = self.config.shards.len() as u64;
        (hash % shard_count) as u32
    }
    
    /// Get shard ID for an organization
    pub fn get_shard_for_organization(&self, org_id: Uuid) -> u32 {
        // Organizations are sharded the same way as users
        self.get_shard_for_user(org_id)
    }
    
    /// Get write pool for a shard
    pub fn get_write_pool(&self, shard_id: u32) -> Result<&PgPool> {
        self.shard_pools
            .get(&shard_id)
            .map(|sp| &sp.primary)
            .ok_or_else(|| anyhow::anyhow!("Shard {} not found", shard_id))
    }
    
    /// Get read pool based on preference
    pub fn get_read_pool(&self, shard_id: u32) -> Result<&PgPool> {
        let shard_pool = self.shard_pools
            .get(&shard_id)
            .ok_or_else(|| anyhow::anyhow!("Shard {} not found", shard_id))?;
        
        match self.config.read_preference {
            ReadPreference::Primary => Ok(&shard_pool.primary),
            ReadPreference::Secondary | ReadPreference::SecondaryPreferred => {
                if !shard_pool.replicas.is_empty() {
                    // Round-robin replica selection
                    let index = rand::random::<usize>() % shard_pool.replicas.len();
                    Ok(&shard_pool.replicas[index])
                } else if self.config.read_preference == ReadPreference::SecondaryPreferred {
                    Ok(&shard_pool.primary)
                } else {
                    Err(anyhow::anyhow!("No replicas available for shard {}", shard_id))
                }
            }
            ReadPreference::Nearest => {
                // In production, would check latency
                // For now, randomly choose between primary and replicas
                if shard_pool.replicas.is_empty() || rand::random::<bool>() {
                    Ok(&shard_pool.primary)
                } else {
                    let index = rand::random::<usize>() % shard_pool.replicas.len();
                    Ok(&shard_pool.replicas[index])
                }
            }
        }
    }
    
    /// Consistent hashing for UUID
    fn hash_uuid(&self, id: Uuid) -> u64 {
        let bytes = id.as_bytes();
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Get all shard IDs
    pub fn get_all_shard_ids(&self) -> Vec<u32> {
        self.config.shards.iter().map(|s| s.id).collect()
    }
}

/// Sharding manager for database operations
pub struct ShardingManager {
    strategy: ShardingStrategy,
}

impl ShardingManager {
    pub fn new(strategy: ShardingStrategy) -> Self {
        Self { strategy }
    }
    
    /// Execute query on correct shard
    pub async fn execute_for_user<T, F>(&self, user_id: Uuid, f: F) -> Result<T>
    where
        F: FnOnce(&PgPool) -> futures::future::BoxFuture<'_, Result<T>>,
    {
        let shard_id = self.strategy.get_shard_for_user(user_id);
        let pool = self.strategy.get_write_pool(shard_id)?;
        f(pool).await
    }
    
    /// Query on correct shard with read preference
    pub async fn query_for_user<T, F>(&self, user_id: Uuid, f: F) -> Result<T>
    where
        F: FnOnce(&PgPool) -> futures::future::BoxFuture<'_, Result<T>>,
    {
        let shard_id = self.strategy.get_shard_for_user(user_id);
        let pool = self.strategy.get_read_pool(shard_id)?;
        f(pool).await
    }
    
    /// Execute across all shards (for admin operations)
    pub async fn execute_on_all_shards<T, F>(&self, f: F) -> Result<Vec<T>>
    where
        F: Fn(&PgPool) -> futures::future::BoxFuture<'_, Result<T>> + Clone,
    {
        let mut results = Vec::new();
        
        for shard_id in self.strategy.get_all_shard_ids() {
            let pool = self.strategy.get_write_pool(shard_id)?;
            let result = f(pool).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Query across all shards (for aggregations)
    pub async fn query_all_shards<T, F>(&self, f: F) -> Result<Vec<T>>
    where
        F: Fn(&PgPool) -> futures::future::BoxFuture<'_, Result<Vec<T>>> + Clone,
    {
        let mut all_results = Vec::new();
        
        for shard_id in self.strategy.get_all_shard_ids() {
            let pool = self.strategy.get_read_pool(shard_id)?;
            let results = f(pool).await?;
            all_results.extend(results);
        }
        
        Ok(all_results)
    }
}

/// Cross-shard transaction support
pub struct CrossShardTransaction {
    transactions: HashMap<u32, sqlx::Transaction<'static, Postgres>>,
}

impl CrossShardTransaction {
    /// Begin transaction across multiple shards
    pub async fn begin(
        strategy: &ShardingStrategy,
        shard_ids: Vec<u32>
    ) -> Result<Self> {
        let mut transactions = HashMap::new();
        
        for shard_id in shard_ids {
            let pool = strategy.get_write_pool(shard_id)?;
            let tx = pool.begin().await?;
            transactions.insert(shard_id, tx);
        }
        
        Ok(Self { transactions })
    }
    
    /// Commit all transactions
    pub async fn commit(self) -> Result<()> {
        for (_, tx) in self.transactions {
            tx.commit().await?;
        }
        Ok(())
    }
    
    /// Rollback all transactions
    pub async fn rollback(self) -> Result<()> {
        for (_, tx) in self.transactions {
            tx.rollback().await?;
        }
        Ok(())
    }
}

/// Shard-aware repository trait
#[async_trait]
pub trait ShardedRepository {
    type Entity;
    
    /// Get entity by ID from correct shard
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Self::Entity>>;
    
    /// Create entity in correct shard
    async fn create(&self, entity: Self::Entity) -> Result<()>;
    
    /// Update entity in correct shard
    async fn update(&self, entity: Self::Entity) -> Result<()>;
    
    /// Delete entity from correct shard
    async fn delete(&self, id: Uuid) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_hashing() {
        let config = ShardingConfig {
            shards: vec![
                ShardConfig {
                    id: 0,
                    name: "shard0".to_string(),
                    primary_url: "postgres://localhost/shard0".to_string(),
                    replica_urls: vec![],
                    key_range: KeyRange { start: 0, end: u64::MAX / 3 },
                },
                ShardConfig {
                    id: 1,
                    name: "shard1".to_string(),
                    primary_url: "postgres://localhost/shard1".to_string(),
                    replica_urls: vec![],
                    key_range: KeyRange { start: u64::MAX / 3, end: (u64::MAX / 3) * 2 },
                },
                ShardConfig {
                    id: 2,
                    name: "shard2".to_string(),
                    primary_url: "postgres://localhost/shard2".to_string(),
                    replica_urls: vec![],
                    key_range: KeyRange { start: (u64::MAX / 3) * 2, end: u64::MAX },
                },
            ],
            replication_factor: 1,
            read_preference: ReadPreference::Primary,
        };
        
        // Test that same UUID always maps to same shard
        let user_id = Uuid::new_v4();
        let strategy = ShardingStrategy {
            config,
            shard_pools: HashMap::new(),
        };
        
        let shard1 = strategy.get_shard_for_user(user_id);
        let shard2 = strategy.get_shard_for_user(user_id);
        assert_eq!(shard1, shard2);
        
        // Test distribution
        let mut shard_counts = vec![0u32; 3];
        for _ in 0..1000 {
            let id = Uuid::new_v4();
            let shard = strategy.get_shard_for_user(id);
            shard_counts[shard as usize] += 1;
        }
        
        // Check roughly even distribution (within 20%)
        for count in shard_counts {
            assert!(count > 250 && count < 400);
        }
    }
}