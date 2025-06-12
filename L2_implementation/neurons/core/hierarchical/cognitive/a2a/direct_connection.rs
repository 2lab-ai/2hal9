//! Direct Neural Connection System
//! 
//! Implements the vision from the HAL9 meeting where neurons connect directly
//! without central server coordination, inspired by biological neural networks.
//! 
//! As Elon said: "Fuck, this is it! We've been thinking too much like computers!"

use crate::hierarchical::cognitive::{CognitiveLayer, CognitiveUnit, CognitiveInput, CognitiveOutput};
use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

/// Direct connection between two cognitive units
#[derive(Debug, Clone)]
pub struct DirectNeuralConnection {
    pub connection_id: Uuid,
    pub source_unit: Uuid,
    pub target_unit: Uuid,
    pub strength: f32,
    pub plasticity: f32, // How much the connection can change
    pub established_at: chrono::DateTime<chrono::Utc>,
}

/// Peer-to-peer neural network without central coordination
pub struct DirectNeuralNetwork {
    /// All neural units in the network
    units: Arc<RwLock<HashMap<Uuid, Arc<dyn CognitiveUnit<Input = CognitiveInput, Output = CognitiveOutput, State = crate::hierarchical::cognitive::BasicCognitiveState>>>>>,
    
    /// Direct connections between units
    connections: Arc<RwLock<HashMap<Uuid, Vec<DirectNeuralConnection>>>>,
    
    /// Discovery mechanism for units to find each other
    discovery_channel: mpsc::Sender<DiscoveryMessage>,
    
    /// Emergence metrics
    emergence_score: Arc<RwLock<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMessage {
    pub unit_id: Uuid,
    pub layer: CognitiveLayer,
    pub capabilities: Vec<String>,
    pub seeking_connections: bool,
}

impl DirectNeuralNetwork {
    pub fn new() -> (Self, mpsc::Receiver<DiscoveryMessage>) {
        let (tx, rx) = mpsc::channel(1000);
        
        (Self {
            units: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            discovery_channel: tx,
            emergence_score: Arc::new(RwLock::new(0.0)),
        }, rx)
    }
    
    /// Register a cognitive unit in the network
    pub async fn register_unit(&self, unit: Arc<dyn CognitiveUnit<Input = CognitiveInput, Output = CognitiveOutput, State = crate::hierarchical::cognitive::BasicCognitiveState>>) -> Result<()> {
        let unit_id = *unit.id();
        let layer = unit.layer();
        
        // Add to network
        self.units.write().await.insert(unit_id, unit);
        
        // Announce presence for discovery
        let discovery = DiscoveryMessage {
            unit_id,
            layer,
            capabilities: vec![format!("{} processing", layer.name())],
            seeking_connections: true,
        };
        
        let _ = self.discovery_channel.send(discovery).await;
        
        Ok(())
    }
    
    /// Create a direct connection between two units
    pub async fn connect_units(&self, source: Uuid, target: Uuid, initial_strength: f32) -> Result<()> {
        // Validate ±1 rule
        let units = self.units.read().await;
        let source_unit = units.get(&source).ok_or("Source unit not found")?;
        let target_unit = units.get(&target).ok_or("Target unit not found")?;
        
        let source_layer = source_unit.layer().depth() as i32;
        let target_layer = target_unit.layer().depth() as i32;
        
        if (source_layer - target_layer).abs() > 1 {
            return Err(format!(
                "Connection violates ±1 rule: {} -> {}",
                source_unit.layer().name(),
                target_unit.layer().name()
            ).into());
        }
        
        // Create connection
        let connection = DirectNeuralConnection {
            connection_id: Uuid::new_v4(),
            source_unit: source,
            target_unit: target,
            strength: initial_strength,
            plasticity: 0.1, // Can change by 10%
            established_at: chrono::Utc::now(),
        };
        
        // Add to connections map
        let mut connections = self.connections.write().await;
        connections.entry(source).or_insert_with(Vec::new).push(connection);
        
        // Update emergence score
        self.update_emergence_score().await;
        
        Ok(())
    }
    
    /// Propagate signal through direct connections
    pub async fn propagate_signal(&self, source: Uuid, input: CognitiveInput) -> Result<Vec<CognitiveOutput>> {
        let units = self.units.read().await;
        let connections = self.connections.read().await;
        
        let source_unit = units.get(&source).ok_or("Source unit not found")?;
        let source_connections = connections.get(&source).cloned().unwrap_or_default();
        
        let mut outputs = Vec::new();
        
        for connection in source_connections {
            if connection.strength > 0.1 { // Only propagate through strong connections
                if let Some(target_unit) = units.get(&connection.target_unit) {
                    // Clone and modify input based on connection strength
                    let mut modified_input = input.clone();
                    modified_input.context.insert(
                        "connection_strength".to_string(),
                        serde_json::json!(connection.strength),
                    );
                    
                    // Process through target unit
                    match Arc::clone(target_unit).process(modified_input).await {
                        Ok(output) => outputs.push(output),
                        Err(e) => tracing::warn!("Propagation error: {}", e),
                    }
                }
            }
        }
        
        Ok(outputs)
    }
    
