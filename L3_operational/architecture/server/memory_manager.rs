//! Memory management for neurons

use std::sync::Arc;
use std::path::Path;
use tracing::{info, error};

use hal9_core::{
    Result, Error,
    memory::{MemoryStore, SqliteMemoryStore},
    config::{MemoryConfig, MemoryCleanupConfig},
};

/// Memory manager for initializing and managing neuron memory
pub struct MemoryManager {
    store: Arc<dyn MemoryStore>,
}

impl MemoryManager {
    /// Create a new memory manager
    pub async fn new(config: &MemoryConfig) -> Result<Self> {
        if !config.enabled {
            return Err(Error::Config("Memory system is disabled".to_string()));
        }
        
        info!("Initializing memory system at: {}", config.database_path);
        
        // Create SQLite memory store
        let store = SqliteMemoryStore::new(&config.database_path).await?;
        
        // Initialize the database schema
        store.initialize().await?;
        
        info!("Memory system initialized successfully");
        
        Ok(Self {
            store: Arc::new(store),
        })
    }
    
    /// Get the memory store for neurons to use
    pub fn get_store(&self) -> Arc<dyn MemoryStore> {
        self.store.clone()
    }
    
    /// Run cleanup based on configuration
    pub async fn cleanup(&self, config: &MemoryCleanupConfig) -> Result<u64> {
        let before = chrono::Utc::now() - chrono::Duration::days(config.retention_days as i64);
        let deleted = self.store.cleanup(before, config.min_importance).await?;
        
        if deleted > 0 {
            info!("Cleaned up {} old memory entries", deleted);
        }
        
        Ok(deleted)
    }
}

/// Background task for periodic memory cleanup
pub async fn cleanup_task(manager: Arc<MemoryManager>, config: MemoryCleanupConfig) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Run hourly
    
    loop {
        interval.tick().await;
        
        match manager.cleanup(&config).await {
            Ok(deleted) => {
                if deleted > 0 {
                    info!("Memory cleanup task deleted {} entries", deleted);
                }
            }
            Err(e) => {
                error!("Memory cleanup task failed: {}", e);
            }
        }
    }
}