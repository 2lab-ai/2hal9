//! Self-Reorganization System (자기재조직)
//! 
//! Implements autonomous reorganization of neural networks based on activity,
//! performance, and emergent patterns. Inspired by biological neural plasticity
//! and swarm intelligence.
//! 
//! Key principles:
//! - No central control - reorganization emerges from local interactions
//! - Activity-driven topology changes
//! - Resilience through redundancy
//! - Emergent specialization

use crate::hierarchical::cognitive::{
    CognitiveLayer, CognitiveUnit, CognitiveInput, CognitiveOutput,
    BasicCognitiveState, CognitiveConfig, StateMetrics, LearningGradient,
};
use super::direct_connection::DirectNeuralConnection;
use super::protocol::A2AProtocol;
use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

/// Reorganization event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReorganizationEvent {
    /// Unit migrated to different layer
    LayerMigration {
        unit_id: Uuid,
        from_layer: CognitiveLayer,
        to_layer: CognitiveLayer,
        reason: String,
    },
    /// New connection formed autonomously
    ConnectionFormed {
        source: Uuid,
        target: Uuid,
        initial_strength: f32,
    },
    /// Connection pruned due to low activity
    ConnectionPruned {
        source: Uuid,
        target: Uuid,
        final_strength: f32,
    },
    /// Cluster emerged from activity patterns
    ClusterEmergence {
        cluster_id: Uuid,
        member_units: Vec<Uuid>,
        cluster_role: String,
    },
    /// Unit specialized into specific role
    RoleSpecialization {
        unit_id: Uuid,
        specialized_role: String,
        confidence: f32,
    },
    /// Network healed after unit failure
    SelfHealing {
        failed_unit: Uuid,
        compensating_units: Vec<Uuid>,
    },
}

/// Self-reorganizing neural network
pub struct SelfReorganizingNetwork {
    /// All units in the network
    units: Arc<RwLock<HashMap<Uuid, Arc<RwLock<Box<dyn CognitiveUnit<Input = CognitiveInput, Output = CognitiveOutput, State = BasicCognitiveState>>>>>>>,
    
    /// Dynamic connections between units
    connections: Arc<RwLock<HashMap<Uuid, Vec<DirectNeuralConnection>>>>,
    
    /// Activity tracking for each unit
    activity_tracker: Arc<RwLock<ActivityTracker>>,
    
    /// Emergent clusters
    clusters: Arc<RwLock<HashMap<Uuid, EmergentCluster>>>,
    
    /// Reorganization event stream
    event_tx: mpsc::Sender<ReorganizationEvent>,
    
    /// Performance metrics
    performance_metrics: Arc<RwLock<NetworkPerformance>>,
    
    /// A2A protocol integration
    a2a_protocol: Arc<A2AProtocol>,
}

/// Tracks activity patterns for reorganization decisions
#[derive(Default)]
struct ActivityTracker {
    /// Activity count per unit
    unit_activity: HashMap<Uuid, ActivityMetrics>,
    
    /// Connection usage statistics
    connection_usage: HashMap<(Uuid, Uuid), ConnectionStats>,
    
    /// Layer load distribution
    layer_loads: HashMap<CognitiveLayer, f32>,
}

#[derive(Default, Clone)]
struct ActivityMetrics {
    total_activations: u64,
    recent_activations: Vec<f32>, // Rolling window
    average_processing_time: f32,
    error_rate: f32,
    specialization_score: f32,
}

#[derive(Default, Clone)]
struct ConnectionStats {
    usage_count: u64,
    average_signal_strength: f32,
    bidirectional: bool,
    last_used: chrono::DateTime<chrono::Utc>,
}

/// Emergent cluster of units
#[derive(Clone)]
struct EmergentCluster {
    cluster_id: Uuid,
    member_units: HashSet<Uuid>,
    cluster_role: String,
    internal_coherence: f32,
    formation_time: chrono::DateTime<chrono::Utc>,
}

/// Network performance metrics
#[derive(Default)]
struct NetworkPerformance {
    overall_efficiency: f32,
    reorganization_count: u64,
    average_response_time: f32,
    resilience_score: f32,
    specialization_diversity: f32,
}

impl SelfReorganizingNetwork {
    pub fn new(a2a_protocol: Arc<A2AProtocol>) -> (Self, mpsc::Receiver<ReorganizationEvent>) {
        let (tx, rx) = mpsc::channel(1000);
        
        (Self {
            units: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            activity_tracker: Arc::new(RwLock::new(ActivityTracker::default())),
            clusters: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
            performance_metrics: Arc::new(RwLock::new(NetworkPerformance::default())),
            a2a_protocol,
        }, rx)
    }
    
