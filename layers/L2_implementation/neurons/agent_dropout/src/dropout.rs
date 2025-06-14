//! Agent dropout mechanism for network quality management

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    agent::{AgentLevel, AgentProfile, AgentNeuron},
    AgentError, AgentResult,
};

/// Decision result for agent dropout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DropoutDecision {
    Keep(String),         // Reason to keep
    Drop(String),         // Reason to drop
    Monitor(String),      // Continue monitoring
}

/// Dropout controller for managing agent quality
pub type DropoutController = DropoutOrchestrator;

/// Dropout manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropoutConfig {
    /// Percentage of bottom performers to drop (0.0 - 1.0)
    pub dropout_threshold: f32,
    /// How often to evaluate for dropout
    pub evaluation_interval: std::time::Duration,
    /// Minimum number of agents to maintain
    pub minimum_agent_count: usize,
    /// Grace period for new agents before they can be dropped
    pub grace_period: std::time::Duration,
    /// Minimum number of agents (alias)
    pub min_agents: usize,
    /// Maximum number of agents allowed
    pub max_agents: usize,
}

impl Default for DropoutConfig {
    fn default() -> Self {
        Self {
            dropout_threshold: 0.1, // Bottom 10%
            evaluation_interval: std::time::Duration::from_secs(300), // 5 minutes
            minimum_agent_count: 10,
            grace_period: std::time::Duration::from_secs(600), // 10 minutes
            min_agents: 10,
            max_agents: 1000,
        }
    }
}

/// Agent replacement pool for quick substitution
pub struct AgentReplacementPool {
    available_agents: Arc<RwLock<Vec<AgentCandidate>>>,
    min_pool_size: usize,
}

#[derive(Clone)]
pub struct AgentCandidate {
    pub id: Uuid,
    pub estimated_level: AgentLevel,
    pub specializations: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl AgentReplacementPool {
    pub fn new(min_pool_size: usize) -> Self {
        Self {
            available_agents: Arc::new(RwLock::new(Vec::new())),
            min_pool_size,
        }
    }
    
    pub async fn add_candidate(&self, candidate: AgentCandidate) {
        let mut agents = self.available_agents.write().await;
        agents.push(candidate);
    }
    
    pub async fn get_replacement(&self) -> Option<AgentCandidate> {
        let mut agents = self.available_agents.write().await;
        agents.pop()
    }
    
    pub async fn pool_size(&self) -> usize {
        self.available_agents.read().await.len()
    }
    
    pub async fn ensure_minimum_pool(&self) {
        let current_size = self.pool_size().await;
        if current_size < self.min_pool_size {
            // Generate new candidates
            for _ in current_size..self.min_pool_size {
                let candidate = self.generate_candidate();
                self.add_candidate(candidate).await;
            }
        }
    }
    
    fn generate_candidate(&self) -> AgentCandidate {
        AgentCandidate {
            id: Uuid::new_v4(),
            estimated_level: AgentLevel::from_value(rand::random::<u8>() % 10 + 5).unwrap(),
            specializations: vec!["general".to_string()],
            created_at: Utc::now(),
        }
    }
}

/// Dropout orchestrator managing the dropout process
pub struct DropoutOrchestrator {
    config: DropoutConfig,
    active_agents: Arc<DashMap<Uuid, AgentState>>,
    performance_history: Arc<DashMap<Uuid, Vec<PerformanceSnapshot>>>,
    replacement_pool: Arc<AgentReplacementPool>,
    last_evaluation: Arc<RwLock<DateTime<Utc>>>,
}

struct AgentState {
    agent: Box<dyn AgentNeuron>,
    joined_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
    dropout_warnings: u32,
}

#[derive(Clone, Serialize, Deserialize)]
struct PerformanceSnapshot {
    timestamp: DateTime<Utc>,
    overall_score: f32,
    task_count: u64,
    peer_rating: f32,
}

impl DropoutOrchestrator {
    /// Create a new dropout controller
    pub fn new(
        _memory_limit: usize,
        idle_timeout: std::time::Duration,
        quality_threshold: f32,
    ) -> Self {
        let config = DropoutConfig {
            dropout_threshold: quality_threshold,
            evaluation_interval: idle_timeout,
            minimum_agent_count: 10,
            grace_period: std::time::Duration::from_secs(600),
            min_agents: 10,
            max_agents: 1000,
        };
        
        Self::with_config(config)
    }
    
