//! Optimized connection pooling for 1000+ users

use anyhow::Result;
use sqlx::{PgPool, Pool, Postgres, postgres::PgPoolOptions};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;
use metrics::{counter, gauge, histogram};

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
    primary_pool: PgPool,
    
    /// Read replica pools by region
    read_pools: HashMap<String, PgPool>,
    
    /// Pool statistics
    stats: Arc<RwLock<HashMap<String, PoolStats>>>,
    
    /// Configuration
    config: PoolConfig,
}

impl OptimizedConnectionPool {
    /// Create new optimized connection pool
    pub async fn new(
        primary_url: &str,
        replica_urls: HashMap<String, String>,
        config: PoolConfig,
    ) -> Result<Self> {
        // Create primary pool
        let primary_pool = Self::create_pool("primary", primary_url, &config).await?;
        
        // Create read replica pools
        let mut read_pools = HashMap::new();
        for (region, url) in replica_urls {
            let pool = Self::create_pool(&region, &url, &config).await?;
            read_pools.insert(region, pool);
        }
        
        let stats = Arc::new(RwLock::new(HashMap::new()));
        
        Ok(Self {
            primary_pool,
            read_pools,
            stats,
            config,
        })
    }
    
    /// Create a single pool with configuration
    async fn create_pool(name: &str, url: &str, config: &PoolConfig) -> Result<PgPool> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect_timeout(config.connect_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .test_before_acquire(config.test_before_acquire)
            .before_acquire(move |conn, _meta| {
                Box::pin(async move {
                    // Custom connection preparation
                    sqlx::query("SET application_name = 'hal9_server'")
                        .execute(conn)
                        .await?;
                    
                    // Set statement timeout
                    sqlx::query("SET statement_timeout = '30s'")
                        .execute(conn)
                        .await?;
                    
                    Ok(true)
                })
            })
            .after_connect(move |conn, _meta| {
                Box::pin(async move {
                    // Prepare commonly used statements
                    sqlx::query("PREPARE get_user AS SELECT * FROM users WHERE id = $1")
                        .execute(conn)
                        .await?;
                    
                    sqlx::query("PREPARE get_org AS SELECT * FROM organizations WHERE id = $1")
                        .execute(conn)
                        .await?;
                    
                    counter!("hal9.db.connections.created", 1, "pool" => name.to_string());
                    Ok(())
                })
            })
            .connect(url)
            .await?;
        
        tracing::info!("Created connection pool '{}' with {} connections", name, config.max_connections);
        
        Ok(pool)
    }
    
    /// Get primary pool for writes
    pub fn primary(&self) -> &PgPool {
        &self.primary_pool
    }
    
    /// Get read pool for region
    pub fn read_pool(&self, region: Option<&str>) -> &PgPool {
        if let Some(region) = region {
            if let Some(pool) = self.read_pools.get(region) {
                return pool;
            }
        }
        
        // Fallback to primary if no replica available
        &self.primary_pool
    }
    
    /// Get the best read pool based on current load
    pub async fn best_read_pool(&self) -> &PgPool {
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
        
        best_pool.unwrap_or(&self.primary_pool)
    }
    
    /// Execute with automatic retry
    pub async fn execute_with_retry<T, F>(&self, f: F) -> Result<T>
    where
        F: Fn(&PgPool) -> futures::future::BoxFuture<'_, Result<T>>,
    {
        let mut attempts = 0;
        let mut backoff = Duration::from_millis(100);
        
        loop {
            match f(&self.primary_pool).await {
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
        let primary_stats = PoolStats {
            total_connections: self.primary_pool.size(),
            idle_connections: self.primary_pool.num_idle(),
            active_connections: self.primary_pool.size() - self.primary_pool.num_idle(),
            wait_count: 0, // Would need custom tracking
            wait_duration_ms: 0,
            timeout_count: 0,
        };
        stats.insert("primary".to_string(), primary_stats.clone());
        
        // Metrics
        gauge!("hal9.db.connections.total", primary_stats.total_connections as f64, "pool" => "primary");
        gauge!("hal9.db.connections.idle", primary_stats.idle_connections as f64, "pool" => "primary");
        gauge!("hal9.db.connections.active", primary_stats.active_connections as f64, "pool" => "primary");
        
        // Update replica pool stats
        for (region, pool) in &self.read_pools {
            let pool_stats = PoolStats {
                total_connections: pool.size(),
                idle_connections: pool.num_idle(),
                active_connections: pool.size() - pool.num_idle(),
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
        match sqlx::query("SELECT 1").fetch_one(&self.primary_pool).await {
            Ok(_) => {
                report.pools.insert("primary".to_string(), PoolHealth {
                    healthy: true,
                    latency_ms: 0, // Would measure actual latency
                    error: None,
                });
            }
            Err(e) => {
                report.healthy = false;
                report.pools.insert("primary".to_string(), PoolHealth {
                    healthy: false,
                    latency_ms: 0,
                    error: Some(e.to_string()),
                });
            }
        }
        
        // Check replicas
        for (region, pool) in &self.read_pools {
            match sqlx::query("SELECT 1").fetch_one(pool).await {
                Ok(_) => {
                    report.pools.insert(region.clone(), PoolHealth {
                        healthy: true,
                        latency_ms: 0,
                        error: None,
                    });
                }
                Err(e) => {
                    // Replica failure doesn't make system unhealthy
                    report.pools.insert(region.clone(), PoolHealth {
                        healthy: false,
                        latency_ms: 0,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        
        Ok(report)
    }
    
    /// Graceful shutdown
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down connection pools...");
        
        // Close all pools
        self.primary_pool.close().await;
        
        for (region, pool) in &self.read_pools {
            tracing::info!("Closing pool for region {}", region);
            pool.close().await;
        }
        
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