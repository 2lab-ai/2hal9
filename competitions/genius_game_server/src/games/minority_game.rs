use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

pub struct MinorityGame {
    history_window: usize,
}

impl Default for MinorityGame {
    fn default() -> Self {
        Self {
            history_window: 10,
        }
    }
}

impl MinorityGame {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn calculate_minority(&self, choices: &HashMap<String, i32>) -> (i32, Vec<String>) {
        let zeros = choices.values().filter(|&&v| v == 0).count();
        let ones = choices.values().filter(|&&v| v == 1).count();
        
        match zeros.cmp(&ones) {
            std::cmp::Ordering::Less => {
                let winners = choices.iter()
                    .filter(|(_, &v)| v == 0)
                    .map(|(k, _)| k.clone())
                    .collect();
                (0, winners)
            },
            std::cmp::Ordering::Greater => {
                let winners = choices.iter()
                    .filter(|(_, &v)| v == 1)
                    .map(|(k, _)| k.clone())
                    .collect();
                (1, winners)
            },
            std::cmp::Ordering::Equal => {
                (-1, vec![]) // Tie
            }
        }
    }
    
    pub fn detect_emergence(&self, state: &GameState, actions: &HashMap<String, Action>) -> Option<EmergenceEvent> {
        // Check for collective pattern breaking
        if state.round > 20 {
            let collective_actions: Vec<_> = actions.iter()
                .filter(|(id, _)| id.starts_with("collective_"))
                .collect();
            
            if collective_actions.len() > 5 {
                // Check if collective made diverse choices
                let collective_choices: Vec<i32> = collective_actions.iter()
                    .filter_map(|(_, action)| action.data.as_i64().map(|v| v as i32))
                    .collect();
                
                let zeros = collective_choices.iter().filter(|&&v| v == 0).count();
                let ones = collective_choices.iter().filter(|&&v| v == 1).count();
                
                // Perfect distribution indicates emergence
                if zeros > 0 && ones > 0 && (zeros as f32 - ones as f32).abs() < 2.0 {
                    return Some(EmergenceEvent {
                        round: state.round,
                        event_type: "perfect_distribution".to_string(),
                        description: "Collective achieved near-perfect minority distribution".to_string(),
                        emergence_score: 0.9,
                    });
                }
            }
        }
        None
    }
}

#[async_trait]
impl Game for MinorityGame {
    async fn initialize(&mut self, _config: GameConfig) -> anyhow::Result<GameState> {
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::MinorityGame,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("history_window".to_string(), serde_json::json!(self.history_window));
                meta
            },
        })
    }
    
    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        // Extract choices
        let mut choices = HashMap::new();
        for (player_id, action) in &actions {
            if let Some(choice) = action.data.as_i64() {
                choices.insert(player_id.clone(), choice as i32);
            }
        }
        
        // Determine minority
        let (winning_choice, winners) = self.calculate_minority(&choices);
        
        // Calculate score changes
        let mut scores_delta = HashMap::new();
        for player_id in choices.keys() {
            if winners.contains(player_id) {
                scores_delta.insert(player_id.clone(), 10);
            } else if winning_choice != -1 {
                scores_delta.insert(player_id.clone(), -5);
            } else {
                scores_delta.insert(player_id.clone(), 0);
            }
        }
        
        // Check for emergence
        let emergence_event = self.detect_emergence(state, &actions);
        
        let losers: Vec<String> = choices.keys()
            .filter(|k| !winners.contains(k))
            .cloned()
            .collect();
        
        Ok(RoundResult {
            round: state.round + 1,
            actions: actions.clone(),
            outcome: Outcome {
                winners: winners.clone(),
                losers,
                special_events: if emergence_event.is_some() {
                    vec!["Emergence detected!".to_string()]
                } else {
                    vec![]
                },
                emergence_detected: emergence_event.is_some(),
            },
            scores_delta,
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn is_game_over(&self, state: &GameState) -> bool {
        state.round >= 100 || 
        state.scores.values().any(|&score| score >= 500 || score <= -200)
    }
    
    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        let winner = state.scores.iter()
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
                        event_type: "minority_distribution".to_string(),
                        description: "Collective achieved optimal distribution".to_string(),
                        emergence_score: 0.8,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        // Calculate analytics
        let collective_scores: Vec<i32> = state.scores.iter()
            .filter(|(id, _)| id.starts_with("collective_"))
            .map(|(_, &score)| score)
            .collect();
        
        let single_scores: Vec<i32> = state.scores.iter()
            .filter(|(id, _)| id.starts_with("sota_"))
            .map(|(_, &score)| score)
            .collect();
        
        let avg_collective = collective_scores.iter().sum::<i32>() as f32 / collective_scores.len().max(1) as f32;
        let avg_single = single_scores.iter().sum::<i32>() as f32 / single_scores.len().max(1) as f32;
        
        let emergence_frequency = emergence_events.len() as f32 / state.round.max(1) as f32;
        
        GameResult {
            game_id: state.game_id,
            winner,
            final_scores: state.scores.clone(),
            total_rounds: state.round,
            emergence_events,
            analytics: GameAnalytics {
                collective_coordination_score: 0.0, // TODO: Calculate from history
                decision_diversity_index: 0.0, // TODO: Calculate diversity
                strategic_depth: state.round as f32 / 100.0,
                emergence_frequency,
                performance_differential: avg_collective - avg_single,
            },
        }
    }
}