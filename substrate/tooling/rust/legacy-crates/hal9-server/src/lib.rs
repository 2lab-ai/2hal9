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
pub mod database_runtime;
// TODO: Fix database abstraction issues
// pub mod enterprise;
pub mod error;
pub mod memory_manager;
pub mod metrics;
pub mod network;
pub mod neuron;
pub mod performance;
pub mod prometheus_exporter;
pub mod router;
// TODO: Fix database abstraction issues
// pub mod scaling;
pub mod server;

#[cfg(feature = "plugins")]
pub mod plugins;

#[cfg(feature = "blockchain")]
pub mod blockchain;

#[cfg(feature = "graphql")]
pub use api::graphql;

pub use claude::{ClaudeAPIClient, ClaudeInterface, MockClaude};
pub use neuron::{ManagedNeuron, NeuronRegistry};
pub use router::{RoutingTable, SignalRouter};
pub use server::HAL9Server;
