//! HAL9 Agent Self-Organization and Dropout System
//! 
//! This module implements autonomous hierarchical structure formation
//! and dynamic quality management for distributed AI agent networks.

pub mod agent;
pub mod assessment;
pub mod dropout;
pub mod evaluation;
pub mod network;

pub use agent::*;
pub use assessment::*;
pub use dropout::*;
pub use evaluation::*;
pub use network::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Agent capability levels from L1 to L20
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AgentLevel(u8);

impl AgentLevel {
    pub fn new(level: u8) -> Result<Self, AgentError> {
        if (1..=20).contains(&level) {
            Ok(AgentLevel(level))
        } else {
            Err(AgentError::InvalidLevel(level))
        }
    }

    pub fn value(&self) -> u8 {
        self.0
    }

    pub fn layer(&self) -> NetworkLayer {
        match self.0 {
            1..=5 => NetworkLayer::Basic,
            6..=10 => NetworkLayer::Intermediate,
            11..=15 => NetworkLayer::Advanced,
            16..=20 => NetworkLayer::Expert,
            _ => unreachable!(),
        }
    }
}

/// Network layers for agent placement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkLayer {
    Basic,
    Intermediate,
    Advanced,
    Expert,
    Probationary,
}

/// Agent profile containing essential information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub id: Uuid,
    pub capability_level: AgentLevel,
    pub context_window: ContextWindow,
    pub specialization: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Context window sizes for different agent levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextWindow {
    Small(usize),   // 4K tokens
    Medium(usize),  // 16K tokens
    Large(usize),   // 64K tokens
    XLarge(usize),  // 128K tokens
    Default(usize), // 8K tokens
}

/// Performance metrics for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub agent_id: Uuid,
    pub success_rate: f32,
    pub response_time_avg: std::time::Duration,
    pub peer_ratings: Vec<f32>,
    pub contribution_score: f32,
    pub last_updated: DateTime<Utc>,
}

/// Errors that can occur in the agent system
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Invalid agent level: {0}. Must be between 1 and 20")]
    InvalidLevel(u8),
    
    #[error("Agent not found: {0}")]
    AgentNotFound(Uuid),
    
    #[error("Evaluation failed: {0}")]
    EvaluationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Result type for agent operations
pub type AgentResult<T> = Result<T, AgentError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_level_creation() {
        assert!(AgentLevel::new(1).is_ok());
        assert!(AgentLevel::new(20).is_ok());
        assert!(AgentLevel::new(0).is_err());
        assert!(AgentLevel::new(21).is_err());
    }

    #[test]
    fn test_agent_level_layer_mapping() {
        assert_eq!(AgentLevel::new(3).unwrap().layer(), NetworkLayer::Basic);
        assert_eq!(AgentLevel::new(8).unwrap().layer(), NetworkLayer::Intermediate);
        assert_eq!(AgentLevel::new(13).unwrap().layer(), NetworkLayer::Advanced);
        assert_eq!(AgentLevel::new(18).unwrap().layer(), NetworkLayer::Expert);
    }
}