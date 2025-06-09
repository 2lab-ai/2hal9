//! Protocol Layer - Communication protocols and message handling
//!
//! This layer defines how components communicate, providing versioning,
//! negotiation, and different protocol types for various communication patterns.

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::Result;

pub mod messages;
pub mod negotiation;
pub mod versioning;
pub mod streams;
pub mod signal;
pub mod gradient;
pub mod consensus;
pub mod manager;

#[cfg(test)]
mod tests;

pub use messages::*;
pub use negotiation::*;
pub use versioning::*;
pub use streams::*;
pub use manager::*;

/// Base protocol trait for all communication protocols
#[async_trait]
pub trait Protocol: Send + Sync + 'static {
    /// Protocol identifier
    fn id(&self) -> &str;
    
    /// Protocol version
    fn version(&self) -> ProtocolVersion;
    
    /// Negotiate protocol parameters with peer
    async fn negotiate(&self, peer_capabilities: &ProtocolCapabilities) -> Result<NegotiatedProtocol>;
    
    /// Encode a message for transmission
    async fn encode<M: Message>(&self, message: M) -> Result<Vec<u8>>;
    
    /// Decode a message from transmission
    async fn decode<M: Message>(&self, data: &[u8]) -> Result<M>;
    
    /// Get protocol capabilities
    fn capabilities(&self) -> ProtocolCapabilities;
}

/// Protocol version information
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl ProtocolVersion {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }
    
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major
    }
}

/// Protocol capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCapabilities {
    pub compression: Vec<CompressionType>,
    pub encryption: Vec<EncryptionType>,
    pub max_message_size: usize,
    pub streaming: bool,
    pub bidirectional: bool,
    pub ordered_delivery: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
    Lz4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionType {
    None,
    Tls,
    Aes256,
}

/// Result of protocol negotiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiatedProtocol {
    pub version: ProtocolVersion,
    pub compression: CompressionType,
    pub encryption: EncryptionType,
    pub max_message_size: usize,
}

/// Signal protocol for neuron activation propagation
pub struct SignalProtocol {
    version: ProtocolVersion,
}

impl SignalProtocol {
    pub fn new() -> Self {
        Self {
            version: ProtocolVersion::new(1, 0, 0),
        }
    }
}

/// Gradient protocol for backward propagation
pub struct GradientProtocol {
    version: ProtocolVersion,
}

impl GradientProtocol {
    pub fn new() -> Self {
        Self {
            version: ProtocolVersion::new(1, 0, 0),
        }
    }
}

/// Query protocol for request/response patterns
pub struct QueryProtocol {
    version: ProtocolVersion,
    timeout: std::time::Duration,
}

impl QueryProtocol {
    pub fn new(timeout: std::time::Duration) -> Self {
        Self {
            version: ProtocolVersion::new(1, 0, 0),
            timeout,
        }
    }
}

/// Stream protocol for continuous data flows
pub struct StreamProtocol {
    version: ProtocolVersion,
    buffer_size: usize,
}

impl StreamProtocol {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            version: ProtocolVersion::new(1, 0, 0),
            buffer_size,
        }
    }
}

/// Consensus protocol for distributed agreement
pub struct ConsensusProtocol {
    version: ProtocolVersion,
    quorum_size: usize,
}

impl ConsensusProtocol {
    pub fn new(quorum_size: usize) -> Self {
        Self {
            version: ProtocolVersion::new(1, 0, 0),
            quorum_size,
        }
    }
}