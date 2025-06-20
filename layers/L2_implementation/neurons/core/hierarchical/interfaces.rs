//! Interface specifications and boundaries between hierarchical layers
//!
//! This module defines the contracts and boundaries between each layer,
//! ensuring clean separation of concerns and enabling independent evolution.

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
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
/// Adapter for legacy neuron interface to work with hierarchical architecture
pub struct LegacyNeuronAdapter {
    legacy_neuron: Box<dyn crate::neuron::NeuronInterface>,
    layer_mapping: LayerId,
    neuron_id: Uuid,
    metrics: std::sync::Arc<parking_lot::Mutex<AdapterMetrics>>,
    signal_buffer: tokio::sync::mpsc::Sender<crate::NeuronSignal>,
    message_receiver: tokio::sync::mpsc::Receiver<LayerMessage>,
}

#[derive(Debug, Default, Clone)]
pub struct AdapterMetrics {
    signals_adapted: u64,
    messages_converted: u64,
    errors: u64,
    average_latency_ms: f64,
}

impl LegacyNeuronAdapter {
    pub fn new(neuron: Box<dyn crate::neuron::NeuronInterface>, layer: LayerId) -> Self {
        let neuron_id = neuron.id();
        let neuron_uuid = Uuid::parse_str(neuron_id).unwrap_or_else(|_| Uuid::new_v4());
        let (signal_tx, _signal_rx) = tokio::sync::mpsc::channel(100);
        let (_message_tx, message_rx) = tokio::sync::mpsc::channel(100);
        
        Self {
            legacy_neuron: neuron,
            layer_mapping: layer,
            neuron_id: neuron_uuid,
            metrics: std::sync::Arc::new(parking_lot::Mutex::new(AdapterMetrics::default())),
            signal_buffer: signal_tx,
            message_receiver: message_rx,
        }
    }
    
    /// Map legacy neuron layer to hierarchical layer
    pub fn map_layer(neuron_layer: &str) -> LayerId {
        match neuron_layer {
            "L1" => LayerId::Cognitive,  // L1 neurons map to Cognitive layer
            "L2" => LayerId::Cognitive,  // L2 neurons also Cognitive
            "L3" => LayerId::Cognitive,  // L3 as well
            "L4" => LayerId::Orchestration, // L4 planning maps to Orchestration
            "L5" => LayerId::Intelligence,  // L5 meta maps to Intelligence
            _ => LayerId::Cognitive, // Default mapping
        }
    }
    
    /// Adapt legacy signal to layer message
    pub async fn adapt_signal(&self, signal: &crate::NeuronSignal) -> Result<LayerMessage> {
        let start = std::time::Instant::now();
        
        let message = LayerMessage {
            id: signal.signal_id,
            source_layer: self.layer_mapping,
            target_layer: self.determine_target_layer(signal),
            message_type: self.map_signal_type(signal),
            payload: serde_json::json!({
                "neuron_id": self.neuron_id,
                "content": signal.payload.activation.content,
                "strength": signal.payload.activation.strength,
                "metadata": signal.metadata,
                "gradient": signal.payload.gradient,
                "legacy_format": true,
            }),
            timestamp: signal.timestamp,
            priority: self.map_priority(signal.payload.activation.strength),
        };
        
        let mut metrics = self.metrics.lock();
        metrics.signals_adapted += 1;
        metrics.average_latency_ms = 
            (metrics.average_latency_ms * (metrics.signals_adapted - 1) as f64 + 
             start.elapsed().as_millis() as f64) / metrics.signals_adapted as f64;
        
        Ok(message)
    }
    
    /// Convert layer message back to legacy signal
    pub async fn convert_message(&self, message: &LayerMessage) -> Result<crate::NeuronSignal> {
        let start = std::time::Instant::now();
        
        let content = message.payload.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
            
        let strength = message.payload.get("strength")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5) as f32;
        
