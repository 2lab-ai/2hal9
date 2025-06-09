//! Protocol negotiation for establishing communication parameters

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::Result;
use super::{ProtocolVersion, ProtocolCapabilities, CompressionType, EncryptionType};
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;

/// Protocol negotiation session
pub struct NegotiationSession {
    pub id: uuid::Uuid,
    pub peer_id: String,
    pub state: NegotiationState,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub agreement: Option<NegotiationAgreement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NegotiationState {
    Initiated,
    Offered,
    Negotiating,
    Agreed,
    Failed { reason: String },
}

/// Session-aware negotiator that tracks ongoing negotiations
pub struct SessionNegotiator {
    inner: Box<dyn ProtocolNegotiator>,
    sessions: Arc<RwLock<HashMap<uuid::Uuid, NegotiationSession>>>,
    timeout: std::time::Duration,
}

impl SessionNegotiator {
    pub fn new(inner: Box<dyn ProtocolNegotiator>, timeout: std::time::Duration) -> Self {
        Self {
            inner,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            timeout,
        }
    }
    
    pub fn get_session(&self, session_id: uuid::Uuid) -> Option<NegotiationSession> {
        self.sessions.read().get(&session_id).cloned()
    }
    
    pub fn active_sessions(&self) -> Vec<NegotiationSession> {
        self.sessions.read().values().cloned().collect()
    }
    
    pub fn cleanup_expired(&self) {
        let now = chrono::Utc::now();
        let timeout = self.timeout;
        
        self.sessions.write().retain(|_, session| {
            if session.completed_at.is_some() {
                return true;
            }
            
            let elapsed = now - session.started_at;
            elapsed.to_std().unwrap_or_default() < timeout
        });
    }
}

#[async_trait]
impl ProtocolNegotiator for SessionNegotiator {
    async fn initiate(&self, offered: &ProtocolOffer) -> Result<NegotiationResponse> {
        self.inner.initiate(offered).await
    }
    
    async fn respond(&self, request: &NegotiationRequest) -> Result<ProtocolOffer> {
        self.inner.respond(request).await
    }
    
    async fn finalize(&self, agreement: &NegotiationAgreement) -> Result<()> {
        // Update session
        if let Some(mut session) = self.sessions.write().get_mut(&agreement.session_id) {
            session.state = NegotiationState::Agreed;
            session.completed_at = Some(chrono::Utc::now());
            session.agreement = Some(agreement.clone());
        }
        
        self.inner.finalize(agreement).await
    }
}

/// Protocol negotiator for establishing communication parameters
#[async_trait]
pub trait ProtocolNegotiator: Send + Sync {
    /// Initiate negotiation as client
    async fn initiate(&self, offered: &ProtocolOffer) -> Result<NegotiationResponse>;
    
    /// Respond to negotiation as server
    async fn respond(&self, request: &NegotiationRequest) -> Result<ProtocolOffer>;
    
    /// Finalize negotiation
    async fn finalize(&self, agreement: &NegotiationAgreement) -> Result<()>;
}

/// Protocol offer during negotiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolOffer {
    pub protocols: Vec<ProtocolDescriptor>,
    pub capabilities: ProtocolCapabilities,
    pub preferences: NegotiationPreferences,
}

/// Protocol descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolDescriptor {
    pub id: String,
    pub versions: Vec<ProtocolVersion>,
    pub features: Vec<String>,
}

/// Negotiation preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationPreferences {
    pub preferred_compression: Option<CompressionType>,
    pub preferred_encryption: Option<EncryptionType>,
    pub require_encryption: bool,
    pub require_ordered_delivery: bool,
}

/// Negotiation request from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationRequest {
    pub client_id: String,
    pub offered_protocols: Vec<ProtocolDescriptor>,
    pub capabilities: ProtocolCapabilities,
}

/// Negotiation response from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationResponse {
    pub selected_protocol: ProtocolDescriptor,
    pub negotiated_params: NegotiatedParameters,
}

/// Negotiated parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiatedParameters {
    pub version: ProtocolVersion,
    pub compression: CompressionType,
    pub encryption: EncryptionType,
    pub max_message_size: usize,
    pub features: Vec<String>,
}

/// Final negotiation agreement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationAgreement {
    pub session_id: uuid::Uuid,
    pub protocol: ProtocolDescriptor,
    pub parameters: NegotiatedParameters,
    pub valid_until: chrono::DateTime<chrono::Utc>,
}

/// Default negotiator implementation
pub struct DefaultNegotiator {
    supported_protocols: Vec<ProtocolDescriptor>,
    capabilities: ProtocolCapabilities,
}

#[async_trait]
impl ProtocolNegotiator for DefaultNegotiator {
    async fn initiate(&self, offered: &ProtocolOffer) -> Result<NegotiationResponse> {
        // Select best matching protocol
        let selected = self.select_best_protocol(&offered.protocols)
            .ok_or_else(|| crate::Error::Protocol("No compatible protocol found".to_string()))?;
        
        // Negotiate parameters
        let params = self.negotiate_parameters(
            &selected,
            &offered.capabilities,
            &offered.preferences,
        );
        
        Ok(NegotiationResponse {
            selected_protocol: selected,
            negotiated_params: params,
        })
    }
    
