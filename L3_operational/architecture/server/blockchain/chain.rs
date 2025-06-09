use anyhow::{Context, Result};
use ethers::{
    core::types::{Address, TransactionRequest, U256},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

// ============ Blockchain Configuration ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub network: Network,
    pub rpc_url: String,
    pub chain_id: u64,
    pub private_key: Option<String>,
    pub contract_addresses: ContractAddresses,
    pub gas_config: GasConfig,
    pub confirmation_blocks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Ethereum,
    Polygon,
    Arbitrum,
    Optimism,
    BSC,
    Avalanche,
    Local,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAddresses {
    pub neuron_registry: String,
    pub incentive_token: String,
    pub governance: String,
    pub computation_market: String,
    pub proof_verifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasConfig {
    pub max_gas_price_gwei: u64,
    pub max_priority_fee_gwei: u64,
    pub gas_limit_multiplier: f64,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            network: Network::Local,
            rpc_url: "http://localhost:8545".to_string(),
            chain_id: 31337, // Hardhat/Anvil default
            private_key: None,
            contract_addresses: ContractAddresses {
                neuron_registry: "0x0000000000000000000000000000000000000000".to_string(),
                incentive_token: "0x0000000000000000000000000000000000000000".to_string(),
                governance: "0x0000000000000000000000000000000000000000".to_string(),
                computation_market: "0x0000000000000000000000000000000000000000".to_string(),
                proof_verifier: "0x0000000000000000000000000000000000000000".to_string(),
            },
            gas_config: GasConfig {
                max_gas_price_gwei: 100,
                max_priority_fee_gwei: 2,
                gas_limit_multiplier: 1.2,
            },
            confirmation_blocks: 1,
        }
    }
}

// ============ Blockchain Client ============

pub struct BlockchainClient {
    config: ChainConfig,
    provider: Arc<Provider<Http>>,
    wallet: Option<LocalWallet>,
    nonce_manager: Arc<RwLock<NonceManager>>,
}

struct NonceManager {
    address: Address,
    current_nonce: U256,
    pending_count: u64,
}

impl BlockchainClient {
    pub async fn new(config: ChainConfig) -> Result<Self> {
        // Create provider
        let provider = Provider::<Http>::try_from(&config.rpc_url)
            .context("Failed to create provider")?;
        
        // Create wallet if private key provided
        let wallet = if let Some(ref key) = config.private_key {
            let wallet = key.parse::<LocalWallet>()
                .context("Failed to parse private key")?;
            Some(wallet.with_chain_id(config.chain_id))
        } else {
            None
        };
        
        // Initialize nonce manager if wallet exists
        let nonce_manager = if let Some(ref wallet) = wallet {
            let address = wallet.address();
            let current_nonce = provider.get_transaction_count(address, None).await?;
            Arc::new(RwLock::new(NonceManager {
                address,
                current_nonce,
                pending_count: 0,
            }))
        } else {
            Arc::new(RwLock::new(NonceManager {
                address: Address::zero(),
                current_nonce: U256::zero(),
                pending_count: 0,
            }))
        };
        
        Ok(Self {
            config,
            provider: Arc::new(provider),
            wallet,
            nonce_manager,
        })
    }
    
    /// Get the current block number
    pub async fn get_block_number(&self) -> Result<u64> {
        let block = self.provider.get_block_number().await?;
        Ok(block.as_u64())
    }
    
    /// Get chain ID
    pub async fn get_chain_id(&self) -> Result<u64> {
        let chain_id = self.provider.get_chainid().await?;
        Ok(chain_id.as_u64())
    }
    
    /// Check if connected to the correct network
    pub async fn verify_network(&self) -> Result<bool> {
        let chain_id = self.get_chain_id().await?;
        Ok(chain_id == self.config.chain_id)
    }
    
    /// Get wallet address
    pub fn get_address(&self) -> Option<Address> {
        self.wallet.as_ref().map(|w| w.address())
    }
    
    /// Get balance of an address
    pub async fn get_balance(&self, address: Address) -> Result<U256> {
        let balance = self.provider.get_balance(address, None).await?;
        Ok(balance)
    }
    