        let signal = crate::NeuronSignal {
            signal_id: message.id,
            from_neuron: self.neuron_id.to_string(),
            to_neuron: String::new(), // Will be determined by routing
            layer_from: self.layer_mapping.name().to_string(),
            layer_to: String::new(),
            propagation_type: crate::PropagationType::Forward,
            batch_id: Uuid::new_v4(),
            timestamp: message.timestamp,
            metadata: HashMap::new(),
            payload: crate::SignalPayload {
                activation: crate::Activation {
                    content: content.to_string(),
                    strength,
                    features: HashMap::new(),
                },
                gradient: message.payload.get("gradient")
                    .and_then(|v| serde_json::from_value(v.clone()).ok()),
            },
        };
        
        let mut metrics = self.metrics.lock();
        metrics.messages_converted += 1;
        metrics.average_latency_ms = 
            (metrics.average_latency_ms * (metrics.messages_converted - 1) as f64 + 
             start.elapsed().as_millis() as f64) / metrics.messages_converted as f64;
        
        Ok(signal)
    }
    
    /// Run the adapter as a bidirectional bridge
    pub async fn run_bridge(self) -> Result<()> {
        let (_signal_tx, mut signal_rx) = tokio::sync::mpsc::channel::<crate::NeuronSignal>(100);
        let adapter = Arc::new(tokio::sync::Mutex::new(self));
        let adapter1 = adapter.clone();
        let adapter2 = adapter.clone();
        
        // Task to process legacy signals
        let signal_task = tokio::spawn(async move {
            while let Some(signal) = signal_rx.recv().await {
                let adapter = adapter1.lock().await;
                match adapter.adapt_signal(&signal).await {
                    Ok(_message) => {
                        // Send to hierarchical system
                        tracing::debug!("Adapted signal {} to message", signal.signal_id);
                    }
                    Err(e) => {
                        tracing::error!("Failed to adapt signal: {}", e);
                        adapter.metrics.lock().errors += 1;
                    }
                }
            }
        });
        
        // Task to process hierarchical messages
        let message_task = tokio::spawn(async move {
            let mut adapter = adapter2.lock().await;
            while let Some(message) = adapter.message_receiver.recv().await {
                match adapter.convert_message(&message).await {
                    Ok(signal) => {
                        if let Err(e) = adapter.signal_buffer.send(signal).await {
                            tracing::error!("Failed to send converted signal: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to convert message: {}", e);
                        adapter.metrics.lock().errors += 1;
                    }
                }
            }
        });
        
        // Wait for both tasks
        let _ = tokio::join!(signal_task, message_task);
        
        Ok(())
    }
    
    fn determine_target_layer(&self, signal: &crate::NeuronSignal) -> LayerId {
        // Analyze signal metadata to determine target layer
        if let Some(target) = signal.metadata.get("target_layer") {
            return Self::map_layer(target);
        }
        
        // Default to same layer
        self.layer_mapping
    }
    
    fn map_signal_type(&self, signal: &crate::NeuronSignal) -> MessageType {
        if signal.metadata.get("is_query").map(|v| v == "true").unwrap_or(false) {
            MessageType::Query
        } else if signal.metadata.get("is_control").map(|v| v == "true").unwrap_or(false) {
            MessageType::Control
        } else {
            MessageType::Data
        }
    }
    
    fn map_priority(&self, strength: f32) -> MessagePriority {
        if strength > 0.9 {
            MessagePriority::Critical
        } else if strength > 0.7 {
            MessagePriority::High
        } else if strength > 0.3 {
            MessagePriority::Normal
        } else {
            MessagePriority::Low
        }
    }
    
    /// Get adapter metrics
    pub fn metrics(&self) -> AdapterMetrics {
        self.metrics.lock().clone()
    }
}

/// Implement LayerInterface for the adapter to act as a layer
#[async_trait]
impl LayerInterface for LegacyNeuronAdapter {
    fn layer_id(&self) -> LayerId {
        self.layer_mapping
    }
    
