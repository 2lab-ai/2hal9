use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;
use rand::Rng;

pub struct KingOfTheHill {
    player_positions: HashMap<String, Position>,
    hill_controller: Option<String>,
    control_duration: HashMap<String, u32>,
    push_cooldowns: HashMap<String, u32>,
    player_strength: HashMap<String, f32>,
    player_health: HashMap<String, i32>,
    hill_radius: f32,
    hill_center: Position,
    arena_size: f32,
    elimination_history: Vec<EliminationEvent>,
    alliance_network: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
}

impl Position {
    fn distance_to(&self, other: &Position) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
    
    fn move_towards(&self, target: &Position, distance: f32) -> Position {
        let dx = target.x - self.x;
        let dy = target.y - self.y;
        let total_dist = (dx * dx + dy * dy).sqrt();
        
        if total_dist <= distance {
            *target
        } else {
            Position {
                x: self.x + (dx / total_dist) * distance,
                y: self.y + (dy / total_dist) * distance,
            }
        }
    }
    
    fn push_away_from(&self, pusher: &Position, force: f32) -> Position {
        let dx = self.x - pusher.x;
        let dy = self.y - pusher.y;
        let dist = (dx * dx + dy * dy).sqrt().max(0.1);
        
        Position {
            x: self.x + (dx / dist) * force,
            y: self.y + (dy / dist) * force,
        }
    }
}

#[derive(Debug, Clone)]
struct EliminationEvent {
    round: u32,
    player: String,
    reason: String,
    final_position: Position,
}

impl Default for KingOfTheHill {
    fn default() -> Self {
        Self::new()
    }
}

impl KingOfTheHill {
    pub fn new() -> Self {
        let arena_size = 100.0;
        let hill_center = Position { x: arena_size / 2.0, y: arena_size / 2.0 };
        
        Self {
            player_positions: HashMap::new(),
            hill_controller: None,
            control_duration: HashMap::new(),
            push_cooldowns: HashMap::new(),
            player_strength: HashMap::new(),
            player_health: HashMap::new(),
            hill_radius: 10.0,
            hill_center,
            arena_size,
            elimination_history: Vec::new(),
            alliance_network: HashMap::new(),
        }
    }
    
    fn spawn_player(&mut self, player_id: &str) {
        let mut rng = rand::thread_rng();
        
        // Spawn at random edge position
        let side = rng.gen_range(0..4);
        let position = match side {
            0 => Position { x: rng.gen_range(0.0..self.arena_size), y: 0.0 },
            1 => Position { x: self.arena_size, y: rng.gen_range(0.0..self.arena_size) },
            2 => Position { x: rng.gen_range(0.0..self.arena_size), y: self.arena_size },
            _ => Position { x: 0.0, y: rng.gen_range(0.0..self.arena_size) },
        };
        
        self.player_positions.insert(player_id.to_string(), position);
        self.control_duration.insert(player_id.to_string(), 0);
        self.push_cooldowns.insert(player_id.to_string(), 0);
        self.player_strength.insert(player_id.to_string(), 1.0);
        self.player_health.insert(player_id.to_string(), 100);
    }
    
    fn is_on_hill(&self, position: &Position) -> bool {
        position.distance_to(&self.hill_center) <= self.hill_radius
    }
    
    fn update_hill_control(&mut self) {
        let mut players_on_hill = Vec::new();
        
        for (player_id, position) in &self.player_positions {
            if self.is_on_hill(position) && self.is_player_alive(player_id) {
                players_on_hill.push(player_id.clone());
            }
        }
        
        // Update controller
        if players_on_hill.len() == 1 {
            let controller = players_on_hill[0].clone();
            self.hill_controller = Some(controller.clone());
            
            // Increment control duration
            *self.control_duration.entry(controller).or_insert(0) += 1;
        } else if players_on_hill.is_empty() {
            self.hill_controller = None;
        } else {
            // Multiple players on hill - contested
            self.hill_controller = None;
        }
    }
    
