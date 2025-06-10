//! Gradient Protocol - Backward error propagation and learning
//!
//! This protocol handles the propagation of learning signals (gradients) backward
//! through the network, enabling distributed learning and adaptation.

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use uuid::Uuid;
use crate::{Result, Error};
use crate::hierarchical::substrate::transport::{DefaultTransport, TypedTransport};
use super::{Protocol, ProtocolVersion, ProtocolCapabilities, NegotiatedProtocol, CompressionType, EncryptionType};

/// Gradient message for backward propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientMessage {
    pub id: Uuid,
    pub source_neuron: Uuid,
    pub target_neuron: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub gradient: Gradient,
    pub learning_context: LearningContext,
}

/// Gradient data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gradient {
    pub error: f32,
    pub direction: Vec<f32>, // Gradient vector
    pub magnitude: f32,
    pub accumulated_steps: u32,
}

impl Gradient {
    pub fn new(error: f32, direction: Vec<f32>) -> Self {
        let magnitude = direction.iter().map(|x| x * x).sum::<f32>().sqrt();
        Self {
            error,
            direction,
            magnitude,
            accumulated_steps: 0,
        }
    }
    
    /// Accumulate gradient for batch processing
    pub fn accumulate(&mut self, other: &Gradient) {
        if self.direction.len() != other.direction.len() {
            tracing::warn!("Gradient dimension mismatch");
            return;
        }
        
        for (i, val) in other.direction.iter().enumerate() {
            self.direction[i] += val;
        }
        
        self.error += other.error;
        self.accumulated_steps += 1;
        self.magnitude = self.direction.iter().map(|x| x * x).sum::<f32>().sqrt();
    }
    
    /// Apply gradient clipping
    pub fn clip(&mut self, max_norm: f32) {
        if self.magnitude > max_norm {
            let scale = max_norm / self.magnitude;
            for val in &mut self.direction {
                *val *= scale;
            }
            self.magnitude = max_norm;
        }
    }
    
    /// Check if gradient is significant enough to propagate
    pub fn is_significant(&self) -> bool {
        self.magnitude > 0.001
    }
}

/// Learning context for gradient application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningContext {
    pub learning_rate: f32,
    pub momentum: f32,
    pub batch_size: u32,
    pub epoch: u32,
    pub loss_type: LossType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LossType {
    MeanSquaredError,
    CrossEntropy,
    Huber,
    Custom(String),
}

/// Gradient accumulator for batch processing
pub struct GradientAccumulator {
    gradients: HashMap<Uuid, Vec<Gradient>>,
    batch_size: usize,
    auto_flush: bool,
}

impl GradientAccumulator {
    pub fn new(batch_size: usize, auto_flush: bool) -> Self {
        Self {
            gradients: HashMap::new(),
            batch_size,
            auto_flush,
        }
    }
    
    pub fn add(&mut self, neuron_id: Uuid, gradient: Gradient) -> Option<Gradient> {
        let entry = self.gradients.entry(neuron_id).or_default();
        entry.push(gradient);
        
        if self.auto_flush && entry.len() >= self.batch_size {
            self.flush_neuron(neuron_id)
        } else {
            None
        }
    }
    
    pub fn flush_neuron(&mut self, neuron_id: Uuid) -> Option<Gradient> {
        self.gradients.remove(&neuron_id).map(|grads| {
            let mut accumulated = grads[0].clone();
            for grad in grads.iter().skip(1) {
                accumulated.accumulate(grad);
            }
            
            // Average the accumulated gradient
            if accumulated.accumulated_steps > 0 {
                let factor = 1.0 / accumulated.accumulated_steps as f32;
                accumulated.error *= factor;
                for val in &mut accumulated.direction {
                    *val *= factor;
                }
                accumulated.magnitude *= factor;
            }
            
            accumulated
        })
    }
    
    pub fn flush_all(&mut self) -> HashMap<Uuid, Gradient> {
        let neuron_ids: Vec<_> = self.gradients.keys().cloned().collect();
        neuron_ids.into_iter()
            .filter_map(|id| self.flush_neuron(id).map(|g| (id, g)))
            .collect()
    }
}

/// Gradient protocol implementation
pub struct GradientProtocol {
    version: ProtocolVersion,
    transport: Arc<DefaultTransport>,
    negotiated: Option<NegotiatedProtocol>,
    accumulator: parking_lot::Mutex<GradientAccumulator>,
    metrics: Arc<GradientMetrics>,
}

