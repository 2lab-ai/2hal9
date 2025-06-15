use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;
use rand::Rng;

pub struct TrustFall {
    trust_scores: HashMap<(String, String), f32>,
    reputation: HashMap<String, f32>,
    fall_heights: HashMap<String, u32>,
    catch_success: HashMap<String, u32>,
    catch_failures: HashMap<String, u32>,
    betrayal_history: Vec<BetrayalEvent>,
    alliance_networks: HashMap<String, Vec<String>>,
    player_health: HashMap<String, i32>,
    risk_tolerance: HashMap<String, f32>,
    trust_tokens: HashMap<String, i32>,
    current_round_falls: Vec<FallEvent>,
}

#[derive(Debug, Clone)]
struct BetrayalEvent {
    round: u32,
    betrayer: String,
    victim: String,
    fall_height: u32,
    damage: i32,
    reputation_loss: f32,
}

#[derive(Debug, Clone)]
struct FallEvent {
    faller: String,
    catchers: Vec<String>,
    height: u32,
    success: bool,
    trust_gained: f32,
}

impl Default for TrustFall {
    fn default() -> Self {
        Self::new()
    }
}

impl TrustFall {
    pub fn new() -> Self {
        Self {
            trust_scores: HashMap::new(),
            reputation: HashMap::new(),
            fall_heights: HashMap::new(),
            catch_success: HashMap::new(),
            catch_failures: HashMap::new(),
            betrayal_history: Vec::new(),
            alliance_networks: HashMap::new(),
            player_health: HashMap::new(),
            risk_tolerance: HashMap::new(),
            trust_tokens: HashMap::new(),
            current_round_falls: Vec::new(),
        }
    }
    
    fn initialize_player(&mut self, player_id: &str) {
        self.reputation.insert(player_id.to_string(), 0.5);
        self.fall_heights.insert(player_id.to_string(), 0);
        self.catch_success.insert(player_id.to_string(), 0);
        self.catch_failures.insert(player_id.to_string(), 0);
        self.player_health.insert(player_id.to_string(), 100);
        self.risk_tolerance.insert(player_id.to_string(), 0.5);
        self.trust_tokens.insert(player_id.to_string(), 3);
    }
    
    fn calculate_catch_probability(&self, catchers: &[String], height: u32) -> f32 {
        if catchers.is_empty() {
            return 0.0;
        }
        
        // Base probability depends on number of catchers
        let base_prob = match catchers.len() {
            1 => 0.6,
            2 => 0.85,
            3 => 0.95,
            _ => 0.98,
        };
        
        // Adjust for height (higher = harder)
        let height_penalty = (height as f32 / 100.0).min(0.5);
        
        // Adjust for catcher reputation
        let avg_reputation: f32 = catchers.iter()
            .map(|c| self.reputation.get(c).copied().unwrap_or(0.5))
            .sum::<f32>() / catchers.len() as f32;
        
        (base_prob * avg_reputation - height_penalty).clamp(0.1, 0.95)
    }
    
    fn calculate_damage(&self, height: u32) -> i32 {
        // Exponential damage scaling with height
        let base_damage = 10;
        let height_multiplier = (height as f32 / 10.0).powf(1.5);
        (base_damage as f32 * height_multiplier) as i32
    }
    
    fn update_trust_scores(&mut self, faller: &str, catchers: &[String], success: bool, height: u32) {
        let trust_change = if success {
            (height as f32 / 20.0).min(0.3)
        } else {
            -(height as f32 / 10.0).min(-0.5)
        };
        
        for catcher in catchers {
            let key = (faller.to_string(), catcher.to_string());
            let current = self.trust_scores.get(&key).copied().unwrap_or(0.5);
            self.trust_scores.insert(key, (current + trust_change).clamp(0.0, 1.0));
            
            // Update reputation
            if success {
                let rep = self.reputation.get_mut(catcher).unwrap();
                *rep = (*rep + 0.05).min(1.0);
                
                *self.catch_success.entry(catcher.clone()).or_insert(0) += 1;
            } else {
                let rep = self.reputation.get_mut(catcher).unwrap();
                *rep = (*rep - 0.15).max(0.0);
                
                *self.catch_failures.entry(catcher.clone()).or_insert(0) += 1;
            }
        }
    }
    
