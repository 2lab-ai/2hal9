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