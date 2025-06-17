use anyhow::Result;
use ethers::{
    abi::{Abi, Token},
    contract::{Contract, ContractFactory},
    core::types::{Address, Bytes, H256, U256},
    providers::Middleware,
    signers::Signer,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::chain::BlockchainClient;

// ============ Contract ABIs ============

const NEURON_REGISTRY_ABI: &str = r#"
[
    {
        "type": "function",
        "name": "registerNeuron",
        "inputs": [
            {"name": "neuronId", "type": "bytes32"},
            {"name": "owner", "type": "address"},
            {"name": "layer", "type": "string"},
            {"name": "capabilities", "type": "string[]"}
        ],
        "outputs": []
    },
    {
        "type": "function",
        "name": "updateNeuron",
        "inputs": [
            {"name": "neuronId", "type": "bytes32"},
            {"name": "metadata", "type": "string"}
        ],
        "outputs": []
    },
    {
        "type": "function",
        "name": "getNeuron",
        "inputs": [{"name": "neuronId", "type": "bytes32"}],
        "outputs": [
            {"name": "owner", "type": "address"},
            {"name": "layer", "type": "string"},
            {"name": "active", "type": "bool"},
            {"name": "reputation", "type": "uint256"}
        ]
    },
    {
        "type": "function",
        "name": "submitProof",
        "inputs": [
            {"name": "neuronId", "type": "bytes32"},
            {"name": "proofHash", "type": "bytes32"},
            {"name": "computationTime", "type": "uint256"}
        ],
        "outputs": []
    },
    {
        "type": "event",
        "name": "NeuronRegistered",
        "inputs": [
            {"name": "neuronId", "type": "bytes32", "indexed": true},
            {"name": "owner", "type": "address", "indexed": true},
            {"name": "layer", "type": "string"}
        ]
    },
    {
        "type": "event",
        "name": "ProofSubmitted",
        "inputs": [
            {"name": "neuronId", "type": "bytes32", "indexed": true},
            {"name": "proofHash", "type": "bytes32"},
            {"name": "reward", "type": "uint256"}
        ]
    }
]
"#;

const INCENTIVE_TOKEN_ABI: &str = r#"
[
    {
        "type": "function",
        "name": "mint",
        "inputs": [
            {"name": "to", "type": "address"},
            {"name": "amount", "type": "uint256"}
        ],
        "outputs": []
    },
    {
        "type": "function",
        "name": "burn",
        "inputs": [{"name": "amount", "type": "uint256"}],
        "outputs": []
    },
    {
        "type": "function",
        "name": "balanceOf",
        "inputs": [{"name": "account", "type": "address"}],
        "outputs": [{"name": "balance", "type": "uint256"}]
    },
    {
        "type": "function",
        "name": "stake",
        "inputs": [{"name": "amount", "type": "uint256"}],
        "outputs": []
    },
    {
        "type": "function",
        "name": "unstake",
        "inputs": [{"name": "amount", "type": "uint256"}],
        "outputs": []
    },
    {
        "type": "function",
        "name": "getStake",
        "inputs": [{"name": "account", "type": "address"}],
        "outputs": [{"name": "staked", "type": "uint256"}]
    }
]
"#;

// ============ Neuron Registry Contract ============

pub struct NeuronContract {
    contract: Contract<Arc<dyn Middleware>>,
    client: Arc<BlockchainClient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronOnChain {
    pub id: Uuid,
    pub owner: Address,
    pub layer: String,
    pub capabilities: Vec<String>,
    pub active: bool,
    pub reputation: U256,
    pub total_computations: u64,
    pub last_proof: Option<H256>,
}

impl NeuronContract {
    pub fn new(address: Address, client: Arc<BlockchainClient>) -> Result<Self> {
        let abi: Abi = serde_json::from_str(NEURON_REGISTRY_ABI)?;
        let contract = Contract::new(address, abi, client.provider.clone() as Arc<dyn Middleware>);

        Ok(Self { contract, client })
    }

    /// Register a new neuron on-chain
    pub async fn register_neuron(
        &self,
        neuron_id: Uuid,
        owner: Address,
        layer: String,
        capabilities: Vec<String>,
    ) -> Result<String> {
        let neuron_id_bytes = H256::from_slice(&neuron_id.as_bytes()[..32]);

        let tx = self
            .contract
            .method::<_, ()>(
                "registerNeuron",
                (neuron_id_bytes, owner, layer, capabilities),
            )?
            .tx;

        let tx_hash = self.client.send_transaction(tx).await?;
        Ok(tx_hash)
    }

    /// Update neuron metadata
    pub async fn update_neuron(&self, neuron_id: Uuid, metadata: String) -> Result<String> {
        let neuron_id_bytes = H256::from_slice(&neuron_id.as_bytes()[..32]);

        let tx = self
            .contract
            .method::<_, ()>("updateNeuron", (neuron_id_bytes, metadata))?
            .tx;

        let tx_hash = self.client.send_transaction(tx).await?;
        Ok(tx_hash)
    }

    /// Get neuron details
    pub async fn get_neuron(&self, neuron_id: Uuid) -> Result<Option<NeuronOnChain>> {
        let neuron_id_bytes = H256::from_slice(&neuron_id.as_bytes()[..32]);

        let result: (Address, String, bool, U256) = self
            .contract
            .method("getNeuron", neuron_id_bytes)?
            .call()
            .await?;

        if result.0 == Address::zero() {
            return Ok(None);
        }

        Ok(Some(NeuronOnChain {
            id: neuron_id,
            owner: result.0,
            layer: result.1,
            capabilities: vec![], // Would need separate query
            active: result.2,
            reputation: result.3,
            total_computations: 0, // Would need separate query
            last_proof: None,
        }))
    }

    /// Submit computation proof
    pub async fn submit_proof(
        &self,
        neuron_id: Uuid,
        proof_hash: H256,
        computation_time: u64,
    ) -> Result<String> {
        let neuron_id_bytes = H256::from_slice(&neuron_id.as_bytes()[..32]);

        let tx = self
            .contract
            .method::<_, ()>(
                "submitProof",
                (neuron_id_bytes, proof_hash, U256::from(computation_time)),
            )?
            .tx;

        let tx_hash = self.client.send_transaction(tx).await?;
        Ok(tx_hash)
    }
}

// ============ Incentive Token Contract ============

pub struct IncentiveContract {
    contract: Contract<Arc<dyn Middleware>>,
    client: Arc<BlockchainClient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStats {
    pub total_supply: U256,
    pub total_staked: U256,
    pub circulating_supply: U256,
    pub reward_rate: U256,
}

impl IncentiveContract {
    pub fn new(address: Address, client: Arc<BlockchainClient>) -> Result<Self> {
        let abi: Abi = serde_json::from_str(INCENTIVE_TOKEN_ABI)?;
        let contract = Contract::new(address, abi, client.provider.clone() as Arc<dyn Middleware>);

        Ok(Self { contract, client })
    }

    /// Get token balance
    pub async fn balance_of(&self, account: Address) -> Result<U256> {
        let balance: U256 = self.contract.method("balanceOf", account)?.call().await?;

        Ok(balance)
    }

    /// Stake tokens
    pub async fn stake(&self, amount: U256) -> Result<String> {
        let tx = self.contract.method::<_, ()>("stake", amount)?.tx;

        let tx_hash = self.client.send_transaction(tx).await?;
        Ok(tx_hash)
    }

    /// Unstake tokens
    pub async fn unstake(&self, amount: U256) -> Result<String> {
        let tx = self.contract.method::<_, ()>("unstake", amount)?.tx;

        let tx_hash = self.client.send_transaction(tx).await?;
        Ok(tx_hash)
    }

    /// Get staked balance
    pub async fn get_stake(&self, account: Address) -> Result<U256> {
        let staked: U256 = self.contract.method("getStake", account)?.call().await?;

        Ok(staked)
    }
}

// ============ Computation Market Contract ============

pub struct ComputationMarketContract {
    address: Address,
    client: Arc<BlockchainClient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputationTask {
    pub id: Uuid,
    pub requester: Address,
    pub task_type: String,
    pub data_hash: H256,
    pub max_price: U256,
    pub deadline: u64,
    pub assigned_neurons: Vec<Uuid>,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Open,
    Assigned,
    Computing,
    Completed,
    Disputed,
    Cancelled,
}

impl ComputationMarketContract {
    pub fn new(address: Address, client: Arc<BlockchainClient>) -> Self {
        Self { address, client }
    }

    /// Create a new computation task
    pub async fn create_task(
        &self,
        task_type: String,
        data_hash: H256,
        max_price: U256,
        deadline: u64,
    ) -> Result<Uuid> {
        // Implementation would interact with smart contract
        let task_id = Uuid::new_v4();

        tracing::info!(
            "Created computation task {} of type {} with max price {}",
            task_id,
            task_type,
            max_price
        );

        Ok(task_id)
    }

    /// Bid on a computation task
    pub async fn bid_on_task(
        &self,
        task_id: Uuid,
        neuron_id: Uuid,
        price: U256,
        estimated_time: u64,
    ) -> Result<String> {
        // Implementation would interact with smart contract
        tracing::info!(
            "Neuron {} bid {} on task {} (est. {} seconds)",
            neuron_id,
            price,
            task_id,
            estimated_time
        );

        Ok("0xmocktxhash".to_string())
    }

    /// Submit computation result
    pub async fn submit_result(
        &self,
        task_id: Uuid,
        neuron_id: Uuid,
        result_hash: H256,
        proof: Vec<u8>,
    ) -> Result<String> {
        // Implementation would interact with smart contract
        tracing::info!(
            "Neuron {} submitted result {} for task {}",
            neuron_id,
            result_hash,
            task_id
        );

        Ok("0xmocktxhash".to_string())
    }
}

// ============ Contract Deployment ============

pub struct ContractDeployer {
    client: Arc<BlockchainClient>,
}

impl ContractDeployer {
    pub fn new(client: Arc<BlockchainClient>) -> Self {
        Self { client }
    }

    /// Deploy neuron registry contract
    pub async fn deploy_neuron_registry(&self) -> Result<Address> {
        // In real implementation, would compile and deploy contract
        // For now, return mock address
        Ok("0x1234567890123456789012345678901234567890".parse()?)
    }

    /// Deploy incentive token contract
    pub async fn deploy_incentive_token(
        &self,
        name: String,
        symbol: String,
        initial_supply: U256,
    ) -> Result<Address> {
        // In real implementation, would compile and deploy contract
        tracing::info!(
            "Deploying incentive token {} ({}) with supply {}",
            name,
            symbol,
            initial_supply
        );

        Ok("0x2345678901234567890123456789012345678901".parse()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neuron_id_to_bytes32() {
        let neuron_id = Uuid::new_v4();
        let bytes = H256::from_slice(&neuron_id.as_bytes()[..32]);
        assert_eq!(bytes.as_bytes().len(), 32);
    }
}
