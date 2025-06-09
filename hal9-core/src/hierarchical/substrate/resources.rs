//! Resource management abstraction

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;
use sysinfo::{System, SystemExt, CpuExt};
use crate::{Result, Error};

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

/// System resource tracker
struct SystemTracker {
    system: parking_lot::Mutex<System>,
    total_cpu_cores: f32,
    total_memory_mb: u64,
}

impl SystemTracker {
    fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        let total_cpu_cores = system.cpus().len() as f32;
        let total_memory_mb = system.total_memory() / 1024 / 1024;
        
        Self {
            system: parking_lot::Mutex::new(system),
            total_cpu_cores,
            total_memory_mb,
        }
    }
    
    fn get_usage(&self) -> (f32, u64) {
        let mut system = self.system.lock();
        system.refresh_cpu();
        system.refresh_memory();
        
        let cpu_usage = system.global_cpu_info().cpu_usage();
        let memory_used_mb = system.used_memory() / 1024 / 1024;
        
        (cpu_usage, memory_used_mb)
    }
    
    fn get_available(&self) -> (f32, u64) {
        let (cpu_usage, memory_used_mb) = self.get_usage();
        
        let cpu_available = self.total_cpu_cores * (1.0 - cpu_usage / 100.0);
        let memory_available_mb = self.total_memory_mb.saturating_sub(memory_used_mb);
        
        (cpu_available, memory_available_mb)
    }
}

/// Local resource manager for single machine
pub struct LocalResources {
    allocations: Arc<dashmap::DashMap<uuid::Uuid, ResourceAllocation>>,
    limits: Arc<dashmap::DashMap<String, ResourceLimits>>,
    neuron_usage: Arc<dashmap::DashMap<String, ResourceUsage>>,
    monitors: Arc<dashmap::DashMap<String, mpsc::Sender<ResourceMetric>>>,
    system_tracker: Arc<SystemTracker>,
    allocated_cpu: AtomicU64, // Stored as millicores (1000 = 1 core)
    allocated_memory: AtomicU64,
}

impl LocalResources {
    pub fn new() -> Self {
        let resources = Self {
            allocations: Arc::new(dashmap::DashMap::new()),
            limits: Arc::new(dashmap::DashMap::new()),
            neuron_usage: Arc::new(dashmap::DashMap::new()),
            monitors: Arc::new(dashmap::DashMap::new()),
            system_tracker: Arc::new(SystemTracker::new()),
            allocated_cpu: AtomicU64::new(0),
            allocated_memory: AtomicU64::new(0),
        };
        
        // Start background monitoring
        resources.start_monitoring();
        
        resources
    }
    
