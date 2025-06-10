//! Orchestration Layer - Dynamic topology and coordination
//!
//! This layer manages the dynamic graph structure of cognitive units,
//! handles routing, load balancing, and distributed coordination.

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;
use crate::Result;

pub mod topology;
pub mod flow;
pub mod coordination;
pub mod routing;

pub use topology::*;
pub use flow::*;
pub use coordination::*;
pub use routing::*;

/// Main orchestration interface
#[async_trait]
pub trait Orchestrator: Send + Sync + 'static {
    /// Initialize orchestration
    async fn initialize(&mut self) -> Result<()>;
    
    /// Add a cognitive unit to the orchestration
    async fn add_unit(&mut self, unit: UnitDescriptor) -> Result<Uuid>;
    
    /// Remove a cognitive unit
    async fn remove_unit(&mut self, unit_id: Uuid) -> Result<()>;
    
    /// Connect two units
    async fn connect(&mut self, from: Uuid, to: Uuid, connection: Connection) -> Result<()>;
    
    /// Disconnect two units
    async fn disconnect(&mut self, from: Uuid, to: Uuid) -> Result<()>;
    
    /// Route a signal through the topology
    async fn route(&self, signal: OrchestrationSignal) -> Result<Vec<Uuid>>;
    
    /// Get current topology
    async fn topology(&self) -> Result<TopologySnapshot>;
    
    /// Optimize topology based on performance metrics
    async fn optimize(&mut self) -> Result<OptimizationReport>;
}

/// Unit descriptor for orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitDescriptor {
    pub id: Uuid,
    pub unit_type: UnitType,
    pub layer: crate::hierarchical::cognitive::CognitiveLayer,
    pub capabilities: Vec<Capability>,
    pub resource_requirements: ResourceRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnitType {
    Neuron,
    Cluster,
    Ensemble,
    Module,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub version: String,
    pub performance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: f32,
    pub memory_mb: u64,
    pub bandwidth_mbps: f32,
}

/// Connection between units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub connection_type: ConnectionType,
    pub weight: f32,
    pub latency_ms: f32,
    pub bandwidth_limit: Option<f32>,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    Forward,
    Backward,
    Lateral,
    Recurrent,
}

