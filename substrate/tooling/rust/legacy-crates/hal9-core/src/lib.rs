//! Core types and abstractions for 2HAL9 neural network

pub mod auth;
pub mod config;
pub mod error;
pub mod learning;
pub mod mcp;
pub mod memory;
pub mod neuron;
pub mod signal;

// Hierarchical architecture modules
pub mod hierarchical;

// Migration infrastructure
pub mod migration;

pub use config::{NeuronConfig, ServerConfig};
pub use error::{Error, Result};
pub use neuron::{Layer, NeuronId, NeuronInterface};
pub use signal::{Activation, Gradient, NeuronSignal, PropagationType, SignalPayload};