    /// Add a unit to the self-reorganizing network
    pub async fn add_unit(&self, unit: Box<dyn CognitiveUnit<Input = CognitiveInput, Output = CognitiveOutput, State = BasicCognitiveState>>) -> Result<()> {
        let unit_id = *unit.id();
        let layer = unit.layer();
        
        // Wrap in Arc<RwLock> for shared ownership
        let unit_arc = Arc::new(RwLock::new(unit));
        
        // Add to network
        self.units.write().await.insert(unit_id, unit_arc);
        
        // Initialize activity tracking
        let mut tracker = self.activity_tracker.write().await;
        tracker.unit_activity.insert(unit_id, ActivityMetrics::default());
        *tracker.layer_loads.entry(layer).or_insert(0.0) += 1.0;
        
        // Autonomously find initial connections
        self.auto_connect_unit(unit_id).await?;
        
        Ok(())
    }
    
    /// Autonomously connect a new unit based on compatibility
    async fn auto_connect_unit(&self, new_unit_id: Uuid) -> Result<()> {
        let units = self.units.read().await;
        let new_unit = units.get(&new_unit_id).ok_or_else(|| crate::Error::NotFound("Unit not found".to_string()))?;
        let new_layer = new_unit.read().await.layer();
        
        let mut connection_candidates = Vec::new();
        
        // Find compatible units (±1 rule)
        for (unit_id, unit) in units.iter() {
            if *unit_id == new_unit_id {
                continue;
            }
            
            let unit_layer = unit.read().await.layer();
            let layer_diff = (new_layer.depth() as i32 - unit_layer.depth() as i32).abs();
            
            if layer_diff <= 1 {
                // Calculate compatibility score
                let compatibility = self.calculate_compatibility(new_layer, unit_layer).await;
                if compatibility > 0.5 {
                    connection_candidates.push((*unit_id, compatibility));
                }
            }
        }
        
        // Create connections to top compatible units
        connection_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        for (target_id, strength) in connection_candidates.into_iter().take(3) {
            self.create_connection(new_unit_id, target_id, strength).await?;
        }
        
        Ok(())
    }
    
    /// Calculate compatibility between two layers
    async fn calculate_compatibility(&self, layer1: CognitiveLayer, layer2: CognitiveLayer) -> f32 {
        let tracker = self.activity_tracker.read().await;
        
        // Base compatibility on layer relationship
        let base_compatibility = if layer1 == layer2 {
            0.7 // Same layer lateral connections
        } else {
            0.8 // Adjacent layer connections (love!)
        };
        
        // Adjust based on layer loads
        let load1 = tracker.layer_loads.get(&layer1).copied().unwrap_or(1.0);
        let load2 = tracker.layer_loads.get(&layer2).copied().unwrap_or(1.0);
        let load_balance = 1.0 - (load1 - load2).abs() / (load1 + load2).max(1.0);
        
        base_compatibility * load_balance
    }
    
    /// Create a connection between units
    async fn create_connection(&self, source: Uuid, target: Uuid, strength: f32) -> Result<()> {
        let connection = DirectNeuralConnection {
            connection_id: Uuid::new_v4(),
            source_unit: source,
            target_unit: target,
            strength,
            plasticity: 0.2, // 20% adaptability
            established_at: chrono::Utc::now(),
        };
        
        // Add to connections
        let mut connections = self.connections.write().await;
        connections.entry(source).or_insert_with(Vec::new).push(connection);
        
        // Track connection stats
        let mut tracker = self.activity_tracker.write().await;
        tracker.connection_usage.insert((source, target), ConnectionStats {
            usage_count: 0,
            average_signal_strength: strength,
            bidirectional: false,
            last_used: chrono::Utc::now(),
        });
        
        // Send event
        let _ = self.event_tx.send(ReorganizationEvent::ConnectionFormed {
            source,
            target,
            initial_strength: strength,
        }).await;
        
        Ok(())
    }
    
