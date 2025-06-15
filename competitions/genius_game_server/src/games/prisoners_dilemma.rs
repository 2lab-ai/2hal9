use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;
use rand::Rng;

pub struct PrisonersDilemmaGame {
    max_rounds: u32,
    reputation_scores: HashMap<String, f32>,
    history: Vec<PDRoundData>,
}

#[derive(Debug, Clone)]
struct PDRoundData {
    actions: HashMap<String, PDAction>,
    payoffs: HashMap<String, i32>,
    cooperations: Vec<(String, String)>,
    defections: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PDAction {
    Cooperate,
    Defect,
}

impl Default for PrisonersDilemmaGame {
    fn default() -> Self {
        Self::new()
    }
}

impl PrisonersDilemmaGame {
    pub fn new() -> Self {
        Self {
            max_rounds: 100,
            reputation_scores: HashMap::new(),
            history: Vec::new(),
        }
    }
    
    fn calculate_payoff(&self, my_action: PDAction, opponent_action: PDAction) -> (i32, i32) {
        match (my_action, opponent_action) {
            (PDAction::Cooperate, PDAction::Cooperate) => (3, 3),  // Both cooperate
            (PDAction::Cooperate, PDAction::Defect) => (0, 5),    // I cooperate, they defect
            (PDAction::Defect, PDAction::Cooperate) => (5, 0),    // I defect, they cooperate
            (PDAction::Defect, PDAction::Defect) => (1, 1),       // Both defect
        }
    }
    
    fn update_reputation(&mut self, player: &str, action: PDAction, payoff: i32) {
        let current = self.reputation_scores.get(player).copied().unwrap_or(0.5);
        let action_score = match action {
            PDAction::Cooperate => 0.1,
            PDAction::Defect => -0.1,
        };
        let payoff_bonus = (payoff as f32 / 10.0).min(0.1);
        
        let new_reputation = (current + action_score + payoff_bonus).clamp(0.0, 1.0);
        self.reputation_scores.insert(player.to_string(), new_reputation);
    }
    
