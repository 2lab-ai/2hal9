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
pub use signal::{SignalProtocol, SignalMessage, Activation};
pub use gradient::{GradientProtocol, GradientMessage, Gradient};
pub use consensus::{ConsensusProtocol, ConsensusMessage};

/// Base protocol trait for all communication protocols
#[async_trait]
pub trait Protocol: Send + Sync + 'static {
    /// Protocol identifier
    fn id(&self) -> &str;
    
    /// Protocol version
    fn version(&self) -> ProtocolVersion;
    
    /// Negotiate protocol parameters with peer
    async fn negotiate(&self, peer_capabilities: &ProtocolCapabilities) -> Result<NegotiatedProtocol>;
    
    /// Encode raw message bytes for transmission
    async fn encode_raw(&self, message_type: &str, data: Vec<u8>) -> Result<Vec<u8>>;
    
    /// Decode raw message bytes from transmission
    async fn decode_raw(&self, data: &[u8]) -> Result<(String, Vec<u8>)>;
    
    /// Get protocol capabilities
    fn capabilities(&self) -> ProtocolCapabilities;
}

/// Helper trait for typed protocol operations
#[async_trait]
pub trait TypedProtocol: Protocol {
    /// Encode a typed message
    async fn encode<M: Message>(&self, message: M) -> Result<Vec<u8>> {
        let data = bincode::serialize(&message)
            .map_err(|e| crate::Error::Serialization(e.to_string()))?;
        self.encode_raw(std::any::type_name::<M>(), data).await
    }
    
    /// Decode a typed message
    async fn decode<M: Message>(&self, data: &[u8]) -> Result<M> {
        let (_msg_type, raw_data) = self.decode_raw(data).await?;
        bincode::deserialize(&raw_data)
            .map_err(|e| crate::Error::Deserialization(e.to_string()))
    }
}

/// Automatically implement TypedProtocol for all Protocol types
impl<T: Protocol> TypedProtocol for T {}

/// Protocol version information
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
    Lz4,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

// Signal and Gradient protocols are defined in their respective modules

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

// Consensus protocol is defined in consensus.rs module