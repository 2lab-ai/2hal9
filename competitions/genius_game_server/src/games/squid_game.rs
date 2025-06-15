use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;
use rand::Rng;

pub struct SquidGame {
    alive_players: HashMap<String, bool>,
    player_positions: HashMap<String, f32>,
    player_speeds: HashMap<String, f32>,
    is_green_light: bool,
    light_duration: u32,
    finish_line: f32,
    elimination_history: Vec<EliminationEvent>,
    winners: Vec<String>,
    light_change_timer: u32,
}

#[derive(Debug, Clone)]
struct EliminationEvent {
    round: u32,
    player: String,
    reason: String,
    position: f32,
}

impl Default for SquidGame {
    fn default() -> Self {
        Self::new()
    }
}

impl SquidGame {
    pub fn new() -> Self {
        Self {
            alive_players: HashMap::new(),
            player_positions: HashMap::new(),
            player_speeds: HashMap::new(),
            is_green_light: false,
            light_duration: 0,
            finish_line: 100.0,
            elimination_history: Vec::new(),
            winners: Vec::new(),
            light_change_timer: 0,
        }
    }
    
    fn check_movement_during_red_light(&mut self, round: u32) {
        if !self.is_green_light {
            let mut eliminated = Vec::new();
            
            for (player, &speed) in &self.player_speeds {
                if speed > 0.01 && *self.alive_players.get(player).unwrap_or(&false) {
                    eliminated.push(player.clone());
                    
                    self.elimination_history.push(EliminationEvent {
                        round,
                        player: player.clone(),
                        reason: "Moved during red light".to_string(),
                        position: *self.player_positions.get(player).unwrap_or(&0.0),
                    });
                }
            }
            
            for player in eliminated {
                self.alive_players.insert(player, false);
            }
        }
    }
    
    fn update_positions(&mut self) {
        if self.is_green_light {
            for (player, &speed) in &self.player_speeds {
                if *self.alive_players.get(player).unwrap_or(&false) {
                    let current_pos = self.player_positions.get(player).copied().unwrap_or(0.0);
                    let new_pos = (current_pos + speed).min(self.finish_line);
                    self.player_positions.insert(player.clone(), new_pos);
                    
                    // Check if player reached finish line
                    if new_pos >= self.finish_line && !self.winners.contains(player) {
                        self.winners.push(player.clone());
                    }
                }
            }
        }
    }
    
    fn toggle_light(&mut self) {
        self.is_green_light = !self.is_green_light;
        let mut rng = rand::thread_rng();
        
        // Random duration between 2-8 seconds (simulated as rounds)
        self.light_duration = rng.gen_range(2..=8);
        self.light_change_timer = self.light_duration;
    }
    
    fn detect_survival_emergence(&self, state: &GameState) -> Option<EmergenceEvent> {
        if self.elimination_history.len() < 5 {
            return None;
        }
        
        // Check if collective players have better survival rate
        let total_players = self.alive_players.len() as f32;
        let alive_collective = self.alive_players.iter()
            .filter(|(id, &alive)| id.starts_with("collective_") && alive)
            .count() as f32;
        let alive_sota = self.alive_players.iter()
            .filter(|(id, &alive)| id.starts_with("sota_") && alive)
            .count() as f32;
        
        let collective_survival_rate = if total_players > 0.0 {
            alive_collective / total_players
        } else {
            0.0
        };
        
        if collective_survival_rate > 0.7 && alive_collective > alive_sota {
            Some(EmergenceEvent {
                round: state.round,
                event_type: "survival_strategy".to_string(),
                description: "Collective developed superior survival strategies".to_string(),
                emergence_score: collective_survival_rate,
            })
        } else {
            None
        }
    }
}

