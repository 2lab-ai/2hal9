//! Optimized connection pooling for 1000+ users

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;
use metrics::{counter, gauge, histogram};
use crate::database::{DatabasePool, DatabaseConfig, DatabaseType, PoolMetrics};
use crate::connection_pool::{ResilientConnectionPool, PoolHealthMetrics};

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum connections per pool
    pub max_connections: u32,
    
    /// Minimum idle connections
    pub min_connections: u32,
    
    /// Connection timeout
    pub connect_timeout: Duration,
    
    /// Idle timeout before closing
    pub idle_timeout: Duration,
    
    /// Maximum lifetime of a connection
    pub max_lifetime: Duration,
    
    /// Statement cache capacity
    pub statement_cache_capacity: usize,
    
    /// Test connections before use
    pub test_before_acquire: bool,
    
    /// Connection retry attempts
    pub connect_retry_count: u32,
    
    /// Backoff multiplier for retries
    pub connect_retry_backoff: f64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            min_connections: 10,
            connect_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(3600),
            statement_cache_capacity: 1000,
            test_before_acquire: true,
            connect_retry_count: 3,
            connect_retry_backoff: 2.0,
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    pub total_connections: u32,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub wait_count: u64,
    pub wait_duration_ms: u64,
    pub timeout_count: u64,
}

/// Optimized connection pool manager
pub struct OptimizedConnectionPool {
    /// Primary pool for writes
    primary_pool: Arc<ResilientConnectionPool>,
    
    /// Read replica pools by region
    read_pools: HashMap<String, Arc<ResilientConnectionPool>>,
    
    /// Pool statistics
    stats: Arc<RwLock<HashMap<String, PoolStats>>>,
    
    /// Configuration
    config: PoolConfig,
    
    /// Database type (must be PostgreSQL for scaling features)
    db_type: DatabaseType,
}

impl OptimizedConnectionPool {
    /// Create new optimized connection pool
    pub async fn new(
        primary_url: &str,
        replica_urls: HashMap<String, String>,
        config: PoolConfig,
    ) -> Result<Self> {
        // Parse database type from URL
        let db_type = if primary_url.starts_with("postgres://") || primary_url.starts_with("postgresql://") {
            DatabaseType::Postgres
        } else {
            return Err(anyhow!("Scaling features require PostgreSQL database"));
        };
        
        // Create primary pool
        let primary_config = DatabaseConfig {
            database_type: db_type,
            url: primary_url.to_string(),
            max_connections: config.max_connections,
            min_connections: config.min_connections,
            connection_timeout: config.connect_timeout,
            idle_timeout: config.idle_timeout,
            max_lifetime: config.max_lifetime,
        };
        
        let primary_pool = Arc::new(ResilientConnectionPool::new(primary_config, None).await?);
        
        // Create read replica pools
        let mut read_pools = HashMap::new();
        for (region, url) in replica_urls {
            let replica_config = DatabaseConfig {
                database_type: db_type,
                url,
                max_connections: config.max_connections,
                min_connections: config.min_connections,
                connection_timeout: config.connect_timeout,
                idle_timeout: config.idle_timeout,
                max_lifetime: config.max_lifetime,
            };
            
            let pool = Arc::new(ResilientConnectionPool::new(replica_config, None).await?);
            read_pools.insert(region, pool);
        }
        
        let stats = Arc::new(RwLock::new(HashMap::new()));
        
        Ok(Self {
            primary_pool,
            read_pools,
            stats,
            config,
            db_type,
        })
    }
    
    /// Get the underlying database pool with PostgreSQL-specific features
    async fn get_pg_pool(&self, pool: &ResilientConnectionPool) -> Result<sqlx::PgPool> {
        let db_pool = pool.get_pool().await?;
        db_pool.as_pg_pool()
            .cloned()
            .ok_or_else(|| anyhow!("Scaling features require PostgreSQL database"))
    }
    
    /// Get primary pool for writes
    pub fn primary(&self) -> Arc<ResilientConnectionPool> {
        self.primary_pool.clone()
    }
    
    /// Get read pool for region
    pub fn read_pool(&self, region: Option<&str>) -> Arc<ResilientConnectionPool> {
        if let Some(region) = region {
            if let Some(pool) = self.read_pools.get(region) {
                return pool.clone();
            }
        }
        
        // Fallback to primary if no replica available
        self.primary_pool.clone()
    }
    
    /// Get the best read pool based on current load
    pub async fn best_read_pool(&self) -> Arc<ResilientConnectionPool> {
        let stats = self.stats.read().await;
        
        let mut best_pool = None;
        let mut lowest_load = f64::MAX;
        
        for (region, pool) in &self.read_pools {
            let pool_stats = stats.get(region).cloned().unwrap_or_default();
            
            // Calculate load factor
            let load = if pool_stats.total_connections > 0 {
                pool_stats.active_connections as f64 / pool_stats.total_connections as f64
            } else {
                0.0
            };
            
            if load < lowest_load {
                lowest_load = load;
                best_pool = Some(pool);
            }
        }
        
        best_pool.map(|p| p.clone()).unwrap_or_else(|| self.primary_pool.clone())
    }
    
