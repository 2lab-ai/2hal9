//! Cognitive Layer - Information processing units and hierarchical neurons
//!
//! This layer implements various types of information processing units with
//! hierarchical neuron types (L1-L5) that have distinct behaviors and capabilities.

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;
use crate::Result;

pub mod neurons;
pub mod processing;
pub mod learning;
pub mod patterns;
pub mod factory;
pub mod a2a;
pub mod consciousness_metrics;

// Individual neuron modules
pub mod l1_reflexive;
pub mod l2_implementation;
pub mod l3_operational;
pub mod l4_tactical;
pub mod l5_strategic;

pub use neurons::*;
pub use processing::*;
pub use learning::*;
pub use patterns::*;
pub use factory::*;
pub use a2a::*;
pub use consciousness_metrics::*;

// Re-export neuron types
pub use l1_reflexive::L1ReflexiveNeuron;
pub use l2_implementation::L2ImplementationNeuron;
pub use l3_operational::L3OperationalNeuron;
pub use l4_tactical::L4TacticalNeuron;
pub use l5_strategic::L5StrategicNeuron;

/// Abstract cognitive unit - base trait for all processing units
#[async_trait]
pub trait CognitiveUnit: Send + Sync + 'static {
    /// Input type for this unit
    type Input: Send + Sync + 'static;
    
    /// Output type for this unit
    type Output: Send + Sync + 'static;
    
    /// Internal state type
    type State: CognitiveState;
    
    /// Get unit identifier
    fn id(&self) -> &Uuid;
    
    /// Get unit layer
    fn layer(&self) -> CognitiveLayer;
    
    /// Process input and produce output
    async fn process(&mut self, input: Self::Input) -> Result<Self::Output>;
    
    /// Learn from feedback/gradient
    async fn learn(&mut self, gradient: LearningGradient) -> Result<()>;
    
    /// Introspect current state
    async fn introspect(&self) -> Self::State;
    
    /// Reset to initial state
    async fn reset(&mut self) -> Result<()>;
}

/// Cognitive state trait
pub trait CognitiveState: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    /// Get state summary
    fn summary(&self) -> String;
    
    /// Check if state is healthy
    fn is_healthy(&self) -> bool;
    
    /// Get state metrics
    fn metrics(&self) -> StateMetrics;
}

/// Cognitive layer hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CognitiveLayer {
    /// L1: Reflexive - Immediate response
    Reflexive,
    /// L2: Implementation - Direct execution
    Implementation,
    /// L3: Operational - Task coordination
    Operational,
    /// L4: Tactical - Medium-term planning
    Tactical,
    /// L5: Strategic - Long-term vision
    Strategic,
}

impl CognitiveLayer {
    /// Get layer depth (1-5)
    pub fn depth(&self) -> u8 {
        match self {
            Self::Reflexive => 1,
            Self::Implementation => 2,
            Self::Operational => 3,
            Self::Tactical => 4,
            Self::Strategic => 5,
        }
    }
    
    /// Get layer name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Reflexive => "Reflexive",
            Self::Implementation => "Implementation",
            Self::Operational => "Operational",
            Self::Tactical => "Tactical",
            Self::Strategic => "Strategic",
        }
    }
    
    /// Get layer description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Reflexive => "Immediate reaction and response",
            Self::Implementation => "Direct code execution and implementation",
            Self::Operational => "Design and task coordination",
            Self::Tactical => "Planning and strategy execution",
            Self::Strategic => "Vision and long-term goals",
        }
    }
    
    /// Get processing characteristics
    pub fn characteristics(&self) -> LayerCharacteristics {
        match self {
            Self::Reflexive => LayerCharacteristics {
                abstraction_level: 0.1,
                time_horizon: std::time::Duration::from_millis(100),
                complexity_threshold: 0.2,
                learning_rate: 0.1,
            },
            Self::Implementation => LayerCharacteristics {
                abstraction_level: 0.3,
                time_horizon: std::time::Duration::from_secs(10),
                complexity_threshold: 0.4,
                learning_rate: 0.05,
            },
            Self::Operational => LayerCharacteristics {
                abstraction_level: 0.5,
                time_horizon: std::time::Duration::from_secs(60),
                complexity_threshold: 0.6,
                learning_rate: 0.02,
            },
            Self::Tactical => LayerCharacteristics {
                abstraction_level: 0.7,
                time_horizon: std::time::Duration::from_secs(300),
                complexity_threshold: 0.8,
                learning_rate: 0.01,
            },
            Self::Strategic => LayerCharacteristics {
                abstraction_level: 0.9,
                time_horizon: std::time::Duration::from_secs(3600),
                complexity_threshold: 0.95,
                learning_rate: 0.005,
            },
        }
    }
}

