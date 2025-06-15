use super::*;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use rand::seq::SliceRandom;

const STARTING_CHIPS: i32 = 1000;
const SMALL_BLIND: i32 = 10;
const BIG_BLIND: i32 = 20;

pub struct MiniHoldemGame {
    chips: HashMap<String, i32>,
    hands: HashMap<String, Hand>,
    community_cards: Vec<Card>,
    pot: i32,
    current_bet: i32,
    player_bets: HashMap<String, i32>,
    folded_players: HashSet<String>,
    active_players: Vec<String>,
    dealer_position: usize,
    current_player_idx: usize,
    betting_round: BettingRound,
    hand_history: Vec<HandResult>,
    deck: Vec<Card>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Card {
    rank: Rank,
    suit: Suit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Rank {
    Two = 2, Three = 3, Four = 4, Five = 5, Six = 6, Seven = 7,
    Eight = 8, Nine = 9, Ten = 10, Jack = 11, Queen = 12, King = 13, Ace = 14,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Suit {
    Hearts, Diamonds, Clubs, Spades,
}

#[derive(Debug, Clone)]
struct Hand {
    cards: Vec<Card>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BettingRound {
    PreFlop,
    Flop,
    Turn,
    River,
    Showdown,
}

#[derive(Debug, Clone)]
struct HandResult {
    winner: String,
    winning_hand: String,
    pot_size: i32,
    showdown: bool,
    bluff_success: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum HandRank {
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

impl Default for MiniHoldemGame {
    fn default() -> Self {
        Self::new()
    }
}

impl MiniHoldemGame {
    pub fn new() -> Self {
        let mut deck = Self::create_deck();
        deck.shuffle(&mut rand::thread_rng());
        
        Self {
            chips: HashMap::new(),
            hands: HashMap::new(),
            community_cards: Vec::new(),
            pot: 0,
            current_bet: 0,
            player_bets: HashMap::new(),
            folded_players: HashSet::new(),
            active_players: Vec::new(),
            dealer_position: 0,
            current_player_idx: 0,
            betting_round: BettingRound::PreFlop,
            hand_history: Vec::new(),
            deck,
        }
    }
    
    fn create_deck() -> Vec<Card> {
        let mut deck = Vec::new();
        let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
        let ranks = [
            Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven,
            Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
        ];
        
        for &suit in &suits {
            for &rank in &ranks {
                deck.push(Card { rank, suit });
            }
        }
        deck
    }
    
    fn deal_cards(&mut self) {
        // Reset deck and shuffle
        self.deck = Self::create_deck();
        self.deck.shuffle(&mut rand::thread_rng());
        
        // Deal 2 cards to each player
        for player in &self.active_players.clone() {
            let cards = vec![
                self.deck.pop().unwrap(),
                self.deck.pop().unwrap(),
            ];
            self.hands.insert(player.clone(), Hand { cards });
        }
        
        // Reset betting state
        self.community_cards.clear();
        self.pot = 0;
        self.current_bet = 0;
        self.player_bets.clear();
        self.folded_players.clear();
        self.betting_round = BettingRound::PreFlop;
        
        // Post blinds
        if self.active_players.len() >= 2 {
            let small_blind_idx = (self.dealer_position + 1) % self.active_players.len();
            let big_blind_idx = (self.dealer_position + 2) % self.active_players.len();
            
            let small_blind_player = self.active_players[small_blind_idx].clone();
            let big_blind_player = self.active_players[big_blind_idx].clone();
            
            self.player_bets.insert(small_blind_player.clone(), SMALL_BLIND);
            self.player_bets.insert(big_blind_player.clone(), BIG_BLIND);
            
            *self.chips.get_mut(&small_blind_player).unwrap() -= SMALL_BLIND;
            *self.chips.get_mut(&big_blind_player).unwrap() -= BIG_BLIND;
            
            self.pot = SMALL_BLIND + BIG_BLIND;
            self.current_bet = BIG_BLIND;
            self.current_player_idx = (big_blind_idx + 1) % self.active_players.len();
        }
    }
    
    fn process_action(&mut self, player_id: &str, action: &str, amount: i32) -> bool {
        if self.folded_players.contains(player_id) {
            return false;
        }
        
        let player_bet = self.player_bets.get(player_id).copied().unwrap_or(0);
        let to_call = self.current_bet - player_bet;
        
        match action {
            "fold" => {
                self.folded_players.insert(player_id.to_string());
                return true;
            }
            "call" => {
                if to_call > 0 {
                    let chips = self.chips.get_mut(player_id).unwrap();
                    let actual_bet = to_call.min(*chips);
                    *chips -= actual_bet;
                    self.pot += actual_bet;
                    *self.player_bets.entry(player_id.to_string()).or_insert(0) += actual_bet;
                }
                return true;
            }
            "raise" => {
                let total_bet = self.current_bet + amount;
                let chips = self.chips.get_mut(player_id).unwrap();
                let bet_amount = total_bet - player_bet;
                
                if bet_amount <= *chips && amount >= self.current_bet {
                    *chips -= bet_amount;
                    self.pot += bet_amount;
                    self.current_bet = total_bet;
                    *self.player_bets.entry(player_id.to_string()).or_insert(0) = total_bet;
                    return true;
                }
            }
            "all-in" => {
                let chips = self.chips.get_mut(player_id).unwrap();
                let all_in_amount = *chips;
                *chips = 0;
                self.pot += all_in_amount;
                let total_bet = player_bet + all_in_amount;
                *self.player_bets.entry(player_id.to_string()).or_insert(0) = total_bet;
                if total_bet > self.current_bet {
                    self.current_bet = total_bet;
                }
                return true;
            }
            _ => {}
        }
        false
    }
    
    fn advance_betting_round(&mut self) {
        self.betting_round = match self.betting_round {
            BettingRound::PreFlop => {
                // Deal flop (3 cards)
                for _ in 0..3 {
                    self.community_cards.push(self.deck.pop().unwrap());
                }
                BettingRound::Flop
            }
            BettingRound::Flop => {
                // Deal turn (1 card)
                self.community_cards.push(self.deck.pop().unwrap());
                BettingRound::Turn
            }
            BettingRound::Turn => {
                // Deal river (1 card)
                self.community_cards.push(self.deck.pop().unwrap());
                BettingRound::River
            }
            BettingRound::River => BettingRound::Showdown,
            BettingRound::Showdown => BettingRound::Showdown,
        };
        
        // Reset betting for new round
        self.current_bet = 0;
        self.player_bets.clear();
        self.current_player_idx = (self.dealer_position + 1) % self.active_players.len();
    }
    
    fn evaluate_hand(&self, player_cards: &[Card], community_cards: &[Card]) -> (HandRank, Vec<Card>) {
        let mut all_cards = player_cards.to_vec();
        all_cards.extend(community_cards);
        
        // Find best 5-card combination
        let mut best_rank = HandRank::HighCard;
        let mut best_cards = vec![];
        
        // Generate all 5-card combinations
        for i in 0..all_cards.len() {
            for j in i+1..all_cards.len() {
                for k in j+1..all_cards.len() {
                    for l in k+1..all_cards.len() {
                        for m in l+1..all_cards.len() {
                            let hand = vec![
                                all_cards[i], all_cards[j], all_cards[k], 
                                all_cards[l], all_cards[m]
                            ];
                            let rank = self.classify_hand(&hand);
                            if rank > best_rank {
                                best_rank = rank;
                                best_cards = hand;
                            }
                        }
                    }
                }
            }
        }
        
        (best_rank, best_cards)
    }
    
    fn classify_hand(&self, cards: &[Card]) -> HandRank {
        let mut cards = cards.to_vec();
        cards.sort_by_key(|c| c.rank);
        
        let is_flush = cards.windows(2).all(|w| w[0].suit == w[1].suit);
        let is_straight = cards.windows(2).all(|w| w[1].rank as u8 == w[0].rank as u8 + 1) ||
                         (cards[0].rank == Rank::Two && cards[1].rank == Rank::Three && 
                          cards[2].rank == Rank::Four && cards[3].rank == Rank::Five && 
                          cards[4].rank == Rank::Ace); // Ace-low straight
        
        let mut rank_counts = HashMap::new();
        for card in &cards {
            *rank_counts.entry(card.rank).or_insert(0) += 1;
        }
        
        let mut counts: Vec<_> = rank_counts.values().copied().collect();
        counts.sort_by(|a, b| b.cmp(a));
        
        match (counts.as_slice(), is_flush, is_straight) {
            (_, true, true) => HandRank::StraightFlush,
            ([4, 1], _, _) => HandRank::FourOfAKind,
            ([3, 2], _, _) => HandRank::FullHouse,
            (_, true, _) => HandRank::Flush,
            (_, _, true) => HandRank::Straight,
            ([3, 1, 1], _, _) => HandRank::ThreeOfAKind,
            ([2, 2, 1], _, _) => HandRank::TwoPair,
            ([2, 1, 1, 1], _, _) => HandRank::Pair,
            _ => HandRank::HighCard,
        }
    }
    
    fn determine_winner(&self) -> (String, HandRank) {
        let mut best_player = String::new();
        let mut best_rank = HandRank::HighCard;
        let mut best_cards = vec![];
        
        for player in &self.active_players {
            if !self.folded_players.contains(player) {
                if let Some(hand) = self.hands.get(player) {
                    let (rank, cards) = self.evaluate_hand(&hand.cards, &self.community_cards);
                    if rank > best_rank || (rank == best_rank && self.compare_hands(&cards, &best_cards) > 0) {
                        best_rank = rank;
                        best_cards = cards;
                        best_player = player.clone();
                    }
                }
            }
        }
        
        (best_player, best_rank)
    }
    
    fn compare_hands(&self, hand1: &[Card], hand2: &[Card]) -> i32 {
        // Simple comparison by highest card
        let mut h1 = hand1.to_vec();
        let mut h2 = hand2.to_vec();
        h1.sort_by_key(|c| c.rank);
        h2.sort_by_key(|c| c.rank);
        
        for i in (0..5).rev() {
            if h1[i].rank > h2[i].rank {
                return 1;
            } else if h1[i].rank < h2[i].rank {
                return -1;
            }
        }
        0
    }
    
    fn detect_bluffing_emergence(&self, state: &GameState) -> Option<EmergenceEvent> {
        if self.hand_history.len() < 10 {
            return None;
        }
        
        // Analyze bluffing patterns
        let recent_hands = self.hand_history.iter().rev().take(10);
        let mut bluff_attempts = 0;
        let mut successful_bluffs = 0;
        
        for hand in recent_hands {
            if !hand.showdown {
                bluff_attempts += 1;
                if hand.bluff_success {
                    successful_bluffs += 1;
                }
            }
        }
        
        let bluff_rate = bluff_attempts as f32 / 10.0;
        let bluff_success_rate = if bluff_attempts > 0 {
            successful_bluffs as f32 / bluff_attempts as f32
        } else {
            0.0
        };
        
        if bluff_rate > 0.3 && bluff_success_rate > 0.5 {
            Some(EmergenceEvent {
                round: state.round,
                event_type: "poker_psychology".to_string(),
                description: "Players mastered bluffing and psychological warfare".to_string(),
                emergence_score: bluff_success_rate,
            })
        } else {
            None
        }
    }
}

#[async_trait]
impl Game for MiniHoldemGame {
    async fn initialize(&mut self, _config: GameConfig) -> anyhow::Result<GameState> {
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::MiniHoldem,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("starting_chips".to_string(), serde_json::json!(STARTING_CHIPS));
                meta.insert("small_blind".to_string(), serde_json::json!(SMALL_BLIND));
                meta.insert("big_blind".to_string(), serde_json::json!(BIG_BLIND));
                meta.insert("game_rules".to_string(), serde_json::json!({
                    "betting": "No-limit hold'em",
                    "hand_rankings": "Standard poker rankings",
                    "elimination": "Players eliminated when chips reach 0"
                }));
                meta
            },
        })
    }
    
    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        // Initialize new players
        for player_id in actions.keys() {
            if !self.chips.contains_key(player_id) {
                self.chips.insert(player_id.clone(), STARTING_CHIPS);
                self.active_players.push(player_id.clone());
            }
        }
        
        // Remove eliminated players
        self.active_players.retain(|p| self.chips.get(p).copied().unwrap_or(0) > 0);
        
        // Start new hand if needed
        if self.betting_round == BettingRound::Showdown || self.hands.is_empty() {
            if self.active_players.len() >= 2 {
                self.deal_cards();
            }
        }
        
        // Process betting actions
        let mut actions_processed = false;
        let mut round_complete = false;
        
        for (player_id, action) in &actions {
            if !self.active_players.contains(player_id) {
                continue;
            }
            
            let poker_action = action.action_type.as_str();
            let amount = action.data.as_i64().unwrap_or(0) as i32;
            
            if self.process_action(player_id, poker_action, amount) {
                actions_processed = true;
            }
        }
        
        // Check if betting round is complete
        let active_unfold = self.active_players.iter()
            .filter(|p| !self.folded_players.contains(*p))
            .count();
        
        if active_unfold <= 1 {
            round_complete = true;
        } else if actions_processed {
            // Check if all active players have matched the current bet
            let all_matched = self.active_players.iter()
                .filter(|p| !self.folded_players.contains(*p))
                .all(|p| {
                    self.player_bets.get(p).copied().unwrap_or(0) == self.current_bet ||
                    self.chips.get(p).copied().unwrap_or(0) == 0
                });
            
            if all_matched {
                round_complete = true;
            }
        }
        
        let mut winners = vec![];
        let mut scores_delta = HashMap::new();
        let mut special_events = vec![];
        
        if round_complete {
            if active_unfold == 1 || self.betting_round == BettingRound::River {
                // Showdown or last player wins
                let (winner, winning_rank) = if active_unfold == 1 {
                    let winner = self.active_players.iter()
                        .find(|p| !self.folded_players.contains(*p))
                        .cloned()
                        .unwrap_or_default();
                    (winner, HandRank::HighCard)
                } else {
                    self.determine_winner()
                };
                
                // Award pot to winner
                if !winner.is_empty() {
                    *self.chips.entry(winner.clone()).or_insert(0) += self.pot;
                    scores_delta.insert(winner.clone(), self.pot);
                    winners.push(winner.clone());
                    
                    let showdown = self.betting_round == BettingRound::River || 
                                  self.betting_round == BettingRound::Showdown;
                    
                    self.hand_history.push(HandResult {
                        winner: winner.clone(),
                        winning_hand: format!("{:?}", winning_rank),
                        pot_size: self.pot,
                        showdown,
                        bluff_success: !showdown && active_unfold == 1,
                    });
                    
                    special_events.push(format!("{} wins {} chips with {:?}", winner, self.pot, winning_rank));
                }
                
                // Start new hand
                self.dealer_position = (self.dealer_position + 1) % self.active_players.len();
                if self.active_players.len() >= 2 {
                    self.deal_cards();
                }
            } else {
                // Advance to next betting round
                self.advance_betting_round();
            }
        }
        
        // Check for emergence
        let emergence_event = self.detect_bluffing_emergence(state);
        if let Some(event) = &emergence_event {
            special_events.push(event.description.clone());
        }
        
        // Determine losers (players who lost chips this round)
        let losers: Vec<String> = scores_delta.iter()
            .filter(|(_, &delta)| delta < 0)
            .map(|(id, _)| id.clone())
            .collect();
        
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
        self.active_players.len() <= 1 || state.round >= 200
    }
    
    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        let winner = if self.active_players.len() == 1 {
            self.active_players[0].clone()
        } else {
            self.chips.iter()
                .max_by_key(|(_, chips)| *chips)
                .map(|(id, _)| id.clone())
                .unwrap_or_else(|| "No winner".to_string())
        };
        
        // Collect emergence events
        let emergence_events: Vec<EmergenceEvent> = state.history.iter()
            .enumerate()
            .filter_map(|(i, round_result)| {
                if round_result.outcome.emergence_detected {
                    Some(EmergenceEvent {
                        round: i as u32,
                        event_type: "poker_psychology".to_string(),
                        description: "Advanced poker psychology emerged".to_string(),
                        emergence_score: 0.8,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        let emergence_frequency = emergence_events.len() as f32 / state.round.max(1) as f32;
        
        // Analyze hand history
        let total_hands = self.hand_history.len() as f32;
        let showdown_hands = self.hand_history.iter()
            .filter(|h| h.showdown)
            .count() as f32;
        let showdown_rate = if total_hands > 0.0 {
            showdown_hands / total_hands
        } else {
            0.0
        };
        
        let bluff_success = self.hand_history.iter()
            .filter(|h| h.bluff_success)
            .count() as f32;
        let bluff_rate = if total_hands > 0.0 {
            bluff_success / total_hands
        } else {
            0.0
        };
        
        GameResult {
            game_id: state.game_id,
            winner,
            final_scores: self.chips.clone(),
            total_rounds: state.round,
            emergence_events,
            analytics: GameAnalytics {
                collective_coordination_score: 0.0, // Not applicable
                decision_diversity_index: showdown_rate, // Variety of play styles
                strategic_depth: 1.0 - showdown_rate, // More folding = deeper strategy
                emergence_frequency,
                performance_differential: bluff_rate, // Bluffing success as key metric
            },
        }
    }
}