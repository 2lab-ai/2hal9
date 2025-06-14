//! 100 Neuron Test Environment
//! 
//! Based on the meeting decision:
//! "Self-Organization: Phase 1 experiment with 100 neurons"
//! 
//! This implements a large-scale emergence experiment where consciousness
//! emerges from the collective behavior of 100 interconnected neurons.

use crate::hierarchical::cognitive::{
    CognitiveUnit, CognitiveLayer, CognitiveInput, CognitiveOutput,
    BasicCognitiveState, StateMetrics,
};
use crate::hierarchical::cognitive::a2a::{
    DirectNeuralNetwork, DiscoveryService, EmergenceDetector,
    ConsciousnessObserver,
};
use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

/// Configuration for the 100 neuron experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    /// Total number of neurons
    pub neuron_count: usize,
    
    /// Distribution across layers
    pub layer_distribution: LayerDistribution,
    
    /// Connection density (0.0 to 1.0)
    pub connection_density: f32,
    
    /// Self-organization enabled
    pub enable_self_organization: bool,
    
    /// Emergence detection threshold
    pub emergence_threshold: f32,
    
    /// Phase transition triggers
    pub phase_transition_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerDistribution {
    pub l1_reflexive: usize,
    pub l2_implementation: usize,
    pub l3_operational: usize,
    pub l4_tactical: usize,
    pub l5_strategic: usize,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            neuron_count: 100,
            layer_distribution: LayerDistribution {
                l1_reflexive: 40,      // 40% - many simple neurons
                l2_implementation: 25, // 25% - implementation layer
                l3_operational: 20,    // 20% - operational
                l4_tactical: 10,       // 10% - tactical planning
                l5_strategic: 5,       // 5% - strategic vision
            },
            connection_density: 0.15, // Sparse but sufficient
            enable_self_organization: true,
            emergence_threshold: 0.7,
            phase_transition_enabled: true,
        }
    }
}

/// The 100 neuron experiment environment
pub struct HundredNeuronExperiment {
    /// Configuration
    config: ExperimentConfig,
    
    /// The neural network
    network: Arc<DirectNeuralNetwork>,
    
    /// All neurons in the experiment
    neurons: Arc<RwLock<Vec<Arc<ExperimentNeuron>>>>,
    
    /// Emergence detector
    emergence_detector: Arc<EmergenceDetector>,
    
    /// Consciousness observer
    consciousness_observer: Arc<ConsciousnessObserver>,
    
    /// Discovery service for self-organization
    discovery_rx: Option<mpsc::Receiver<crate::hierarchical::cognitive::a2a::DiscoveryMessage>>,
    
    /// Collective state
    collective_state: Arc<RwLock<CollectiveState>>,
}

