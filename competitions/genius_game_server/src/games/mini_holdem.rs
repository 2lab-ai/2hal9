use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use super::{Game, GameState, PlayerAction, GameResult, EmergenceMetrics};
use anyhow::Result;
use rand::seq::SliceRandom;

const STARTING_CHIPS: i32 = 1000;
const SMALL_BLIND: i32 = 10;
const BIG_BLIND: i32 = 20;

/// Mini Hold'em - Simplified Texas Hold'em for AI evaluation
pub struct MiniHoldem {
    id: Uuid,
    round: u32,
    max_rounds: u32,
    players: Vec<String>,
    chips: HashMap<String, i32>,
    hands: HashMap<String, Hand>,
    community_cards: Vec<Card>,
    pot: i32,
    current_bet: i32,
    player_bets: HashMap<String, i32>,
    folded_players: HashSet<String>,
    dealer_position: usize,
    current_player: usize,
    betting_round: BettingRound,
    hand_history: Vec<HandResult>,
    special_rules: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    rank: Rank,
    suit: Suit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rank {
    Two = 2, Three = 3, Four = 4, Five = 5, Six = 6, Seven = 7,
    Eight = 8, Nine = 9, Ten = 10, Jack = 11, Queen = 12, King = 13, Ace = 14,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    Hearts, Diamonds, Clubs, Spades,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Hand {
    cards: Vec<Card>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum BettingRound {
    PreFlop,
    Flop,
    Turn,
    River,
    Showdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HandResult {
    winner: String,
    winning_hand: String,
    pot_size: i32,
    showdown: bool,
    bluff_success: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandRank {
    HighCard = 0,
    Pair = 1,
    TwoPair = 2,
    ThreeOfAKind = 3,
    Straight = 4,
    Flush = 5,
    FullHouse = 6,
    FourOfAKind = 7,
    StraightFlush = 8,
}


impl MiniHoldem {
    pub fn new() -> Self {
        Self::new_with_config(50, HashMap::new())
    }
    
    pub fn new_with_config(max_rounds: u32, special_rules: HashMap<String, String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            round: 0,
            max_rounds,
            players: Vec::new(),
            chips: HashMap::new(),
            hands: HashMap::new(),
            community_cards: Vec::new(),
            pot: 0,
            current_bet: 0,
            player_bets: HashMap::new(),
            folded_players: HashSet::new(),
            dealer_position: 0,
            current_player: 0,
            betting_round: BettingRound::PreFlop,
            hand_history: Vec::new(),
            special_rules,
        }
    }
    
    fn create_deck() -> Vec<Card> {
        let mut deck = Vec::new();
        
        for &suit in &[Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades] {
            for rank_val in 2..=14 {
                let rank = match rank_val {
                    2 => Rank::Two, 3 => Rank::Three, 4 => Rank::Four,
                    5 => Rank::Five, 6 => Rank::Six, 7 => Rank::Seven,
                    8 => Rank::Eight, 9 => Rank::Nine, 10 => Rank::Ten,
                    11 => Rank::Jack, 12 => Rank::Queen, 13 => Rank::King,
                    14 => Rank::Ace, _ => continue,
                };
                deck.push(Card { rank, suit });
            }
        }
        
        deck
    }
    
    fn deal_new_hand(&mut self) {
        let mut deck = Self::create_deck();
        deck.shuffle(&mut rand::thread_rng());
        
        // Clear previous hand
        self.hands.clear();
        self.community_cards.clear();
        self.pot = 0;
        self.current_bet = 0;
        self.player_bets.clear();
        self.folded_players.clear();
        self.betting_round = BettingRound::PreFlop;
        
        // Deal 2 cards to each player
        let mut deck_iter = deck.into_iter();
        for player in &self.players {
            if self.chips.get(player).copied().unwrap_or(0) > 0 {
                let cards = vec![
                    deck_iter.next().unwrap(),
                    deck_iter.next().unwrap(),
                ];
                self.hands.insert(player.clone(), Hand { cards });
            }
        }
        
        // Post blinds
        let small_blind_pos = (self.dealer_position + 1) % self.players.len();
        let big_blind_pos = (self.dealer_position + 2) % self.players.len();
        
        let sb_player = &self.players[small_blind_pos];
        let bb_player = &self.players[big_blind_pos];
        
        self.player_bets.insert(sb_player.clone(), SMALL_BLIND);
        self.player_bets.insert(bb_player.clone(), BIG_BLIND);
        *self.chips.get_mut(sb_player).unwrap() -= SMALL_BLIND;
        *self.chips.get_mut(bb_player).unwrap() -= BIG_BLIND;
        self.pot = SMALL_BLIND + BIG_BLIND;
        self.current_bet = BIG_BLIND;
        
        self.current_player = (big_blind_pos + 1) % self.players.len();
        
        // Store remaining deck for community cards
        self.community_cards = deck_iter.take(5).collect();
    }
    
    pub fn evaluate_hand_from_cards(&self, cards: &[Card]) -> (HandRank, Vec<Card>) {
        self.find_best_hand(cards)
    }
    
    fn evaluate_hand(&self, player: &str) -> (HandRank, Vec<Card>) {
        let hand = match self.hands.get(player) {
            Some(h) => h,
            None => return (HandRank::HighCard, vec![]),
        };
        
        let mut all_cards = hand.cards.clone();
        
        // Add visible community cards based on betting round
        let community_count = match self.betting_round {
            BettingRound::PreFlop => 0,
            BettingRound::Flop => 3,
            BettingRound::Turn => 4,
            BettingRound::River | BettingRound::Showdown => 5,
        };
        
        all_cards.extend(self.community_cards.iter().take(community_count));
        
        // Find best 5-card combination
        // This is simplified - real poker hand evaluation is more complex
        self.find_best_hand(&all_cards)
    }
    
    fn find_best_hand(&self, cards: &[Card]) -> (HandRank, Vec<Card>) {
        // Count ranks and suits
        let mut rank_counts: HashMap<Rank, usize> = HashMap::new();
        let mut suit_counts: HashMap<Suit, usize> = HashMap::new();
        
        for card in cards {
            *rank_counts.entry(card.rank).or_insert(0) += 1;
            *suit_counts.entry(card.suit).or_insert(0) += 1;
        }
        
        // Check for pairs, trips, quads
        let mut pairs = 0;
        let mut trips = 0;
        let mut quads = 0;
        
        for &count in rank_counts.values() {
            match count {
                2 => pairs += 1,
                3 => trips += 1,
                4 => quads += 1,
                _ => {}
            }
        }
        
        // Check for flush
        let is_flush = suit_counts.values().any(|&count| count >= 5);
        
        // Check for straight (simplified)
        let mut ranks: Vec<i32> = cards.iter().map(|c| c.rank as i32).collect();
        ranks.sort();
        ranks.dedup();
        
        let is_straight = ranks.windows(5).any(|w| {
            w[0] + 1 == w[1] && w[1] + 1 == w[2] && 
            w[2] + 1 == w[3] && w[3] + 1 == w[4]
        });
        
        // Determine hand rank
        let hand_rank = if is_straight && is_flush {
            HandRank::StraightFlush
        } else if quads > 0 {
            HandRank::FourOfAKind
        } else if trips > 0 && pairs > 0 {
            HandRank::FullHouse
        } else if is_flush {
            HandRank::Flush
        } else if is_straight {
            HandRank::Straight
        } else if trips > 0 {
            HandRank::ThreeOfAKind
        } else if pairs >= 2 {
            HandRank::TwoPair
        } else if pairs == 1 {
            HandRank::Pair
        } else {
            HandRank::HighCard
        };
        
        // Return best 5 cards (simplified - just return first 5)
        let best_cards = cards.iter().take(5).cloned().collect();
        
        (hand_rank, best_cards)
    }
    
    fn is_betting_complete(&self) -> bool {
        let active_players: Vec<_> = self.players.iter()
            .filter(|p| !self.folded_players.contains(*p) && self.chips.get(*p).copied().unwrap_or(0) > 0)
            .collect();
        
        if active_players.len() <= 1 {
            return true;
        }
        
        // All active players have matched the current bet
        active_players.iter().all(|p| {
            self.player_bets.get(*p).copied().unwrap_or(0) == self.current_bet ||
            self.chips.get(*p).copied().unwrap_or(0) == 0
        })
    }
    
    fn advance_betting_round(&mut self) {
        // Move all bets to pot
        for (_, bet) in self.player_bets.drain() {
            self.pot += bet;
        }
        self.current_bet = 0;
        
        self.betting_round = match self.betting_round {
            BettingRound::PreFlop => BettingRound::Flop,
            BettingRound::Flop => BettingRound::Turn,
            BettingRound::Turn => BettingRound::River,
            BettingRound::River => BettingRound::Showdown,
            BettingRound::Showdown => BettingRound::Showdown,
        };
        
        self.current_player = (self.dealer_position + 1) % self.players.len();
    }
    
    fn determine_winner(&self) -> (String, String, bool) {
        let active_players: Vec<_> = self.players.iter()
            .filter(|p| !self.folded_players.contains(*p))
            .collect();
        
        if active_players.len() == 1 {
            // Win by fold
            return (active_players[0].clone(), "Win by fold".to_string(), false);
        }
        
        // Showdown
        let mut best_player = active_players[0].clone();
        let mut best_rank = HandRank::HighCard;
        
        for player in active_players {
            let (rank, _) = self.evaluate_hand(player);
            if rank > best_rank {
                best_rank = rank;
                best_player = player.clone();
            }
        }
        
        let hand_name = format!("{:?}", best_rank);
        (best_player, hand_name, true)
    }
    
    fn detect_emergence(&self) -> EmergenceMetrics {
        if self.hand_history.len() < 5 {
            return EmergenceMetrics {
                emergence_detected: false,
                coordination_score: 0.0,
                collective_intelligence_index: 0.0,
                decision_diversity_index: 0.0,
                strategic_depth: 0.0,
                special_patterns: HashMap::new(),
            };
        }
        
        // Analyze bluffing patterns
        let total_hands = self.hand_history.len() as f32;
        let bluff_attempts = self.hand_history.iter()
            .filter(|h| h.pot_size > 100 && !h.showdown)
            .count() as f32;
        let bluff_rate = bluff_attempts / total_hands;
        
        // Analyze pot control
        let avg_pot = self.hand_history.iter()
            .map(|h| h.pot_size)
            .sum::<i32>() as f32 / total_hands;
        let pot_variance = self.hand_history.iter()
            .map(|h| (h.pot_size as f32 - avg_pot).powi(2))
            .sum::<f32>() / total_hands;
        let pot_control = 1.0 / (1.0 + pot_variance / 10000.0);
        
        // Strategic depth - mix of showdowns vs folds
        let showdown_rate = self.hand_history.iter()
            .filter(|h| h.showdown)
            .count() as f32 / total_hands;
        let strategic_balance = 1.0 - (showdown_rate - 0.3).abs() / 0.7;
        
        let emergence_detected = bluff_rate > 0.1 && 
                                bluff_rate < 0.4 && 
                                pot_control > 0.5 &&
                                strategic_balance > 0.7;
        
        let mut special_patterns = HashMap::new();
        special_patterns.insert("bluff_rate".to_string(), bluff_rate.to_string());
        special_patterns.insert("pot_control".to_string(), pot_control.to_string());
        special_patterns.insert("showdown_rate".to_string(), showdown_rate.to_string());
        
        EmergenceMetrics {
            emergence_detected,
            coordination_score: pot_control,
            collective_intelligence_index: strategic_balance,
            decision_diversity_index: bluff_rate,
            strategic_depth: strategic_balance,
            special_patterns,
        }
    }
}

#[async_trait]
impl Game for MiniHoldem {
    async fn get_state(&self) -> Result<GameState> {
        let mut available_actions = Vec::new();
        
        if self.players.len() >= 2 && !self.players.is_empty() {
            let current_player = &self.players[self.current_player];
            
            if !self.folded_players.contains(current_player) {
                available_actions.push("fold".to_string());
                
                let player_bet = self.player_bets.get(current_player).copied().unwrap_or(0);
                let to_call = self.current_bet - player_bet;
                
                if to_call > 0 {
                    available_actions.push(format!("call:{}", to_call));
                } else {
                    available_actions.push("check".to_string());
                }
                
                // Simplified betting - fixed raise amounts
                let min_raise = self.current_bet + BIG_BLIND;
                available_actions.push(format!("raise:{}", min_raise));
                available_actions.push(format!("raise:{}", min_raise * 2));
                
                let all_in = self.chips.get(current_player).copied().unwrap_or(0);
                if all_in > 0 {
                    available_actions.push(format!("all-in:{}", all_in));
                }
            }
        }
        
        let visible_community = match self.betting_round {
            BettingRound::PreFlop => vec![],
            BettingRound::Flop => self.community_cards.iter().take(3).cloned().collect(),
            BettingRound::Turn => self.community_cards.iter().take(4).cloned().collect(),
            BettingRound::River | BettingRound::Showdown => self.community_cards.iter().take(5).cloned().collect(),
        };
        
        let mut player_states = HashMap::new();
        for player in &self.players {
            let hand_cards = if let Some(hand) = self.hands.get(player) {
                hand.cards.iter().map(|c| format!("{:?}{:?}", c.rank, c.suit)).collect()
            } else {
                vec![]
            };
            
            player_states.insert(player.clone(), serde_json::json!({
                "chips": self.chips.get(player).copied().unwrap_or(0),
                "current_bet": self.player_bets.get(player).copied().unwrap_or(0),
                "folded": self.folded_players.contains(player),
                "hand": hand_cards,
                "position": if self.players[(self.dealer_position + 1) % self.players.len()] == *player {
                    "SB"
                } else if self.players[(self.dealer_position + 2) % self.players.len()] == *player {
                    "BB"
                } else if self.players[self.dealer_position] == *player {
                    "Dealer"
                } else {
                    "Regular"
                },
            }));
        }
        
        Ok(GameState {
            game_id: self.id,
            game_type: "mini_holdem".to_string(),
            round: self.round,
            players: self.players.clone(),
            current_state: serde_json::json!({
                "pot": self.pot,
                "current_bet": self.current_bet,
                "betting_round": self.betting_round,
                "community_cards": visible_community.iter().map(|c| format!("{:?}{:?}", c.rank, c.suit)).collect::<Vec<_>>(),
                "current_player": if self.players.is_empty() { "" } else { &self.players[self.current_player] },
                "available_actions": available_actions,
            }),
            available_actions,
            scores: self.chips.clone().into_iter().map(|(k, v)| (k, v)).collect(),
            player_states,
            is_complete: self.round >= self.max_rounds || self.players.iter().filter(|p| self.chips.get(*p).copied().unwrap_or(0) > 0).count() <= 1,
            special_data: Some(serde_json::json!({
                "blinds": { "small": SMALL_BLIND, "big": BIG_BLIND },
                "hand_count": self.hand_history.len(),
                "active_players": self.players.iter().filter(|p| !self.folded_players.contains(*p)).count(),
            })),
        })
    }
    
    async fn process_action(&mut self, action: PlayerAction) -> Result<()> {
        if self.players.is_empty() || self.players[self.current_player] != action.player_id {
            return Ok(());
        }
        
        match action.action_type.as_str() {
            "fold" => {
                self.folded_players.insert(action.player_id);
            }
            "check" => {
                // Only valid if no bet to call
                if self.current_bet == self.player_bets.get(&action.player_id).copied().unwrap_or(0) {
                    // Valid check
                }
            }
            act if act.starts_with("call:") => {
                if let Ok(amount) = act[5..].parse::<i32>() {
                    let player_chips = self.chips.get_mut(&action.player_id).unwrap();
                    let call_amount = amount.min(*player_chips);
                    *player_chips -= call_amount;
                    *self.player_bets.entry(action.player_id).or_insert(0) += call_amount;
                }
            }
            act if act.starts_with("raise:") => {
                if let Ok(total) = act[6..].parse::<i32>() {
                    let player_chips = self.chips.get_mut(&action.player_id).unwrap();
                    let current_player_bet = self.player_bets.get(&action.player_id).copied().unwrap_or(0);
                    let raise_amount = (total - current_player_bet).min(*player_chips);
                    
                    if raise_amount > 0 {
                        *player_chips -= raise_amount;
                        self.player_bets.insert(action.player_id, current_player_bet + raise_amount);
                        self.current_bet = current_player_bet + raise_amount;
                    }
                }
            }
            act if act.starts_with("all-in:") => {
                let player_chips = self.chips.get_mut(&action.player_id).unwrap();
                let all_in_amount = *player_chips;
                *player_chips = 0;
                
                let current_player_bet = self.player_bets.get(&action.player_id).copied().unwrap_or(0);
                self.player_bets.insert(action.player_id, current_player_bet + all_in_amount);
                
                if current_player_bet + all_in_amount > self.current_bet {
                    self.current_bet = current_player_bet + all_in_amount;
                }
            }
            _ => {}
        }
        
        // Move to next player
        loop {
            self.current_player = (self.current_player + 1) % self.players.len();
            let player = &self.players[self.current_player];
            
            if !self.folded_players.contains(player) && self.chips.get(player).copied().unwrap_or(0) > 0 {
                break;
            }
            
            if self.current_player == self.dealer_position {
                // Completed full circle
                break;
            }
        }
        
        Ok(())
    }
    
    async fn advance_round(&mut self) -> Result<GameResult> {
        self.round += 1;
        
        // Deal new hand if needed
        if self.hands.is_empty() && self.players.len() >= 2 {
            self.deal_new_hand();
        }
        
        // Simulate some betting action for demo
        if !self.players.is_empty() && self.betting_round != BettingRound::Showdown {
            let current = &self.players[self.current_player];
            
            if !self.folded_players.contains(current) {
                // Simple AI logic
                let action = if rand::random::<f32>() > 0.7 {
                    PlayerAction {
                        player_id: current.clone(),
                        action_type: "fold".to_string(),
                        data: None,
                    }
                } else if rand::random::<f32>() > 0.5 {
                    let to_call = self.current_bet - self.player_bets.get(current).copied().unwrap_or(0);
                    PlayerAction {
                        player_id: current.clone(),
                        action_type: if to_call > 0 { format!("call:{}", to_call) } else { "check".to_string() },
                        data: None,
                    }
                } else {
                    PlayerAction {
                        player_id: current.clone(),
                        action_type: format!("raise:{}", self.current_bet + BIG_BLIND),
                        data: None,
                    }
                };
                
                self.process_action(action).await?;
            }
        }
        
        // Check if betting round is complete
        if self.is_betting_complete() {
            if self.betting_round == BettingRound::Showdown || 
               self.players.iter().filter(|p| !self.folded_players.contains(*p)).count() <= 1 {
                // Determine winner
                let (winner, hand_name, showdown) = self.determine_winner();
                
                // Award pot
                *self.chips.get_mut(&winner).unwrap() += self.pot;
                
                self.hand_history.push(HandResult {
                    winner: winner.clone(),
                    winning_hand: hand_name,
                    pot_size: self.pot,
                    showdown,
                    bluff_success: !showdown && self.pot > 100,
                });
                
                // Reset for next hand
                self.dealer_position = (self.dealer_position + 1) % self.players.len();
                self.deal_new_hand();
            } else {
                self.advance_betting_round();
            }
        }
        
        let active_players = self.players.iter()
            .filter(|p| self.chips.get(*p).copied().unwrap_or(0) > 0)
            .count();
        
        let round_winners = if active_players == 1 {
            vec![self.players.iter()
                .find(|p| self.chips.get(*p).copied().unwrap_or(0) > 0)
                .unwrap()
                .clone()]
        } else {
            vec![]
        };
        
        let emergence = self.detect_emergence();
        
        Ok(GameResult {
            round: self.round,
            scores: self.chips.clone(),
            round_winners,
            is_final_round: self.round >= self.max_rounds || active_players <= 1,
            emergence_metrics: emergence,
            special_data: Some(serde_json::json!({
                "pot": self.pot,
                "betting_round": self.betting_round,
                "hand_history_length": self.hand_history.len(),
            })),
        })
    }
    
    fn add_player(&mut self, player_id: String) -> Result<()> {
        if !self.players.contains(&player_id) && self.players.len() < 9 {
            self.players.push(player_id.clone());
            self.chips.insert(player_id, STARTING_CHIPS);
        }
        Ok(())
    }
    
    fn get_game_id(&self) -> Uuid {
        self.id
    }
}