#[async_trait]
impl Game for SquidGame {
    async fn initialize(&mut self, _config: GameConfig) -> anyhow::Result<GameState> {
        // Start with green light
        self.is_green_light = true;
        self.light_duration = 5;
        self.light_change_timer = 5;
        
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::SquidGame,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("finish_line".to_string(), serde_json::json!(self.finish_line));
                meta.insert("game_rules".to_string(), serde_json::json!({
                    "green_light": "Players can move safely",
                    "red_light": "Any movement results in elimination",
                    "objective": "Reach the finish line without being eliminated"
                }));
                meta.insert("light_status".to_string(), serde_json::json!("green"));
                meta
            },
        })
    }
    
    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        // Initialize new players
        for player_id in actions.keys() {
            if !self.alive_players.contains_key(player_id) {
                self.alive_players.insert(player_id.clone(), true);
                self.player_positions.insert(player_id.clone(), 0.0);
                self.player_speeds.insert(player_id.clone(), 0.0);
            }
        }
        
        // Update light timer
        self.light_change_timer = self.light_change_timer.saturating_sub(1);
        if self.light_change_timer == 0 {
            self.toggle_light();
        }
        
        // Process player actions
        for (player_id, action) in &actions {
            if !*self.alive_players.get(player_id).unwrap_or(&false) {
                continue; // Skip dead players
            }
            
            let speed = match action.action_type.as_str() {
                "stop" => 0.0,
                "move_slow" => 2.0,
                "move_normal" => 5.0,
                "move_fast" => 10.0,
                "move_risky" => 15.0, // Very fast but dangerous during red light
                _ => 0.0,
            };
            
            self.player_speeds.insert(player_id.clone(), speed);
        }
        
        // Check for red light violations
        self.check_movement_during_red_light(state.round + 1);
        
        // Update positions
        self.update_positions();
        
        // Calculate scores (distance traveled)
        let mut scores_delta = HashMap::new();
        for (player_id, &position) in &self.player_positions {
            if *self.alive_players.get(player_id).unwrap_or(&false) {
                scores_delta.insert(player_id.clone(), position as i32);
            }
        }
        
        // Determine round winners and losers
        let winners = self.winners.clone();
        let losers: Vec<String> = self.elimination_history.iter()
            .filter(|e| e.round == state.round + 1)
            .map(|e| e.player.clone())
            .collect();
        
        // Check for emergence
        let emergence_event = self.detect_survival_emergence(state);
        
        let mut special_events = vec![];
        if self.is_green_light {
            special_events.push("GREEN LIGHT - Move safely!".to_string());
        } else {
            special_events.push("RED LIGHT - FREEZE!".to_string());
        }
        
        if !losers.is_empty() {
            special_events.push(format!("{} players eliminated!", losers.len()));
        }
        
        if !winners.is_empty() {
            special_events.push(format!("{} reached the finish line!", winners.len()));
        }
        
        if let Some(event) = &emergence_event {
            special_events.push(event.description.clone());
        }
        
        Ok(RoundResult {
            round: state.round + 1,
            actions: actions.clone(),
            outcome: Outcome {
                winners: winners.clone(),
                losers,
                special_events,
                emergence_detected: emergence_event.is_some(),
            },
            scores_delta,
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn is_game_over(&self, state: &GameState) -> bool {
        let alive_count = self.alive_players.values().filter(|&&alive| alive).count();
        
        alive_count <= 1 || 
        state.round >= 100 ||
        self.winners.len() >= self.alive_players.len() / 2
    }
    
    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        // Winner is the player who got furthest or reached finish line
        let winner = if !self.winners.is_empty() {
            self.winners[0].clone()
        } else {
            self.player_positions.iter()
                .filter(|(id, _)| *self.alive_players.get(*id).unwrap_or(&false))
                .max_by(|(_, pos1), (_, pos2)| pos1.partial_cmp(pos2).unwrap())
                .map(|(id, _)| id.clone())
                .unwrap_or_else(|| "No winner".to_string())
        };
        
        // Final scores based on distance
        let final_scores: HashMap<String, i32> = self.player_positions.iter()
            .map(|(id, &pos)| {
                let score = if self.winners.contains(id) {
                    1000 + (100.0 - pos) as i32 // Bonus for finishing
                } else if *self.alive_players.get(id).unwrap_or(&false) {
                    pos as i32
                } else {
                    -(100.0 - pos) as i32 // Negative score for eliminated
                };
                (id.clone(), score)
            })
            .collect();
        
        // Collect emergence events
        let emergence_events: Vec<EmergenceEvent> = state.history.iter()
            .enumerate()
            .filter_map(|(i, round_result)| {
                if round_result.outcome.emergence_detected {
                    Some(EmergenceEvent {
                        round: i as u32,
                        event_type: "survival_strategy".to_string(),
                        description: "Collective survival strategies emerged".to_string(),
                        emergence_score: 0.8,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        let emergence_frequency = emergence_events.len() as f32 / state.round.max(1) as f32;
        
        // Calculate survival analytics
        let total_players = self.alive_players.len() as f32;
        let survivors = self.alive_players.values().filter(|&&alive| alive).count() as f32;
        let survival_rate = survivors / total_players.max(1.0);
        
        let collective_survivors = self.alive_players.iter()
            .filter(|(id, &alive)| id.starts_with("collective_") && alive)
            .count() as f32;
        let sota_survivors = self.alive_players.iter()
            .filter(|(id, &alive)| id.starts_with("sota_") && alive)
            .count() as f32;
        
        GameResult {
            game_id: state.game_id,
            winner,
            final_scores,
            total_rounds: state.round,
            emergence_events,
            analytics: GameAnalytics {
                collective_coordination_score: survival_rate,
                decision_diversity_index: 0.5, // Could analyze movement patterns
                strategic_depth: 0.7, // Based on timing decisions
                emergence_frequency,
                performance_differential: collective_survivors - sota_survivors,
            },
        }
    }
}