    fn capabilities(&self) -> LayerCapabilities {
        LayerCapabilities {
            layer_id: self.layer_mapping,
            version: "legacy-adapter-1.0".to_string(),
            features: vec![
                "legacy-signal-conversion".to_string(),
                "bidirectional-bridge".to_string(),
                format!("neuron-{}", self.legacy_neuron.layer().as_str()),
            ],
            dependencies: vec![],
            resource_requirements: ResourceRequirements {
                min_memory_mb: 50,
                min_cpu_cores: 0.1,
                requires_gpu: false,
                network_bandwidth_mbps: None,
            },
        }
    }
    
    async fn initialize(&mut self, _config: LayerConfig) -> Result<()> {
        // Legacy neurons are pre-initialized
        Ok(())
    }
    
    async fn process_upward(&mut self, input: LayerMessage) -> Result<LayerMessage> {
        // Convert to signal and process through legacy neuron
        let signal = self.convert_message(&input).await?;
        
        match self.legacy_neuron.process_signal(&signal).await {
            Ok(response) => {
                // Process_signal returns a string, need to convert back to signal
                let response_signal = crate::NeuronSignal {
                    signal_id: Uuid::new_v4(),
                    from_neuron: self.legacy_neuron.id().to_string(),
                    to_neuron: String::new(),
                    layer_from: self.legacy_neuron.layer().as_str().to_string(),
                    layer_to: String::new(),
                    propagation_type: crate::PropagationType::Forward,
                    batch_id: signal.batch_id,
                    timestamp: chrono::Utc::now(),
                    metadata: HashMap::new(),
                    payload: crate::SignalPayload {
                        activation: crate::Activation {
                            content: response,
                            strength: 0.8,
                            features: HashMap::new(),
                        },
                        gradient: None,
                    },
                };
                self.adapt_signal(&response_signal).await
            }
            Err(e) => {
                self.metrics.lock().errors += 1;
                Err(e)
            }
        }
    }
    
    async fn process_downward(&mut self, input: LayerMessage) -> Result<LayerMessage> {
        // Same as upward for now
        self.process_upward(input).await
    }
    
    async fn metrics(&self) -> Result<LayerMetrics> {
        let adapter_metrics = self.metrics.lock();
        
        Ok(LayerMetrics {
            layer_id: self.layer_mapping,
            messages_processed: adapter_metrics.signals_adapted + adapter_metrics.messages_converted,
            average_latency_ms: adapter_metrics.average_latency_ms,
            error_rate: if adapter_metrics.signals_adapted > 0 {
                adapter_metrics.errors as f64 / adapter_metrics.signals_adapted as f64
            } else {
                0.0
            },
            resource_usage: ResourceUsage {
                cpu_percent: 1.0, // Placeholder
                memory_mb: 50,
                gpu_percent: None,
                network_mbps: 0.1,
            },
            custom_metrics: [
                ("signals_adapted".to_string(), adapter_metrics.signals_adapted as f64),
                ("messages_converted".to_string(), adapter_metrics.messages_converted as f64),
            ].into_iter().collect(),
        })
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        self.legacy_neuron.shutdown().await
    }
}

/// Migration coordinator for managing the transition
pub struct MigrationCoordinator {
    legacy_neurons: Vec<Box<dyn crate::neuron::NeuronInterface>>,
    adapters: Vec<LegacyNeuronAdapter>,
    progress: std::sync::Arc<parking_lot::Mutex<MigrationProgress>>,
}

#[derive(Debug, Default, Clone)]
pub struct MigrationProgress {
    pub total_neurons: usize,
    pub migrated_neurons: usize,
    pub failed_migrations: usize,
    pub start_time: Option<std::time::Instant>,
    pub estimated_completion: Option<std::time::Duration>,
}

impl MigrationCoordinator {
    pub fn new(neurons: Vec<Box<dyn crate::neuron::NeuronInterface>>) -> Self {
        let total = neurons.len();
        Self {
            legacy_neurons: neurons,
            adapters: Vec::new(),
            progress: std::sync::Arc::new(parking_lot::Mutex::new(MigrationProgress {
                total_neurons: total,
                ..Default::default()
            })),
        }
    }
    
