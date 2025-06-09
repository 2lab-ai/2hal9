//! Protocol negotiation for establishing communication parameters

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::Result;
use super::{ProtocolVersion, ProtocolCapabilities, CompressionType, EncryptionType};

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
}