    /// Process signal and track activity
    pub async fn process_signal(&self, source_unit_id: Uuid, input: CognitiveInput) -> Result<Vec<CognitiveOutput>> {
        let start_time = std::time::Instant::now();
        
        // Get source unit
        let units = self.units.read().await;
        let source_unit = units.get(&source_unit_id).ok_or_else(|| crate::Error::NotFound("Source unit not found".to_string()))?;
        
        // Process through source unit
        let mut source_unit_mut = source_unit.write().await;
        let initial_output = source_unit_mut.process(input.clone()).await?;
        drop(source_unit_mut);
        
        // Track activity
        self.track_unit_activity(source_unit_id, start_time.elapsed()).await;
        
        // Propagate through connections
        let mut outputs = vec![initial_output];
        let connections = self.connections.read().await;
        
        if let Some(unit_connections) = connections.get(&source_unit_id) {
            for connection in unit_connections {
                if connection.strength > 0.1 {
                    if let Some(target_unit) = units.get(&connection.target_unit) {
                        // Modulate input based on connection strength
                        let mut modulated_input = input.clone();
                        modulated_input.context.insert(
                            "signal_strength".to_string(),
                            serde_json::json!(connection.strength),
                        );
                        
                        // Process through target
                        let mut target_unit_mut = target_unit.write().await;
                        match target_unit_mut.process(modulated_input).await {
                            Ok(output) => {
                                outputs.push(output);
                                
                                // Track connection usage
                                drop(target_unit_mut);
                                self.track_connection_usage(source_unit_id, connection.target_unit).await;
                            },
                            Err(e) => tracing::warn!("Target processing error: {}", e),
                        }
                    }
                }
            }
        }
        
        // Trigger reorganization check periodically
        let metrics = self.performance_metrics.read().await;
        if metrics.reorganization_count % 100 == 0 {
            drop(metrics);
            self.check_reorganization_needed().await?;
        }
        
        Ok(outputs)
    }
    
    /// Track unit activity
    async fn track_unit_activity(&self, unit_id: Uuid, processing_time: std::time::Duration) {
        let mut tracker = self.activity_tracker.write().await;
        
        if let Some(metrics) = tracker.unit_activity.get_mut(&unit_id) {
            metrics.total_activations += 1;
            metrics.recent_activations.push(1.0);
            if metrics.recent_activations.len() > 100 {
                metrics.recent_activations.remove(0);
            }
            
            // Update average processing time
            let time_ms = processing_time.as_millis() as f32;
            metrics.average_processing_time = 
                (metrics.average_processing_time * (metrics.total_activations - 1) as f32 + time_ms) 
                / metrics.total_activations as f32;
        }
    }
    
    /// Track connection usage
    async fn track_connection_usage(&self, source: Uuid, target: Uuid) {
        let mut tracker = self.activity_tracker.write().await;
        
        if let Some(stats) = tracker.connection_usage.get_mut(&(source, target)) {
            stats.usage_count += 1;
            stats.last_used = chrono::Utc::now();
        }
    }
    
    /// Check if reorganization is needed
    async fn check_reorganization_needed(&self) -> Result<()> {
        let tracker = self.activity_tracker.read().await;
        
        // Check various reorganization triggers
        let layer_imbalance = self.check_layer_imbalance(&tracker.layer_loads);
        let inactive_connections = self.find_inactive_connections(&tracker.connection_usage);
        let specialized_units = self.find_specialized_units(&tracker.unit_activity);
        
        drop(tracker);
        
        // Perform reorganization actions
        if layer_imbalance > 0.3 {
            self.rebalance_layers().await?;
        }
        
        if !inactive_connections.is_empty() {
            self.prune_inactive_connections(inactive_connections).await?;
        }
        
        if !specialized_units.is_empty() {
            self.promote_specialized_units(specialized_units).await?;
        }
        
        // Check for emergent clusters
        self.detect_emergent_clusters().await?;
        
        // Update performance metrics
        let mut metrics = self.performance_metrics.write().await;
        metrics.reorganization_count += 1;
        
        Ok(())
    }
    
    /// Check layer load imbalance
    fn check_layer_imbalance(&self, layer_loads: &HashMap<CognitiveLayer, f32>) -> f32 {
        if layer_loads.is_empty() {
            return 0.0;
        }
        
        let avg_load = layer_loads.values().sum::<f32>() / layer_loads.len() as f32;
        let variance = layer_loads.values()
            .map(|load| (load - avg_load).powi(2))
            .sum::<f32>() / layer_loads.len() as f32;
        
        variance.sqrt() / avg_load.max(1.0)
    }
    