    /// Migrate neurons in batches
    pub async fn migrate_batch(&mut self, batch_size: usize) -> Result<usize> {
        let mut batch_adapters = Vec::new();
        let neurons_to_migrate: Vec<_> = self.legacy_neurons.drain(..batch_size.min(self.legacy_neurons.len())).collect();
        
        {
            let mut progress = self.progress.lock();
            if progress.start_time.is_none() {
                progress.start_time = Some(std::time::Instant::now());
            }
        }
        
        for neuron in neurons_to_migrate {
            match self.migrate_single_neuron(neuron).await {
                Ok(adapter) => {
                    batch_adapters.push(adapter);
                    let mut progress = self.progress.lock();
                    progress.migrated_neurons += 1;
                    self.update_eta(&mut progress);
                }
                Err(e) => {
                    tracing::error!("Failed to migrate neuron: {}", e);
                    let mut progress = self.progress.lock();
                    progress.failed_migrations += 1;
                }
            }
        }
        
        let adapter_count = batch_adapters.len();
        self.adapters.extend(batch_adapters);
        Ok(adapter_count)
    }
    
    async fn migrate_single_neuron(&self, neuron: Box<dyn crate::neuron::NeuronInterface>) -> Result<LegacyNeuronAdapter> {
        let layer = LegacyNeuronAdapter::map_layer(neuron.layer().as_str());
        let adapter = LegacyNeuronAdapter::new(neuron, layer);
        
        // Validate adapter works
        let test_signal = crate::NeuronSignal {
            signal_id: Uuid::new_v4(),
            from_neuron: adapter.neuron_id.to_string(),
            to_neuron: String::new(),
            layer_from: adapter.layer_mapping.name().to_string(),
            layer_to: String::new(),
            propagation_type: crate::PropagationType::Forward,
            batch_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
            payload: crate::SignalPayload {
                activation: crate::Activation {
                    content: "test".to_string(),
                    strength: 0.5,
                    features: HashMap::new(),
                },
                gradient: None,
            },
        };
        
        adapter.adapt_signal(&test_signal).await?;
        
        Ok(adapter)
    }
    
    fn update_eta(&self, progress: &mut MigrationProgress) {
        if let Some(start) = progress.start_time {
            let elapsed = start.elapsed();
            let rate = progress.migrated_neurons as f64 / elapsed.as_secs_f64();
            let remaining = progress.total_neurons - progress.migrated_neurons;
            
            if rate > 0.0 {
                let eta_secs = remaining as f64 / rate;
                progress.estimated_completion = Some(std::time::Duration::from_secs_f64(eta_secs));
            }
        }
    }
    
    pub fn progress(&self) -> MigrationProgress {
        self.progress.lock().clone()
    }
    
