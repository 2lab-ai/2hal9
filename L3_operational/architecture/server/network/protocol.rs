//! Network protocol and message definitions

use serde::{Serialize, Deserialize};
use hal9_core::{NeuronSignal, Result, Error};

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NetworkMessage {
    /// Hello message for handshake
    Hello {
        version: String,
        server_id: String,
        capabilities: Vec<String>,
    },
    
    /// Signal forwarding
    Signal(NeuronSignal),
    
    /// Heartbeat/keepalive
    Ping,
    
    /// Heartbeat response
    Pong,
    
    /// Metrics sharing
    Metrics {
        server_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        metrics: MetricsSnapshot,
    },
    
    /// Error notification
    Error {
        code: String,
        message: String,
    },
    
    /// Server discovery announcement
    Discovery {
        server_id: String,
        address: String,
        neurons: Vec<NeuronInfo>,
    },
    
    /// Request for server info
    InfoRequest,
    
    /// Server info response
    InfoResponse {
        server_id: String,
        version: String,
        neurons: Vec<NeuronInfo>,
        connections: Vec<String>,
    },
}

/// Neuron information for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronInfo {
    pub id: String,
    pub layer: String,
    pub server_id: String,
}

/// Metrics snapshot for sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub signals_sent: u64,
    pub signals_processed: u64,
    pub signals_failed: u64,
    pub neurons_active: u64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f32,
}

/// Message codec for encoding/decoding
pub struct MessageCodec;

impl MessageCodec {
    /// Encode a message with length prefix
    pub fn encode(msg: &NetworkMessage) -> Result<Vec<u8>> {
        // Serialize to JSON
        let json = serde_json::to_vec(msg)
            .map_err(|e| Error::Serialization(format!("Failed to encode message: {}", e)))?;
            
        // Create buffer with 4-byte length prefix
        let mut buffer = Vec::with_capacity(4 + json.len());
        
        // Write length as big-endian u32
        let len = json.len() as u32;
        buffer.extend_from_slice(&len.to_be_bytes());
        
        // Write JSON data
        buffer.extend_from_slice(&json);
        
        Ok(buffer)
    }
    
    /// Decode a message from bytes
    pub fn decode(data: &[u8]) -> Result<NetworkMessage> {
        // Need at least 4 bytes for length
        if data.len() < 4 {
            return Err(Error::Network("Incomplete message header".to_string()));
        }
        
        // Read length
        let len_bytes: [u8; 4] = data[0..4].try_into()
            .map_err(|_| Error::Network("Invalid length bytes".to_string()))?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        
        // Check if we have enough data
        if data.len() < 4 + len {
            return Err(Error::Network("Incomplete message data".to_string()));
        }
        
        // Deserialize JSON
        let msg = serde_json::from_slice(&data[4..4+len])
            .map_err(|e| Error::Serialization(format!("Failed to decode message: {}", e)))?;
            
        Ok(msg)
    }
    
    /// Encode multiple messages into a single buffer
    pub fn encode_batch(messages: &[NetworkMessage]) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        
        for msg in messages {
            let encoded = Self::encode(msg)?;
            buffer.extend_from_slice(&encoded);
        }
        
        Ok(buffer)
    }
    
    /// Decode multiple messages from a buffer
    pub fn decode_batch(mut data: &[u8]) -> Result<Vec<NetworkMessage>> {
        let mut messages = Vec::new();
        
        while !data.is_empty() {
            // Try to decode one message
            match Self::decode(data) {
                Ok(msg) => {
                    // Calculate how much data this message consumed
                    let msg_size = {
                        let len_bytes: [u8; 4] = data[0..4].try_into().unwrap();
                        4 + u32::from_be_bytes(len_bytes) as usize
                    };
                    
                    messages.push(msg);
                    
                    // Advance to next message
                    data = &data[msg_size..];
                }
                Err(e) => {
                    // If we have some messages, return them
                    if !messages.is_empty() {
                        return Ok(messages);
                    }
                    // Otherwise propagate error
                    return Err(e);
                }
            }
        }
        
        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hal9_core::NeuronSignal;
    
    #[test]
    fn test_message_encode_decode() {
        let msg = NetworkMessage::Hello {
            version: "1.0".to_string(),
            server_id: "test-server".to_string(),
            capabilities: vec!["signal".to_string()],
        };
        
        let encoded = MessageCodec::encode(&msg).unwrap();
        let decoded = MessageCodec::decode(&encoded).unwrap();
        
        match decoded {
            NetworkMessage::Hello { server_id, .. } => {
                assert_eq!(server_id, "test-server");
            }
            _ => panic!("Wrong message type"),
        }
    }
    
    #[test]
    fn test_signal_message() {
        let signal = NeuronSignal::forward(
            "neuron-1",
            "neuron-2",
            "L4",
            "L3",
            "test content".to_string(),
        );
        
        let msg = NetworkMessage::Signal(signal.clone());
        let encoded = MessageCodec::encode(&msg).unwrap();
        let decoded = MessageCodec::decode(&encoded).unwrap();
        
        match decoded {
            NetworkMessage::Signal(decoded_signal) => {
                assert_eq!(decoded_signal.signal_id, signal.signal_id);
                assert_eq!(decoded_signal.from_neuron, signal.from_neuron);
                assert_eq!(decoded_signal.to_neuron, signal.to_neuron);
            }
            _ => panic!("Wrong message type"),
        }
    }
    
    #[test]
    fn test_batch_encoding() {
        let messages = vec![
            NetworkMessage::Ping,
            NetworkMessage::Pong,
            NetworkMessage::Error {
                code: "TEST".to_string(),
                message: "Test error".to_string(),
            },
        ];
        
        let encoded = MessageCodec::encode_batch(&messages).unwrap();
        let decoded = MessageCodec::decode_batch(&encoded).unwrap();
        
        assert_eq!(decoded.len(), 3);
        matches!(decoded[0], NetworkMessage::Ping);
        matches!(decoded[1], NetworkMessage::Pong);
        matches!(decoded[2], NetworkMessage::Error { .. });
    }
}