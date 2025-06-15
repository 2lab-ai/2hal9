use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use super::{Game, GameConfig, GameState, GameType, Action, RoundResult, Outcome, GameResult, GameAnalytics};

const MAP_SIZE: usize = 20;
const INITIAL_SAFE_ZONE: usize = 20;
const ZONE_SHRINK_RATE: usize = 2;
const STORM_DAMAGE: i32 = 10;
const COMBAT_DAMAGE: i32 = 20;
const LOOT_BONUS: i32 = 5;
const SURVIVAL_POINTS: i32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleRoyaleGame {
    map_size: usize,
    safe_zone_radius: usize,
    safe_zone_center: (usize, usize),
    player_positions: HashMap<String, Position>,
    player_health: HashMap<String, i32>,
    player_loot: HashMap<String, i32>,
    eliminated_players: Vec<String>,
    round_actions: Vec<BRAction>,
    max_rounds: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum BRAction {
    Move { player: String, direction: Direction },
    Attack { player: String, target: String },
    Loot { player: String },
    Hide { player: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum Direction {
    North,
    South,
    East,
    West,
    Northeast,
    Northwest,
    Southeast,
    Southwest,
}

impl Default for BattleRoyaleGame {
    fn default() -> Self {
        Self::new()
    }
}

impl BattleRoyaleGame {
    pub fn new() -> Self {
        Self {
            map_size: MAP_SIZE,
            safe_zone_radius: INITIAL_SAFE_ZONE,
            safe_zone_center: (MAP_SIZE / 2, MAP_SIZE / 2),
            player_positions: HashMap::new(),
            player_health: HashMap::new(),
            player_loot: HashMap::new(),
            eliminated_players: Vec::new(),
            round_actions: Vec::new(),
            max_rounds: 50,
        }
    }

    fn spawn_players(&mut self, players: Vec<String>) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        for player in players {
            // Spawn players randomly within the initial safe zone
            let angle = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
            let radius = rng.gen::<f32>() * (self.safe_zone_radius as f32 * 0.8);
            
            let x = (self.safe_zone_center.0 as f32 + radius * angle.cos()) as usize;
            let y = (self.safe_zone_center.1 as f32 + radius * angle.sin()) as usize;
            
            self.player_positions.insert(player.clone(), Position { 
                x: x.min(self.map_size - 1), 
                y: y.min(self.map_size - 1) 
            });
            self.player_health.insert(player.clone(), 100);
            self.player_loot.insert(player, 0);
        }
    }

    fn shrink_safe_zone(&mut self) {
        if self.safe_zone_radius > ZONE_SHRINK_RATE {
            self.safe_zone_radius -= ZONE_SHRINK_RATE;
        } else if self.safe_zone_radius > 1 {
            self.safe_zone_radius = 1;
        }
    }

    fn is_in_safe_zone(&self, pos: &Position) -> bool {
        let dx = pos.x as f32 - self.safe_zone_center.0 as f32;
        let dy = pos.y as f32 - self.safe_zone_center.1 as f32;
        let distance = (dx * dx + dy * dy).sqrt();
        distance <= self.safe_zone_radius as f32
    }

    fn apply_storm_damage(&mut self) {
        let mut damage_events = Vec::new();
        
        for (player, pos) in &self.player_positions {
            if !self.is_in_safe_zone(pos) && !self.eliminated_players.contains(player) {
                damage_events.push((player.clone(), STORM_DAMAGE));
            }
        }
        
        for (player, damage) in damage_events {
            if let Some(health) = self.player_health.get_mut(&player) {
                *health -= damage;
                if *health <= 0 {
                    self.eliminated_players.push(player);
                }
            }
        }
    }

    fn move_player(&mut self, player: &str, direction: Direction) -> bool {
        if let Some(pos) = self.player_positions.get_mut(player) {
            let (new_x, new_y) = match direction {
                Direction::North => (pos.x, pos.y.saturating_sub(1)),
                Direction::South => (pos.x, (pos.y + 1).min(self.map_size - 1)),
                Direction::East => ((pos.x + 1).min(self.map_size - 1), pos.y),
                Direction::West => (pos.x.saturating_sub(1), pos.y),
                Direction::Northeast => ((pos.x + 1).min(self.map_size - 1), pos.y.saturating_sub(1)),
                Direction::Northwest => (pos.x.saturating_sub(1), pos.y.saturating_sub(1)),
                Direction::Southeast => ((pos.x + 1).min(self.map_size - 1), (pos.y + 1).min(self.map_size - 1)),
                Direction::Southwest => (pos.x.saturating_sub(1), (pos.y + 1).min(self.map_size - 1)),
            };
            
            pos.x = new_x;
            pos.y = new_y;
            true
        } else {
            false
        }
    }

    fn can_attack(&self, attacker: &str, target: &str) -> bool {
        if let (Some(pos1), Some(pos2)) = (self.player_positions.get(attacker), self.player_positions.get(target)) {
            let dx = (pos1.x as i32 - pos2.x as i32).abs();
            let dy = (pos1.y as i32 - pos2.y as i32).abs();
            dx <= 2 && dy <= 2 // Attack range of 2
        } else {
            false
        }
    }

    fn attack_player(&mut self, attacker: &str, target: &str) {
        if self.can_attack(attacker, target) {
            if let Some(health) = self.player_health.get_mut(target) {
                *health -= COMBAT_DAMAGE;
                if *health <= 0 {
                    self.eliminated_players.push(target.to_string());
                    
                    // Transfer loot
                    if let Some(target_loot) = self.player_loot.get(target).copied() {
                        if let Some(attacker_loot) = self.player_loot.get_mut(attacker) {
                            *attacker_loot += target_loot;
                        }
                    }
                }
            }
        }
    }

    fn loot_area(&mut self, player: &str) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        if rng.gen::<f32>() < 0.7 { // 70% chance to find loot
            if let Some(loot) = self.player_loot.get_mut(player) {
                *loot += LOOT_BONUS;
            }
            
            // Heal if lucky
            if rng.gen::<f32>() < 0.3 { // 30% chance to find healing
                if let Some(health) = self.player_health.get_mut(player) {
                    *health = (*health + 20).min(100);
                }
            }
        }
    }

    fn calculate_analytics(&self, history: &[RoundResult]) -> GameAnalytics {
        let total_players = self.player_positions.len();
        let survival_rate = (total_players - self.eliminated_players.len()) as f32 / total_players as f32;
        
        // Calculate combat intensity
        let combat_actions = history.iter()
            .flat_map(|r| r.actions.values())
            .filter(|a| a.action_type == "attack")
            .count();
        let combat_intensity = combat_actions as f32 / history.len().max(1) as f32;
        
        // Strategic diversity (different action types used)
        let action_diversity = history.iter()
            .flat_map(|r| r.actions.values())
            .map(|a| &a.action_type)
            .collect::<std::collections::HashSet<_>>()
            .len() as f32 / 4.0; // 4 possible actions
        
        GameAnalytics {
            collective_coordination_score: 0.0, // Battle Royale is not collaborative
            decision_diversity_index: action_diversity,
            strategic_depth: combat_intensity * 0.5 + survival_rate * 0.5,
            emergence_frequency: 0.0,
            performance_differential: 1.0 - survival_rate,
        }
    }
}

#[async_trait]
impl Game for BattleRoyaleGame {
    async fn initialize(&mut self, config: GameConfig) -> anyhow::Result<GameState> {
        if let Some(rounds_str) = config.special_rules.get("max_rounds") {
            if let Ok(rounds) = rounds_str.parse::<u32>() {
                self.max_rounds = rounds;
            }
        }
        
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::BattleRoyale,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("map_size".to_string(), serde_json::json!(self.map_size));
                meta.insert("safe_zone_radius".to_string(), serde_json::json!(self.safe_zone_radius));
                meta.insert("safe_zone_center".to_string(), serde_json::json!(self.safe_zone_center));
                meta
            },
        })
    }

    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        // Initialize players on first round
        if state.round == 0 && self.player_positions.is_empty() {
            let players: Vec<String> = actions.keys().cloned().collect();
            self.spawn_players(players);
        }

        // Process actions
        self.round_actions.clear();
        
        for (player_id, action) in &actions {
            if self.eliminated_players.contains(player_id) {
                continue;
            }
            
            match action.action_type.as_str() {
                "move" => {
                    if let Ok(direction) = serde_json::from_value::<Direction>(action.data.clone()) {
                        self.move_player(player_id, direction);
                        self.round_actions.push(BRAction::Move {
                            player: player_id.clone(),
                            direction,
                        });
                    }
                }
                "attack" => {
                    if let Ok(target) = serde_json::from_value::<String>(action.data.clone()) {
                        self.attack_player(player_id, &target);
                        self.round_actions.push(BRAction::Attack {
                            player: player_id.clone(),
                            target,
                        });
                    }
                }
                "loot" => {
                    self.loot_area(player_id);
                    self.round_actions.push(BRAction::Loot {
                        player: player_id.clone(),
                    });
                }
                "hide" => {
                    self.round_actions.push(BRAction::Hide {
                        player: player_id.clone(),
                    });
                }
                _ => {}
            }
        }
        
        // Shrink safe zone every 5 rounds
        if state.round > 0 && state.round % 5 == 0 {
            self.shrink_safe_zone();
        }
        
        // Apply storm damage
        self.apply_storm_damage();
        
        // Calculate scores (survival bonus + loot)
        let mut scores_delta = HashMap::new();
        for player in self.player_positions.keys() {
            if !self.eliminated_players.contains(player) {
                let loot_score = self.player_loot.get(player).copied().unwrap_or(0);
                scores_delta.insert(player.clone(), SURVIVAL_POINTS + loot_score / 5);
            }
        }
        
        // Determine winners/losers for this round
        let survivors: Vec<String> = self.player_positions.keys()
            .filter(|p| !self.eliminated_players.contains(p))
            .cloned()
            .collect();
        
        let special_events = vec![
            format!("Safe zone radius: {}", self.safe_zone_radius),
            format!("{} players eliminated", self.eliminated_players.len()),
        ];
        
        Ok(RoundResult {
            round: state.round,
            actions,
            outcome: Outcome {
                winners: survivors.clone(),
                losers: self.eliminated_players.clone(),
                special_events,
                emergence_detected: false,
            },
            scores_delta,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn is_game_over(&self, state: &GameState) -> bool {
        let survivors = self.player_positions.len() - self.eliminated_players.len();
        survivors <= 1 || state.round >= self.max_rounds || self.safe_zone_radius == 0
    }

    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        let winner = self.player_positions.keys()
            .find(|p| !self.eliminated_players.contains(p))
            .cloned()
            .unwrap_or_else(|| "No Winner".to_string());
        
        let mut final_scores = state.scores.clone();
        
        // Bonus for winner
        if winner != "No Winner" {
            *final_scores.entry(winner.clone()).or_insert(0) += 50;
        }
        
        GameResult {
            game_id: state.game_id,
            winner,
            final_scores,
            total_rounds: state.round,
            emergence_events: vec![],
            analytics: self.calculate_analytics(&state.history),
        }
    }
}