#[derive(Default)]
struct GradientMetrics {
    gradients_sent: AtomicU64,
    gradients_received: AtomicU64,
    gradients_accumulated: AtomicU64,
    total_error: AtomicU64, // Stored as fixed point (x1000)
    clipping_events: AtomicU64,
}

impl GradientProtocol {
    pub fn new(transport: Arc<DefaultTransport>, batch_size: usize) -> Self {
        Self {
            version: ProtocolVersion::new(1, 0, 0),
            transport,
            negotiated: None,
            accumulator: parking_lot::Mutex::new(GradientAccumulator::new(batch_size, true)),
            metrics: Arc::new(GradientMetrics::default()),
        }
    }
    
    /// Send a gradient to a specific neuron
    pub async fn send_gradient(&self, gradient: GradientMessage) -> Result<()> {
        // Validate gradient
        if !gradient.gradient.is_significant() {
            return Ok(()); // Too small to matter
        }
        
        // Encode and send
        let encoded = self.encode_internal(&gradient)?;
        let destination = format!("neuron:{}:gradient", gradient.target_neuron);
        
        self.transport.send(&destination, encoded).await?;
        
        // Update metrics
        self.metrics.gradients_sent.fetch_add(1, Ordering::Relaxed);
        self.metrics.total_error.fetch_add(
            (gradient.gradient.error.abs() * 1000.0) as u64,
            Ordering::Relaxed
        );
        
        Ok(())
    }
    
    /// Accumulate gradient for batch processing
    pub async fn accumulate_gradient(&self, neuron_id: Uuid, gradient: Gradient) -> Result<()> {
        let mut accumulator = self.accumulator.lock();
        
        if let Some(flushed) = accumulator.add(neuron_id, gradient) {
            // Auto-flushed, send it
            drop(accumulator); // Release lock before async operation
            
            let message = GradientMessage {
                id: Uuid::new_v4(),
                source_neuron: Uuid::nil(), // Accumulated gradient has no single source
                target_neuron: neuron_id,
                timestamp: chrono::Utc::now(),
                gradient: flushed,
                learning_context: LearningContext {
                    learning_rate: 0.01,
                    momentum: 0.9,
                    batch_size: self.accumulator.lock().batch_size as u32,
                    epoch: 0,
                    loss_type: LossType::MeanSquaredError,
                },
            };
            
            self.send_gradient(message).await?;
        }
        
        self.metrics.gradients_accumulated.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    
    /// Flush all accumulated gradients
    pub async fn flush_gradients(&self) -> Result<()> {
        let flushed = self.accumulator.lock().flush_all();
        
        for (neuron_id, gradient) in flushed {
            let message = GradientMessage {
                id: Uuid::new_v4(),
                source_neuron: Uuid::nil(),
                target_neuron: neuron_id,
                timestamp: chrono::Utc::now(),
                gradient,
                learning_context: LearningContext {
                    learning_rate: 0.01,
                    momentum: 0.9,
                    batch_size: self.accumulator.lock().batch_size as u32,
                    epoch: 0,
                    loss_type: LossType::MeanSquaredError,
                },
            };
            
            self.send_gradient(message).await?;
        }
        
        Ok(())
    }
    
    /// Receive gradients for a neuron
    pub async fn receive_gradients(&self, neuron_id: Uuid) -> Result<GradientReceiver> {
        let endpoint = format!("neuron:{}:gradient", neuron_id);
        let receiver = self.transport.receive::<Vec<u8>>(&endpoint).await?;
        
        Ok(GradientReceiver {
            receiver,
            protocol: self,
        })
    }
    
    /// Apply gradient clipping
    pub fn clip_gradient(&self, mut gradient: Gradient, max_norm: f32) -> Gradient {
        gradient.clip(max_norm);
        
        if gradient.magnitude >= max_norm {
            self.metrics.clipping_events.fetch_add(1, Ordering::Relaxed);
        }
        
        gradient
    }
    
    fn encode_internal(&self, gradient: &GradientMessage) -> Result<Vec<u8>> {
        let data = bincode::serialize(gradient)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        
        // Apply compression if negotiated (gradients benefit from compression)
        let compressed = if let Some(neg) = &self.negotiated {
            match neg.compression {
                CompressionType::Zstd => {
                    zstd::encode_all(data.as_slice(), 3)
                        .map_err(|e| Error::Protocol(format!("Compression failed: {}", e)))?
                }
                CompressionType::Gzip => {
                    use flate2::write::GzEncoder;
                    use flate2::Compression;
                    use std::io::Write;
                    
                    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                    encoder.write_all(&data)
                        .map_err(|e| Error::Protocol(format!("Compression failed: {}", e)))?;
                    encoder.finish()
                        .map_err(|e| Error::Protocol(format!("Compression failed: {}", e)))?
                }
                _ => data,
            }
        } else {
            data
        };
        
        Ok(compressed)
    }
    
    fn decode_internal(&self, data: &[u8]) -> Result<GradientMessage> {
        // Decompress if needed
        let decompressed = if let Some(neg) = &self.negotiated {
            match neg.compression {
                CompressionType::Zstd => {
                    zstd::decode_all(data)
                        .map_err(|e| Error::Protocol(format!("Decompression failed: {}", e)))?
                }
                CompressionType::Gzip => {
                    use flate2::read::GzDecoder;
                    use std::io::Read;
                    
                    let mut decoder = GzDecoder::new(data);
                    let mut decompressed = Vec::new();
                    decoder.read_to_end(&mut decompressed)
                        .map_err(|e| Error::Protocol(format!("Decompression failed: {}", e)))?;
                    decompressed
                }
                _ => data.to_vec(),
            }
        } else {
            data.to_vec()
        };
        
        let gradient = bincode::deserialize(&decompressed)
            .map_err(|e| Error::Deserialization(e.to_string()))?;
        
        self.metrics.gradients_received.fetch_add(1, Ordering::Relaxed);
        
        Ok(gradient)
    }
    
    /// Get protocol metrics
    pub fn metrics(&self) -> GradientProtocolMetrics {
        let sent = self.metrics.gradients_sent.load(Ordering::Relaxed);
        let received = self.metrics.gradients_received.load(Ordering::Relaxed);
        let accumulated = self.metrics.gradients_accumulated.load(Ordering::Relaxed);
        let total_error = self.metrics.total_error.load(Ordering::Relaxed) as f32 / 1000.0;
        let clipping_events = self.metrics.clipping_events.load(Ordering::Relaxed);
        
        GradientProtocolMetrics {
            gradients_sent: sent,
            gradients_received: received,
            gradients_accumulated: accumulated,
            average_error: if sent > 0 {
                total_error / sent as f32
            } else {
                0.0
            },
            clipping_events,
            batch_efficiency: if accumulated > 0 {
                sent as f32 / accumulated as f32
            } else {
                0.0
            },
        }
    }
}

/// Receiver for gradient messages
pub struct GradientReceiver<'a> {
    receiver: crate::hierarchical::substrate::TransportReceiver<Vec<u8>>,
    protocol: &'a GradientProtocol,
}

