//! Signal Protocol - Forward activation propagation
//!
//! This protocol handles the propagation of activation signals between neurons,
//! supporting both point-to-point and broadcast communication patterns.

use super::{
    CompressionType, EncryptionType, NegotiatedProtocol, Protocol, ProtocolCapabilities,
    ProtocolVersion,
};
use crate::hierarchical::substrate::transport::{DefaultTransport, MessageTransport};
use crate::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use uuid::Uuid;

/// Signal message for neuron activation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMessage {
    pub id: Uuid,
    pub source_neuron: Uuid,
    pub target_neuron: Option<Uuid>, // None for broadcast
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub activation: Activation,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Activation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activation {
    pub content: String,
    pub strength: f32,
    pub decay_rate: f32,
    pub propagation_depth: u32,
}

impl Activation {
    pub fn new(content: String, strength: f32) -> Self {
        Self {
            content,
            strength,
            decay_rate: 0.1, // Default 10% decay per hop
            propagation_depth: 0,
        }
    }

    /// Apply decay for next hop
    pub fn decay(&mut self) {
        self.strength *= 1.0 - self.decay_rate;
        self.propagation_depth += 1;
    }

    /// Check if signal is still strong enough to propagate
    pub fn should_propagate(&self) -> bool {
        self.strength > 0.01 && self.propagation_depth < 10
    }
}

/// Signal protocol implementation
pub struct SignalProtocol {
    version: ProtocolVersion,
    transport: Arc<DefaultTransport>,
    negotiated: Option<NegotiatedProtocol>,
    metrics: Arc<SignalMetrics>,
}

#[derive(Default)]
struct SignalMetrics {
    signals_sent: AtomicU64,
    signals_received: AtomicU64,
    signals_dropped: AtomicU64,
    total_strength: AtomicU64, // Stored as fixed point (x1000)
}

impl SignalProtocol {
    pub fn new(transport: Arc<DefaultTransport>) -> Self {
        Self {
            version: ProtocolVersion::new(1, 0, 0),
            transport,
            negotiated: None,
            metrics: Arc::new(SignalMetrics::default()),
        }
    }

    /// Send a signal to a specific neuron
    pub async fn send_signal(&self, signal: SignalMessage) -> Result<()> {
        // Validate signal
        if !signal.activation.should_propagate() {
            self.metrics.signals_dropped.fetch_add(1, Ordering::Relaxed);
            return Ok(()); // Signal too weak
        }

        // Encode based on negotiated protocol
        let encoded = self.encode_internal(&signal)?;

        // Send via transport
        if let Some(target) = signal.target_neuron {
            // Send to specific neuron
            let destination = format!("neuron:{}", target);
            self.transport.send_raw(&destination, encoded).await?;
        } else {
            // Broadcast to all
            self.transport
                .publish_raw("broadcast:signals", encoded)
                .await?;
        }

        // Update metrics
        self.metrics.signals_sent.fetch_add(1, Ordering::Relaxed);
        self.metrics.total_strength.fetch_add(
            (signal.activation.strength * 1000.0) as u64,
            Ordering::Relaxed,
        );

        Ok(())
    }

    /// Broadcast a signal to all neurons
    pub async fn broadcast_signal(&self, mut signal: SignalMessage) -> Result<()> {
        signal.target_neuron = None;
        self.send_signal(signal).await
    }

    /// Receive signals for a neuron
    pub async fn receive_signals(&self, neuron_id: Uuid) -> Result<SignalReceiver> {
        let endpoint = format!("neuron:{}", neuron_id);
        let receiver = self.transport.receive_raw(&endpoint).await?;

        Ok(SignalReceiver {
            receiver,
            protocol: self,
        })
    }

    /// Subscribe to broadcast signals
    pub async fn subscribe_broadcasts(&self) -> Result<SignalReceiver> {
        let receiver = self.transport.subscribe_raw("broadcast:signals").await?;

        Ok(SignalReceiver {
            receiver,
            protocol: self,
        })
    }

