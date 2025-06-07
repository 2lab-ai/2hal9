//! Core types and abstractions for 2HAL9 neural network

pub mod error;
pub mod signal;
pub mod config;
pub mod neuron;

pub use error::{Error, Result};
pub use signal::{NeuronSignal, PropagationType, SignalPayload, Activation, Gradient};
pub use config::{ServerConfig, NeuronConfig};
pub use neuron::{NeuronInterface, NeuronId, Layer};