    /// Execute with automatic retry
    pub async fn execute_with_retry<T, F>(&self, f: F) -> Result<T>
    where
        F: Fn(Arc<ResilientConnectionPool>) -> futures::future::BoxFuture<'static, Result<T>>,
    {
        let mut attempts = 0;
        let mut backoff = Duration::from_millis(100);
        
        loop {
            let pool = self.primary_pool.clone();
            match f(pool).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    
                    if attempts >= self.config.connect_retry_count {
                        return Err(e);
                    }
                    
                    tracing::warn!(
                        "Database operation failed (attempt {}/{}): {}",
                        attempts,
                        self.config.connect_retry_count,
                        e
                    );
                    
                    tokio::time::sleep(backoff).await;
                    backoff = Duration::from_millis(
                        (backoff.as_millis() as f64 * self.config.connect_retry_backoff) as u64
                    );
                }
            }
        }
    }
    
    /// Update pool statistics
    pub async fn update_stats(&self) {
        let mut stats = self.stats.write().await;
        
        // Update primary pool stats
        if let Ok(db_pool) = self.primary_pool.get_pool().await {
            let metrics = db_pool.metrics();
            let primary_stats = PoolStats {
                total_connections: metrics.max_size,
                idle_connections: metrics.idle,
                active_connections: metrics.size - metrics.idle,
                wait_count: 0, // Would need custom tracking
                wait_duration_ms: 0,
                timeout_count: 0,
            };
            stats.insert("primary".to_string(), primary_stats.clone());
            
            // Metrics
            gauge!("hal9.db.connections.total", primary_stats.total_connections as f64, "pool" => "primary");
            gauge!("hal9.db.connections.idle", primary_stats.idle_connections as f64, "pool" => "primary");
            gauge!("hal9.db.connections.active", primary_stats.active_connections as f64, "pool" => "primary");
        }
        
        // Update replica pool stats
        for (region, pool) in &self.read_pools {
            if let Ok(db_pool) = pool.get_pool().await {
                let metrics = db_pool.metrics();
                let pool_stats = PoolStats {
                    total_connections: metrics.max_size,
                    idle_connections: metrics.idle,
                    active_connections: metrics.size - metrics.idle,
                    wait_count: 0,
                    wait_duration_ms: 0,
                    timeout_count: 0,
                };
                stats.insert(region.clone(), pool_stats.clone());
                
                gauge!("hal9.db.connections.total", pool_stats.total_connections as f64, "pool" => region.clone());
                gauge!("hal9.db.connections.idle", pool_stats.idle_connections as f64, "pool" => region.clone());
                gauge!("hal9.db.connections.active", pool_stats.active_connections as f64, "pool" => region.clone());
            }
        }
    }
    
    /// Get current statistics
    pub async fn get_stats(&self) -> HashMap<String, PoolStats> {
        self.stats.read().await.clone()
    }
    
    /// Health check all pools
    pub async fn health_check(&self) -> Result<HealthReport> {
        let mut report = HealthReport {
            healthy: true,
            pools: HashMap::new(),
        };
        
        // Check primary
        let primary_healthy = self.primary_pool.is_healthy().await;
        report.pools.insert("primary".to_string(), PoolHealth {
            healthy: primary_healthy,
            latency_ms: 0, // Would measure actual latency
            error: if !primary_healthy { Some("Circuit breaker open".to_string()) } else { None },
        });
        
        if !primary_healthy {
            report.healthy = false;
        }
        
        // Check replicas
        for (region, pool) in &self.read_pools {
            let healthy = pool.is_healthy().await;
            report.pools.insert(region.clone(), PoolHealth {
                healthy,
                latency_ms: 0,
                error: if !healthy { Some("Circuit breaker open".to_string()) } else { None },
            });
        }
        
        Ok(report)
    }
    
    /// Graceful shutdown
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down connection pools...");
        
        // Pools will be closed when dropped
        Ok(())
    }
}

/// Health report for all pools
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub healthy: bool,
    pub pools: HashMap<String, PoolHealth>,
}

/// Health status for a single pool
#[derive(Debug, Clone)]
pub struct PoolHealth {
    pub healthy: bool,
    pub latency_ms: u64,
    pub error: Option<String>,
}

/// Connection pool middleware for automatic metrics
pub struct PoolMetricsMiddleware;

impl PoolMetricsMiddleware {
    pub async fn track_query<T, F>(pool_name: &str, query_name: &str, f: F) -> Result<T>
    where
        F: futures::Future<Output = Result<T>>,
    {
        let start = std::time::Instant::now();
        
        counter!("hal9.db.queries.total", 1, 
            "pool" => pool_name.to_string(),
            "query" => query_name.to_string()
        );
        
        match f.await {
            Ok(result) => {
                let duration = start.elapsed();
                histogram!("hal9.db.query.duration", duration.as_millis() as f64,
                    "pool" => pool_name.to_string(),
                    "query" => query_name.to_string(),
                    "status" => "success"
                );
                Ok(result)
            }
            Err(e) => {
                let duration = start.elapsed();
                histogram!("hal9.db.query.duration", duration.as_millis() as f64,
                    "pool" => pool_name.to_string(),
                    "query" => query_name.to_string(),
                    "status" => "error"
                );
                counter!("hal9.db.queries.errors", 1,
                    "pool" => pool_name.to_string(),
                    "query" => query_name.to_string()
                );
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_defaults() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.min_connections, 10);
        assert_eq!(config.statement_cache_capacity, 1000);
        assert!(config.test_before_acquire);
    }
}