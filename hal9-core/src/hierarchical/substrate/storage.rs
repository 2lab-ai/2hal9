//! Storage abstraction for persistent data

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::time::Duration;
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
    async fn put<V>(&mut self, key: &str, value: V) -> Result<()>
    where
        V: Serialize + Send + Sync;
    
    /// Add a delete operation to the transaction
    async fn delete(&mut self, key: &str) -> Result<()>;
    
    /// Commit the transaction
    async fn commit(self: Box<Self>) -> Result<()>;
    
    /// Rollback the transaction
    async fn rollback(self: Box<Self>) -> Result<()>;
}

/// Storage performance metrics
#[derive(Debug, Clone, Default)]
pub struct StorageMetrics {
    pub total_keys: u64,
    pub storage_bytes: u64,
    pub reads_per_sec: f64,
    pub writes_per_sec: f64,
    pub cache_hit_rate: f64,
}

/// SQLite storage for local deployment
pub struct SqliteStorage {
    pool: Option<sqlx::SqlitePool>,
    metrics: std::sync::Arc<parking_lot::Mutex<StorageMetrics>>,
}

impl SqliteStorage {
    pub fn new(path: &str) -> Self {
        Self {
            pool: None, // Would be initialized in initialize()
            metrics: std::sync::Arc::new(parking_lot::Mutex::new(StorageMetrics::default())),
        }
    }
}

/// PostgreSQL storage for distributed deployment
pub struct PostgresStorage {
    pool: Option<sqlx::PgPool>,
    metrics: std::sync::Arc<parking_lot::Mutex<StorageMetrics>>,
}

impl PostgresStorage {
    pub fn new(connection_string: &str) -> Self {
        Self {
            pool: None, // Would be initialized in initialize()
            metrics: std::sync::Arc::new(parking_lot::Mutex::new(StorageMetrics::default())),
        }
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