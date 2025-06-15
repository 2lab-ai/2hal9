use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use super::{Game, GameConfig, GameState, GameType, Action, RoundResult, Outcome, GameResult, EmergenceEvent, GameAnalytics};

const INITIAL_HEALTH: i32 = 100;
const HUNGER_DAMAGE: i32 = 5;
const THIRST_DAMAGE: i32 = 8;
const COMBAT_DAMAGE: i32 = 25;
const RESOURCE_HEAL: i32 = 15;
const ALLIANCE_BONUS: i32 = 10;
const BETRAYAL_PENALTY: i32 = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HungerGamesGame {
    tributes: HashMap<String, TributeStatus>,
    alliances: HashMap<String, String>, // player -> alliance_id
    resources: HashMap<String, ResourceCache>,
    environment_events: Vec<EnvironmentEvent>,
    eliminated_tributes: Vec<String>,
    cornucopia_claimed: bool,
    max_rounds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TributeStatus {
    health: i32,
    hunger: i32,
    thirst: i32,
    items: Vec<Item>,
    kills: u32,
    position: Position,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Position {
    district: u32,
    location: Location,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
enum Location {
    Cornucopia,
    Forest,
    River,
    Mountain,
    Cave,
    Plains,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Item {
    Weapon(String),
    Food,
    Water,
    Medicine,
    Tool(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResourceCache {
    location: Location,
    items: Vec<Item>,
    discovered_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum EnvironmentEvent {
    Wildfire { location: Location },
    Flood { location: Location },
    ColdSnap,
    PoisonFog { location: Location },
    MuttAttack { location: Location },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum HGAction {
    Hunt { player: String, target: Option<String> },
    Gather { player: String },
    FormAlliance { player: String, with: String },
    BetrayAlliance { player: String },
    Hide { player: String },
    MoveTo { player: String, location: Location },
    UseItem { player: String, item: String },
}

impl Default for HungerGamesGame {
    fn default() -> Self {
        Self::new()
    }
}

impl HungerGamesGame {
    pub fn new() -> Self {
        Self {
            tributes: HashMap::new(),
            alliances: HashMap::new(),
            resources: Self::initialize_resources(),
            environment_events: Vec::new(),
            eliminated_tributes: Vec::new(),
            cornucopia_claimed: false,
            max_rounds: 100,
        }
    }

    fn initialize_resources() -> HashMap<String, ResourceCache> {
        let mut resources = HashMap::new();
        
        resources.insert("cornucopia".to_string(), ResourceCache {
            location: Location::Cornucopia,
            items: vec![
                Item::Weapon("Sword".to_string()),
                Item::Weapon("Bow".to_string()),
                Item::Medicine,
                Item::Food,
                Item::Water,
            ],
            discovered_by: Vec::new(),
        });
        
        resources.insert("forest_cache".to_string(), ResourceCache {
            location: Location::Forest,
            items: vec![Item::Food, Item::Tool("Trap".to_string())],
            discovered_by: Vec::new(),
        });
        
        resources.insert("river_cache".to_string(), ResourceCache {
            location: Location::River,
            items: vec![Item::Water, Item::Food],
            discovered_by: Vec::new(),
        });
        
        resources
    }

    fn spawn_tributes(&mut self, players: Vec<String>) {
        for (i, player) in players.into_iter().enumerate() {
            self.tributes.insert(player, TributeStatus {
                health: INITIAL_HEALTH,
                hunger: 50,
                thirst: 50,
                items: vec![],
                kills: 0,
                position: Position {
                    district: (i as u32 % 12) + 1,
                    location: Location::Plains,
                },
            });
        }
    }

    fn apply_survival_damage(&mut self) {
        let mut damage_events = Vec::new();
        
        for (player, status) in &self.tributes {
            if self.eliminated_tributes.contains(player) {
                continue;
            }
            
            let mut total_damage = 0;
            
            // Hunger damage
            if status.hunger <= 0 {
                total_damage += HUNGER_DAMAGE;
            }
            
            // Thirst damage
            if status.thirst <= 0 {
                total_damage += THIRST_DAMAGE;
            }
            
            if total_damage > 0 {
                damage_events.push((player.clone(), total_damage));
            }
        }
        
        for (player, damage) in damage_events {
            if let Some(status) = self.tributes.get_mut(&player) {
                status.health -= damage;
                if status.health <= 0 {
                    self.eliminated_tributes.push(player);
                }
            }
        }
    }

    fn consume_resources(&mut self) {
        for (player, status) in self.tributes.iter_mut() {
            if !self.eliminated_tributes.contains(player) {
                status.hunger = (status.hunger - 5).max(0);
                status.thirst = (status.thirst - 8).max(0);
            }
        }
    }

    fn combat(&mut self, attacker: &str, target: &str) -> bool {
        if self.can_attack(attacker, target) {
            let has_weapon = self.tributes.get(attacker)
                .map(|s| s.items.iter().any(|i| matches!(i, Item::Weapon(_))))
                .unwrap_or(false);
            
            let damage = if has_weapon { COMBAT_DAMAGE + 15 } else { COMBAT_DAMAGE };
            
            let target_killed;
            let target_items;
            
            if let Some(target_status) = self.tributes.get_mut(target) {
                target_status.health -= damage;
                target_killed = target_status.health <= 0;
                target_items = if target_killed {
                    Some(target_status.items.clone())
                } else {
                    None
                };
            } else {
                return false;
            }
            
            if target_killed {
                self.eliminated_tributes.push(target.to_string());
                
                // Transfer items
                if let Some(items) = target_items {
                    if let Some(attacker_status) = self.tributes.get_mut(attacker) {
                        attacker_status.kills += 1;
                        attacker_status.items.extend(items);
                    }
                }
                
                return true;
            }
        }
        false
    }

    fn can_attack(&self, attacker: &str, target: &str) -> bool {
        if let (Some(att_status), Some(tar_status)) = 
            (self.tributes.get(attacker), self.tributes.get(target)) {
            att_status.position.location == tar_status.position.location
        } else {
            false
        }
    }

    fn gather_resources(&mut self, player: &str) {
        if let Some(status) = self.tributes.get_mut(player) {
            // Check for resources at current location
            for (_, cache) in self.resources.iter_mut() {
                if cache.location == status.position.location && !cache.items.is_empty() && !cache.discovered_by.contains(&player.to_string()) {
                    // Take an item
                    if let Some(item) = cache.items.pop() {
                        status.items.push(item);
                        cache.discovered_by.push(player.to_string());
                    }
                }
            }
            
            // Basic foraging
            use rand::Rng;
            let mut rng = rand::thread_rng();
            if rng.gen::<f32>() < 0.4 {
                match status.position.location {
                    Location::Forest => status.items.push(Item::Food),
                    Location::River => status.items.push(Item::Water),
                    _ => {}
                }
            }
        }
    }

    fn use_item(&mut self, player: &str, item_type: &str) {
        if let Some(status) = self.tributes.get_mut(player) {
            let mut used_index = None;
            
            for (i, item) in status.items.iter().enumerate() {
                match (item, item_type) {
                    (Item::Food, "food") => {
                        status.hunger = (status.hunger + 30).min(100);
                        used_index = Some(i);
                        break;
                    }
                    (Item::Water, "water") => {
                        status.thirst = (status.thirst + 40).min(100);
                        used_index = Some(i);
                        break;
                    }
                    (Item::Medicine, "medicine") => {
                        status.health = (status.health + RESOURCE_HEAL * 2).min(INITIAL_HEALTH);
                        used_index = Some(i);
                        break;
                    }
                    _ => {}
                }
            }
            
            if let Some(index) = used_index {
                status.items.remove(index);
            }
        }
    }

    fn form_alliance(&mut self, player1: &str, player2: &str) {
        use uuid::Uuid;
        let alliance_id = Uuid::new_v4().to_string();
        
        self.alliances.insert(player1.to_string(), alliance_id.clone());
        self.alliances.insert(player2.to_string(), alliance_id);
        
        // Share resources - update players separately to avoid double borrow
        if let Some(status1) = self.tributes.get_mut(player1) {
            status1.health = (status1.health + ALLIANCE_BONUS).min(INITIAL_HEALTH);
        }
        if let Some(status2) = self.tributes.get_mut(player2) {
            status2.health = (status2.health + ALLIANCE_BONUS).min(INITIAL_HEALTH);
        }
    }

    fn betray_alliance(&mut self, player: &str) {
        if let Some(alliance_id) = self.alliances.get(player).cloned() {
            // Find all members of the alliance
            let members: Vec<String> = self.alliances.iter()
                .filter(|(_, id)| **id == alliance_id)
                .map(|(p, _)| p.clone())
                .collect();
            
            // Remove from alliance
            for member in &members {
                self.alliances.remove(member);
            }
            
            // Apply betrayal penalty to others
            for member in members {
                if member != player {
                    if let Some(status) = self.tributes.get_mut(&member) {
                        status.health -= BETRAYAL_PENALTY;
                        if status.health <= 0 {
                            self.eliminated_tributes.push(member);
                        }
                    }
                }
            }
        }
    }

    fn trigger_environment_event(&mut self, round: u32) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        if round % 10 == 0 && rng.gen::<f32>() < 0.5 {
            let locations = [Location::Forest, Location::Plains, Location::Mountain];
            let location = locations[rng.gen_range(0..locations.len())];
            
            let event = match rng.gen_range(0..4) {
                0 => EnvironmentEvent::Wildfire { location },
                1 => EnvironmentEvent::Flood { location },
                2 => EnvironmentEvent::PoisonFog { location },
                _ => EnvironmentEvent::MuttAttack { location },
            };
            
            self.apply_environment_damage(&event);
            self.environment_events.push(event);
        }
    }

    fn apply_environment_damage(&mut self, event: &EnvironmentEvent) {
        let (location, damage) = match event {
            EnvironmentEvent::Wildfire { location } => (Some(*location), 30),
            EnvironmentEvent::Flood { location } => (Some(*location), 20),
            EnvironmentEvent::PoisonFog { location } => (Some(*location), 25),
            EnvironmentEvent::MuttAttack { location } => (Some(*location), 35),
            EnvironmentEvent::ColdSnap => (None, 15),
        };
        
        for (player, status) in self.tributes.iter_mut() {
            if self.eliminated_tributes.contains(player) {
                continue;
            }
            
            let should_damage = match location {
                Some(loc) => status.position.location == loc,
                None => true, // ColdSnap affects everyone
            };
            
            if should_damage {
                status.health -= damage;
                if status.health <= 0 {
                    self.eliminated_tributes.push(player.clone());
                }
            }
        }
    }

    fn calculate_analytics(&self, history: &[RoundResult]) -> GameAnalytics {
        let total_tributes = self.tributes.len();
        let survival_rate = (total_tributes - self.eliminated_tributes.len()) as f32 / total_tributes as f32;
        
        // Alliance formation rate
        let alliance_count = self.alliances.len();
        let alliance_rate = alliance_count as f32 / total_tributes as f32;
        
        // Combat intensity
        let combat_actions = history.iter()
            .flat_map(|r| r.actions.values())
            .filter(|a| a.action_type == "hunt")
            .count();
        let combat_intensity = combat_actions as f32 / history.len().max(1) as f32;
        
        GameAnalytics {
            collective_coordination_score: alliance_rate,
            decision_diversity_index: 0.7, // Mix of combat and survival
            strategic_depth: combat_intensity * 0.4 + alliance_rate * 0.6,
            emergence_frequency: self.environment_events.len() as f32 / history.len().max(1) as f32,
            performance_differential: 1.0 - survival_rate,
        }
    }
}

#[async_trait]
impl Game for HungerGamesGame {
    async fn initialize(&mut self, config: GameConfig) -> anyhow::Result<GameState> {
        if let Some(rounds_str) = config.special_rules.get("max_rounds") {
            if let Ok(rounds) = rounds_str.parse::<u32>() {
                self.max_rounds = rounds;
            }
        }
        
        Ok(GameState {
            game_id: Uuid::new_v4(),
            game_type: GameType::HungerGames,
            round: 0,
            scores: HashMap::new(),
            history: vec![],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("environment".to_string(), serde_json::json!("Arena"));
                meta.insert("sponsors_active".to_string(), serde_json::json!(true));
                meta
            },
        })
    }

    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        // Initialize tributes on first round
        if state.round == 0 && self.tributes.is_empty() {
            let players: Vec<String> = actions.keys().cloned().collect();
            self.spawn_tributes(players);
        }

        // Process actions
        let mut round_events = Vec::new();
        
        for (player_id, action) in &actions {
            if self.eliminated_tributes.contains(player_id) {
                continue;
            }
            
            match action.action_type.as_str() {
                "hunt" => {
                    if let Ok(Some(target_player)) = serde_json::from_value::<Option<String>>(action.data.clone()) {
                        if self.combat(player_id, &target_player) {
                            round_events.push(format!("{} eliminated {}", player_id, target_player));
                        }
                    }
                }
                "gather" => {
                    self.gather_resources(player_id);
                }
                "form_alliance" => {
                    if let Ok(ally) = serde_json::from_value::<String>(action.data.clone()) {
                        self.form_alliance(player_id, &ally);
                        round_events.push(format!("{} allied with {}", player_id, ally));
                    }
                }
                "betray" => {
                    self.betray_alliance(player_id);
                    round_events.push(format!("{} betrayed their alliance", player_id));
                }
                "move" => {
                    if let Ok(location) = serde_json::from_value::<Location>(action.data.clone()) {
                        if let Some(status) = self.tributes.get_mut(player_id) {
                            status.position.location = location;
                        }
                    }
                }
                "use_item" => {
                    if let Ok(item) = serde_json::from_value::<String>(action.data.clone()) {
                        self.use_item(player_id, &item);
                    }
                }
                _ => {}
            }
        }
        
        // Consume resources
        self.consume_resources();
        
        // Apply survival damage
        self.apply_survival_damage();
        
        // Trigger environment events
        self.trigger_environment_event(state.round);
        
        // Calculate scores
        let mut scores_delta = HashMap::new();
        for (player, status) in &self.tributes {
            if !self.eliminated_tributes.contains(player) {
                let survival_score = 1;
                let kill_score = status.kills as i32 * 5;
                let health_bonus = status.health / 20;
                scores_delta.insert(player.clone(), survival_score + kill_score + health_bonus);
            }
        }
        
        let survivors: Vec<String> = self.tributes.keys()
            .filter(|p| !self.eliminated_tributes.contains(p))
            .cloned()
            .collect();
        
        round_events.push(format!("{} tributes remaining", survivors.len()));
        
        Ok(RoundResult {
            round: state.round,
            actions,
            outcome: Outcome {
                winners: survivors,
                losers: self.eliminated_tributes.clone(),
                special_events: round_events,
                emergence_detected: self.alliances.len() > 2,
            },
            scores_delta,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn is_game_over(&self, state: &GameState) -> bool {
        let survivors = self.tributes.len() - self.eliminated_tributes.len();
        survivors <= 1 || state.round >= self.max_rounds
    }

    async fn calculate_final_result(&self, state: &GameState) -> GameResult {
        let winner = self.tributes.keys()
            .find(|p| !self.eliminated_tributes.contains(p))
            .cloned()
            .unwrap_or_else(|| "No Victor".to_string());
        
        let mut final_scores = state.scores.clone();
        
        // Victory bonus
        if winner != "No Victor" {
            *final_scores.entry(winner.clone()).or_insert(0) += 100;
        }
        
        // Emergence events from alliances and betrayals
        let emergence_events: Vec<EmergenceEvent> = state.history.iter()
            .filter(|r| r.outcome.emergence_detected)
            .map(|r| EmergenceEvent {
                round: r.round,
                event_type: "Alliance Formation".to_string(),
                description: "Tributes formed strategic alliances".to_string(),
                emergence_score: 0.7,
            })
            .collect();
        
        GameResult {
            game_id: state.game_id,
            winner,
            final_scores,
            total_rounds: state.round,
            emergence_events,
            analytics: self.calculate_analytics(&state.history),
        }
    }
}