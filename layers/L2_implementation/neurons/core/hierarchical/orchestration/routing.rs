//! Signal routing algorithms and strategies

use async_trait::async_trait;
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Ordering;
use uuid::Uuid;
use crate::Result;
pub use super::{TopologyChange, RoutingHints};

/// Signal router for intelligent message routing
#[async_trait]
pub trait SignalRouter: Send + Sync {
    /// Route a signal to appropriate targets
    async fn route(&self, signal: &RoutableSignal) -> Result<Vec<RoutingPath>>;
    
    /// Update routing tables based on topology changes
    async fn update_topology(&mut self, change: TopologyChange) -> Result<()>;
    
    /// Get routing statistics
    async fn statistics(&self) -> Result<RoutingStatistics>;
    
    /// Optimize routing tables
    async fn optimize(&mut self) -> Result<()>;
}

/// Signal that can be routed
#[derive(Debug, Clone)]
pub struct RoutableSignal {
    pub signal_id: Uuid,
    pub source: Uuid,
    pub signal_type: SignalType,
    pub payload_size: usize,
    pub routing_hints: RoutingHints,
}

#[derive(Debug, Clone)]
pub enum SignalType {
    Activation { layer: u8 },
    Gradient { magnitude: f32 },
    Control { command: String },
    Data { content_type: String },
}

// RoutingHints is imported from mod.rs

#[derive(Debug, Clone)]
pub struct QosRequirements {
    pub max_latency_ms: Option<f32>,
    pub min_bandwidth_mbps: Option<f32>,
    pub reliability: Option<f32>,
}

// TopologyChange is imported from mod.rs

#[derive(Debug, Clone)]
pub struct NodeProperties {
    pub layer: u8,
    pub capabilities: HashSet<String>,
    pub capacity: f32,
}

