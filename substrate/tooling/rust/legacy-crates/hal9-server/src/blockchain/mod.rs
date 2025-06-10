pub mod chain;
pub mod consensus;
pub mod contracts;
pub mod incentives;
pub mod proof;
pub mod storage;
pub mod service;

pub use chain::{BlockchainClient, ChainConfig, Network};
pub use consensus::{ConsensusEngine, ConsensusProtocol};
pub use contracts::{NeuronContract, IncentiveContract};
pub use incentives::{TokenEconomics, RewardDistribution};
pub use proof::{ComputationProof, ProofVerifier};
pub use storage::{IPFSStorage, DecentralizedStorage};
pub use service::{BlockchainService, BlockchainConfig, BlockchainStats};

#[cfg(test)]
mod tests;