    pub async fn complete_migration(self) -> Result<Vec<LegacyNeuronAdapter>> {
        Ok(self.adapters)
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_legacy_adapter_signal_conversion() {
        // Create a mock neuron
        struct MockNeuron {
            #[allow(dead_code)]
            id: Uuid,
            id_str: String,
        }
        
        #[async_trait]
        impl crate::neuron::NeuronInterface for MockNeuron {
            fn id(&self) -> &str { 
                &self.id_str
            }
            
            fn layer(&self) -> crate::neuron::Layer { 
                crate::neuron::Layer::L4 
            }
            
            async fn process_signal(&self, signal: &crate::NeuronSignal) -> Result<String> {
                Ok(format!("Processed signal: {}", signal.signal_id))
            }
            
            async fn health(&self) -> Result<crate::neuron::NeuronHealth> {
                Ok(crate::neuron::NeuronHealth {
                    state: crate::neuron::NeuronState::Running,
                    last_signal: Some(chrono::Utc::now()),
                    signals_processed: 0,
                    errors_count: 0,
                    uptime_seconds: 0,
                })
            }
            
            async fn shutdown(&self) -> Result<()> { Ok(()) }
        }
        
        let id = Uuid::new_v4();
        let neuron = Box::new(MockNeuron { 
            id,
            id_str: id.to_string(),
        });
        let adapter = LegacyNeuronAdapter::new(neuron, LayerId::Orchestration);
        
        // Test signal adaptation
        let signal = crate::NeuronSignal {
            signal_id: Uuid::new_v4(),
            from_neuron: adapter.neuron_id.to_string(),
            to_neuron: String::new(),
            layer_from: adapter.layer_mapping.name().to_string(),
            layer_to: String::new(),
            propagation_type: crate::PropagationType::Forward,
            batch_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            metadata: serde_json::json!({"key": "value"}).as_object().unwrap().iter().map(|(k, v)| (k.clone(), v.to_string())).collect(),
            payload: crate::SignalPayload {
                activation: crate::Activation {
                    content: "test content".to_string(),
                    strength: 0.8,
                    features: HashMap::new(),
                },
                gradient: None,
            },
        };
        
        let message = adapter.adapt_signal(&signal).await.unwrap();
        assert_eq!(message.source_layer, LayerId::Orchestration);
        assert_eq!(message.priority, MessagePriority::High);
        assert_eq!(message.payload["content"], "test content");
        
        // Test message conversion back
        let converted = adapter.convert_message(&message).await.unwrap();
        assert_eq!(converted.payload.activation.content, "test content");
        assert_eq!(converted.payload.activation.strength, 0.8);
    }
    
    #[tokio::test]
    async fn test_migration_coordinator() {
        struct MockNeuron {
            #[allow(dead_code)]
            id: Uuid,
            id_str: String,
            layer: crate::neuron::Layer,
        }
        
        #[async_trait]
        impl crate::neuron::NeuronInterface for MockNeuron {
            fn id(&self) -> &str { &self.id_str }
            
            fn layer(&self) -> crate::neuron::Layer { self.layer }
            
            async fn process_signal(&self, signal: &crate::NeuronSignal) -> Result<String> {
                Ok(format!("Processed signal: {}", signal.signal_id))
            }
            
            async fn health(&self) -> Result<crate::neuron::NeuronHealth> {
                Ok(crate::neuron::NeuronHealth {
                    state: crate::neuron::NeuronState::Running,
                    last_signal: Some(chrono::Utc::now()),
                    signals_processed: 0,
                    errors_count: 0,
                    uptime_seconds: 0,
                })
            }
            
            async fn shutdown(&self) -> Result<()> { Ok(()) }
        }
        
        // Create test neurons
        let neurons: Vec<Box<dyn crate::neuron::NeuronInterface>> = vec![
            {
                let id = Uuid::new_v4();
                Box::new(MockNeuron { 
                    id, 
                    id_str: id.to_string(),
                    layer: crate::neuron::Layer::L1
                })
            },
            {
                let id = Uuid::new_v4();
                Box::new(MockNeuron { 
                    id, 
                    id_str: id.to_string(),
                    layer: crate::neuron::Layer::L2
                })
            },
            {
                let id = Uuid::new_v4();
                Box::new(MockNeuron { 
                    id, 
                    id_str: id.to_string(),
                    layer: crate::neuron::Layer::L4
                })
            },
            {
                let id = Uuid::new_v4();
                Box::new(MockNeuron { 
                    id, 
                    id_str: id.to_string(),
                    layer: crate::neuron::Layer::L5
                })
            },
        ];
        
        let mut coordinator = MigrationCoordinator::new(neurons);
        
        // Migrate in batches
        let batch1 = coordinator.migrate_batch(2).await.unwrap();
        assert_eq!(batch1, 2);
        
        let progress = coordinator.progress();
        assert_eq!(progress.migrated_neurons, 2);
        assert_eq!(progress.total_neurons, 4);
        
        // Migrate remaining
        let batch2 = coordinator.migrate_batch(10).await.unwrap();
        assert_eq!(batch2, 2);
        
        let final_progress = coordinator.progress();
        assert_eq!(final_progress.migrated_neurons, 4);
        assert_eq!(final_progress.failed_migrations, 0);
    }
}