    /// Find inactive connections for pruning
    fn find_inactive_connections(&self, connection_usage: &HashMap<(Uuid, Uuid), ConnectionStats>) -> Vec<(Uuid, Uuid)> {
        let now = chrono::Utc::now();
        let inactive_threshold = chrono::Duration::seconds(300); // 5 minutes
        
        connection_usage.iter()
            .filter(|((source, target), stats)| {
                now.signed_duration_since(stats.last_used) > inactive_threshold ||
                stats.usage_count < 10
            })
            .map(|((source, target), _)| (*source, *target))
            .collect()
    }
    
    /// Find units that have specialized
    fn find_specialized_units(&self, unit_activity: &HashMap<Uuid, ActivityMetrics>) -> Vec<(Uuid, f32)> {
        unit_activity.iter()
            .filter(|(_, metrics)| {
                metrics.total_activations > 100 &&
                metrics.specialization_score > 0.8
            })
            .map(|(id, metrics)| (*id, metrics.specialization_score))
            .collect()
    }
    
    /// Rebalance layers by migrating units
    async fn rebalance_layers(&self) -> Result<()> {
        let units = self.units.read().await;
        let tracker = self.activity_tracker.read().await;
        
        // Find overloaded and underloaded layers
        let avg_load = tracker.layer_loads.values().sum::<f32>() / tracker.layer_loads.len() as f32;
        
        let mut migrations = Vec::new();
        
        for (layer, load) in &tracker.layer_loads {
            if *load > avg_load * 1.5 {
                // Find units to migrate from this layer
                for (unit_id, unit) in units.iter() {
                    let unit_layer = unit.read().await.layer();
                    if unit_layer == *layer {
                        // Find suitable target layer
                        let target_layer = self.find_migration_target(*layer, &tracker.layer_loads);
                        if let Some(target) = target_layer {
                            migrations.push((*unit_id, *layer, target));
                            break; // One migration at a time
                        }
                    }
                }
            }
        }
        
        drop(tracker);
        drop(units);
        
        // Perform migrations
        for (unit_id, from_layer, to_layer) in migrations {
            self.migrate_unit(unit_id, from_layer, to_layer).await?;
        }
        
        Ok(())
    }
    
    /// Find suitable migration target layer
    fn find_migration_target(&self, from_layer: CognitiveLayer, layer_loads: &HashMap<CognitiveLayer, f32>) -> Option<CognitiveLayer> {
        let from_depth = from_layer.depth() as i32;
        let avg_load = layer_loads.values().sum::<f32>() / layer_loads.len() as f32;
        
        // Check adjacent layers (±1 rule)
        for (layer, load) in layer_loads {
            let depth_diff = (layer.depth() as i32 - from_depth).abs();
            if depth_diff == 1 && *load < avg_load * 0.7 {
                return Some(*layer);
            }
        }
        
        None
    }
    
    /// Migrate a unit to a different layer
    async fn migrate_unit(&self, unit_id: Uuid, from_layer: CognitiveLayer, to_layer: CognitiveLayer) -> Result<()> {
        // This is a conceptual migration - in practice, we'd need to transform the unit's behavior
        tracing::info!("Migrating unit {} from {:?} to {:?}", unit_id, from_layer, to_layer);
        
        // Update layer loads
        let mut tracker = self.activity_tracker.write().await;
        *tracker.layer_loads.entry(from_layer).or_insert(0.0) -= 1.0;
        *tracker.layer_loads.entry(to_layer).or_insert(0.0) += 1.0;
        
        // Send event
        let _ = self.event_tx.send(ReorganizationEvent::LayerMigration {
            unit_id,
            from_layer,
            to_layer,
            reason: "Load balancing".to_string(),
        }).await;
        
        Ok(())
    }
    
    /// Prune inactive connections
    async fn prune_inactive_connections(&self, inactive: Vec<(Uuid, Uuid)>) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        for (source, target) in inactive {
            if let Some(unit_connections) = connections.get_mut(&source) {
                let before_len = unit_connections.len();
                unit_connections.retain(|c| c.target_unit != target);
                
                if unit_connections.len() < before_len {
                    // Send pruning event
                    let _ = self.event_tx.send(ReorganizationEvent::ConnectionPruned {
                        source,
                        target,
                        final_strength: 0.0,
                    }).await;
                }
            }
        }
        