    fn start_monitoring(&self) {
        let allocations = Arc::clone(&self.allocations);
        let monitors = Arc::clone(&self.monitors);
        let system_tracker = Arc::clone(&self.system_tracker);
        let neuron_usage = Arc::clone(&self.neuron_usage);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            
            loop {
                interval.tick().await;
                
                // Get system usage
                let (cpu_usage, memory_mb) = system_tracker.get_usage();
                
                // Send metrics to all monitors
                let metric = ResourceMetric {
                    timestamp: chrono::Utc::now(),
                    cpu_usage,
                    memory_mb,
                    gpu_usage: vec![],
                };
                
                let mut to_remove = Vec::new();
                for entry in monitors.iter() {
                    if entry.value().send(metric.clone()).await.is_err() {
                        to_remove.push(entry.key().clone());
                    }
                }
                
                // Remove closed monitors
                for key in to_remove {
                    monitors.remove(&key);
                }
                
                // Clean up expired allocations
                let now = chrono::Utc::now();
                allocations.retain(|_, alloc| {
                    alloc.expires_at.map_or(true, |exp| exp > now)
                });
            }
        });
    }
    
    fn check_limits(&self, neuron_id: &str, request: &ResourceRequest) -> Result<()> {
        if let Some(limits) = self.limits.get(neuron_id) {
            if let Some(max_cpu) = limits.max_cpu_cores {
                if request.cpu_cores.unwrap_or(0.0) > max_cpu {
                    return Err(Error::ResourceExhausted(
                        format!("CPU request exceeds limit: {} > {}", 
                            request.cpu_cores.unwrap_or(0.0), max_cpu)
                    ));
                }
            }
            
            if let Some(max_memory) = limits.max_memory_mb {
                if request.memory_mb.unwrap_or(0) > max_memory {
                    return Err(Error::ResourceExhausted(
                        format!("Memory request exceeds limit: {} > {}", 
                            request.memory_mb.unwrap_or(0), max_memory)
                    ));
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ComputeResource for LocalResources {
    async fn allocate(&self, request: ResourceRequest) -> Result<ResourceAllocation> {
        // Check limits
        self.check_limits(&request.requester_id, &request)?;
        
        // Get available resources
        let (cpu_available, memory_available_mb) = self.system_tracker.get_available();
        
        // Check if we have enough resources
        let cpu_requested = request.cpu_cores.unwrap_or(0.1);
        let memory_requested = request.memory_mb.unwrap_or(100);
        
        let cpu_allocated_cores = self.allocated_cpu.load(Ordering::Relaxed) as f32 / 1000.0;
        let memory_allocated_mb = self.allocated_memory.load(Ordering::Relaxed);
        
        if cpu_allocated_cores + cpu_requested > cpu_available {
            return Err(Error::ResourceExhausted(
                format!("Insufficient CPU: requested {}, available {}", 
                    cpu_requested, cpu_available - cpu_allocated_cores)
            ));
        }
        
        if memory_allocated_mb + memory_requested > memory_available_mb {
            return Err(Error::ResourceExhausted(
                format!("Insufficient memory: requested {}MB, available {}MB", 
                    memory_requested, memory_available_mb - memory_allocated_mb)
            ));
        }
        
        // Create allocation
        let allocation = ResourceAllocation {
            allocation_id: uuid::Uuid::new_v4(),
            allocated_to: request.requester_id.clone(),
            cpu_cores: cpu_requested,
            memory_mb: memory_requested,
            gpu_ids: vec![],
            allocated_at: chrono::Utc::now(),
            expires_at: request.duration.map(|d| chrono::Utc::now() + chrono::Duration::from_std(d).unwrap()),
        };
        
        // Update allocated resources
        self.allocated_cpu.fetch_add((cpu_requested * 1000.0) as u64, Ordering::Relaxed);
        self.allocated_memory.fetch_add(memory_requested, Ordering::Relaxed);
        
        // Store allocation
        self.allocations.insert(allocation.allocation_id, allocation.clone());
        
        Ok(allocation)
    }
    
    async fn release(&self, allocation: ResourceAllocation) -> Result<()> {
        if self.allocations.remove(&allocation.allocation_id).is_some() {
            // Free allocated resources
            self.allocated_cpu.fetch_sub((allocation.cpu_cores * 1000.0) as u64, Ordering::Relaxed);
            self.allocated_memory.fetch_sub(allocation.memory_mb, Ordering::Relaxed);
            Ok(())
        } else {
            Err(Error::ResourceNotFound("Allocation not found".to_string()))
        }
    }
    
    async fn usage(&self) -> Result<ResourceUsage> {
        let (cpu_usage, memory_used_mb) = self.system_tracker.get_usage();
        
        Ok(ResourceUsage {
            cpu_usage_percent: cpu_usage,
            memory_used_mb,
            memory_total_mb: self.system_tracker.total_memory_mb,
            gpu_usage_percent: vec![],
            active_allocations: self.allocations.len(),
        })
    }
    
    async fn available(&self) -> Result<ResourceCapacity> {
        let (cpu_available, memory_available_mb) = self.system_tracker.get_available();
        
        let cpu_allocated_cores = self.allocated_cpu.load(Ordering::Relaxed) as f32 / 1000.0;
        let memory_allocated_mb = self.allocated_memory.load(Ordering::Relaxed);
        
        Ok(ResourceCapacity {
            cpu_cores_total: self.system_tracker.total_cpu_cores,
            cpu_cores_available: cpu_available - cpu_allocated_cores,
            memory_mb_total: self.system_tracker.total_memory_mb,
            memory_mb_available: memory_available_mb - memory_allocated_mb,
            gpu_count_total: 0,
            gpu_count_available: 0,
        })
    }
    
    async fn set_limits(&self, neuron_id: &str, limits: ResourceLimits) -> Result<()> {
        self.limits.insert(neuron_id.to_string(), limits);
        Ok(())
    }
    
    async fn monitor(&self, neuron_id: &str) -> Result<ResourceMonitor> {
        let (tx, rx) = mpsc::channel(100);
        self.monitors.insert(neuron_id.to_string(), tx);
        
        Ok(ResourceMonitor {
            neuron_id: neuron_id.to_string(),
            receiver: rx,
        })
    }
}

/// Cluster resource manager for distributed deployment
pub struct ClusterResources {
    allocations: Arc<dashmap::DashMap<uuid::Uuid, ResourceAllocation>>,
    node_resources: Arc<dashmap::DashMap<String, NodeResources>>,
    scheduler: Arc<ClusterScheduler>,
}

#[derive(Debug, Clone)]
struct NodeResources {
    node_id: String,
    cpu_cores: f32,
    memory_mb: u64,
    gpu_count: u32,
    allocated_cpu: f32,
    allocated_memory: u64,
    allocated_gpu: u32,
}

struct ClusterScheduler {
    strategy: SchedulingStrategy,
}

#[derive(Debug, Clone, Copy)]
enum SchedulingStrategy {
    BestFit,
    FirstFit,
    RoundRobin,
    LeastLoaded,
}

impl ClusterScheduler {
    fn select_node(
        &self,
        nodes: &dashmap::DashMap<String, NodeResources>,
        request: &ResourceRequest,
    ) -> Option<String> {
        let cpu_needed = request.cpu_cores.unwrap_or(0.1);
        let memory_needed = request.memory_mb.unwrap_or(100);
        let gpu_needed = request.gpu_count.unwrap_or(0);
        
        match self.strategy {
            SchedulingStrategy::FirstFit => {
                for entry in nodes.iter() {
                    let node = entry.value();
                    if node.can_allocate(cpu_needed, memory_needed, gpu_needed) {
                        return Some(node.node_id.clone());
                    }
                }
                None
            }
            SchedulingStrategy::LeastLoaded => {
                let mut best_node = None;
                let mut best_load = f32::MAX;
                
                for entry in nodes.iter() {
                    let node = entry.value();
                    if node.can_allocate(cpu_needed, memory_needed, gpu_needed) {
                        let load = node.get_load_factor();
                        if load < best_load {
                            best_load = load;
                            best_node = Some(node.node_id.clone());
                        }
                    }
                }
                
                best_node
            }
            _ => None, // Other strategies not implemented yet
        }
    }
}

impl NodeResources {
    fn can_allocate(&self, cpu: f32, memory: u64, gpu: u32) -> bool {
        self.cpu_cores - self.allocated_cpu >= cpu &&
        self.memory_mb - self.allocated_memory >= memory &&
        self.gpu_count - self.allocated_gpu >= gpu
    }
    
    fn get_load_factor(&self) -> f32 {
        let cpu_load = self.allocated_cpu / self.cpu_cores;
        let memory_load = self.allocated_memory as f32 / self.memory_mb as f32;
        (cpu_load + memory_load) / 2.0
    }
}

impl ClusterResources {
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(dashmap::DashMap::new()),
            node_resources: Arc::new(dashmap::DashMap::new()),
            scheduler: Arc::new(ClusterScheduler {
                strategy: SchedulingStrategy::LeastLoaded,
            }),
        }
    }
    
    pub fn register_node(&self, node_id: &str, cpu_cores: f32, memory_mb: u64, gpu_count: u32) {
        let node = NodeResources {
            node_id: node_id.to_string(),
            cpu_cores,
            memory_mb,
            gpu_count,
            allocated_cpu: 0.0,
            allocated_memory: 0,
            allocated_gpu: 0,
        };
        
        self.node_resources.insert(node_id.to_string(), node);
    }
}

#[async_trait]
impl ComputeResource for ClusterResources {
    async fn allocate(&self, request: ResourceRequest) -> Result<ResourceAllocation> {
        let node_id = self.scheduler.select_node(&self.node_resources, &request)
            .ok_or_else(|| Error::ResourceExhausted("No suitable node found".to_string()))?;
        
        let mut node = self.node_resources.get_mut(&node_id)
            .ok_or_else(|| Error::ResourceNotFound("Node not found".to_string()))?;
        
        let cpu_requested = request.cpu_cores.unwrap_or(0.1);
        let memory_requested = request.memory_mb.unwrap_or(100);
        let gpu_requested = request.gpu_count.unwrap_or(0);
        
        // Double-check allocation is still possible
        if !node.can_allocate(cpu_requested, memory_requested, gpu_requested) {
            return Err(Error::ResourceExhausted("Node resources changed".to_string()));
        }
        
        // Update node resources
        node.allocated_cpu += cpu_requested;
        node.allocated_memory += memory_requested;
        node.allocated_gpu += gpu_requested;
        
        let allocation = ResourceAllocation {
            allocation_id: uuid::Uuid::new_v4(),
            allocated_to: request.requester_id.clone(),
            cpu_cores: cpu_requested,
            memory_mb: memory_requested,
            gpu_ids: (0..gpu_requested).collect(),
            allocated_at: chrono::Utc::now(),
            expires_at: request.duration.map(|d| chrono::Utc::now() + chrono::Duration::from_std(d).unwrap()),
        };
        
        self.allocations.insert(allocation.allocation_id, allocation.clone());
        
        Ok(allocation)
    }
    
    async fn release(&self, allocation: ResourceAllocation) -> Result<()> {
        if let Some((_id, alloc)) = self.allocations.remove(&allocation.allocation_id) {
            // Find which node had this allocation
            // In real implementation, would track this mapping
            for mut entry in self.node_resources.iter_mut() {
                let node = entry.value_mut();
                // Assume we can identify the node somehow
                node.allocated_cpu -= alloc.cpu_cores;
                node.allocated_memory -= alloc.memory_mb;
                node.allocated_gpu -= alloc.gpu_ids.len() as u32;
                break;
            }
            Ok(())
        } else {
            Err(Error::ResourceNotFound("Allocation not found".to_string()))
        }
    }
    
    async fn usage(&self) -> Result<ResourceUsage> {
        let mut total_cpu = 0.0;
        let mut allocated_cpu = 0.0;
        let mut total_memory = 0;
        let mut allocated_memory = 0;
        
        for entry in self.node_resources.iter() {
            let node = entry.value();
            total_cpu += node.cpu_cores;
            allocated_cpu += node.allocated_cpu;
            total_memory += node.memory_mb;
            allocated_memory += node.allocated_memory;
        }
        
        let cpu_usage_percent = if total_cpu > 0.0 {
            (allocated_cpu / total_cpu) * 100.0
        } else {
            0.0
        };
        
        Ok(ResourceUsage {
            cpu_usage_percent,
            memory_used_mb: allocated_memory,
            memory_total_mb: total_memory,
            gpu_usage_percent: vec![],
            active_allocations: self.allocations.len(),
        })
    }
    
    async fn available(&self) -> Result<ResourceCapacity> {
        let mut total_cpu = 0.0;
        let mut available_cpu = 0.0;
        let mut total_memory = 0;
        let mut available_memory = 0;
        let mut total_gpu = 0;
        let mut available_gpu = 0;
        
        for entry in self.node_resources.iter() {
            let node = entry.value();
            total_cpu += node.cpu_cores;
            available_cpu += node.cpu_cores - node.allocated_cpu;
            total_memory += node.memory_mb;
            available_memory += node.memory_mb - node.allocated_memory;
            total_gpu += node.gpu_count;
            available_gpu += node.gpu_count - node.allocated_gpu;
        }
        
        Ok(ResourceCapacity {
            cpu_cores_total: total_cpu,
            cpu_cores_available: available_cpu,
            memory_mb_total: total_memory,
            memory_mb_available: available_memory,
            gpu_count_total: total_gpu,
            gpu_count_available: available_gpu,
        })
    }
    
    async fn set_limits(&self, _neuron_id: &str, _limits: ResourceLimits) -> Result<()> {
        // In cluster mode, limits would be enforced at the orchestrator level
        Ok(())
    }
    
    async fn monitor(&self, neuron_id: &str) -> Result<ResourceMonitor> {
        // In cluster mode, monitoring would be done through metrics aggregation
        let (_tx, rx) = mpsc::channel(100);
        
        Ok(ResourceMonitor {
            neuron_id: neuron_id.to_string(),
            receiver: rx,
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_local_resources_allocation() {
        let resources = LocalResources::new();
        
        // Request resources
        let request = ResourceRequest {
            requester_id: "test-neuron".to_string(),
            cpu_cores: Some(0.5),
            memory_mb: Some(512),
            gpu_count: None,
            priority: ResourcePriority::Normal,
            duration: Some(Duration::from_secs(60)),
        };
        
        let allocation = resources.allocate(request).await.unwrap();
        assert_eq!(allocation.cpu_cores, 0.5);
        assert_eq!(allocation.memory_mb, 512);
        
        // Check available resources decreased
        let capacity = resources.available().await.unwrap();
        assert!(capacity.cpu_cores_available < capacity.cpu_cores_total);
        
        // Release allocation
        resources.release(allocation).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_resource_limits() {
        let resources = LocalResources::new();
        
        // Set limits
        let limits = ResourceLimits {
            max_cpu_cores: Some(0.2),
            max_memory_mb: Some(256),
            max_gpu_count: None,
            max_concurrent_tasks: None,
        };
        
        resources.set_limits("limited-neuron", limits).await.unwrap();
        
        // Try to exceed limits
        let request = ResourceRequest {
            requester_id: "limited-neuron".to_string(),
            cpu_cores: Some(0.5),
            memory_mb: Some(512),
            gpu_count: None,
            priority: ResourcePriority::Normal,
            duration: None,
        };
        
        let result = resources.allocate(request).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_cluster_resources() {
        let cluster = ClusterResources::new();
        
        // Register nodes
        cluster.register_node("node1", 4.0, 8192, 0);
        cluster.register_node("node2", 8.0, 16384, 2);
        
        // Check total capacity
        let capacity = cluster.available().await.unwrap();
        assert_eq!(capacity.cpu_cores_total, 12.0);
        assert_eq!(capacity.memory_mb_total, 24576);
        assert_eq!(capacity.gpu_count_total, 2);
        
        // Allocate resources
        let request = ResourceRequest {
            requester_id: "test-app".to_string(),
            cpu_cores: Some(2.0),
            memory_mb: Some(4096),
            gpu_count: Some(1),
            priority: ResourcePriority::High,
            duration: None,
        };
        
        let allocation = cluster.allocate(request).await.unwrap();
        assert_eq!(allocation.gpu_ids.len(), 1);
    }
}