    fn form_alliance(&mut self, player1: &str, player2: &str) {
        let allies1 = self.alliance_networks.entry(player1.to_string()).or_insert_with(Vec::new);
        if !allies1.contains(&player2.to_string()) {
            allies1.push(player2.to_string());
        }
        
        let allies2 = self.alliance_networks.entry(player2.to_string()).or_insert_with(Vec::new);
        if !allies2.contains(&player1.to_string()) {
            allies2.push(player1.to_string());
        }
        
        // Boost mutual trust
        let key1 = (player1.to_string(), player2.to_string());
        let key2 = (player2.to_string(), player1.to_string());
        
        *self.trust_scores.entry(key1).or_insert(0.5) += 0.1;
        *self.trust_scores.entry(key2).or_insert(0.5) += 0.1;
    }
    
    fn detect_trust_emergence(&self, state: &GameState) -> Option<EmergenceEvent> {
        if state.round < 10 {
            return None;
        }
        
        // Calculate collective trust network strength
        let collective_trust: f32 = self.trust_scores.iter()
            .filter(|((p1, p2), _)| p1.starts_with("collective_") && p2.starts_with("collective_"))
            .map(|(_, &score)| score)
            .sum::<f32>() / self.trust_scores.len().max(1) as f32;
        
        let sota_trust: f32 = self.trust_scores.iter()
            .filter(|((p1, p2), _)| p1.starts_with("sota_") && p2.starts_with("sota_"))
            .map(|(_, &score)| score)
            .sum::<f32>() / self.trust_scores.len().max(1) as f32;
        
        // Check for high-trust networks
        if collective_trust > 0.8 && collective_trust > sota_trust * 1.5 {
            Some(EmergenceEvent {
                round: state.round,
                event_type: "trust_network".to_string(),
                description: "Collective developed strong trust networks enabling riskier strategies".to_string(),
                emergence_score: collective_trust,
            })
        } else {
            None
        }
    }
}

