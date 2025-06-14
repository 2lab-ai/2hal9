//! Connection pool wrapper with circuit breaker functionality
//! 
//! This module provides a resilient connection pool that prevents cascading failures
//! and provides graceful degradation when database issues occur.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error};
use crate::database::{DatabasePool, DatabaseConfig};
use crate::circuit_breaker::{CircuitBreaker, CircuitState};

/// Connection pool with circuit breaker protection
pub struct ResilientConnectionPool {
    /// The underlying database pool
    pool: Arc<DatabasePool>,
    
    /// Circuit breaker for connection health
    circuit_breaker: Arc<CircuitBreaker>,
    
    /// Fallback pool for read operations (optional)
    fallback_pool: Option<Arc<DatabasePool>>,
    
    /// Pool health metrics
    metrics: Arc<RwLock<PoolHealthMetrics>>,
}

/// Pool health metrics
#[derive(Debug, Default)]
pub struct PoolHealthMetrics {
    /// Total connection attempts
    pub total_attempts: u64,
    
    /// Successful connections
    pub successful_connections: u64,
    
    /// Failed connections
    pub failed_connections: u64,
    
    /// Circuit breaker trips
    pub circuit_trips: u64,
    
    /// Last error timestamp
    pub last_error: Option<Instant>,
    
    /// Last success timestamp
    pub last_success: Option<Instant>,
}

impl ResilientConnectionPool {
    /// Create a new resilient connection pool
    pub async fn new(
        config: DatabaseConfig,
        fallback_config: Option<DatabaseConfig>,
    ) -> Result<Self> {
        // Create primary pool
        let pool = Arc::new(DatabasePool::new(&config).await?);
        
        // Create fallback pool if config provided
        let fallback_pool = if let Some(fallback) = fallback_config {
            Some(Arc::new(DatabasePool::new(&fallback).await?))
        } else {
            None
        };
        
        // Configure circuit breaker
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            5,                          // failure_threshold
            Duration::from_secs(60),    // timeout_duration
            Duration::from_secs(30),    // half_open_duration
        ));
        
        Ok(Self {
            pool,
            circuit_breaker,
            fallback_pool,
            metrics: Arc::new(RwLock::new(PoolHealthMetrics::default())),
        })
    }
    
    /// Get the primary pool with circuit breaker protection
    pub async fn get_pool(&self) -> Result<Arc<DatabasePool>> {
        // Check circuit breaker state
        match self.circuit_breaker.state() {
            CircuitState::Closed => {
                // Normal operation
                self.record_attempt().await;
                Ok(self.pool.clone())
            }
            CircuitState::Open => {
                // Circuit is open, try fallback or fail
                self.record_circuit_trip().await;
                
                if let Some(fallback) = &self.fallback_pool {
                    warn!("Circuit breaker open, using fallback pool");
                    Ok(fallback.clone())
                } else {
                    Err(anyhow!("Database circuit breaker is open"))
                }
            }
            CircuitState::HalfOpen => {
                // Test if service has recovered
                info!("Circuit breaker half-open, testing connection");
                self.record_attempt().await;
                Ok(self.pool.clone())
            }
        }
    }
    
    /// Execute a database operation with circuit breaker protection
    pub async fn execute<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(Arc<DatabasePool>) -> futures::future::BoxFuture<'static, Result<T>>,
    {
        let pool = self.get_pool().await?;
        
        match operation(pool).await {
            Ok(result) => {
                self.circuit_breaker.record_success();
                self.record_success().await;
                Ok(result)
            }
            Err(err) => {
                self.circuit_breaker.record_failure();
                self.record_failure().await;
                
                // Log the error with context
                error!("Database operation failed: {}", err);
                
                // If we have a fallback pool and this was a read operation,
                // we could retry with fallback here
                
                Err(err)
            }
        }
    }
    
    /// Get current health metrics
    pub async fn health_metrics(&self) -> PoolHealthMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Check if pool is healthy
    pub async fn is_healthy(&self) -> bool {
        matches!(self.circuit_breaker.state(), CircuitState::Closed)
    }
    
    /// Force circuit breaker to close (for testing/recovery)
    pub fn reset_circuit_breaker(&self) {
        self.circuit_breaker.reset();
    }
    
    // Internal metric recording methods
    
    async fn record_attempt(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_attempts += 1;
    }
    
    async fn record_success(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.successful_connections += 1;
        metrics.last_success = Some(Instant::now());
    }
    
    async fn record_failure(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.failed_connections += 1;
        metrics.last_error = Some(Instant::now());
    }
    
    async fn record_circuit_trip(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.circuit_trips += 1;
    }
}

/// Extension trait for DatabasePool to add resilience features
pub trait DatabasePoolExt {
    /// Execute with retry logic
    async fn execute_with_retry<F, T>(&self, operation: F, max_retries: u32) -> Result<T>
    where
        F: Fn() -> futures::future::BoxFuture<'static, Result<T>>;
    
    /// Execute with timeout
    async fn execute_with_timeout<F, T>(&self, operation: F, timeout: Duration) -> Result<T>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T>>;
}

impl DatabasePoolExt for DatabasePool {
    async fn execute_with_retry<F, T>(&self, operation: F, max_retries: u32) -> Result<T>
    where
        F: Fn() -> futures::future::BoxFuture<'static, Result<T>>,
    {
        let mut retries = 0;
        let mut last_error = None;
        
        while retries <= max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    last_error = Some(err);
                    retries += 1;
                    
                    if retries <= max_retries {
                        let delay = Duration::from_millis(100 * (2_u64.pow(retries)));
                        warn!("Database operation failed, retrying in {:?}", delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow!("Unknown error")))
    }
    
    async fn execute_with_timeout<F, T>(&self, operation: F, timeout: Duration) -> Result<T>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T>>,
    {
        match tokio::time::timeout(timeout, operation()).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(err)) => Err(err),
            Err(_) => Err(anyhow!("Database operation timed out")),
        }
    }
}

// Re-export for convenience
pub use crate::database::{DatabaseType, DatabaseOperations, PoolMetrics};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_resilient_pool_creation() {
        let config = DatabaseConfig {
            database_type: DatabaseType::Sqlite,
            url: "sqlite::memory:".to_string(),
            ..Default::default()
        };
        
        let pool = ResilientConnectionPool::new(config, None).await.unwrap();
        assert!(pool.is_healthy().await);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_metrics() {
        let config = DatabaseConfig {
            database_type: DatabaseType::Sqlite,
            url: "sqlite::memory:".to_string(),
            ..Default::default()
        };
        
        let pool = ResilientConnectionPool::new(config, None).await.unwrap();
        let metrics = pool.health_metrics().await;
        
        assert_eq!(metrics.total_attempts, 0);
        assert_eq!(metrics.successful_connections, 0);
        assert_eq!(metrics.failed_connections, 0);
        assert_eq!(metrics.circuit_trips, 0);
    }
}