//! Interface specifications and boundaries between hierarchical layers
//!
//! This module defines the contracts and boundaries between each layer,
//! ensuring clean separation of concerns and enabling independent evolution.

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::any::Any;
use crate::Result;

/// Base interface that all layers must implement
#[async_trait]
pub trait LayerInterface: Send + Sync + 'static {
    /// Get the layer identifier
    fn layer_id(&self) -> LayerId;
    
    /// Get layer capabilities
    fn capabilities(&self) -> LayerCapabilities;
    
    /// Initialize the layer
    async fn initialize(&mut self, config: LayerConfig) -> Result<()>;
    
    /// Process input from lower layer
    async fn process_upward(&mut self, input: LayerMessage) -> Result<LayerMessage>;
    
    /// Process input from higher layer
    async fn process_downward(&mut self, input: LayerMessage) -> Result<LayerMessage>;
    
    /// Get layer metrics
    async fn metrics(&self) -> Result<LayerMetrics>;
    
    /// Shutdown the layer gracefully
    async fn shutdown(&mut self) -> Result<()>;
}

/// Layer identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerId {
    Substrate,
    Protocol,
    Cognitive,
    Orchestration,
    Intelligence,
}

impl LayerId {
    pub fn level(&self) -> u8 {
        match self {
            Self::Substrate => 1,
            Self::Protocol => 2,
            Self::Cognitive => 3,
            Self::Orchestration => 4,
            Self::Intelligence => 5,
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Self::Substrate => "Substrate",
            Self::Protocol => "Protocol",
            Self::Cognitive => "Cognitive",
            Self::Orchestration => "Orchestration",
            Self::Intelligence => "Intelligence",
        }
    }
}

/// Layer capabilities declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerCapabilities {
    pub layer_id: LayerId,
    pub version: String,
    pub features: Vec<String>,
    pub dependencies: Vec<LayerId>,
    pub resource_requirements: ResourceRequirements,
}

/// Resource requirements for a layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub min_memory_mb: u64,
    pub min_cpu_cores: f32,
    pub requires_gpu: bool,
    pub network_bandwidth_mbps: Option<f32>,
}

/// Configuration for layer initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerConfig {
    pub layer_id: LayerId,
    pub parameters: serde_json::Value,
    pub connections: ConnectionConfig,
}

/// Connection configuration between layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub upward: Option<LayerId>,
    pub downward: Option<LayerId>,
    pub lateral: Vec<LayerId>,
}

/// Message passed between layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerMessage {
    pub id: Uuid,
    pub source_layer: LayerId,
    pub target_layer: LayerId,
    pub message_type: MessageType,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub priority: MessagePriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Data,
    Control,
    Query,
    Response,
    Event,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Layer performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerMetrics {
    pub layer_id: LayerId,
    pub messages_processed: u64,
    pub average_latency_ms: f64,
    pub error_rate: f64,
    pub resource_usage: ResourceUsage,
    pub custom_metrics: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub gpu_percent: Option<f32>,
    pub network_mbps: f32,
}

/// Boundary between two layers
#[async_trait]
pub trait LayerBoundary: Send + Sync {
    /// Get the upper layer ID
    fn upper_layer(&self) -> LayerId;
    
    /// Get the lower layer ID
    fn lower_layer(&self) -> LayerId;
    
    /// Transform message going upward
    async fn transform_upward(&self, message: LayerMessage) -> Result<LayerMessage>;
    
    /// Transform message going downward
    async fn transform_downward(&self, message: LayerMessage) -> Result<LayerMessage>;
    
    /// Validate message crossing boundary
    async fn validate_message(&self, message: &LayerMessage) -> Result<bool>;
    
    /// Get boundary metrics
    async fn metrics(&self) -> Result<BoundaryMetrics>;
}

/// Metrics for layer boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryMetrics {
    pub messages_transformed: u64,
    pub validation_failures: u64,
    pub average_transform_time_ms: f64,
}

/// Factory for creating layer instances
#[async_trait]
pub trait LayerFactory: Send + Sync {
    /// Create a layer instance
    async fn create_layer(&self, layer_id: LayerId, config: LayerConfig) -> Result<Box<dyn LayerInterface>>;
    
    /// Create a boundary between layers
    async fn create_boundary(&self, upper: LayerId, lower: LayerId) -> Result<Box<dyn LayerBoundary>>;
}

/// Standard layer implementations

/// Substrate layer interface
#[async_trait]
pub trait SubstrateInterface: LayerInterface {
    /// Get runtime handle
    fn runtime(&self) -> &dyn Any;
    
    /// Get transport handle
    fn transport(&self) -> &dyn Any;
    
    /// Get storage handle
    fn storage(&self) -> &dyn Any;
    
    /// Get resource manager
    fn resources(&self) -> &dyn Any;
}

/// Protocol layer interface
#[async_trait]
pub trait ProtocolInterface: LayerInterface {
    /// Negotiate protocol with peer
    async fn negotiate(&self, peer_id: &str) -> Result<String>;
    
    /// Get supported protocols
    fn supported_protocols(&self) -> Vec<String>;
}

