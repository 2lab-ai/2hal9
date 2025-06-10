//! Processing patterns and strategies for cognitive units

use async_trait::async_trait;
use std::collections::HashMap;
use crate::Result;
use super::*;

/// Processing pattern trait
#[async_trait]
pub trait ProcessingPattern: Send + Sync {
    /// Pattern identifier
    fn id(&self) -> &str;
    
    /// Check if pattern applies to input
    fn matches(&self, input: &CognitiveInput) -> bool;
    
    /// Apply pattern to process input
    async fn apply(&self, input: CognitiveInput) -> Result<CognitiveOutput>;
}

/// Sequential processing pattern
pub struct SequentialPattern {
    steps: Vec<Box<dyn ProcessingStep>>,
}

#[async_trait]
pub trait ProcessingStep: Send + Sync {
    async fn execute(&self, state: &mut ProcessingState) -> Result<()>;
}

/// Parallel processing pattern
pub struct ParallelPattern {
    branches: Vec<Box<dyn ProcessingBranch>>,
    aggregator: Box<dyn ResultAggregator>,
}

#[async_trait]
pub trait ProcessingBranch: Send + Sync {
    async fn process(&self, input: CognitiveInput) -> Result<BranchResult>;
}

#[async_trait]
pub trait ResultAggregator: Send + Sync {
    async fn aggregate(&self, results: Vec<BranchResult>) -> Result<CognitiveOutput>;
}

#[derive(Debug, Clone)]
pub struct BranchResult {
    pub content: String,
    pub confidence: f32,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Recursive processing pattern
pub struct RecursivePattern {
    max_depth: usize,
    decomposer: Box<dyn TaskDecomposer>,
    composer: Box<dyn ResultComposer>,
}

#[async_trait]
pub trait TaskDecomposer: Send + Sync {
    async fn decompose(&self, input: CognitiveInput) -> Result<Vec<CognitiveInput>>;
}

#[async_trait]
pub trait ResultComposer: Send + Sync {
    async fn compose(&self, results: Vec<CognitiveOutput>) -> Result<CognitiveOutput>;
}

/// Emergent processing pattern
pub struct EmergentPattern {
    activation_threshold: f32,
    interaction_rules: Vec<InteractionRule>,
}

pub struct InteractionRule {
    pub condition: Box<dyn Fn(&ProcessingState) -> bool + Send + Sync>,
    pub action: Box<dyn Fn(&mut ProcessingState) -> Result<()> + Send + Sync>,
}

/// Processing state for pattern execution
#[derive(Debug, Clone)]
pub struct ProcessingState {
    pub input: CognitiveInput,
    pub intermediate: HashMap<String, serde_json::Value>,
    pub partial_results: Vec<PartialResult>,
}

#[derive(Debug, Clone)]
pub struct PartialResult {
    pub stage: String,
    pub content: String,
    pub confidence: f32,
}

/// Quantum-inspired superposition processing
pub struct QuantumPattern {
    superposition_states: Vec<SuperpositionState>,
    collapse_function: Box<dyn CollapseFunction>,
}

pub struct SuperpositionState {
    pub possibility: CognitiveOutput,
    pub amplitude: f32,
}

#[async_trait]
pub trait CollapseFunction: Send + Sync {
    async fn collapse(&self, states: &[SuperpositionState]) -> Result<CognitiveOutput>;
}

/// Processing strategy selector
pub struct ProcessingStrategySelector {
    strategies: HashMap<CognitiveLayer, Vec<Box<dyn ProcessingPattern>>>,
}

impl Default for ProcessingStrategySelector {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessingStrategySelector {
    pub fn new() -> Self {
        Self {
            strategies: HashMap::new(),
        }
    }
    
    pub fn register_strategy(&mut self, layer: CognitiveLayer, pattern: Box<dyn ProcessingPattern>) {
        self.strategies.entry(layer)
            .or_default()
            .push(pattern);
    }
    
    pub async fn select_and_apply(&self, layer: CognitiveLayer, input: CognitiveInput) -> Result<CognitiveOutput> {
        let patterns = self.strategies.get(&layer)
            .ok_or_else(|| crate::Error::Config(format!("No strategies for layer {:?}", layer)))?;
        
        for pattern in patterns {
            if pattern.matches(&input) {
                return pattern.apply(input).await;
            }
        }
        
        Err(crate::Error::Processing("No matching pattern found".to_string()))
    }
}

/// Attention mechanism for focusing processing
pub struct AttentionMechanism {
    focus_weights: HashMap<String, f32>,
    attention_window: usize,
}

impl AttentionMechanism {
    pub fn new(window_size: usize) -> Self {
        Self {
            focus_weights: HashMap::new(),
            attention_window: window_size,
        }
    }
    
    pub fn attend(&mut self, feature: &str, importance: f32) {
        self.focus_weights.insert(feature.to_string(), importance);
    }
    
    pub fn get_focus(&self) -> Vec<(&str, f32)> {
        let mut items: Vec<_> = self.focus_weights.iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        items.truncate(self.attention_window);
        items
    }
}