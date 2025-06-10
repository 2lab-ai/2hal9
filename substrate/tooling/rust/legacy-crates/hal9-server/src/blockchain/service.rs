use anyhow::{Context, Result};
use ethers::core::types::{Address, H256, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{
    chain::{BlockchainClient, ChainConfig},
    consensus::{ConsensusEngine, ConsensusProtocol, ProposalType},
    contracts::{ComputationMarketContract, IncentiveContract, NeuronContract},
    incentives::{RewardDistribution, StakingMechanism, TokenEconomics},
    proof::{ComputationProof, ProofBuilder, ProofVerifier},
    storage::{DecentralizedStorage, IPFSConfig},
};

use crate::signal::Signal;

// ============ Blockchain Service ============

pub struct BlockchainService {
    config: BlockchainConfig,
    client: Arc<BlockchainClient>,
    consensus: Arc<RwLock<ConsensusEngine>>,
    neuron_contract: Arc<NeuronContract>,
    incentive_contract: Arc<IncentiveContract>,
    market_contract: Arc<ComputationMarketContract>,
    reward_distribution: Arc<RwLock<RewardDistribution>>,
    staking: Arc<RwLock<StakingMechanism>>,
    proof_verifier: Arc<ProofVerifier>,
    storage: Arc<DecentralizedStorage>,
    active_tasks: Arc<RwLock<HashMap<Uuid, ComputationTaskState>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub chain: ChainConfig,
    pub consensus: ConsensusProtocol,
    pub economics: TokenEconomics,
    pub storage: StorageConfig,
    pub enable_mining: bool,
    pub validator_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub ipfs: Option<IPFSConfig>,
    pub redundancy_level: u32,
    pub max_storage_per_neuron: u64,
}

#[derive(Debug, Clone)]
struct ComputationTaskState {
    task_id: Uuid,
    signal: Signal,
    assigned_neurons: Vec<Uuid>,
    results: HashMap<Uuid, ComputationResult>,
    consensus_reached: bool,
    reward_distributed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComputationResult {
    neuron_id: Uuid,
    result_hash: H256,
    proof: ComputationProof,
    storage_proof: Option<H256>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl BlockchainService {
    pub async fn new(config: BlockchainConfig) -> Result<Self> {
        // Create blockchain client
        let client = Arc::new(
            BlockchainClient::new(config.chain.clone())
                .await
                .context("Failed to create blockchain client")?,
        );

        // Verify network connection
        if !client.verify_network().await? {
            return Err(anyhow::anyhow!("Network mismatch"));
        }

        // Create contracts
        let neuron_contract = Arc::new(NeuronContract::new(
            config.chain.contract_addresses.neuron_registry.parse()?,
            client.clone(),
        )?);

        let incentive_contract = Arc::new(IncentiveContract::new(
            config.chain.contract_addresses.incentive_token.parse()?,
            client.clone(),
        )?);

        let market_contract = Arc::new(ComputationMarketContract::new(
            config.chain.contract_addresses.computation_market.parse()?,
            client.clone(),
        ));

        // Create consensus engine
        let consensus = Arc::new(RwLock::new(ConsensusEngine::new(config.consensus.clone())));

        // Create reward distribution
        let reward_distribution = Arc::new(RwLock::new(RewardDistribution::new(
            config.economics.clone(),
        )));

        // Create staking mechanism
        let staking = Arc::new(RwLock::new(StakingMechanism::new()));

        // Create proof verifier
        let proof_verifier = Arc::new(ProofVerifier::new());

        // Create storage
        let mut storage = DecentralizedStorage::new(config.storage.redundancy_level);
        if let Some(ipfs_config) = config.storage.ipfs {
            storage = storage.with_ipfs(ipfs_config)?;
        }
        let storage = Arc::new(storage);

        Ok(Self {
            config,
            client,
            consensus,
            neuron_contract,
            incentive_contract,
            market_contract,
            reward_distribution,
            staking,
            proof_verifier,
            storage,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Register a neuron on the blockchain
    pub async fn register_neuron(
        &self,
        neuron_id: Uuid,
        owner: Address,
        layer: String,
        capabilities: Vec<String>,
    ) -> Result<String> {
        // Register on-chain
        let tx_hash = self
            .neuron_contract
            .register_neuron(neuron_id, owner, layer.clone(), capabilities.clone())
            .await?;

        // Submit to consensus if validator
        if let Some(ref validator) = self.config.validator_address {
            let mut consensus = self.consensus.write().await;
            consensus.submit_proposal(
                validator.clone(),
                ProposalType::NeuronRegistration {
                    neuron_id,
                    capabilities,
                },
                serde_json::json!({
                    "owner": owner.to_string(),
                    "layer": layer,
                }),
            )?;
        }

        Ok(tx_hash)
    }

    /// Submit a computation task
    pub async fn submit_task(
        &self,
        signal: Signal,
        max_price: U256,
        deadline_seconds: u64,
    ) -> Result<Uuid> {
        // Store signal data
        let signal_data = serde_json::to_vec(&signal)?;
        let storage_proof = self
            .storage
            .store_with_redundancy(&signal_data, "signal")
            .await?;

        // Create task on market
        let task_id = self
            .market_contract
            .create_task(
                signal.signal_type.clone(),
                storage_proof.content_hash,
                max_price,
                deadline_seconds,
            )
            .await?;

        // Track task state
        let task_state = ComputationTaskState {
            task_id,
            signal: signal.clone(),
            assigned_neurons: Vec::new(),
            results: HashMap::new(),
            consensus_reached: false,
            reward_distributed: false,
        };

        self.active_tasks.write().await.insert(task_id, task_state);

        Ok(task_id)
    }

    /// Process computation result from a neuron
    pub async fn submit_computation_result(
        &self,
        task_id: Uuid,
        neuron_id: Uuid,
        result: serde_json::Value,
        proof_builder: ProofBuilder,
    ) -> Result<()> {
        // Build computation proof
        let input_data = serde_json::to_vec(&result)?;
        let output_data = serde_json::to_vec(&result)?;

        let proof = proof_builder.build_deterministic(
            42,             // seed
            H256::random(), // algorithm hash
            &input_data,
            &output_data,
        );

        // Verify proof
        let verification = self.proof_verifier.verify_proof(&proof).await?;
        if !verification.is_valid {
            return Err(anyhow::anyhow!("Invalid computation proof"));
        }

        // Store result
        let result_data = serde_json::to_vec(&result)?;
        let storage_proof = self
            .storage
            .store_with_redundancy(&result_data, "result")
            .await?;

        // Submit proof on-chain
        let tx_hash = self
            .neuron_contract
            .submit_proof(
                neuron_id,
                verification.proof_hash,
                proof.resource_usage.execution_time_ms,
            )
            .await?;

        // Update task state
        let mut tasks = self.active_tasks.write().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.results.insert(
                neuron_id,
                ComputationResult {
                    neuron_id,
                    result_hash: storage_proof.content_hash,
                    proof,
                    storage_proof: Some(storage_proof.content_hash),
                    timestamp: chrono::Utc::now(),
                },
            );

            // Check for consensus
            if task.results.len() >= 3 && !task.consensus_reached {
                self.check_task_consensus(task_id).await?;
            }
        }

        Ok(())
    }

    /// Check if consensus reached for a task
    async fn check_task_consensus(&self, task_id: Uuid) -> Result<bool> {
        let mut tasks = self.active_tasks.write().await;
        let task = tasks
            .get_mut(&task_id)
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        // Group results by hash
        let mut result_groups: HashMap<H256, Vec<Uuid>> = HashMap::new();
        for (neuron_id, result) in &task.results {
            result_groups
                .entry(result.result_hash)
                .or_insert_with(Vec::new)
                .push(*neuron_id);
        }

        // Find majority
        let total_results = task.results.len();
        for (hash, neurons) in result_groups {
            if neurons.len() > total_results / 2 {
                task.consensus_reached = true;

                // Distribute rewards
                self.distribute_task_rewards(task_id, &neurons).await?;

                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Distribute rewards for completed task
    async fn distribute_task_rewards(&self, task_id: Uuid, winning_neurons: &[Uuid]) -> Result<()> {
        let tasks = self.active_tasks.read().await;
        let task = tasks
            .get(&task_id)
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        // Calculate rewards for each neuron
        let mut reward_dist = self.reward_distribution.write().await;

        for neuron_id in winning_neurons {
            if let Some(result) = task.results.get(neuron_id) {
                let reward = reward_dist.calculate_computation_reward(
                    0.8,  // complexity score
                    0.95, // accuracy score
                    result.proof.resource_usage.execution_time_ms,
                );

                // Update neuron performance
                if let Some(owner) = self.client.get_address() {
                    reward_dist.update_neuron_performance(
                        *neuron_id, owner, 1.0, // performance score
                    );
                }
            }
        }

        // Trigger epoch distribution if needed
        // This would typically be done periodically
        let epoch = chrono::Utc::now().timestamp() as u64 / 86400; // Daily epochs
        reward_dist.distribute_epoch_rewards(epoch).await?;

        Ok(())
    }

    /// Stake tokens for a neuron
    pub async fn stake_tokens(&self, amount: U256, lock_days: u64) -> Result<String> {
        // Check balance
        let address = self
            .client
            .get_address()
            .ok_or_else(|| anyhow::anyhow!("No wallet configured"))?;

        let balance = self.incentive_contract.balance_of(address).await?;
        if balance < amount {
            return Err(anyhow::anyhow!("Insufficient balance"));
        }

        // Stake on-chain
        let tx_hash = self.incentive_contract.stake(amount).await?;

        // Update local staking state
        let mut staking = self.staking.write().await;
        staking.stake(address, amount, lock_days).await?;

        Ok(tx_hash)
    }

    /// Get blockchain statistics
    pub async fn get_stats(&self) -> Result<BlockchainStats> {
        let block_number = self.client.get_block_number().await?;
        let gas_price = self.client.get_gas_price().await?;

        let consensus_state = self.consensus.read().await.get_state();
        let storage_stats = self.storage.get_stats().await?;

        Ok(BlockchainStats {
            block_number,
            gas_price: format!("{} gwei", gas_price / U256::exp10(9)),
            consensus_protocol: consensus_state.protocol,
            validator_count: consensus_state.validator_count,
            pending_proposals: consensus_state.pending_proposals,
            finalized_blocks: consensus_state.finalized_blocks,
            storage_objects: storage_stats.num_objects,
            storage_size: storage_stats.repo_size,
            active_tasks: self.active_tasks.read().await.len(),
        })
    }

    /// Initialize validator node
    pub async fn initialize_validator(&mut self, validator_id: String, stake: u64) -> Result<()> {
        let address = self
            .client
            .get_address()
            .ok_or_else(|| anyhow::anyhow!("No wallet configured"))?;

        let mut consensus = self.consensus.write().await;
        consensus.register_validator(validator_id, address.to_string(), stake)?;

        Ok(())
    }
}

// ============ Blockchain Statistics ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStats {
    pub block_number: u64,
    pub gas_price: String,
    pub consensus_protocol: ConsensusProtocol,
    pub validator_count: usize,
    pub pending_proposals: usize,
    pub finalized_blocks: usize,
    pub storage_objects: u64,
    pub storage_size: u64,
    pub active_tasks: usize,
}

// ============ Blockchain Events Handler ============

pub struct BlockchainEventHandler {
    service: Arc<BlockchainService>,
    event_listeners: Arc<RwLock<HashMap<String, EventListener>>>,
}

type EventListener = Box<dyn Fn(BlockchainEvent) + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainEvent {
    pub event_type: String,
    pub data: serde_json::Value,
    pub block_number: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl BlockchainEventHandler {
    pub fn new(service: Arc<BlockchainService>) -> Self {
        Self {
            service,
            event_listeners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start listening for blockchain events
    pub async fn start(&self) -> Result<()> {
        // This would typically subscribe to contract events
        // For now, we'll just log that it's started
        tracing::info!("Blockchain event handler started");
        Ok(())
    }

    /// Register event listener
    pub async fn on_event<F>(&self, event_type: String, listener: F)
    where
        F: Fn(BlockchainEvent) + Send + Sync + 'static,
    {
        self.event_listeners
            .write()
            .await
            .insert(event_type, Box::new(listener));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blockchain_service_creation() {
        let config = BlockchainConfig {
            chain: ChainConfig::default(),
            consensus: ConsensusProtocol::default(),
            economics: TokenEconomics::default(),
            storage: StorageConfig {
                ipfs: Some(IPFSConfig::default()),
                redundancy_level: 3,
                max_storage_per_neuron: 1024 * 1024 * 1024, // 1GB
            },
            enable_mining: false,
            validator_address: None,
        };

        // This will fail to connect in test environment
        let service = BlockchainService::new(config).await;
        assert!(service.is_err()); // Expected to fail without actual blockchain
    }
}
