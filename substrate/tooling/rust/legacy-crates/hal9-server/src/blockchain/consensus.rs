use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

use crate::signal::Signal;

// ============ Consensus Protocol ============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusProtocol {
    ProofOfWork {
        difficulty: u32,
        target_time: u64,
    },
    ProofOfStake {
        min_stake: u64,
        validator_count: u32,
    },
    ProofOfComputation {
        min_neurons: u32,
        threshold: f64,
    },
    DelegatedProofOfStake {
        delegates: u32,
        voting_period: u64,
    },
    ByzantineFaultTolerant {
        max_faulty_nodes: u32,
    },
}

impl Default for ConsensusProtocol {
    fn default() -> Self {
        Self::ProofOfComputation {
            min_neurons: 3,
            threshold: 0.67, // 2/3 majority
        }
    }
}

// ============ Consensus Engine ============

pub struct ConsensusEngine {
    protocol: ConsensusProtocol,
    validators: HashMap<String, Validator>,
    pending_proposals: HashMap<Uuid, ConsensusProposal>,
    finalized_blocks: Vec<ConsensusBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Validator {
    id: String,
    address: String,
    stake: u64,
    reputation: f64,
    last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusProposal {
    pub id: Uuid,
    pub proposer: String,
    pub proposal_type: ProposalType,
    pub data: serde_json::Value,
    pub votes: HashMap<String, Vote>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalType {
    SignalValidation(Signal),
    NeuronRegistration {
        neuron_id: Uuid,
        capabilities: Vec<String>,
    },
    ComputationResult {
        task_id: Uuid,
        result_hash: String,
    },
    GovernanceChange {
        parameter: String,
        new_value: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Vote {
    validator: String,
    vote_type: VoteType,
    signature: String,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum VoteType {
    Approve,
    Reject,
    Abstain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusBlock {
    pub block_number: u64,
    pub parent_hash: String,
    pub block_hash: String,
    pub timestamp: DateTime<Utc>,
    pub proposer: String,
    pub proposals: Vec<ConsensusProposal>,
    pub state_root: String,
}

impl ConsensusEngine {
    pub fn new(protocol: ConsensusProtocol) -> Self {
        Self {
            protocol,
            validators: HashMap::new(),
            pending_proposals: HashMap::new(),
            finalized_blocks: Vec::new(),
        }
    }
    
    /// Register a new validator
    pub fn register_validator(
        &mut self,
        id: String,
        address: String,
        stake: u64,
    ) -> Result<()> {
        let validator = Validator {
            id: id.clone(),
            address,
            stake,
            reputation: 1.0,
            last_active: Utc::now(),
        };
        
        self.validators.insert(id, validator);
        Ok(())
    }
    
    /// Submit a proposal for consensus
    pub fn submit_proposal(
        &mut self,
        proposer: String,
        proposal_type: ProposalType,
        data: serde_json::Value,
    ) -> Result<Uuid> {
        // Verify proposer is a validator
        if !self.validators.contains_key(&proposer) {
            return Err(anyhow::anyhow!("Proposer is not a validator"));
        }
        
        let proposal = ConsensusProposal {
            id: Uuid::new_v4(),
            proposer,
            proposal_type,
            data,
            votes: HashMap::new(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(5),
        };
        
        let id = proposal.id;
        self.pending_proposals.insert(id, proposal);
        
        Ok(id)
    }
    
    /// Vote on a proposal
    pub fn vote(
        &mut self,
        proposal_id: Uuid,
        validator_id: String,
        vote_type: VoteType,
        signature: String,
    ) -> Result<()> {
        // Verify validator
        if !self.validators.contains_key(&validator_id) {
            return Err(anyhow::anyhow!("Invalid validator"));
        }
        
        // Get proposal
        let proposal = self.pending_proposals.get_mut(&proposal_id)
            .ok_or_else(|| anyhow::anyhow!("Proposal not found"))?;
        
        // Check if already voted
        if proposal.votes.contains_key(&validator_id) {
            return Err(anyhow::anyhow!("Already voted"));
        }
        
        // Record vote
        proposal.votes.insert(
            validator_id.clone(),
            Vote {
                validator: validator_id,
                vote_type,
                signature,
                timestamp: Utc::now(),
            },
        );
        
        // Check if consensus reached
        self.check_consensus(proposal_id)?;
        
        Ok(())
    }
    
    /// Check if consensus has been reached
    fn check_consensus(&mut self, proposal_id: Uuid) -> Result<bool> {
        let proposal = self.pending_proposals.get(&proposal_id)
            .ok_or_else(|| anyhow::anyhow!("Proposal not found"))?;
        
        match &self.protocol {
            ConsensusProtocol::ProofOfComputation { min_neurons, threshold } => {
                let total_votes = proposal.votes.len();
                if total_votes < *min_neurons as usize {
                    return Ok(false);
                }
                
                let approve_count = proposal.votes.values()
                    .filter(|v| matches!(v.vote_type, VoteType::Approve))
                    .count();
                
                let approval_ratio = approve_count as f64 / total_votes as f64;
                
                if approval_ratio >= *threshold {
                    self.finalize_proposal(proposal_id)?;
                    return Ok(true);
                }
            }
            ConsensusProtocol::ProofOfStake { min_stake, .. } => {
                let total_stake: u64 = proposal.votes.keys()
                    .filter_map(|v| self.validators.get(v))
                    .map(|v| v.stake)
                    .sum();
                
                if total_stake < *min_stake {
                    return Ok(false);
                }
                
                let approve_stake: u64 = proposal.votes.iter()
                    .filter(|(_, vote)| matches!(vote.vote_type, VoteType::Approve))
                    .filter_map(|(id, _)| self.validators.get(id))
                    .map(|v| v.stake)
                    .sum();
                
                if approve_stake > total_stake / 2 {
                    self.finalize_proposal(proposal_id)?;
                    return Ok(true);
                }
            }
            _ => {
                // Simple majority for other protocols
                let approve_count = proposal.votes.values()
                    .filter(|v| matches!(v.vote_type, VoteType::Approve))
                    .count();
                
                if approve_count > self.validators.len() / 2 {
                    self.finalize_proposal(proposal_id)?;
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    /// Finalize a proposal that reached consensus
    fn finalize_proposal(&mut self, proposal_id: Uuid) -> Result<()> {
        let proposal = self.pending_proposals.remove(&proposal_id)
            .ok_or_else(|| anyhow::anyhow!("Proposal not found"))?;
        
        // Create new block
        let block_number = self.finalized_blocks.len() as u64;
        let parent_hash = self.finalized_blocks.last()
            .map(|b| b.block_hash.clone())
            .unwrap_or_else(|| "0x0".to_string());
        
        let block = ConsensusBlock {
            block_number,
            parent_hash: parent_hash.clone(),
            block_hash: self.calculate_block_hash(block_number, &parent_hash, &proposal),
            timestamp: Utc::now(),
            proposer: proposal.proposer.clone(),
            proposals: vec![proposal],
            state_root: self.calculate_state_root(),
        };
        
        self.finalized_blocks.push(block);
        
        // Update validator reputations
        self.update_reputations();
        
        Ok(())
    }
    
    /// Calculate block hash
    fn calculate_block_hash(
        &self,
        block_number: u64,
        parent_hash: &str,
        proposal: &ConsensusProposal,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(block_number.to_be_bytes());
        hasher.update(parent_hash.as_bytes());
        hasher.update(serde_json::to_string(proposal).unwrap_or_default().as_bytes());
        
        format!("{:x}", hasher.finalize())
    }
    
    /// Calculate state root
    fn calculate_state_root(&self) -> String {
        let mut hasher = Sha256::new();
        
        // Hash validators
        for (id, validator) in &self.validators {
            hasher.update(id.as_bytes());
            hasher.update(validator.stake.to_be_bytes());
        }
        
        // Hash finalized blocks
        for block in &self.finalized_blocks {
            hasher.update(block.block_hash.as_bytes());
        }
        
        format!("{:x}", hasher.finalize())
    }
    
    /// Update validator reputations based on participation
    fn update_reputations(&mut self) {
        let current_time = Utc::now();
        
        for validator in self.validators.values_mut() {
            let time_since_active = current_time - validator.last_active;
            
            if time_since_active > chrono::Duration::hours(24) {
                // Decrease reputation for inactivity
                validator.reputation *= 0.95;
            } else {
                // Increase reputation for activity
                validator.reputation = (validator.reputation * 1.01).min(2.0);
            }
        }
    }
    
    /// Get consensus state
    pub fn get_state(&self) -> ConsensusState {
        ConsensusState {
            protocol: self.protocol.clone(),
            validator_count: self.validators.len(),
            pending_proposals: self.pending_proposals.len(),
            finalized_blocks: self.finalized_blocks.len(),
            latest_block: self.finalized_blocks.last().cloned(),
        }
    }
}

// ============ Consensus State ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    pub protocol: ConsensusProtocol,
    pub validator_count: usize,
    pub pending_proposals: usize,
    pub finalized_blocks: usize,
    pub latest_block: Option<ConsensusBlock>,
}

// ============ Distributed Neuron Consensus ============

pub struct NeuronConsensus {
    engine: ConsensusEngine,
    neuron_votes: HashMap<Uuid, NeuronVote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NeuronVote {
    signal_id: Uuid,
    neuron_id: Uuid,
    layer: String,
    result_hash: String,
    confidence: f64,
    timestamp: DateTime<Utc>,
}

impl NeuronConsensus {
    pub fn new() -> Self {
        Self {
            engine: ConsensusEngine::new(ConsensusProtocol::default()),
            neuron_votes: HashMap::new(),
        }
    }
    
    /// Submit neuron computation result for consensus
    pub async fn submit_result(
        &mut self,
        signal_id: Uuid,
        neuron_id: Uuid,
        layer: String,
        result: &serde_json::Value,
        confidence: f64,
    ) -> Result<()> {
        // Hash the result
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(result)?.as_bytes());
        let result_hash = format!("{:x}", hasher.finalize());
        
        // Record vote
        let vote = NeuronVote {
            signal_id,
            neuron_id,
            layer,
            result_hash,
            confidence,
            timestamp: Utc::now(),
        };
        
        self.neuron_votes.insert(neuron_id, vote);
        
        // Check if enough votes for consensus
        let votes_for_signal: Vec<_> = self.neuron_votes.values()
            .filter(|v| v.signal_id == signal_id && v.layer == layer)
            .collect();
        
        if votes_for_signal.len() >= 3 {
            // Find majority result
            let mut result_counts: HashMap<String, u32> = HashMap::new();
            for vote in &votes_for_signal {
                *result_counts.entry(vote.result_hash.clone()).or_insert(0) += 1;
            }
            
            if let Some((consensus_hash, count)) = result_counts.iter()
                .max_by_key(|(_, &count)| count) {
                if *count > votes_for_signal.len() as u32 / 2 {
                    // Consensus reached
                    tracing::info!(
                        "Consensus reached for signal {} with hash {}",
                        signal_id,
                        consensus_hash
                    );
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_consensus_engine() {
        let mut engine = ConsensusEngine::new(ConsensusProtocol::default());
        
        // Register validators
        engine.register_validator("validator1".to_string(), "0x1".to_string(), 1000).unwrap();
        engine.register_validator("validator2".to_string(), "0x2".to_string(), 2000).unwrap();
        engine.register_validator("validator3".to_string(), "0x3".to_string(), 1500).unwrap();
        
        // Submit proposal
        let proposal_id = engine.submit_proposal(
            "validator1".to_string(),
            ProposalType::GovernanceChange {
                parameter: "min_stake".to_string(),
                new_value: serde_json::json!(500),
            },
            serde_json::json!({}),
        ).unwrap();
        
        // Vote on proposal
        engine.vote(proposal_id, "validator1".to_string(), VoteType::Approve, "sig1".to_string()).unwrap();
        engine.vote(proposal_id, "validator2".to_string(), VoteType::Approve, "sig2".to_string()).unwrap();
        engine.vote(proposal_id, "validator3".to_string(), VoteType::Approve, "sig3".to_string()).unwrap();
        
        // Check state
        let state = engine.get_state();
        assert_eq!(state.finalized_blocks, 1);
    }
}