//! HAL9 MVP Library - Exports for testing

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Re-export modules
pub mod recorder;
pub mod exporter;
// web module is only available in the binary, not the library

// Define shared types needed by tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub from: String,
    pub to: String,
    pub content: String,
    pub layer: String,
    pub timestamp: DateTime<Utc>,
}

/// Deterministic mock neuron for each layer
pub struct MockNeuron {
    pub id: String,
    pub layer: String,
}

impl MockNeuron {
    pub fn new(id: &str, layer: &str) -> Self {
        Self {
            id: id.to_string(),
            layer: layer.to_string(),
        }
    }

    /// Process signal with deterministic responses for tests
    pub async fn process(&self, signal: &Signal) -> Vec<Signal> {
        // Simple test implementation
        match self.layer.as_str() {
            "L4" => {
                // L4 creates 2 L3 tasks
                vec![
                    Signal {
                        id: Uuid::new_v4(),
                        parent_id: Some(signal.id),
                        from: self.id.clone(),
                        to: "neuron-2".to_string(),
                        content: format!("Design architecture for: {}", signal.content),
                        layer: "L3".to_string(),
                        timestamp: Utc::now(),
                    },
                    Signal {
                        id: Uuid::new_v4(),
                        parent_id: Some(signal.id),
                        from: self.id.clone(),
                        to: "neuron-3".to_string(),
                        content: format!("Plan user interface for: {}", signal.content),
                        layer: "L3".to_string(),
                        timestamp: Utc::now(),
                    },
                ]
            }
            "L3" => {
                // Each L3 creates 2 L2 tasks
                vec![
                    Signal {
                        id: Uuid::new_v4(),
                        parent_id: Some(signal.id),
                        from: self.id.clone(),
                        to: "neuron-4".to_string(),
                        content: format!("Implement part 1 of: {}", signal.content),
                        layer: "L2".to_string(),
                        timestamp: Utc::now(),
                    },
                    Signal {
                        id: Uuid::new_v4(),
                        parent_id: Some(signal.id),
                        from: self.id.clone(),
                        to: "neuron-4".to_string(),
                        content: format!("Implement part 2 of: {}", signal.content),
                        layer: "L2".to_string(),
                        timestamp: Utc::now(),
                    },
                ]
            }
            _ => vec![]
        }
    }
}