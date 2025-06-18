//! HAL9 Server implementation

pub mod api;
pub mod api_auth;
pub mod api_codegen;
pub mod auth_middleware;
pub mod cache;
pub mod circuit_breaker;
pub mod claude;
pub mod claude_enhanced;
pub mod connection_pool;
pub mod cost_tracker;
pub mod database;
pub mod database_logging;
pub mod database_runtime;
pub mod enterprise;
pub mod error;
pub mod error_recovery;
pub mod health;
pub mod logging;
pub mod memory_manager;
pub mod metrics;
pub mod middleware;
pub mod network;
pub mod neuron;
pub mod performance;
pub mod prometheus_exporter;
pub mod rate_limiter;
pub mod router;
pub mod scaling;
pub mod server;
pub mod genius_game;
pub mod models;

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