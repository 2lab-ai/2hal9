use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;
use anyhow::Result;
use rand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveConfig {
    pub name: String,
    pub config_type: CollectiveType,
    pub models: Vec<ModelConfig>,
    pub coordination: CoordinationStrategy,
    pub cost_per_hour: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollectiveType {
    OpusOrchestra,      // 6x Claude Opus 4
    SwarmIntelligence,  // 32x lightweight models
    HybridCouncil,      // Mix of different models
    ChaosEngine,        // No coordination, pure emergence
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_type: String,
    pub instances: u32,
    pub role: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationStrategy {
    HierarchicalDemocracy { master: String },
    EmergentConsensus { communication: String },
    SpecialistDemocracy { time_limit_ms: u64 },
    NoCoordination,
}

#[derive(Debug, Clone)]
pub struct CollectiveIntelligence {
    pub id: String,
    pub config: CollectiveConfig,
    decision_channel: mpsc::Sender<CollectiveDecision>,
    state: CollectiveState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CollectiveState {
    active_models: u32,
    consensus_history: Vec<ConsensusEvent>,
    communication_graph: HashMap<String, Vec<String>>,
    decision_distribution: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveDecision {
    pub decision_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub individual_decisions: Vec<IndividualDecision>,
    pub final_decision: serde_json::Value,
    pub consensus_method: String,
    pub dissent_rate: f32,
    pub emergence_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualDecision {
    pub model_id: String,
    pub role: Option<String>,
    pub decision: serde_json::Value,
    pub reasoning: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsensusEvent {
    pub round: u32,
    pub method: String,
    pub convergence_time_ms: u64,
    pub dissent_rate: f32,
}

impl CollectiveIntelligence {
    pub fn new(id: String, config: CollectiveConfig) -> Self {
        let (tx, _rx) = mpsc::channel(100);
        
        Self {
            id,
            config,
            decision_channel: tx,
            state: CollectiveState {
                active_models: 0,
                consensus_history: vec![],
                communication_graph: HashMap::new(),
                decision_distribution: HashMap::new(),
            },
        }
    }
    
    pub async fn make_decision(&mut self, context: serde_json::Value) -> Result<CollectiveDecision> {
        match &self.config.config_type {
            CollectiveType::OpusOrchestra => self.opus_orchestra_decision(context).await,
            CollectiveType::SwarmIntelligence => self.swarm_intelligence_decision(context).await,
            CollectiveType::HybridCouncil => self.hybrid_council_decision(context).await,
            CollectiveType::ChaosEngine => self.chaos_engine_decision(context).await,
        }
    }
    
    async fn opus_orchestra_decision(&mut self, context: serde_json::Value) -> Result<CollectiveDecision> {
        let mut individual_decisions = vec![];
        
        // Simulate 6 Claude Opus instances with different roles
        let roles = vec![
            "master_strategist",
            "pattern_analyzer", 
            "opponent_modeler",
            "creative_thinker",
            "risk_calculator",
            "meta_reasoner"
        ];
        
        for (i, role) in roles.iter().enumerate() {
            let decision = self.simulate_opus_decision(role, &context).await?;
            individual_decisions.push(IndividualDecision {
                model_id: format!("opus_{}", i),
                role: Some(role.to_string()),
                decision: decision.clone(),
                reasoning: Some(format!("{} analysis complete", role)),
                confidence: 0.85 + (rand::random::<f32>() * 0.15),
            });
        }
        
        // Hierarchical democracy with master strategist
        let final_decision = self.hierarchical_consensus(&individual_decisions, "master_strategist").await?;
        
        let dissent_rate = self.calculate_dissent_rate(&individual_decisions);
        
        Ok(CollectiveDecision {
            decision_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            individual_decisions,
            final_decision,
            consensus_method: "hierarchical_democracy".to_string(),
            dissent_rate,
            emergence_detected: false,
        })
    }
    
    async fn swarm_intelligence_decision(&mut self, context: serde_json::Value) -> Result<CollectiveDecision> {
        let mut individual_decisions = vec![];
        
        // Simulate 32 lightweight models
        for i in 0..32 {
            let decision = self.simulate_lightweight_decision(i, &context).await?;
            individual_decisions.push(IndividualDecision {
                model_id: format!("swarm_{}", i),
                role: None,
                decision: decision.clone(),
                reasoning: None,
                confidence: 0.6 + (rand::random::<f32>() * 0.3),
            });
        }
        
        // Emergent consensus through local communication
        let final_decision = self.emergent_consensus(&mut individual_decisions).await?;
        
        // Check for emergence
        let emergence_detected = self.detect_swarm_emergence(&individual_decisions);
        let dissent_rate = self.calculate_dissent_rate(&individual_decisions);
        
        Ok(CollectiveDecision {
            decision_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            individual_decisions,
            final_decision,
            consensus_method: "emergent_consensus".to_string(),
            dissent_rate,
            emergence_detected,
        })
    }
    
    async fn hybrid_council_decision(&mut self, context: serde_json::Value) -> Result<CollectiveDecision> {
        // Mix of different model types
        let mut individual_decisions = vec![];
        
        // 2 Opus for strategy
        for i in 0..2 {
            individual_decisions.push(IndividualDecision {
                model_id: format!("opus_{}", i),
                role: Some("strategic_leader".to_string()),
                decision: serde_json::json!({"choice": if rand::random::<bool>() { 0 } else { 1 }}),
                reasoning: Some("Strategic analysis".to_string()),
                confidence: 0.9,
            });
        }
        
        // 3 rapid responders
        for i in 0..3 {
            individual_decisions.push(IndividualDecision {
                model_id: format!("rapid_{}", i),
                role: Some("rapid_responder".to_string()),
                decision: serde_json::json!({"choice": if rand::random::<bool>() { 0 } else { 1 }}),
                reasoning: None,
                confidence: 0.7,
            });
        }
        
        let final_decision = self.specialist_democracy(&individual_decisions).await?;
        let dissent_rate = self.calculate_dissent_rate(&individual_decisions);
        
        Ok(CollectiveDecision {
            decision_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            individual_decisions,
            final_decision,
            consensus_method: "specialist_democracy".to_string(),
            dissent_rate,
            emergence_detected: false,
        })
    }
    
    async fn chaos_engine_decision(&mut self, context: serde_json::Value) -> Result<CollectiveDecision> {
        let mut individual_decisions = vec![];
        
        // 32 models with no coordination
        for i in 0..32 {
            individual_decisions.push(IndividualDecision {
                model_id: format!("chaos_{}", i),
                role: None,
                decision: serde_json::json!({"choice": if rand::random::<bool>() { 0 } else { 1 }}),
                reasoning: None,
                confidence: rand::random::<f32>(),
            });
        }
        
        // Pure majority vote
        let final_decision = self.majority_vote(&individual_decisions)?;
        let dissent_rate = self.calculate_dissent_rate(&individual_decisions);
        
        Ok(CollectiveDecision {
            decision_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            individual_decisions,
            final_decision,
            consensus_method: "majority_emergence".to_string(),
            dissent_rate,
            emergence_detected: true, // Chaos always has emergence
        })
    }
    
    // Helper methods
    async fn simulate_opus_decision(&self, role: &str, context: &serde_json::Value) -> Result<serde_json::Value> {
        // In real implementation, this would call Claude API
        // For now, simulate strategic decision based on role
        let choice = match role {
            "master_strategist" => if rand::random::<f32>() > 0.6 { 0 } else { 1 },
            "pattern_analyzer" => if context.get("round").and_then(|r| r.as_u64()).unwrap_or(0) % 2 == 0 { 0 } else { 1 },
            _ => if rand::random::<bool>() { 0 } else { 1 },
        };
        
        Ok(serde_json::json!({
            "choice": choice,
            "role": role
        }))
    }
    
    async fn simulate_lightweight_decision(&self, id: usize, context: &serde_json::Value) -> Result<serde_json::Value> {
        // Simulate lightweight model with simple heuristics
        let choice = if id % 2 == 0 { 0 } else { 1 };
        Ok(serde_json::json!({"choice": choice}))
    }
    
    async fn hierarchical_consensus(&self, decisions: &[IndividualDecision], master_role: &str) -> Result<serde_json::Value> {
        // Find master's decision
        let master_decision = decisions.iter()
            .find(|d| d.role.as_deref() == Some(master_role))
            .map(|d| d.decision.clone())
            .unwrap_or_else(|| self.majority_vote(decisions).unwrap());
        
        Ok(master_decision)
    }
    
    async fn emergent_consensus(&self, decisions: &mut [IndividualDecision]) -> Result<serde_json::Value> {
        // Simulate local communication and consensus
        // In real implementation, this would involve iterative message passing
        self.majority_vote(decisions)
    }
    
    async fn specialist_democracy(&self, decisions: &[IndividualDecision]) -> Result<serde_json::Value> {
        // Weight votes by role importance
        let mut weighted_votes: HashMap<i32, f32> = HashMap::new();
        
        for decision in decisions {
            if let Some(choice) = decision.decision.get("choice").and_then(|c| c.as_i64()) {
                let weight = match decision.role.as_deref() {
                    Some("strategic_leader") => 2.0,
                    Some("rapid_responder") => 1.0,
                    _ => 0.5,
                };
                *weighted_votes.entry(choice as i32).or_insert(0.0) += weight;
            }
        }
        
        let best_choice = weighted_votes.iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(choice, _)| *choice)
            .unwrap_or(0);
        
        Ok(serde_json::json!({"choice": best_choice}))
    }
    
    fn majority_vote(&self, decisions: &[IndividualDecision]) -> Result<serde_json::Value> {
        let mut votes: HashMap<i32, u32> = HashMap::new();
        
        for decision in decisions {
            if let Some(choice) = decision.decision.get("choice").and_then(|c| c.as_i64()) {
                *votes.entry(choice as i32).or_insert(0) += 1;
            }
        }
        
        let best_choice = votes.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(choice, _)| *choice)
            .unwrap_or(0);
        
        Ok(serde_json::json!({"choice": best_choice}))
    }
    
    fn calculate_dissent_rate(&self, decisions: &[IndividualDecision]) -> f32 {
        if decisions.is_empty() {
            return 0.0;
        }
        
        let mut choice_counts: HashMap<String, u32> = HashMap::new();
        for decision in decisions {
            let choice_str = decision.decision.to_string();
            *choice_counts.entry(choice_str).or_insert(0) += 1;
        }
        
        let max_agreement = choice_counts.values().max().copied().unwrap_or(0) as f32;
        let total = decisions.len() as f32;
        
        1.0 - (max_agreement / total)
    }
    
    fn detect_swarm_emergence(&self, decisions: &[IndividualDecision]) -> bool {
        // Detect if swarm achieved perfect distribution
        let mut zeros = 0;
        let mut ones = 0;
        
        for decision in decisions {
            if let Some(choice) = decision.decision.get("choice").and_then(|c| c.as_i64()) {
                if choice == 0 {
                    zeros += 1;
                } else {
                    ones += 1;
                }
            }
        }
        
        // Near perfect split indicates emergence
        (zeros as f32 - ones as f32).abs() < 2.0 && zeros > 0 && ones > 0
    }
    
    pub fn get_active_count(&self) -> u32 {
        self.state.active_models
    }
    
    pub fn get_consensus_visualization(&self) -> serde_json::Value {
        serde_json::json!({
            "history": self.state.consensus_history,
            "current_method": self.config.coordination,
        })
    }
    
    pub fn get_communication_graph(&self) -> HashMap<String, Vec<String>> {
        self.state.communication_graph.clone()
    }
    
    pub fn get_decision_distribution(&self) -> HashMap<String, u32> {
        self.state.decision_distribution.clone()
    }
}