    fn is_player_alive(&self, player_id: &str) -> bool {
        self.player_health.get(player_id).copied().unwrap_or(0) > 0
    }
    
    fn process_combat(&mut self, round: u32) {
        let mut damage_dealt = HashMap::new();
        
        // Check for players in combat range
        let positions = self.player_positions.clone();
        for (player1, pos1) in &positions {
            if !self.is_player_alive(player1) {
                continue;
            }
            
            for (player2, pos2) in &positions {
                if player1 == player2 || !self.is_player_alive(player2) {
                    continue;
                }
                
                let distance = pos1.distance_to(pos2);
                if distance < 5.0 {
                    // Close combat
                    let strength1 = self.player_strength.get(player1).copied().unwrap_or(1.0);
                    let damage = (strength1 * 10.0 / distance.max(1.0)) as i32;
                    
                    *damage_dealt.entry(player2.clone()).or_insert(0) += damage;
                }
            }
        }
        
        // Apply damage
        for (player, damage) in damage_dealt {
            let health = self.player_health.get_mut(&player).unwrap();
            *health = (*health - damage).max(0);
            
            if *health == 0 {
                self.elimination_history.push(EliminationEvent {
                    round,
                    player: player.clone(),
                    reason: "Eliminated in combat".to_string(),
                    final_position: *self.player_positions.get(&player).unwrap(),
                });
            }
        }
    }
    
    fn detect_alliance_emergence(&self, state: &GameState) -> Option<EmergenceEvent> {
        if state.round < 10 {
            return None;
        }
        
        // Check for coordinated movements among collective players
        let collective_players: Vec<_> = self.player_positions.keys()
            .filter(|id| id.starts_with("collective_"))
            .collect();
        
        if collective_players.len() < 3 {
            return None;
        }
        
        // Calculate average distance between collective players
        let mut total_distance = 0.0;
        let mut pairs = 0;
        
        for i in 0..collective_players.len() {
            for j in i+1..collective_players.len() {
                if let (Some(pos1), Some(pos2)) = (
                    self.player_positions.get(collective_players[i]),
                    self.player_positions.get(collective_players[j])
                ) {
                    total_distance += pos1.distance_to(pos2);
                    pairs += 1;
                }
            }
        }
        
        let avg_distance = if pairs > 0 { total_distance / pairs as f32 } else { f32::MAX };
        
        // If collective players are clustering (alliance behavior)
        if avg_distance < 20.0 {
            Some(EmergenceEvent {
                round: state.round,
                event_type: "alliance_formation".to_string(),
                description: "Collective players formed strategic alliances".to_string(),
                emergence_score: 1.0 - (avg_distance / self.arena_size),
            })
        } else {
            None
        }
    }
}

