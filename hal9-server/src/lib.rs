//! HAL9 Server implementation

pub mod api;
pub mod api_auth;
pub mod api_codegen;
pub mod auth_middleware;
pub mod cache;
pub mod circuit_breaker;
pub mod claude;
pub mod cost_tracker;
pub mod database;
pub mod enterprise;
pub mod error;
pub mod memory_manager;
pub mod metrics;
pub mod network;
pub mod neuron;
pub mod performance;
pub mod prometheus_exporter;
pub mod router;
pub mod scaling;
pub mod server;

#[cfg(feature = "plugins")]
pub mod plugins;

#[cfg(feature = "blockchain")]
pub mod blockchain;

#[cfg(feature = "graphql")]
pub mod api_graphql;

pub use server::HAL9Server;
pub use claude::{ClaudeInterface, MockClaude, ClaudeAPIClient};
pub use neuron::{ManagedNeuron, NeuronRegistry};
pub use router::{SignalRouter, RoutingTable};