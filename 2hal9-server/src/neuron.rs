//! Neuron management and registry

use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use chrono::Utc;

use 2hal9_core::{
    Error, Result, NeuronSignal, Layer, NeuronInterface, NeuronState, NeuronHealth,
    NeuronConfig, PropagationType, SignalPayload, Activation, Gradient,
};

use crate::claude::ClaudeInterface;

/// A managed neuron that wraps a Claude instance
pub struct ManagedNeuron {
    pub id: String,
    pub layer: Layer,
    pub config: NeuronConfig,
    claude: Box<dyn ClaudeInterface>,
    state: RwLock<NeuronState>,
    stats: RwLock<NeuronStats>,
}

#[derive(Default)]
struct NeuronStats {
    signals_processed: u64,
    errors_count: u64,
    last_signal: Option<chrono::DateTime<chrono::Utc>>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ManagedNeuron {
    /// Create a new managed neuron
    pub fn new(
        config: NeuronConfig,
        claude: Box<dyn ClaudeInterface>,
    ) -> Result<Self> {
        let layer = Layer::from_str(&config.layer)
            .ok_or_else(|| Error::Config(format!("Invalid layer: {}", config.layer)))?;
            
        Ok(Self {
            id: config.id.clone(),
            layer,
            config,
            claude,
            state: RwLock::new(NeuronState::Starting),
            stats: RwLock::new(NeuronStats {
                started_at: Some(Utc::now()),
                ..Default::default()
            }),
        })
    }
    
    /// Start the neuron
    pub async fn start(&self) -> Result<()> {
        info!("Starting neuron {} on layer {}", self.id, self.layer.as_str());
        *self.state.write().await = NeuronState::Running;
        Ok(())
    }
    
    /// Format a signal into a prompt for Claude
    fn format_prompt(&self, signal: &NeuronSignal) -> String {
        match signal.propagation_type {
            PropagationType::Forward => {
                format!(
                    "FORWARD_SIGNAL\nFrom: {}\nLayer: {}\nStrength: {}\nContent: {}\nFeatures: {:?}\n",
                    signal.from_neuron,
                    signal.layer_from,
                    signal.payload.activation.strength,
                    signal.payload.activation.content,
                    signal.payload.activation.features
                )
            }
            PropagationType::Backward => {
                let gradient = signal.payload.gradient.as_ref().unwrap();
                format!(
                    "BACKWARD_SIGNAL\nFrom: {}\nError: {}\nMagnitude: {}\nLoss: {}\nAdjustments: {:?}\n",
                    signal.from_neuron,
                    gradient.error_type,
                    gradient.magnitude,
                    gradient.loss,
                    gradient.adjustments
                )
            }
        }
    }
    
    /// Parse response and determine next signals
    pub fn parse_response(&self, response: &str, original_signal: &NeuronSignal) -> Vec<NeuronSignal> {
        let mut signals = Vec::new();
        
        // Parse FORWARD_TO directive
        if let Some(forward_line) = response.lines().find(|l| l.starts_with("FORWARD_TO:")) {
            let targets = forward_line
                .strip_prefix("FORWARD_TO:")
                .unwrap_or("")
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty());
                
            // Extract content after CONTENT: line
            let content = response.lines()
                .skip_while(|l| !l.starts_with("CONTENT:"))
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n");
                
            for target in targets {
                if self.config.forward_connections.contains(&target.to_string()) {
                    signals.push(NeuronSignal::forward(
                        &self.id,
                        target,
                        self.layer.as_str(),
                        &self.get_target_layer(target),
                        content.clone(),
                    ));
                }
            }
        }
        
        // Parse BACKWARD_TO directive for error propagation
        if let Some(backward_line) = response.lines().find(|l| l.starts_with("BACKWARD_TO:")) {
            let targets = backward_line
                .strip_prefix("BACKWARD_TO:")
                .unwrap_or("")
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty());
                
            let error_type = response.lines()
                .find(|l| l.starts_with("ERROR_TYPE:"))
                .and_then(|l| l.strip_prefix("ERROR_TYPE:"))
                .unwrap_or("Unknown")
                .trim()
                .to_string();
                
            for target in targets {
                if self.config.backward_connections.contains(&target.to_string()) {
                    signals.push(NeuronSignal::backward(
                        &self.id,
                        target,
                        self.layer.as_str(),
                        &self.get_target_layer(target),
                        Gradient::new(error_type.clone(), 0.5),
                    ));
                }
            }
        }
        
