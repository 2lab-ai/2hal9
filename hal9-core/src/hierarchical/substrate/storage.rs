//! Storage abstraction for persistent data

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use sqlx::{SqlitePool, PgPool, Row};
use crate::{Result, Error};

/// Persistent storage abstraction
#[async_trait]
pub trait PersistentStorage: Send + Sync + 'static {
    /// Store a value with a key
    async fn put<V>(&self, key: &str, value: V) -> Result<()>
    where
        V: Serialize + Send + Sync;
    
    /// Retrieve a value by key
    async fn get<V>(&self, key: &str) -> Result<Option<V>>
    where
        V: for<'de> Deserialize<'de>;
    
    /// Delete a value by key
    async fn delete(&self, key: &str) -> Result<()>;
    
    /// Check if a key exists
    async fn exists(&self, key: &str) -> Result<bool>;
    
    /// List keys with a prefix
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;
    
    /// Atomic compare-and-swap operation
    async fn compare_and_swap<V>(&self, key: &str, old: Option<V>, new: V) -> Result<bool>
    where
        V: Serialize + for<'de> Deserialize<'de> + PartialEq + Send + Sync;
    
    /// Set TTL for a key
    async fn set_ttl(&self, key: &str, ttl: Duration) -> Result<()>;
    
    /// Get storage metrics
    fn metrics(&self) -> StorageMetrics;
    
    /// Create a transaction
    async fn transaction(&self) -> Result<Box<dyn StorageTransaction>>;
}

/// Storage transaction for atomic operations
#[async_trait]
pub trait StorageTransaction: Send {
    /// Add a put operation to the transaction
    async fn put(&mut self, key: &str, value: Vec<u8>) -> Result<()>;
    
    /// Add a delete operation to the transaction
    async fn delete(&mut self, key: &str) -> Result<()>;
    
    /// Commit the transaction
    async fn commit(self: Box<Self>) -> Result<()>;
    
    /// Rollback the transaction
    async fn rollback(self: Box<Self>) -> Result<()>;
}

/// Storage performance metrics
#[derive(Debug, Clone)]
pub struct StorageMetrics {
    pub total_keys: u64,
    pub storage_bytes: u64,
    pub reads_per_sec: f64,
    pub writes_per_sec: f64,
    pub cache_hit_rate: f64,
    pub avg_read_latency_ms: f64,
    pub avg_write_latency_ms: f64,
}

impl Default for StorageMetrics {
    fn default() -> Self {
        Self {
            total_keys: 0,
            storage_bytes: 0,
            reads_per_sec: 0.0,
            writes_per_sec: 0.0,
            cache_hit_rate: 0.0,
            avg_read_latency_ms: 0.0,
            avg_write_latency_ms: 0.0,
        }
    }
}

/// Metrics tracker for storage operations
struct MetricsTracker {
    read_count: AtomicU64,
    write_count: AtomicU64,
    read_latency_sum: AtomicU64,
    write_latency_sum: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    last_update: parking_lot::Mutex<Instant>,
}

impl Default for MetricsTracker {
    fn default() -> Self {
        Self {
            read_count: AtomicU64::new(0),
            write_count: AtomicU64::new(0),
            read_latency_sum: AtomicU64::new(0),
            write_latency_sum: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            last_update: parking_lot::Mutex::new(Instant::now()),
        }
    }
}