#[async_trait]
impl Game for KingOfTheHill {
    async fn initialize(&mut self, _config: GameConfig) -> anyhow::Result<GameState> {
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::KingOfTheHill,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("arena_size".to_string(), serde_json::json!(self.arena_size));
                meta.insert("hill_radius".to_string(), serde_json::json!(self.hill_radius));
                meta.insert("hill_position".to_string(), serde_json::json!({
                    "x": self.hill_center.x,
                    "y": self.hill_center.y
                }));
                meta.insert("game_rules".to_string(), serde_json::json!({
                    "actions": {
                        "move_to_hill": "Move towards the hill",
                        "push": "Push nearby players (has cooldown)",
                        "defend": "Increase defense, reduce movement",
                        "charge": "Fast movement towards target",
                        "form_alliance": "Propose alliance with nearby player"
                    },
                    "scoring": "1 point per round controlling hill",
                    "combat": "Automatic when players are close",
                    "objective": "Control the hill for the most time"
                }));
                meta
            },
        })
    }
    
    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        // Initialize new players
        for player_id in actions.keys() {
            if !self.player_positions.contains_key(player_id) {
                self.spawn_player(player_id);
            }
        }
        
        // Update cooldowns
        for cooldown in self.push_cooldowns.values_mut() {
            *cooldown = cooldown.saturating_sub(1);
        }
        
        let mut scores_delta = HashMap::new();
        let mut special_events = vec![];
        
        // Process player actions
        for (player_id, action) in &actions {
            if !self.is_player_alive(player_id) {
                continue;
            }
            
            let current_pos = self.player_positions.get(player_id).copied()
                .unwrap_or(Position { x: 0.0, y: 0.0 });
            
            match action.action_type.as_str() {
                "move_to_hill" => {
                    let new_pos = current_pos.move_towards(&self.hill_center, 5.0);
                    self.player_positions.insert(player_id.clone(), new_pos);
                }
                "push" => {
                    if self.push_cooldowns.get(player_id).copied().unwrap_or(0) == 0 {
                        // Find nearby players to push
                        let mut pushed_players = Vec::new();
                        let pusher_strength = self.player_strength.get(player_id).copied().unwrap_or(1.0);
                        
                        for (other_id, other_pos) in self.player_positions.clone() {
                            if other_id != *player_id && self.is_player_alive(&other_id) {
                                let distance = current_pos.distance_to(&other_pos);
                                if distance < 10.0 {
                                    let push_force = pusher_strength * 15.0 / distance.max(1.0);
                                    let new_pos = other_pos.push_away_from(&current_pos, push_force);
                                    
                                    // Keep within arena bounds
                                    let bounded_pos = Position {
                                        x: new_pos.x.clamp(0.0, self.arena_size),
                                        y: new_pos.y.clamp(0.0, self.arena_size),
                                    };
                                    
                                    self.player_positions.insert(other_id.clone(), bounded_pos);
                                    pushed_players.push(other_id.clone());
                                    
                                    // Damage from push
                                    let health = self.player_health.get_mut(&other_id).unwrap();
                                    *health = (*health - 5).max(0);
                                }
                            }
                        }
                        
                        if !pushed_players.is_empty() {
                            special_events.push(format!("{} pushed {} players!", player_id, pushed_players.len()));
                            self.push_cooldowns.insert(player_id.clone(), 5);
                        }
                    }
                }
                "defend" => {
                    // Increase defense, reduce damage taken
                    let strength = self.player_strength.get_mut(player_id).unwrap();
                    *strength = (*strength * 1.2).min(3.0);
                    
                    // Heal slightly
                    let health = self.player_health.get_mut(player_id).unwrap();
                    *health = (*health + 5).min(100);
                }
                "charge" => {
                    // Fast movement towards hill
                    let new_pos = current_pos.move_towards(&self.hill_center, 10.0);
                    self.player_positions.insert(player_id.clone(), new_pos);
                    
                    // But take damage from exertion
                    let health = self.player_health.get_mut(player_id).unwrap();
                    *health = (*health - 3).max(0);
                }
                "form_alliance" => {
                    if let Some(target) = action.data.get("target").and_then(|v| v.as_str()) {
                        let allies = self.alliance_network.entry(player_id.clone()).or_insert_with(Vec::new);
                        if !allies.contains(&target.to_string()) {
                            allies.push(target.to_string());
                            
                            // Reciprocal alliance
                            let target_allies = self.alliance_network.entry(target.to_string()).or_insert_with(Vec::new);
                            if !target_allies.contains(player_id) {
                                target_allies.push(player_id.clone());
                            }
                            
                            special_events.push(format!("{} and {} formed an alliance!", player_id, target));
                        }
                    }
                }
                _ => {
                    // Default: move slowly towards hill
                    let new_pos = current_pos.move_towards(&self.hill_center, 2.0);
                    self.player_positions.insert(player_id.clone(), new_pos);
                }
            }
        }
        
        // Process combat
        self.process_combat(state.round + 1);
        
        // Update hill control
        self.update_hill_control();
        
        // Award points for hill control
        if let Some(controller) = &self.hill_controller {
            scores_delta.insert(controller.clone(), 10);
            special_events.push(format!("👑 {} controls the hill!", controller));
        } else {
            special_events.push("⚔️ Hill is contested!".to_string());
        }
        
        // Award survival points
        for (player_id, &health) in &self.player_health {
            if health > 0 {
                *scores_delta.entry(player_id.clone()).or_insert(0) += 1;
            }
        }
        
        // Check for emergence
        let emergence_event = self.detect_alliance_emergence(state);
        if let Some(event) = &emergence_event {
            special_events.push(event.description.clone());
        }
        
        // Determine round outcomes
        let losers: Vec<String> = self.elimination_history.iter()
            .filter(|e| e.round == state.round + 1)
            .map(|e| e.player.clone())
            .collect();
        
        let winners = if let Some(controller) = &self.hill_controller {
            vec![controller.clone()]
        } else {
            vec![]
        };
        
        // Arena shrinks every 20 rounds
        if (state.round + 1) % 20 == 0 && self.arena_size > 50.0 {
            self.arena_size *= 0.9;
            special_events.push(format!("⚠️ Arena shrinking! New size: {:.0}", self.arena_size));
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
            scores_delta,
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn is_game_over(&self, state: &GameState) -> bool {
        let alive_count = self.player_health.values().filter(|&&h| h > 0).count();
        
        alive_count <= 1 || state.round >= 100 || 
        self.control_duration.values().any(|&duration| duration >= 50)
    }
    
    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        // Winner is player with most hill control time
        let winner = self.control_duration.iter()
            .max_by_key(|(_, &duration)| duration)
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| {
                // Fallback to highest scorer
                state.scores.iter()
                    .max_by_key(|(_, &score)| score)
                    .map(|(id, _)| id.clone())
                    .unwrap_or_else(|| "No winner".to_string())
            });
        
        // Final scores include control time bonus
        let mut final_scores = state.scores.clone();
        for (player, &duration) in &self.control_duration {
            *final_scores.entry(player.clone()).or_insert(0) += duration as i32 * 10;
        }
        
        // Survival bonus
        for (player, &health) in &self.player_health {
            if health > 0 {
                *final_scores.entry(player.clone()).or_insert(0) += health;
            }
        }
        
        // Collect emergence events
        let emergence_events: Vec<EmergenceEvent> = state.history.iter()
            .enumerate()
            .filter_map(|(i, round_result)| {
                if round_result.outcome.emergence_detected {
                    Some(EmergenceEvent {
                        round: i as u32,
                        event_type: "alliance_formation".to_string(),
                        description: "Strategic alliances emerged".to_string(),
                        emergence_score: 0.9,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        let emergence_frequency = emergence_events.len() as f32 / state.round.max(1) as f32;
        
        // Calculate alliance metrics
        let alliance_count = self.alliance_network.values()
            .map(|allies| allies.len())
            .sum::<usize>() as f32;
        
        let collective_control = self.control_duration.iter()
            .filter(|(id, _)| id.starts_with("collective_"))
            .map(|(_, &d)| d)
            .sum::<u32>() as f32;
        let sota_control = self.control_duration.iter()
            .filter(|(id, _)| id.starts_with("sota_"))
            .map(|(_, &d)| d)
            .sum::<u32>() as f32;
        
        GameResult {
            game_id: state.game_id,
            winner,
            final_scores,
            total_rounds: state.round,
            emergence_events,
            analytics: GameAnalytics {
                collective_coordination_score: alliance_count / self.player_positions.len().max(1) as f32,
                decision_diversity_index: 0.7,
                strategic_depth: 0.85,
                emergence_frequency,
                performance_differential: collective_control - sota_control,
            },
        }
    }
}