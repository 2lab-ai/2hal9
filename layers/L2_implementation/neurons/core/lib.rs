//! Core types and abstractions for 2HAL9 neural network

pub mod error;
pub mod signal;
pub mod config;
pub mod neuron;
pub mod mcp;
pub mod memory;
pub mod learning;
pub mod auth;

// Hierarchical architecture modules
pub mod hierarchical;

// Migration infrastructure
pub mod migration;

// Consciousness measurement
pub mod consciousness;

// Performance optimizations
pub mod performance;

pub use error::{Error, Result};
pub use signal::{NeuronSignal, PropagationType, SignalPayload, Activation, Gradient, Signal};
pub use config::{ServerConfig, NeuronConfig};
pub use neuron::{NeuronInterface, NeuronId, Layer, Neuron};