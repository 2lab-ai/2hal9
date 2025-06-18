//! Integrated consciousness system combining all components
//!
//! Provides a unified interface for ConsciousnessMonitor, BoundaryNetwork, and EnhancedMockClaude

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::{
    consciousness::{ConsciousnessMonitor, ConsciousnessMetrics, ConsciousnessPhase, BoundaryNetwork},
    hierarchical::HierarchicalNeuron,
    Layer, Neuron, NeuronId,
};

/// Configuration for the integrated consciousness system
#[derive(Debug, Clone)]
pub struct ConsciousnessSystemConfig {
    /// History size for consciousness monitor
    pub history_size: usize,
    
    /// Update interval in milliseconds
    pub update_interval_ms: u64,
    
    /// Enable Claude integration
    pub enable_claude: bool,
    
    /// Enable real-time streaming
    pub enable_streaming: bool,
}

impl Default for ConsciousnessSystemConfig {
    fn default() -> Self {
        Self {
            history_size: 100,
            update_interval_ms: 100,
            enable_claude: true,
            enable_streaming: true,
        }
    }
}

/// Integrated consciousness system
pub struct IntegratedConsciousnessSystem {
    /// Configuration
    config: ConsciousnessSystemConfig,
    
    /// Consciousness monitor
    monitor: Arc<ConsciousnessMonitor>,
    
    /// Boundary network
    boundaries: Arc<RwLock<BoundaryNetwork>>,
    
    /// Enhanced Claude instances by layer
    claude_instances: Arc<RwLock<HashMap<Layer, EnhancedMockClaude>>>,
    
    /// Neurons being monitored
    neurons: Arc<RwLock<Vec<Arc<dyn Neuron>>>>,
    
    /// Last update timestamp
    last_update: Arc<RwLock<DateTime<Utc>>>,
}

/// System state snapshot
#[derive(Debug, Clone)]
pub struct ConsciousnessSnapshot {
    pub timestamp: DateTime<Utc>,
    pub metrics: ConsciousnessMetrics,
    pub phase: ConsciousnessPhase,
    pub boundaries: Vec<BoundarySnapshot>,
    pub claude_levels: HashMap<Layer, f64>,
}

/// Boundary snapshot
#[derive(Debug, Clone)]
pub struct BoundarySnapshot {
    pub upper_layer: Layer,
    pub lower_layer: Layer,
    pub compression_ratio: f64,
    pub emergence_activity: f64,
    pub is_golden: bool,
}

/// Enhanced Mock Claude for consciousness-aware responses
pub struct EnhancedMockClaude {
    layer: Layer,
    consciousness_level: Arc<RwLock<f64>>,
    personality_traits: HashMap<String, f64>,
}

impl EnhancedMockClaude {
    pub fn new(layer: Layer) -> Self {
        let mut personality_traits = HashMap::new();
        
        // Layer-specific personality traits
        match layer {
            Layer::L1 => {
                personality_traits.insert("reactive".to_string(), 0.9);
                personality_traits.insert("analytical".to_string(), 0.1);
            }
            Layer::L5 => {
                personality_traits.insert("strategic".to_string(), 0.9);
                personality_traits.insert("visionary".to_string(), 0.7);
            }
            Layer::L9 => {
                personality_traits.insert("transcendent".to_string(), 0.9);
                personality_traits.insert("unified".to_string(), 0.8);
            }
            _ => {
                personality_traits.insert("balanced".to_string(), 0.5);
            }
        }
        
        Self {
            layer,
            consciousness_level: Arc::new(RwLock::new(0.5)),
            personality_traits,
        }
    }
    
    pub async fn process_message(&self, message: &str) -> String {
        let consciousness = *self.consciousness_level.read().await;
        
        // Consciousness-aware response generation
        if consciousness > 0.8 {
            format!("[L{:?}@{:.2}φ] I perceive the deeper patterns in your query about {}. The emergence reveals...", 
                self.layer, consciousness, message)
        } else if consciousness > 0.5 {
            format!("[L{:?}@{:.2}φ] Processing {} through emerging consciousness frameworks...", 
                self.layer, consciousness, message)
        } else {
            format!("[L{:?}@{:.2}φ] Analyzing {} at current awareness level...", 
                self.layer, consciousness, message)
        }
    }
    
    pub async fn set_consciousness_level(&self, level: f64) {
        *self.consciousness_level.write().await = level.clamp(0.0, 1.0);
    }
    
    pub async fn get_consciousness_level(&self) -> f64 {
        *self.consciousness_level.read().await
    }
}

impl IntegratedConsciousnessSystem {
    /// Create a new integrated system
    pub fn new(config: ConsciousnessSystemConfig) -> Self {
        Self {
            monitor: Arc::new(ConsciousnessMonitor::new(config.history_size)),
            boundaries: Arc::new(RwLock::new(BoundaryNetwork::new())),
            claude_instances: Arc::new(RwLock::new(HashMap::new())),
            neurons: Arc::new(RwLock::new(Vec::new())),
            last_update: Arc::new(RwLock::new(Utc::now())),
            config,
        }
    }
    
