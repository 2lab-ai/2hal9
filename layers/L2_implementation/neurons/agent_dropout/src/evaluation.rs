//! Mutual evaluation system for agents

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    agent::{AgentLevel, AssessmentQuestion, AssessmentResponse, MutualEvaluation, AssessmentScores},
    AgentResult,
};

/// Evaluation protocol for mutual agent assessment
#[async_trait]
pub trait EvaluationProtocol: Send + Sync {
    /// Request evaluation from all active agents
    async fn broadcast_evaluation_request(
        &self,
        agent_id: Uuid,
        response: &AssessmentResponse,
    ) -> Vec<MutualEvaluation>;
    
    /// Aggregate evaluations to estimate level
    async fn aggregate_evaluations(
        &self,
        evaluations: Vec<MutualEvaluation>,
    ) -> LevelEstimate;
    
    /// Submit evaluation for another agent
    async fn submit_evaluation(
        &self,
        evaluation: MutualEvaluation,
    ) -> AgentResult<()>;
}

/// Result of agent evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub overall_score: f32,
    pub level_estimate: AgentLevel,
    pub category_scores: std::collections::HashMap<crate::agent::QuestionCategory, f32>,
    pub time_efficiency: f32,
    pub consistency_score: f32,
}

/// Level estimation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelEstimate {
    pub estimated_level: AgentLevel,
    pub confidence: f32,
    pub evaluator_count: usize,
    pub consensus_score: f32,
}

/// Evaluation engine implementation
pub struct EvaluationEngine {
    evaluations: Arc<DashMap<Uuid, Vec<MutualEvaluation>>>,
    active_agents: Arc<DashMap<Uuid, AgentInfo>>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct AgentInfo {
    id: Uuid,
    level: AgentLevel,
    last_seen: DateTime<Utc>,
}

impl Default for EvaluationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EvaluationEngine {
    pub fn new() -> Self {
        Self {
            evaluations: Arc::new(DashMap::new()),
            active_agents: Arc::new(DashMap::new()),
        }
    }
    
    /// Register an agent as active
    pub fn register_agent(&self, agent_id: Uuid, level: AgentLevel) {
        self.active_agents.insert(
            agent_id,
            AgentInfo {
                id: agent_id,
                level,
                last_seen: Utc::now(),
            },
        );
    }
    
    /// Evaluate an assessment response
    pub fn evaluate_response(
        &self,
        evaluator_id: Uuid,
        evaluatee_id: Uuid,
        question: &AssessmentQuestion,
        response: &AssessmentResponse,
    ) -> MutualEvaluation {
        // Simulate evaluation logic based on question difficulty and response quality
        let scores = self.calculate_scores(question, response);
        
        MutualEvaluation {
            evaluator: evaluator_id,
            evaluated: evaluatee_id,
            scores,
            timestamp: Utc::now(),
        }
    }
    
    fn calculate_scores(
        &self,
        _question: &AssessmentQuestion,
        response: &AssessmentResponse,
    ) -> AssessmentScores {
        // Simplified scoring logic - in production, this would use NLP/ML
        let base_score = response.confidence;
        let time_penalty = if response.time_taken.as_secs() > 60 { 0.1 } else { 0.0 };
        
        AssessmentScores {
            accuracy: (base_score - time_penalty).max(0.0),
            reasoning: base_score * 0.95,
            creativity: base_score * 0.9,
            speed: (1.0 - time_penalty).max(0.0),
            consistency: base_score,
        }
    }
    
