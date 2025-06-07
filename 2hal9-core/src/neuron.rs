//! Neuron abstraction and interfaces

use async_trait::async_trait;
use crate::{Result, NeuronSignal};

/// Neuron identifier type
pub type NeuronId = String;

/// Neural network layers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    L4, // Strategic
    L3, // Design
    L2, // Implementation
    L1, // Execution (future)
}

impl Layer {
    /// Parse layer from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "L4" => Some(Layer::L4),
            "L3" => Some(Layer::L3),
            "L2" => Some(Layer::L2),
            "L1" => Some(Layer::L1),
            _ => None,
        }
    }
    
    /// Get layer name
    pub fn as_str(&self) -> &'static str {
        match self {
            Layer::L4 => "L4",
            Layer::L3 => "L3",
            Layer::L2 => "L2",
            Layer::L1 => "L1",
        }
    }
    
    /// Get layer description
    pub fn description(&self) -> &'static str {
        match self {
            Layer::L4 => "Strategic Layer",
            Layer::L3 => "Design Layer",
            Layer::L2 => "Implementation Layer",
            Layer::L1 => "Execution Layer",
        }
    }
}

/// Neuron state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeuronState {
    Starting,
    Running,
    Processing,
    Failed,
    Stopped,
}

/// Neuron health information
#[derive(Debug, Clone)]
pub struct NeuronHealth {
    pub state: NeuronState,
    pub last_signal: Option<chrono::DateTime<chrono::Utc>>,
    pub signals_processed: u64,
    pub errors_count: u64,
    pub uptime_seconds: u64,
}

/// Common interface for all neuron implementations
#[async_trait]
pub trait NeuronInterface: Send + Sync {
    /// Get neuron ID
    fn id(&self) -> &str;
    
    /// Get neuron layer
    fn layer(&self) -> Layer;
    
    /// Process an incoming signal
    async fn process_signal(&self, signal: &NeuronSignal) -> Result<String>;
    
    /// Get current health status
    async fn health(&self) -> Result<NeuronHealth>;
    
    /// Shutdown the neuron gracefully
    async fn shutdown(&self) -> Result<()>;
}