    /// Create with specific config
    pub fn with_config(config: DropoutConfig) -> Self {
        Self {
            config,
            active_agents: Arc::new(DashMap::new()),
            performance_history: Arc::new(DashMap::new()),
            replacement_pool: Arc::new(AgentReplacementPool::new(20)),
            last_evaluation: Arc::new(RwLock::new(Utc::now())),
        }
    }
    
    /// Register a new agent
    pub async fn register_agent(&self, agent: Box<dyn AgentNeuron>) -> AgentResult<()> {
        let agent_id = agent.id();
        let state = AgentState {
            agent,
            joined_at: Utc::now(),
            last_activity: Utc::now(),
            dropout_warnings: 0,
        };
        
        self.active_agents.insert(agent_id, state);
        self.performance_history.insert(agent_id, Vec::new());
        
        Ok(())
    }
    
    /// Main health check cycle
    pub async fn health_check_cycle(&self) -> AgentResult<DropoutReport> {
        // Check if it's time to evaluate
        let mut last_eval = self.last_evaluation.write().await;
        let now = Utc::now();
        
        if now.signed_duration_since(*last_eval).to_std().unwrap() < self.config.evaluation_interval {
            return Ok(DropoutReport {
                evaluated_agents: 0,
                dropped_agents: vec![],
                replaced_agents: vec![],
                warnings_issued: 0,
            });
        }
        
        *last_eval = now;
        drop(last_eval);
        
        // Collect performance metrics
        let rankings = self.calculate_agent_rankings().await;
        
        // Identify bottom performers
        let dropout_candidates = self.identify_bottom_performers(&rankings);
        
        // Process dropouts
        let mut report = DropoutReport {
            evaluated_agents: rankings.len(),
            dropped_agents: vec![],
            replaced_agents: vec![],
            warnings_issued: 0,
        };
        
        for agent_id in dropout_candidates {
            // Check grace period
            if self.is_in_grace_period(agent_id).await {
                self.issue_warning(agent_id).await;
                report.warnings_issued += 1;
                continue;
            }
            
            // Perform graceful dropout
            if let Ok(()) = self.graceful_dropout(agent_id).await {
                report.dropped_agents.push(agent_id);
                
                // Request replacement
                if let Ok(new_agent_id) = self.request_replacement().await {
                    report.replaced_agents.push(new_agent_id);
                }
            }
        }
        
        // Ensure replacement pool is maintained
        self.replacement_pool.ensure_minimum_pool().await;
        
        Ok(report)
    }
    
    async fn calculate_agent_rankings(&self) -> Vec<(Uuid, f32)> {
        let mut rankings = Vec::new();
        
        for entry in self.active_agents.iter() {
            let agent_id = *entry.key();
            let state = entry.value();
            let performance = state.agent.performance_score();
            
            // Take snapshot
            let snapshot = PerformanceSnapshot {
                timestamp: Utc::now(),
                overall_score: performance,
                task_count: 0, // Would come from performance tracker
                peer_rating: performance, // Simplified
            };
            
            if let Some(mut history) = self.performance_history.get_mut(&agent_id) {
                history.push(snapshot);
                
                // Keep only last 100 snapshots
                if history.len() > 100 {
                    history.remove(0);
                }
            }
            
            rankings.push((agent_id, performance));
        }
        
        // Sort by performance (ascending, so worst performers first)
        rankings.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        rankings
    }
    
    fn identify_bottom_performers(&self, rankings: &[(Uuid, f32)]) -> Vec<Uuid> {
        let total_agents = rankings.len();
        
        // Don't drop if we're at minimum agent count
        if total_agents <= self.config.minimum_agent_count {
            return vec![];
        }
        
        // Calculate how many to drop
        let dropout_count = ((total_agents as f32 * self.config.dropout_threshold) as usize)
            .min(total_agents - self.config.minimum_agent_count);
        
        rankings.iter()
            .take(dropout_count)
            .map(|(id, _)| *id)
            .collect()
    }
    