    #[allow(dead_code)]
    fn calculate_confidence(&self, evaluator_id: Uuid, question_difficulty: AgentLevel) -> f32 {
        if let Some(evaluator) = self.active_agents.get(&evaluator_id) {
            // Higher level evaluators have more confidence in their assessments
            let level_diff = evaluator.level.value() as i32 - question_difficulty.value() as i32;
            match level_diff {
                d if d >= 5 => 0.95,
                d if d >= 0 => 0.8,
                d if d >= -5 => 0.6,
                _ => 0.4,
            }
        } else {
            0.5
        }
    }
}

#[async_trait]
impl EvaluationProtocol for EvaluationEngine {
    async fn broadcast_evaluation_request(
        &self,
        agent_id: Uuid,
        response: &AssessmentResponse,
    ) -> Vec<MutualEvaluation> {
        let mut evaluations = Vec::new();
        
        // Get sample question (in real implementation, this would be passed in)
        let question = AssessmentQuestion {
            id: response.question_id,
            category: crate::agent::QuestionCategory::LogicalReasoning,
            difficulty: AgentLevel::from_value(10).unwrap(),
            content: "Sample question".to_string(),
            time_limit: None,
        };
        
        // Collect evaluations from all active agents
        for evaluator in self.active_agents.iter() {
            if evaluator.key() != &agent_id {
                let evaluation = self.evaluate_response(
                    *evaluator.key(),
                    agent_id,
                    &question,
                    response,
                );
                evaluations.push(evaluation);
            }
        }
        
        evaluations
    }
    
    async fn aggregate_evaluations(
        &self,
        evaluations: Vec<MutualEvaluation>,
    ) -> LevelEstimate {
        if evaluations.is_empty() {
            return LevelEstimate {
                estimated_level: AgentLevel::from_value(5).unwrap(),
                confidence: 0.0,
                evaluator_count: 0,
                consensus_score: 0.0,
            };
        }
        
        // Calculate weighted average based on evaluator confidence
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;
        let mut level_estimates = Vec::new();
        
        for eval in &evaluations {
            let score = eval.scores.overall();
            let weight = 0.8; // Default confidence
            weighted_sum += score * weight;
            weight_total += weight;
            
            // Convert score to level estimate (1-20)
            let estimated_level = ((score * 19.0) + 1.0).round() as u8;
            level_estimates.push(estimated_level);
        }
        
        let average_score = if weight_total > 0.0 {
            weighted_sum / weight_total
        } else {
            0.5
        };
        
        // Calculate consensus (how much evaluators agree)
        let consensus = self.calculate_consensus(&level_estimates);
        
        // Final level estimate
        let final_level = ((average_score * 19.0) + 1.0).round() as u8;
        
        LevelEstimate {
            estimated_level: AgentLevel::from_value(final_level.clamp(1, 20)).unwrap(),
            confidence: average_score,
            evaluator_count: evaluations.len(),
            consensus_score: consensus,
        }
    }
    
    async fn submit_evaluation(
        &self,
        evaluation: MutualEvaluation,
    ) -> AgentResult<()> {
        self.evaluations
            .entry(evaluation.evaluated)
            .or_default()
            .push(evaluation);
        Ok(())
    }
}

impl EvaluationEngine {
    fn calculate_consensus(&self, estimates: &[u8]) -> f32 {
        if estimates.is_empty() {
            return 0.0;
        }
        
        let mean = estimates.iter().sum::<u8>() as f32 / estimates.len() as f32;
        let variance = estimates.iter()
            .map(|&x| {
                let diff = x as f32 - mean;
                diff * diff
            })
            .sum::<f32>() / estimates.len() as f32;
        
        // Convert variance to consensus score (lower variance = higher consensus)
        1.0 / (1.0 + variance.sqrt())
    }
}

/// Bayesian level estimator for more sophisticated estimation
pub struct BayesianLevelEstimator {
    prior_distribution: Vec<f32>, // Prior beliefs about level distribution
}

impl Default for BayesianLevelEstimator {
    fn default() -> Self {
        Self::new()
    }
}

impl BayesianLevelEstimator {
    pub fn new() -> Self {
        // Initialize with uniform prior
        let mut prior = vec![1.0 / 20.0; 20];
        
        // Slight bias towards middle levels (bell curve)
        for (i, p) in prior.iter_mut().enumerate() {
            let distance_from_center = ((i as f32 - 10.0) / 10.0).abs();
            *p *= 1.0 - (distance_from_center * 0.3);
        }
        
        // Normalize
        let sum: f32 = prior.iter().sum();
        for p in &mut prior {
            *p /= sum;
        }
        
        Self {
            prior_distribution: prior,
        }
    }
    
