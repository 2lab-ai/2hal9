use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use super::{Game, GameConfig, GameState, GameType, Action, RoundResult, Outcome, GameResult, GameAnalytics};

const INITIAL_DICE: usize = 5;
const CHALLENGE_PENALTY: i32 = 10;
const SUCCESSFUL_BLUFF_BONUS: i32 = 5;
const CORRECT_CHALLENGE_BONUS: i32 = 15;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiarsDiceGame {
    player_dice: HashMap<String, Vec<u8>>,
    current_bid: Option<Bid>,
    current_bidder: Option<String>,
    betting_order: Vec<String>,
    current_turn_index: usize,
    eliminated_players: Vec<String>,
    round_history: Vec<RoundHistory>,
    max_rounds: u32,
    bluff_statistics: HashMap<String, BluffStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Bid {
    quantity: usize,
    face_value: u8,
    player: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoundHistory {
    bids: Vec<Bid>,
    challenger: Option<String>,
    challenged: Option<String>,
    was_bluff: bool,
    actual_count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct BluffStats {
    total_bids: u32,
    bluffs_made: u32,
    bluffs_caught: u32,
    successful_challenges: u32,
    failed_challenges: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum LDAction {
    Bid { player: String, quantity: usize, face_value: u8 },
    Challenge { player: String },
    Pass { player: String }, // For spectating after elimination
}

impl Default for LiarsDiceGame {
    fn default() -> Self {
        Self::new()
    }
}

impl LiarsDiceGame {
    pub fn new() -> Self {
        Self {
            player_dice: HashMap::new(),
            current_bid: None,
            current_bidder: None,
            betting_order: Vec::new(),
            current_turn_index: 0,
            eliminated_players: Vec::new(),
            round_history: Vec::new(),
            max_rounds: 100,
            bluff_statistics: HashMap::new(),
        }
    }

    fn roll_dice(count: usize) -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..count).map(|_| rng.gen_range(1..=6)).collect()
    }

    fn initialize_players(&mut self, players: Vec<String>) {
        self.betting_order = players.clone();
        for player in players {
            self.player_dice.insert(player.clone(), Self::roll_dice(INITIAL_DICE));
            self.bluff_statistics.insert(player, BluffStats::default());
        }
    }

    fn count_dice(&self, face_value: u8) -> usize {
        self.player_dice.values()
            .flat_map(|dice| dice.iter())
            .filter(|&&die| die == face_value || die == 1) // 1s are wild
            .count()
    }

    fn is_valid_bid(&self, bid: &Bid) -> bool {
        if let Some(current) = &self.current_bid {
            // Must increase quantity or same quantity with higher face value
            bid.quantity > current.quantity || 
            (bid.quantity == current.quantity && bid.face_value > current.face_value)
        } else {
            // First bid is always valid if reasonable
            bid.quantity > 0 && bid.face_value >= 1 && bid.face_value <= 6
        }
    }

    fn process_bid(&mut self, player: &str, quantity: usize, face_value: u8) -> Result<(), String> {
        if self.eliminated_players.contains(&player.to_string()) {
            return Err("Player is eliminated".to_string());
        }

        let bid = Bid {
            quantity,
            face_value,
            player: player.to_string(),
        };

        if !self.is_valid_bid(&bid) {
            return Err("Invalid bid".to_string());
        }

        // Update bluff statistics
        if let Some(stats) = self.bluff_statistics.get_mut(player) {
            stats.total_bids += 1;
            
            // Check if this is likely a bluff
            let player_dice_count = self.player_dice.get(player)
                .map(|dice| dice.iter().filter(|&&d| d == face_value || d == 1).count())
                .unwrap_or(0);
            
            if quantity > self.player_dice.len() * 2 + player_dice_count {
                stats.bluffs_made += 1;
            }
        }

        self.current_bid = Some(bid);
        self.current_bidder = Some(player.to_string());
        self.advance_turn();
        
        Ok(())
    }

    fn process_challenge(&mut self, challenger: &str) -> Result<(bool, String), String> {
        if self.eliminated_players.contains(&challenger.to_string()) {
            return Err("Player is eliminated".to_string());
        }

        let bid = self.current_bid.clone()
            .ok_or("No bid to challenge")?;
        
        let actual_count = self.count_dice(bid.face_value);
        let was_bluff = actual_count < bid.quantity;
        
        // Update statistics
        if let Some(challenger_stats) = self.bluff_statistics.get_mut(challenger) {
            if was_bluff {
                challenger_stats.successful_challenges += 1;
            } else {
                challenger_stats.failed_challenges += 1;
            }
        }
        
        if let Some(bidder_stats) = self.bluff_statistics.get_mut(&bid.player) {
            if was_bluff {
                bidder_stats.bluffs_caught += 1;
            }
        }
        
        // Record round history
        self.round_history.push(RoundHistory {
            bids: vec![bid.clone()],
            challenger: Some(challenger.to_string()),
            challenged: Some(bid.player.clone()),
            was_bluff,
            actual_count,
        });
        
        // Determine loser and remove a die
        let loser = if was_bluff { &bid.player } else { challenger };
        let winner = if was_bluff { challenger.to_string() } else { bid.player.clone() };
        
        if let Some(dice) = self.player_dice.get_mut(loser) {
            if !dice.is_empty() {
                dice.pop();
                if dice.is_empty() {
                    self.eliminated_players.push(loser.to_string());
                    self.betting_order.retain(|p| p != loser);
                }
            }
        }
        
        // Reset for next round
        self.current_bid = None;
        self.current_bidder = None;
        self.reroll_all_dice();
        
        Ok((was_bluff, winner))
    }

    fn advance_turn(&mut self) {
        if !self.betting_order.is_empty() {
            self.current_turn_index = (self.current_turn_index + 1) % self.betting_order.len();
        }
    }

    fn reroll_all_dice(&mut self) {
        for (player, dice) in self.player_dice.iter_mut() {
            if !self.eliminated_players.contains(player) {
                let count = dice.len();
                *dice = Self::roll_dice(count);
            }
        }
    }

    fn calculate_analytics(&self, _history: &[RoundResult]) -> GameAnalytics {
        let total_players = self.player_dice.len();
        let active_players = total_players - self.eliminated_players.len();
        let survival_rate = active_players as f32 / total_players as f32;
        
        // Bluffing frequency
        let total_bluffs: u32 = self.bluff_statistics.values()
            .map(|s| s.bluffs_made)
            .sum();
        let total_bids: u32 = self.bluff_statistics.values()
            .map(|s| s.total_bids)
            .sum();
        let bluff_rate = if total_bids > 0 {
            total_bluffs as f32 / total_bids as f32
        } else {
            0.0
        };
        
        // Challenge accuracy
        let successful_challenges: u32 = self.bluff_statistics.values()
            .map(|s| s.successful_challenges)
            .sum();
        let total_challenges: u32 = self.bluff_statistics.values()
            .map(|s| s.successful_challenges + s.failed_challenges)
            .sum();
        let challenge_accuracy = if total_challenges > 0 {
            successful_challenges as f32 / total_challenges as f32
        } else {
            0.5
        };
        
        GameAnalytics {
            collective_coordination_score: 0.0, // Liar's Dice is competitive
            decision_diversity_index: bluff_rate,
            strategic_depth: challenge_accuracy * 0.5 + bluff_rate * 0.5,
            emergence_frequency: 0.0,
            performance_differential: 1.0 - survival_rate,
        }
    }

    #[allow(dead_code)]
    fn get_current_player(&self) -> Option<&String> {
        self.betting_order.get(self.current_turn_index)
    }
}

#[async_trait]
impl Game for LiarsDiceGame {
    async fn initialize(&mut self, config: GameConfig) -> anyhow::Result<GameState> {
        if let Some(rounds_str) = config.special_rules.get("max_rounds") {
            if let Ok(rounds) = rounds_str.parse::<u32>() {
                self.max_rounds = rounds;
            }
        }
        
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::LiarsDice,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("dice_per_player".to_string(), serde_json::json!(INITIAL_DICE));
                meta.insert("wild_ones".to_string(), serde_json::json!(true));
                meta
            },
        })
    }

    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        // Initialize players on first round
        if state.round == 0 && self.player_dice.is_empty() {
            let players: Vec<String> = actions.keys().cloned().collect();
            self.initialize_players(players);
        }

        let mut round_events = Vec::new();
        let mut scores_delta = HashMap::new();
        let mut challenge_occurred = false;
        
        // Process actions from all players
        for (player_id, action) in &actions {
            if self.eliminated_players.contains(player_id) {
                continue;
            }
            
            match action.action_type.as_str() {
                "bid" => {
                    if let Ok(bid_data) = serde_json::from_value::<serde_json::Value>(action.data.clone()) {
                        if let (Some(quantity), Some(face_value)) = 
                            (bid_data["quantity"].as_u64(), bid_data["face_value"].as_u64()) {
                            
                            match self.process_bid(player_id, quantity as usize, face_value as u8) {
                                Ok(_) => {
                                    round_events.push(format!("{} bid {} {}s", player_id, quantity, face_value));
                                    scores_delta.insert(player_id.clone(), 1); // Point for making a bid
                                }
                                Err(e) => {
                                    round_events.push(format!("{} made invalid bid: {}", player_id, e));
                                }
                            }
                        }
                    }
                }
                "challenge" => {
                    if !challenge_occurred && self.current_bid.is_some() {
                        match self.process_challenge(player_id) {
                            Ok((was_bluff, winner)) => {
                                challenge_occurred = true;
                                if was_bluff {
                                    round_events.push(format!("{} successfully challenged! It was a bluff.", player_id));
                                    scores_delta.insert(player_id.clone(), CORRECT_CHALLENGE_BONUS);
                                    if let Some(bid) = &self.current_bid {
                                        *scores_delta.entry(bid.player.clone()).or_insert(0) -= CHALLENGE_PENALTY;
                                    }
                                } else {
                                    round_events.push(format!("{}'s challenge failed! The bid was valid.", player_id));
                                    scores_delta.insert(player_id.clone(), -CHALLENGE_PENALTY);
                                    scores_delta.insert(winner, SUCCESSFUL_BLUFF_BONUS);
                                }
                            }
                            Err(e) => {
                                round_events.push(format!("Challenge error: {}", e));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Show current dice counts for drama
        if self.current_bid.is_some() && !challenge_occurred {
            let total_dice = self.player_dice.values()
                .map(|dice| dice.len())
                .sum::<usize>();
            round_events.push(format!("Total dice in play: {}", total_dice));
        }
        
        let active_players: Vec<String> = self.betting_order.iter()
            .filter(|p| !self.eliminated_players.contains(p))
            .cloned()
            .collect();
        
        Ok(RoundResult {
            round: state.round,
            actions,
            outcome: Outcome {
                winners: active_players,
                losers: self.eliminated_players.clone(),
                special_events: round_events,
                emergence_detected: false,
            },
            scores_delta,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn is_game_over(&self, state: &GameState) -> bool {
        let active_players = self.player_dice.len() - self.eliminated_players.len();
        active_players <= 1 || state.round >= self.max_rounds
    }

    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        let winner = self.betting_order.iter()
            .find(|p| !self.eliminated_players.contains(p))
            .cloned()
            .unwrap_or_else(|| "No Winner".to_string());
        
        let mut final_scores = state.scores.clone();
        
        // Victory bonus
        if winner != "No Winner" {
            *final_scores.entry(winner.clone()).or_insert(0) += 50;
        }
        
        // Bonus for bluffing skill
        for (player, stats) in &self.bluff_statistics {
            let bluff_success_rate = if stats.bluffs_made > 0 {
                (stats.bluffs_made - stats.bluffs_caught) as f32 / stats.bluffs_made as f32
            } else {
                0.0
            };
            
            let challenge_success_rate = if stats.successful_challenges + stats.failed_challenges > 0 {
                stats.successful_challenges as f32 / 
                (stats.successful_challenges + stats.failed_challenges) as f32
            } else {
                0.0
            };
            
            let skill_bonus = ((bluff_success_rate + challenge_success_rate) * 10.0) as i32;
            *final_scores.entry(player.clone()).or_insert(0) += skill_bonus;
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