    async fn is_in_grace_period(&self, agent_id: Uuid) -> bool {
        if let Some(state) = self.active_agents.get(&agent_id) {
            let age = Utc::now().signed_duration_since(state.joined_at);
            age.to_std().unwrap() < self.config.grace_period
        } else {
            false
        }
    }
    
    async fn issue_warning(&self, agent_id: Uuid) {
        if let Some(mut state) = self.active_agents.get_mut(&agent_id) {
            state.dropout_warnings += 1;
        }
    }
    
    async fn graceful_dropout(&self, agent_id: Uuid) -> AgentResult<()> {
        // Remove agent from active set
        if let Some((_, state)) = self.active_agents.remove(&agent_id) {
            // Log dropout event
            tracing::info!(
                "Agent {} dropped. Level: {}, Performance: {:.2}",
                agent_id,
                state.agent.self_assess_level().value(),
                state.agent.performance_score()
            );
            
            // Clean up performance history
            self.performance_history.remove(&agent_id);
            
            Ok(())
        } else {
            Err(AgentError::NotFound(agent_id))
        }
    }
    
    async fn request_replacement(&self) -> AgentResult<Uuid> {
        if let Some(candidate) = self.replacement_pool.get_replacement().await {
            // In a real implementation, this would create a new agent instance
            // For now, just return the candidate ID
            tracing::info!(
                "New agent {} joined. Level: {}",
                candidate.id,
                candidate.estimated_level.value()
            );
            
            Ok(candidate.id)
        } else {
            Err(AgentError::NetworkError("No replacement available".to_string()))
        }
    }
    
    /// Check if an agent should be dropped
    pub async fn should_dropout(&self, _profile: &AgentProfile, quality_score: f32) -> bool {
        // Check quality threshold
        if quality_score < self.config.dropout_threshold {
            return true;
        }
        
        // Check if we're at max capacity
        let agent_count = self.active_agents.len();
        if agent_count >= self.config.max_agents {
            // Drop lowest performers
            let mut scores: Vec<(Uuid, f32)> = Vec::new();
            for entry in self.active_agents.iter() {
                scores.push((*entry.key(), entry.value().agent.performance_score()));
            }
            scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            
            // Check if this agent is in the bottom percentage
            let dropout_count = (agent_count as f32 * self.config.dropout_threshold) as usize;
            if let Some((_, threshold_score)) = scores.get(dropout_count) {
                return quality_score < *threshold_score;
            }
        }
        
        false
    }
    
    /// Update agent activity timestamp
    pub async fn update_agent_activity(&self, agent_id: Uuid) {
        if let Some(mut state) = self.active_agents.get_mut(&agent_id) {
            state.last_activity = Utc::now();
        }
    }
    
    /// Check memory pressure
    pub async fn check_memory_pressure(&self) -> bool {
        // For now, just check agent count
        self.active_agents.len() < self.config.max_agents
    }
    
    /// Get current network statistics
    pub async fn network_stats(&self) -> NetworkStats {
        let agents: Vec<_> = self.active_agents.iter()
            .map(|entry| (entry.value().agent.self_assess_level().value(), 
                         entry.value().agent.performance_score()))
            .collect();
        
        let total_agents = agents.len();
        let avg_level = if total_agents > 0 {
            agents.iter().map(|(level, _)| *level as f32).sum::<f32>() / total_agents as f32
        } else {
            0.0
        };
        
        let avg_performance = if total_agents > 0 {
            agents.iter().map(|(_, perf)| perf).sum::<f32>() / total_agents as f32
        } else {
            0.0
        };
        
        NetworkStats {
            total_agents,
            average_level: avg_level,
            average_performance: avg_performance,
            dropout_rate: self.config.dropout_threshold,
            last_evaluation: *self.last_evaluation.read().await,
        }
    }
}

/// Report of a dropout cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropoutReport {
    pub evaluated_agents: usize,
    pub dropped_agents: Vec<Uuid>,
    pub replaced_agents: Vec<Uuid>,
    pub warnings_issued: usize,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_agents: usize,
    pub average_level: f32,
    pub average_performance: f32,
    pub dropout_rate: f32,
    pub last_evaluation: DateTime<Utc>,
}