impl MetricsTracker {
    fn record_read(&self, latency_ms: u64, cache_hit: bool) {
        self.read_count.fetch_add(1, Ordering::Relaxed);
        self.read_latency_sum.fetch_add(latency_ms, Ordering::Relaxed);
        
        if cache_hit {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    fn record_write(&self, latency_ms: u64) {
        self.write_count.fetch_add(1, Ordering::Relaxed);
        self.write_latency_sum.fetch_add(latency_ms, Ordering::Relaxed);
    }
    
    fn to_metrics(&self, total_keys: u64, storage_bytes: u64) -> StorageMetrics {
        let now = Instant::now();
        let elapsed = {
            let mut last = self.last_update.lock();
            let elapsed = now.duration_since(*last);
            *last = now;
            elapsed.as_secs_f64()
        };
        
        let read_count = self.read_count.load(Ordering::Relaxed);
        let write_count = self.write_count.load(Ordering::Relaxed);
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_total = cache_hits + self.cache_misses.load(Ordering::Relaxed);
        
        StorageMetrics {
            total_keys,
            storage_bytes,
            reads_per_sec: read_count as f64 / elapsed.max(1.0),
            writes_per_sec: write_count as f64 / elapsed.max(1.0),
            cache_hit_rate: if cache_total > 0 {
                cache_hits as f64 / cache_total as f64
            } else {
                0.0
            },
            avg_read_latency_ms: if read_count > 0 {
                self.read_latency_sum.load(Ordering::Relaxed) as f64 / read_count as f64
            } else {
                0.0
            },
            avg_write_latency_ms: if write_count > 0 {
                self.write_latency_sum.load(Ordering::Relaxed) as f64 / write_count as f64
            } else {
                0.0
            },
        }
    }
}

/// SQLite storage for local deployment
pub struct SqliteStorage {
    pool: Option<SqlitePool>,
    path: String,
    metrics: Arc<MetricsTracker>,
    cache: Arc<moka::future::Cache<String, Vec<u8>>>,
}

impl SqliteStorage {
    pub fn new(path: &str) -> Self {
        let cache = moka::future::Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(300))
            .build();
        
        Self {
            pool: None,
            path: path.to_string(),
            metrics: Arc::new(MetricsTracker::default()),
            cache: Arc::new(cache),
        }
    }
    
    pub async fn initialize(&mut self) -> Result<()> {
        let pool = SqlitePool::connect(&self.path).await
            .map_err(|e| Error::Storage(format!("Failed to connect to SQLite: {}", e)))?;
        
        // Create storage table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS kv_storage (
                key TEXT PRIMARY KEY,
                value BLOB NOT NULL,
                expires_at INTEGER,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );
            
            CREATE INDEX IF NOT EXISTS idx_expires_at ON kv_storage(expires_at)
            WHERE expires_at IS NOT NULL;
            "#
        )
        .execute(&pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to create tables: {}", e)))?;
        
        self.pool = Some(pool);
        Ok(())
    }
    
    fn pool(&self) -> Result<&SqlitePool> {
        self.pool.as_ref()
            .ok_or_else(|| Error::Storage("Storage not initialized".to_string()))
    }
}

