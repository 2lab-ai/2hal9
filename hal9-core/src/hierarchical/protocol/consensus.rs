//! Consensus Protocol - Distributed agreement and coordination
//!
//! This protocol enables multiple neurons to reach consensus on decisions,
//! supporting various consensus algorithms like voting, quorum, and Byzantine fault tolerance.

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use uuid::Uuid;
use tokio::sync::RwLock;
use crate::{Result, Error};
use crate::hierarchical::substrate::transport::{DefaultTransport, TypedTransport};
use super::{Protocol, ProtocolVersion, ProtocolCapabilities, NegotiatedProtocol, CompressionType, EncryptionType};

/// Consensus message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusMessage {
    /// Propose a value for consensus
    Propose {
        proposal_id: Uuid,
        proposer: Uuid,
        value: serde_json::Value,
        timestamp: chrono::DateTime<chrono::Utc>,
        ttl: Duration,
    },
    
    /// Vote on a proposal
    Vote {
        proposal_id: Uuid,
        voter: Uuid,
        vote: Vote,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    /// Request current state
    StateRequest {
        requester: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    /// Share current state
    StateResponse {
        responder: Uuid,
        proposals: Vec<ProposalState>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    /// Announce consensus reached
    ConsensusReached {
        proposal_id: Uuid,
        value: serde_json::Value,
        votes: Vec<(Uuid, Vote)>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Vote types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vote {
    Accept,
    Reject,
    Abstain,
}

/// Proposal state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalState {
    pub proposal_id: Uuid,
    pub proposer: Uuid,
    pub value: serde_json::Value,
    pub votes: HashMap<Uuid, Vote>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub status: ProposalStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Pending,
    Accepted,
    Rejected,
    Expired,
}

/// Consensus algorithm types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConsensusAlgorithm {
    /// Simple majority (>50%)
    SimpleMajority,
    
    /// Super majority (>66%)
    SuperMajority,
    
    /// Unanimous (100%)
    Unanimous,
    
    /// Quorum-based (configurable threshold)
    Quorum { threshold: f32 },
    
    /// Byzantine fault tolerant (>66% with fault tolerance)
    Byzantine,
}

impl ConsensusAlgorithm {
    fn required_votes(&self, total_participants: usize) -> usize {
        match self {
            Self::SimpleMajority => (total_participants / 2) + 1,
            Self::SuperMajority | Self::Byzantine => (total_participants * 2 / 3) + 1,
            Self::Unanimous => total_participants,
            Self::Quorum { threshold } => ((total_participants as f32 * threshold).ceil() as usize).max(1),
        }
    }
}

/// Consensus protocol implementation
pub struct ConsensusProtocol {
    version: ProtocolVersion,
    transport: Arc<DefaultTransport>,
    negotiated: Option<NegotiatedProtocol>,
    node_id: Uuid,
    algorithm: ConsensusAlgorithm,
    participants: Arc<RwLock<HashSet<Uuid>>>,
    proposals: Arc<RwLock<HashMap<Uuid, ProposalState>>>,
    metrics: Arc<ConsensusMetrics>,
}

#[derive(Default)]
struct ConsensusMetrics {
    proposals_created: AtomicU64,
    votes_cast: AtomicU64,
    consensus_reached: AtomicU64,
    consensus_failed: AtomicU64,
    average_consensus_time_ms: AtomicU64,
}

impl ConsensusProtocol {
    pub fn new(
        transport: Arc<DefaultTransport>,
        node_id: Uuid,
        algorithm: ConsensusAlgorithm,
    ) -> Self {
        Self {
            version: ProtocolVersion::new(1, 0, 0),
            transport,
            negotiated: None,
            node_id,
            algorithm,
            participants: Arc::new(RwLock::new(HashSet::new())),
            proposals: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(ConsensusMetrics::default()),
        }
    }
    
    /// Add a participant to the consensus group
    pub async fn add_participant(&self, participant: Uuid) -> Result<()> {
        self.participants.write().await.insert(participant);
        Ok(())
    }
    
    /// Remove a participant from the consensus group
    pub async fn remove_participant(&self, participant: Uuid) -> Result<()> {
        self.participants.write().await.remove(&participant);
        
        // Re-evaluate all pending proposals
        let mut proposals = self.proposals.write().await;
        for proposal in proposals.values_mut() {
            if proposal.status == ProposalStatus::Pending {
                self.evaluate_proposal(proposal, self.participants.read().await.len());
            }
        }
        
        Ok(())
    }
    
    /// Propose a value for consensus
    pub async fn propose(&self, value: serde_json::Value, ttl: Duration) -> Result<Uuid> {
        let proposal_id = Uuid::new_v4();
        let now = chrono::Utc::now();
        
        let proposal = ProposalState {
            proposal_id,
            proposer: self.node_id,
            value: value.clone(),
            votes: HashMap::new(),
            created_at: now,
            expires_at: now + chrono::Duration::from_std(ttl).unwrap(),
            status: ProposalStatus::Pending,
        };
        
        // Store proposal
        self.proposals.write().await.insert(proposal_id, proposal);
        
        // Broadcast proposal
        let message = ConsensusMessage::Propose {
            proposal_id,
            proposer: self.node_id,
            value,
            timestamp: now,
            ttl,
        };
        
        self.broadcast_message(message).await?;
        
        self.metrics.proposals_created.fetch_add(1, Ordering::Relaxed);
        
        Ok(proposal_id)
    }
    
    /// Vote on a proposal
    pub async fn vote(&self, proposal_id: Uuid, vote: Vote) -> Result<()> {
        let mut proposals = self.proposals.write().await;
        
        let proposal = proposals.get_mut(&proposal_id)
            .ok_or_else(|| Error::Protocol("Unknown proposal".to_string()))?;
        
        if proposal.status != ProposalStatus::Pending {
            return Err(Error::Protocol("Proposal is not pending".to_string()));
        }
        
        if chrono::Utc::now() > proposal.expires_at {
            proposal.status = ProposalStatus::Expired;
            return Err(Error::Protocol("Proposal has expired".to_string()));
        }
        
        // Record vote
        proposal.votes.insert(self.node_id, vote);
        
        // Broadcast vote
        let message = ConsensusMessage::Vote {
            proposal_id,
            voter: self.node_id,
            vote,
            timestamp: chrono::Utc::now(),
        };
        
        drop(proposals); // Release lock before async operation
        self.broadcast_message(message).await?;
        
        // Re-acquire lock and check if consensus reached
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(&proposal_id).unwrap();
        let participant_count = self.participants.read().await.len();
        
        self.evaluate_proposal(proposal, participant_count);
        
        if proposal.status == ProposalStatus::Accepted {
            self.announce_consensus(proposal.clone()).await?;
        }
        
        self.metrics.votes_cast.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Check if a proposal has reached consensus
    fn evaluate_proposal(&self, proposal: &mut ProposalState, participant_count: usize) {
        let accept_votes = proposal.votes.values()
            .filter(|v| **v == Vote::Accept)
            .count();
        
        let reject_votes = proposal.votes.values()
            .filter(|v| **v == Vote::Reject)
            .count();
        
        let required = self.algorithm.required_votes(participant_count);
        
        if accept_votes >= required {
            proposal.status = ProposalStatus::Accepted;
            self.metrics.consensus_reached.fetch_add(1, Ordering::Relaxed);
        } else if reject_votes > participant_count - required {
            // Enough rejections to prevent consensus
            proposal.status = ProposalStatus::Rejected;
            self.metrics.consensus_failed.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// Announce that consensus has been reached
    async fn announce_consensus(&self, proposal: ProposalState) -> Result<()> {
        let message = ConsensusMessage::ConsensusReached {
            proposal_id: proposal.proposal_id,
            value: proposal.value,
            votes: proposal.votes.into_iter().collect(),
            timestamp: chrono::Utc::now(),
        };
        
        self.broadcast_message(message).await
    }
    
    /// Handle incoming consensus messages
    pub async fn handle_message(&self, message: ConsensusMessage) -> Result<()> {
        match message {
            ConsensusMessage::Propose { proposal_id, proposer, value, timestamp, ttl } => {
                let proposal = ProposalState {
                    proposal_id,
                    proposer,
                    value,
                    votes: HashMap::new(),
                    created_at: timestamp,
                    expires_at: timestamp + chrono::Duration::from_std(ttl).unwrap(),
                    status: ProposalStatus::Pending,
                };
                
                self.proposals.write().await.insert(proposal_id, proposal);
            }
            
            ConsensusMessage::Vote { proposal_id, voter, vote, .. } => {
                let mut proposals = self.proposals.write().await;
                if let Some(proposal) = proposals.get_mut(&proposal_id) {
                    proposal.votes.insert(voter, vote);
                    let participant_count = self.participants.read().await.len();
                    self.evaluate_proposal(proposal, participant_count);
                }
            }
            
            ConsensusMessage::StateRequest { requester, .. } => {
                let proposals = self.proposals.read().await;
                let state: Vec<_> = proposals.values().cloned().collect();
                
                let response = ConsensusMessage::StateResponse {
                    responder: self.node_id,
                    proposals: state,
                    timestamp: chrono::Utc::now(),
                };
                
                self.send_to_node(requester, response).await?;
            }
            
            ConsensusMessage::StateResponse { proposals, .. } => {
                // Merge state from other node
                let mut our_proposals = self.proposals.write().await;
                for proposal in proposals {
                    our_proposals.entry(proposal.proposal_id)
                        .or_insert(proposal);
                }
            }
            
            ConsensusMessage::ConsensusReached { proposal_id, .. } => {
                let mut proposals = self.proposals.write().await;
                if let Some(proposal) = proposals.get_mut(&proposal_id) {
                    proposal.status = ProposalStatus::Accepted;
                }
            }
        }
        
        Ok(())
    }
    
    /// Start receiving consensus messages
    pub async fn start_receiver(&self) -> Result<()> {
        let endpoint = format!("consensus:{}", self.node_id);
        let mut receiver = self.transport.receive::<Vec<u8>>(&endpoint).await?;
        
        let protocol = self.clone();
        tokio::spawn(async move {
            while let Some(data) = receiver.recv().await {
                if let Ok(message) = protocol.decode_message(&data) {
                    if let Err(e) = protocol.handle_message(message).await {
                        tracing::error!("Failed to handle consensus message: {}", e);
                    }
                }
            }
        });
        
        // Also subscribe to broadcast channel
        let mut broadcast_receiver = self.transport.subscribe::<Vec<u8>>("consensus:broadcast").await?;
        
        let protocol = self.clone();
        tokio::spawn(async move {
            while let Some(data) = broadcast_receiver.recv().await {
                if let Ok(message) = protocol.decode_message(&data) {
                    if let Err(e) = protocol.handle_message(message).await {
                        tracing::error!("Failed to handle broadcast consensus message: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    async fn broadcast_message(&self, message: ConsensusMessage) -> Result<()> {
        let encoded = self.encode_message(&message)?;
        self.transport.publish("consensus:broadcast", encoded).await
    }
    
    async fn send_to_node(&self, node_id: Uuid, message: ConsensusMessage) -> Result<()> {
        let encoded = self.encode_message(&message)?;
        let endpoint = format!("consensus:{}", node_id);
        self.transport.send(&endpoint, encoded).await
    }
    
    fn encode_message(&self, message: &ConsensusMessage) -> Result<Vec<u8>> {
        serde_json::to_vec(message)
            .map_err(|e| Error::Serialization(e.to_string()))
    }
    
    fn decode_message(&self, data: &[u8]) -> Result<ConsensusMessage> {
        serde_json::from_slice(data)
            .map_err(|e| Error::Deserialization(e.to_string()))
    }
    
    /// Get consensus protocol metrics
    pub fn metrics(&self) -> ConsensusProtocolMetrics {
        ConsensusProtocolMetrics {
            proposals_created: self.metrics.proposals_created.load(Ordering::Relaxed),
            votes_cast: self.metrics.votes_cast.load(Ordering::Relaxed),
            consensus_reached: self.metrics.consensus_reached.load(Ordering::Relaxed),
            consensus_failed: self.metrics.consensus_failed.load(Ordering::Relaxed),
            average_consensus_time_ms: self.metrics.average_consensus_time_ms.load(Ordering::Relaxed),
        }
    }
}

// Implement Clone manually to handle Arc fields
impl Clone for ConsensusProtocol {
    fn clone(&self) -> Self {
        Self {
            version: self.version.clone(),
            transport: Arc::clone(&self.transport),
            negotiated: self.negotiated.clone(),
            node_id: self.node_id,
            algorithm: self.algorithm,
            participants: Arc::clone(&self.participants),
            proposals: Arc::clone(&self.proposals),
            metrics: Arc::clone(&self.metrics),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsensusProtocolMetrics {
    pub proposals_created: u64,
    pub votes_cast: u64,
    pub consensus_reached: u64,
    pub consensus_failed: u64,
    pub average_consensus_time_ms: u64,
}

#[async_trait]
impl Protocol for ConsensusProtocol {
    fn id(&self) -> &str {
        "consensus-protocol"
    }
    
    fn version(&self) -> ProtocolVersion {
        self.version.clone()
    }
    
    async fn negotiate(&self, peer_capabilities: &ProtocolCapabilities) -> Result<NegotiatedProtocol> {
        let negotiated = NegotiatedProtocol {
            version: self.version.clone(),
            compression: CompressionType::None, // Consensus messages are typically small
            encryption: EncryptionType::Tls, // Important for Byzantine fault tolerance
            max_message_size: peer_capabilities.max_message_size.min(100_000), // 100KB max
        };
        
        Ok(negotiated)
    }
    
    async fn encode_raw(&self, _message_type: &str, _data: Vec<u8>) -> Result<Vec<u8>> {
        Err(Error::Protocol("Use consensus-specific methods".to_string()))
    }
    
    async fn decode_raw(&self, _data: &[u8]) -> Result<(String, Vec<u8>)> {
        Err(Error::Protocol("Use consensus-specific methods".to_string()))
    }
    
    fn capabilities(&self) -> ProtocolCapabilities {
        ProtocolCapabilities {
            compression: vec![CompressionType::None],
            encryption: vec![EncryptionType::None, EncryptionType::Tls],
            max_message_size: 100_000, // 100KB
            streaming: false,
            bidirectional: true,
            ordered_delivery: true, // Important for consensus
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hierarchical::substrate::ChannelTransport;
    
    #[tokio::test]
    async fn test_consensus_algorithm() {
        assert_eq!(ConsensusAlgorithm::SimpleMajority.required_votes(10), 6);
        assert_eq!(ConsensusAlgorithm::SuperMajority.required_votes(10), 7);
        assert_eq!(ConsensusAlgorithm::Unanimous.required_votes(10), 10);
        assert_eq!(ConsensusAlgorithm::Quorum { threshold: 0.75 }.required_votes(10), 8);
    }
    
    #[tokio::test]
    async fn test_consensus_protocol() {
        let transport = Arc::new(ChannelTransport::new());
        
        // Create three nodes
        let node1 = Uuid::new_v4();
        let node2 = Uuid::new_v4();
        let node3 = Uuid::new_v4();
        
        let protocol1 = ConsensusProtocol::new(
            transport.clone(),
            node1,
            ConsensusAlgorithm::SimpleMajority,
        );
        
        let protocol2 = ConsensusProtocol::new(
            transport.clone(),
            node2,
            ConsensusAlgorithm::SimpleMajority,
        );
        
        let protocol3 = ConsensusProtocol::new(
            transport.clone(),
            node3,
            ConsensusAlgorithm::SimpleMajority,
        );
        
        // Add participants
        for p in &[&protocol1, &protocol2, &protocol3] {
            p.add_participant(node1).await.unwrap();
            p.add_participant(node2).await.unwrap();
            p.add_participant(node3).await.unwrap();
        }
        
        // Start receivers
        protocol1.start_receiver().await.unwrap();
        protocol2.start_receiver().await.unwrap();
        protocol3.start_receiver().await.unwrap();
        
        // Node 1 proposes a value
        let value = serde_json::json!({"action": "test", "value": 42});
        let proposal_id = protocol1.propose(value, Duration::from_secs(60)).await.unwrap();
        
        // Wait for proposal to propagate
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Nodes vote
        protocol1.vote(proposal_id, Vote::Accept).await.unwrap();
        protocol2.vote(proposal_id, Vote::Accept).await.unwrap();
        // Node 3 abstains
        
        // Wait for consensus
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Check that consensus was reached
        let proposals = protocol1.proposals.read().await;
        let proposal = proposals.get(&proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Accepted);
        
        // Check metrics
        let metrics = protocol1.metrics();
        assert_eq!(metrics.proposals_created, 1);
        assert_eq!(metrics.consensus_reached, 1);
    }
}