/// Evolutionary optimizer for long-term network improvement
pub struct EvolutionaryOptimizer {
    #[allow(dead_code)]
    diversity_factor: f32,
    mutation_rate: f32,
    fitness_history: Vec<f32>,
}

impl EvolutionaryOptimizer {
    pub fn new(diversity_factor: f32, mutation_rate: f32) -> Self {
        Self {
            diversity_factor,
            mutation_rate,
            fitness_history: Vec::new(),
        }
    }
    
    pub fn optimize_network(&mut self, current_stats: &NetworkStats) -> OptimizationSuggestion {
        // Track fitness over time
        let fitness = current_stats.average_performance;
        self.fitness_history.push(fitness);
        
        // Keep only last 100 measurements
        if self.fitness_history.len() > 100 {
            self.fitness_history.remove(0);
        }
        
        // Analyze trends
        let trend = self.calculate_fitness_trend();
        
        // Generate suggestions
        let mut suggestion = OptimizationSuggestion {
            adjust_dropout_rate: None,
            introduce_random_agent: false,
            specialization_needed: vec![],
        };
        
        // If performance is stagnating, increase diversity
        if trend < 0.01 && self.fitness_history.len() > 20 {
            suggestion.introduce_random_agent = rand::random::<f32>() < self.mutation_rate;
            
            // Consider adjusting dropout rate
            if current_stats.average_performance < 0.6 {
                suggestion.adjust_dropout_rate = Some(current_stats.dropout_rate * 0.9); // Reduce dropout
            }
        }
        
        // If performance is declining, identify needed specializations
        if trend < -0.01 {
            suggestion.specialization_needed = vec![
                "problem_solving".to_string(),
                "optimization".to_string(),
            ];
        }
        
        suggestion
    }
    
    fn calculate_fitness_trend(&self) -> f32 {
        if self.fitness_history.len() < 2 {
            return 0.0;
        }
        
        // Simple linear regression
        let n = self.fitness_history.len() as f32;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = self.fitness_history.iter().sum::<f32>() / n;
        
        let mut numerator = 0.0;
        let mut denominator = 0.0;
        
        for (i, &y) in self.fitness_history.iter().enumerate() {
            let x = i as f32;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean) * (x - x_mean);
        }
        
        if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub adjust_dropout_rate: Option<f32>,
    pub introduce_random_agent: bool,
    pub specialization_needed: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Create mock AgentNeuron implementation for testing
    // #[tokio::test]
    // async fn test_dropout_orchestrator() {
    //     let config = DropoutConfig {
    //         dropout_threshold: 0.2, // Drop bottom 20%
    //         evaluation_interval: std::time::Duration::from_millis(100),
    //         minimum_agent_count: 3,
    //         grace_period: std::time::Duration::from_millis(50),
    //     };
    //     
    //     let orchestrator = DropoutOrchestrator::with_config(config);
    //     
    //     // Register some agents
    //     for i in 1..=5 {
    //         let agent = AgentNeuron::new(
    //             AgentLevel::from_value(i * 4).unwrap(),
    //             vec!["test".to_string()],
    //         );
    //         orchestrator.register_agent(agent).await.unwrap();
    //     }
    //     
    //     // Wait for grace period
    //     tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    //     
    //     // Run health check
    //     let report = orchestrator.health_check_cycle().await.unwrap();
    //     
    //     assert_eq!(report.evaluated_agents, 5);
    //     assert!(report.dropped_agents.len() <= 2); // Should drop at most 2 (to maintain minimum)
    // }

    #[test]
    fn test_evolutionary_optimizer() {
        let mut optimizer = EvolutionaryOptimizer::new(0.2, 0.1);
        
        let stats = NetworkStats {
            total_agents: 10,
            average_level: 10.0,
            average_performance: 0.5,
            dropout_rate: 0.1,
            last_evaluation: Utc::now(),
        };
        
        // Simulate improving performance
        for i in 0..10 {
            let mut improving_stats = stats.clone();
            improving_stats.average_performance = 0.5 + (i as f32 * 0.02);
            let suggestion = optimizer.optimize_network(&improving_stats);
            
            // Should not suggest changes when improving
            assert!(suggestion.adjust_dropout_rate.is_none());
        }
    }
}