#[async_trait]
impl PersistentStorage for SqliteStorage {
    async fn put<V>(&self, key: &str, value: V) -> Result<()>
    where
        V: Serialize + Send + Sync,
    {
        let start = Instant::now();
        let pool = self.pool()?;
        
        let data = bincode::serialize(&value)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        
        sqlx::query(
            r#"
            INSERT INTO kv_storage (key, value, updated_at)
            VALUES (?1, ?2, strftime('%s', 'now'))
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at;
            "#
        )
        .bind(key)
        .bind(&data)
        .execute(pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to put: {}", e)))?;
        
        // Update cache
        self.cache.insert(key.to_string(), data).await;
        
        let latency = start.elapsed().as_millis() as u64;
        self.metrics.record_write(latency);
        
        Ok(())
    }
    
    async fn get<V>(&self, key: &str) -> Result<Option<V>>
    where
        V: for<'de> Deserialize<'de>,
    {
        let start = Instant::now();
        
        // Check cache first
        if let Some(cached) = self.cache.get(key).await {
            let value = bincode::deserialize(&cached)
                .map_err(|e| Error::Deserialization(e.to_string()))?;
            
            let latency = start.elapsed().as_millis() as u64;
            self.metrics.record_read(latency, true);
            
            return Ok(Some(value));
        }
        
        let pool = self.pool()?;
        
        let row = sqlx::query(
            r#"
            SELECT value FROM kv_storage
            WHERE key = ?1
            AND (expires_at IS NULL OR expires_at > strftime('%s', 'now'));
            "#
        )
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to get: {}", e)))?;
        
        let latency = start.elapsed().as_millis() as u64;
        self.metrics.record_read(latency, false);
        
        match row {
            Some(row) => {
                let data: Vec<u8> = row.get(0);
                
                // Update cache
                self.cache.insert(key.to_string(), data.clone()).await;
                
                let value = bincode::deserialize(&data)
                    .map_err(|e| Error::Deserialization(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
    
    async fn delete(&self, key: &str) -> Result<()> {
        let pool = self.pool()?;
        
        sqlx::query("DELETE FROM kv_storage WHERE key = ?1")
            .bind(key)
            .execute(pool)
            .await
            .map_err(|e| Error::Storage(format!("Failed to delete: {}", e)))?;
        
        // Remove from cache
        self.cache.invalidate(key).await;
        
        Ok(())
    }
    
    async fn exists(&self, key: &str) -> Result<bool> {
        // Check cache first
        if self.cache.contains_key(key) {
            return Ok(true);
        }
        
        let pool = self.pool()?;
        
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM kv_storage
            WHERE key = ?1
            AND (expires_at IS NULL OR expires_at > strftime('%s', 'now'));
            "#
        )
        .bind(key)
        .fetch_one(pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to check exists: {}", e)))?;
        
        Ok(count > 0)
    }
    
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let pool = self.pool()?;
        
        let pattern = format!("{}%", prefix);
        let keys: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT key FROM kv_storage
            WHERE key LIKE ?1
            AND (expires_at IS NULL OR expires_at > strftime('%s', 'now'))
            ORDER BY key;
            "#
        )
        .bind(pattern)
        .fetch_all(pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to list keys: {}", e)))?;
        
        Ok(keys)
    }
    
    async fn compare_and_swap<V>(&self, key: &str, old: Option<V>, new: V) -> Result<bool>
    where
        V: Serialize + for<'de> Deserialize<'de> + PartialEq + Send + Sync,
    {
        let pool = self.pool()?;
        let mut tx = pool.begin().await
            .map_err(|e| Error::Storage(format!("Failed to begin transaction: {}", e)))?;
        
        // Get current value
        let current = sqlx::query(
            r#"
            SELECT value FROM kv_storage
            WHERE key = ?1
            AND (expires_at IS NULL OR expires_at > strftime('%s', 'now'))
            FOR UPDATE;
            "#
        )
        .bind(key)
        .fetch_optional(tx.as_mut())
        .await
        .map_err(|e| Error::Storage(format!("Failed to get for CAS: {}", e)))?;
        
        let current_value = match current {
            Some(row) => {
                let data: Vec<u8> = row.get(0);
                Some(bincode::deserialize(&data)
                    .map_err(|e| Error::Deserialization(e.to_string()))?)
            }
            None => None,
        };
        
        // Check if old value matches
        if current_value != old {
            tx.rollback().await
                .map_err(|e| Error::Storage(format!("Failed to rollback: {}", e)))?;
            return Ok(false);
        }
        
        // Update with new value
        let new_data = bincode::serialize(&new)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        
        sqlx::query(
            r#"
            INSERT INTO kv_storage (key, value, updated_at)
            VALUES (?1, ?2, strftime('%s', 'now'))
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at;
            "#
        )
        .bind(key)
        .bind(&new_data)
        .execute(tx.as_mut())
        .await
        .map_err(|e| Error::Storage(format!("Failed to update in CAS: {}", e)))?;
        
        tx.commit().await
            .map_err(|e| Error::Storage(format!("Failed to commit: {}", e)))?;
        
        // Update cache
        self.cache.insert(key.to_string(), new_data).await;
        
        Ok(true)
    }
    
    async fn set_ttl(&self, key: &str, ttl: Duration) -> Result<()> {
        let pool = self.pool()?;
        let expires_at = chrono::Utc::now().timestamp() + ttl.as_secs() as i64;
        
        sqlx::query(
            r#"
            UPDATE kv_storage
            SET expires_at = ?2
            WHERE key = ?1;
            "#
        )
        .bind(key)
        .bind(expires_at)
        .execute(pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to set TTL: {}", e)))?;
        
        Ok(())
    }
    
    fn metrics(&self) -> StorageMetrics {
        // Would need to query DB for accurate counts
        self.metrics.to_metrics(0, 0)
    }
    
    async fn transaction(&self) -> Result<Box<dyn StorageTransaction>> {
        let pool = self.pool()?;
        let tx = pool.begin().await
            .map_err(|e| Error::Storage(format!("Failed to begin transaction: {}", e)))?;
        
        Ok(Box::new(SqliteTransaction {
            tx: Some(tx),
            operations: Vec::new(),
        }))
    }
}

/// SQLite transaction implementation
struct SqliteTransaction {
    tx: Option<sqlx::Transaction<'static, sqlx::Sqlite>>,
    operations: Vec<TransactionOp>,
}

enum TransactionOp {
    Put { key: String, value: Vec<u8> },
    Delete { key: String },
}

#[async_trait]
impl StorageTransaction for SqliteTransaction {
    async fn put(&mut self, key: &str, value: Vec<u8>) -> Result<()> {
        self.operations.push(TransactionOp::Put {
            key: key.to_string(),
            value,
        });
        
        Ok(())
    }
    
    async fn delete(&mut self, key: &str) -> Result<()> {
        self.operations.push(TransactionOp::Delete {
            key: key.to_string(),
        });
        Ok(())
    }
    
    async fn commit(mut self: Box<Self>) -> Result<()> {
        let mut tx = self.tx.take()
            .ok_or_else(|| Error::Storage("Transaction already consumed".to_string()))?;
        
        for op in self.operations {
            match op {
                TransactionOp::Put { key, value } => {
                    sqlx::query(
                        r#"
                        INSERT INTO kv_storage (key, value, updated_at)
                        VALUES (?1, ?2, strftime('%s', 'now'))
                        ON CONFLICT(key) DO UPDATE SET
                            value = excluded.value,
                            updated_at = excluded.updated_at;
                        "#
                    )
                    .bind(key)
                    .bind(value)
                    .execute(tx.as_mut())
                    .await
                    .map_err(|e| Error::Storage(format!("Failed to put in transaction: {}", e)))?;
                }
                TransactionOp::Delete { key } => {
                    sqlx::query("DELETE FROM kv_storage WHERE key = ?1")
                        .bind(key)
                        .execute(tx.as_mut())
                        .await
                        .map_err(|e| Error::Storage(format!("Failed to delete in transaction: {}", e)))?;
                }
            }
        }
        
        tx.commit().await
            .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;
        
        Ok(())
    }
    
    async fn rollback(mut self: Box<Self>) -> Result<()> {
        if let Some(tx) = self.tx.take() {
            tx.rollback().await
                .map_err(|e| Error::Storage(format!("Failed to rollback transaction: {}", e)))?;
        }
        Ok(())
    }
}

/// PostgreSQL storage for distributed deployment
pub struct PostgresStorage {
    pool: Option<PgPool>,
    connection_string: String,
    metrics: Arc<MetricsTracker>,
    cache: Arc<moka::future::Cache<String, Vec<u8>>>,
}

impl PostgresStorage {
    pub fn new(connection_string: &str) -> Self {
        let cache = moka::future::Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(300))
            .build();
        
        Self {
            pool: None,
            connection_string: connection_string.to_string(),
            metrics: Arc::new(MetricsTracker::default()),
            cache: Arc::new(cache),
        }
    }
    
    pub async fn initialize(&mut self) -> Result<()> {
        let pool = PgPool::connect(&self.connection_string).await
            .map_err(|e| Error::Storage(format!("Failed to connect to PostgreSQL: {}", e)))?;
        
        // Create storage table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS kv_storage (
                key TEXT PRIMARY KEY,
                value BYTEA NOT NULL,
                expires_at TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_expires_at ON kv_storage(expires_at)
            WHERE expires_at IS NOT NULL;
            "#
        )
        .execute(&pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to create tables: {}", e)))?;
        
        self.pool = Some(pool);
        Ok(())
    }
    
    fn pool(&self) -> Result<&PgPool> {
        self.pool.as_ref()
            .ok_or_else(|| Error::Storage("Storage not initialized".to_string()))
    }
}

#[async_trait]
impl PersistentStorage for PostgresStorage {
    async fn put<V>(&self, key: &str, value: V) -> Result<()>
    where
        V: Serialize + Send + Sync,
    {
        let start = Instant::now();
        let pool = self.pool()?;
        
        let data = bincode::serialize(&value)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        
        sqlx::query(
            r#"
            INSERT INTO kv_storage (key, value, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT(key) DO UPDATE SET
                value = EXCLUDED.value,
                updated_at = EXCLUDED.updated_at;
            "#
        )
        .bind(key)
        .bind(&data)
        .execute(pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to put: {}", e)))?;
        
        // Update cache
        self.cache.insert(key.to_string(), data).await;
        
        let latency = start.elapsed().as_millis() as u64;
        self.metrics.record_write(latency);
        
        Ok(())
    }
    
    async fn get<V>(&self, key: &str) -> Result<Option<V>>
    where
        V: for<'de> Deserialize<'de>,
    {
        let start = Instant::now();
        
        // Check cache first
        if let Some(cached) = self.cache.get(key).await {
            let value = bincode::deserialize(&cached)
                .map_err(|e| Error::Deserialization(e.to_string()))?;
            
            let latency = start.elapsed().as_millis() as u64;
            self.metrics.record_read(latency, true);
            
            return Ok(Some(value));
        }
        
        let pool = self.pool()?;
        
        let row = sqlx::query(
            r#"
            SELECT value FROM kv_storage
            WHERE key = $1
            AND (expires_at IS NULL OR expires_at > NOW());
            "#
        )
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to get: {}", e)))?;
        
        let latency = start.elapsed().as_millis() as u64;
        self.metrics.record_read(latency, false);
        
        match row {
            Some(row) => {
                let data: Vec<u8> = row.get(0);
                
                // Update cache
                self.cache.insert(key.to_string(), data.clone()).await;
                
                let value = bincode::deserialize(&data)
                    .map_err(|e| Error::Deserialization(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
    
    async fn delete(&self, key: &str) -> Result<()> {
        let pool = self.pool()?;
        
        sqlx::query("DELETE FROM kv_storage WHERE key = $1")
            .bind(key)
            .execute(pool)
            .await
            .map_err(|e| Error::Storage(format!("Failed to delete: {}", e)))?;
        
        // Remove from cache
        self.cache.invalidate(key).await;
        
        Ok(())
    }
    
    async fn exists(&self, key: &str) -> Result<bool> {
        // Check cache first
        if self.cache.contains_key(key) {
            return Ok(true);
        }
        
        let pool = self.pool()?;
        
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM kv_storage
                WHERE key = $1
                AND (expires_at IS NULL OR expires_at > NOW())
            );
            "#
        )
        .bind(key)
        .fetch_one(pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to check exists: {}", e)))?;
        
        Ok(exists)
    }
    
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let pool = self.pool()?;
        
        let pattern = format!("{}%", prefix);
        let keys: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT key FROM kv_storage
            WHERE key LIKE $1
            AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY key;
            "#
        )
        .bind(pattern)
        .fetch_all(pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to list keys: {}", e)))?;
        
        Ok(keys)
    }
    
    async fn compare_and_swap<V>(&self, key: &str, old: Option<V>, new: V) -> Result<bool>
    where
        V: Serialize + for<'de> Deserialize<'de> + PartialEq + Send + Sync,
    {
        let pool = self.pool()?;
        let mut tx = pool.begin().await
            .map_err(|e| Error::Storage(format!("Failed to begin transaction: {}", e)))?;
        
        // Get current value with row lock
        let current = sqlx::query(
            r#"
            SELECT value FROM kv_storage
            WHERE key = $1
            AND (expires_at IS NULL OR expires_at > NOW())
            FOR UPDATE;
            "#
        )
        .bind(key)
        .fetch_optional(tx.as_mut())
        .await
        .map_err(|e| Error::Storage(format!("Failed to get for CAS: {}", e)))?;
        
        let current_value = match current {
            Some(row) => {
                let data: Vec<u8> = row.get(0);
                Some(bincode::deserialize(&data)
                    .map_err(|e| Error::Deserialization(e.to_string()))?)
            }
            None => None,
        };
        
        // Check if old value matches
        if current_value != old {
            tx.rollback().await
                .map_err(|e| Error::Storage(format!("Failed to rollback: {}", e)))?;
            return Ok(false);
        }
        
        // Update with new value
        let new_data = bincode::serialize(&new)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        
        sqlx::query(
            r#"
            INSERT INTO kv_storage (key, value, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT(key) DO UPDATE SET
                value = EXCLUDED.value,
                updated_at = EXCLUDED.updated_at;
            "#
        )
        .bind(key)
        .bind(&new_data)
        .execute(tx.as_mut())
        .await
        .map_err(|e| Error::Storage(format!("Failed to update in CAS: {}", e)))?;
        
        tx.commit().await
            .map_err(|e| Error::Storage(format!("Failed to commit: {}", e)))?;
        
        // Update cache
        self.cache.insert(key.to_string(), new_data).await;
        
        Ok(true)
    }
    
    async fn set_ttl(&self, key: &str, ttl: Duration) -> Result<()> {
        let pool = self.pool()?;
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(ttl.as_secs() as i64);
        
        sqlx::query(
            r#"
            UPDATE kv_storage
            SET expires_at = $2
            WHERE key = $1;
            "#
        )
        .bind(key)
        .bind(expires_at)
        .execute(pool)
        .await
        .map_err(|e| Error::Storage(format!("Failed to set TTL: {}", e)))?;
        
        Ok(())
    }
    
    fn metrics(&self) -> StorageMetrics {
        // Would need to query DB for accurate counts
        self.metrics.to_metrics(0, 0)
    }
    
    async fn transaction(&self) -> Result<Box<dyn StorageTransaction>> {
        let pool = self.pool()?;
        let tx = pool.begin().await
            .map_err(|e| Error::Storage(format!("Failed to begin transaction: {}", e)))?;
        
        Ok(Box::new(PostgresTransaction {
            tx: Some(tx),
            operations: Vec::new(),
        }))
    }
}

/// PostgreSQL transaction implementation
struct PostgresTransaction {
    tx: Option<sqlx::Transaction<'static, sqlx::Postgres>>,
    operations: Vec<TransactionOp>,
}

#[async_trait]
impl StorageTransaction for PostgresTransaction {
    async fn put(&mut self, key: &str, value: Vec<u8>) -> Result<()> {
        self.operations.push(TransactionOp::Put {
            key: key.to_string(),
            value,
        });
        
        Ok(())
    }
    
    async fn delete(&mut self, key: &str) -> Result<()> {
        self.operations.push(TransactionOp::Delete {
            key: key.to_string(),
        });
        Ok(())
    }
    
    async fn commit(mut self: Box<Self>) -> Result<()> {
        let mut tx = self.tx.take()
            .ok_or_else(|| Error::Storage("Transaction already consumed".to_string()))?;
        
        for op in self.operations {
            match op {
                TransactionOp::Put { key, value } => {
                    sqlx::query(
                        r#"
                        INSERT INTO kv_storage (key, value, updated_at)
                        VALUES ($1, $2, NOW())
                        ON CONFLICT(key) DO UPDATE SET
                            value = EXCLUDED.value,
                            updated_at = EXCLUDED.updated_at;
                        "#
                    )
                    .bind(key)
                    .bind(value)
                    .execute(tx.as_mut())
                    .await
                    .map_err(|e| Error::Storage(format!("Failed to put in transaction: {}", e)))?;
                }
                TransactionOp::Delete { key } => {
                    sqlx::query("DELETE FROM kv_storage WHERE key = $1")
                        .bind(key)
                        .execute(tx.as_mut())
                        .await
                        .map_err(|e| Error::Storage(format!("Failed to delete in transaction: {}", e)))?;
                }
            }
        }
        
        tx.commit().await
            .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;
        
        Ok(())
    }
    
    async fn rollback(mut self: Box<Self>) -> Result<()> {
        if let Some(tx) = self.tx.take() {
            tx.rollback().await
                .map_err(|e| Error::Storage(format!("Failed to rollback transaction: {}", e)))?;
        }
        Ok(())
    }
}

/// S3 storage for cloud deployment
pub struct S3Storage {
    bucket: String,
    // Would use AWS SDK
    metrics: std::sync::Arc<parking_lot::Mutex<StorageMetrics>>,
}

impl S3Storage {
    pub fn new(bucket: &str) -> Self {
        Self {
            bucket: bucket.to_string(),
            metrics: std::sync::Arc::new(parking_lot::Mutex::new(StorageMetrics::default())),
        }
    }
}

/// Storage key builder for hierarchical organization
pub struct StorageKey {
    parts: Vec<String>,
}

impl StorageKey {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }
    
    pub fn layer(mut self, layer: &str) -> Self {
        self.parts.push(format!("layer:{}", layer));
        self
    }
    
    pub fn neuron(mut self, neuron_id: &str) -> Self {
        self.parts.push(format!("neuron:{}", neuron_id));
        self
    }
    
    pub fn data_type(mut self, dtype: &str) -> Self {
        self.parts.push(format!("type:{}", dtype));
        self
    }
    
    pub fn id(mut self, id: &str) -> Self {
        self.parts.push(id.to_string());
        self
    }
    
    pub fn build(&self) -> String {
        self.parts.join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sqlite_storage_basic() {
        let mut storage = SqliteStorage::new(":memory:");
        storage.initialize().await.unwrap();
        
        // Test put and get
        storage.put("test-key", "test-value".to_string()).await.unwrap();
        let value: String = storage.get("test-key").await.unwrap().unwrap();
        assert_eq!(value, "test-value");
        
        // Test exists
        assert!(storage.exists("test-key").await.unwrap());
        assert!(!storage.exists("non-existent").await.unwrap());
        
        // Test delete
        storage.delete("test-key").await.unwrap();
        assert!(!storage.exists("test-key").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_storage_key_builder() {
        let key = StorageKey::new()
            .layer("L4")
            .neuron("planning-neuron")
            .data_type("state")
            .id("12345")
            .build();
        
        assert_eq!(key, "layer:L4/neuron:planning-neuron/type:state/12345");
    }
    
    #[tokio::test]
    async fn test_compare_and_swap() {
        let mut storage = SqliteStorage::new(":memory:");
        storage.initialize().await.unwrap();
        
        // Initial value
        storage.put("counter", 0i32).await.unwrap();
        
        // Successful CAS
        let success = storage.compare_and_swap("counter", Some(0i32), 1i32).await.unwrap();
        assert!(success);
        
        // Failed CAS (wrong old value)
        let success = storage.compare_and_swap("counter", Some(0i32), 2i32).await.unwrap();
        assert!(!success);
        
        // Verify value is still 1
        let value: i32 = storage.get("counter").await.unwrap().unwrap();
        assert_eq!(value, 1);
    }
    
    #[tokio::test]
    async fn test_transaction() {
        let mut storage = SqliteStorage::new(":memory:");
        storage.initialize().await.unwrap();
        
        // Create transaction
        let mut tx = storage.transaction().await.unwrap();
        
        // Add operations
        tx.put("key1", "value1".to_string()).await.unwrap();
        tx.put("key2", "value2".to_string()).await.unwrap();
        
        // Commit
        tx.commit().await.unwrap();
        
        // Verify both keys exist
        assert!(storage.exists("key1").await.unwrap());
        assert!(storage.exists("key2").await.unwrap());
    }
}