impl GradientReceiver<'_> {
    pub async fn recv(&mut self) -> Option<GradientMessage> {
        let data = self.receiver.recv().await?;
        match self.protocol.decode_internal(&data) {
            Ok(gradient) => Some(gradient),
            Err(e) => {
                tracing::error!("Failed to decode gradient: {}", e);
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GradientProtocolMetrics {
    pub gradients_sent: u64,
    pub gradients_received: u64,
    pub gradients_accumulated: u64,
    pub average_error: f32,
    pub clipping_events: u64,
    pub batch_efficiency: f32,
}

#[async_trait]
impl Protocol for GradientProtocol {
    fn id(&self) -> &str {
        "gradient-protocol"
    }
    
    fn version(&self) -> ProtocolVersion {
        self.version.clone()
    }
    
    async fn negotiate(&self, peer_capabilities: &ProtocolCapabilities) -> Result<NegotiatedProtocol> {
        // Prefer Zstd for gradients (better compression ratio)
        let compression = if peer_capabilities.compression.contains(&CompressionType::Zstd) {
            CompressionType::Zstd
        } else if peer_capabilities.compression.contains(&CompressionType::Gzip) {
            CompressionType::Gzip
        } else {
            CompressionType::None
        };
        
        let negotiated = NegotiatedProtocol {
            version: self.version.clone(),
            compression,
            encryption: EncryptionType::None, // Could add encryption for federated learning
            max_message_size: peer_capabilities.max_message_size.min(10_000_000), // 10MB max
        };
        
        Ok(negotiated)
    }
    
    async fn encode_raw(&self, _message_type: &str, _data: Vec<u8>) -> Result<Vec<u8>> {
        Err(Error::Protocol("Use send_gradient for gradient protocol".to_string()))
    }
    
    async fn decode_raw(&self, _data: &[u8]) -> Result<(String, Vec<u8>)> {
        Err(Error::Protocol("Use receive_gradients for gradient protocol".to_string()))
    }
    
    fn capabilities(&self) -> ProtocolCapabilities {
        ProtocolCapabilities {
            compression: vec![CompressionType::None, CompressionType::Gzip, CompressionType::Zstd],
            encryption: vec![EncryptionType::None, EncryptionType::Tls],
            max_message_size: 10_000_000, // 10MB for large gradient batches
            streaming: false,
            bidirectional: true,
            ordered_delivery: true, // Gradients should be applied in order
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hierarchical::substrate::ChannelTransport;
    
    #[tokio::test]
    async fn test_gradient_operations() {
        let gradient = Gradient::new(0.5, vec![0.1, -0.2, 0.3]);
        assert!((gradient.magnitude - (0.01 + 0.04 + 0.09_f32).sqrt()).abs() < 0.0001);
        assert!(gradient.is_significant());
        
        // Test accumulation
        let mut grad1 = Gradient::new(0.1, vec![1.0, 0.0, -1.0]);
        let grad2 = Gradient::new(0.2, vec![0.0, 1.0, 1.0]);
        grad1.accumulate(&grad2);
        
        assert_eq!(grad1.direction, vec![1.0, 1.0, 0.0]);
        assert_eq!(grad1.error, 0.3);
        assert_eq!(grad1.accumulated_steps, 1);
        
        // Test clipping
        let mut large_grad = Gradient::new(1.0, vec![10.0, 10.0, 10.0]);
        large_grad.clip(1.0);
        assert!((large_grad.magnitude - 1.0).abs() < 0.01);
    }
    
    #[tokio::test]
    async fn test_gradient_accumulator() {
        let mut accumulator = GradientAccumulator::new(3, true);
        let neuron_id = Uuid::new_v4();
        
        // Add gradients
        let g1 = Gradient::new(0.1, vec![1.0, 0.0]);
        let g2 = Gradient::new(0.2, vec![0.0, 1.0]);
        
        assert!(accumulator.add(neuron_id, g1).is_none());
        assert!(accumulator.add(neuron_id, g2).is_none());
        
        // Third gradient should trigger flush
        let g3 = Gradient::new(0.3, vec![1.0, 1.0]);
        let flushed = accumulator.add(neuron_id, g3).unwrap();
        
        // Check averaged gradient
        assert!((flushed.error - 0.2).abs() < 0.0001); // (0.1 + 0.2 + 0.3) / 3
        assert!((flushed.direction[0] - 2.0 / 3.0).abs() < 0.0001); // (1 + 0 + 1) / 3
        assert!((flushed.direction[1] - 2.0 / 3.0).abs() < 0.0001); // (0 + 1 + 1) / 3
    }
    
    #[tokio::test]
    async fn test_gradient_protocol() {
        let transport = Arc::new(ChannelTransport::new());
        let protocol = GradientProtocol::new(transport.clone(), 5);
        
        // Set up receiver
        let neuron_id = Uuid::new_v4();
        let mut receiver = protocol.receive_gradients(neuron_id).await.unwrap();
        
        // Send gradient
        let gradient_msg = GradientMessage {
            id: Uuid::new_v4(),
            source_neuron: Uuid::new_v4(),
            target_neuron: neuron_id,
            timestamp: chrono::Utc::now(),
            gradient: Gradient::new(0.25, vec![0.1, -0.1, 0.2]),
            learning_context: LearningContext {
                learning_rate: 0.01,
                momentum: 0.9,
                batch_size: 32,
                epoch: 5,
                loss_type: LossType::CrossEntropy,
            },
        };
        
        protocol.send_gradient(gradient_msg.clone()).await.unwrap();
        
        // Receive gradient
        let received = receiver.recv().await.unwrap();
        assert_eq!(received.id, gradient_msg.id);
        assert_eq!(received.gradient.error, 0.25);
        assert_eq!(received.learning_context.epoch, 5);
        
        // Check metrics
        let metrics = protocol.metrics();
        assert_eq!(metrics.gradients_sent, 1);
        assert_eq!(metrics.gradients_received, 1);
    }
}