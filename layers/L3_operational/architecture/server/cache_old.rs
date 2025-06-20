//! Redis-based caching layer for HAL9

use anyhow::{Result, anyhow};
use redis::{AsyncCommands, aio::ConnectionManager, RedisError, cmd, FromRedisValue, Client};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use std::ops::DerefMut;
use tracing::{info, warn, debug};
use uuid::Uuid;

/// Type alias for Redis connection pool
pub type RedisPool = bb8::Pool<RedisConnectionManager>;

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Redis URL
    pub url: String,
    
    /// Maximum connections in pool
    pub max_connections: u32,
    
    /// Minimum connections to maintain
    pub min_connections: u32,
    
    /// Connection timeout
    pub connection_timeout: Duration,
    
    /// Default TTL for cache entries
    pub default_ttl: Duration,
    
    /// Cache key prefix
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

/// Cache strategy for different operations
#[derive(Debug, Clone, Copy)]
pub enum CacheStrategy {
    /// Write-through: Write to cache and database simultaneously
    WriteThrough,
    
    /// Write-behind: Write to cache first, async to database
    WriteBehind {
        batch_size: usize,
        flush_interval: Duration,
    },
    
    /// Cache-aside: Read from cache, fallback to database
    CacheAside {
        ttl: Duration,
    },
}

/// Redis cache pool
pub struct CachePool {
    pool: bb8::Pool<RedisConnectionManager>,
    config: CacheConfig,
}

impl CachePool {
    /// Create new cache pool
    pub async fn new(config: CacheConfig) -> Result<Self> {
        info!("Connecting to Redis: {}", config.url);
        
        let manager = RedisConnectionManager::new(config.url.clone())?;
        
        let pool = bb8::Pool::builder()
            .max_size(config.max_connections)
            .min_idle(Some(config.min_connections))
            .connection_timeout(config.connection_timeout)
            .build(manager)
            .await?;
        
        Ok(Self { pool, config })
    }
    