        signals
    }
    
    /// Infer target layer from neuron ID (simplified)
    fn get_target_layer(&self, target_id: &str) -> String {
        // In a real implementation, this would look up the target's actual layer
        match self.layer {
            Layer::L4 => "L3",
            Layer::L3 => "L2", 
            Layer::L2 => "L1",
            Layer::L1 => "L1",
        }.to_string()
    }
}

#[async_trait]
impl NeuronInterface for ManagedNeuron {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn layer(&self) -> Layer {
        self.layer
    }
    
    async fn process_signal(&self, signal: &NeuronSignal) -> Result<String> {
        // Update state
        *self.state.write().await = NeuronState::Processing;
        
        // Format prompt
        let prompt = self.format_prompt(signal);
        debug!("Neuron {} processing signal: {}", self.id, signal.signal_id);
        
        // Send to Claude
        let response = match self.claude.send_message(&prompt).await {
            Ok(resp) => {
                // Update stats
                let mut stats = self.stats.write().await;
                stats.signals_processed += 1;
                stats.last_signal = Some(Utc::now());
                drop(stats);
                
                resp
            }
            Err(e) => {
                // Update error stats
                let mut stats = self.stats.write().await;
                stats.errors_count += 1;
                drop(stats);
                
                error!("Neuron {} failed to process signal: {}", self.id, e);
                return Err(e);
            }
        };
        
        // Return to running state
        *self.state.write().await = NeuronState::Running;
        
        Ok(response)
    }
    
    async fn health(&self) -> Result<NeuronHealth> {
        let state = *self.state.read().await;
        let stats = self.stats.read().await;
        
        let uptime_seconds = stats.started_at
            .map(|start| (Utc::now() - start).num_seconds() as u64)
            .unwrap_or(0);
            
        Ok(NeuronHealth {
            state,
            last_signal: stats.last_signal,
            signals_processed: stats.signals_processed,
            errors_count: stats.errors_count,
            uptime_seconds,
        })
    }
    
    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down neuron {}", self.id);
        *self.state.write().await = NeuronState::Stopped;
        Ok(())
    }
}

/// Registry for managing multiple neurons
pub struct NeuronRegistry {
    neurons: Arc<DashMap<String, Arc<ManagedNeuron>>>,
}

impl NeuronRegistry {
    /// Create a new neuron registry
    pub fn new() -> Self {
        Self {
            neurons: Arc::new(DashMap::new()),
        }
    }
    
    /// Register a neuron
    pub async fn register(&self, neuron: ManagedNeuron) -> Result<()> {
        let id = neuron.id.clone();
        neuron.start().await?;
        self.neurons.insert(id.clone(), Arc::new(neuron));
        info!("Registered neuron: {}", id);
        Ok(())
    }
    
    /// Get a neuron by ID
    pub fn get(&self, id: &str) -> Option<Arc<ManagedNeuron>> {
        self.neurons.get(id).map(|entry| entry.clone())
    }
    
    /// Get all neurons
    pub fn all(&self) -> Vec<Arc<ManagedNeuron>> {
        self.neurons.iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    /// Get neurons by layer
    pub fn by_layer(&self, layer: Layer) -> Vec<Arc<ManagedNeuron>> {
        self.neurons.iter()
            .filter(|entry| entry.value().layer == layer)
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    /// Remove a neuron
    pub async fn remove(&self, id: &str) -> Result<()> {
        if let Some((_, neuron)) = self.neurons.remove(id) {
            neuron.shutdown().await?;
            info!("Removed neuron: {}", id);
        }
        Ok(())
    }
    
    /// Shutdown all neurons
    pub async fn shutdown_all(&self) -> Result<()> {
        info!("Shutting down all neurons");
        let neurons: Vec<_> = self.all();
        
        for neuron in neurons {
            if let Err(e) = neuron.shutdown().await {
                warn!("Error shutting down neuron {}: {}", neuron.id(), e);
            }
        }
        
        self.neurons.clear();
        Ok(())
    }
    
    /// Health check all neurons
    pub async fn health_check(&self) -> HashMap<String, NeuronHealth> {
        let mut health_map = HashMap::new();
        
        for neuron in self.all() {
            match neuron.health().await {
                Ok(health) => {
                    health_map.insert(neuron.id().to_string(), health);
                }
                Err(e) => {
                    warn!("Failed to get health for neuron {}: {}", neuron.id(), e);
                }
            }
        }
        
        health_map
    }
}

use std::collections::HashMap;