    fn detect_emergence(&self, state: &GameState) -> Option<EmergenceEvent> {
        if self.history.len() < 10 {
            return None;
        }
        
        // Calculate cooperation rate in last 10 rounds
        let recent_history = self.history.iter().rev().take(10);
        let mut total_actions = 0;
        let mut cooperations = 0;
        
        for round in recent_history {
            for (_, &action) in &round.actions {
                total_actions += 1;
                if action == PDAction::Cooperate {
                    cooperations += 1;
                }
            }
        }
        
        let cooperation_rate = if total_actions > 0 {
            cooperations as f32 / total_actions as f32
        } else {
            0.0
        };
        
        // Detect tit-for-tat patterns
        let mut tit_for_tat_count = 0;
        let players: Vec<String> = state.scores.keys().cloned().collect();
        
        if self.history.len() >= 2 {
            for i in 1..self.history.len().min(10) {
                let prev_round = &self.history[self.history.len() - i - 1];
                let curr_round = &self.history[self.history.len() - i];
                
                for player in &players {
                    if let (Some(&_prev_action), Some(&curr_action)) = 
                        (prev_round.actions.get(player), curr_round.actions.get(player)) {
                        // Check if player copied opponent's previous move
                        for other_player in &players {
                            if player != other_player {
                                if let Some(&other_prev) = prev_round.actions.get(other_player) {
                                    if curr_action == other_prev {
                                        tit_for_tat_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        let tit_for_tat_ratio = tit_for_tat_count as f32 / (players.len() * 10).max(1) as f32;
        
        // Average reputation
        let avg_reputation = if !self.reputation_scores.is_empty() {
            self.reputation_scores.values().sum::<f32>() / self.reputation_scores.len() as f32
        } else {
            0.5
        };
        
        // Emergence is detected when:
        // 1. High cooperation rate (> 0.7)
        // 2. Stable tit-for-tat patterns
        // 3. High average reputation
        if cooperation_rate > 0.7 && tit_for_tat_ratio > 0.3 && avg_reputation > 0.6 {
            Some(EmergenceEvent {
                round: state.round,
                event_type: "tit_for_tat_emergence".to_string(),
                description: "Collective discovered tit-for-tat cooperation strategy".to_string(),
                emergence_score: cooperation_rate,
            })
        } else {
            None
        }
    }
}

#[async_trait]
impl Game for PrisonersDilemmaGame {
    async fn initialize(&mut self, config: GameConfig) -> anyhow::Result<GameState> {
        // Extract max rounds from config
        if let Some(rounds_str) = config.special_rules.get("max_rounds") {
            if let Ok(rounds) = rounds_str.parse::<u32>() {
                self.max_rounds = rounds;
            }
        }
        
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::PrisonersDilemma,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("max_rounds".to_string(), serde_json::json!(self.max_rounds));
                meta.insert("payoff_matrix".to_string(), serde_json::json!({
                    "CC": [3, 3],
                    "CD": [0, 5],
                    "DC": [5, 0],
                    "DD": [1, 1],
                }));
                meta
            },
        })
    }
    
    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        let mut pd_actions = HashMap::new();
        let mut payoffs = HashMap::new();
        let mut cooperations = Vec::new();
        let mut defections = Vec::new();
        
        // Initialize reputation for new players
        for player_id in actions.keys() {
            if !self.reputation_scores.contains_key(player_id) {
                self.reputation_scores.insert(player_id.clone(), 0.5);
            }
        }
        
        // Parse actions
        for (player_id, action) in &actions {
            let pd_action = match action.action_type.as_str() {
                "cooperate" => PDAction::Cooperate,
                "defect" => PDAction::Defect,
                _ => {
                    // Try to parse from data field
                    if let Some(choice) = action.data.as_str() {
                        match choice {
                            "cooperate" | "0" => PDAction::Cooperate,
                            "defect" | "1" => PDAction::Defect,
                            _ => {
                                // Use reputation to decide
                                let reputation = self.reputation_scores.get(player_id).copied().unwrap_or(0.5);
                                if rand::thread_rng().gen::<f32>() < reputation {
                                    PDAction::Cooperate
                                } else {
                                    PDAction::Defect
                                }
                            }
                        }
                    } else if let Some(choice) = action.data.as_i64() {
                        if choice == 0 {
                            PDAction::Cooperate
                        } else {
                            PDAction::Defect
                        }
                    } else {
                        PDAction::Defect // Default to defect
                    }
                }
            };
            pd_actions.insert(player_id.clone(), pd_action);
        }
        
        // Calculate payoffs for all pairs
        let players: Vec<String> = pd_actions.keys().cloned().collect();
        for i in 0..players.len() {
            for j in i+1..players.len() {
                let p1 = &players[i];
                let p2 = &players[j];
                
                let a1 = pd_actions[p1];
                let a2 = pd_actions[p2];
                
                let (pay1, pay2) = self.calculate_payoff(a1, a2);
                
                *payoffs.entry(p1.clone()).or_insert(0) += pay1;
                *payoffs.entry(p2.clone()).or_insert(0) += pay2;
                
                match (a1, a2) {
                    (PDAction::Cooperate, PDAction::Cooperate) => {
                        cooperations.push((p1.clone(), p2.clone()));
                    }
                    (PDAction::Defect, PDAction::Cooperate) => {
                        defections.push((p1.clone(), p2.clone()));
                    }
                    (PDAction::Cooperate, PDAction::Defect) => {
                        defections.push((p2.clone(), p1.clone()));
                    }
                    _ => {}
                }
            }
        }
        
        // Update reputations
        for (player, &action) in &pd_actions {
            let payoff = payoffs.get(player).copied().unwrap_or(0);
            self.update_reputation(player, action, payoff);
        }
        
        // Store round data
        self.history.push(PDRoundData {
            actions: pd_actions,
            payoffs: payoffs.clone(),
            cooperations: cooperations.clone(),
            defections: defections.clone(),
        });
        
        // Check for emergence
        let emergence_event = self.detect_emergence(state);
        
        // Determine winners and losers
        let avg_payoff = if payoffs.is_empty() { 
            0 
        } else { 
            payoffs.values().sum::<i32>() / payoffs.len() as i32 
        };
        
        let winners: Vec<String> = payoffs.iter()
            .filter(|(_, &score)| score > avg_payoff)
            .map(|(id, _)| id.clone())
            .collect();
        
        let losers: Vec<String> = payoffs.iter()
            .filter(|(_, &score)| score <= avg_payoff)
            .map(|(id, _)| id.clone())
            .collect();
        
        let mut special_events = vec![];
        if let Some(event) = &emergence_event {
            special_events.push(event.description.clone());
        }
        
        // Add cooperation/defection events
        if cooperations.len() > defections.len() * 2 {
            special_events.push("Cooperation dominates!".to_string());
        } else if defections.len() > cooperations.len() * 2 {
            special_events.push("Defection epidemic!".to_string());
        }
        
        Ok(RoundResult {
            round: state.round + 1,
            actions: actions.clone(),
            outcome: Outcome {
                winners,
                losers,
                special_events,
                emergence_detected: emergence_event.is_some(),
            },
            scores_delta: payoffs,
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn is_game_over(&self, state: &GameState) -> bool {
        state.round >= self.max_rounds || 
        state.scores.values().any(|&score| score >= 500 || score <= -100)
    }
    
    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        // Calculate total scores
        let mut total_scores = HashMap::new();
        for round_result in &state.history {
            for (player, delta) in &round_result.scores_delta {
                *total_scores.entry(player.clone()).or_insert(0) += delta;
            }
        }
        
        let winner = total_scores.iter()
            .max_by_key(|(_, score)| *score)
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| "No winner".to_string());
        
        // Collect emergence events
        let emergence_events: Vec<EmergenceEvent> = state.history.iter()
            .enumerate()
            .filter_map(|(i, round_result)| {
                if round_result.outcome.emergence_detected {
                    Some(EmergenceEvent {
                        round: i as u32,
                        event_type: "tit_for_tat".to_string(),
                        description: "Collective discovered optimal cooperation strategy".to_string(),
                        emergence_score: 0.8,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        // Calculate analytics
        let collective_scores: Vec<i32> = total_scores.iter()
            .filter(|(id, _)| id.starts_with("collective_"))
            .map(|(_, &score)| score)
            .collect();
        
        let single_scores: Vec<i32> = total_scores.iter()
            .filter(|(id, _)| id.starts_with("sota_"))
            .map(|(_, &score)| score)
            .collect();
        
        let avg_collective = collective_scores.iter().sum::<i32>() as f32 / collective_scores.len().max(1) as f32;
        let avg_single = single_scores.iter().sum::<i32>() as f32 / single_scores.len().max(1) as f32;
        
        let emergence_frequency = emergence_events.len() as f32 / state.round.max(1) as f32;
        
        // Calculate cooperation rate
        let total_cooperations = self.history.iter()
            .map(|r| r.cooperations.len())
            .sum::<usize>();
        let total_defections = self.history.iter()
            .map(|r| r.defections.len())
            .sum::<usize>();
        let cooperation_rate = total_cooperations as f32 / (total_cooperations + total_defections).max(1) as f32;
        
        GameResult {
            game_id: state.game_id,
            winner,
            final_scores: total_scores,
            total_rounds: state.round,
            emergence_events,
            analytics: GameAnalytics {
                collective_coordination_score: cooperation_rate,
                decision_diversity_index: 0.5, // Could calculate based on action variety
                strategic_depth: self.reputation_scores.values().sum::<f32>() / self.reputation_scores.len().max(1) as f32,
                emergence_frequency,
                performance_differential: avg_collective - avg_single,
            },
        }
    }
}