    /// Get a connection from the pool
    pub async fn get_connection(&self) -> Result<bb8::PooledConnection<'_, RedisConnectionManager>> {
        self.pool.get().await
            .map_err(|e| anyhow!("Failed to get Redis connection: {}", e))
    }
    
    /// Build a cache key with prefix
    fn build_key(&self, key: &str) -> String {
        format!("{}:{}", self.config.key_prefix, key)
    }
    
    /// Set a value with TTL
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let key = self.build_key(key);
        let value = serde_json::to_string(value)?;
        let ttl = ttl.unwrap_or(self.config.default_ttl);
        
        // Use raw Redis command to work around bb8-redis compatibility issues
        let result: redis::RedisResult<()> = redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl.as_secs())
            .arg(value)
            .query_async(&mut *conn as &mut dyn redis::aio::ConnectionLike)
            .await;
        result.map_err(|e| anyhow!("Redis error: {}", e))?;
        debug!("Cache SET: {} (TTL: {}s)", key, ttl.as_secs());
        
        Ok(())
    }
    
    /// Get a value
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.get_connection().await?;
        let key = self.build_key(key);
        
        let value: Option<String> = cmd("GET")
            .arg(&key)
            .query_async(conn.deref_mut())
            .await?;
        
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
        let mut conn = self.get_connection().await?;
        let key = self.build_key(key);
        
        cmd("DEL")
            .arg(&key)
            .query_async(conn.deref_mut())
            .await?;
        debug!("Cache DELETE: {}", key);
        
        Ok(())
    }
    
    /// Delete multiple values by pattern
    pub async fn delete_pattern(&self, pattern: &str) -> Result<u64> {
        let mut conn = self.get_connection().await?;
        let pattern = self.build_key(pattern);
        
        // Use SCAN to find keys matching pattern
        let keys: Vec<String> = {
            let mut cursor = 0u64;
            let mut all_keys = Vec::new();
            loop {
                let (new_cursor, mut keys): (u64, Vec<String>) = cmd("SCAN")
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(&pattern)
                    .query_async(conn.deref_mut())
                    .await?;
                all_keys.append(&mut keys);
                cursor = new_cursor;
                if cursor == 0 {
                    break;
                }
            }
            all_keys
        };
        
        if keys.is_empty() {
            return Ok(0);
        }
        
        let count = keys.len() as u64;
        cmd("DEL")
            .arg(keys)
            .query_async(conn.deref_mut())
            .await?;
        debug!("Cache DELETE pattern: {} ({} keys)", pattern, count);
        
        Ok(count)
    }
    
    /// Check if key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let key = self.build_key(key);
        
        let exists: bool = cmd("EXISTS")
            .arg(&key)
            .query_async(conn.deref_mut())
            .await?;
        Ok(exists)
    }
    
    /// Increment a counter
    pub async fn incr(&self, key: &str) -> Result<i64> {
        let mut conn = self.get_connection().await?;
        let key = self.build_key(key);
        
        let value: i64 = cmd("INCR")
            .arg(&key)
            .query_async(conn.deref_mut())
            .await?;
        Ok(value)
    }
    
    /// Add to a set
    pub async fn sadd(&self, key: &str, member: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let key = self.build_key(key);
        
        cmd("SADD")
            .arg(&key)
            .arg(member)
            .query_async(conn.deref_mut())
            .await?;
        Ok(())
    }
    
    /// Get set members
    pub async fn smembers(&self, key: &str) -> Result<Vec<String>> {
        let mut conn = self.get_connection().await?;
        let key = self.build_key(key);
        
        let members: Vec<String> = cmd("SMEMBERS")
            .arg(&key)
            .query_async(conn.deref_mut())
            .await?;
        Ok(members)
    }
    
    /// Push to list
    pub async fn lpush(&self, key: &str, value: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let key = self.build_key(key);
        
        cmd("LPUSH")
            .arg(&key)
            .arg(value)
            .query_async(conn.deref_mut())
            .await?;
        Ok(())
    }
    
    /// Get list range
    pub async fn lrange(&self, key: &str, start: isize, stop: isize) -> Result<Vec<String>> {
        let mut conn = self.get_connection().await?;
        let key = self.build_key(key);
        
        let values: Vec<String> = cmd("LRANGE")
            .arg(&key)
            .arg(start)
            .arg(stop)
            .query_async(conn.deref_mut())
            .await?;
        Ok(values)
    }
    
    /// Set hash field
    pub async fn hset<T: Serialize>(&self, key: &str, field: &str, value: &T) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let key = self.build_key(key);
        let value = serde_json::to_string(value)?;
        
        cmd("HSET")
            .arg(&key)
            .arg(field)
            .arg(value)
            .query_async(conn.deref_mut())
            .await?;
        Ok(())
    }
    
    /// Get hash field
    pub async fn hget<T: for<'de> Deserialize<'de>>(&self, key: &str, field: &str) -> Result<Option<T>> {
        let mut conn = self.get_connection().await?;
        let key = self.build_key(key);
        
        let value: Option<String> = cmd("HGET")
            .arg(&key)
            .arg(field)
            .query_async(conn.deref_mut())
            .await?;
        
        match value {
            Some(v) => {
                let parsed = serde_json::from_str(&v)?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }
    
    /// Get all hash fields
    pub async fn hgetall(&self, key: &str) -> Result<std::collections::HashMap<String, String>> {
        let mut conn = self.get_connection().await?;
        let key = self.build_key(key);
        
        let values: std::collections::HashMap<String, String> = cmd("HGETALL")
            .arg(&key)
            .query_async(conn.deref_mut())
            .await?;
        Ok(values)
    }
    
    /// Publish message to channel
    pub async fn publish(&self, channel: &str, message: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        cmd("PUBLISH")
            .arg(channel)
            .arg(message)
            .query_async(conn.deref_mut())
            .await?;
        Ok(())
    }
    
    /// Get pool metrics
    pub fn metrics(&self) -> PoolMetrics {
        let state = self.pool.state();
        PoolMetrics {
            connections: state.connections,
            idle_connections: state.idle_connections,
        }
    }
}

/// Pool metrics
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    pub connections: u32,
    pub idle_connections: u32,
}