/// Cognitive layer interface
#[async_trait]
pub trait CognitiveInterface: LayerInterface {
    /// Get cognitive units
    fn units(&self) -> Vec<Uuid>;
    
    /// Process cognitive input
    async fn process_cognitive(&mut self, input: CognitiveInput) -> Result<CognitiveOutput>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveInput {
    pub content: String,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveOutput {
    pub content: String,
    pub confidence: f32,
}

/// Orchestration layer interface
#[async_trait]
pub trait OrchestrationInterface: LayerInterface {
    /// Get current topology
    async fn topology(&self) -> Result<serde_json::Value>;
    
    /// Route signal
    async fn route(&self, signal: RoutingSignal) -> Result<Vec<Uuid>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingSignal {
    pub source: Uuid,
    pub signal_type: String,
    pub payload: serde_json::Value,
}

/// Intelligence layer interface
#[async_trait]
pub trait IntelligenceInterface: LayerInterface {
    /// Set system goals
    async fn set_goals(&mut self, goals: Vec<serde_json::Value>) -> Result<()>;
    
    /// Get emergence report
    async fn emergence_report(&self) -> Result<serde_json::Value>;
}

/// Migration support for transitioning between architectures

/// Migration strategy from old to new architecture
#[async_trait]
pub trait MigrationStrategy: Send + Sync {
    /// Get migration phases
    fn phases(&self) -> Vec<MigrationPhase>;
    
    /// Execute migration phase
    async fn execute_phase(&mut self, phase: &MigrationPhase) -> Result<MigrationResult>;
    
    /// Rollback migration phase
    async fn rollback_phase(&mut self, phase: &MigrationPhase) -> Result<()>;
    
    /// Validate migration state
    async fn validate(&self) -> Result<ValidationReport>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPhase {
    pub phase_id: String,
    pub description: String,
    pub estimated_duration: std::time::Duration,
    pub risk_level: RiskLevel,
    pub rollback_supported: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    pub phase_id: String,
    pub success: bool,
    pub items_migrated: u64,
    pub errors: Vec<String>,
    pub duration: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub component: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Compatibility layer for legacy support

/// Adapter for legacy neuron interface
pub struct LegacyNeuronAdapter {
    legacy_neuron: Box<dyn crate::neuron::NeuronInterface>,
    layer_mapping: LayerId,
}

impl LegacyNeuronAdapter {
    pub fn new(neuron: Box<dyn crate::neuron::NeuronInterface>, layer: LayerId) -> Self {
        Self {
            legacy_neuron: neuron,
            layer_mapping: layer,
        }
    }
    
    pub async fn adapt_signal(&self, signal: &crate::NeuronSignal) -> Result<LayerMessage> {
        Ok(LayerMessage {
            id: signal.signal_id,
            source_layer: self.layer_mapping,
            target_layer: self.layer_mapping,
            message_type: MessageType::Data,
            payload: serde_json::json!({
                "content": signal.payload.activation.content,
                "strength": signal.payload.activation.strength,
            }),
            timestamp: signal.timestamp,
            priority: MessagePriority::Normal,
        })
    }
}

/// Testing support for hierarchical architecture

/// Mock layer for testing
pub struct MockLayer {
    layer_id: LayerId,
    messages_received: std::sync::Arc<std::sync::Mutex<Vec<LayerMessage>>>,
}

impl MockLayer {
    pub fn new(layer_id: LayerId) -> Self {
        Self {
            layer_id,
            messages_received: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
    
    pub fn received_messages(&self) -> Vec<LayerMessage> {
        self.messages_received.lock().unwrap().clone()
    }
}

#[async_trait]
impl LayerInterface for MockLayer {
    fn layer_id(&self) -> LayerId {
        self.layer_id
    }
    
    fn capabilities(&self) -> LayerCapabilities {
        LayerCapabilities {
            layer_id: self.layer_id,
            version: "1.0.0".to_string(),
            features: vec!["mock".to_string()],
            dependencies: vec![],
            resource_requirements: ResourceRequirements {
                min_memory_mb: 10,
                min_cpu_cores: 0.1,
                requires_gpu: false,
                network_bandwidth_mbps: None,
            },
        }
    }
    
    async fn initialize(&mut self, _config: LayerConfig) -> Result<()> {
        Ok(())
    }
    
    async fn process_upward(&mut self, input: LayerMessage) -> Result<LayerMessage> {
        self.messages_received.lock().unwrap().push(input.clone());
        Ok(input)
    }
    
    async fn process_downward(&mut self, input: LayerMessage) -> Result<LayerMessage> {
        self.messages_received.lock().unwrap().push(input.clone());
        Ok(input)
    }
    
    async fn metrics(&self) -> Result<LayerMetrics> {
        Ok(LayerMetrics {
            layer_id: self.layer_id,
            messages_processed: self.messages_received.lock().unwrap().len() as u64,
            average_latency_ms: 1.0,
            error_rate: 0.0,
            resource_usage: ResourceUsage {
                cpu_percent: 0.1,
                memory_mb: 10,
                gpu_percent: None,
                network_mbps: 0.0,
            },
            custom_metrics: std::collections::HashMap::new(),
        })
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}