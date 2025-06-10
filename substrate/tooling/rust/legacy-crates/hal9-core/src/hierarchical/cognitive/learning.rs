//! Learning mechanisms for cognitive units

use crate::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

/// Learning mechanism trait
#[async_trait]
pub trait LearningMechanism: Send + Sync {
    /// Apply learning from gradient
    async fn apply_gradient(&mut self, gradient: &LearningGradient) -> Result<LearningOutcome>;

    /// Extract patterns from experience
    async fn extract_patterns(&self, experiences: &[Experience]) -> Result<Vec<LearnedPattern>>;

    /// Consolidate learning into long-term memory
    async fn consolidate(&mut self) -> Result<ConsolidationReport>;
}

use super::LearningGradient;

/// Learning outcome from gradient application
#[derive(Debug, Clone)]
pub struct LearningOutcome {
    pub parameters_updated: usize,
    pub average_change: f32,
    pub stability_score: f32,
}

/// Experience record for learning
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Experience {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub input: String,
    pub output: String,
    pub feedback: Feedback,
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Feedback {
    Positive { strength: f32 },
    Negative { strength: f32, reason: String },
    Neutral,
}

/// Learned pattern from experience
#[derive(Debug, Clone)]
pub struct LearnedPattern {
    pub pattern_id: Uuid,
    pub description: String,
    pub trigger_conditions: Vec<Condition>,
    pub recommended_action: String,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub field: String,
    pub operator: ComparisonOp,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum ComparisonOp {
    Equals,
    Contains,
    GreaterThan,
    LessThan,
    Matches(String), // Regex pattern
}

/// Consolidation report
#[derive(Debug, Clone)]
pub struct ConsolidationReport {
    pub patterns_strengthened: usize,
    pub patterns_weakened: usize,
    pub patterns_pruned: usize,
    pub memory_optimized_bytes: u64,
}

/// Hebbian learning implementation
pub struct HebbianLearning {
    connection_weights: HashMap<(Uuid, Uuid), f32>,
    learning_rate: f32,
}

impl HebbianLearning {
    pub fn new(learning_rate: f32) -> Self {
        Self {
            connection_weights: HashMap::new(),
            learning_rate,
        }
    }

    pub fn strengthen_connection(&mut self, from: Uuid, to: Uuid, activation: f32) {
        let key = (from, to);
        let current = self.connection_weights.get(&key).copied().unwrap_or(0.5);
        let new_weight = current + self.learning_rate * activation * (1.0 - current);
        self.connection_weights
            .insert(key, new_weight.clamp(0.0, 1.0));
    }
}

/// Reinforcement learning for decision making
pub struct ReinforcementLearning {
    q_table: HashMap<(State, Action), f32>,
    learning_rate: f32,
    discount_factor: f32,
    #[allow(dead_code)]
    exploration_rate: f32,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct State {
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Action {
    pub action_type: String,
    pub parameters: Vec<String>,
}

impl ReinforcementLearning {
    pub fn new(learning_rate: f32, discount_factor: f32, exploration_rate: f32) -> Self {
        Self {
            q_table: HashMap::new(),
            learning_rate,
            discount_factor,
            exploration_rate,
        }
    }

    pub fn update_q_value(&mut self, state: State, action: Action, reward: f32, next_state: State) {
        let current_q = self
            .q_table
            .get(&(state.clone(), action.clone()))
            .copied()
            .unwrap_or(0.0);

        let max_next_q = self.get_max_q_value(&next_state);
        let new_q = current_q
            + self.learning_rate * (reward + self.discount_factor * max_next_q - current_q);

        self.q_table.insert((state, action), new_q);
    }

    fn get_max_q_value(&self, state: &State) -> f32 {
        self.q_table
            .iter()
            .filter(|((s, _), _)| s == state)
            .map(|(_, q)| *q)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }
}

/// Meta-learning for learning how to learn
pub struct MetaLearning {
    learning_strategies: Vec<Box<dyn LearningStrategy>>,
    strategy_performance: HashMap<String, StrategyMetrics>,
}

#[async_trait]
pub trait LearningStrategy: Send + Sync {
    fn id(&self) -> &str;
    async fn learn(&mut self, data: &LearningData) -> Result<LearningResult>;
}

pub struct LearningData {
    pub inputs: Vec<serde_json::Value>,
    pub expected_outputs: Vec<serde_json::Value>,
    pub context: HashMap<String, serde_json::Value>,
}

pub struct LearningResult {
    pub accuracy: f32,
    pub loss: f32,
    pub iterations: usize,
}

#[derive(Debug, Clone, Default)]
pub struct StrategyMetrics {
    pub total_uses: usize,
    pub average_accuracy: f32,
    pub average_time_ms: f64,
    pub success_rate: f32,
}

impl Default for MetaLearning {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaLearning {
    pub fn new() -> Self {
        Self {
            learning_strategies: Vec::new(),
            strategy_performance: HashMap::new(),
        }
    }

    pub async fn select_best_strategy(
        &self,
        _data_characteristics: &DataCharacteristics,
    ) -> Option<&dyn LearningStrategy> {
        // Select strategy based on past performance and data characteristics
        self.learning_strategies
            .iter()
            .max_by_key(|s| {
                let metrics = self
                    .strategy_performance
                    .get(s.id())
                    .cloned()
                    .unwrap_or_default();
                (metrics.success_rate * 1000.0) as u32
            })
            .map(|s| s.as_ref())
    }
}

#[derive(Debug, Clone)]
pub struct DataCharacteristics {
    pub size: usize,
    pub dimensionality: usize,
    pub sparsity: f32,
    pub noise_level: f32,
}

/// Continual learning to avoid catastrophic forgetting
pub struct ContinualLearning {
    #[allow(dead_code)]
    memory_buffer: ExperienceReplay,
    #[allow(dead_code)]
    regularization_strength: f32,
}

pub struct ExperienceReplay {
    buffer: Vec<Experience>,
    capacity: usize,
}

impl ExperienceReplay {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn add(&mut self, experience: Experience) {
        if self.buffer.len() >= self.capacity {
            self.buffer.remove(0);
        }
        self.buffer.push(experience);
    }

    pub fn sample(&self, n: usize) -> Vec<&Experience> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.buffer
            .choose_multiple(&mut rng, n.min(self.buffer.len()))
            .collect()
    }
}
