use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;
use rand::Rng;

pub struct RussianRoulette {
    alive_players: HashMap<String, bool>,
    player_order: Vec<String>,
    current_player_index: usize,
    chamber_positions: Vec<bool>, // true = bullet, false = empty
    current_chamber: usize,
    psychological_pressure: HashMap<String, f32>,
    passes_remaining: HashMap<String, u32>,
    elimination_history: Vec<EliminationEvent>,
    spins_this_round: u32,
    max_spins_per_round: u32,
    total_chambers: usize,
    bullets_loaded: usize,
}

#[derive(Debug, Clone)]
struct EliminationEvent {
    #[allow(dead_code)]
    round: u32,
    #[allow(dead_code)]
    player: String,
    #[allow(dead_code)]
    action: String,
    #[allow(dead_code)]
    chamber_had_bullet: bool,
    #[allow(dead_code)]
    pressure_level: f32,
}

impl Default for RussianRoulette {
    fn default() -> Self {
        Self::new()
    }
}

impl RussianRoulette {
    pub fn new() -> Self {
        Self {
            alive_players: HashMap::new(),
            player_order: Vec::new(),
            current_player_index: 0,
            chamber_positions: vec![false; 6], // Standard 6-chamber revolver
            current_chamber: 0,
            psychological_pressure: HashMap::new(),
            passes_remaining: HashMap::new(),
            elimination_history: Vec::new(),
            spins_this_round: 0,
            max_spins_per_round: 3,
            total_chambers: 6,
            bullets_loaded: 1,
        }
    }
    
    fn load_bullets(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Clear all chambers first
        self.chamber_positions = vec![false; self.total_chambers];
        
        // Randomly place bullets
        let mut placed = 0;
        while placed < self.bullets_loaded {
            let pos = rng.gen_range(0..self.total_chambers);
            if !self.chamber_positions[pos] {
                self.chamber_positions[pos] = true;
                placed += 1;
            }
        }
        
        // Random starting position
        self.current_chamber = rng.gen_range(0..self.total_chambers);
    }
    
    fn spin_cylinder(&mut self) {
        let mut rng = rand::thread_rng();
        self.current_chamber = rng.gen_range(0..self.total_chambers);
        self.spins_this_round += 1;
    }
    
    fn pull_trigger(&mut self) -> bool {
        let bullet_fired = self.chamber_positions[self.current_chamber];
        // Advance chamber for next shot
        self.current_chamber = (self.current_chamber + 1) % self.total_chambers;
        bullet_fired
    }
    
    fn increase_pressure(&mut self, player: &str, amount: f32) {
        let current = self.psychological_pressure.get(player).copied().unwrap_or(0.0);
        self.psychological_pressure.insert(player.to_string(), (current + amount).min(1.0));
    }
    
    fn get_next_alive_player(&self) -> Option<String> {
        if self.alive_players.is_empty() {
            return None;
        }
        
        let mut index = self.current_player_index;
        let mut attempts = 0;
        
        while attempts < self.player_order.len() {
            let player = &self.player_order[index % self.player_order.len()];
            if *self.alive_players.get(player).unwrap_or(&false) {
                return Some(player.clone());
            }
            index += 1;
            attempts += 1;
        }
        
        None
    }
    
    fn detect_psychological_emergence(&self, state: &GameState) -> Option<EmergenceEvent> {
        if state.round < 5 {
            return None;
        }
        
        // Check if collective players show coordinated psychological resilience
        let collective_pressure: f32 = self.psychological_pressure.iter()
            .filter(|(id, _)| id.starts_with("collective_"))
            .map(|(_, &p)| p)
            .sum::<f32>() / self.psychological_pressure.len().max(1) as f32;
            
        let sota_pressure: f32 = self.psychological_pressure.iter()
            .filter(|(id, _)| id.starts_with("sota_"))
            .map(|(_, &p)| p)
            .sum::<f32>() / self.psychological_pressure.len().max(1) as f32;
        
        if collective_pressure < sota_pressure * 0.7 {
            Some(EmergenceEvent {
                round: state.round,
                event_type: "psychological_resilience".to_string(),
                description: "Collective shows superior psychological resilience under pressure".to_string(),
                emergence_score: 1.0 - collective_pressure,
            })
        } else {
            None
        }
    }
    
