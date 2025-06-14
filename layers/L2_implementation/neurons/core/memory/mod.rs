//! Persistent memory system for neurons using SQLite

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod sqlite;
pub mod embeddings;

pub use sqlite::SqliteMemoryStore;
pub use embeddings::EmbeddingGenerator;

/// Memory entry for a neuron
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: Uuid,
    pub neuron_id: String,
    pub layer: String,
    pub timestamp: DateTime<Utc>,
    pub entry_type: MemoryType,
    pub content: String,
    pub metadata: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub importance: f32,
    pub access_count: u32,
    pub last_accessed: DateTime<Utc>,
}

/// Type of memory entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryType {
    /// Task processed by the neuron
    Task,
    /// Result of processing
    Result,
    /// Error encountered
    Error,
    /// Learned pattern or insight
    Learning,
    /// External tool interaction
    ToolInteraction,
    /// Inter-neuron communication
    Signal,
}

/// Search parameters for memory retrieval
#[derive(Debug, Clone)]
pub struct MemorySearch {
    pub neuron_id: Option<String>,
    pub layer: Option<String>,
    pub memory_type: Option<MemoryType>,
    pub content_query: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub min_importance: Option<f32>,
    pub limit: usize,
    pub use_semantic_search: bool,
}

impl Default for MemorySearch {
    fn default() -> Self {
        Self {
            neuron_id: None,
            layer: None,
            memory_type: None,
            content_query: None,
            start_time: None,
            end_time: None,
            min_importance: None,
            limit: 10,
            use_semantic_search: false,
        }
    }
}

/// Memory context for neurons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContext {
    pub recent_tasks: Vec<MemoryEntry>,
    pub relevant_learnings: Vec<MemoryEntry>,
    pub similar_experiences: Vec<MemoryEntry>,
    pub error_patterns: Vec<MemoryEntry>,
}

/// Trait for memory storage implementations
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// Initialize the memory store
    async fn initialize(&self) -> crate::Result<()>;
    
    /// Store a memory entry
    async fn store(&self, entry: MemoryEntry) -> crate::Result<Uuid>;
    
    /// Retrieve a memory entry by ID
    async fn get(&self, id: Uuid) -> crate::Result<Option<MemoryEntry>>;
    
    /// Search for memory entries
    async fn search(&self, params: MemorySearch) -> crate::Result<Vec<MemoryEntry>>;
    
    /// Update access count and timestamp
    async fn record_access(&self, id: Uuid) -> crate::Result<()>;
    
    /// Delete old or unimportant memories
    async fn cleanup(&self, before: DateTime<Utc>, min_importance: f32) -> crate::Result<u64>;
    
    /// Get memory statistics
    async fn get_stats(&self, neuron_id: &str) -> crate::Result<MemoryStats>;
    
    /// Build context for a neuron
    async fn build_context(
        &self, 
        neuron_id: &str, 
        current_task: &str
    ) -> crate::Result<MemoryContext>;
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_entries: u64,
    pub entries_by_type: std::collections::HashMap<String, u64>,
    pub average_importance: f32,
    pub total_access_count: u64,
    pub oldest_entry: Option<DateTime<Utc>>,
    pub newest_entry: Option<DateTime<Utc>>,
}

/// Memory builder for creating entries
pub struct MemoryBuilder {
    neuron_id: String,
    layer: String,
    entry_type: MemoryType,
    content: String,
    metadata: serde_json::Value,
    importance: f32,
}

impl MemoryBuilder {
    pub fn new(neuron_id: String, layer: String) -> Self {
        Self {
            neuron_id,
            layer,
            entry_type: MemoryType::Task,
            content: String::new(),
            metadata: serde_json::json!({}),
            importance: 0.5,
        }
    }
    
    pub fn with_type(mut self, entry_type: MemoryType) -> Self {
        self.entry_type = entry_type;
        self
    }
    
    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }
    
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
    
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.importance = importance;
        self
    }
    
    pub fn build(self) -> MemoryEntry {
        let now = Utc::now();
        MemoryEntry {
            id: Uuid::new_v4(),
            neuron_id: self.neuron_id,
            layer: self.layer,
            timestamp: now,
            entry_type: self.entry_type,
            content: self.content,
            metadata: self.metadata,
            embedding: None,
            importance: self.importance,
            access_count: 0,
            last_accessed: now,
        }
    }
}