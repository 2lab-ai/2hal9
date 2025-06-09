//! Message definitions and traits for protocol layer

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Base message trait that all protocol messages must implement
pub trait Message: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static {
    /// Get message ID
    fn id(&self) -> &Uuid;
    
    /// Get message type identifier
    fn message_type(&self) -> &str;
    
    /// Get message timestamp
    fn timestamp(&self) -> &DateTime<Utc>;
    
    /// Get message priority
    fn priority(&self) -> MessagePriority;
    
    /// Validate message integrity
    fn validate(&self) -> bool {
        true // Default implementation
    }
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Base fields for all messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub id: Uuid,
    pub message_type: String,
    pub timestamp: DateTime<Utc>,
    pub priority: MessagePriority,
    pub source: String,
    pub destination: String,
    pub correlation_id: Option<Uuid>,
    pub metadata: HashMap<String, String>,
}

impl MessageHeader {
    pub fn new(source: &str, destination: &str, message_type: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_type: message_type.to_string(),
            timestamp: Utc::now(),
            priority: MessagePriority::Normal,
            source: source.to_string(),
            destination: destination.to_string(),
            correlation_id: None,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_correlation(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }
}

/// Activation message for forward propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationMessage {
    pub header: MessageHeader,
    pub content: String,
    pub strength: f32,
    pub features: HashMap<String, f32>,
    pub layer_from: String,
    pub layer_to: String,
}

impl Message for ActivationMessage {
    fn id(&self) -> &Uuid {
        &self.header.id
    }
    
    fn message_type(&self) -> &str {
        "activation"
    }
    
    fn timestamp(&self) -> &DateTime<Utc> {
        &self.header.timestamp
    }
    
    fn priority(&self) -> MessagePriority {
        self.header.priority
    }
}

/// Gradient message for backward propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientMessage {
    pub header: MessageHeader,
    pub error_type: String,
    pub magnitude: f32,
    pub loss: f32,
    pub adjustments: Vec<Adjustment>,
    pub propagation_path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adjustment {
    pub parameter: String,
    pub delta: f32,
    pub learning_rate: f32,
}

impl Message for GradientMessage {
    fn id(&self) -> &Uuid {
        &self.header.id
    }
    
    fn message_type(&self) -> &str {
        "gradient"
    }
    
    fn timestamp(&self) -> &DateTime<Utc> {
        &self.header.timestamp
    }
    
    fn priority(&self) -> MessagePriority {
        self.header.priority
    }
}

/// Query message for request/response pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMessage {
    pub header: MessageHeader,
    pub query: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timeout_ms: u64,
}

impl Message for QueryMessage {
    fn id(&self) -> &Uuid {
        &self.header.id
    }
    
    fn message_type(&self) -> &str {
        "query"
    }
    
    fn timestamp(&self) -> &DateTime<Utc> {
        &self.header.timestamp
    }
    
    fn priority(&self) -> MessagePriority {
        self.header.priority
    }
}

/// Response message for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub header: MessageHeader,
    pub result: ResponseResult,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseResult {
    Success(serde_json::Value),
    Error { code: String, message: String },
}

impl Message for ResponseMessage {
    fn id(&self) -> &Uuid {
        &self.header.id
    }
    
    fn message_type(&self) -> &str {
        "response"
    }
    
    fn timestamp(&self) -> &DateTime<Utc> {
        &self.header.timestamp
    }
    
    fn priority(&self) -> MessagePriority {
        self.header.priority
    }
}

/// Stream data chunk for continuous flows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub header: MessageHeader,
    pub stream_id: Uuid,
    pub sequence_number: u64,
    pub data: Vec<u8>,
    pub is_last: bool,
}

impl Message for StreamChunk {
    fn id(&self) -> &Uuid {
        &self.header.id
    }
    
    fn message_type(&self) -> &str {
        "stream_chunk"
    }
    
    fn timestamp(&self) -> &DateTime<Utc> {
        &self.header.timestamp
    }
    
    fn priority(&self) -> MessagePriority {
        self.header.priority
    }
}

/// Control message for protocol management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMessage {
    pub header: MessageHeader,
    pub command: ControlCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlCommand {
    Ping,
    Pong,
    Subscribe { topic: String },
    Unsubscribe { topic: String },
    Shutdown { reason: String },
    Reset,
}

impl Message for ControlMessage {
    fn id(&self) -> &Uuid {
        &self.header.id
    }
    
    fn message_type(&self) -> &str {
        "control"
    }
    
    fn timestamp(&self) -> &DateTime<Utc> {
        &self.header.timestamp
    }
    
    fn priority(&self) -> MessagePriority {
        MessagePriority::High // Control messages are high priority
    }
}