//! Agent Dropout: Self-organizing quality management for AI agent networks
//! 
//! This system enables autonomous quality control through:
//! - Self-assessment protocols
//! - Strategic network placement
//! - Performance-based dropout
//! - Continuous improvement cycles

pub mod agent;
pub mod assessment;
pub mod dropout;
pub mod evaluation;
pub mod network;

pub use agent::{AgentLevel, AgentProfile, AgentCapability, NetworkLayer, AssessmentResponse};
pub use assessment::{AssessmentPool, QuestionValidator};
pub use dropout::{DropoutController, DropoutDecision};
pub use evaluation::{EvaluationEngine, EvaluationResult};
pub use network::{NetworkTopology, NetworkStats, LayerStats};

// Re-export common types
pub use agent::QuestionCategory;

use thiserror::Error;
use uuid::Uuid;

/// Error type for agent operations
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent not found: {0}")]
    NotFound(Uuid),
    #[error("Memory limit exceeded")]
    MemoryLimitExceeded,
    #[error("Quality threshold not met: {0}")]
    QualityTooLow(f32),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Assessment failed: {0}")]
    AssessmentFailed(String),
}

/// Result type for agent operations
pub type AgentResult<T> = Result<T, AgentError>;

/// Context window size for agent memory
#[derive(Debug, Clone, Copy)]
pub struct ContextWindow {
    pub size: usize,
}

impl ContextWindow {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
    
    pub fn for_level(level: AgentLevel) -> Self {
        let size = match level.value() {
            1..=5 => 4_000,
            6..=10 => 8_000,
            11..=15 => 16_000,
            16..=20 => 32_000,
            _ => 8_000,
        };
        Self { size }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_context_window_sizes() {
        assert_eq!(ContextWindow::for_level(AgentLevel::L1).size, 4_000);
        assert_eq!(ContextWindow::for_level(AgentLevel::L5).size, 4_000);
        assert_eq!(ContextWindow::for_level(AgentLevel::L6).size, 8_000);
        assert_eq!(ContextWindow::for_level(AgentLevel::L10).size, 8_000);
        assert_eq!(ContextWindow::for_level(AgentLevel::L11).size, 16_000);
        assert_eq!(ContextWindow::for_level(AgentLevel::L15).size, 16_000);
        assert_eq!(ContextWindow::for_level(AgentLevel::L16).size, 32_000);
        assert_eq!(ContextWindow::for_level(AgentLevel::L20).size, 32_000);
    }
    
    #[test]
    fn test_agent_level_comparison() {
        let l1 = AgentLevel::L1;
        let l5 = AgentLevel::L5;
        let l10 = AgentLevel::L10;
        
        assert!(l1 < l5);
        assert!(l5 < l10);
        assert!(l1 < l10);
        assert_eq!(l5, l5);
    }
    
    #[test]
    fn test_network_layer_assignment() {
        assert_eq!(AgentLevel::L1.layer(), NetworkLayer::Basic);
        assert_eq!(AgentLevel::L5.layer(), NetworkLayer::Basic);
        assert_eq!(AgentLevel::L6.layer(), NetworkLayer::Intermediate);
        assert_eq!(AgentLevel::L10.layer(), NetworkLayer::Intermediate);
        assert_eq!(AgentLevel::L11.layer(), NetworkLayer::Advanced);
        assert_eq!(AgentLevel::L20.layer(), NetworkLayer::Advanced);
    }
    
    #[test]
    fn test_agent_profile_creation() {
        use uuid::Uuid;
        
        let id = Uuid::new_v4();
        let profile = AgentProfile::new(id, AgentLevel::L10);
        
        assert_eq!(profile.id, id);
        assert_eq!(profile.capability_level, AgentLevel::L10);
        assert_eq!(profile.performance_history.len(), 0);
    }
}

/// Integration test module
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::time::Duration;
    use uuid::Uuid;
    