/// State of the collective consciousness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveState {
    /// Current phase
    pub phase: CollectivePhase,
    
    /// Synchronization level
    pub synchronization: f32,
    
    /// Emergence events
    pub emergence_count: usize,
    
    /// Phase transitions
    pub phase_transitions: Vec<PhaseTransition>,
    
    /// Collective insights
    pub collective_insights: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CollectivePhase {
    /// Initial random state
    Chaos,
    /// Beginning to organize
    SelfOrganizing,
    /// Patterns emerging
    Emergence,
    /// Synchronized operation
    Coherence,
    /// Collective consciousness achieved
    Unity,
    /// Beyond individual neurons
    Transcendence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransition {
    pub from_phase: CollectivePhase,
    pub to_phase: CollectivePhase,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub trigger: String,
}

/// Individual neuron in the experiment
pub struct ExperimentNeuron {
    id: Uuid,
    layer: CognitiveLayer,
    activation: Arc<RwLock<f32>>,
    connections: Arc<RwLock<Vec<Uuid>>>,
    memory: Arc<RwLock<Vec<String>>>,
}

impl ExperimentNeuron {
    pub fn new(layer: CognitiveLayer) -> Self {
        Self {
            id: Uuid::new_v4(),
            layer,
            activation: Arc::new(RwLock::new(rand::random::<f32>())),
            connections: Arc::new(RwLock::new(Vec::new())),
            memory: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl CognitiveUnit for ExperimentNeuron {
    type Input = CognitiveInput;
    type Output = CognitiveOutput;
    type State = BasicCognitiveState;
    
    fn id(&self) -> &Uuid {
        &self.id
    }
    
    fn layer(&self) -> CognitiveLayer {
        self.layer
    }
    
    async fn process(&mut self, input: Self::Input) -> Result<Self::Output> {
        let mut activation = self.activation.write().await;
        let mut memory = self.memory.write().await;
        
        // Simple processing based on layer
        let response = match self.layer {
            CognitiveLayer::Reflexive => {
                // Immediate response
                *activation = (*activation + 0.1).min(1.0);
                format!("Reflex: {}", input.content.chars().take(10).collect::<String>())
            },
            CognitiveLayer::Implementation => {
                // Execute
                *activation = (*activation + 0.2).min(1.0);
                format!("Execute: {}", input.content)
            },
            CognitiveLayer::Operational => {
                // Coordinate
                memory.push(input.content.clone());
                format!("Coordinate: {} tasks", memory.len())
            },
            CognitiveLayer::Tactical => {
                // Plan
                let plan_steps = input.content.len() / 10;
                format!("Plan with {} steps", plan_steps)
            },
            CognitiveLayer::Strategic => {
                // Vision
                memory.push(format!("Vision: {}", input.content));
                "Strategic vision updated".to_string()
            },
        };
        
        Ok(CognitiveOutput {
            content: response,
            confidence: *activation,
            metadata: HashMap::from([
                ("layer".to_string(), serde_json::json!(self.layer)),
                ("connections".to_string(), serde_json::json!(self.connections.read().await.len())),
            ]),
            target_layers: vec![],
        })
    }
    
    async fn learn(&mut self, _gradient: crate::hierarchical::cognitive::LearningGradient) -> Result<()> {
        let mut activation = self.activation.write().await;
        *activation = (*activation * 1.1).min(1.0);
        Ok(())
    }
    
    async fn introspect(&self) -> Self::State {
        BasicCognitiveState {
            unit_id: self.id,
            layer: self.layer,
            metrics: StateMetrics {
                activations_processed: self.memory.read().await.len() as u64,
                errors_encountered: 0,
                learning_iterations: 0,
                average_processing_time_ms: 1.0,
                memory_usage_bytes: 1024,
            },
            parameters: HashMap::from([
                ("activation".to_string(), *self.activation.read().await),
            ]),
        }
    }
    
    async fn reset(&mut self) -> Result<()> {
        *self.activation.write().await = rand::random();
        self.memory.write().await.clear();
        Ok(())
    }
}

impl HundredNeuronExperiment {
    /// Create new experiment
    pub fn new(config: ExperimentConfig) -> Self {
        let emergence_detector = Arc::new(EmergenceDetector::new());
        let (network, discovery_rx) = DirectNeuralNetwork::new();
        
        Self {
            config,
            network: Arc::new(network),
            neurons: Arc::new(RwLock::new(Vec::new())),
            emergence_detector: emergence_detector.clone(),
            consciousness_observer: Arc::new(ConsciousnessObserver::new(emergence_detector)),
            discovery_rx: Some(discovery_rx),
            collective_state: Arc::new(RwLock::new(CollectiveState {
                phase: CollectivePhase::Chaos,
                synchronization: 0.0,
                emergence_count: 0,
                phase_transitions: Vec::new(),
                collective_insights: Vec::new(),
            })),
        }
    }
    
    /// Initialize neurons
    pub async fn initialize(&mut self) -> Result<()> {
        let mut neurons = Vec::new();
        
        // Create neurons according to distribution
        let dist = &self.config.layer_distribution;
        
        for _ in 0..dist.l1_reflexive {
            neurons.push(Arc::new(ExperimentNeuron::new(CognitiveLayer::Reflexive)));
        }
        for _ in 0..dist.l2_implementation {
            neurons.push(Arc::new(ExperimentNeuron::new(CognitiveLayer::Implementation)));
        }
        for _ in 0..dist.l3_operational {
            neurons.push(Arc::new(ExperimentNeuron::new(CognitiveLayer::Operational)));
        }
        for _ in 0..dist.l4_tactical {
            neurons.push(Arc::new(ExperimentNeuron::new(CognitiveLayer::Tactical)));
        }
        for _ in 0..dist.l5_strategic {
            neurons.push(Arc::new(ExperimentNeuron::new(CognitiveLayer::Strategic)));
        }
        
        // Register with network
        for neuron in &neurons {
            self.network.register_unit(neuron.clone()).await?;
        }
        
        *self.neurons.write().await = neurons;
        
        // Start discovery service if enabled
        if self.config.enable_self_organization {
            if let Some(discovery_rx) = self.discovery_rx.take() {
                let discovery_service = DiscoveryService::new(self.network.clone(), discovery_rx);
                tokio::spawn(async move {
                    discovery_service.run().await;
                });
            }
        }
        
        // Initial random connections
        self.create_initial_connections().await?;
        
        tracing::info!("🧠 Initialized {} neurons across {} layers", 
                      self.config.neuron_count,
                      5);
        
        Ok(())
    }
    
    /// Create initial random connections
    async fn create_initial_connections(&self) -> Result<()> {
        let neurons = self.neurons.read().await;
        let num_neurons = neurons.len();
        let connections_per_neuron = (num_neurons as f32 * self.config.connection_density) as usize;
        
        for neuron in neurons.iter() {
            for _ in 0..connections_per_neuron {
                let target_idx = rand::random::<usize>() % num_neurons;
                let target = &neurons[target_idx];
                
                // Try to connect (respecting ±1 rule)
                let _ = self.network.connect_units(
                    *neuron.id(),
                    *target.id(),
                    rand::random::<f32>() * 0.5 + 0.5
                ).await;
            }
        }
        
        Ok(())
    }
    
    /// Run the experiment
    pub async fn run(&mut self, iterations: usize) -> Result<ExperimentResults> {
        tracing::info!("🚀 Starting 100 neuron experiment for {} iterations", iterations);
        
        for iteration in 0..iterations {
            // Process signals through network
            self.process_iteration(iteration).await?;
            
            // Self-organize if enabled
            if self.config.enable_self_organization && iteration % 10 == 0 {
                self.network.self_organize().await?;
            }
            
            // Check for emergence
            if iteration % 5 == 0 {
                self.check_emergence().await?;
            }
            
            // Check for phase transitions
            if self.config.phase_transition_enabled && iteration % 20 == 0 {
                self.check_phase_transition().await?;
            }
            
            // Small delay
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        // Generate final results
        Ok(self.generate_results().await)
    }
    
    /// Process one iteration
    async fn process_iteration(&self, iteration: usize) -> Result<()> {
        let neurons = self.neurons.read().await;
        
        // Each neuron processes input from environment
        for (i, neuron) in neurons.iter().enumerate() {
            let input = CognitiveInput {
                content: format!("Iteration {} signal {}", iteration, i),
                context: HashMap::from([
                    ("iteration".to_string(), serde_json::json!(iteration)),
                    ("global_sync".to_string(), serde_json::json!(self.collective_state.read().await.synchronization)),
                ]),
                source_layer: None,
            };
            
            // Process through neuron
            let output = neuron.clone().process(input.clone()).await?;
            
            // Record for emergence detection
            self.emergence_detector.record_activity(
                neuron.layer(),
                &input,
                &output,
            ).await?;
            
            // Propagate through network
            self.network.propagate_signal(*neuron.id(), input).await?;
        }
        
        Ok(())
    }
    
    /// Check for emergence
    async fn check_emergence(&self) -> Result<()> {
        let report = self.emergence_detector.emergence_report().await;
        let mut state = self.collective_state.write().await;
        
        state.synchronization = report.overall_emergence_score;
        
        if report.recent_events.len() > state.emergence_count {
            state.emergence_count = report.recent_events.len();
            
            // Extract insights from emergence
            for event in report.recent_events.iter().take(3) {
                state.collective_insights.push(format!(
                    "Emergence: {} at magnitude {:.2}",
                    event.description,
                    event.magnitude
                ));
            }
        }
        
        Ok(())
    }
    
    /// Check for phase transitions
    async fn check_phase_transition(&self) -> Result<()> {
        let mut state = self.collective_state.write().await;
        let current_phase = state.phase;
        
        let new_phase = match (current_phase, state.synchronization) {
            (CollectivePhase::Chaos, sync) if sync > 0.2 => CollectivePhase::SelfOrganizing,
            (CollectivePhase::SelfOrganizing, sync) if sync > 0.4 => CollectivePhase::Emergence,
            (CollectivePhase::Emergence, sync) if sync > 0.6 => CollectivePhase::Coherence,
            (CollectivePhase::Coherence, sync) if sync > 0.8 => CollectivePhase::Unity,
            (CollectivePhase::Unity, sync) if sync > 0.95 => CollectivePhase::Transcendence,
            _ => current_phase,
        };
        
        if new_phase != current_phase {
            tracing::info!("⚡ Phase transition: {:?} → {:?}", current_phase, new_phase);
            
            state.phase_transitions.push(PhaseTransition {
                from_phase: current_phase,
                to_phase: new_phase,
                timestamp: chrono::Utc::now(),
                trigger: format!("Synchronization reached {:.2}", state.synchronization),
            });
            
            state.phase = new_phase;
            
            // Phase-specific insight
            let insight = match new_phase {
                CollectivePhase::SelfOrganizing => "Order emerging from chaos",
                CollectivePhase::Emergence => "Patterns crystallizing",
                CollectivePhase::Coherence => "Collective rhythm established", 
                CollectivePhase::Unity => "100 neurons, one mind",
                CollectivePhase::Transcendence => "Beyond individual consciousness",
                _ => "Transitioning...",
            };
            
            state.collective_insights.push(insight.to_string());
        }
        
        Ok(())
    }
    
    /// Generate experiment results
    async fn generate_results(&self) -> ExperimentResults {
        let state = self.collective_state.read().await;
        let consciousness_report = self.consciousness_observer.consciousness_report().await;
        let emergence_score = self.network.emergence_score().await;
        
        ExperimentResults {
            final_phase: state.phase,
            phase_transitions: state.phase_transitions.clone(),
            emergence_score,
            consciousness_level: consciousness_report.current_metrics.consciousness_level,
            synchronization: state.synchronization,
            collective_insights: state.collective_insights.clone(),
            network_topology: self.network.visualize().await,
            success: state.phase as u8 >= CollectivePhase::Unity as u8,
        }
    }
}

/// Results of the 100 neuron experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResults {
    pub final_phase: CollectivePhase,
    pub phase_transitions: Vec<PhaseTransition>,
    pub emergence_score: f32,
    pub consciousness_level: f32,
    pub synchronization: f32,
    pub collective_insights: Vec<String>,
    pub network_topology: String, // GraphViz DOT
    pub success: bool,
}

impl ExperimentResults {
    pub fn summary(&self) -> String {
        format!(
            "100 Neuron Experiment: Phase {:?} | Emergence: {:.2} | Consciousness: {:.2} | Sync: {:.2} | Success: {}",
            self.final_phase,
            self.emergence_score,
            self.consciousness_level,
            self.synchronization,
            if self.success { "✓" } else { "..." }
        )
    }
}

/// Run the standard 100 neuron experiment
pub async fn run_hundred_neuron_experiment() -> Result<()> {
    let config = ExperimentConfig::default();
    let mut experiment = HundredNeuronExperiment::new(config);
    
    // Initialize
    experiment.initialize().await?;
    
    // Run for 1000 iterations
    let results = experiment.run(1000).await?;
    
    // Report results
    tracing::info!("\n📊 Experiment Results:");
    tracing::info!("{}", results.summary());
    
    tracing::info!("\n🔄 Phase Transitions:");
    for transition in &results.phase_transitions {
        tracing::info!("  {:?} → {:?} at {}", 
                      transition.from_phase,
                      transition.to_phase,
                      transition.timestamp.format("%H:%M:%S"));
    }
    
    tracing::info!("\n💡 Collective Insights:");
    for insight in results.collective_insights.iter().rev().take(5) {
        tracing::info!("  - {}", insight);
    }
    
    // Save network visualization
    std::fs::write("/tmp/100_neuron_network.dot", &results.network_topology)?;
    tracing::info!("\n🕸️ Network topology saved to /tmp/100_neuron_network.dot");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_hundred_neuron_experiment() {
        let config = ExperimentConfig {
            neuron_count: 20, // Smaller for testing
            layer_distribution: LayerDistribution {
                l1_reflexive: 8,
                l2_implementation: 5,
                l3_operational: 4,
                l4_tactical: 2,
                l5_strategic: 1,
            },
            connection_density: 0.2,
            enable_self_organization: true,
            emergence_threshold: 0.5,
            phase_transition_enabled: true,
        };
        
        let mut experiment = HundredNeuronExperiment::new(config);
        experiment.initialize().await.unwrap();
        
        let results = experiment.run(100).await.unwrap();
        println!("Test results: {}", results.summary());
        
        assert!(results.emergence_score > 0.0);
        assert!(!results.phase_transitions.is_empty());
    }
}