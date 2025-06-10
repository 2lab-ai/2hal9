//! Network layer for distributed neuron communication

pub mod tcp_transport;
pub mod discovery;
pub mod protocol;
pub mod connection_pool;

pub use tcp_transport::{TcpTransport, TransportConfig};
pub use discovery::{ServiceDiscovery, DiscoveryConfig, ServerInfo};
pub use protocol::{NetworkMessage, MessageCodec};
pub use connection_pool::{ConnectionPool, ConnectionManager};