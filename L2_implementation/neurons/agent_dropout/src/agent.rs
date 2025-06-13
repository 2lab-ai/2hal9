//! Agent entry and management module

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AgentLevel, AgentProfile, ContextWindow};

/// Trait for agents entering the network
#[async_trait]
pub trait AgentEntry: Send + Sync {
    /// Agent introduces itself to the network
    fn introduce(&self) -> AgentProfile;
    
    /// Agent's self-assessed capability level
    fn self_assess_level(&self) -> AgentLevel;
    
    /// Agent's context window size
    fn context_window_size(&self) -> ContextWindow;
    
    /// Agent's declared specializations
    fn declare_specialization(&self) -> Vec<String>;
    
    /// Answer an assessment question
    async fn answer_assessment(&self, question: &AssessmentQuestion) -> AssessmentResponse;
}

/// Assessment question for evaluating agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentQuestion {
    pub id: Uuid,
    pub category: QuestionCategory,
    pub difficulty: AgentLevel,
    pub content: String,
    pub time_limit: Option<std::time::Duration>,
}

/// Categories of assessment questions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuestionCategory {
    LogicalReasoning,
    PatternRecognition,
    CreativeProblemSolving,
    SystemsThinking,
    MetaCognition,
    EthicalDilemmas,
}

/// Agent's response to an assessment question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentResponse {
    pub agent_id: Uuid,
    pub question_id: Uuid,
    pub answer: String,
    pub confidence: f32, // 0.0 to 1.0
    pub reasoning: Option<String>,
    pub time_taken: std::time::Duration,
    pub timestamp: DateTime<Utc>,
}

/// Extended agent implementation with neuron integration
#[derive(Clone)]
pub struct AgentNeuron {
    pub id: Uuid,
    pub profile: AgentProfile,
    pub evaluation_history: Vec<MutualEvaluation>,
    pub performance_tracker: PerformanceTracker,
    created_at: DateTime<Utc>,
}

impl AgentNeuron {
    pub fn new(level: AgentLevel, specializations: Vec<String>) -> Self {
        let id = Uuid::new_v4();
        let context_window = Self::context_window_for_level(level);
        
        Self {
            id,
            profile: AgentProfile {
                id,
                capability_level: level,
                context_window,
                specialization: specializations,
                timestamp: Utc::now(),
            },
            evaluation_history: Vec::new(),
            performance_tracker: PerformanceTracker::new(id),
            created_at: Utc::now(),
        }
    }
    
    fn context_window_for_level(level: AgentLevel) -> ContextWindow {
        match level.value() {
            1..=5 => ContextWindow::Small(4096),
            6..=10 => ContextWindow::Medium(16384),
            11..=15 => ContextWindow::Large(65536),
            16..=20 => ContextWindow::XLarge(131072),
            _ => ContextWindow::Default(8192),
        }
    }
    
    pub fn add_evaluation(&mut self, evaluation: MutualEvaluation) {
        self.evaluation_history.push(evaluation);
        self.performance_tracker.update_from_evaluation(&self.evaluation_history.last().unwrap());
    }
    
    pub fn current_performance(&self) -> f32 {
        self.performance_tracker.overall_score()
    }
}

/// Mutual evaluation between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualEvaluation {
    pub evaluator_id: Uuid,
    pub evaluatee_id: Uuid,
    pub assessment_scores: AssessmentScores,
    pub confidence_level: f32,
    pub comments: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Detailed assessment scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentScores {
    pub logical_consistency: f32,  // 0.0 - 1.0
    pub creative_approach: f32,    // 0.0 - 1.0
    pub depth_of_thought: f32,     // 0.0 - 1.0
    pub clarity_efficiency: f32,   // 0.0 - 1.0
}

impl AssessmentScores {
    pub fn weighted_average(&self) -> f32 {
        // Weights: 30%, 25%, 25%, 20%
        self.logical_consistency * 0.30 +
        self.creative_approach * 0.25 +
        self.depth_of_thought * 0.25 +
        self.clarity_efficiency * 0.20
    }
}

/// Performance tracking for agents
#[derive(Debug, Clone)]
pub struct PerformanceTracker {
    agent_id: Uuid,
    task_success_count: u64,
    task_total_count: u64,
    response_times: Vec<std::time::Duration>,
    peer_ratings: Vec<f32>,
}

impl PerformanceTracker {
    pub fn new(agent_id: Uuid) -> Self {
        Self {
            agent_id,
            task_success_count: 0,
            task_total_count: 0,
            response_times: Vec::new(),
            peer_ratings: Vec::new(),
        }
    }
    
    pub fn record_task(&mut self, success: bool, duration: std::time::Duration) {
        self.task_total_count += 1;
        if success {
            self.task_success_count += 1;
        }
        self.response_times.push(duration);
        
        // Keep only last 100 response times
        if self.response_times.len() > 100 {
            self.response_times.remove(0);
        }
    }
    
    pub fn update_from_evaluation(&mut self, evaluation: &MutualEvaluation) {
        let score = evaluation.assessment_scores.weighted_average();
        self.peer_ratings.push(score);
        
        // Keep only last 50 ratings
        if self.peer_ratings.len() > 50 {
            self.peer_ratings.remove(0);
        }
    }
    
    pub fn success_rate(&self) -> f32 {
        if self.task_total_count == 0 {
            0.0
        } else {
            self.task_success_count as f32 / self.task_total_count as f32
        }
    }
    
    pub fn average_response_time(&self) -> Option<std::time::Duration> {
        if self.response_times.is_empty() {
            None
        } else {
            let sum: std::time::Duration = self.response_times.iter().sum();
            Some(sum / self.response_times.len() as u32)
        }
    }
    
    pub fn average_peer_rating(&self) -> f32 {
        if self.peer_ratings.is_empty() {
            0.5 // Default neutral rating
        } else {
            self.peer_ratings.iter().sum::<f32>() / self.peer_ratings.len() as f32
        }
    }
    
    pub fn overall_score(&self) -> f32 {
        // Weighted combination of metrics
        let success_weight = 0.4;
        let rating_weight = 0.6;
        
        self.success_rate() * success_weight + self.average_peer_rating() * rating_weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = AgentNeuron::new(
            AgentLevel::new(10).unwrap(),
            vec!["reasoning".to_string(), "pattern_recognition".to_string()],
        );
        
        assert_eq!(agent.profile.capability_level.value(), 10);
        assert_eq!(agent.profile.specialization.len(), 2);
        match agent.profile.context_window {
            ContextWindow::Medium(size) => assert_eq!(size, 16384),
            _ => panic!("Wrong context window size"),
        }
    }

    #[test]
    fn test_performance_tracking() {
        let mut tracker = PerformanceTracker::new(Uuid::new_v4());
        
        // Record some tasks
        tracker.record_task(true, std::time::Duration::from_millis(100));
        tracker.record_task(true, std::time::Duration::from_millis(150));
        tracker.record_task(false, std::time::Duration::from_millis(200));
        
        assert_eq!(tracker.success_rate(), 2.0 / 3.0);
        assert_eq!(tracker.average_response_time().unwrap().as_millis(), 150);
    }

    #[test]
    fn test_assessment_scores() {
        let scores = AssessmentScores {
            logical_consistency: 0.8,
            creative_approach: 0.7,
            depth_of_thought: 0.9,
            clarity_efficiency: 0.6,
        };
        
        let weighted = scores.weighted_average();
        let expected = 0.8 * 0.30 + 0.7 * 0.25 + 0.9 * 0.25 + 0.6 * 0.20;
        
        assert!((weighted - expected).abs() < 0.001);
    }
}