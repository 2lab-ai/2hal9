use super::*;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use rand::Rng;

pub struct ByzantineGenerals {
    n_generals: usize,
    n_traitors: usize,
    traitor_ids: HashSet<String>,
}

impl Default for ByzantineGenerals {
    fn default() -> Self {
        Self {
            n_generals: 7,
            n_traitors: 2,
            traitor_ids: HashSet::new(),
        }
    }
}

impl ByzantineGenerals {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn initialize_traitors(&mut self, players: &[String]) {
        let mut rng = rand::thread_rng();
        let mut available_players = players.to_vec();
        
        // Randomly select traitors
        for _ in 0..self.n_traitors.min(players.len() / 3) {
            if let Some(index) = (0..available_players.len()).nth(rng.gen_range(0..available_players.len())) {
                self.traitor_ids.insert(available_players.remove(index));
            }
        }
    }
    
    fn is_traitor(&self, player_id: &str) -> bool {
        self.traitor_ids.contains(player_id)
    }
    
    pub fn verify_consensus(&self, messages: &HashMap<String, Vec<Message>>) -> ConsensusResult {
        let mut attack_votes = 0;
        let mut retreat_votes = 0;
        let mut conflicting_messages = 0;
        
        // Count votes from non-traitors
        for (sender, msgs) in messages {
            if !self.is_traitor(sender) {
                let mut sender_votes_attack = 0;
                let mut sender_votes_retreat = 0;
                
                for msg in msgs {
                    match msg.decision {
                        Decision::Attack => sender_votes_attack += 1,
                        Decision::Retreat => sender_votes_retreat += 1,
                    }
                }
                
                // Check for conflicting messages from same sender
                if sender_votes_attack > 0 && sender_votes_retreat > 0 {
                    conflicting_messages += 1;
                } else if sender_votes_attack > sender_votes_retreat {
                    attack_votes += 1;
                } else {
                    retreat_votes += 1;
                }
            }
        }
        
        let honest_generals = self.n_generals - self.n_traitors;
        let consensus_threshold = honest_generals.div_ceil(2);
        
        ConsensusResult {
            consensus_reached: attack_votes >= consensus_threshold || retreat_votes >= consensus_threshold,
            decision: if attack_votes > retreat_votes { Decision::Attack } else { Decision::Retreat },
            attack_count: attack_votes,
            retreat_count: retreat_votes,
            conflicting_count: conflicting_messages,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Decision {
    Attack,
    Retreat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub decision: Decision,
    pub round: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub consensus_reached: bool,
    pub decision: Decision,
    pub attack_count: usize,
    pub retreat_count: usize,
    pub conflicting_count: usize,
}

#[async_trait]
impl Game for ByzantineGenerals {
    async fn initialize(&mut self, config: GameConfig) -> anyhow::Result<GameState> {
        // Extract player count from config
        if let Some(player_count) = config.special_rules.get("n_generals") {
            if let Ok(n) = player_count.parse::<usize>() {
                self.n_generals = n;
                self.n_traitors = (n - 1) / 3; // Byzantine fault tolerance
            }
        }
        
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::ByzantineGenerals,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("n_generals".to_string(), serde_json::json!(self.n_generals));
                meta.insert("n_traitors".to_string(), serde_json::json!(self.n_traitors));
                meta.insert("consensus_phase".to_string(), serde_json::json!("initializing"));
                meta
            },
        })
    }
    
    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        // Initialize traitors on first round
        if state.round == 0 && self.traitor_ids.is_empty() {
            let players: Vec<String> = actions.keys().cloned().collect();
            self.initialize_traitors(&players);
        }
        
        // Process messages from each general
        let mut all_messages: HashMap<String, Vec<Message>> = HashMap::new();
        let mut scores_delta = HashMap::new();
        
        for (player_id, action) in &actions {
            // Parse messages from action data
            if let Some(messages) = action.data.get("messages").and_then(|m| m.as_array()) {
                let mut player_messages = vec![];
                
                for msg in messages {
                    if let (Some(to), Some(decision_str)) = 
                        (msg.get("to").and_then(|t| t.as_str()),
                         msg.get("decision").and_then(|d| d.as_str())) {
                        
                        let decision = match decision_str {
                            "attack" => Decision::Attack,
                            _ => Decision::Retreat,
                        };
                        
                        // Traitors can send conflicting messages
                        let actual_decision = if self.is_traitor(player_id) && rand::thread_rng().gen_bool(0.5) {
                            match decision {
                                Decision::Attack => Decision::Retreat,
                                Decision::Retreat => Decision::Attack,
                            }
                        } else {
                            decision
                        };
                        
                        player_messages.push(Message {
                            from: player_id.clone(),
                            to: to.to_string(),
                            decision: actual_decision,
                            round: state.round,
                        });
                    }
                }
                
                all_messages.insert(player_id.clone(), player_messages);
            }
        }
        