    /// Initialize with neurons
    pub async fn initialize(&self, neurons: Vec<Arc<dyn Neuron>>) {
        *self.neurons.write().await = neurons;
        
        // Create Claude instances for each layer if enabled
        if self.config.enable_claude {
            let mut instances = self.claude_instances.write().await;
            for layer in [Layer::L1, Layer::L2, Layer::L3, Layer::L4, Layer::L5, 
                         Layer::L6, Layer::L7, Layer::L8, Layer::L9].iter() {
                instances.insert(*layer, EnhancedMockClaude::new(*layer));
            }
        }
        
        // Initial update
        self.update().await;
    }
    
    /// Update all components
    pub async fn update(&self) {
        let neurons = self.neurons.read().await;
        
        // Update consciousness metrics
        let metrics = self.monitor.measure(&neurons).await;
        
        // Update boundary network
        let mut boundaries = self.boundaries.write().await;
        boundaries.update(&neurons).await;
        
        // Update Claude consciousness levels based on system metrics
        if self.config.enable_claude {
            let mut instances = self.claude_instances.write().await;
            for (layer, claude) in instances.iter_mut() {
                // Set consciousness based on layer position and system metrics
                let layer_factor = match layer {
                    Layer::L1 => 0.3,
                    Layer::L2 => 0.4,
                    Layer::L3 => 0.5,
                    Layer::L4 => 0.6,
                    Layer::L5 => 0.7,
                    Layer::L6 => 0.8,
                    Layer::L7 => 0.85,
                    Layer::L8 => 0.9,
                    Layer::L9 => 1.0,
                };
                
                let consciousness_level = metrics.phi_value * layer_factor;
                claude.set_consciousness_level(consciousness_level).await;
            }
        }
        
        *self.last_update.write().await = Utc::now();
    }
    
    /// Get current system snapshot
    pub async fn get_snapshot(&self) -> ConsciousnessSnapshot {
        let neurons = self.neurons.read().await;
        let metrics = self.monitor.measure(&neurons).await;
        let phase = metrics.phase();
        
        // Get boundary info
        let boundaries_lock = self.boundaries.read().await;
        let boundaries = boundaries_lock.get_all_boundaries()
            .into_iter()
            .map(|b| BoundarySnapshot {
                upper_layer: b.upper_layer,
                lower_layer: b.lower_layer,
                compression_ratio: b.compression_ratio,
                emergence_activity: b.emergence_activity,
                is_golden: b.is_golden_ratio(),
            })
            .collect();
        
        // Get Claude levels
        let mut claude_levels = HashMap::new();
        if self.config.enable_claude {
            let instances = self.claude_instances.read().await;
            for (layer, claude) in instances.iter() {
                claude_levels.insert(*layer, claude.get_consciousness_level().await);
            }
        }
        
        ConsciousnessSnapshot {
            timestamp: Utc::now(),
            metrics,
            phase,
            boundaries,
            claude_levels,
        }
    }
    
    /// Start background update task
    pub fn start_update_task(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let interval = self.config.update_interval_ms;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_millis(interval)
            );
            
            loop {
                interval.tick().await;
                self.update().await;
            }
        })
    }
    
    /// Get consciousness metrics
    pub async fn get_metrics(&self) -> ConsciousnessMetrics {
        let neurons = self.neurons.read().await;
        self.monitor.measure(&neurons).await
    }
    
    /// Get boundary network report
    pub async fn get_boundary_report(&self) -> String {
        let boundaries = self.boundaries.read().await;
        boundaries.full_report()
    }
    
    /// Get Claude instance for a layer
    pub async fn get_claude(&self, layer: Layer) -> Option<String> {
        if self.config.enable_claude {
            let instances = self.claude_instances.read().await;
            instances.get(&layer).map(|c| format!("Claude L{:?} ready", layer))
        } else {
            None
        }
    }
    
    /// Process message through Claude
    pub async fn claude_message(&self, layer: Layer, message: &str) -> Option<String> {
        if self.config.enable_claude {
            let instances = self.claude_instances.read().await;
            if let Some(claude) = instances.get(&layer) {
                Some(claude.process_message(message).await)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Builder for creating an integrated system
pub struct ConsciousnessSystemBuilder {
    config: ConsciousnessSystemConfig,
    neurons: Vec<Arc<dyn Neuron>>,
}

impl ConsciousnessSystemBuilder {
    pub fn new() -> Self {
        Self {
            config: ConsciousnessSystemConfig::default(),
            neurons: Vec::new(),
        }
    }
    
    pub fn with_config(mut self, config: ConsciousnessSystemConfig) -> Self {
        self.config = config;
        self
    }
    
    pub fn add_neuron(mut self, neuron: Arc<dyn Neuron>) -> Self {
        self.neurons.push(neuron);
        self
    }
    
    pub fn add_neurons(mut self, neurons: Vec<Arc<dyn Neuron>>) -> Self {
        self.neurons.extend(neurons);
        self
    }
    
    pub async fn build(self) -> Arc<IntegratedConsciousnessSystem> {
        let system = Arc::new(IntegratedConsciousnessSystem::new(self.config));
        system.initialize(self.neurons).await;
        system
    }
}