/// Layer processing characteristics
#[derive(Debug, Clone)]
pub struct LayerCharacteristics {
    /// Level of abstraction (0.0 = concrete, 1.0 = abstract)
    pub abstraction_level: f32,
    /// Time horizon for planning/consideration
    pub time_horizon: std::time::Duration,
    /// Complexity threshold for delegating to lower layers
    pub complexity_threshold: f32,
    /// Learning rate for this layer
    pub learning_rate: f32,
}

/// Learning gradient for cognitive units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningGradient {
    pub gradient_id: Uuid,
    pub error_signal: ErrorSignal,
    pub adjustments: Vec<ParameterAdjustment>,
    pub importance: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Error signal for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSignal {
    pub error_type: String,
    pub magnitude: f32,
    pub context: HashMap<String, serde_json::Value>,
}

/// Parameter adjustment suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterAdjustment {
    pub parameter: String,
    pub current_value: f32,
    pub suggested_delta: f32,
    pub confidence: f32,
}

/// State metrics for cognitive units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMetrics {
    pub activations_processed: u64,
    pub errors_encountered: u64,
    pub learning_iterations: u64,
    pub average_processing_time_ms: f64,
    pub memory_usage_bytes: u64,
}

/// Factory for creating cognitive units
pub trait CognitiveFactory: Send + Sync {
    /// Create a cognitive unit for a specific layer
    fn create_unit(&self, layer: CognitiveLayer, config: CognitiveConfig) -> Result<Box<dyn CognitiveUnit<Input = CognitiveInput, Output = CognitiveOutput, State = BasicCognitiveState>>>;
}

/// Generic input for cognitive units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveInput {
    pub content: String,
    pub context: HashMap<String, serde_json::Value>,
    pub source_layer: Option<CognitiveLayer>,
}

/// Generic output from cognitive units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveOutput {
    pub content: String,
    pub confidence: f32,
    pub metadata: HashMap<String, serde_json::Value>,
    pub target_layers: Vec<CognitiveLayer>,
}

/// Basic cognitive state implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicCognitiveState {
    pub unit_id: Uuid,
    pub layer: CognitiveLayer,
    pub metrics: StateMetrics,
    pub parameters: HashMap<String, f32>,
}

impl CognitiveState for BasicCognitiveState {
    fn summary(&self) -> String {
        format!("{} unit {} - {} activations processed", 
                self.layer.name(), 
                self.unit_id, 
                self.metrics.activations_processed)
    }
    
    fn is_healthy(&self) -> bool {
        self.metrics.errors_encountered < self.metrics.activations_processed / 10
    }
    
    fn metrics(&self) -> StateMetrics {
        self.metrics.clone()
    }
}

/// Configuration for cognitive units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveConfig {
    pub id: Uuid,
    pub layer: CognitiveLayer,
    pub initial_parameters: HashMap<String, f32>,
    pub connections: ConnectionConfig,
}

/// Connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub upward_connections: Vec<Uuid>,
    pub lateral_connections: Vec<Uuid>,
    pub downward_connections: Vec<Uuid>,
}