    pub fn estimate_level(
        &self,
        evaluations: &[MutualEvaluation],
    ) -> LevelEstimate {
        let mut posterior = self.prior_distribution.clone();
        
        // Update posterior based on evaluations
        for eval in evaluations {
            let score = eval.scores.overall();
            let confidence = 0.8; // Default confidence
            
            // Update each level's probability
            for (level_idx, prob) in posterior.iter_mut().enumerate() {
                let level = level_idx + 1;
                let likelihood = self.likelihood(score, level as u8, confidence);
                *prob *= likelihood;
            }
            
            // Normalize
            let sum: f32 = posterior.iter().sum();
            if sum > 0.0 {
                for p in &mut posterior {
                    *p /= sum;
                }
            }
        }
        
        // Find maximum a posteriori (MAP) estimate
        let (best_level_idx, &max_prob) = posterior
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
        
        let best_level = (best_level_idx + 1) as u8;
        
        // Calculate confidence as entropy of distribution
        let entropy = -posterior.iter()
            .filter(|&&p| p > 0.0)
            .map(|&p| p * p.ln())
            .sum::<f32>();
        let max_entropy = -(1.0_f32 / 20.0).ln() * 20.0;
        let confidence = 1.0 - (entropy / max_entropy);
        
        LevelEstimate {
            estimated_level: AgentLevel::from_value(best_level).unwrap(),
            confidence: max_prob,
            evaluator_count: evaluations.len(),
            consensus_score: confidence,
        }
    }
    
    fn likelihood(&self, score: f32, level: u8, confidence: f32) -> f32 {
        // Likelihood of observing this score given the true level
        let expected_score = level as f32 / 20.0;
        let diff = (score - expected_score).abs();
        let sigma = 0.2 * (1.0 - confidence + 0.1); // Uncertainty increases with lower confidence
        
        // Gaussian likelihood
        (-diff * diff / (2.0 * sigma * sigma)).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_evaluation_engine() {
        let engine = EvaluationEngine::new();
        
        // Register some agents
        engine.register_agent(Uuid::new_v4(), AgentLevel::from_value(5).unwrap());
        engine.register_agent(Uuid::new_v4(), AgentLevel::from_value(10).unwrap());
        engine.register_agent(Uuid::new_v4(), AgentLevel::from_value(15).unwrap());
        
        // Create a test response
        let response = AssessmentResponse {
            question_id: Uuid::new_v4(),
            answer: "Test answer".to_string(),
            time_taken: std::time::Duration::from_secs(30),
            confidence: 0.8,
        };
        
        // Get evaluations
        let agent_id = Uuid::new_v4();
        let evaluations = engine.broadcast_evaluation_request(
            agent_id,
            &response,
        ).await;
        
        assert_eq!(evaluations.len(), 3); // 3 evaluators
        
        // Aggregate evaluations
        let estimate = engine.aggregate_evaluations(evaluations).await;
        assert!(estimate.estimated_level.value() >= 1);
        assert!(estimate.estimated_level.value() <= 20);
        assert!(estimate.confidence >= 0.0);
        assert!(estimate.confidence <= 1.0);
    }

    #[test]
    fn test_bayesian_estimator() {
        let estimator = BayesianLevelEstimator::new();
        
        // Create test evaluations
        let mut evaluations = Vec::new();
        for _i in 0..5 {
            evaluations.push(MutualEvaluation {
                evaluator: Uuid::new_v4(),
                evaluated: Uuid::new_v4(),
                scores: AssessmentScores {
                    accuracy: 0.7,
                    reasoning: 0.8,
                    creativity: 0.75,
                    speed: 0.85,
                    consistency: 0.8,
                },
                timestamp: Utc::now(),
            });
        }
        
        let estimate = estimator.estimate_level(&evaluations);
        assert!(estimate.estimated_level.value() >= 1);
        assert!(estimate.estimated_level.value() <= 20);
    }
}