    async fn respond(&self, request: &NegotiationRequest) -> Result<ProtocolOffer> {
        // Filter our protocols to those the client supports
        let compatible: Vec<ProtocolDescriptor> = self.supported_protocols.iter()
            .filter(|our| request.offered_protocols.iter().any(|their| their.id == our.id))
            .cloned()
            .collect();
        
        if compatible.is_empty() {
            return Err(crate::Error::Protocol("No compatible protocols".to_string()));
        }
        
        Ok(ProtocolOffer {
            protocols: compatible,
            capabilities: self.capabilities.clone(),
            preferences: NegotiationPreferences {
                preferred_compression: Some(CompressionType::Zstd),
                preferred_encryption: Some(EncryptionType::Tls),
                require_encryption: false,
                require_ordered_delivery: true,
            },
        })
    }
    
    async fn finalize(&self, agreement: &NegotiationAgreement) -> Result<()> {
        // Validate agreement
        if !self.supported_protocols.iter().any(|p| p.id == agreement.protocol.id) {
            return Err(crate::Error::Protocol("Protocol not supported".to_string()));
        }
        
        // Could store agreement for later reference
        tracing::info!(
            "Finalized protocol negotiation: {} v{}.{}.{}",
            agreement.protocol.id,
            agreement.parameters.version.major,
            agreement.parameters.version.minor,
            agreement.parameters.version.patch
        );
        
        Ok(())
    }
}

impl DefaultNegotiator {
    pub fn new(supported: Vec<ProtocolDescriptor>, capabilities: ProtocolCapabilities) -> Self {
        Self {
            supported_protocols: supported,
            capabilities,
        }
    }
    
    fn select_best_protocol(&self, offered: &[ProtocolDescriptor]) -> Option<ProtocolDescriptor> {
        for our_proto in &self.supported_protocols {
            for their_proto in offered {
                if our_proto.id == their_proto.id {
                    // Find compatible version
                    for our_ver in &our_proto.versions {
                        for their_ver in &their_proto.versions {
                            if our_ver.is_compatible_with(their_ver) {
                                return Some(ProtocolDescriptor {
                                    id: our_proto.id.clone(),
                                    versions: vec![our_ver.clone()],
                                    features: our_proto.features.iter()
                                        .filter(|f| their_proto.features.contains(f))
                                        .cloned()
                                        .collect(),
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }
    
    fn negotiate_parameters(
        &self,
        protocol: &ProtocolDescriptor,
        peer_capabilities: &ProtocolCapabilities,
        preferences: &NegotiationPreferences,
    ) -> NegotiatedParameters {
        // Select compression
        let compression = if preferences.preferred_compression.is_some() {
            let pref = preferences.preferred_compression.as_ref().unwrap();
            if peer_capabilities.compression.contains(pref) && self.capabilities.compression.contains(pref) {
                pref.clone()
            } else {
                CompressionType::None
            }
        } else {
            // Choose best available
            self.select_best_compression(&peer_capabilities.compression)
        };
        
        // Select encryption
        let encryption = if preferences.require_encryption {
            if let Some(enc) = preferences.preferred_encryption.as_ref() {
                if peer_capabilities.encryption.contains(enc) && self.capabilities.encryption.contains(enc) {
                    enc.clone()
                } else {
                    self.select_best_encryption(&peer_capabilities.encryption)
                        .unwrap_or(EncryptionType::None)
                }
            } else {
                self.select_best_encryption(&peer_capabilities.encryption)
                    .unwrap_or(EncryptionType::None)
            }
        } else {
            preferences.preferred_encryption.as_ref()
                .cloned()
                .unwrap_or(EncryptionType::None)
        };
        
        // Calculate max message size
        let max_message_size = self.capabilities.max_message_size
            .min(peer_capabilities.max_message_size);
        
        NegotiatedParameters {
            version: protocol.versions[0].clone(),
            compression,
            encryption,
            max_message_size,
            features: protocol.features.clone(),
        }
    }
    
    fn select_best_compression(&self, available: &[CompressionType]) -> CompressionType {
        // Preference order: Zstd > Lz4 > Gzip > None
        for pref in &[CompressionType::Zstd, CompressionType::Lz4, CompressionType::Gzip] {
            if available.contains(pref) && self.capabilities.compression.contains(pref) {
                return pref.clone();
            }
        }
        CompressionType::None
    }
    
    fn select_best_encryption(&self, available: &[EncryptionType]) -> Option<EncryptionType> {
        // Preference order: Aes256 > Tls > None
        for pref in &[EncryptionType::Aes256, EncryptionType::Tls] {
            if available.contains(pref) && self.capabilities.encryption.contains(pref) {
                return Some(pref.clone());
            }
        }
        None
    }
}