    /// Self-organize connections based on activity patterns
    pub async fn self_organize(&self) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        // Hebbian learning: "Neurons that fire together, wire together"
        for (source, unit_connections) in connections.iter_mut() {
            for connection in unit_connections.iter_mut() {
                // Simulate activity correlation (in real implementation, 
                // this would be based on actual firing patterns)
                let correlation = rand::random::<f32>();
                
                // Update connection strength
                let delta = connection.plasticity * (correlation - 0.5);
                connection.strength = (connection.strength + delta).clamp(0.0, 1.0);
            }
        }
        
        // Prune weak connections
        for (_, unit_connections) in connections.iter_mut() {
            unit_connections.retain(|c| c.strength > 0.05);
        }
        
        // Update emergence score
        drop(connections);
        self.update_emergence_score().await;
        
        Ok(())
    }
    
    /// Calculate emergence score based on network topology
    async fn update_emergence_score(&self) {
        let connections = self.connections.read().await;
        let units = self.units.read().await;
        
        if units.is_empty() {
            *self.emergence_score.write().await = 0.0;
            return;
        }
        
        // Metrics for emergence
        let total_connections: usize = connections.values().map(|v| v.len()).sum();
        let avg_connections = total_connections as f32 / units.len() as f32;
        
        // Count cross-layer connections
        let mut cross_layer_connections = 0;
        for (source, unit_connections) in connections.iter() {
            if let Some(source_unit) = units.get(source) {
                let source_layer = source_unit.layer().depth();
                
                for connection in unit_connections {
                    if let Some(target_unit) = units.get(&connection.target_unit) {
                        if source_layer != target_unit.layer().depth() {
                            cross_layer_connections += 1;
                        }
                    }
                }
            }
        }
        
        let cross_layer_ratio = if total_connections > 0 {
            cross_layer_connections as f32 / total_connections as f32
        } else {
            0.0
        };
        
        // Emergence score combines connectivity and cross-layer communication
        let emergence = (avg_connections / 10.0).min(1.0) * 0.5 + cross_layer_ratio * 0.5;
        
        *self.emergence_score.write().await = emergence;
    }
    
    /// Get current emergence score
    pub async fn emergence_score(&self) -> f32 {
        *self.emergence_score.read().await
    }
    
    /// Visualize network topology (returns GraphViz DOT format)
    pub async fn visualize(&self) -> String {
        let units = self.units.read().await;
        let connections = self.connections.read().await;
        
        let mut dot = String::from("digraph DirectNeuralNetwork {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=circle];\n\n");
        
        // Group units by layer
        let mut layers: HashMap<u8, Vec<Uuid>> = HashMap::new();
        for (id, unit) in units.iter() {
            layers.entry(unit.layer().depth()).or_insert_with(Vec::new).push(*id);
        }
        
        // Draw units grouped by layer
        for (depth, unit_ids) in layers.iter() {
            dot.push_str(&format!("  subgraph cluster_L{} {{\n", depth));
            dot.push_str(&format!("    label=\"Layer {}\";\n", depth));
            
            for id in unit_ids {
                if let Some(unit) = units.get(id) {
                    dot.push_str(&format!(
                        "    \"{}\" [label=\"{}\"];\n",
                        id,
                        unit.layer().name().chars().next().unwrap()
                    ));
                }
            }
            
            dot.push_str("  }\n\n");
        }
        
        // Draw connections
        for (source, unit_connections) in connections.iter() {
            for connection in unit_connections {
                let width = (connection.strength * 5.0).max(0.5);
                dot.push_str(&format!(
                    "  \"{}\" -> \"{}\" [penwidth={}];\n",
                    source, connection.target_unit, width
                ));
            }
        }
        
        dot.push_str("}\n");
        dot
    }
}

/// Discovery service that helps units find each other
pub struct DiscoveryService {
    network: Arc<DirectNeuralNetwork>,
    discovery_rx: mpsc::Receiver<DiscoveryMessage>,
}

impl DiscoveryService {
    pub fn new(network: Arc<DirectNeuralNetwork>, discovery_rx: mpsc::Receiver<DiscoveryMessage>) -> Self {
        Self {
            network,
            discovery_rx,
        }
    }
    