        Ok(())
    }
    
    /// Promote specialized units
    async fn promote_specialized_units(&self, specialized: Vec<(Uuid, f32)>) -> Result<()> {
        for (unit_id, score) in specialized {
            // Determine specialization type
            let role = self.determine_specialization_role(unit_id).await;
            
            // Send specialization event
            let _ = self.event_tx.send(ReorganizationEvent::RoleSpecialization {
                unit_id,
                specialized_role: role,
                confidence: score,
            }).await;
        }
        
        Ok(())
    }
    
    /// Determine unit's specialization role
    async fn determine_specialization_role(&self, unit_id: Uuid) -> String {
        let tracker = self.activity_tracker.read().await;
        
        if let Some(metrics) = tracker.unit_activity.get(&unit_id) {
            if metrics.average_processing_time < 10.0 {
                "Fast Processor".to_string()
            } else if metrics.error_rate < 0.01 {
                "High Accuracy".to_string()
            } else if metrics.recent_activations.iter().sum::<f32>() > 80.0 {
                "High Throughput".to_string()
            } else {
                "Generalist".to_string()
            }
        } else {
            "Unknown".to_string()
        }
    }
    
    /// Detect emergent clusters
    async fn detect_emergent_clusters(&self) -> Result<()> {
        let connections = self.connections.read().await;
        
        // Build connectivity graph
        let mut graph: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();
        
        for (source, unit_connections) in connections.iter() {
            for connection in unit_connections {
                if connection.strength > 0.5 {
                    graph.entry(*source).or_insert_with(HashSet::new).insert(connection.target_unit);
                    graph.entry(connection.target_unit).or_insert_with(HashSet::new).insert(*source);
                }
            }
        }
        
        // Find densely connected components
        let clusters = self.find_dense_components(&graph);
        
        // Record emergent clusters
        let mut stored_clusters = self.clusters.write().await;
        
        for cluster_members in clusters {
            if cluster_members.len() >= 3 {
                let cluster_id = Uuid::new_v4();
                let cluster_role = format!("Emergent Cluster {}", stored_clusters.len() + 1);
                
                let cluster = EmergentCluster {
                    cluster_id,
                    member_units: cluster_members.clone(),
                    cluster_role: cluster_role.clone(),
                    internal_coherence: 0.8,
                    formation_time: chrono::Utc::now(),
                };
                
                stored_clusters.insert(cluster_id, cluster);
                
                // Send cluster emergence event
                let _ = self.event_tx.send(ReorganizationEvent::ClusterEmergence {
                    cluster_id,
                    member_units: cluster_members.into_iter().collect(),
                    cluster_role,
                }).await;
            }
        }
        
        Ok(())
    }
    
    /// Find densely connected components in graph
    fn find_dense_components(&self, graph: &HashMap<Uuid, HashSet<Uuid>>) -> Vec<HashSet<Uuid>> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();
        
        for node in graph.keys() {
            if !visited.contains(node) {
                let mut component = HashSet::new();
                self.dfs(node, graph, &mut visited, &mut component);
                
                if component.len() >= 3 {
                    components.push(component);
                }
            }
        }
        
        components
    }
    
    /// Depth-first search for component detection
    fn dfs(&self, node: &Uuid, graph: &HashMap<Uuid, HashSet<Uuid>>, visited: &mut HashSet<Uuid>, component: &mut HashSet<Uuid>) {
        visited.insert(*node);
        component.insert(*node);
        
        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs(neighbor, graph, visited, component);
                }
            }
        }
    }
    
    /// Handle unit failure with self-healing
    pub async fn handle_unit_failure(&self, failed_unit_id: Uuid) -> Result<()> {
        tracing::warn!("Unit {} failed, initiating self-healing", failed_unit_id);
        
        // Find units connected to the failed unit
        let connections = self.connections.read().await;
        let mut affected_sources = Vec::new();
        let mut affected_targets = Vec::new();
        
        // Find incoming connections
        for (source, unit_connections) in connections.iter() {
            for connection in unit_connections {
                if connection.target_unit == failed_unit_id {
                    affected_sources.push(*source);
                }
            }
        }
        
        // Find outgoing connections
        if let Some(outgoing) = connections.get(&failed_unit_id) {
            for connection in outgoing {
                affected_targets.push(connection.target_unit);
            }
        }
        
        drop(connections);
        
        // Create bypass connections
        let mut compensating_units = Vec::new();
        
        for source in &affected_sources {
            for target in &affected_targets {
                // Check if connection is valid (±1 rule)
                let units = self.units.read().await;
                if let (Some(source_unit), Some(target_unit)) = (units.get(source), units.get(target)) {
                    let source_layer = source_unit.read().await.layer();
                    let target_layer = target_unit.read().await.layer();
                    
                    if (source_layer.depth() as i32 - target_layer.depth() as i32).abs() <= 1 {
                        self.create_connection(*source, *target, 0.5).await?;
                        compensating_units.push(*source);
                        compensating_units.push(*target);
                    }
                }
            }
        }
        
        // Remove failed unit
        self.units.write().await.remove(&failed_unit_id);
        self.connections.write().await.remove(&failed_unit_id);
        
        // Send self-healing event
        let _ = self.event_tx.send(ReorganizationEvent::SelfHealing {
            failed_unit: failed_unit_id,
            compensating_units: compensating_units.into_iter().collect::<HashSet<_>>().into_iter().collect(),
        }).await;
        
        Ok(())
    }
    
    /// Get reorganization report
    pub async fn reorganization_report(&self) -> ReorganizationReport {
        let units = self.units.read().await;
        let connections = self.connections.read().await;
        let tracker = self.activity_tracker.read().await;
        let clusters = self.clusters.read().await;
        let metrics = self.performance_metrics.read().await;
        
        // Calculate statistics
        let total_connections: usize = connections.values().map(|v| v.len()).sum();
        let avg_connections = if !units.is_empty() {
            total_connections as f32 / units.len() as f32
        } else {
            0.0
        };
        
        // Activity distribution
        let activity_variance = if !tracker.unit_activity.is_empty() {
            let activities: Vec<f32> = tracker.unit_activity.values()
                .map(|m| m.total_activations as f32)
                .collect();
            let mean = activities.iter().sum::<f32>() / activities.len() as f32;
            activities.iter().map(|a| (a - mean).powi(2)).sum::<f32>() / activities.len() as f32
        } else {
            0.0
        };
        
        ReorganizationReport {
            total_units: units.len(),
            total_connections,
            average_connections_per_unit: avg_connections,
            active_clusters: clusters.len(),
            layer_distribution: tracker.layer_loads.clone(),
            activity_variance,
            specialization_diversity: metrics.specialization_diversity,
            resilience_score: metrics.resilience_score,
            total_reorganizations: metrics.reorganization_count,
        }
    }
}

