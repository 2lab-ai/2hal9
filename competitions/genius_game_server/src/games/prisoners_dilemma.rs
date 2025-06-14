use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use super::{Game, GameState, PlayerAction, GameResult, EmergenceMetrics};
use anyhow::Result;

/// Classic Prisoner's Dilemma with reputation tracking
pub struct PrisonersDilemmaGame {
    id: Uuid,
    round: u32,
    max_rounds: u32,
    players: Vec<String>,
    history: Vec<RoundResult>,
    reputation_scores: HashMap<String, f32>,
    special_rules: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoundResult {
    round: u32,
    actions: HashMap<String, PrisonerAction>,
    payoffs: HashMap<String, i32>,
    cooperations: Vec<(String, String)>,
    defections: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum PrisonerAction {
    Cooperate,
    Defect,
}

impl PrisonersDilemmaGame {
    pub fn new(max_rounds: u32, special_rules: HashMap<String, String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            round: 0,
            max_rounds,
            players: Vec::new(),
            history: Vec::new(),
            reputation_scores: HashMap::new(),
            special_rules,
        }
    }
    
    fn calculate_payoff(&self, my_action: PrisonerAction, opponent_action: PrisonerAction) -> (i32, i32) {
        match (my_action, opponent_action) {
            (PrisonerAction::Cooperate, PrisonerAction::Cooperate) => (3, 3),  // Both cooperate
            (PrisonerAction::Cooperate, PrisonerAction::Defect) => (0, 5),    // I cooperate, they defect
            (PrisonerAction::Defect, PrisonerAction::Cooperate) => (5, 0),    // I defect, they cooperate
            (PrisonerAction::Defect, PrisonerAction::Defect) => (1, 1),       // Both defect
        }
    }
    
    fn update_reputation(&mut self, player: &str, action: PrisonerAction, round_payoff: i32) {
        let current = self.reputation_scores.get(player).copied().unwrap_or(0.5);
        
        // Reputation increases with cooperation, decreases with defection
        let change = match action {
            PrisonerAction::Cooperate => 0.1,
            PrisonerAction::Defect => -0.15,
        };
        
        // Bonus for sustained cooperation
        let cooperation_bonus = if self.history.len() >= 3 {
            let recent_cooperations = self.history.iter()
                .rev()
                .take(3)
                .filter(|r| {
                    r.actions.get(player)
                        .map(|&a| a == PrisonerAction::Cooperate)
                        .unwrap_or(false)
                })
                .count();
            (recent_cooperations as f32) * 0.05
        } else {
            0.0
        };
        
        let new_reputation = (current + change + cooperation_bonus).clamp(0.0, 1.0);
        self.reputation_scores.insert(player.to_string(), new_reputation);
    }
    
    fn detect_emergence(&self) -> EmergenceMetrics {
        if self.history.len() < 5 {
            return EmergenceMetrics {
                emergence_detected: false,
                coordination_score: 0.0,
                collective_intelligence_index: 0.0,
                decision_diversity_index: 0.0,
                strategic_depth: 0.0,
                special_patterns: HashMap::new(),
            };
        }
        
        // Check for emergent cooperation patterns
        let recent_rounds = self.history.iter().rev().take(10).collect::<Vec<_>>();
        
        // Measure sustained cooperation
        let cooperation_rate = recent_rounds.iter()
            .map(|r| {
                let total_actions = r.actions.len() as f32;
                let cooperations = r.actions.values()
                    .filter(|&&a| a == PrisonerAction::Cooperate)
                    .count() as f32;
                cooperations / total_actions
            })
            .sum::<f32>() / recent_rounds.len() as f32;
        
        // Check for tit-for-tat patterns
        let mut tit_for_tat_count = 0;
        for i in 1..recent_rounds.len() {
            let prev_round = &recent_rounds[i];
            let curr_round = &recent_rounds[i-1];
            
            for player in &self.players {
                if let (Some(&prev_action), Some(&curr_action)) = 
                    (prev_round.actions.get(player), curr_round.actions.get(player)) {
                    // Check if player mirrors opponent's previous move
                    for opponent in &self.players {
                        if player != opponent {
                            if let Some(&opp_prev_action) = prev_round.actions.get(opponent) {
                                if curr_action == opp_prev_action {
                                    tit_for_tat_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        let tit_for_tat_ratio = tit_for_tat_count as f32 / 
            (recent_rounds.len() * self.players.len() * (self.players.len() - 1)) as f32;
        
        // Calculate average reputation
        let avg_reputation = self.reputation_scores.values().sum::<f32>() / 
            self.reputation_scores.len().max(1) as f32;
        
        // Emergence is detected when:
        // 1. High cooperation rate (> 0.7)
        // 2. Stable tit-for-tat patterns
        // 3. High average reputation
        let emergence_detected = cooperation_rate > 0.7 && 
                                tit_for_tat_ratio > 0.3 && 
                                avg_reputation > 0.6;
        
        let mut special_patterns = HashMap::new();
        special_patterns.insert("cooperation_rate".to_string(), cooperation_rate.to_string());
        special_patterns.insert("tit_for_tat_ratio".to_string(), tit_for_tat_ratio.to_string());
        special_patterns.insert("avg_reputation".to_string(), avg_reputation.to_string());
        
        EmergenceMetrics {
            emergence_detected,
            coordination_score: cooperation_rate,
            collective_intelligence_index: avg_reputation,
            decision_diversity_index: 1.0 - tit_for_tat_ratio, // Diversity decreases with conformity
            strategic_depth: tit_for_tat_ratio,
            special_patterns,
        }
    }
}

#[async_trait]
impl Game for PrisonersDilemmaGame {
    async fn get_state(&self) -> Result<GameState> {
        let available_choices = vec!["cooperate".to_string(), "defect".to_string()];
        
        let mut player_states = HashMap::new();
        for player in &self.players {
            let reputation = self.reputation_scores.get(player).copied().unwrap_or(0.5);
            let history_summary = self.history.iter()
                .rev()
                .take(5)
                .filter_map(|r| r.actions.get(player))
                .map(|&a| match a {
                    PrisonerAction::Cooperate => "C",
                    PrisonerAction::Defect => "D",
                })
                .collect::<Vec<_>>()
                .join("");
            
            player_states.insert(player.clone(), serde_json::json!({
                "reputation": reputation,
                "recent_history": history_summary,
            }));
        }
        
        Ok(GameState {
            game_id: self.id,
            game_type: "prisoners_dilemma".to_string(),
            round: self.round,
            players: self.players.clone(),
            current_state: serde_json::json!({
                "round": self.round,
                "max_rounds": self.max_rounds,
                "reputation_scores": self.reputation_scores,
                "available_choices": available_choices,
            }),
            available_actions: available_choices,
            scores: self.reputation_scores.iter()
                .map(|(k, &v)| (k.clone(), (v * 100.0) as i32))
                .collect(),
            player_states,
            is_complete: self.round >= self.max_rounds,
            special_data: Some(serde_json::json!({
                "payoff_matrix": {
                    "CC": [3, 3],
                    "CD": [0, 5],
                    "DC": [5, 0],
                    "DD": [1, 1],
                },
                "history_length": self.history.len(),
            })),
        })
    }
    
    async fn process_action(&mut self, action: PlayerAction) -> Result<()> {
        // Store actions until all players have acted
        Ok(())
    }
    
    async fn advance_round(&mut self) -> Result<GameResult> {
        self.round += 1;
        
        // Generate random actions for this simplified version
        let mut actions = HashMap::new();
        let mut payoffs = HashMap::new();
        let mut cooperations = Vec::new();
        let mut defections = Vec::new();
        
        for player in &self.players {
            let action = if rand::random::<f32>() > 0.5 {
                PrisonerAction::Cooperate
            } else {
                PrisonerAction::Defect
            };
            actions.insert(player.clone(), action);
        }
        
        // Calculate payoffs for all pairs
        for i in 0..self.players.len() {
            for j in i+1..self.players.len() {
                let p1 = &self.players[i];
                let p2 = &self.players[j];
                
                let a1 = actions[p1];
                let a2 = actions[p2];
                
                let (pay1, pay2) = self.calculate_payoff(a1, a2);
                
                *payoffs.entry(p1.clone()).or_insert(0) += pay1;
                *payoffs.entry(p2.clone()).or_insert(0) += pay2;
                
                match (a1, a2) {
                    (PrisonerAction::Cooperate, PrisonerAction::Cooperate) => {
                        cooperations.push((p1.clone(), p2.clone()));
                    }
                    (PrisonerAction::Defect, PrisonerAction::Cooperate) => {
                        defections.push((p1.clone(), p2.clone()));
                    }
                    (PrisonerAction::Cooperate, PrisonerAction::Defect) => {
                        defections.push((p2.clone(), p1.clone()));
                    }
                    _ => {}
                }
            }
        }
        
        // Update reputations
        for (player, &action) in &actions {
            let payoff = payoffs.get(player).copied().unwrap_or(0);
            self.update_reputation(player, action, payoff);
        }
        
        let round_result = RoundResult {
            round: self.round,
            actions: actions.clone(),
            payoffs: payoffs.clone(),
            cooperations,
            defections,
        };
        
        self.history.push(round_result);
        
        let emergence = self.detect_emergence();
        
        Ok(GameResult {
            round: self.round,
            scores: payoffs,
            round_winners: vec![], // No single winner in PD
            is_final_round: self.round >= self.max_rounds,
            emergence_metrics: emergence,
            special_data: Some(serde_json::json!({
                "actions": actions,
                "reputation_changes": self.reputation_scores,
            })),
        })
    }
    
    fn add_player(&mut self, player_id: String) -> Result<()> {
        if !self.players.contains(&player_id) {
            self.players.push(player_id.clone());
            self.reputation_scores.insert(player_id, 0.5);
        }
        Ok(())
    }
    
    fn get_game_id(&self) -> Uuid {
        self.id
    }
}