impl Default for NodeProperties {
    fn default() -> Self {
        Self {
            layer: 0,
            capabilities: HashSet::new(),
            capacity: 100.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinkProperties {
    pub latency_ms: f32,
    pub bandwidth_mbps: f32,
    pub reliability: f32,
}

impl Default for LinkProperties {
    fn default() -> Self {
        Self {
            latency_ms: 1.0,
            bandwidth_mbps: 1000.0,
            reliability: 0.99,
        }
    }
}

/// Routing path with metadata
#[derive(Debug, Clone)]
pub struct RoutingPath {
    pub path: Vec<Uuid>,
    pub total_latency_ms: f32,
    pub min_bandwidth_mbps: f32,
    pub reliability: f32,
    pub cost: f32,
}

/// Routing statistics
#[derive(Debug, Clone)]
pub struct RoutingStatistics {
    pub total_routed: u64,
    pub failed_routes: u64,
    pub average_path_length: f32,
    pub average_latency_ms: f32,
    pub cache_hit_rate: f32,
}

/// Dijkstra-based router implementation
pub struct DijkstraRouter {
    graph: Graph,
    cache: RoutingCache,
    statistics: RoutingStatistics,
}

struct Graph {
    nodes: HashMap<Uuid, NodeProperties>,
    adjacency: HashMap<Uuid, HashMap<Uuid, LinkProperties>>,
}

struct RoutingCache {
    paths: HashMap<(Uuid, Uuid), RoutingPath>,
    capacity: usize,
}

impl DijkstraRouter {
    pub fn new(cache_capacity: usize) -> Self {
        Self {
            graph: Graph {
                nodes: HashMap::new(),
                adjacency: HashMap::new(),
            },
            cache: RoutingCache {
                paths: HashMap::new(),
                capacity: cache_capacity,
            },
            statistics: RoutingStatistics {
                total_routed: 0,
                failed_routes: 0,
                average_path_length: 0.0,
                average_latency_ms: 0.0,
                cache_hit_rate: 0.0,
            },
        }
    }
    
    fn find_shortest_path(&self, source: Uuid, targets: &[Uuid], requirements: &QosRequirements) -> Option<RoutingPath> {
        #[derive(Clone, PartialEq)]
        struct State {
            cost: OrderedFloat,
            node: Uuid,
            path: Vec<Uuid>,
            latency: f32,
            bandwidth: f32,
        }
        
        impl Eq for State {}
        
        impl Ord for State {
            fn cmp(&self, other: &Self) -> Ordering {
                other.cost.cmp(&self.cost)
            }
        }
        
        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        
        let mut heap = BinaryHeap::new();
        let mut visited = HashSet::new();
        
        heap.push(State {
            cost: OrderedFloat(0.0),
            node: source,
            path: vec![source],
            latency: 0.0,
            bandwidth: f32::MAX,
        });
        
        while let Some(State { cost, node, path, latency, bandwidth }) = heap.pop() {
            if targets.contains(&node) && path.len() > 1 {
                // Check if path meets requirements
                if let Some(max_latency) = requirements.max_latency_ms {
                    if latency > max_latency {
                        continue;
                    }
                }
                
                if let Some(min_bandwidth) = requirements.min_bandwidth_mbps {
                    if bandwidth < min_bandwidth {
                        continue;
                    }
                }
                
                return Some(RoutingPath {
                    path,
                    total_latency_ms: latency,
                    min_bandwidth_mbps: bandwidth,
                    reliability: 0.99, // Simplified
                    cost: cost.0,
                });
            }
            
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node);
            
            if let Some(neighbors) = self.graph.adjacency.get(&node) {
                for (next_node, link) in neighbors {
                    if visited.contains(next_node) {
                        continue;
                    }
                    
                    let mut next_path = path.clone();
                    next_path.push(*next_node);
                    
                    let next_latency = latency + link.latency_ms;
                    let next_bandwidth = bandwidth.min(link.bandwidth_mbps);
                    let next_cost = cost.0 + self.calculate_link_cost(link, requirements);
                    
                    heap.push(State {
                        cost: OrderedFloat(next_cost),
                        node: *next_node,
                        path: next_path,
                        latency: next_latency,
                        bandwidth: next_bandwidth,
                    });
                }
            }
        }
        
        None
    }
    
    fn calculate_link_cost(&self, link: &LinkProperties, requirements: &QosRequirements) -> f32 {
        let mut cost = link.latency_ms;
        
        // Factor in bandwidth if required
        if requirements.min_bandwidth_mbps.is_some() {
            cost += 100.0 / link.bandwidth_mbps;
        }
        
        // Factor in reliability if required
        if requirements.reliability.is_some() {
            cost += 10.0 * (1.0 - link.reliability);
        }
        
        cost
    }
    
    fn get_target_nodes(&self, hints: &RoutingHints) -> Vec<Uuid> {
        let mut targets = Vec::new();
        
        // If preferred path is specified, use the last node as target
        if let Some(ref path) = hints.preferred_path {
            if let Some(target) = path.last() {
                targets.push(*target);
                return targets;
            }
        }
        
        // Otherwise return all nodes except those to avoid
        for node_id in self.graph.nodes.keys() {
            if !hints.avoid_units.contains(node_id) {
                targets.push(*node_id);
            }
        }
        
        targets
    }
}

#[async_trait]
impl SignalRouter for DijkstraRouter {
    async fn route(&self, signal: &RoutableSignal) -> Result<Vec<RoutingPath>> {
        // Check cache first
        let targets = self.get_target_nodes(&signal.routing_hints);
        let mut paths = Vec::new();
        
        for target in targets {
            let cache_key = (signal.source, target);
            
            if let Some(cached_path) = self.cache.paths.get(&cache_key) {
                paths.push(cached_path.clone());
            } else if let Some(path) = self.find_shortest_path(
                signal.source, 
                &[target], 
                &QosRequirements {
                    max_latency_ms: None,
                    min_bandwidth_mbps: None,
                    reliability: None,
                }
            ) {
                paths.push(path);
            }
        }
        
        Ok(paths)
    }
    
    async fn update_topology(&mut self, change: TopologyChange) -> Result<()> {
        match change {
            TopologyChange::UnitAdded { id } => {
                // Create default properties for the unit
                self.graph.nodes.insert(id, NodeProperties::default());
                self.graph.adjacency.insert(id, HashMap::new());
            }
            TopologyChange::UnitRemoved { id } => {
                self.graph.nodes.remove(&id);
                self.graph.adjacency.remove(&id);
                // Remove all edges to this node
                for (_, neighbors) in self.graph.adjacency.iter_mut() {
                    neighbors.remove(&id);
                }
                // Clear cache entries involving this node
                self.cache.paths.retain(|(from, to), _| *from != id && *to != id);
            }
            TopologyChange::ConnectionAdded { from, to } => {
                // Create default link properties
                self.graph.adjacency.entry(from)
                    .or_default()
                    .insert(to, LinkProperties::default());
            }
            TopologyChange::ConnectionRemoved { from, to } => {
                if let Some(neighbors) = self.graph.adjacency.get_mut(&from) {
                    neighbors.remove(&to);
                }
                // Clear cache entries using this link
                self.cache.paths.retain(|(_f, _t), path| {
                    !path.path.windows(2).any(|w| w[0] == from && w[1] == to)
                });
            }
            TopologyChange::ConnectionWeightChanged { from, to, old: _, new: weight } => {
                if let Some(neighbors) = self.graph.adjacency.get_mut(&from) {
                    if let Some(link) = neighbors.get_mut(&to) {
                        // Update the weight/bandwidth based on the new weight
                        link.bandwidth_mbps = weight * 100.0; // Convert weight to bandwidth
                    }
                }
                // Clear cache entries using this link
                self.cache.paths.retain(|(_f, _t), path| {
                    !path.path.windows(2).any(|w| w[0] == from && w[1] == to)
                });
            }
        }
        Ok(())
    }
    
    async fn statistics(&self) -> Result<RoutingStatistics> {
        Ok(self.statistics.clone())
    }
    
    async fn optimize(&mut self) -> Result<()> {
        // Prune cache of least recently used entries
        if self.cache.paths.len() > self.cache.capacity {
            // Simple strategy: clear half the cache
            let to_remove = self.cache.paths.len() / 2;
            let keys_to_remove: Vec<_> = self.cache.paths.keys()
                .take(to_remove)
                .cloned()
                .collect();
                
            for key in keys_to_remove {
                self.cache.paths.remove(&key);
            }
        }
        
        Ok(())
    }
}

// Helper type for ordered floats in heap
#[derive(Clone, PartialEq)]
struct OrderedFloat(f32);

impl Eq for OrderedFloat {}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Hierarchical router for multi-level routing
#[allow(dead_code)]
pub struct HierarchicalRouter {
    level_routers: HashMap<u8, Box<dyn SignalRouter>>,
    inter_level_links: HashMap<(u8, u8), Vec<(Uuid, Uuid)>>,
}

impl Default for HierarchicalRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl HierarchicalRouter {
    pub fn new() -> Self {
        Self {
            level_routers: HashMap::new(),
            inter_level_links: HashMap::new(),
        }
    }
    
    pub fn add_level(&mut self, level: u8, router: Box<dyn SignalRouter>) {
        self.level_routers.insert(level, router);
    }
}