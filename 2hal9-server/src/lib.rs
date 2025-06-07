//! 2HAL9 Server implementation

pub mod claude;
pub mod neuron;
pub mod router;
pub mod server;
pub mod metrics;

pub use server::HAL9Server;
pub use claude::{ClaudeInterface, MockClaude, ClaudeAPIClient};
pub use neuron::{ManagedNeuron, NeuronRegistry};
pub use router::{SignalRouter, RoutingTable};