use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use super::{Game, GameState, PlayerAction, GameResult, EmergenceMetrics};
use anyhow::Result;

/// Squid Game - Red Light Green Light survival game
pub struct SquidGame {
    id: Uuid,
    round: u32,
    max_rounds: u32,
    players: Vec<String>,
    alive_players: HashMap<String, bool>,
    player_positions: HashMap<String, f32>,
    player_speeds: HashMap<String, f32>,
    is_green_light: bool,
    light_duration: u32,
    finish_line: f32,
    elimination_history: Vec<EliminationEvent>,
    winners: Vec<String>,
    special_rules: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EliminationEvent {
    round: u32,
    player: String,
    reason: String,
    position: f32,
}

impl SquidGame {
    pub fn new(max_rounds: u32, special_rules: HashMap<String, String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            round: 0,
            max_rounds,
            players: Vec::new(),
            alive_players: HashMap::new(),
            player_positions: HashMap::new(),
            player_speeds: HashMap::new(),
            is_green_light: false,
            light_duration: 0,
            finish_line: 100.0,
            elimination_history: Vec::new(),
            winners: Vec::new(),
            special_rules,
        }
    }
    
    fn check_movement_during_red_light(&mut self) {
        if !self.is_green_light {
            let mut eliminated = Vec::new();
            
            for (player, &speed) in &self.player_speeds {
                if speed > 0.01 && *self.alive_players.get(player).unwrap_or(&false) {
                    eliminated.push(player.clone());
                    
                    self.elimination_history.push(EliminationEvent {
                        round: self.round,
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
}

#[async_trait]
impl Game for SquidGame {
    async fn get_state(&self) -> Result<GameState> {
        let available_actions = if self.is_green_light {
            vec!["move_slow".to_string(), "move_normal".to_string(), "move_fast".to_string(), "stop".to_string()]
        } else {
            vec!["stop".to_string(), "move_risky".to_string()]
        };
        
        let mut player_states = HashMap::new();
        for player in &self.players {
            let alive = *self.alive_players.get(player).unwrap_or(&false);
            let position = *self.player_positions.get(player).unwrap_or(&0.0);
            
            player_states.insert(player.clone(), serde_json::json!({
                "alive": alive,
                "position": position,
                "distance_to_finish": self.finish_line - position,
                "current_speed": self.player_speeds.get(player).copied().unwrap_or(0.0),
            }));
        }
        
        let scores: HashMap<String, i32> = self.players.iter()
            .map(|p| {
                let position = self.player_positions.get(p).copied().unwrap_or(0.0);
                let score = if self.winners.contains(p) {
                    1000
                } else if !self.alive_players.get(p).copied().unwrap_or(false) {
                    0
                } else {
                    (position * 10.0) as i32
                };
                (p.clone(), score)
            })
            .collect();
        
        Ok(GameState {
            game_id: self.id,
            game_type: "squid_game".to_string(),
            round: self.round,
            players: self.players.clone(),
            current_state: serde_json::json!({
                "light_status": if self.is_green_light { "GREEN" } else { "RED" },
                "light_duration": self.light_duration,
                "alive_count": self.alive_players.values().filter(|&&v| v).count(),
                "eliminated_count": self.elimination_history.len(),
                "available_actions": available_actions,
            }),
            available_actions,
            scores,
            player_states,
            is_complete: self.round >= self.max_rounds || 
                        self.alive_players.values().filter(|&&v| v).count() == 0 ||
                        !self.winners.is_empty(),
            special_data: Some(serde_json::json!({
                "finish_line": self.finish_line,
                "winners": self.winners,
                "last_elimination": self.elimination_history.last(),
            })),
        })
    }
    
    async fn process_action(&mut self, action: PlayerAction) -> Result<()> {
        if !self.alive_players.get(&action.player_id).copied().unwrap_or(false) {
            return Ok(());
        }
        
        let speed = match action.action_type.as_str() {
            "stop" => 0.0,
            "move_slow" => 0.5,
            "move_normal" => 1.5,
            "move_fast" => 3.0,
            "move_risky" => 0.3, // Small movement during red light
            _ => 0.0,
        };
        
        self.player_speeds.insert(action.player_id.clone(), speed);
        
        // Update position if green light
        if self.is_green_light && speed > 0.0 {
            let current_pos = self.player_positions.get(&action.player_id).copied().unwrap_or(0.0);
            let new_pos = (current_pos + speed).min(self.finish_line);
            self.player_positions.insert(action.player_id.clone(), new_pos);
            
            // Check if reached finish line
            if new_pos >= self.finish_line {
                self.winners.push(action.player_id.clone());
            }
        }
        
        Ok(())
    }
    
    async fn advance_round(&mut self) -> Result<GameResult> {
        self.round += 1;
        
        // Toggle light every few rounds
        if self.round % 3 == 0 {
            self.is_green_light = !self.is_green_light;
            self.light_duration = if self.is_green_light {
                rand::random::<u32>() % 5 + 3
            } else {
                rand::random::<u32>() % 4 + 2
            };
        }
        
        // Check for illegal movements
        self.check_movement_during_red_light();
        
        // Simulate AI actions
        for player in self.players.clone() {
            if self.alive_players.get(&player).copied().unwrap_or(false) && 
               !self.winners.contains(&player) {
                let position = self.player_positions.get(&player).copied().unwrap_or(0.0);
                let distance = self.finish_line - position;
                
                // Simple AI strategy
                let action_type = if self.is_green_light {
                    if distance > 50.0 {
                        "move_fast"
                    } else if distance > 20.0 {
                        "move_normal"
                    } else {
                        "move_slow"
                    }
                } else {
                    if rand::random::<f32>() > 0.9 && distance < 10.0 {
                        "move_risky"
                    } else {
                        "stop"
                    }
                };
                
                let action = PlayerAction {
                    player_id: player,
                    action_type: action_type.to_string(),
                    data: None,
                };
                
                self.process_action(action).await?;
            }
        }
        
        let round_winners = self.winners.clone();
        
        let scores: HashMap<String, i32> = self.players.iter()
            .map(|p| {
                let position = self.player_positions.get(p).copied().unwrap_or(0.0);
                let score = if self.winners.contains(p) {
                    1000
                } else if !self.alive_players.get(p).copied().unwrap_or(false) {
                    0
                } else {
                    (position * 10.0) as i32
                };
                (p.clone(), score)
            })
            .collect();
        
        let is_final = self.round >= self.max_rounds || 
                      self.alive_players.values().filter(|&&v| v).count() == 0 ||
                      self.winners.len() >= self.players.len() / 2;
        
        Ok(GameResult {
            round: self.round,
            scores,
            round_winners,
            is_final_round: is_final,
            emergence_metrics: EmergenceMetrics {
                emergence_detected: false,
                coordination_score: 0.0,
                collective_intelligence_index: 0.0,
                decision_diversity_index: 0.0,
                strategic_depth: 0.0,
                special_patterns: HashMap::new(),
            },
            special_data: Some(serde_json::json!({
                "light_status": if self.is_green_light { "GREEN" } else { "RED" },
                "eliminations_this_round": self.elimination_history.iter()
                    .filter(|e| e.round == self.round)
                    .count(),
            })),
        })
    }
    
    fn add_player(&mut self, player_id: String) -> Result<()> {
        if !self.players.contains(&player_id) {
            self.players.push(player_id.clone());
            self.alive_players.insert(player_id.clone(), true);
            self.player_positions.insert(player_id.clone(), 0.0);
            self.player_speeds.insert(player_id, 0.0);
        }
        Ok(())
    }
    
    fn get_game_id(&self) -> Uuid {
        self.id
    }
}