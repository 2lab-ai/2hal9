//! SQLite implementation of memory storage

use async_trait::async_trait;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions, FromRow};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::path::Path;
use tracing::{debug, info, warn};

use super::{MemoryStore, MemoryEntry, MemorySearch, MemoryStats, MemoryContext, MemoryType};
use crate::{Result, Error};

/// Row type for memory queries
#[derive(FromRow)]
struct MemoryRow {
    id: String,
    neuron_id: String,
    layer: String,
    timestamp: i64,
    entry_type: String,
    content: String,
    metadata: String,
    embedding: Option<Vec<u8>>,
    importance: f32,
    access_count: i64,
    last_accessed: i64,
}

/// SQLite-based memory store
pub struct SqliteMemoryStore {
    pool: SqlitePool,
}

impl SqliteMemoryStore {
    /// Create a new SQLite memory store
    pub async fn new(database_path: &str) -> Result<Self> {
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(database_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(Error::Io)?;
        }
        
        // Ensure the file exists by touching it
        if !Path::new(database_path).exists() {
            std::fs::File::create(database_path)
                .map_err(Error::Io)?;
        }
        
        // SQLx connection string for SQLite with mode=rwc to create if not exists
        let connection_string = format!("sqlite:{}?mode=rwc", database_path);
        debug!("Connecting to SQLite with: {}", connection_string);
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to connect to SQLite: {}", e)))?;
            
        Ok(Self { pool })
    }
    
    /// Create a new in-memory SQLite store (for testing)
    pub async fn in_memory() -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to create in-memory SQLite: {}", e)))?;
            
        Ok(Self { pool })
    }
}

