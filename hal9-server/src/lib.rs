//! 2HAL9 Server implementation

pub mod api;
pub mod api_auth;
pub mod auth_middleware;
pub mod circuit_breaker;
pub mod claude;
pub mod cost_tracker;
pub mod error;
pub mod memory_manager;
pub mod metrics;
pub mod network;
pub mod neuron;
pub mod performance;
pub mod prometheus_exporter;
pub mod router;
pub mod server;

pub use server::HAL9Server;
pub use claude::{ClaudeInterface, MockClaude, ClaudeAPIClient};
pub use neuron::{ManagedNeuron, NeuronRegistry};
pub use router::{SignalRouter, RoutingTable};