    fn encode_internal(&self, signal: &SignalMessage) -> Result<Vec<u8>> {
        let data = bincode::serialize(signal).map_err(|e| Error::Serialization(e.to_string()))?;

        // Apply compression if negotiated
        let compressed = if let Some(neg) = &self.negotiated {
            match neg.compression {
                CompressionType::Gzip => {
                    use flate2::write::GzEncoder;
                    use flate2::Compression;
                    use std::io::Write;

                    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
                    encoder
                        .write_all(&data)
                        .map_err(|e| Error::Protocol(format!("Compression failed: {}", e)))?;
                    encoder
                        .finish()
                        .map_err(|e| Error::Protocol(format!("Compression failed: {}", e)))?
                }
                CompressionType::Lz4 => lz4_flex::compress_prepend_size(&data),
                _ => data,
            }
        } else {
            data
        };

        Ok(compressed)
    }

    fn decode_internal(&self, data: &[u8]) -> Result<SignalMessage> {
        // Decompress if needed
        let decompressed = if let Some(neg) = &self.negotiated {
            match neg.compression {
                CompressionType::Gzip => {
                    use flate2::read::GzDecoder;
                    use std::io::Read;

                    let mut decoder = GzDecoder::new(data);
                    let mut decompressed = Vec::new();
                    decoder
                        .read_to_end(&mut decompressed)
                        .map_err(|e| Error::Protocol(format!("Decompression failed: {}", e)))?;
                    decompressed
                }
                CompressionType::Lz4 => lz4_flex::decompress_size_prepended(data)
                    .map_err(|e| Error::Protocol(format!("Decompression failed: {}", e)))?,
                _ => data.to_vec(),
            }
        } else {
            data.to_vec()
        };

        let signal = bincode::deserialize(&decompressed)
            .map_err(|e| Error::Deserialization(e.to_string()))?;

        // Update metrics
        self.metrics
            .signals_received
            .fetch_add(1, Ordering::Relaxed);

        Ok(signal)
    }

    /// Get protocol metrics
    pub fn metrics(&self) -> SignalProtocolMetrics {
        let sent = self.metrics.signals_sent.load(Ordering::Relaxed);
        let received = self.metrics.signals_received.load(Ordering::Relaxed);
        let dropped = self.metrics.signals_dropped.load(Ordering::Relaxed);
        let total_strength = self.metrics.total_strength.load(Ordering::Relaxed) as f32 / 1000.0;

        SignalProtocolMetrics {
            signals_sent: sent,
            signals_received: received,
            signals_dropped: dropped,
            average_strength: if sent > 0 {
                total_strength / sent as f32
            } else {
                0.0
            },
            efficiency: if sent > 0 {
                (sent - dropped) as f32 / sent as f32
            } else {
                1.0
            },
        }
    }
}

/// Receiver for signal messages
pub struct SignalReceiver<'a> {
    receiver: crate::hierarchical::substrate::transport::RawTransportReceiver,
    protocol: &'a SignalProtocol,
}