    /// Send a transaction
    pub async fn send_transaction(&self, tx: TransactionRequest) -> Result<String> {
        if self.wallet.is_none() {
            return Err(anyhow::anyhow!("No wallet configured"));
        }
        
        // Get and update nonce
        let nonce = {
            let mut nonce_mgr = self.nonce_manager.write().await;
            let nonce = nonce_mgr.current_nonce;
            nonce_mgr.current_nonce = nonce_mgr.current_nonce + 1;
            nonce_mgr.pending_count += 1;
            nonce
        };
        
        // Add nonce to transaction
        let tx = tx.nonce(nonce);
        
        // Create signer middleware
        let client = SignerMiddleware::new(
            self.provider.clone(),
            self.wallet.as_ref().unwrap().clone(),
        );
        
        // Send transaction
        let pending_tx = client.send_transaction(tx, None).await?;
        let tx_hash = pending_tx.tx_hash();
        
        // Wait for confirmation
        let receipt = pending_tx
            .confirmations(self.config.confirmation_blocks as usize)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Transaction failed"))?;
        
        // Update nonce manager
        {
            let mut nonce_mgr = self.nonce_manager.write().await;
            nonce_mgr.pending_count -= 1;
            if nonce_mgr.pending_count == 0 {
                // Refresh nonce from chain
                let current = self.provider
                    .get_transaction_count(nonce_mgr.address, None)
                    .await?;
                nonce_mgr.current_nonce = current;
            }
        }
        
        Ok(format!("0x{:x}", tx_hash))
    }
    
    /// Call a contract function (read-only)
    pub async fn call_contract(
        &self,
        to: Address,
        data: Vec<u8>,
    ) -> Result<Vec<u8>> {
        let tx = TransactionRequest::new()
            .to(to)
            .data(data);
        
        let result = self.provider.call(&tx.into(), None).await?;
        Ok(result.to_vec())
    }
    
    /// Estimate gas for a transaction
    pub async fn estimate_gas(&self, tx: TransactionRequest) -> Result<U256> {
        let gas = self.provider.estimate_gas(&tx, None).await?;
        
        // Apply multiplier for safety
        let adjusted = gas.as_u64() as f64 * self.config.gas_config.gas_limit_multiplier;
        Ok(U256::from(adjusted as u64))
    }
    
    /// Get current gas price
    pub async fn get_gas_price(&self) -> Result<U256> {
        let gas_price = self.provider.get_gas_price().await?;
        
        // Cap at max configured price
        let max_price = U256::from(self.config.gas_config.max_gas_price_gwei) * U256::exp10(9);
        Ok(gas_price.min(max_price))
    }
    
    /// Subscribe to new blocks
    pub async fn subscribe_blocks(&self) -> Result<()> {
        // Note: Requires WebSocket provider for real subscriptions
        // This is a placeholder for HTTP polling
        tracing::warn!("Block subscription requires WebSocket provider");
        Ok(())
    }
}

// ============ Blockchain Events ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainEvent {
    pub event_type: EventType,
    pub block_number: u64,
    pub transaction_hash: String,
    pub timestamp: u64,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    NeuronRegistered,
    NeuronUpdated,
    SignalSubmitted,
    ProofVerified,
    RewardDistributed,
    GovernanceProposal,
    GovernanceVote,
}

// ============ Chain Statistics ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStats {
    pub block_number: u64,
    pub gas_price: String,
    pub total_neurons: u64,
    pub total_signals: u64,
    pub total_rewards: String,
    pub active_validators: u64,
}

// ============ Error Types ============

#[derive(Debug, thiserror::Error)]
pub enum BlockchainError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Gas price too high: {0} gwei")]
    GasPriceTooHigh(u64),
    
    #[error("Network mismatch: expected {expected}, got {actual}")]
    NetworkMismatch { expected: u64, actual: u64 },
    
    #[error("Contract error: {0}")]
    ContractError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_blockchain_client_creation() {
        let config = ChainConfig::default();
        let client = BlockchainClient::new(config).await;
        assert!(client.is_ok());
    }
}