/// Cache key builders for different entities
pub struct CacheKeys;

impl CacheKeys {
    /// User cache key
    pub fn user(user_id: &Uuid) -> String {
        format!("user:{}", user_id)
    }
    
    /// User sessions key
    pub fn user_sessions(user_id: &Uuid) -> String {
        format!("user:{}:sessions", user_id)
    }
    
    /// API key cache key
    pub fn api_key(key_hash: &str) -> String {
        format!("api_key:{}", key_hash)
    }
    
    /// Neuron config key
    pub fn neuron(neuron_id: &str) -> String {
        format!("neuron:{}", neuron_id)
    }
    
    /// Neuron state key
    pub fn neuron_state(neuron_id: &str) -> String {
        format!("neuron:{}:state", neuron_id)
    }
    
    /// Neuron metrics key
    pub fn neuron_metrics(neuron_id: &str) -> String {
        format!("neuron:{}:metrics", neuron_id)
    }
    
    /// Signal key
    pub fn signal(signal_id: &Uuid) -> String {
        format!("signal:{}", signal_id)
    }
    
    /// Signal batch key
    pub fn signal_batch(batch_id: &Uuid) -> String {
        format!("signal:batch:{}", batch_id)
    }
    
    /// Memory search cache key
    pub fn memory_search(query_hash: &str) -> String {
        format!("memory:search:{}", query_hash)
    }
    
    /// Recent memories key
    pub fn recent_memories(neuron_id: &str) -> String {
        format!("memory:{}:recent", neuron_id)
    }
}

/// Cached data wrapper with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedData<T> {
    pub data: T,
    pub cached_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl<T> CachedData<T> {
    pub fn new(data: T, ttl: Duration) -> Self {
        let now = chrono::Utc::now();
        Self {
            data,
            cached_at: now,
            expires_at: now + chrono::Duration::from_std(ttl).unwrap(),
        }
    }
    
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }
}

/// Write-behind buffer for async database writes
pub struct WriteBehindBuffer {
    cache: CachePool,
    buffer_key: String,
}

impl WriteBehindBuffer {
    pub fn new(cache: CachePool, buffer_name: &str) -> Self {
        Self {
            cache,
            buffer_key: format!("buffer:{}", buffer_name),
        }
    }
    
    /// Add item to buffer
    pub async fn push<T: Serialize>(&self, item: &T) -> Result<()> {
        let value = serde_json::to_string(item)?;
        let mut conn = self.cache.get_connection().await?;
        cmd("LPUSH")
            .arg(&self.buffer_key)
            .arg(&value)
            .query_async(conn.deref_mut())
            .await?;
        Ok(())
    }
    
    /// Flush buffer and return items
    pub async fn flush<T: for<'de> Deserialize<'de>>(&self, batch_size: usize) -> Result<Vec<T>> {
        let mut conn = self.cache.get_connection().await?;
        let values: Vec<String> = cmd("LRANGE")
            .arg(&self.buffer_key)
            .arg(0)
            .arg(batch_size as isize - 1)
            .query_async(conn.deref_mut())
            .await?;
        
        if !values.is_empty() {
            // Remove flushed items
            cmd("LTRIM")
                .arg(&self.buffer_key)
                .arg(values.len() as isize)
                .arg(-1)
                .query_async(conn.deref_mut())
                .await?;
        }
        
        let items: Result<Vec<T>> = values.iter()
            .map(|v| serde_json::from_str(v).map_err(Into::into))
            .collect();
        
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_keys() {
        let user_id = Uuid::new_v4();
        assert!(CacheKeys::user(&user_id).starts_with("user:"));
        assert!(CacheKeys::neuron("test-neuron").starts_with("neuron:"));
    }
    
    #[tokio::test]
    async fn test_cached_data() {
        let data = CachedData::new("test", Duration::from_secs(60));
        assert!(!data.is_expired());
        assert_eq!(data.data, "test");
    }
}