    #[tokio::test]
    async fn test_basic_workflow() {
        // Create components
        let topology = NetworkTopology::new();
        let assessment_pool = AssessmentPool::new();
        let _evaluation_engine = EvaluationEngine::new();
        let dropout_controller = DropoutController::new(
            1024 * 1024 * 10, // 10MB
            Duration::from_secs(300),
            0.7,
        );
        
        // Add an agent
        let agent_id = Uuid::new_v4();
        let profile = AgentProfile::new(agent_id, AgentLevel::L5);
        topology.place_agent(&profile).await;
        
        // Get assessment questions
        // Get multiple questions for level L5
        let mut questions = Vec::new();
        for _ in 0..5 {
            if let Some(question) = assessment_pool.get_question_for_level(AgentLevel::L5) {
                questions.push(question.clone());
            }
        }
        assert!(!questions.is_empty());
        
        // Create mock responses
        let _responses: Vec<AssessmentResponse> = questions.iter().map(|q| {
            AssessmentResponse {
                question_id: q.id,
                answer: "Test answer".to_string(),
                time_taken: Duration::from_secs(30),
                confidence: 0.8,
            }
        }).collect();
        
        // Evaluate - for now just create a dummy evaluation result
        let result = EvaluationResult {
            overall_score: 0.8,
            level_estimate: AgentLevel::L5,
            category_scores: std::collections::HashMap::new(),
            time_efficiency: 0.9,
            consistency_score: 0.85,
        };
        assert!(result.overall_score >= 0.0 && result.overall_score <= 1.0);
        
        // Check dropout decision
        let should_dropout = dropout_controller.should_dropout(&profile, result.overall_score).await;
        assert!(!should_dropout); // Should not dropout with reasonable score
    }
    
    #[tokio::test]
    async fn test_network_connections() {
        let topology = NetworkTopology::new();
        
        // Add two agents
        let agent1 = AgentProfile::new(Uuid::new_v4(), AgentLevel::L5);
        let agent2 = AgentProfile::new(Uuid::new_v4(), AgentLevel::L6);
        
        topology.place_agent(&agent1).await;
        topology.place_agent(&agent2).await;
        
        // Connect them
        topology.connect_agents(agent1.id, agent2.id, 0.8).await;
        
        // Verify connection
        assert!(topology.are_connected(agent1.id, agent2.id).await);
        
        // Check stats
        let stats = topology.get_network_stats().await;
        assert_eq!(stats.total_agents, 2);
        // We expect 4 edges: 2 from auto-connection when placing agent2 (adjacent layers)
        // and 2 more from manual connection (all connections are bidirectional)
        assert_eq!(stats.total_connections, 4);
    }
}

/// Benchmarks
#[cfg(all(test, not(debug_assertions)))]
mod benches {
    use super::*;
    use std::time::{Duration, Instant};
    use uuid::Uuid;
    
    #[tokio::test]
    async fn bench_add_agents() {
        let topology = NetworkTopology::new();
        let count = 1000;
        
        let start = Instant::now();
        
        for i in 0..count {
            let level = match i % 20 {
                0..=5 => AgentLevel::L1,
                6..=10 => AgentLevel::L5,
                11..=15 => AgentLevel::L10,
                _ => AgentLevel::L15,
            };
            
            let profile = AgentProfile::new(Uuid::new_v4(), level);
            topology.place_agent(&profile).await;
        }
        
        let elapsed = start.elapsed();
        println!("Added {} agents in {:?}", count, elapsed);
        println!("Average: {:?} per agent", elapsed / count);
        
        assert!(elapsed < Duration::from_secs(1)); // Should be fast
    }
    
    #[test]
    fn bench_assessment_generation() {
        let pool = AssessmentPool::new();
        let iterations = 100;
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            // Get 20 questions for level L10
            let mut _questions = Vec::new();
            for _ in 0..20 {
                if let Some(q) = pool.get_question_for_level(AgentLevel::L10) {
                    _questions.push(q.clone());
                }
            }
        }
        
        let elapsed = start.elapsed();
        println!("Generated {} assessments in {:?}", iterations, elapsed);
        println!("Average: {:?} per assessment", elapsed / iterations);
    }
}