/// Signal for orchestration routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationSignal {
    pub id: Uuid,
    pub source: Uuid,
    pub signal_type: SignalType,
    pub priority: f32,
    pub payload: serde_json::Value,
    pub routing_hints: RoutingHints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalType {
    Activation,
    Gradient,
    Control,
    Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingHints {
    pub preferred_path: Option<Vec<Uuid>>,
    pub avoid_units: Vec<Uuid>,
    pub max_hops: Option<usize>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

/// Topology snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologySnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub units: HashMap<Uuid, UnitDescriptor>,
    pub connections: Vec<(Uuid, Uuid, Connection)>,
    pub metrics: TopologyMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyMetrics {
    pub total_units: usize,
    pub total_connections: usize,
    pub average_degree: f32,
    pub clustering_coefficient: f32,
    pub diameter: usize,
}

/// Optimization report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    pub changes_made: Vec<TopologyChange>,
    pub performance_improvement: f32,
    pub resource_savings: ResourceSavings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TopologyChange {
    UnitAdded { id: Uuid },
    UnitRemoved { id: Uuid },
    ConnectionAdded { from: Uuid, to: Uuid },
    ConnectionRemoved { from: Uuid, to: Uuid },
    ConnectionWeightChanged { from: Uuid, to: Uuid, old: f32, new: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSavings {
    pub cpu_cores_saved: f32,
    pub memory_mb_saved: u64,
    pub bandwidth_mbps_saved: f32,
}

/// Default orchestrator implementation
pub struct DefaultOrchestrator {
    topology_manager: Box<dyn TopologyManager>,
    flow_controller: Box<dyn FlowController>,
    state_coordinator: Box<dyn StateCoordinator>,
    router: Box<dyn SignalRouter>,
}

impl DefaultOrchestrator {
    pub fn new(
        topology: Box<dyn TopologyManager>,
        flow: Box<dyn FlowController>,
        state: Box<dyn StateCoordinator>,
        router: Box<dyn SignalRouter>,
    ) -> Self {
        Self {
            topology_manager: topology,
            flow_controller: flow,
            state_coordinator: state,
            router,
        }
    }
}

#[async_trait]
impl Orchestrator for DefaultOrchestrator {
    async fn initialize(&mut self) -> Result<()> {
        // Initialize state synchronization
        let initial_state = DistributedState {
            state_id: Uuid::new_v4(),
            version: 0,
            data: HashMap::new(),
            metadata: StateMetadata {
                owner: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                ttl: None,
                replication_factor: 3,
            },
        };
        
        self.state_coordinator.synchronize(initial_state).await?;
        Ok(())
    }
    
    async fn add_unit(&mut self, unit: UnitDescriptor) -> Result<Uuid> {
        let unit_id = unit.id;
        
        // Add to topology
        self.topology_manager.add_node(unit.clone()).await?;
        
        // Update router topology
        self.router.update_topology(routing::TopologyChange::NodeAdded { 
            id: unit_id,
            properties: routing::NodeProperties {
                layer: unit.layer as u8,
                capabilities: unit.capabilities.iter().map(|c| c.name.clone()).collect(),
                capacity: unit.resource_requirements.cpu_cores * 100.0,
            }
        }).await?;
        
        Ok(unit_id)
    }
    
    async fn remove_unit(&mut self, unit_id: Uuid) -> Result<()> {
        // Remove from topology
        self.topology_manager.remove_node(unit_id).await?;
        
        // Update router
        self.router.update_topology(routing::TopologyChange::NodeRemoved { id: unit_id }).await?;
        
        Ok(())
    }
    
    async fn connect(&mut self, from: Uuid, to: Uuid, connection: Connection) -> Result<()> {
        // Add edge to topology
        self.topology_manager.add_edge(from, to, connection.clone()).await?;
        
        // Update router
        self.router.update_topology(routing::TopologyChange::LinkAdded { 
            from, 
            to,
            properties: routing::LinkProperties {
                latency_ms: connection.latency_ms,
                bandwidth_mbps: connection.bandwidth_limit.unwrap_or(1000.0),
                reliability: 0.99,
            }
        }).await?;
        
        Ok(())
    }
    
    async fn disconnect(&mut self, from: Uuid, to: Uuid) -> Result<()> {
        // Remove edge from topology
        self.topology_manager.remove_edge(from, to).await?;
        
        // Update router
        self.router.update_topology(routing::TopologyChange::LinkRemoved { from, to }).await?;
        
        Ok(())
    }
    
    async fn route(&self, signal: OrchestrationSignal) -> Result<Vec<Uuid>> {
        // Convert to routable signal
        let routable = RoutableSignal {
            signal_id: signal.id,
            source: signal.source,
            signal_type: match signal.signal_type {
                SignalType::Activation => routing::SignalType::Activation { layer: 0 },
                SignalType::Gradient => routing::SignalType::Gradient { magnitude: 1.0 },
                SignalType::Control => routing::SignalType::Control { command: "".to_string() },
                SignalType::Data => routing::SignalType::Data { content_type: "json".to_string() },
            },
            payload_size: signal.payload.to_string().len(),
            routing_hints: routing::RoutingHints {
                target_layers: None,
                target_capabilities: None,
                preferred_paths: signal.routing_hints.preferred_path.map(|p| vec![p]),
                qos_requirements: routing::QosRequirements {
                    max_latency_ms: None,
                    min_bandwidth_mbps: None,
                    reliability: None,
                },
            },
        };
        
        // Get routing paths
        let paths = self.router.route(&routable).await?;
        
        // Extract target nodes
        let targets: Vec<Uuid> = paths.into_iter()
            .filter_map(|path| path.path.last().copied())
            .collect();
        
        Ok(targets)
    }
    
    async fn topology(&self) -> Result<TopologySnapshot> {
        let metrics = self.topology_manager.metrics().await?;
        
        // Get all units and connections from state
        let state_snapshot = self.state_coordinator.snapshot().await?;
        
        let units: HashMap<Uuid, UnitDescriptor> = state_snapshot.units.into_iter()
            .map(|(id, _unit_state)| {
                (id, UnitDescriptor {
                    id,
                    unit_type: UnitType::Neuron,
                    layer: crate::hierarchical::cognitive::CognitiveLayer::Reflexive,
                    capabilities: vec![],
                    resource_requirements: ResourceRequirements {
                        cpu_cores: 1.0,
                        memory_mb: 128,
                        bandwidth_mbps: 10.0,
                    },
                })
            })
            .collect();
        
        Ok(TopologySnapshot {
            timestamp: chrono::Utc::now(),
            units,
            connections: vec![], // Would be populated from topology manager
            metrics,
        })
    }
    
    async fn optimize(&mut self) -> Result<OptimizationReport> {
        // Balance load across units
        let load_report = self.flow_controller.balance_load().await?;
        
        // Optimize routing tables
        self.router.optimize().await?;
        
        // Calculate performance improvement
        let performance_improvement = if load_report.load_variance_before > 0.0 {
            (load_report.load_variance_before - load_report.load_variance_after) 
                / load_report.load_variance_before
        } else {
            0.0
        };
        
        Ok(OptimizationReport {
            changes_made: vec![
                TopologyChange::ConnectionWeightChanged {
                    from: Uuid::new_v4(),
                    to: Uuid::new_v4(),
                    old: 1.0,
                    new: 0.8,
                },
            ],
            performance_improvement,
            resource_savings: ResourceSavings {
                cpu_cores_saved: 0.0,
                memory_mb_saved: 0,
                bandwidth_mbps_saved: 0.0,
            },
        })
    }
}

#[cfg(test)]
mod tests;