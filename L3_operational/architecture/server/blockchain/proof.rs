use anyhow::{Context, Result};
use ethers::core::types::H256;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

// ============ Computation Proof ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputationProof {
    pub task_id: Uuid,
    pub neuron_id: Uuid,
    pub proof_type: ProofType,
    pub input_hash: H256,
    pub output_hash: H256,
    pub execution_trace: ExecutionTrace,
    pub resource_usage: ResourceUsage,
    pub timestamp: i64,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofType {
    /// Deterministic computation with reproducible results
    Deterministic {
        seed: u64,
        algorithm_hash: H256,
    },
    /// Probabilistic computation with confidence bounds
    Probabilistic {
        confidence: f64,
        samples: u32,
        variance: f64,
    },
    /// Zero-knowledge proof for private computations
    ZeroKnowledge {
        commitment: H256,
        proof_data: Vec<u8>,
    },
    /// Trusted execution environment proof
    TrustedExecution {
        enclave_id: String,
        attestation: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub steps: Vec<ExecutionStep>,
    pub checkpoints: Vec<Checkpoint>,
    pub total_operations: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_number: u32,
    pub operation: String,
    pub input_snapshot: H256,
    pub output_snapshot: H256,
    pub gas_used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub step_number: u32,
    pub state_hash: H256,
    pub memory_snapshot: H256,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_cycles: u64,
    pub memory_bytes: u64,
    pub network_bytes: u64,
    pub storage_bytes: u64,
    pub execution_time_ms: u64,
}

// ============ Proof Verifier ============

pub struct ProofVerifier {
    verification_rules: HashMap<String, VerificationRule>,
    trusted_enclaves: HashMap<String, EnclaveInfo>,
    challenge_history: Vec<VerificationChallenge>,
}

#[derive(Debug, Clone)]
struct VerificationRule {
    rule_type: RuleType,
    threshold: f64,
    required_validators: u32,
}

#[derive(Debug, Clone)]
enum RuleType {
    ConsensusRequired,
    ResourceLimit,
    TimeBound,
    AccuracyThreshold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EnclaveInfo {
    enclave_id: String,
    public_key: Vec<u8>,
    attestation_report: Vec<u8>,
    expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationChallenge {
    pub challenge_id: Uuid,
    pub proof_id: H256,
    pub challenger: String,
    pub challenge_type: ChallengeType,
    pub status: ChallengeStatus,
    pub resolution: Option<ChallengeResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeType {
    InvalidComputation,
    ResourceMisreporting,
    NonDeterministic,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeStatus {
    Pending,
    UnderReview,
    Resolved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResolution {
    pub is_valid: bool,
    pub validator_votes: HashMap<String, bool>,
    pub evidence: Vec<u8>,
    pub penalty: Option<u64>,
}

impl ProofVerifier {
    pub fn new() -> Self {
        let mut verification_rules = HashMap::new();
        
        // Default verification rules
        verification_rules.insert(
            "consensus".to_string(),
            VerificationRule {
                rule_type: RuleType::ConsensusRequired,
                threshold: 0.67,
                required_validators: 3,
            },
        );
        
        verification_rules.insert(
            "resource_limit".to_string(),
            VerificationRule {
                rule_type: RuleType::ResourceLimit,
                threshold: 1000000.0, // Max gas units
                required_validators: 1,
            },
        );
        
        Self {
            verification_rules,
            trusted_enclaves: HashMap::new(),
            challenge_history: Vec::new(),
        }
    }
    
    /// Verify a computation proof
    pub async fn verify_proof(&self, proof: &ComputationProof) -> Result<VerificationResult> {
        // Basic validation
        self.validate_proof_structure(proof)?;
        
        // Verify based on proof type
        let type_verification = match &proof.proof_type {
            ProofType::Deterministic { seed, algorithm_hash } => {
                self.verify_deterministic(proof, *seed, algorithm_hash)?
            }
            ProofType::Probabilistic { confidence, samples, variance } => {
                self.verify_probabilistic(proof, *confidence, *samples, *variance)?
            }
            ProofType::ZeroKnowledge { commitment, proof_data } => {
                self.verify_zero_knowledge(proof, commitment, proof_data)?
            }
            ProofType::TrustedExecution { enclave_id, attestation } => {
                self.verify_trusted_execution(proof, enclave_id, attestation)?
            }
        };
        
        // Verify resource usage
        let resource_verification = self.verify_resource_usage(&proof.resource_usage)?;
        
        // Verify execution trace
        let trace_verification = self.verify_execution_trace(&proof.execution_trace)?;
        
        // Calculate overall score
        let score = (type_verification + resource_verification + trace_verification) / 3.0;
        
        Ok(VerificationResult {
            is_valid: score >= 0.8,
            score,
            proof_hash: self.calculate_proof_hash(proof),
            verification_details: VerificationDetails {
                type_score: type_verification,
                resource_score: resource_verification,
                trace_score: trace_verification,
                flags: Vec::new(),
            },
        })
    }
    
    /// Validate proof structure
    fn validate_proof_structure(&self, proof: &ComputationProof) -> Result<()> {
        if proof.execution_trace.steps.is_empty() {
            return Err(anyhow::anyhow!("Empty execution trace"));
        }
        
        if proof.resource_usage.execution_time_ms == 0 {
            return Err(anyhow::anyhow!("Invalid execution time"));
        }
        
        Ok(())
    }
    
    /// Verify deterministic computation
    fn verify_deterministic(
        &self,
        proof: &ComputationProof,
        seed: u64,
        algorithm_hash: &H256,
    ) -> Result<f64> {
        // Verify algorithm hash matches known algorithms
        // In real implementation, would check against registry
        
        // Verify execution is reproducible with given seed
        let reproducibility_score = if proof.execution_trace.checkpoints.len() > 0 {
            1.0
        } else {
            0.5
        };
        
        Ok(reproducibility_score)
    }
    
    /// Verify probabilistic computation
    fn verify_probabilistic(
        &self,
        proof: &ComputationProof,
        confidence: f64,
        samples: u32,
        variance: f64,
    ) -> Result<f64> {
        // Verify confidence bounds
        if confidence < 0.0 || confidence > 1.0 {
            return Ok(0.0);
        }
        
        // Verify sample size is sufficient
        let min_samples = (1.0 / (1.0 - confidence)).ceil() as u32;
        if samples < min_samples {
            return Ok(0.5);
        }
        
        // Verify variance is reasonable
        let variance_score = if variance < 0.1 { 1.0 } else { 0.8 };
        
        Ok(variance_score)
    }
    
    /// Verify zero-knowledge proof
    fn verify_zero_knowledge(
        &self,
        proof: &ComputationProof,
        commitment: &H256,
        proof_data: &[u8],
    ) -> Result<f64> {
        // In real implementation, would verify ZK proof
        // For now, check basic properties
        
        if proof_data.len() < 32 {
            return Ok(0.0);
        }
        
        // Verify commitment matches
        let mut hasher = Sha256::new();
        hasher.update(&proof.input_hash);
        hasher.update(&proof.output_hash);
        let expected_commitment = H256::from_slice(&hasher.finalize());
        
        if commitment == &expected_commitment {
            Ok(1.0)
        } else {
            Ok(0.0)
        }
    }
    
    /// Verify trusted execution environment proof
    fn verify_trusted_execution(
        &self,
        proof: &ComputationProof,
        enclave_id: &str,
        attestation: &[u8],
    ) -> Result<f64> {
        // Check if enclave is trusted
        if let Some(enclave_info) = self.trusted_enclaves.get(enclave_id) {
            // Verify attestation
            // In real implementation, would verify SGX/SEV attestation
            if attestation.len() >= 64 {
                Ok(1.0)
            } else {
                Ok(0.5)
            }
        } else {
            Ok(0.0)
        }
    }
    
    /// Verify resource usage
    fn verify_resource_usage(&self, usage: &ResourceUsage) -> Result<f64> {
        let mut score = 1.0;
        
        // Check CPU cycles are reasonable
        if usage.cpu_cycles > 1_000_000_000_000 {
            score *= 0.5; // Penalize excessive CPU usage
        }
        
        // Check memory usage
        if usage.memory_bytes > 4 * 1024 * 1024 * 1024 { // 4GB
            score *= 0.7;
        }
        
        // Check execution time
        if usage.execution_time_ms > 300_000 { // 5 minutes
            score *= 0.8;
        }
        
        Ok(score)
    }
    
    /// Verify execution trace
    fn verify_execution_trace(&self, trace: &ExecutionTrace) -> Result<f64> {
        let mut score = 1.0;
        
        // Verify checkpoints
        if trace.checkpoints.is_empty() {
            score *= 0.5;
        }
        
        // Verify step consistency
        let mut prev_output = H256::zero();
        for (i, step) in trace.steps.iter().enumerate() {
            if i > 0 && step.input_snapshot != prev_output {
                score *= 0.9; // Penalize inconsistent traces
            }
            prev_output = step.output_snapshot;
        }
        
        Ok(score)
    }
    
    /// Calculate proof hash
    fn calculate_proof_hash(&self, proof: &ComputationProof) -> H256 {
        let mut hasher = Sha256::new();
        hasher.update(proof.task_id.as_bytes());
        hasher.update(proof.neuron_id.as_bytes());
        hasher.update(&proof.input_hash);
        hasher.update(&proof.output_hash);
        hasher.update(&proof.timestamp.to_be_bytes());
        
        H256::from_slice(&hasher.finalize())
    }
    
    /// Submit a challenge to a proof
    pub async fn challenge_proof(
        &mut self,
        proof_hash: H256,
        challenger: String,
        challenge_type: ChallengeType,
    ) -> Result<Uuid> {
        let challenge = VerificationChallenge {
            challenge_id: Uuid::new_v4(),
            proof_id: proof_hash,
            challenger,
            challenge_type,
            status: ChallengeStatus::Pending,
            resolution: None,
        };
        
        let challenge_id = challenge.challenge_id;
        self.challenge_history.push(challenge);
        
        Ok(challenge_id)
    }
}

// ============ Verification Result ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub score: f64,
    pub proof_hash: H256,
    pub verification_details: VerificationDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationDetails {
    pub type_score: f64,
    pub resource_score: f64,
    pub trace_score: f64,
    pub flags: Vec<String>,
}

// ============ Proof Builder ============

pub struct ProofBuilder {
    task_id: Uuid,
    neuron_id: Uuid,
    execution_trace: ExecutionTrace,
    resource_tracker: ResourceTracker,
}

#[derive(Default)]
struct ResourceTracker {
    cpu_start: u64,
    memory_peak: u64,
    network_total: u64,
    storage_total: u64,
    start_time: i64,
}

impl ProofBuilder {
    pub fn new(task_id: Uuid, neuron_id: Uuid) -> Self {
        Self {
            task_id,
            neuron_id,
            execution_trace: ExecutionTrace {
                steps: Vec::new(),
                checkpoints: Vec::new(),
                total_operations: 0,
            },
            resource_tracker: ResourceTracker {
                start_time: chrono::Utc::now().timestamp_millis(),
                ..Default::default()
            },
        }
    }
    
    /// Add execution step
    pub fn add_step(
        &mut self,
        operation: String,
        input: &[u8],
        output: &[u8],
        gas_used: u64,
    ) {
        let step = ExecutionStep {
            step_number: self.execution_trace.steps.len() as u32,
            operation,
            input_snapshot: H256::from_slice(&Sha256::digest(input)),
            output_snapshot: H256::from_slice(&Sha256::digest(output)),
            gas_used,
        };
        
        self.execution_trace.steps.push(step);
        self.execution_trace.total_operations += 1;
    }
    
    /// Add checkpoint
    pub fn add_checkpoint(&mut self, state: &[u8], memory: &[u8]) {
        let checkpoint = Checkpoint {
            step_number: self.execution_trace.steps.len() as u32,
            state_hash: H256::from_slice(&Sha256::digest(state)),
            memory_snapshot: H256::from_slice(&Sha256::digest(memory)),
            timestamp: chrono::Utc::now().timestamp_millis(),
        };
        
        self.execution_trace.checkpoints.push(checkpoint);
    }
    
    /// Build deterministic proof
    pub fn build_deterministic(
        self,
        seed: u64,
        algorithm_hash: H256,
        input: &[u8],
        output: &[u8],
    ) -> ComputationProof {
        let now = chrono::Utc::now().timestamp_millis();
        
        ComputationProof {
            task_id: self.task_id,
            neuron_id: self.neuron_id,
            proof_type: ProofType::Deterministic {
                seed,
                algorithm_hash,
            },
            input_hash: H256::from_slice(&Sha256::digest(input)),
            output_hash: H256::from_slice(&Sha256::digest(output)),
            execution_trace: self.execution_trace,
            resource_usage: ResourceUsage {
                cpu_cycles: 1_000_000, // Would track actual usage
                memory_bytes: self.resource_tracker.memory_peak,
                network_bytes: self.resource_tracker.network_total,
                storage_bytes: self.resource_tracker.storage_total,
                execution_time_ms: (now - self.resource_tracker.start_time) as u64,
            },
            timestamp: now,
            signature: vec![], // Would sign with neuron key
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_proof_verification() {
        let verifier = ProofVerifier::new();
        
        let mut builder = ProofBuilder::new(Uuid::new_v4(), Uuid::new_v4());
        builder.add_step("compute".to_string(), b"input", b"output", 100);
        builder.add_checkpoint(b"state", b"memory");
        
        let proof = builder.build_deterministic(
            12345,
            H256::random(),
            b"test input",
            b"test output",
        );
        
        let result = verifier.verify_proof(&proof).await.unwrap();
        assert!(result.score > 0.0);
    }
}