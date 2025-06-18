//! Neuron abstraction and interfaces

use async_trait::async_trait;
use crate::{Result, NeuronSignal};
use std::fmt;

/// Neuron identifier type
pub type NeuronId = String;

/// Alias for NeuronInterface for backwards compatibility
pub use NeuronInterface as Neuron;

/// Neural network layers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    L9, // Universal
    L8, // Visionary  
    L7, // Business
    L6, // Executive
    L5, // Strategic
    L4, // Tactical
    L3, // Operational
    L2, // Implementation
    L1, // Reflexive
}

impl Layer {
    /// Parse layer from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "L9" => Some(Layer::L9),
            "L8" => Some(Layer::L8),
            "L7" => Some(Layer::L7),
            "L6" => Some(Layer::L6),
            "L5" => Some(Layer::L5),
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
            Layer::L9 => "L9",
            Layer::L8 => "L8",
            Layer::L7 => "L7",
            Layer::L6 => "L6",
            Layer::L5 => "L5",
            Layer::L4 => "L4",
            Layer::L3 => "L3",
            Layer::L2 => "L2",
            Layer::L1 => "L1",
        }
    }
    
    /// Get layer description
    pub fn description(&self) -> &'static str {
        match self {
            Layer::L9 => "Universal Layer",
            Layer::L8 => "Visionary Layer",
            Layer::L7 => "Business Layer",
            Layer::L6 => "Executive Layer",
            Layer::L5 => "Strategic Layer",
            Layer::L4 => "Tactical Layer",
            Layer::L3 => "Operational Layer",
            Layer::L2 => "Implementation Layer",
            Layer::L1 => "Reflexive Layer",
        }
    }
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Neuron state
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NeuronState {
    Starting,
    Running,
    Processing,
    Failed,
    Stopped,
}

/// Neuron health information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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