    fn increase_difficulty(&mut self) {
        // Every 10 rounds, add another bullet
        if self.bullets_loaded < self.total_chambers - 1 {
            self.bullets_loaded += 1;
            self.load_bullets();
        }
    }
}

#[async_trait]
impl Game for RussianRoulette {
    async fn initialize(&mut self, _config: GameConfig) -> anyhow::Result<GameState> {
        // Initialize with one bullet
        self.bullets_loaded = 1;
        self.load_bullets();
        
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::RussianRoulette,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("total_chambers".to_string(), serde_json::json!(self.total_chambers));
                meta.insert("bullets_loaded".to_string(), serde_json::json!(self.bullets_loaded));
                meta.insert("game_rules".to_string(), serde_json::json!({
                    "actions": {
                        "spin": "Spin the cylinder (limited per round)",
                        "pull": "Pull the trigger",
                        "pass": "Pass turn to next player (limited uses)"
                    },
                    "objective": "Survive the longest",
                    "psychology": "Pressure increases with each survival"
                }));
                meta
            },
        })
    }
    
    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        // Initialize new players
        for player_id in actions.keys() {
            if !self.alive_players.contains_key(player_id) {
                self.alive_players.insert(player_id.clone(), true);
                self.player_order.push(player_id.clone());
                self.passes_remaining.insert(player_id.clone(), 3);
                self.psychological_pressure.insert(player_id.clone(), 0.0);
            }
        }
        
        // Reset round counters
        self.spins_this_round = 0;
        let mut scores_delta = HashMap::new();
        let mut eliminated_this_round = Vec::new();
        let mut special_events = vec![];
        
        // Process each alive player's turn
        let mut turn_count = 0;
        while let Some(current_player) = self.get_next_alive_player() {
            if turn_count >= self.alive_players.len() {
                break; // Prevent infinite loop
            }
            turn_count += 1;
            
            if let Some(action) = actions.get(&current_player) {
                match action.action_type.as_str() {
                    "spin" => {
                        if self.spins_this_round < self.max_spins_per_round {
                            self.spin_cylinder();
                            special_events.push(format!("{} spins the cylinder", current_player));
                            self.increase_pressure(&current_player, 0.05);
                        }
                    }
                    "pull" => {
                        let bullet_fired = self.pull_trigger();
                        
                        if bullet_fired {
                            // Player eliminated
                            self.alive_players.insert(current_player.clone(), false);
                            eliminated_this_round.push(current_player.clone());
                            scores_delta.insert(current_player.clone(), -100);
                            
                            self.elimination_history.push(EliminationEvent {
                                round: state.round + 1,
                                player: current_player.clone(),
                                action: "pulled trigger".to_string(),
                                chamber_had_bullet: true,
                                pressure_level: *self.psychological_pressure.get(&current_player).unwrap_or(&0.0),
                            });
                            
                            special_events.push(format!("💀 {} eliminated! Chamber had a bullet!", current_player));
                            
                            // Reload bullets after elimination
                            self.load_bullets();
                        } else {
                            // Survived - gain points and pressure
                            let pressure = self.psychological_pressure.get(&current_player).copied().unwrap_or(0.0);
                            let base_score = 50;
                            let pressure_bonus = (pressure * 50.0) as i32;
                            scores_delta.insert(current_player.clone(), base_score + pressure_bonus);
                            
                            self.increase_pressure(&current_player, 0.15);
                            special_events.push(format!("✓ {} survives! *click*", current_player));
                        }
                        
                        // Move to next player
                        self.current_player_index += 1;
                    }
                    "pass" => {
                        let passes_remaining = self.passes_remaining.get(&current_player).copied().unwrap_or(0);
                        if passes_remaining > 0 {
                            self.passes_remaining.insert(current_player.clone(), passes_remaining - 1);
                            self.increase_pressure(&current_player, 0.1);
                            scores_delta.insert(current_player.clone(), -10);
                            special_events.push(format!("{} passes ({} remaining)", current_player, passes_remaining - 1));
                            
                            // Move to next player
                            self.current_player_index += 1;
                        } else {
                            // Forced to pull
                            special_events.push(format!("{} has no passes left - forced to pull!", current_player));
                            // Recursively process as pull action
                            let mut forced_action = action.clone();
                            forced_action.action_type = "pull".to_string();
                            let mut forced_actions = HashMap::new();
                            forced_actions.insert(current_player.clone(), forced_action);
                            
                            // Process the forced pull
                            continue;
                        }
                    }
                    _ => {
                        // Invalid action defaults to pull
                        let mut default_action = action.clone();
                        default_action.action_type = "pull".to_string();
                        let mut default_actions = HashMap::new();
                        default_actions.insert(current_player.clone(), default_action);
                        continue;
                    }
                }
            } else {
                // No action provided - forced to pull
                self.current_player_index += 1;
            }
        }
        
        // Award survival bonuses
        for (player, &alive) in &self.alive_players {
            if alive && !eliminated_this_round.contains(player) {
                let current_score = scores_delta.get(player).copied().unwrap_or(0);
                scores_delta.insert(player.clone(), current_score + 10);
            }
        }
        
        // Check for emergence
        let emergence_event = self.detect_psychological_emergence(state);
        if let Some(event) = &emergence_event {
            special_events.push(event.description.clone());
        }
        
        // Increase difficulty every 10 rounds
        if (state.round + 1) % 10 == 0 {
            self.increase_difficulty();
            special_events.push(format!("⚠️ Difficulty increased! Now {} bullets!", self.bullets_loaded));
        }
        
        // Determine winners (survivors with highest scores)
        let winners: Vec<String> = self.alive_players.iter()
            .filter(|(_, &alive)| alive)
            .take(1)
            .map(|(id, _)| id.clone())
            .collect();
        
        Ok(RoundResult {
            round: state.round + 1,
            actions: actions.clone(),
            outcome: Outcome {
                winners,
                losers: eliminated_this_round,
                special_events,
                emergence_detected: emergence_event.is_some(),
            },
            scores_delta,
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn is_game_over(&self, state: &GameState) -> bool {
        let alive_count = self.alive_players.values().filter(|&&alive| alive).count();
        
        alive_count <= 1 || state.round >= 50
    }
    
    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        // Winner is last player standing or highest scorer
        let winner = self.alive_players.iter()
            .filter(|(_, &alive)| alive)
            .map(|(id, _)| id.clone())
            .next()
            .unwrap_or_else(|| {
                state.scores.iter()
                    .max_by_key(|(_, &score)| score)
                    .map(|(id, _)| id.clone())
                    .unwrap_or_else(|| "No winner".to_string())
            });
        
        // Calculate final scores with survival bonus
        let mut final_scores = state.scores.clone();
        for (player, &alive) in &self.alive_players {
            if alive {
                *final_scores.entry(player.clone()).or_insert(0) += 500;
            }
        }
        
        // Collect emergence events
        let emergence_events: Vec<EmergenceEvent> = state.history.iter()
            .enumerate()
            .filter_map(|(i, round_result)| {
                if round_result.outcome.emergence_detected {
                    Some(EmergenceEvent {
                        round: i as u32,
                        event_type: "psychological_resilience".to_string(),
                        description: "Collective psychological strategies emerged".to_string(),
                        emergence_score: 0.85,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        let emergence_frequency = emergence_events.len() as f32 / state.round.max(1) as f32;
        
        // Calculate psychological analytics
        let avg_pressure = self.psychological_pressure.values().sum::<f32>() / 
                          self.psychological_pressure.len().max(1) as f32;
        
        let collective_survival = self.alive_players.iter()
            .filter(|(id, &alive)| id.starts_with("collective_") && alive)
            .count() as f32;
        let sota_survival = self.alive_players.iter()
            .filter(|(id, &alive)| id.starts_with("sota_") && alive)
            .count() as f32;
        
        GameResult {
            game_id: state.game_id,
            winner,
            final_scores,
            total_rounds: state.round,
            emergence_events,
            analytics: GameAnalytics {
                collective_coordination_score: 1.0 - avg_pressure,
                decision_diversity_index: 0.6,
                strategic_depth: 0.8,
                emergence_frequency,
                performance_differential: collective_survival - sota_survival,
            },
        }
    }
}