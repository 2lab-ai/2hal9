//! Protocol Manager - Central coordination for all protocol operations
//!
//! This module provides a unified interface for managing protocols, negotiation,
//! versioning, and message routing across the hierarchical architecture.

use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use uuid::Uuid;
use crate::{Result, Error};
use super::{
    Protocol, ProtocolVersion, ProtocolCapabilities, NegotiatedProtocol,
    negotiation::{
        ProtocolNegotiator, SessionNegotiator, DefaultNegotiator,
        ProtocolOffer, NegotiationRequest, NegotiationResponse, NegotiationAgreement,
        ProtocolDescriptor, NegotiationPreferences,
    },
    CompressionType, EncryptionType,
    versioning::{VersionRegistry, VersionedMessage},
    signal::SignalProtocol,
    gradient::GradientProtocol,
    consensus::ConsensusProtocol,
};
use crate::hierarchical::substrate::MessageTransport;

/// Protocol manager configuration
pub struct ProtocolManagerConfig {
    pub negotiation_timeout: std::time::Duration,
    pub enable_compression: bool,
    pub enable_encryption: bool,
    pub max_message_size: usize,
    pub version_migration_enabled: bool,
}

impl Default for ProtocolManagerConfig {
    fn default() -> Self {
        Self {
            negotiation_timeout: std::time::Duration::from_secs(30),
            enable_compression: true,
            enable_encryption: false,
            max_message_size: 10_000_000, // 10MB
            version_migration_enabled: true,
        }
    }
}

/// Centralized protocol manager
pub struct ProtocolManager {
    config: ProtocolManagerConfig,
    transport: Arc<dyn MessageTransport>,
    protocols: Arc<RwLock<HashMap<String, Arc<dyn Protocol>>>>,
    negotiator: Arc<SessionNegotiator>,
    version_registry: Arc<RwLock<VersionRegistry>>,
    active_connections: Arc<RwLock<HashMap<String, ConnectionState>>>,
}

/// Connection state tracking
#[derive(Clone)]
struct ConnectionState {
    peer_id: String,
    negotiated_protocols: HashMap<String, NegotiatedProtocol>,
    established_at: chrono::DateTime<chrono::Utc>,
    last_activity: chrono::DateTime<chrono::Utc>,
}

