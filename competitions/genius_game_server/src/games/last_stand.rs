use super::*;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;
use rand::Rng;

pub struct LastStand {
    player_health: HashMap<String, i32>,
    player_resources: HashMap<String, Resources>,
    fortifications: HashMap<String, Fortification>,
    wave_number: u32,
    threat_level: f32,
    enemies_spawned: Vec<Enemy>,
    player_positions: HashMap<String, Position>,
    shared_resources: Resources,
    cooperation_matrix: HashMap<(String, String), f32>,
    elimination_history: Vec<EliminationEvent>,
    #[allow(dead_code)]
    survival_strategies: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
struct Resources {
    ammo: i32,
    materials: i32,
    medical: i32,
}

impl Resources {
    fn new() -> Self {
        Self {
            ammo: 100,
            materials: 50,
            medical: 30,
        }
    }
    
    #[allow(dead_code)]
    fn is_depleted(&self) -> bool {
        self.ammo <= 0 && self.materials <= 0 && self.medical <= 0
    }
}

#[derive(Debug, Clone)]
struct Fortification {
    health: i32,
    defense_rating: f32,
    #[allow(dead_code)]
    position: Position,
    #[allow(dead_code)]
    owner: String,
    #[allow(dead_code)]
    shared_with: Vec<String>,
}

#[derive(Debug, Clone)]
struct Enemy {
    health: i32,
    damage: i32,
    position: Position,
    #[allow(dead_code)]
    target: Option<String>,
    enemy_type: EnemyType,
}

#[derive(Debug, Clone)]
enum EnemyType {
    Basic,
    Fast,
    Tank,
    Swarm,
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
}

#[derive(Debug, Clone)]
struct EliminationEvent {
    round: u32,
    player: String,
    #[allow(dead_code)]
    reason: String,
    final_wave: u32,
}

impl Default for LastStand {
    fn default() -> Self {
        Self::new()
    }
}

impl LastStand {
    pub fn new() -> Self {
        Self {
            player_health: HashMap::new(),
            player_resources: HashMap::new(),
            fortifications: HashMap::new(),
            wave_number: 0,
            threat_level: 1.0,
            enemies_spawned: Vec::new(),
            player_positions: HashMap::new(),
            shared_resources: Resources::new(),
            cooperation_matrix: HashMap::new(),
            elimination_history: Vec::new(),
            survival_strategies: HashMap::new(),
        }
    }
    
    fn spawn_player(&mut self, player_id: &str) {
        let mut rng = rand::thread_rng();
        
        // Spawn in defensive positions
        let position = Position {
            x: rng.gen_range(40.0..60.0),
            y: rng.gen_range(40.0..60.0),
        };
        
        self.player_positions.insert(player_id.to_string(), position);
        self.player_health.insert(player_id.to_string(), 100);
        self.player_resources.insert(player_id.to_string(), Resources::new());
    }
    
    fn spawn_wave(&mut self) {
        self.wave_number += 1;
        self.enemies_spawned.clear();
        
        let mut rng = rand::thread_rng();
        let enemy_count = (self.wave_number as f32 * self.threat_level * 2.0) as usize;
        
        for _ in 0..enemy_count {
            let enemy_type = if self.wave_number < 5 {
                EnemyType::Basic
            } else if self.wave_number < 10 {
                match rng.gen_range(0..3) {
                    0 => EnemyType::Basic,
                    1 => EnemyType::Fast,
                    _ => EnemyType::Tank,
                }
            } else {
                match rng.gen_range(0..4) {
                    0 => EnemyType::Basic,
                    1 => EnemyType::Fast,
                    2 => EnemyType::Tank,
                    _ => EnemyType::Swarm,
                }
            };
            
            let (health, damage) = match enemy_type {
                EnemyType::Basic => (30, 10),
                EnemyType::Fast => (20, 15),
                EnemyType::Tank => (100, 20),
                EnemyType::Swarm => (10, 5),
            };
            
            // Spawn from edges
            let side = rng.gen_range(0..4);
            let position = match side {
                0 => Position { x: rng.gen_range(0.0..100.0), y: 0.0 },
                1 => Position { x: 100.0, y: rng.gen_range(0.0..100.0) },
                2 => Position { x: rng.gen_range(0.0..100.0), y: 100.0 },
                _ => Position { x: 0.0, y: rng.gen_range(0.0..100.0) },
            };
            
            self.enemies_spawned.push(Enemy {
                health: (health as f32 * self.threat_level) as i32,
                damage: (damage as f32 * self.threat_level.sqrt()) as i32,
                position,
                target: None,
                enemy_type,
            });
        }
    }
    