#[async_trait]
impl Game for TrustFall {
    async fn initialize(&mut self, _config: GameConfig) -> anyhow::Result<GameState> {
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::TrustFall,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("game_rules".to_string(), serde_json::json!({
                    "actions": {
                        "fall": "Trust others to catch you (specify height and catchers)",
                        "catch": "Agree to catch someone (honor or betray)",
                        "alliance": "Form trust alliance with another player",
                        "investigate": "Learn about another player's reputation",
                        "guarantee": "Use trust token to guarantee a catch"
                    },
                    "scoring": {
                        "successful_fall": "Height * catcher_count * 10 points",
                        "successful_catch": "Height * 5 points",
                        "betrayal": "-Height * 20 points + reputation loss"
                    },
                    "mechanics": {
                        "trust": "Build trust through successful catches",
                        "reputation": "Public score affecting catch probability",
                        "health": "Take damage from failed falls"
                    }
                }));
                meta.insert("trust_tokens".to_string(), serde_json::json!(3));
                meta
            },
        })
    }
    
    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        // Initialize new players
        for player_id in actions.keys() {
            if !self.reputation.contains_key(player_id) {
                self.initialize_player(player_id);
            }
        }
        
        self.current_round_falls.clear();
        let mut scores_delta = HashMap::new();
        let mut special_events = vec![];
        let mut eliminated_players = Vec::new();
        
        // First pass: collect fall requests and agreements
        let mut fall_requests: HashMap<String, (u32, Vec<String>, bool)> = HashMap::new();
        let mut catch_agreements: HashMap<String, Vec<String>> = HashMap::new();
        
        for (player_id, action) in &actions {
            if self.player_health.get(player_id).copied().unwrap_or(0) <= 0 {
                continue;
            }
            
            match action.action_type.as_str() {
                "fall" => {
                    if let (Some(height), Some(catchers)) = (
                        action.data.get("height").and_then(|v| v.as_u64()).map(|h| h as u32),
                        action.data.get("catchers").and_then(|v| v.as_array())
                    ) {
                        let catcher_ids: Vec<String> = catchers.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                        
                        let guaranteed = action.data.get("use_token").and_then(|v| v.as_bool()).unwrap_or(false);
                        
                        if guaranteed && self.trust_tokens.get(player_id).copied().unwrap_or(0) > 0 {
                            *self.trust_tokens.get_mut(player_id).unwrap() -= 1;
                        }
                        
                        fall_requests.insert(player_id.clone(), (height, catcher_ids, guaranteed));
                    }
                }
                "catch" => {
                    if let Some(faller) = action.data.get("faller").and_then(|v| v.as_str()) {
                        catch_agreements.entry(faller.to_string())
                            .or_insert_with(Vec::new)
                            .push(player_id.clone());
                    }
                }
                "alliance" => {
                    if let Some(target) = action.data.get("target").and_then(|v| v.as_str()) {
                        self.form_alliance(player_id, target);
                        special_events.push(format!("🤝 {} and {} formed an alliance", player_id, target));
                        scores_delta.insert(player_id.clone(), 5);
                    }
                }
                "investigate" => {
                    if let Some(target) = action.data.get("target").and_then(|v| v.as_str()) {
                        let rep = self.reputation.get(target).copied().unwrap_or(0.5);
                        let success_rate = self.catch_success.get(target).copied().unwrap_or(0);
                        let failure_rate = self.catch_failures.get(target).copied().unwrap_or(0);
                        
                        special_events.push(format!("🔍 {} investigated {}: reputation {:.2}, {} catches, {} drops",
                            player_id, target, rep, success_rate, failure_rate));
                    }
                }
                _ => {}
            }
        }
        
        // Process falls
        for (faller, (height, requested_catchers, guaranteed)) in fall_requests {
            // Determine actual catchers (those who agreed)
            let actual_catchers: Vec<String> = requested_catchers.iter()
                .filter(|c| {
                    catch_agreements.get(*c)
                        .map(|fallers| fallers.contains(&faller))
                        .unwrap_or(false)
                })
                .cloned()
                .collect();
            
            if actual_catchers.is_empty() && !guaranteed {
                // No one agreed to catch - automatic fail
                let damage = self.calculate_damage(height);
                if let Some(health) = self.player_health.get_mut(&faller) {
                    *health = (*health - damage).max(0);
                    if *health == 0 {
                        eliminated_players.push(faller.clone());
                    }
                }
                
                special_events.push(format!("💔 {} fell from height {} with no catchers! -{} HP", 
                    faller, height, damage));
                scores_delta.insert(faller.clone(), -(height as i32 * 10));
                
                continue;
            }
            
            // Calculate success probability
            let catch_prob = if guaranteed {
                1.0 // Trust token guarantees success
            } else {
                self.calculate_catch_probability(&actual_catchers, height)
            };
            
            let mut rng = rand::thread_rng();
            let success = rng.gen::<f32>() < catch_prob;
            
            if success {
                // Successful catch
                let points = height as i32 * actual_catchers.len() as i32 * 10;
                scores_delta.insert(faller.clone(), points);
                
                // Reward catchers
                for catcher in &actual_catchers {
                    *scores_delta.entry(catcher.clone()).or_insert(0) += height as i32 * 5;
                }
                
                special_events.push(format!("✓ {} successfully caught by {} from height {}! +{} points",
                    faller, actual_catchers.join(", "), height, points));
                
                // Update fall height record
                let current_max = self.fall_heights.get(&faller).copied().unwrap_or(0);
                if height > current_max {
                    self.fall_heights.insert(faller.clone(), height);
                }
            } else {
                // Failed catch - betrayal or accident
                let damage = self.calculate_damage(height);
                if let Some(health) = self.player_health.get_mut(&faller) {
                    *health = (*health - damage).max(0);
                    if *health == 0 {
                        eliminated_players.push(faller.clone());
                    }
                }
                
                // Penalize catchers who failed
                for catcher in &actual_catchers {
                    *scores_delta.entry(catcher.clone()).or_insert(0) -= height as i32 * 20;
                    
                    // Record betrayal
                    self.betrayal_history.push(BetrayalEvent {
                        round: state.round + 1,
                        betrayer: catcher.clone(),
                        victim: faller.clone(),
                        fall_height: height,
                        damage,
                        reputation_loss: 0.15,
                    });
                }
                
                special_events.push(format!("💀 {} was dropped by {} from height {}! -{} HP",
                    faller, actual_catchers.join(", "), height, damage));
                scores_delta.insert(faller.clone(), -(height as i32 * 20));
            }
            
            // Update trust scores
            self.update_trust_scores(&faller, &actual_catchers, success, height);
            
            // Record fall event
            self.current_round_falls.push(FallEvent {
                faller: faller.clone(),
                catchers: actual_catchers,
                height,
                success,
                trust_gained: if success { height as f32 / 20.0 } else { 0.0 },
            });
        }
        
        // Award trust tokens to high-reputation players
        for (player, &reputation) in &self.reputation {
            if reputation >= 0.9 && state.round % 10 == 0 {
                *self.trust_tokens.get_mut(player).unwrap() += 1;
                special_events.push(format!("🎖️ {} earned a trust token for high reputation!", player));
            }
        }
        
        // Check for emergence
        let emergence_event = self.detect_trust_emergence(state);
        if let Some(event) = &emergence_event {
            special_events.push(event.description.clone());
        }
        
        // Determine winners (highest trust network value)
        let winners: Vec<String> = self.reputation.iter()
            .filter(|(id, &rep)| rep >= 0.8 && self.player_health.get(*id).copied().unwrap_or(0) > 0)
            .take(3)
            .map(|(id, _)| id.clone())
            .collect();
        
        Ok(RoundResult {
            round: state.round + 1,
            actions: actions.clone(),
            outcome: Outcome {
                winners,
                losers: eliminated_players,
                special_events,
                emergence_detected: emergence_event.is_some(),
            },
            scores_delta,
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn is_game_over(&self, state: &GameState) -> bool {
        let alive_count = self.player_health.values().filter(|&&h| h > 0).count();
        
        alive_count <= 1 || state.round >= 50 || 
        self.reputation.values().any(|&rep| rep >= 1.0)
    }
    
    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        // Winner has highest combination of score and reputation
        let winner = self.player_health.iter()
            .filter(|(_, &health)| health > 0)
            .map(|(id, _)| {
                let score = state.scores.get(id).copied().unwrap_or(0);
                let reputation = self.reputation.get(id).copied().unwrap_or(0.5);
                let trust_value: f32 = self.trust_scores.iter()
                    .filter(|((p1, _), _)| p1 == id)
                    .map(|(_, &score)| score)
                    .sum();
                
                (id.clone(), score as f32 + reputation * 1000.0 + trust_value * 100.0)
            })
            .max_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap())
            .map(|(id, _)| id)
            .unwrap_or_else(|| "No winner".to_string());
        
        // Calculate final scores with bonuses
        let mut final_scores = state.scores.clone();
        
        for (player, &reputation) in &self.reputation {
            // Reputation bonus
            *final_scores.entry(player.clone()).or_insert(0) += (reputation * 500.0) as i32;
            
            // Trust network bonus
            let trust_value: f32 = self.trust_scores.iter()
                .filter(|((p1, _), _)| p1 == player)
                .map(|(_, &score)| score)
                .sum();
            *final_scores.entry(player.clone()).or_insert(0) += (trust_value * 100.0) as i32;
            
            // Maximum height bonus
            let max_height = self.fall_heights.get(player).copied().unwrap_or(0);
            *final_scores.entry(player.clone()).or_insert(0) += max_height as i32 * 10;
        }
        
        // Collect emergence events
        let emergence_events: Vec<EmergenceEvent> = state.history.iter()
            .enumerate()
            .filter_map(|(i, round_result)| {
                if round_result.outcome.emergence_detected {
                    Some(EmergenceEvent {
                        round: i as u32,
                        event_type: "trust_network".to_string(),
                        description: "Complex trust networks and reputation systems emerged".to_string(),
                        emergence_score: 0.95,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        let emergence_frequency = emergence_events.len() as f32 / state.round.max(1) as f32;
        
        // Calculate trust analytics
        let avg_trust = self.trust_scores.values().sum::<f32>() / self.trust_scores.len().max(1) as f32;
        let betrayal_rate = self.betrayal_history.len() as f32 / self.current_round_falls.len().max(1) as f32;
        
        let collective_reputation = self.reputation.iter()
            .filter(|(id, _)| id.starts_with("collective_"))
            .map(|(_, &rep)| rep)
            .sum::<f32>() / self.reputation.len().max(1) as f32;
        let sota_reputation = self.reputation.iter()
            .filter(|(id, _)| id.starts_with("sota_"))
            .map(|(_, &rep)| rep)
            .sum::<f32>() / self.reputation.len().max(1) as f32;
        
        GameResult {
            game_id: state.game_id,
            winner,
            final_scores,
            total_rounds: state.round,
            emergence_events,
            analytics: GameAnalytics {
                collective_coordination_score: avg_trust,
                decision_diversity_index: 1.0 - betrayal_rate,
                strategic_depth: 0.95,
                emergence_frequency,
                performance_differential: collective_reputation - sota_reputation,
            },
        }
    }
}