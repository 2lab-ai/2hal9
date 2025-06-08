//! 2HAL9 Server implementation

pub mod api;
pub mod circuit_breaker;
pub mod claude;
pub mod cost_tracker;
pub mod error;
pub mod metrics;
pub mod network;
pub mod neuron;
pub mod performance;
pub mod router;
pub mod server;

pub use server::HAL9Server;
pub use claude::{ClaudeInterface, MockClaude, ClaudeAPIClient};
pub use neuron::{ManagedNeuron, NeuronRegistry};
pub use router::{SignalRouter, RoutingTable};