//! Agent entry and management module

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ContextWindow;

/// Agent capability levels from L1 to L20
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AgentLevel {
    L1, L2, L3, L4, L5, L6, L7, L8, L9, L10,
    L11, L12, L13, L14, L15, L16, L17, L18, L19, L20,
}

impl AgentLevel {
    /// Create from numeric value (1-20)
    pub fn from_value(value: u8) -> Option<Self> {
        match value {
            1 => Some(AgentLevel::L1),
            2 => Some(AgentLevel::L2),
            3 => Some(AgentLevel::L3),
            4 => Some(AgentLevel::L4),
            5 => Some(AgentLevel::L5),
            6 => Some(AgentLevel::L6),
            7 => Some(AgentLevel::L7),
            8 => Some(AgentLevel::L8),
            9 => Some(AgentLevel::L9),
            10 => Some(AgentLevel::L10),
            11 => Some(AgentLevel::L11),
            12 => Some(AgentLevel::L12),
            13 => Some(AgentLevel::L13),
            14 => Some(AgentLevel::L14),
            15 => Some(AgentLevel::L15),
            16 => Some(AgentLevel::L16),
            17 => Some(AgentLevel::L17),
            18 => Some(AgentLevel::L18),
            19 => Some(AgentLevel::L19),
            20 => Some(AgentLevel::L20),
            _ => None,
        }
    }
    
    /// Get numeric value (1-20)
    pub fn value(&self) -> u8 {
        match self {
            AgentLevel::L1 => 1,
            AgentLevel::L2 => 2,
            AgentLevel::L3 => 3,
            AgentLevel::L4 => 4,
            AgentLevel::L5 => 5,
            AgentLevel::L6 => 6,
            AgentLevel::L7 => 7,
            AgentLevel::L8 => 8,
            AgentLevel::L9 => 9,
            AgentLevel::L10 => 10,
            AgentLevel::L11 => 11,
            AgentLevel::L12 => 12,
            AgentLevel::L13 => 13,
            AgentLevel::L14 => 14,
            AgentLevel::L15 => 15,
            AgentLevel::L16 => 16,
            AgentLevel::L17 => 17,
            AgentLevel::L18 => 18,
            AgentLevel::L19 => 19,
            AgentLevel::L20 => 20,
        }
    }
    
    /// Get network layer for this level
    pub fn layer(&self) -> NetworkLayer {
        match self.value() {
            1..=5 => NetworkLayer::Basic,
            6..=10 => NetworkLayer::Intermediate,
            11..=20 => NetworkLayer::Advanced,
            _ => NetworkLayer::Intermediate,
        }
    }
}

/// Network layer categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkLayer {
    Basic,
    Intermediate,
    Advanced,
}

/// Agent profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub id: Uuid,
    pub capability_level: AgentLevel,
    pub specializations: Vec<String>,
    pub performance_history: Vec<f32>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

impl AgentProfile {
    pub fn new(id: Uuid, level: AgentLevel) -> Self {
        let now = Utc::now();
        Self {
            id,
            capability_level: level,
            specializations: Vec::new(),
            performance_history: Vec::new(),
            created_at: now,
            last_active: now,
        }
    }
}

/// Agent capability assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub level: AgentLevel,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub improvement_areas: Vec<String>,
}

/// Trait for agents entering the network
#[async_trait]
pub trait AgentEntry: Send + Sync {
    /// Agent introduces itself to the network
    fn introduce(&self) -> AgentProfile;
    
    /// Agent's self-assessed capability level
    fn self_assess_level(&self) -> AgentLevel;
    
    /// Agent's context window size
    fn context_window_size(&self) -> ContextWindow;
    
    /// Whether agent accepts assessment
    async fn accepts_assessment(&self) -> bool;
    
    /// Answer an assessment question
    async fn answer_assessment(&self, question: &AssessmentQuestion) -> AssessmentResponse;
}

/// Assessment question for evaluating agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Response to an assessment question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentResponse {
    pub question_id: Uuid,
    pub answer: String,
    pub time_taken: std::time::Duration,
    pub confidence: f32,
}

/// Result of mutual agent evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualEvaluation {
    pub evaluator: Uuid,
    pub evaluated: Uuid,
    pub scores: AssessmentScores,
    pub timestamp: DateTime<Utc>,
}

/// Assessment scores across different dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentScores {
    pub accuracy: f32,
    pub reasoning: f32,
    pub creativity: f32,
    pub speed: f32,
    pub consistency: f32,
}

impl AssessmentScores {
    pub fn overall(&self) -> f32 {
        self.accuracy * 0.3 
         + self.reasoning * 0.25 
         + self.creativity * 0.2 
         + self.speed * 0.15 
         + self.consistency * 0.1
    }
}

/// Trait for agents that can be evaluated
#[async_trait]
pub trait Evaluatable: Send + Sync {
    /// Evaluate another agent based on their responses
    async fn evaluate_peer(
        &self,
        peer_id: Uuid,
        responses: &[AssessmentResponse],
    ) -> MutualEvaluation;
    
    /// Accept evaluation from another agent
    async fn accept_evaluation(&self, evaluation: MutualEvaluation);
}

/// Represents an agent in the network
pub trait AgentNeuron: AgentEntry + Evaluatable {
    /// Unique identifier for this agent
    fn id(&self) -> Uuid;
    
    /// Current performance score
    fn performance_score(&self) -> f32;
    
    /// Update performance based on evaluations
    fn update_performance(&mut self, score: f32);
}