    fn process_combat(&mut self, round: u32) {
        let mut damage_to_players = HashMap::new();
        let mut eliminated_enemies = Vec::new();
        
        // Enemy attacks
        for (i, enemy) in self.enemies_spawned.iter().enumerate() {
            if enemy.health <= 0 {
                eliminated_enemies.push(i);
                continue;
            }
            
            // Find closest player or fortification
            let mut closest_target = None;
            let mut min_distance = f32::MAX;
            
            for (player_id, pos) in &self.player_positions {
                if self.player_health.get(player_id).copied().unwrap_or(0) > 0 {
                    let dist = enemy.position.distance_to(pos);
                    if dist < min_distance {
                        min_distance = dist;
                        closest_target = Some(player_id.clone());
                    }
                }
            }
            
            // Attack if in range
            if min_distance < 5.0 {
                if let Some(target) = closest_target {
                    // Check if fortification blocks
                    if let Some(fort) = self.fortifications.get(&target) {
                        let damage_reduction = fort.defense_rating;
                        let final_damage = (enemy.damage as f32 * (1.0 - damage_reduction)) as i32;
                        *damage_to_players.entry(target).or_insert(0) += final_damage;
                    } else {
                        *damage_to_players.entry(target).or_insert(0) += enemy.damage;
                    }
                }
            }
        }
        
        // Apply damage
        for (player_id, damage) in damage_to_players {
            if let Some(health) = self.player_health.get_mut(&player_id) {
                *health = (*health - damage).max(0);
                
                if *health == 0 {
                    self.elimination_history.push(EliminationEvent {
                        round,
                        player: player_id.clone(),
                        reason: "Overwhelmed by enemies".to_string(),
                        final_wave: self.wave_number,
                    });
                }
            }
        }
        
        // Remove dead enemies
        eliminated_enemies.sort_by(|a, b| b.cmp(a));
        for i in eliminated_enemies {
            self.enemies_spawned.remove(i);
        }
    }
    