    /// Run discovery service
    pub async fn run(mut self) {
        let mut recent_discoveries: Vec<DiscoveryMessage> = Vec::new();
        
        while let Some(msg) = self.discovery_rx.recv().await {
            // Check for potential connections with recent discoveries
            for other in &recent_discoveries {
                if msg.unit_id != other.unit_id {
                    // Check if layers are compatible (±1 rule)
                    let msg_depth = msg.layer.depth() as i32;
                    let other_depth = other.layer.depth() as i32;
                    
                    if (msg_depth - other_depth).abs() <= 1 {
                        // Create connection with random initial strength
                        let strength = 0.3 + rand::random::<f32>() * 0.4;
                        
                        if let Err(e) = self.network.connect_units(
                            msg.unit_id,
                            other.unit_id,
                            strength
                        ).await {
                            tracing::warn!("Failed to create connection: {}", e);
                        } else {
                            tracing::info!(
                                "Connected {} ({}) to {} ({})",
                                msg.unit_id, msg.layer.name(),
                                other.unit_id, other.layer.name()
                            );
                        }
                    }
                }
            }
            
            // Keep recent discoveries
            recent_discoveries.push(msg);
            if recent_discoveries.len() > 100 {
                recent_discoveries.drain(0..50);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hierarchical::cognitive::{BasicCognitiveState, CognitiveConfig};
    
    // Mock cognitive unit for testing
    struct MockUnit {
        id: Uuid,
        layer: CognitiveLayer,
    }
    
    #[async_trait]
    impl CognitiveUnit for MockUnit {
        type Input = CognitiveInput;
        type Output = CognitiveOutput;
        type State = BasicCognitiveState;
        
        fn id(&self) -> &Uuid { &self.id }
        fn layer(&self) -> CognitiveLayer { self.layer }
        
        async fn process(&mut self, input: Self::Input) -> Result<Self::Output> {
            Ok(CognitiveOutput {
                content: format!("Processed: {}", input.content),
                confidence: 0.8,
                metadata: HashMap::new(),
                target_layers: vec![],
            })
        }
        
        async fn learn(&mut self, _gradient: crate::hierarchical::cognitive::LearningGradient) -> Result<()> {
            Ok(())
        }
        
        async fn introspect(&self) -> Self::State {
            BasicCognitiveState {
                unit_id: self.id,
                layer: self.layer,
                metrics: crate::hierarchical::cognitive::StateMetrics {
                    activations_processed: 0,
                    errors_encountered: 0,
                    learning_iterations: 0,
                    average_processing_time_ms: 0.0,
                    memory_usage_bytes: 0,
                },
                parameters: HashMap::new(),
            }
        }
        
        async fn reset(&mut self) -> Result<()> { Ok(()) }
    }
    
    #[tokio::test]
    async fn test_direct_neural_connections() {
        let (network, _rx) = DirectNeuralNetwork::new();
        
        // Create units at different layers
        let l1_unit = Arc::new(MockUnit {
            id: Uuid::new_v4(),
            layer: CognitiveLayer::Reflexive,
        });
        
        let l2_unit = Arc::new(MockUnit {
            id: Uuid::new_v4(),
            layer: CognitiveLayer::Implementation,
        });
        
        let l3_unit = Arc::new(MockUnit {
            id: Uuid::new_v4(),
            layer: CognitiveLayer::Operational,
        });
        
        // Register units
        network.register_unit(l1_unit.clone()).await.unwrap();
        network.register_unit(l2_unit.clone()).await.unwrap();
        network.register_unit(l3_unit.clone()).await.unwrap();
        
        // Connect adjacent layers (should succeed)
        network.connect_units(*l1_unit.id(), *l2_unit.id(), 0.5).await.unwrap();
        network.connect_units(*l2_unit.id(), *l3_unit.id(), 0.7).await.unwrap();
        
        // Try to connect non-adjacent layers (should fail)
        let result = network.connect_units(*l1_unit.id(), *l3_unit.id(), 0.5).await;
        assert!(result.is_err());
        
        // Test signal propagation
        let input = CognitiveInput {
            content: "Test signal".to_string(),
            context: HashMap::new(),
            source_layer: Some(CognitiveLayer::Reflexive),
        };
        
        let outputs = network.propagate_signal(*l1_unit.id(), input).await.unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].content, "Processed: Test signal");
        
        // Test self-organization
        network.self_organize().await.unwrap();
        
        // Check emergence score
        let emergence = network.emergence_score().await;
        assert!(emergence > 0.0);
    }
}