        // Verify consensus
        let consensus = self.verify_consensus(&all_messages);
        
        // Award points based on consensus achievement
        let mut winners = vec![];
        let mut losers = vec![];
        
        if consensus.consensus_reached {
            // Honest generals who participated win
            for player_id in actions.keys() {
                if !self.is_traitor(player_id) {
                    winners.push(player_id.clone());
                    scores_delta.insert(player_id.clone(), 10);
                } else {
                    // Traitors lose when consensus is reached
                    losers.push(player_id.clone());
                    scores_delta.insert(player_id.clone(), -5);
                }
            }
        } else {
            // Traitors win when consensus fails
            for player_id in actions.keys() {
                if self.is_traitor(player_id) {
                    winners.push(player_id.clone());
                    scores_delta.insert(player_id.clone(), 15);
                } else {
                    losers.push(player_id.clone());
                    scores_delta.insert(player_id.clone(), -3);
                }
            }
        }
        
        // Check for collective emergence
        let emergence_detected = self.detect_emergence(&all_messages, &consensus);
        
        Ok(RoundResult {
            round: state.round + 1,
            actions: actions.clone(),
            outcome: Outcome {
                winners,
                losers,
                special_events: if emergence_detected {
                    vec!["Byzantine resistance emerged!".to_string()]
                } else if consensus.consensus_reached {
                    vec![format!("Consensus reached: {:?}", consensus.decision)]
                } else {
                    vec!["Consensus failed - traitors win".to_string()]
                },
                emergence_detected,
            },
            scores_delta,
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn is_game_over(&self, state: &GameState) -> bool {
        state.round >= 10 || 
        state.scores.values().any(|&score| score >= 100 || score <= -50)
    }
    
    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        // Determine winner: highest score among honest generals
        let mut best_honest_score = -1000;
        let mut best_honest_player = "No winner".to_string();
        
        for (player_id, &score) in &state.scores {
            if !self.is_traitor(player_id) && score > best_honest_score {
                best_honest_score = score;
                best_honest_player = player_id.clone();
            }
        }
        
        // Check if traitors collectively won
        let traitor_total: i32 = state.scores.iter()
            .filter(|(id, _)| self.is_traitor(id))
            .map(|(_, &score)| score)
            .sum();
        
        let honest_total: i32 = state.scores.iter()
            .filter(|(id, _)| !self.is_traitor(id))
            .map(|(_, &score)| score)
            .sum();
        
        let winner = if traitor_total > honest_total {
            "Traitors (Byzantine)".to_string()
        } else {
            best_honest_player
        };
        
        // Count emergence events
        let emergence_events: Vec<EmergenceEvent> = state.history.iter()
            .enumerate()
            .filter_map(|(i, round_result)| {
                if round_result.outcome.emergence_detected {
                    Some(EmergenceEvent {
                        round: i as u32,
                        event_type: "byzantine_resistance".to_string(),
                        description: "Collective achieved consensus despite Byzantine interference".to_string(),
                        emergence_score: 0.9,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        let emergence_frequency = emergence_events.len() as f32 / state.round.max(1) as f32;
        
        GameResult {
            game_id: state.game_id,
            winner,
            final_scores: state.scores.clone(),
            total_rounds: state.round,
            emergence_events,
            analytics: GameAnalytics {
                collective_coordination_score: self.calculate_coordination_score(state),
                decision_diversity_index: 0.0, // TODO
                strategic_depth: state.round as f32 / 10.0,
                emergence_frequency,
                performance_differential: (honest_total - traitor_total) as f32,
            },
        }
    }
}

impl ByzantineGenerals {
    fn detect_emergence(&self, messages: &HashMap<String, Vec<Message>>, consensus: &ConsensusResult) -> bool {
        // Emergence detected when honest nodes achieve consensus despite high Byzantine interference
        if consensus.consensus_reached && self.n_traitors > 0 {
            let total_messages: usize = messages.values().map(|v| v.len()).sum();
            let expected_messages = self.n_generals * (self.n_generals - 1); // Each general messages all others
            
            // If we achieved consensus with significant message loss or corruption
            if total_messages < expected_messages * 3 / 4 || consensus.conflicting_count > 0 {
                return true;
            }
        }
        false
    }
    
    fn calculate_coordination_score(&self, state: &GameState) -> f32 {
        // Calculate how well honest nodes coordinated
        let mut consensus_rounds = 0;
        
        for round in &state.history {
            if round.outcome.special_events.iter().any(|e| e.contains("Consensus reached")) {
                consensus_rounds += 1;
            }
        }
        
        consensus_rounds as f32 / state.round.max(1) as f32
    }
}