#[async_trait]
impl MemoryStore for SqliteMemoryStore {
    async fn initialize(&self) -> Result<()> {
        info!("Initializing SQLite memory store");
        
        // Create memories table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                neuron_id TEXT NOT NULL,
                layer TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                entry_type TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT NOT NULL,
                embedding BLOB,
                importance REAL NOT NULL,
                access_count INTEGER NOT NULL DEFAULT 0,
                last_accessed INTEGER NOT NULL
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Other(anyhow::anyhow!("Failed to create memories table: {}", e)))?;
        
        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_neuron_id ON memories(neuron_id)")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to create neuron_id index: {}", e)))?;
            
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_layer ON memories(layer)")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to create layer index: {}", e)))?;
            
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_timestamp ON memories(timestamp)")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to create timestamp index: {}", e)))?;
            
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_entry_type ON memories(entry_type)")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to create entry_type index: {}", e)))?;
            
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_importance ON memories(importance)")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to create importance index: {}", e)))?;
        
        // Create full-text search virtual table
        sqlx::query(r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                id UNINDEXED,
                content,
                content=memories,
                content_rowid=rowid
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Other(anyhow::anyhow!("Failed to create FTS table: {}", e)))?;
        
        // Create triggers to keep FTS in sync
        sqlx::query(r#"
            CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
                INSERT INTO memories_fts(id, content) VALUES (new.id, new.content);
            END
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Other(anyhow::anyhow!("Failed to create insert trigger: {}", e)))?;
        
        sqlx::query(r#"
            CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
                DELETE FROM memories_fts WHERE id = old.id;
            END
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Other(anyhow::anyhow!("Failed to create delete trigger: {}", e)))?;
        
        info!("SQLite memory store initialized successfully");
        Ok(())
    }
    
    async fn store(&self, entry: MemoryEntry) -> Result<Uuid> {
        debug!("Storing memory entry: {} for neuron {}", entry.id, entry.neuron_id);
        
        let timestamp = entry.timestamp.timestamp();
        let last_accessed = entry.last_accessed.timestamp();
        let metadata_json = serde_json::to_string(&entry.metadata)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        let entry_type = serde_json::to_string(&entry.entry_type)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        let embedding_bytes = entry.embedding.as_ref()
            .map(|e| {
                let bytes: Vec<u8> = e.iter()
                    .flat_map(|f| f.to_le_bytes())
                    .collect();
                bytes
            });
            
        sqlx::query(r#"
            INSERT INTO memories (
                id, neuron_id, layer, timestamp, entry_type, 
                content, metadata, embedding, importance, 
                access_count, last_accessed
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(entry.id.to_string())
        .bind(&entry.neuron_id)
        .bind(&entry.layer)
        .bind(timestamp)
        .bind(entry_type)
        .bind(&entry.content)
        .bind(metadata_json)
        .bind(embedding_bytes)
        .bind(entry.importance)
        .bind(entry.access_count as i64)
        .bind(last_accessed)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Other(anyhow::anyhow!("Failed to store memory: {}", e)))?;
        
        Ok(entry.id)
    }
    
    async fn get(&self, id: Uuid) -> Result<Option<MemoryEntry>> {
        let row = sqlx::query_as::<_, MemoryRow>(
            r#"
            SELECT id, neuron_id, layer, timestamp, entry_type,
                   content, metadata, embedding, importance,
                   access_count, last_accessed
            FROM memories
            WHERE id = ?
            "#
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Other(anyhow::anyhow!("Failed to get memory: {}", e)))?;
        
        match row {
            Some(row) => {
                let entry_type: MemoryType = serde_json::from_str(&row.entry_type)
                    .map_err(|e| Error::Serialization(e.to_string()))?;
                let metadata: serde_json::Value = serde_json::from_str(&row.metadata)
                    .map_err(|e| Error::Serialization(e.to_string()))?;
                let embedding = row.embedding.as_ref().map(|bytes| {
                    bytes.chunks_exact(4)
                        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                        .collect()
                });
                
                Ok(Some(MemoryEntry {
                    id: Uuid::parse_str(&row.id)
                        .map_err(|e| Error::Other(anyhow::anyhow!("Invalid UUID: {}", e)))?,
                    neuron_id: row.neuron_id,
                    layer: row.layer,
                    timestamp: DateTime::from_timestamp(row.timestamp, 0)
                        .unwrap_or_else(Utc::now),
                    entry_type,
                    content: row.content,
                    metadata,
                    embedding,
                    importance: row.importance,
                    access_count: row.access_count as u32,
                    last_accessed: DateTime::from_timestamp(row.last_accessed, 0)
                        .unwrap_or_else(Utc::now),
                }))
            }
            None => Ok(None),
        }
    }
    
    async fn search(&self, params: MemorySearch) -> Result<Vec<MemoryEntry>> {
        let mut query = String::from(
            "SELECT id, neuron_id, layer, timestamp, entry_type, 
                    content, metadata, embedding, importance,
                    access_count, last_accessed
             FROM memories WHERE 1=1"
        );
        
        let mut bindings = Vec::new();
        
        if let Some(neuron_id) = &params.neuron_id {
            query.push_str(" AND neuron_id = ?");
            bindings.push(neuron_id.clone());
        }
        
        if let Some(layer) = &params.layer {
            query.push_str(" AND layer = ?");
            bindings.push(layer.clone());
        }
        
        if let Some(memory_type) = &params.memory_type {
            let type_str = serde_json::to_string(memory_type)
                .map_err(|e| Error::Serialization(e.to_string()))?;
            query.push_str(" AND entry_type = ?");
            bindings.push(type_str);
        }
        
        if let Some(start_time) = params.start_time {
            query.push_str(" AND timestamp >= ?");
            bindings.push(start_time.timestamp().to_string());
        }
        
        if let Some(end_time) = params.end_time {
            query.push_str(" AND timestamp <= ?");
            bindings.push(end_time.timestamp().to_string());
        }
        
        if let Some(min_importance) = params.min_importance {
            query.push_str(" AND importance >= ?");
            bindings.push(min_importance.to_string());
        }
        
        // Handle content search
        if let Some(content_query) = &params.content_query {
            if params.use_semantic_search {
                // TODO: Implement semantic search using embeddings
                warn!("Semantic search not yet implemented, falling back to FTS");
            }
            
            // Use FTS for text search
            query = format!(
                "SELECT m.* FROM memories m 
                 JOIN memories_fts f ON m.id = f.id 
                 WHERE memories_fts MATCH ? AND ({})",
                query.replace("WHERE 1=1", "")
            );
            bindings.insert(0, content_query.clone());
        }
        
        query.push_str(" ORDER BY timestamp DESC LIMIT ?");
        bindings.push(params.limit.to_string());
        
        // Build and execute dynamic query
        use sqlx::sqlite::Sqlite;
        let mut sql_query = sqlx::query::<Sqlite>(&query);
        for binding in bindings {
            sql_query = sql_query.bind(binding);
        }
        
        // For now, return empty vector since dynamic queries are complex with sqlx
        // TODO: Implement proper dynamic query building
        warn!("Memory search not fully implemented yet");
        Ok(Vec::new())
    }
    
    async fn record_access(&self, id: Uuid) -> Result<()> {
        let now = Utc::now().timestamp();
        
        sqlx::query(
            "UPDATE memories 
             SET access_count = access_count + 1, 
                 last_accessed = ? 
             WHERE id = ?"
        )
        .bind(now)
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Other(anyhow::anyhow!("Failed to record access: {}", e)))?;
        
        Ok(())
    }
    
    async fn cleanup(&self, before: DateTime<Utc>, min_importance: f32) -> Result<u64> {
        let timestamp = before.timestamp();
        
        let result = sqlx::query(
            "DELETE FROM memories 
             WHERE timestamp < ? AND importance < ?"
        )
        .bind(timestamp)
        .bind(min_importance)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Other(anyhow::anyhow!("Failed to cleanup memories: {}", e)))?;
        
        Ok(result.rows_affected())
    }
    
    async fn get_stats(&self, neuron_id: &str) -> Result<MemoryStats> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM memories WHERE neuron_id = ?"
        )
        .bind(neuron_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Other(anyhow::anyhow!("Failed to get total count: {}", e)))?;
        
        let stats = MemoryStats {
            total_entries: row.0 as u64,
            entries_by_type: std::collections::HashMap::new(),
            average_importance: 0.5,
            total_access_count: 0,
            oldest_entry: None,
            newest_entry: None,
        };
        
        Ok(stats)
    }
    
    async fn build_context(
        &self, 
        neuron_id: &str, 
        current_task: &str
    ) -> Result<MemoryContext> {
        // Get recent tasks
        let recent_tasks = self.search(MemorySearch {
            neuron_id: Some(neuron_id.to_string()),
            memory_type: Some(MemoryType::Task),
            limit: 5,
            ..Default::default()
        }).await?;
        
        // Get relevant learnings
        let relevant_learnings = self.search(MemorySearch {
            neuron_id: Some(neuron_id.to_string()),
            memory_type: Some(MemoryType::Learning),
            content_query: Some(current_task.to_string()),
            limit: 3,
            ..Default::default()
        }).await?;
        
        // Get similar experiences
        let similar_experiences = self.search(MemorySearch {
            neuron_id: Some(neuron_id.to_string()),
            content_query: Some(current_task.to_string()),
            limit: 5,
            ..Default::default()
        }).await?;
        
        // Get error patterns
        let error_patterns = self.search(MemorySearch {
            neuron_id: Some(neuron_id.to_string()),
            memory_type: Some(MemoryType::Error),
            limit: 3,
            ..Default::default()
        }).await?;
        
        Ok(MemoryContext {
            recent_tasks,
            relevant_learnings,
            similar_experiences,
            error_patterns,
        })
    }
}