impl ProtocolManager {
    pub fn new(config: ProtocolManagerConfig, transport: Arc<dyn MessageTransport>) -> Self {
        // Create supported protocol descriptors
        let supported_protocols = vec![
            ProtocolDescriptor {
                id: "signal-protocol".to_string(),
                versions: vec![ProtocolVersion::new(1, 0, 0)],
                features: vec!["broadcast".to_string(), "decay".to_string()],
            },
            ProtocolDescriptor {
                id: "gradient-protocol".to_string(),
                versions: vec![ProtocolVersion::new(1, 0, 0)],
                features: vec!["accumulation".to_string(), "clipping".to_string()],
            },
            ProtocolDescriptor {
                id: "consensus-protocol".to_string(),
                versions: vec![ProtocolVersion::new(1, 0, 0)],
                features: vec!["voting".to_string(), "byzantine".to_string()],
            },
        ];
        
        // Create protocol capabilities
        let capabilities = ProtocolCapabilities {
            compression: if config.enable_compression {
                vec![CompressionType::None, CompressionType::Gzip, CompressionType::Zstd, CompressionType::Lz4]
            } else {
                vec![CompressionType::None]
            },
            encryption: if config.enable_encryption {
                vec![EncryptionType::None, EncryptionType::Tls, EncryptionType::Aes256]
            } else {
                vec![EncryptionType::None]
            },
            max_message_size: config.max_message_size,
            streaming: true,
            bidirectional: true,
            ordered_delivery: true,
        };
        
        // Create negotiator
        let default_negotiator = DefaultNegotiator::new(supported_protocols, capabilities);
        let negotiator = SessionNegotiator::new(
            Box::new(default_negotiator),
            config.negotiation_timeout,
        );
        
        // Create version registry
        let version_registry = VersionRegistry::new(ProtocolVersion::new(1, 0, 0));
        
        Self {
            config,
            transport,
            protocols: Arc::new(RwLock::new(HashMap::new())),
            negotiator: Arc::new(negotiator),
            version_registry: Arc::new(RwLock::new(version_registry)),
            active_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Initialize all standard protocols
    pub async fn initialize_protocols(&self) -> Result<()> {
        // Register signal protocol
        let signal_protocol = Arc::new(SignalProtocol::new(self.transport.clone()));
        self.register_protocol("signal-protocol", signal_protocol)?;
        
        // Register gradient protocol
        let gradient_protocol = Arc::new(GradientProtocol::new(self.transport.clone(), 32));
        self.register_protocol("gradient-protocol", gradient_protocol)?;
        
        // Register consensus protocol
        let consensus_protocol = Arc::new(ConsensusProtocol::new(
            self.transport.clone(),
            Uuid::new_v4(),
            super::consensus::ConsensusAlgorithm::SimpleMajority,
        ));
        self.register_protocol("consensus-protocol", consensus_protocol)?;
        
        Ok(())
    }
    
    /// Register a protocol
    pub fn register_protocol(&self, id: &str, protocol: Arc<dyn Protocol>) -> Result<()> {
        self.protocols.write().insert(id.to_string(), protocol);
        Ok(())
    }
    
    /// Get a protocol by ID
    pub fn get_protocol(&self, id: &str) -> Option<Arc<dyn Protocol>> {
        self.protocols.read().get(id).cloned()
    }
    
    /// Negotiate protocol with a peer
    pub async fn negotiate_with_peer(&self, peer_id: &str) -> Result<NegotiationAgreement> {
        // Create our offer
        let offer = ProtocolOffer {
            protocols: self.get_supported_protocols(),
            capabilities: self.get_capabilities(),
            preferences: NegotiationPreferences {
                preferred_compression: if self.config.enable_compression {
                    Some(CompressionType::Zstd)
                } else {
                    None
                },
                preferred_encryption: if self.config.enable_encryption {
                    Some(EncryptionType::Tls)
                } else {
                    None
                },
                require_encryption: false,
                require_ordered_delivery: true,
            },
        };
        
        // Send negotiation request
        let request_msg = serde_json::to_vec(&offer)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        
        self.transport.send(
            &format!("peer:{}:negotiation", peer_id),
            request_msg,
        ).await?;
        
        // Wait for response (simplified - in real implementation would use proper async handling)
        let response: NegotiationResponse = self.receive_negotiation_response(peer_id).await?;
        
        // Create agreement
        let agreement = NegotiationAgreement {
            session_id: Uuid::new_v4(),
            protocol: response.selected_protocol,
            parameters: response.negotiated_params,
            valid_until: chrono::Utc::now() + chrono::Duration::hours(24),
        };
        
        // Finalize negotiation
        self.negotiator.finalize(&agreement).await?;
        
        // Store connection state
        let connection = ConnectionState {
            peer_id: peer_id.to_string(),
            negotiated_protocols: [(agreement.protocol.id.clone(), NegotiatedProtocol {
                version: agreement.parameters.version.clone(),
                compression: agreement.parameters.compression.clone(),
                encryption: agreement.parameters.encryption.clone(),
                max_message_size: agreement.parameters.max_message_size,
            })].into_iter().collect(),
            established_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
        };
        
        self.active_connections.write().insert(peer_id.to_string(), connection);
        
        Ok(agreement)
    }
    
    /// Send a versioned message
    pub async fn send_versioned_message(
        &self,
        peer_id: &str,
        protocol_id: &str,
        message: &[u8],
    ) -> Result<()> {
        let protocol = self.get_protocol(protocol_id)
            .ok_or_else(|| Error::Protocol(format!("Unknown protocol: {}", protocol_id)))?;
        
        // Create versioned message
        let versioned = VersionedMessage::new(
            protocol_id,
            protocol.version(),
            message.to_vec(),
        );
        
        // Serialize
        let data = serde_json::to_vec(&versioned)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        
        // Send
        self.transport.send(
            &format!("peer:{}:protocol:{}", peer_id, protocol_id),
            data,
        ).await?;
        
        // Update last activity
        if let Some(mut conn) = self.active_connections.write().get_mut(peer_id) {
            conn.last_activity = chrono::Utc::now();
        }
        
        Ok(())
    }
    
    /// Receive and process a versioned message
    pub async fn receive_versioned_message(&self, data: &[u8]) -> Result<(String, Vec<u8>)> {
        let versioned: VersionedMessage = serde_json::from_slice(data)
            .map_err(|e| Error::Deserialization(e.to_string()))?;
        
        // Check if we need to migrate
        let protocol = self.get_protocol(&versioned.protocol_id)
            .ok_or_else(|| Error::Protocol(format!("Unknown protocol: {}", versioned.protocol_id)))?;
        
        let message = if versioned.version != protocol.version() && self.config.version_migration_enabled {
            // Migrate to current version
            self.version_registry.read()
                .migrate_to_current(&versioned.version, &versioned.payload)?
        } else if versioned.version.is_compatible_with(&protocol.version()) {
            // Compatible version, use as-is
            versioned.payload
        } else {
            return Err(Error::Protocol(format!(
                "Incompatible protocol version: {:?}",
                versioned.version
            )));
        };
        
        Ok((versioned.protocol_id, message))
    }
    
    /// Get list of supported protocols
    fn get_supported_protocols(&self) -> Vec<ProtocolDescriptor> {
        self.protocols.read().iter().map(|(id, protocol)| {
            ProtocolDescriptor {
                id: id.clone(),
                versions: vec![protocol.version()],
                features: vec![], // Could be extended to query protocol features
            }
        }).collect()
    }
    
    /// Get combined capabilities
    fn get_capabilities(&self) -> ProtocolCapabilities {
        ProtocolCapabilities {
            compression: if self.config.enable_compression {
                vec![CompressionType::None, CompressionType::Gzip, CompressionType::Zstd, CompressionType::Lz4]
            } else {
                vec![CompressionType::None]
            },
            encryption: if self.config.enable_encryption {
                vec![EncryptionType::None, EncryptionType::Tls, EncryptionType::Aes256]
            } else {
                vec![EncryptionType::None]
            },
            max_message_size: self.config.max_message_size,
            streaming: true,
            bidirectional: true,
            ordered_delivery: true,
        }
    }
    
    /// Receive negotiation response (simplified)
    async fn receive_negotiation_response(&self, peer_id: &str) -> Result<NegotiationResponse> {
        let mut receiver = self.transport.receive::<Vec<u8>>(
            &format!("peer:{}:negotiation:response", peer_id)
        ).await?;
        
        let data = receiver.recv().await
            .ok_or_else(|| Error::Protocol("No negotiation response received".to_string()))?;
        
        serde_json::from_slice(&data)
            .map_err(|e| Error::Deserialization(e.to_string()))
    }
    
    /// Clean up expired connections
    pub fn cleanup_expired_connections(&self, timeout: std::time::Duration) {
        let now = chrono::Utc::now();
        
        self.active_connections.write().retain(|_, conn| {
            let elapsed = now - conn.last_activity;
            elapsed.to_std().unwrap_or_default() < timeout
        });
        
        // Also cleanup negotiation sessions
        self.negotiator.cleanup_expired();
    }
    
    /// Get active connection count
    pub fn active_connection_count(&self) -> usize {
        self.active_connections.read().len()
    }
    
    /// Get protocol metrics
    pub fn get_metrics(&self) -> ProtocolManagerMetrics {
        let protocols = self.protocols.read();
        let connections = self.active_connections.read();
        
        ProtocolManagerMetrics {
            registered_protocols: protocols.len(),
            active_connections: connections.len(),
            negotiation_sessions: self.negotiator.active_sessions().len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProtocolManagerMetrics {
    pub registered_protocols: usize,
    pub active_connections: usize,
    pub negotiation_sessions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hierarchical::substrate::ChannelTransport;
    
    #[tokio::test]
    async fn test_protocol_manager() {
        let transport = Arc::new(ChannelTransport::new());
        let config = ProtocolManagerConfig::default();
        let manager = ProtocolManager::new(config, transport);
        
        // Initialize protocols
        manager.initialize_protocols().await.unwrap();
        
        // Check registered protocols
        assert!(manager.get_protocol("signal-protocol").is_some());
        assert!(manager.get_protocol("gradient-protocol").is_some());
        assert!(manager.get_protocol("consensus-protocol").is_some());
        
        // Check metrics
        let metrics = manager.get_metrics();
        assert_eq!(metrics.registered_protocols, 3);
        assert_eq!(metrics.active_connections, 0);
    }
}