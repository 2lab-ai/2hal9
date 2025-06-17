//! Network layer for distributed neuron communication

pub mod connection_pool;
pub mod discovery;
pub mod protocol;
pub mod tcp_transport;

pub use connection_pool::{ConnectionManager, ConnectionPool};
pub use discovery::{DiscoveryConfig, ServerInfo, ServiceDiscovery};
pub use protocol::{MessageCodec, NetworkMessage};
pub use tcp_transport::{TcpTransport, TransportConfig};
