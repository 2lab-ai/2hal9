//! Network topology and agent placement management

use dashmap::DashMap;
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{AgentLevel, AgentProfile, NetworkLayer, ContextWindow};

/// Network topology manager
pub struct NetworkTopology {
    graph: Arc<RwLock<DiGraph<AgentNode, ConnectionEdge>>>,
    agent_indices: Arc<DashMap<Uuid, NodeIndex>>,
    layer_groups: Arc<DashMap<NetworkLayer, Vec<Uuid>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentNode {
    pub id: Uuid,
    pub level: AgentLevel,
    pub layer: NetworkLayer,
    pub context_window: ContextWindow,
    pub connections_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionEdge {
    pub strength: f32,
    pub interaction_count: u64,
    pub last_interaction: chrono::DateTime<chrono::Utc>,
}

impl NetworkTopology {
    pub fn new() -> Self {
        Self {
            graph: Arc::new(RwLock::new(DiGraph::new())),
            agent_indices: Arc::new(DashMap::new()),
            layer_groups: Arc::new(DashMap::new()),
        }
    }
    
    /// Place an agent in the network
    pub async fn place_agent(&self, profile: &AgentProfile) -> NetworkPosition {
        let layer = profile.capability_level.layer();
        let context_size = match profile.context_window {
            ContextWindow::Small(s) | ContextWindow::Medium(s) | 
            ContextWindow::Large(s) | ContextWindow::XLarge(s) | 
            ContextWindow::Default(s) => s,
        };
        
        // Create agent node
        let node = AgentNode {
            id: profile.id,
            level: profile.capability_level,
            layer,
            context_window: profile.context_window,
            connections_count: 0,
        };
        
        // Add to graph
        let mut graph = self.graph.write().await;
        let node_idx = graph.add_node(node.clone());
        drop(graph);
        
        // Update indices
        self.agent_indices.insert(profile.id, node_idx);
        
        // Add to layer group
        self.layer_groups
            .entry(layer)
            .or_insert_with(Vec::new)
            .push(profile.id);
        
        // Find optimal connections
        let connections = self.find_optimal_connections(&profile).await;
        
        // Establish connections
        for target_id in &connections {
            self.connect_agents(profile.id, *target_id, 0.5).await;
        }
        
        NetworkPosition {
            agent_id: profile.id,
            layer,
            context_window_size: context_size,
            initial_connections: connections,
            position_quality: self.calculate_position_quality(&profile).await,
        }
    }
    
    /// Find optimal connections for a new agent
    async fn find_optimal_connections(&self, profile: &AgentProfile) -> Vec<Uuid> {
        let mut connections = Vec::new();
        let target_layer = profile.capability_level.layer();
        
        // Connect to agents in the same layer (peers)
        if let Some(layer_agents) = self.layer_groups.get(&target_layer) {
            let peers: Vec<_> = layer_agents.iter()
                .filter(|&id| *id != profile.id)
                .take(3) // Connect to up to 3 peers
                .cloned()
                .collect();
            connections.extend(peers);
        }
        
        // Connect to adjacent layers (±1 rule)
        let adjacent_layers = self.get_adjacent_layers(target_layer);
        for adj_layer in adjacent_layers {
            if let Some(layer_agents) = self.layer_groups.get(&adj_layer) {
                if let Some(&mentor) = layer_agents.first() {
                    connections.push(mentor);
                }
            }
        }
        
        // Ensure minimum connectivity
        if connections.len() < 2 {
            // Find any available agents
            let graph = self.graph.read().await;
            for node_idx in graph.node_indices() {
                if connections.len() >= 3 {
                    break;
                }
                if let Some(node) = graph.node_weight(node_idx) {
                    if node.id != profile.id && !connections.contains(&node.id) {
                        connections.push(node.id);
                    }
                }
            }
        }
        
        connections
    }
    
    /// Connect two agents
    pub async fn connect_agents(&self, agent1: Uuid, agent2: Uuid, initial_strength: f32) {
        let idx1 = self.agent_indices.get(&agent1).map(|v| *v);
        let idx2 = self.agent_indices.get(&agent2).map(|v| *v);
        
        if let (Some(idx1), Some(idx2)) = (idx1, idx2) {
            let mut graph = self.graph.write().await;
            
            let edge = ConnectionEdge {
                strength: initial_strength,
                interaction_count: 0,
                last_interaction: chrono::Utc::now(),
            };
            
            graph.add_edge(idx1, idx2, edge.clone());
            graph.add_edge(idx2, idx1, edge); // Bidirectional
            
            // Update connection counts
            if let Some(node1) = graph.node_weight_mut(idx1) {
                node1.connections_count += 1;
            }
            if let Some(node2) = graph.node_weight_mut(idx2) {
                node2.connections_count += 1;
            }
        }
    }
    
    /// Update connection strength based on interaction
    pub async fn update_connection(&self, agent1: Uuid, agent2: Uuid, success: bool) {
        let idx1 = self.agent_indices.get(&agent1).map(|v| *v);
        let idx2 = self.agent_indices.get(&agent2).map(|v| *v);
        
        if let (Some(idx1), Some(idx2)) = (idx1, idx2) {
            let mut graph = self.graph.write().await;
            
            // Update edge from agent1 to agent2
            if let Some(edge_idx) = graph.find_edge(idx1, idx2) {
                if let Some(edge) = graph.edge_weight_mut(edge_idx) {
                    edge.interaction_count += 1;
                    edge.last_interaction = chrono::Utc::now();
                    
                    // Adjust strength based on success
                    if success {
                        edge.strength = (edge.strength * 1.1).min(1.0);
                    } else {
                        edge.strength = (edge.strength * 0.9).max(0.1);
                    }
                }
            }
        }
    }
    
    /// Remove an agent from the network
    pub async fn remove_agent(&self, agent_id: Uuid) {
        if let Some((_, node_idx)) = self.agent_indices.remove(&agent_id) {
            let mut graph = self.graph.write().await;
            
            // Get the agent's layer before removal
            let layer = graph.node_weight(node_idx).map(|n| n.layer);
            
            // Remove from graph
            graph.remove_node(node_idx);
            
            // Remove from layer groups
            if let Some(layer) = layer {
                if let Some(mut layer_agents) = self.layer_groups.get_mut(&layer) {
                    layer_agents.retain(|&id| id != agent_id);
                }
            }
        }
    }
    
    /// Get network statistics for a specific layer
    pub async fn layer_statistics(&self, layer: NetworkLayer) -> LayerStats {
        let graph = self.graph.read().await;
        let layer_agents = self.layer_groups.get(&layer)
            .map(|v| v.clone())
            .unwrap_or_default();
        
        let mut total_connections = 0;
        let mut avg_level = 0.0;
        
        for agent_id in &layer_agents {
            if let Some(&node_idx) = self.agent_indices.get(agent_id) {
                if let Some(node) = graph.node_weight(node_idx) {
                    total_connections += node.connections_count;
                    avg_level += node.level.value() as f32;
                }
            }
        }
        
        let agent_count = layer_agents.len();
        if agent_count > 0 {
            avg_level /= agent_count as f32;
        }
        
        LayerStats {
            layer,
            agent_count,
            average_level: avg_level,
            total_connections,
            connectivity_ratio: if agent_count > 1 {
                total_connections as f32 / (agent_count * (agent_count - 1)) as f32
            } else {
                0.0
            },
        }
    }
    
    /// Calculate position quality for an agent
    async fn calculate_position_quality(&self, profile: &AgentProfile) -> f32 {
        let graph = self.graph.read().await;
        
        if let Some(&node_idx) = self.agent_indices.get(&profile.id) {
            let connections = graph.edges(node_idx).count();
            let optimal_connections = match profile.capability_level.layer() {
                NetworkLayer::Basic => 2.0,
                NetworkLayer::Intermediate => 4.0,
                NetworkLayer::Advanced => 6.0,
                NetworkLayer::Expert => 8.0,
                NetworkLayer::Probationary => 1.0,
            };
            
            // Quality based on connection count relative to optimal
            let connection_quality = 1.0 - ((connections as f32 - optimal_connections).abs() / optimal_connections).min(1.0);
            
            // Factor in connection strengths
            let mut strength_sum = 0.0;
            for edge in graph.edges(node_idx) {
                strength_sum += edge.weight().strength;
            }
            let avg_strength = if connections > 0 {
                strength_sum / connections as f32
            } else {
                0.0
            };
            
            // Combined quality score
            connection_quality * 0.6 + avg_strength * 0.4
        } else {
            0.0
        }
    }
    
    fn get_adjacent_layers(&self, layer: NetworkLayer) -> Vec<NetworkLayer> {
        match layer {
            NetworkLayer::Basic => vec![NetworkLayer::Intermediate],
            NetworkLayer::Intermediate => vec![NetworkLayer::Basic, NetworkLayer::Advanced],
            NetworkLayer::Advanced => vec![NetworkLayer::Intermediate, NetworkLayer::Expert],
            NetworkLayer::Expert => vec![NetworkLayer::Advanced],
            NetworkLayer::Probationary => vec![NetworkLayer::Basic],
        }
    }
    
    /// Export network topology for visualization
    pub async fn export_topology(&self) -> NetworkVisualization {
        let graph = self.graph.read().await;
        
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        // Export nodes
        for node_idx in graph.node_indices() {
            if let Some(node) = graph.node_weight(node_idx) {
                nodes.push(VisualizationNode {
                    id: node.id.to_string(),
                    label: format!("L{}", node.level.value()),
                    layer: node.layer,
                    size: node.connections_count as f32,
                });
            }
        }
        
        // Export edges
        for edge_ref in graph.edge_references() {
            let source_idx = edge_ref.source();
            let target_idx = edge_ref.target();
            
            if let (Some(source), Some(target)) = (
                graph.node_weight(source_idx),
                graph.node_weight(target_idx)
            ) {
                edges.push(VisualizationEdge {
                    source: source.id.to_string(),
                    target: target.id.to_string(),
                    weight: edge_ref.weight().strength,
                    interactions: edge_ref.weight().interaction_count,
                });
            }
        }
        
        NetworkVisualization {
            nodes,
            edges,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Network position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPosition {
    pub agent_id: Uuid,
    pub layer: NetworkLayer,
    pub context_window_size: usize,
    pub initial_connections: Vec<Uuid>,
    pub position_quality: f32,
}

/// Layer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStats {
    pub layer: NetworkLayer,
    pub agent_count: usize,
    pub average_level: f32,
    pub total_connections: usize,
    pub connectivity_ratio: f32,
}

/// Network visualization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkVisualization {
    pub nodes: Vec<VisualizationNode>,
    pub edges: Vec<VisualizationEdge>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationNode {
    pub id: String,
    pub label: String,
    pub layer: NetworkLayer,
    pub size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationEdge {
    pub source: String,
    pub target: String,
    pub weight: f32,
    pub interactions: u64,
}

/// Context window manager for dynamic allocation
pub struct ContextWindowManager {
    allocations: Arc<DashMap<Uuid, ContextAllocation>>,
    total_memory: usize,
    used_memory: Arc<RwLock<usize>>,
}

#[derive(Clone)]
struct ContextAllocation {
    agent_id: Uuid,
    allocated_size: usize,
    utilization: f32,
}

impl ContextWindowManager {
    pub fn new(total_memory: usize) -> Self {
        Self {
            allocations: Arc::new(DashMap::new()),
            total_memory,
            used_memory: Arc::new(RwLock::new(0)),
        }
    }
    
    pub async fn allocate_context(&self, agent: &AgentProfile) -> Result<ContextWindow, String> {
        let requested_size = match agent.capability_level.value() {
            1..=5 => 4096,
            6..=10 => 16384,
            11..=15 => 65536,
            16..=20 => 131072,
            _ => 8192,
        };
        
        let mut used = self.used_memory.write().await;
        
        if *used + requested_size > self.total_memory {
            // Try to free up memory from underutilized agents
            self.reclaim_unused_memory().await;
            
            let current_used = *used;
            if current_used + requested_size > self.total_memory {
                return Err("Insufficient memory".to_string());
            }
        }
        
        *used += requested_size;
        drop(used);
        
        let allocation = ContextAllocation {
            agent_id: agent.id,
            allocated_size: requested_size,
            utilization: 0.0,
        };
        
        self.allocations.insert(agent.id, allocation);
        
        Ok(match agent.capability_level.value() {
            1..=5 => ContextWindow::Small(requested_size),
            6..=10 => ContextWindow::Medium(requested_size),
            11..=15 => ContextWindow::Large(requested_size),
            16..=20 => ContextWindow::XLarge(requested_size),
            _ => ContextWindow::Default(requested_size),
        })
    }
    
    async fn reclaim_unused_memory(&self) {
        let mut to_reclaim = Vec::new();
        
        for entry in self.allocations.iter() {
            if entry.value().utilization < 0.2 {
                to_reclaim.push((*entry.key(), entry.value().allocated_size));
            }
        }
        
        let mut used = self.used_memory.write().await;
        for (agent_id, size) in to_reclaim {
            if let Some((_, allocation)) = self.allocations.remove(&agent_id) {
                *used -= allocation.allocated_size;
            }
        }
    }
    
    pub async fn update_utilization(&self, agent_id: Uuid, utilization: f32) {
        if let Some(mut allocation) = self.allocations.get_mut(&agent_id) {
            allocation.utilization = utilization;
        }
    }
    
    pub async fn deallocate(&self, agent_id: Uuid) {
        if let Some((_, allocation)) = self.allocations.remove(&agent_id) {
            let mut used = self.used_memory.write().await;
            *used -= allocation.allocated_size;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_topology() {
        let topology = NetworkTopology::new();
        
        // Create test profiles
        let profiles = vec![
            AgentProfile {
                id: Uuid::new_v4(),
                capability_level: AgentLevel::new(5).unwrap(),
                context_window: ContextWindow::Small(4096),
                specialization: vec!["test".to_string()],
                timestamp: chrono::Utc::now(),
            },
            AgentProfile {
                id: Uuid::new_v4(),
                capability_level: AgentLevel::new(10).unwrap(),
                context_window: ContextWindow::Medium(16384),
                specialization: vec!["test".to_string()],
                timestamp: chrono::Utc::now(),
            },
            AgentProfile {
                id: Uuid::new_v4(),
                capability_level: AgentLevel::new(15).unwrap(),
                context_window: ContextWindow::Large(65536),
                specialization: vec!["test".to_string()],
                timestamp: chrono::Utc::now(),
            },
        ];
        
        // Place agents
        for profile in &profiles {
            let position = topology.place_agent(profile).await;
            assert!(position.position_quality >= 0.0);
            assert!(position.position_quality <= 1.0);
        }
        
        // Check layer statistics
        let basic_stats = topology.layer_statistics(NetworkLayer::Basic).await;
        assert_eq!(basic_stats.agent_count, 1);
        
        let intermediate_stats = topology.layer_statistics(NetworkLayer::Intermediate).await;
        assert_eq!(intermediate_stats.agent_count, 1);
        
        // Test connection updates
        topology.update_connection(profiles[0].id, profiles[1].id, true).await;
        
        // Export visualization
        let viz = topology.export_topology().await;
        assert_eq!(viz.nodes.len(), 3);
        assert!(viz.edges.len() > 0);
    }

    #[tokio::test]
    async fn test_context_window_manager() {
        let manager = ContextWindowManager::new(1_000_000); // 1MB total
        
        let profile1 = AgentProfile {
            id: Uuid::new_v4(),
            capability_level: AgentLevel::new(5).unwrap(),
            context_window: ContextWindow::Small(4096),
            specialization: vec![],
            timestamp: chrono::Utc::now(),
        };
        
        let profile2 = AgentProfile {
            id: Uuid::new_v4(),
            capability_level: AgentLevel::new(15).unwrap(),
            context_window: ContextWindow::Large(65536),
            specialization: vec![],
            timestamp: chrono::Utc::now(),
        };
        
        // Test allocation
        let ctx1 = manager.allocate_context(&profile1).await.unwrap();
        match ctx1 {
            ContextWindow::Small(size) => assert_eq!(size, 4096),
            _ => panic!("Wrong context window type"),
        }
        
        let ctx2 = manager.allocate_context(&profile2).await.unwrap();
        match ctx2 {
            ContextWindow::Large(size) => assert_eq!(size, 65536),
            _ => panic!("Wrong context window type"),
        }
        
        // Test deallocation
        manager.deallocate(profile1.id).await;
        
        // Verify memory is freed
        let used = *manager.used_memory.read().await;
        assert_eq!(used, 65536);
    }
}