//! Resource management abstraction

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::Result;

/// Compute resource management abstraction
#[async_trait]
pub trait ComputeResource: Send + Sync + 'static {
    /// Allocate resources for a task
    async fn allocate(&self, request: ResourceRequest) -> Result<ResourceAllocation>;
    
    /// Release allocated resources
    async fn release(&self, allocation: ResourceAllocation) -> Result<()>;
    
    /// Get current resource usage
    async fn usage(&self) -> Result<ResourceUsage>;
    
    /// Get available resources
    async fn available(&self) -> Result<ResourceCapacity>;
    
    /// Set resource limits for a neuron
    async fn set_limits(&self, neuron_id: &str, limits: ResourceLimits) -> Result<()>;
    
    /// Monitor resource usage for a neuron
    async fn monitor(&self, neuron_id: &str) -> Result<ResourceMonitor>;
}

/// Resource allocation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequest {
    pub requester_id: String,
    pub cpu_cores: Option<f32>,
    pub memory_mb: Option<u64>,
    pub gpu_count: Option<u32>,
    pub priority: ResourcePriority,
    pub duration: Option<std::time::Duration>,
}

/// Resource priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourcePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Allocated resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub allocation_id: uuid::Uuid,
    pub allocated_to: String,
    pub cpu_cores: f32,
    pub memory_mb: u64,
    pub gpu_ids: Vec<u32>,
    pub allocated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Current resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub gpu_usage_percent: Vec<f32>,
    pub active_allocations: usize,
}

/// Available resource capacity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapacity {
    pub cpu_cores_total: f32,
    pub cpu_cores_available: f32,
    pub memory_mb_total: u64,
    pub memory_mb_available: u64,
    pub gpu_count_total: u32,
    pub gpu_count_available: u32,
}

/// Resource limits for a neuron
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_cpu_cores: Option<f32>,
    pub max_memory_mb: Option<u64>,
    pub max_gpu_count: Option<u32>,
    pub max_concurrent_tasks: Option<usize>,
}

/// Resource monitor for tracking usage
pub struct ResourceMonitor {
    pub neuron_id: String,
    receiver: tokio::sync::mpsc::Receiver<ResourceMetric>,
}

impl ResourceMonitor {
    pub async fn next_metric(&mut self) -> Option<ResourceMetric> {
        self.receiver.recv().await
    }
}

/// Resource usage metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetric {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_usage: f32,
    pub memory_mb: u64,
    pub gpu_usage: Vec<f32>,
}

/// Local resource manager for single machine
pub struct LocalResources {
    allocations: dashmap::DashMap<uuid::Uuid, ResourceAllocation>,
    limits: dashmap::DashMap<String, ResourceLimits>,
}

impl LocalResources {
    pub fn new() -> Self {
        Self {
            allocations: dashmap::DashMap::new(),
            limits: dashmap::DashMap::new(),
        }
    }
}

/// Cluster resource manager for distributed deployment
pub struct ClusterResources {
    // Would integrate with cluster scheduler
    allocations: dashmap::DashMap<uuid::Uuid, ResourceAllocation>,
}

impl ClusterResources {
    pub fn new() -> Self {
        Self {
            allocations: dashmap::DashMap::new(),
        }
    }
}

/// Kubernetes resource manager
pub struct K8sResources {
    namespace: String,
    // Would integrate with K8s API
}

impl K8sResources {
    pub fn new(namespace: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
        }
    }
}