use anyhow::Result;
use chrono::{DateTime, Utc};
use ethers::core::types::{Address, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============ Token Economics ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEconomics {
    pub token_symbol: String,
    pub total_supply: U256,
    pub circulating_supply: U256,
    pub staked_supply: U256,
    pub reward_pool: U256,
    pub emission_schedule: EmissionSchedule,
    pub fee_structure: FeeStructure,
    pub governance_params: GovernanceParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissionSchedule {
    pub initial_rate: U256,
    pub decay_factor: f64,
    pub halving_period_days: u64,
    pub min_emission_rate: U256,
    pub last_halving: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeStructure {
    pub computation_fee_percent: f64,
    pub network_fee_percent: f64,
    pub governance_fee_percent: f64,
    pub burn_rate_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceParams {
    pub proposal_threshold: U256,
    pub quorum_percent: f64,
    pub voting_period_hours: u64,
    pub execution_delay_hours: u64,
}

impl Default for TokenEconomics {
    fn default() -> Self {
        Self {
            token_symbol: "HAL9".to_string(),
            total_supply: U256::from(1_000_000_000) * U256::exp10(18), // 1 billion tokens
            circulating_supply: U256::from(100_000_000) * U256::exp10(18),
            staked_supply: U256::zero(),
            reward_pool: U256::from(500_000_000) * U256::exp10(18),
            emission_schedule: EmissionSchedule {
                initial_rate: U256::from(1000) * U256::exp10(18), // 1000 tokens per day
                decay_factor: 0.95,
                halving_period_days: 365,
                min_emission_rate: U256::from(10) * U256::exp10(18),
                last_halving: Utc::now(),
            },
            fee_structure: FeeStructure {
                computation_fee_percent: 2.0,
                network_fee_percent: 0.5,
                governance_fee_percent: 0.5,
                burn_rate_percent: 1.0,
            },
            governance_params: GovernanceParams {
                proposal_threshold: U256::from(10_000) * U256::exp10(18),
                quorum_percent: 4.0,
                voting_period_hours: 72,
                execution_delay_hours: 48,
            },
        }
    }
}

// ============ Reward Distribution ============

pub struct RewardDistribution {
    economics: TokenEconomics,
    neuron_rewards: HashMap<Uuid, NeuronRewards>,
    validator_rewards: HashMap<Address, ValidatorRewards>,
    epoch_rewards: Vec<EpochReward>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NeuronRewards {
    neuron_id: Uuid,
    owner: Address,
    total_earned: U256,
    pending_rewards: U256,
    last_claim: DateTime<Utc>,
    performance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ValidatorRewards {
    validator: Address,
    total_earned: U256,
    pending_rewards: U256,
    last_claim: DateTime<Utc>,
    blocks_validated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochReward {
    pub epoch: u64,
    pub total_distributed: U256,
    pub neuron_rewards: U256,
    pub validator_rewards: U256,
    pub governance_rewards: U256,
    pub timestamp: DateTime<Utc>,
}

impl RewardDistribution {
    pub fn new(economics: TokenEconomics) -> Self {
        Self {
            economics,
            neuron_rewards: HashMap::new(),
            validator_rewards: HashMap::new(),
            epoch_rewards: Vec::new(),
        }
    }
    
    /// Calculate rewards for a computation
    pub fn calculate_computation_reward(
        &self,
        complexity_score: f64,
        accuracy_score: f64,
        time_taken_ms: u64,
    ) -> U256 {
        // Base reward from daily emission
        let base_reward = self.get_current_emission_rate() / 1000; // Per computation
        
        // Performance multiplier
        let performance_multiplier = complexity_score * accuracy_score;
        
        // Time bonus (faster is better)
        let time_bonus = if time_taken_ms < 1000 {
            1.2 // 20% bonus for sub-second
        } else if time_taken_ms < 5000 {
            1.0 // Normal
        } else {
            0.8 // 20% penalty for slow
        };
        
        // Calculate final reward
        let reward = base_reward.as_u128() as f64 * performance_multiplier * time_bonus;
        U256::from(reward as u128)
    }
    
    /// Distribute rewards for an epoch
    pub async fn distribute_epoch_rewards(&mut self, epoch: u64) -> Result<EpochReward> {
        let total_emission = self.get_epoch_emission();
        
        // Allocate rewards
        let neuron_allocation = total_emission * 70 / 100; // 70% to neurons
        let validator_allocation = total_emission * 20 / 100; // 20% to validators
        let governance_allocation = total_emission * 10 / 100; // 10% to governance
        
        // Distribute to neurons based on performance
        let total_performance: f64 = self.neuron_rewards.values()
            .map(|n| n.performance_score)
            .sum();
        
        for neuron in self.neuron_rewards.values_mut() {
            let share = neuron.performance_score / total_performance;
            let reward = U256::from((neuron_allocation.as_u128() as f64 * share) as u128);
            neuron.pending_rewards = neuron.pending_rewards + reward;
        }
        
        // Distribute to validators based on blocks validated
        let total_blocks: u64 = self.validator_rewards.values()
            .map(|v| v.blocks_validated)
            .sum();
        
        for validator in self.validator_rewards.values_mut() {
            let share = validator.blocks_validated as f64 / total_blocks as f64;
            let reward = U256::from((validator_allocation.as_u128() as f64 * share) as u128);
            validator.pending_rewards = validator.pending_rewards + reward;
        }
        
        // Record epoch
        let epoch_reward = EpochReward {
            epoch,
            total_distributed: total_emission,
            neuron_rewards: neuron_allocation,
            validator_rewards: validator_allocation,
            governance_rewards: governance_allocation,
            timestamp: Utc::now(),
        };
        
        self.epoch_rewards.push(epoch_reward.clone());
        
        Ok(epoch_reward)
    }
    
    /// Claim rewards for a neuron
    pub async fn claim_neuron_rewards(
        &mut self,
        neuron_id: Uuid,
    ) -> Result<U256> {
        let neuron = self.neuron_rewards.get_mut(&neuron_id)
            .ok_or_else(|| anyhow::anyhow!("Neuron not found"))?;
        
        let amount = neuron.pending_rewards;
        if amount == U256::zero() {
            return Ok(U256::zero());
        }
        
        // Update state
        neuron.pending_rewards = U256::zero();
        neuron.total_earned = neuron.total_earned + amount;
        neuron.last_claim = Utc::now();
        
        // TODO: Actually transfer tokens on-chain
        
        Ok(amount)
    }
    
    /// Update neuron performance score
    pub fn update_neuron_performance(
        &mut self,
        neuron_id: Uuid,
        owner: Address,
        new_score: f64,
    ) {
        self.neuron_rewards
            .entry(neuron_id)
            .and_modify(|n| n.performance_score = new_score)
            .or_insert(NeuronRewards {
                neuron_id,
                owner,
                total_earned: U256::zero(),
                pending_rewards: U256::zero(),
                last_claim: Utc::now(),
                performance_score: new_score,
            });
    }
    
    /// Get current emission rate
    fn get_current_emission_rate(&self) -> U256 {
        let schedule = &self.economics.emission_schedule;
        let now = Utc::now();
        let days_since_halving = (now - schedule.last_halving).num_days() as u64;
        
        if days_since_halving >= schedule.halving_period_days {
            // Apply halving
            let halvings = days_since_halving / schedule.halving_period_days;
            let rate = schedule.initial_rate.as_u128() as f64 
                * schedule.decay_factor.powi(halvings as i32);
            
            U256::from(rate as u128).max(schedule.min_emission_rate)
        } else {
            schedule.initial_rate
        }
    }
    
    /// Get emission for an epoch (24 hours)
    fn get_epoch_emission(&self) -> U256 {
        self.get_current_emission_rate()
    }
}

// ============ Staking Mechanism ============

pub struct StakingMechanism {
    stakes: HashMap<Address, StakeInfo>,
    total_staked: U256,
    min_stake: U256,
    lock_periods: Vec<LockPeriod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StakeInfo {
    staker: Address,
    amount: U256,
    locked_until: DateTime<Utc>,
    reward_multiplier: f64,
    last_reward_claim: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockPeriod {
    pub days: u64,
    pub reward_multiplier: f64,
    pub description: String,
}

impl StakingMechanism {
    pub fn new() -> Self {
        Self {
            stakes: HashMap::new(),
            total_staked: U256::zero(),
            min_stake: U256::from(100) * U256::exp10(18), // 100 tokens
            lock_periods: vec![
                LockPeriod {
                    days: 0,
                    reward_multiplier: 1.0,
                    description: "Flexible".to_string(),
                },
                LockPeriod {
                    days: 30,
                    reward_multiplier: 1.2,
                    description: "1 Month".to_string(),
                },
                LockPeriod {
                    days: 90,
                    reward_multiplier: 1.5,
                    description: "3 Months".to_string(),
                },
                LockPeriod {
                    days: 180,
                    reward_multiplier: 2.0,
                    description: "6 Months".to_string(),
                },
                LockPeriod {
                    days: 365,
                    reward_multiplier: 3.0,
                    description: "1 Year".to_string(),
                },
            ],
        }
    }
    
    /// Stake tokens with lock period
    pub async fn stake(
        &mut self,
        staker: Address,
        amount: U256,
        lock_days: u64,
    ) -> Result<()> {
        if amount < self.min_stake {
            return Err(anyhow::anyhow!("Amount below minimum stake"));
        }
        
        let lock_period = self.lock_periods.iter()
            .find(|p| p.days == lock_days)
            .ok_or_else(|| anyhow::anyhow!("Invalid lock period"))?;
        
        let stake_info = StakeInfo {
            staker,
            amount,
            locked_until: Utc::now() + chrono::Duration::days(lock_days as i64),
            reward_multiplier: lock_period.reward_multiplier,
            last_reward_claim: Utc::now(),
        };
        
        self.stakes.insert(staker, stake_info);
        self.total_staked = self.total_staked + amount;
        
        Ok(())
    }
    
    /// Calculate staking rewards
    pub fn calculate_staking_rewards(
        &self,
        staker: Address,
        base_apy: f64,
    ) -> Result<U256> {
        let stake = self.stakes.get(&staker)
            .ok_or_else(|| anyhow::anyhow!("No stake found"))?;
        
        let days_staked = (Utc::now() - stake.last_reward_claim).num_days() as f64;
        let effective_apy = base_apy * stake.reward_multiplier;
        
        let daily_rate = effective_apy / 365.0;
        let reward = stake.amount.as_u128() as f64 * daily_rate * days_staked;
        
        Ok(U256::from(reward as u128))
    }
}

// ============ Fee Distribution ============

pub struct FeeDistributor {
    fee_structure: FeeStructure,
    collected_fees: CollectedFees,
    distribution_history: Vec<FeeDistribution>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CollectedFees {
    computation_fees: U256,
    network_fees: U256,
    governance_fees: U256,
    total_burned: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FeeDistribution {
    timestamp: DateTime<Utc>,
    staker_rewards: U256,
    governance_treasury: U256,
    burned: U256,
}

impl FeeDistributor {
    pub fn new(fee_structure: FeeStructure) -> Self {
        Self {
            fee_structure,
            collected_fees: CollectedFees::default(),
            distribution_history: Vec::new(),
        }
    }
    
    /// Process a computation fee
    pub fn process_computation_fee(
        &mut self,
        computation_cost: U256,
    ) -> FeeBreakdown {
        let total_fee = U256::from(
            (computation_cost.as_u128() as f64 * self.fee_structure.computation_fee_percent / 100.0) as u128
        );
        
        let burn_amount = U256::from(
            (total_fee.as_u128() as f64 * self.fee_structure.burn_rate_percent / 100.0) as u128
        );
        
        self.collected_fees.computation_fees = self.collected_fees.computation_fees + total_fee;
        self.collected_fees.total_burned = self.collected_fees.total_burned + burn_amount;
        
        FeeBreakdown {
            total_fee,
            burn_amount,
            staker_reward: total_fee - burn_amount,
            treasury_allocation: U256::zero(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeBreakdown {
    pub total_fee: U256,
    pub burn_amount: U256,
    pub staker_reward: U256,
    pub treasury_allocation: U256,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reward_calculation() {
        let economics = TokenEconomics::default();
        let distribution = RewardDistribution::new(economics);
        
        let reward = distribution.calculate_computation_reward(
            0.8,  // complexity
            0.95, // accuracy
            800,  // time in ms
        );
        
        assert!(reward > U256::zero());
    }
    
    #[tokio::test]
    async fn test_staking() {
        let mut staking = StakingMechanism::new();
        let staker: Address = "0x0000000000000000000000000000000000000001".parse().unwrap();
        let amount = U256::from(1000) * U256::exp10(18);
        
        staking.stake(staker, amount, 30).await.unwrap();
        
        let rewards = staking.calculate_staking_rewards(staker, 0.12).unwrap();
        assert_eq!(rewards, U256::zero()); // No time has passed
    }
}