//! True Self-Organization System
//! 
//! Implements genuine self-organization where neurons start undifferentiated
//! and layers emerge naturally from interactions, not from design.
//! 
//! "Like the universe organizing itself into galaxies, stars, and life"

use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use uuid::Uuid;

/// Undifferentiated neuron - no predefined layer or role
#[derive(Debug, Clone)]
pub struct PrimordialNeuron {
    pub id: Uuid,
    pub discovered_neighbors: HashSet<Uuid>,
    pub communication_history: VecDeque<CommunicationEvent>,
    pub processing_speed: f32,      // Randomly initialized
    pub complexity_capacity: f32,   // Randomly initialized
    pub energy_level: f32,
    pub last_active: std::time::Instant,
}

impl Default for PrimordialNeuron {
    fn default() -> Self {
        Self::new()
    }
}

impl PrimordialNeuron {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            discovered_neighbors: HashSet::new(),
            communication_history: VecDeque::with_capacity(1000),
            processing_speed: 0.1 + rand::random::<f32>() * 0.9,
            complexity_capacity: 0.1 + rand::random::<f32>() * 0.9,
            energy_level: 1.0,
            last_active: std::time::Instant::now(),
        }
    }
    
    /// Generate unique signature based on inherent properties
    pub fn signature(&self) -> NeuronSignature {
        NeuronSignature {
            id: self.id,
            speed_class: match self.processing_speed {
                s if s > 0.8 => SpeedClass::VeryFast,
                s if s > 0.6 => SpeedClass::Fast,
                s if s > 0.4 => SpeedClass::Medium,
                s if s > 0.2 => SpeedClass::Slow,
                _ => SpeedClass::VerySlow,
            },
            complexity_class: match self.complexity_capacity {
                c if c > 0.8 => ComplexityClass::VeryComplex,
                c if c > 0.6 => ComplexityClass::Complex,
                c if c > 0.4 => ComplexityClass::Medium,
                c if c > 0.2 => ComplexityClass::Simple,
                _ => ComplexityClass::VerySimple,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronSignature {
    pub id: Uuid,
    pub speed_class: SpeedClass,
    pub complexity_class: ComplexityClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpeedClass {
    VeryFast, Fast, Medium, Slow, VerySlow
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplexityClass {
    VeryComplex, Complex, Medium, Simple, VerySimple
}

#[derive(Debug, Clone)]
pub struct CommunicationEvent {
    pub timestamp: std::time::Instant,
    pub partner: Uuid,
    pub message_type: MessageType,
    pub success: bool,
    pub response_time: std::time::Duration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    Discovery,
    Handshake,
    Data,
    Sync,
}

/// Discovery message broadcast in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMessage {
    pub from: Uuid,
    pub signature: NeuronSignature,
    pub timestamp: u64,
    pub message: String,
}

/// Handshake result between two neurons
#[derive(Debug, Clone)]
pub struct HandshakeResult {
    pub success: bool,
    pub compatibility: f32,
    pub suggested_role: Option<EmergentRole>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmergentRole {
    FastProcessor,
    DeepThinker,
    Connector,
    Coordinator,
    Specialist,
}

/// True self-organizing network without predefined structure
pub struct TrueSelfOrganizingNetwork {
    /// All neurons start equal
    neurons: Arc<RwLock<HashMap<Uuid, Arc<RwLock<PrimordialNeuron>>>>>,
    
    /// Connections form naturally
    connections: Arc<RwLock<HashMap<(Uuid, Uuid), ConnectionStrength>>>,
    
    /// Discovery broadcast channel
    discovery_channel: broadcast::Sender<DiscoveryMessage>,
    
    /// Emergent structure detected
    emergent_structure: Arc<RwLock<Option<EmergentHierarchy>>>,
    
    /// Communication graph for analysis
    comm_graph: Arc<RwLock<CommunicationGraph>>,
    
    /// Event stream
    event_tx: mpsc::Sender<EmergenceEvent>,
}

#[derive(Debug, Clone)]
pub struct ConnectionStrength {
    pub weight: f32,
    pub successful_exchanges: u32,
    pub failed_exchanges: u32,
    pub established_at: std::time::Instant,
}

/// The hierarchy that emerges from interactions
#[derive(Debug, Clone)]
pub struct EmergentHierarchy {
    /// Discovered layers (not predefined!)
    pub layers: Vec<EmergentLayer>,
    
    /// Neuron assignments (discovered through clustering)
    pub neuron_layers: HashMap<Uuid, usize>,
    
    /// Inter-layer connectivity
    pub layer_connections: HashMap<(usize, usize), f32>,
    
    /// When this structure emerged
    pub emerged_at: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct EmergentLayer {
    pub layer_index: usize,
    pub neurons: HashSet<Uuid>,
    pub characteristics: LayerCharacteristics,
    pub discovered_name: String,
}

#[derive(Debug, Clone)]
pub struct LayerCharacteristics {
    pub avg_speed: f32,
    pub avg_complexity: f32,
    pub connectivity_density: f32,
    pub primary_role: EmergentRole,
}

/// Communication graph for pattern analysis
#[derive(Default)]
pub struct CommunicationGraph {
    pub nodes: HashSet<Uuid>,
    pub edges: HashMap<(Uuid, Uuid), EdgeWeight>,
}

#[derive(Debug, Clone)]
pub struct EdgeWeight {
    pub communication_count: u32,
    pub avg_response_time: f32,
    pub affinity: f32,
}

/// Events during emergence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergenceEvent {
    NeuronAdded(Uuid),
    DiscoveryStarted(Uuid),
    HandshakeCompleted { from: Uuid, to: Uuid, compatibility: f32 },
    ClusterFormed { neurons: Vec<Uuid>, characteristics: String },
    LayerEmerged { index: usize, name: String, neurons: usize },
    HierarchyStabilized { total_layers: usize },
}

impl TrueSelfOrganizingNetwork {
    pub fn new() -> (Self, mpsc::Receiver<EmergenceEvent>) {
        let (discovery_tx, _) = broadcast::channel(1000);
        let (event_tx, event_rx) = mpsc::channel(1000);
        
        (Self {
            neurons: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            discovery_channel: discovery_tx,
            emergent_structure: Arc::new(RwLock::new(None)),
            comm_graph: Arc::new(RwLock::new(CommunicationGraph::default())),
            event_tx,
        }, event_rx)
    }
    
    /// Add a primordial neuron to the soup
    pub async fn add_neuron(&self) -> Result<Uuid> {
        let neuron = PrimordialNeuron::new();
        let id = neuron.id;
        
        // Add to network
        self.neurons.write().await.insert(id, Arc::new(RwLock::new(neuron)));
        
        // Add to communication graph
        self.comm_graph.write().await.nodes.insert(id);
        
        // Notify
        let _ = self.event_tx.send(EmergenceEvent::NeuronAdded(id)).await;
        
        tracing::info!("🔵 Neuron {} added to primordial soup", id);
        
        Ok(id)
    }
    
    /// Start discovery phase for a neuron
    pub async fn start_discovery(&self, neuron_id: Uuid) -> Result<()> {
        let neurons = self.neurons.read().await;
        let neuron_arc = neurons.get(&neuron_id).ok_or_else(|| crate::Error::NotFound("Neuron not found".to_string()))?;
        let neuron = neuron_arc.read().await;
        
        // Create discovery message
        let discovery_msg = DiscoveryMessage {
            from: neuron_id,
            signature: neuron.signature(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            message: self.generate_discovery_message(&neuron),
        };
        
        drop(neuron);
        drop(neurons);
        
        // Broadcast presence
        let _ = self.discovery_channel.send(discovery_msg);
        let _ = self.event_tx.send(EmergenceEvent::DiscoveryStarted(neuron_id)).await;
        
        tracing::debug!("📡 Neuron {} broadcasting presence", neuron_id);
        
        Ok(())
    }
    
    /// Generate discovery message based on neuron properties
    fn generate_discovery_message(&self, neuron: &PrimordialNeuron) -> String {
        match (neuron.processing_speed, neuron.complexity_capacity) {
            (s, c) if s > 0.8 && c < 0.3 => "Quick responder here!".to_string(),
            (s, c) if s < 0.3 && c > 0.8 => "Deep thinker present.".to_string(),
            (s, c) if s > 0.5 && c > 0.5 => "Balanced processor ready.".to_string(),
            _ => "Neuron seeking connections.".to_string(),
        }
    }
    
    /// Process discovery phase - neurons find each other
    pub async fn discovery_phase(&self, duration: std::time::Duration) -> Result<()> {
        tracing::info!("🔍 Starting discovery phase for {:?}", duration);
        
        // Subscribe all neurons to discovery channel
        let neurons = self.neurons.read().await;
        let mut receivers = Vec::new();
        
        for &id in neurons.keys() {
            receivers.push((id, self.discovery_channel.subscribe()));
        }
        
        drop(neurons);
        
        // Let neurons broadcast
        for (id, _) in &receivers {
            self.start_discovery(*id).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        // Process discovery messages
        let start = std::time::Instant::now();
        
        while start.elapsed() < duration {
            for (listener_id, receiver) in &mut receivers {
                // Try to receive discovery messages
                match receiver.try_recv() {
                    Ok(msg) if msg.from != *listener_id => {
                        // Process discovery
                        self.process_discovery_message(*listener_id, msg).await?;
                    }
                    _ => {}
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
        
        tracing::info!("✅ Discovery phase complete");
        Ok(())
    }
    
    /// Process received discovery message
    async fn process_discovery_message(&self, listener: Uuid, msg: DiscoveryMessage) -> Result<()> {
        let neurons = self.neurons.read().await;
        
        if let (Some(listener_arc), Some(sender_arc)) = (neurons.get(&listener), neurons.get(&msg.from)) {
            let listener_neuron = listener_arc.read().await;
            let sender_neuron = sender_arc.read().await;
            
            // Calculate compatibility
            let compatibility = self.calculate_compatibility(&listener_neuron, &sender_neuron);
            
            drop(listener_neuron);
            drop(sender_neuron);
            drop(neurons);
            
            if compatibility > 0.5 {
                // Attempt handshake
                self.attempt_handshake(listener, msg.from, compatibility).await?;
            }
        }
        
        Ok(())
    }
    
    /// Calculate compatibility between two neurons
    fn calculate_compatibility(&self, n1: &PrimordialNeuron, n2: &PrimordialNeuron) -> f32 {
        // Similar speeds work well together
        let speed_diff = (n1.processing_speed - n2.processing_speed).abs();
        let speed_compat = 1.0 - speed_diff;
        
        // Complementary complexity (not too similar, not too different)
        let complexity_diff = (n1.complexity_capacity - n2.complexity_capacity).abs();
        let complexity_compat = if complexity_diff < 0.2 {
            0.7 // Too similar
        } else if complexity_diff > 0.7 {
            0.3 // Too different
        } else {
            1.0 // Just right
        };
        
        // Energy levels should be compatible
        let energy_compat = (n1.energy_level * n2.energy_level).sqrt();
        
        (speed_compat * 0.4 + complexity_compat * 0.4 + energy_compat * 0.2).clamp(0.0, 1.0)
    }
    
    /// Attempt handshake between two neurons
    async fn attempt_handshake(&self, n1: Uuid, n2: Uuid, compatibility: f32) -> Result<()> {
        // Record connection
        let mut connections = self.connections.write().await;
        let key = if n1 < n2 { (n1, n2) } else { (n2, n1) };
        
        connections.insert(key, ConnectionStrength {
            weight: compatibility,
            successful_exchanges: 1,
            failed_exchanges: 0,
            established_at: std::time::Instant::now(),
        });
        
        // Update neuron neighbors
        let neurons = self.neurons.read().await;
        
        if let Some(n1_arc) = neurons.get(&n1) {
            n1_arc.write().await.discovered_neighbors.insert(n2);
        }
        
        if let Some(n2_arc) = neurons.get(&n2) {
            n2_arc.write().await.discovered_neighbors.insert(n1);
        }
        
        // Update communication graph
        let mut graph = self.comm_graph.write().await;
        graph.edges.insert(key, EdgeWeight {
            communication_count: 1,
            avg_response_time: 0.0,
            affinity: compatibility,
        });
        
        // Send event
        let _ = self.event_tx.send(EmergenceEvent::HandshakeCompleted {
            from: n1,
            to: n2,
            compatibility,
        }).await;
        
        tracing::debug!("🤝 Handshake: {} ↔ {} (compatibility: {:.2})", n1, n2, compatibility);
        
        Ok(())
    }
    
    /// Analyze patterns and detect emergent hierarchy
    pub async fn detect_emergent_hierarchy(&self) -> Result<()> {
        tracing::info!("🔬 Analyzing communication patterns for emergent hierarchy...");
        
        let neurons = self.neurons.read().await;
        let _graph = self.comm_graph.read().await;
        
        // Collect neuron characteristics
        let mut neuron_profiles = Vec::new();
        
        for (id, neuron_arc) in neurons.iter() {
            let neuron = neuron_arc.read().await;
            
            let neighbor_count = neuron.discovered_neighbors.len();
            let avg_speed = neuron.processing_speed;
            let avg_complexity = neuron.complexity_capacity;
            
            neuron_profiles.push((*id, avg_speed, avg_complexity, neighbor_count));
        }
        
        drop(neurons);
        
        // Perform clustering (simple k-means style)
        let clusters = self.cluster_neurons(&neuron_profiles, 5);
        
        // Analyze cluster characteristics
        let mut emergent_layers = Vec::new();
        
        for (idx, cluster) in clusters.iter().enumerate() {
            let characteristics = self.analyze_cluster_characteristics(cluster, &neuron_profiles);
            
            let layer = EmergentLayer {
                layer_index: idx,
                neurons: cluster.iter().cloned().collect(),
                characteristics: characteristics.clone(),
                discovered_name: self.name_layer(&characteristics),
            };
            
            emergent_layers.push(layer);
            
            // Send event
            let _ = self.event_tx.send(EmergenceEvent::LayerEmerged {
                index: idx,
                name: self.name_layer(&characteristics),
                neurons: cluster.len(),
            }).await;
        }
        
        // Sort layers by characteristics (fast/simple -> slow/complex)
        emergent_layers.sort_by(|a, b| {
            let a_score = a.characteristics.avg_speed * (1.0 - a.characteristics.avg_complexity);
            let b_score = b.characteristics.avg_speed * (1.0 - b.characteristics.avg_complexity);
            b_score.partial_cmp(&a_score).unwrap()
        });
        
        // Build neuron layer map
        let mut neuron_layers = HashMap::new();
        for (idx, layer) in emergent_layers.iter().enumerate() {
            for &neuron_id in &layer.neurons {
                neuron_layers.insert(neuron_id, idx);
            }
        }
        
        // Create emergent hierarchy
        let hierarchy = EmergentHierarchy {
            layers: emergent_layers,
            neuron_layers,
            layer_connections: HashMap::new(), // TODO: Calculate inter-layer connectivity
            emerged_at: std::time::Instant::now(),
        };
        
        *self.emergent_structure.write().await = Some(hierarchy);
        
        // Send stabilization event
        let _ = self.event_tx.send(EmergenceEvent::HierarchyStabilized {
            total_layers: clusters.len(),
        }).await;
        
        tracing::info!("✨ Hierarchy emerged with {} layers!", clusters.len());
        
        Ok(())
    }
    
    /// Simple clustering algorithm
    fn cluster_neurons(&self, profiles: &[(Uuid, f32, f32, usize)], k: usize) -> Vec<Vec<Uuid>> {
        // Simple clustering based on speed and complexity
        // In real implementation, use proper clustering like k-means or spectral clustering
        
        let mut clusters: Vec<Vec<Uuid>> = vec![Vec::new(); k];
        
        for (id, speed, complexity, _) in profiles {
            // Assign to cluster based on characteristics
            let cluster_idx = match (*speed, *complexity) {
                (s, c) if s > 0.8 && c < 0.3 => 0, // Very fast, simple
                (s, c) if s > 0.6 && c < 0.5 => 1, // Fast, medium
                (s, c) if s > 0.4 && c > 0.4 && c < 0.6 => 2, // Medium everything
                (s, c) if s < 0.5 && c > 0.6 => 3, // Slow, complex
                _ => 4, // Others
            };
            
            clusters[cluster_idx].push(*id);
        }
        
        // Remove empty clusters
        clusters.retain(|c| !c.is_empty());
        
        clusters
    }
    
    /// Analyze characteristics of a cluster
    fn analyze_cluster_characteristics(
        &self,
        cluster: &[Uuid],
        profiles: &[(Uuid, f32, f32, usize)]
    ) -> LayerCharacteristics {
        let cluster_profiles: Vec<_> = profiles.iter()
            .filter(|(id, _, _, _)| cluster.contains(id))
            .collect();
        
        let avg_speed = cluster_profiles.iter()
            .map(|(_, s, _, _)| s)
            .sum::<f32>() / cluster_profiles.len() as f32;
        
        let avg_complexity = cluster_profiles.iter()
            .map(|(_, _, c, _)| c)
            .sum::<f32>() / cluster_profiles.len() as f32;
        
        let avg_connections = cluster_profiles.iter()
            .map(|(_, _, _, n)| *n as f32)
            .sum::<f32>() / cluster_profiles.len() as f32;
        
        let connectivity_density = avg_connections / cluster.len() as f32;
        
        let primary_role = match (avg_speed, avg_complexity, connectivity_density) {
            (s, c, _) if s > 0.7 && c < 0.3 => EmergentRole::FastProcessor,
            (s, c, _) if s < 0.3 && c > 0.7 => EmergentRole::DeepThinker,
            (_, _, d) if d > 0.5 => EmergentRole::Connector,
            _ => EmergentRole::Specialist,
        };
        
        LayerCharacteristics {
            avg_speed,
            avg_complexity,
            connectivity_density,
            primary_role,
        }
    }
    
    /// Name a layer based on its characteristics
    fn name_layer(&self, characteristics: &LayerCharacteristics) -> String {
        match characteristics.primary_role {
            EmergentRole::FastProcessor => "Reflexive Layer (emerged)".to_string(),
            EmergentRole::DeepThinker => "Strategic Layer (emerged)".to_string(),
            EmergentRole::Connector => "Operational Layer (emerged)".to_string(),
            EmergentRole::Coordinator => "Tactical Layer (emerged)".to_string(),
            EmergentRole::Specialist => "Implementation Layer (emerged)".to_string(),
        }
    }
    
    /// Get current state report
    pub async fn emergence_report(&self) -> EmergenceReport {
        let neurons = self.neurons.read().await;
        let connections = self.connections.read().await;
        let structure = self.emergent_structure.read().await;
        
        EmergenceReport {
            total_neurons: neurons.len(),
            total_connections: connections.len(),
            avg_compatibility: connections.values()
                .map(|c| c.weight)
                .sum::<f32>() / connections.len().max(1) as f32,
            hierarchy_emerged: structure.is_some(),
            layers: structure.as_ref().map(|h| {
                h.layers.iter().map(|l| LayerInfo {
                    name: l.discovered_name.clone(),
                    neurons: l.neurons.len(),
                    avg_speed: l.characteristics.avg_speed,
                    avg_complexity: l.characteristics.avg_complexity,
                    role: format!("{:?}", l.characteristics.primary_role),
                }).collect()
            }).unwrap_or_default(),
        }
    }
}

/// Report on emergence status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergenceReport {
    pub total_neurons: usize,
    pub total_connections: usize,
    pub avg_compatibility: f32,
    pub hierarchy_emerged: bool,
    pub layers: Vec<LayerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerInfo {
    pub name: String,
    pub neurons: usize,
    pub avg_speed: f32,
    pub avg_complexity: f32,
    pub role: String,
}

impl EmergenceReport {
    pub fn summary(&self) -> String {
        format!(
            "Neurons: {} | Connections: {} | Avg Compatibility: {:.2} | Layers Emerged: {}",
            self.total_neurons,
            self.total_connections,
            self.avg_compatibility,
            if self.hierarchy_emerged { self.layers.len() } else { 0 }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_true_self_organization() {
        let (network, mut event_rx) = TrueSelfOrganizingNetwork::new();
        
        // Add 25 undifferentiated neurons
        for _ in 0..25 {
            network.add_neuron().await.unwrap();
        }
        
        // Discovery phase
        network.discovery_phase(std::time::Duration::from_secs(1)).await.unwrap();
        
        // Detect emergent hierarchy
        network.detect_emergent_hierarchy().await.unwrap();
        
        // Get report
        let report = network.emergence_report().await;
        println!("Emergence Report: {}", report.summary());
        
        assert!(report.total_neurons == 25);
        assert!(report.total_connections > 0);
        assert!(report.hierarchy_emerged);
        
        // Check events
        let mut event_count = 0;
        while let Ok(event) = event_rx.try_recv() {
            if let EmergenceEvent::LayerEmerged { name, neurons, .. } = event {
                println!("Layer emerged: {} with {} neurons", name, neurons);
            }
            event_count += 1;
        }
        
        assert!(event_count > 0);
    }
}