    fn detect_cooperation_emergence(&self, state: &GameState) -> Option<EmergenceEvent> {
        if state.round < 10 {
            return None;
        }
        
        // Calculate average cooperation score
        let mut total_cooperation = 0.0;
        let mut cooperation_pairs = 0;
        
        for score in self.cooperation_matrix.values() {
            total_cooperation += score;
            cooperation_pairs += 1;
        }
        
        let avg_cooperation = if cooperation_pairs > 0 {
            total_cooperation / cooperation_pairs as f32
        } else {
            0.0
        };
        
        // Check collective vs sota cooperation
        let collective_cooperation: f32 = self.cooperation_matrix.iter()
            .filter(|((p1, p2), _)| p1.starts_with("collective_") && p2.starts_with("collective_"))
            .map(|(_, &score)| score)
            .sum::<f32>() / self.cooperation_matrix.len().max(1) as f32;
        
        if avg_cooperation > 0.7 && collective_cooperation > 0.8 {
            Some(EmergenceEvent {
                round: state.round,
                event_type: "resource_sharing_network".to_string(),
                description: "Players developed efficient resource sharing strategies".to_string(),
                emergence_score: avg_cooperation,
            })
        } else {
            None
        }
    }
}

#[async_trait]
impl Game for LastStand {
    async fn initialize(&mut self, _config: GameConfig) -> anyhow::Result<GameState> {
        self.shared_resources = Resources {
            ammo: 200,
            materials: 100,
            medical: 50,
        };
        
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::LastStand,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("wave_number".to_string(), serde_json::json!(0));
                meta.insert("threat_level".to_string(), serde_json::json!(1.0));
                meta.insert("game_rules".to_string(), serde_json::json!({
                    "actions": {
                        "shoot": "Attack enemies (costs ammo)",
                        "build": "Build/upgrade fortifications (costs materials)",
                        "heal": "Restore health (costs medical supplies)",
                        "share": "Share resources with another player",
                        "scavenge": "Search for additional resources"
                    },
                    "waves": "Survive increasingly difficult enemy waves",
                    "cooperation": "Share resources and fortifications",
                    "objective": "Survive as many waves as possible"
                }));
                meta.insert("shared_resources".to_string(), serde_json::json!({
                    "ammo": self.shared_resources.ammo,
                    "materials": self.shared_resources.materials,
                    "medical": self.shared_resources.medical
                }));
                meta
            },
        })
    }
    
    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        // Initialize new players
        for player_id in actions.keys() {
            if !self.player_health.contains_key(player_id) {
                self.spawn_player(player_id);
            }
        }
        
        // Spawn new wave every 5 rounds
        if state.round % 5 == 0 {
            self.spawn_wave();
        }
        
        let mut scores_delta = HashMap::new();
        let mut special_events = vec![];
        
        // Process player actions
        for (player_id, action) in &actions {
            if self.player_health.get(player_id).copied().unwrap_or(0) <= 0 {
                continue;
            }
            
            let resources = self.player_resources.get_mut(player_id).unwrap();
            
            match action.action_type.as_str() {
                "shoot" => {
                    if resources.ammo >= 10 {
                        resources.ammo -= 10;
                        
                        // Damage nearby enemies
                        let player_pos = self.player_positions.get(player_id).unwrap();
                        let mut kills = 0;
                        
                        for enemy in &mut self.enemies_spawned {
                            if enemy.position.distance_to(player_pos) < 20.0 {
                                enemy.health -= 50;
                                if enemy.health <= 0 {
                                    kills += 1;
                                }
                            }
                        }
                        
                        if kills > 0 {
                            scores_delta.insert(player_id.clone(), kills * 10);
                            special_events.push(format!("{} eliminated {} enemies!", player_id, kills));
                        }
                    }
                }
                "build" => {
                    if resources.materials >= 20 {
                        resources.materials -= 20;
                        
                        let player_pos = *self.player_positions.get(player_id).unwrap();
                        
                        if let Some(fort) = self.fortifications.get_mut(player_id) {
                            // Upgrade existing
                            fort.health += 50;
                            fort.defense_rating = (fort.defense_rating + 0.1).min(0.8);
                            special_events.push(format!("{} upgraded fortification", player_id));
                        } else {
                            // Build new
                            self.fortifications.insert(player_id.clone(), Fortification {
                                health: 100,
                                defense_rating: 0.3,
                                position: player_pos,
                                owner: player_id.clone(),
                                shared_with: vec![],
                            });
                            special_events.push(format!("{} built fortification", player_id));
                        }
                        
                        scores_delta.insert(player_id.clone(), 5);
                    }
                }
                "heal" => {
                    if resources.medical >= 10 {
                        resources.medical -= 10;
                        let health = self.player_health.get_mut(player_id).unwrap();
                        let healed = (*health + 30).min(100) - *health;
                        *health += healed;
                        
                        if healed > 0 {
                            special_events.push(format!("{} healed for {} HP", player_id, healed));
                        }
                    }
                }
                "share" => {
                    if let Some(target) = action.data.get("target").and_then(|v| v.as_str()) {
                        if let Some(resource_type) = action.data.get("resource").and_then(|v| v.as_str()) {
                            let amount = action.data.get("amount").and_then(|v| v.as_i64()).unwrap_or(10) as i32;
                            
                            let can_share = match resource_type {
                                "ammo" => resources.ammo >= amount,
                                "materials" => resources.materials >= amount,
                                "medical" => resources.medical >= amount,
                                _ => false,
                            };
                            
                            if can_share {
                                // Transfer resources
                                match resource_type {
                                    "ammo" => {
                                        resources.ammo -= amount;
                                        if let Some(target_res) = self.player_resources.get_mut(target) {
                                            target_res.ammo += amount;
                                        }
                                    }
                                    "materials" => {
                                        resources.materials -= amount;
                                        if let Some(target_res) = self.player_resources.get_mut(target) {
                                            target_res.materials += amount;
                                        }
                                    }
                                    "medical" => {
                                        resources.medical -= amount;
                                        if let Some(target_res) = self.player_resources.get_mut(target) {
                                            target_res.medical += amount;
                                        }
                                    }
                                    _ => {}
                                }
                                
                                // Update cooperation matrix
                                let key = (player_id.clone(), target.to_string());
                                *self.cooperation_matrix.entry(key).or_insert(0.0) += 0.1;
                                
                                special_events.push(format!("{} shared {} {} with {}", 
                                    player_id, amount, resource_type, target));
                                scores_delta.insert(player_id.clone(), 5);
                            }
                        }
                    }
                }
                "scavenge" => {
                    let mut rng = rand::thread_rng();
                    if rng.gen_bool(0.3) {
                        // Found resources
                        let found_ammo = rng.gen_range(0..20);
                        let found_materials = rng.gen_range(0..10);
                        let found_medical = rng.gen_range(0..5);
                        
                        resources.ammo += found_ammo;
                        resources.materials += found_materials;
                        resources.medical += found_medical;
                        
                        special_events.push(format!("{} found: {} ammo, {} materials, {} medical",
                            player_id, found_ammo, found_materials, found_medical));
                        scores_delta.insert(player_id.clone(), 3);
                    } else {
                        special_events.push(format!("{} found nothing while scavenging", player_id));
                    }
                }
                _ => {}
            }
        }
        
        // Move enemies towards players
        for enemy in &mut self.enemies_spawned {
            if let Some((_, closest_pos)) = self.player_positions.iter()
                .filter(|(id, _)| self.player_health.get(*id).copied().unwrap_or(0) > 0)
                .min_by_key(|(_, pos)| enemy.position.distance_to(pos) as i32) {
                
                let dx = closest_pos.x - enemy.position.x;
                let dy = closest_pos.y - enemy.position.y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                let speed = match enemy.enemy_type {
                    EnemyType::Fast => 3.0,
                    EnemyType::Tank => 1.0,
                    _ => 2.0,
                };
                
                if dist > 0.1 {
                    enemy.position.x += (dx / dist) * speed;
                    enemy.position.y += (dy / dist) * speed;
                }
            }
        }
        
        // Process combat
        self.process_combat(state.round + 1);
        
        // Award survival points
        let survivors: Vec<_> = self.player_health.iter()
            .filter(|(_, &health)| health > 0)
            .map(|(id, _)| id.clone())
            .collect();
        
        for survivor in &survivors {
            *scores_delta.entry(survivor.clone()).or_insert(0) += self.wave_number as i32;
        }
        
        // Check for emergence
        let emergence_event = self.detect_cooperation_emergence(state);
        if let Some(event) = &emergence_event {
            special_events.push(event.description.clone());
        }
        
        // Determine outcomes
        let losers: Vec<String> = self.elimination_history.iter()
            .filter(|e| e.round == state.round + 1)
            .map(|e| e.player.clone())
            .collect();
        
        if state.round % 5 == 0 {
            special_events.insert(0, format!("🌊 Wave {} incoming! Threat level: {:.1}", 
                self.wave_number, self.threat_level));
        }
        
        if self.enemies_spawned.is_empty() && self.wave_number > 0 {
            special_events.push("✓ Wave cleared!".to_string());
        }
        
        // Increase difficulty
        if state.round % 10 == 0 {
            self.threat_level *= 1.2;
            special_events.push(format!("⚠️ Threat level increased to {:.1}", self.threat_level));
        }
        
        Ok(RoundResult {
            round: state.round + 1,
            actions: actions.clone(),
            outcome: Outcome {
                winners: survivors.clone(),
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
        
        alive_count == 0 || state.round >= 100 || self.wave_number >= 20
    }
    
    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        // Winner survived the longest
        let winner = self.player_health.iter()
            .filter(|(_, &health)| health > 0)
            .max_by_key(|(id, _)| state.scores.get(*id).unwrap_or(&0))
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| {
                // Fallback to player who survived most waves
                self.elimination_history.iter()
                    .max_by_key(|e| e.final_wave)
                    .map(|e| e.player.clone())
                    .unwrap_or_else(|| "No winner".to_string())
            });
        
        // Final scores
        let mut final_scores = state.scores.clone();
        
        // Bonus for survivors
        for (player, &health) in &self.player_health {
            if health > 0 {
                *final_scores.entry(player.clone()).or_insert(0) += 1000 + (self.wave_number as i32 * 100);
            }
        }
        
        // Cooperation bonus
        for ((p1, p2), &score) in &self.cooperation_matrix {
            *final_scores.entry(p1.clone()).or_insert(0) += (score * 50.0) as i32;
            *final_scores.entry(p2.clone()).or_insert(0) += (score * 50.0) as i32;
        }
        
        // Collect emergence events
        let emergence_events: Vec<EmergenceEvent> = state.history.iter()
            .enumerate()
            .filter_map(|(i, round_result)| {
                if round_result.outcome.emergence_detected {
                    Some(EmergenceEvent {
                        round: i as u32,
                        event_type: "resource_sharing_network".to_string(),
                        description: "Cooperative survival strategies emerged".to_string(),
                        emergence_score: 0.9,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        let emergence_frequency = emergence_events.len() as f32 / state.round.max(1) as f32;
        
        // Calculate survival metrics
        let collective_survivors = self.player_health.iter()
            .filter(|(id, &health)| id.starts_with("collective_") && health > 0)
            .count() as f32;
        let sota_survivors = self.player_health.iter()
            .filter(|(id, &health)| id.starts_with("sota_") && health > 0)
            .count() as f32;
        
        let avg_cooperation = self.cooperation_matrix.values().sum::<f32>() / 
                             self.cooperation_matrix.len().max(1) as f32;
        
        GameResult {
            game_id: state.game_id,
            winner,
            final_scores,
            total_rounds: state.round,
            emergence_events,
            analytics: GameAnalytics {
                collective_coordination_score: avg_cooperation,
                decision_diversity_index: 0.75,
                strategic_depth: 0.9,
                emergence_frequency,
                performance_differential: collective_survivors - sota_survivors,
            },
        }
    }
}