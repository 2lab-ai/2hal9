//! Simplified Redis cache implementation that works around bb8-redis issues

use anyhow::{Result, anyhow};
use redis::{Client, AsyncCommands, aio::MultiplexedConnection};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use tokio::sync::Mutex;
use std::sync::Arc;
use tracing::{info, debug};

/// Simple Redis connection wrapper
#[allow(dead_code)]
pub struct SimpleRedisPool {
    client: Client,
    connections: Arc<Mutex<Vec<MultiplexedConnection>>>,
    max_connections: usize,
}

impl SimpleRedisPool {
    pub async fn new(url: &str, max_connections: usize) -> Result<Self> {
        let client = Client::open(url)
            .map_err(|e| anyhow!("Failed to create Redis client: {}", e))?;
        
        // Pre-create some connections
        let mut connections = Vec::with_capacity(max_connections);
        for _ in 0..std::cmp::min(2, max_connections) {
            let conn = client.get_multiplexed_async_connection().await
                .map_err(|e| anyhow!("Failed to create connection: {}", e))?;
            connections.push(conn);
        }
        
        Ok(Self {
            client,
            connections: Arc::new(Mutex::new(connections)),
            max_connections,
        })
    }
    
    pub async fn get_connection(&self) -> Result<MultiplexedConnection> {
        // MultiplexedConnection can be cloned and shared
        let pool = self.connections.lock().await;
        
        // Try to reuse existing connection
        if let Some(conn) = pool.first() {
            return Ok(conn.clone());
        }
        
        // Create new connection if none exist
        drop(pool); // Release lock before creating connection
        let conn = self.client.get_multiplexed_async_connection().await
            .map_err(|e| anyhow!("Failed to create connection: {}", e))?;
        
        // Store for reuse
        let mut pool = self.connections.lock().await;
        if pool.is_empty() {
            pool.push(conn.clone());
        }
        
        Ok(conn)
    }
}

/// Type alias for compatibility
pub type RedisPool = Arc<SimpleRedisPool>;

/// Cache configuration (same as before)
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: Duration,
    pub default_ttl: Duration,
    pub key_prefix: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            max_connections: 50,
            min_connections: 10,
            connection_timeout: Duration::from_secs(5),
            default_ttl: Duration::from_secs(3600),
            key_prefix: "hal9".to_string(),
        }
    }
}

/// Redis cache pool
pub struct CachePool {
    pool: RedisPool,
    config: CacheConfig,
}

impl CachePool {
    /// Create new cache pool
    pub async fn new(config: CacheConfig) -> Result<Self> {
        info!("Connecting to Redis: {}", config.url);
        
        let simple_pool = SimpleRedisPool::new(&config.url, config.max_connections as usize).await?;
        let pool = Arc::new(simple_pool);
        
        Ok(Self { pool, config })
    }
    
    /// Build a cache key with prefix
    fn build_key(&self, key: &str) -> String {
        format!("{}:{}", self.config.key_prefix, key)
    }
    
    /// Set a value with TTL
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()> {
        let mut conn = self.pool.get_connection().await?;
        let key = self.build_key(key);
        let value = serde_json::to_string(value)?;
        let ttl = ttl.unwrap_or(self.config.default_ttl);
        
        let _: () = conn.set_ex(&key, value, ttl.as_secs() as usize).await
            .map_err(|e| anyhow!("Redis SET error: {}", e))?;
        
        debug!("Cache SET: {} (TTL: {}s)", key, ttl.as_secs());
        Ok(())
    }
    
    /// Get a value
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.pool.get_connection().await?;
        let key = self.build_key(key);
        
        let value: Option<String> = conn.get(&key).await
            .map_err(|e| anyhow!("Redis GET error: {}", e))?;
        
        match value {
            Some(v) => {
                debug!("Cache HIT: {}", key);
                let parsed = serde_json::from_str(&v)?;
                Ok(Some(parsed))
            }
            None => {
                debug!("Cache MISS: {}", key);
                Ok(None)
            }
        }
    }
    
    /// Delete a value
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.pool.get_connection().await?;
        let key = self.build_key(key);
        
        let _: () = conn.del(&key).await
            .map_err(|e| anyhow!("Redis DEL error: {}", e))?;
        
        debug!("Cache DELETE: {}", key);
        Ok(())
    }
    
    /// Check if key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.pool.get_connection().await?;
        let key = self.build_key(key);
        
        let exists: bool = conn.exists(&key).await
            .map_err(|e| anyhow!("Redis EXISTS error: {}", e))?;
        
        Ok(exists)
    }
    
    /// Increment a counter
    pub async fn incr(&self, key: &str) -> Result<i64> {
        let mut conn = self.pool.get_connection().await?;
        let key = self.build_key(key);
        
        let value: i64 = conn.incr(&key, 1).await
            .map_err(|e| anyhow!("Redis INCR error: {}", e))?;
        
        Ok(value)
    }
    
    /// Get pool metrics
    pub async fn metrics(&self) -> PoolMetrics {
        let pool = self.pool.connections.lock().await;
        PoolMetrics {
            connections: pool.len() as u32,
            idle_connections: pool.len() as u32,
        }
    }
}

/// Pool metrics
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    pub connections: u32,
    pub idle_connections: u32,
}

/// Cache key builders (same as before)
pub struct CacheKeys;

impl CacheKeys {
    pub fn user(user_id: &uuid::Uuid) -> String {
        format!("user:{}", user_id)
    }
    
    pub fn user_sessions(user_id: &uuid::Uuid) -> String {
        format!("user:{}:sessions", user_id)
    }
    
    pub fn api_key(key_hash: &str) -> String {
        format!("api_key:{}", key_hash)
    }
    
    pub fn neuron(neuron_id: &str) -> String {
        format!("neuron:{}", neuron_id)
    }
    
    pub fn neuron_state(neuron_id: &str) -> String {
        format!("neuron:{}:state", neuron_id)
    }
    
    pub fn neuron_metrics(neuron_id: &str) -> String {
        format!("neuron:{}:metrics", neuron_id)
    }
    
    pub fn signal(signal_id: &uuid::Uuid) -> String {
        format!("signal:{}", signal_id)
    }
    
    pub fn signal_batch(batch_id: &uuid::Uuid) -> String {
        format!("signal:batch:{}", batch_id)
    }
    
    pub fn memory_search(query_hash: &str) -> String {
        format!("memory:search:{}", query_hash)
    }
    
    pub fn recent_memories(neuron_id: &str) -> String {
        format!("memory:{}:recent", neuron_id)
    }
}