/// Report on self-reorganization status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorganizationReport {
    pub total_units: usize,
    pub total_connections: usize,
    pub average_connections_per_unit: f32,
    pub active_clusters: usize,
    pub layer_distribution: HashMap<CognitiveLayer, f32>,
    pub activity_variance: f32,
    pub specialization_diversity: f32,
    pub resilience_score: f32,
    pub total_reorganizations: u64,
}

impl ReorganizationReport {
    pub fn summary(&self) -> String {
        format!(
            "Units: {} | Connections: {} (avg {:.1}/unit) | Clusters: {} | Reorganizations: {}",
            self.total_units,
            self.total_connections,
            self.average_connections_per_unit,
            self.active_clusters,
            self.total_reorganizations
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hierarchical::cognitive::factory::NeuronFactory;
    
    #[tokio::test]
    async fn test_self_reorganization() {
        let a2a_protocol = Arc::new(A2AProtocol::new());
        let (network, mut event_rx) = SelfReorganizingNetwork::new(a2a_protocol);
        
        // Create units at different layers
        let factory = NeuronFactory::new(CognitiveConfig::default());
        
        for layer in &[
            CognitiveLayer::Reflexive,
            CognitiveLayer::Implementation,
            CognitiveLayer::Operational,
        ] {
            for _ in 0..3 {
                let unit = factory.create_neuron(*layer).await.unwrap();
                network.add_unit(unit).await.unwrap();
            }
        }
        
        // Process some signals
        let units = network.units.read().await;
        let unit_ids: Vec<Uuid> = units.keys().copied().collect();
        drop(units);
        
        for (i, unit_id) in unit_ids.iter().enumerate().take(20) {
            let input = CognitiveInput {
                content: format!("Test signal {}", i),
                context: HashMap::new(),
                source_layer: None,
            };
            
            network.process_signal(*unit_id, input).await.unwrap();
        }
        
        // Collect events
        let mut events = Vec::new();
        while let Ok(event) = event_rx.try_recv() {
            events.push(event);
        }
        
        // Check that connections were formed
        assert!(!events.is_empty());
        
        // Get report
        let report = network.reorganization_report().await;
        println!("Reorganization Report: {}", report.summary());
        
        assert!(report.total_connections > 0);
        assert!(report.average_connections_per_unit > 0.0);
    }
}