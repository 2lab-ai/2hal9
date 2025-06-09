//! Substrate Layer - Foundation abstractions for runtime, transport, and storage
//!
//! This layer provides the fundamental computational resources abstracted from 
//! implementation details. It allows HAL9 to run on different infrastructures
//! without changing higher layers.

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::Result;

pub mod runtime;
pub mod transport;
pub mod storage;
pub mod resources;

#[cfg(test)]
mod tests;

pub use runtime::*;
pub use transport::*;
pub use storage::*;
pub use resources::*;

/// Main substrate abstraction that combines all foundational components
#[async_trait]
pub trait Substrate: Send + Sync + 'static {
    /// Associated runtime for async execution
    type Runtime: AsyncRuntime;
    
    /// Associated transport for message passing
    type Transport: MessageTransport;
    
    /// Associated storage for persistence
    type Storage: PersistentStorage;
    
    /// Associated resource manager
    type Resource: ComputeResource;
    
    /// Initialize the substrate
    async fn initialize(&mut self) -> Result<()>;
    
    /// Get runtime instance
    fn runtime(&self) -> &Self::Runtime;
    
    /// Get transport instance
    fn transport(&self) -> &Self::Transport;
    
    /// Get storage instance
    fn storage(&self) -> &Self::Storage;
    
    /// Get resource manager
    fn resources(&self) -> &Self::Resource;
    
    /// Shutdown the substrate gracefully
    async fn shutdown(&mut self) -> Result<()>;
}

/// Substrate capabilities that can be queried
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstrateCapabilities {
    pub distributed: bool,
    pub persistent_storage: bool,
    pub gpu_support: bool,
    pub max_concurrent_tasks: usize,
    pub network_protocols: Vec<String>,
    pub storage_backends: Vec<String>,
}

/// Local substrate implementation for single-machine deployment
pub struct LocalSubstrate {
    runtime: runtime::TokioRuntime,
    transport: transport::ChannelTransport,
    storage: storage::SqliteStorage,
    resources: resources::LocalResources,
}

/// Distributed substrate for multi-node deployment
pub struct DistributedSubstrate {
    runtime: runtime::TokioRuntime,
    transport: transport::TcpTransport,
    storage: storage::PostgresStorage,
    resources: resources::ClusterResources,
}

/// Cloud-native substrate for Kubernetes deployment
pub struct CloudSubstrate {
    runtime: runtime::TokioRuntime,
    transport: transport::GrpcTransport,
    storage: storage::S3Storage,
    resources: resources::K8sResources,
}