impl SignalReceiver<'_> {
    pub async fn recv(&mut self) -> Option<SignalMessage> {
        let data = self.receiver.recv().await?;
        match self.protocol.decode_internal(&data) {
            Ok(signal) => Some(signal),
            Err(e) => {
                tracing::error!("Failed to decode signal: {}", e);
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SignalProtocolMetrics {
    pub signals_sent: u64,
    pub signals_received: u64,
    pub signals_dropped: u64,
    pub average_strength: f32,
    pub efficiency: f32,
}

#[async_trait]
impl Protocol for SignalProtocol {
    fn id(&self) -> &str {
        "signal-protocol"
    }

    fn version(&self) -> ProtocolVersion {
        self.version.clone()
    }

    async fn negotiate(
        &self,
        peer_capabilities: &ProtocolCapabilities,
    ) -> Result<NegotiatedProtocol> {
        // Choose best compression
        let compression = if peer_capabilities
            .compression
            .contains(&CompressionType::Lz4)
        {
            CompressionType::Lz4
        } else if peer_capabilities
            .compression
            .contains(&CompressionType::Gzip)
        {
            CompressionType::Gzip
        } else {
            CompressionType::None
        };

        let negotiated = NegotiatedProtocol {
            version: self.version.clone(),
            compression,
            encryption: EncryptionType::None, // Signals are not sensitive
            max_message_size: peer_capabilities.max_message_size.min(1_000_000), // 1MB max
        };

        Ok(negotiated)
    }

    async fn encode_raw(&self, _message_type: &str, _data: Vec<u8>) -> Result<Vec<u8>> {
        Err(Error::Protocol(
            "Use send_signal for signal protocol".to_string(),
        ))
    }

    async fn decode_raw(&self, _data: &[u8]) -> Result<(String, Vec<u8>)> {
        Err(Error::Protocol(
            "Use receive_signals for signal protocol".to_string(),
        ))
    }

    fn capabilities(&self) -> ProtocolCapabilities {
        ProtocolCapabilities {
            compression: vec![
                CompressionType::None,
                CompressionType::Gzip,
                CompressionType::Lz4,
            ],
            encryption: vec![EncryptionType::None],
            max_message_size: 1_000_000, // 1MB
            streaming: false,
            bidirectional: true,
            ordered_delivery: false, // Signals can arrive out of order
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hierarchical::substrate::transport::ChannelTransport;

    #[tokio::test]
    async fn test_signal_activation() {
        let mut activation = Activation::new("test".to_string(), 1.0);
        assert!(activation.should_propagate());

        // Test decay
        for _ in 0..5 {
            activation.decay();
        }
        assert!(activation.strength < 0.6);
        assert_eq!(activation.propagation_depth, 5);

        // Test propagation limit
        for _ in 0..10 {
            activation.decay();
        }
        assert!(!activation.should_propagate()); // Too many hops
    }

    #[tokio::test]
    async fn test_signal_protocol() {
        let transport = Arc::new(ChannelTransport::new());
        let protocol = SignalProtocol::new(transport.clone());

        // Set up receiver
        let neuron_id = Uuid::new_v4();
        let mut receiver = protocol.receive_signals(neuron_id).await.unwrap();

        // Send signal
        let signal = SignalMessage {
            id: Uuid::new_v4(),
            source_neuron: Uuid::new_v4(),
            target_neuron: Some(neuron_id),
            timestamp: chrono::Utc::now(),
            activation: Activation::new("Hello neurons!".to_string(), 0.8),
            metadata: [("type".to_string(), "test".to_string())]
                .into_iter()
                .collect(),
        };

        protocol.send_signal(signal.clone()).await.unwrap();

        // Receive signal
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.id, signal.id);
        assert_eq!(received.activation.content, "Hello neurons!");

        // Check metrics
        let metrics = protocol.metrics();
        assert_eq!(metrics.signals_sent, 1);
        assert_eq!(metrics.signals_received, 1);
        assert_eq!(metrics.efficiency, 1.0);
    }

    #[tokio::test]
    async fn test_broadcast_signals() {
        let transport = Arc::new(ChannelTransport::new());
        let protocol = SignalProtocol::new(transport.clone());

        // Set up multiple subscribers
        let mut sub1 = protocol.subscribe_broadcasts().await.unwrap();
        let mut sub2 = protocol.subscribe_broadcasts().await.unwrap();

        // Broadcast signal
        let signal = SignalMessage {
            id: Uuid::new_v4(),
            source_neuron: Uuid::new_v4(),
            target_neuron: None,
            timestamp: chrono::Utc::now(),
            activation: Activation::new("Broadcast test".to_string(), 0.9),
            metadata: std::collections::HashMap::new(),
        };

        protocol.broadcast_signal(signal.clone()).await.unwrap();

        // Both should receive
        let recv1 = sub1.recv().await.unwrap();
        let recv2 = sub2.recv().await.unwrap();

        assert_eq!(recv1.id, signal.id);
        assert_eq!(recv2.id, signal.id);
    }
}
