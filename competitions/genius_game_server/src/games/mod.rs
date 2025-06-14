pub mod minority_game;
pub mod byzantine_generals;
pub mod collective_maze;
pub mod recursive_reasoning;
pub mod swarm_optimization;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub game_type: GameType,
    pub rounds: u32,
    pub time_limit_ms: u64,
    pub special_rules: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameType {
    MinorityGame,
    ByzantineGenerals,
    CollectiveMaze,
    RecursiveReasoning,
    SwarmOptimization,
    LiarGame,
    BeautyContest,
    OraclesCurse,
    ConsciousnessTuringTest,
    MetaGameDesign,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub game_id: Uuid,
    pub game_type: GameType,
    pub round: u32,
    pub scores: HashMap<String, i32>,
    pub history: Vec<RoundResult>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundResult {
    pub round: u32,
    pub actions: HashMap<String, Action>,
    pub outcome: Outcome,
    pub scores_delta: HashMap<String, i32>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub player_id: String,
    pub action_type: String,
    pub data: serde_json::Value,
    pub reasoning: Option<String>,
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub winners: Vec<String>,
    pub losers: Vec<String>,
    pub special_events: Vec<String>,
    pub emergence_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResult {
    pub game_id: Uuid,
    pub winner: String,
    pub final_scores: HashMap<String, i32>,
    pub total_rounds: u32,
    pub emergence_events: Vec<EmergenceEvent>,
    pub analytics: GameAnalytics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergenceEvent {
    pub round: u32,
    pub event_type: String,
    pub description: String,
    pub emergence_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAnalytics {
    pub collective_coordination_score: f32,
    pub decision_diversity_index: f32,
    pub strategic_depth: f32,
    pub emergence_frequency: f32,
    pub performance_differential: f32,
}

#[async_trait]
pub trait Game: Send + Sync {
    async fn initialize(&mut self, config: GameConfig) -> anyhow::Result<GameState>;
    async fn process_round(&mut self, state: &GameState, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult>;
    async fn is_game_over(&self, state: &GameState) -> bool;
    async fn calculate_final_result(&self, state: &GameState) -> GameResult;
}

pub struct GameEngine {
    active_games: dashmap::DashMap<Uuid, (GameState, Box<dyn Game>)>,
}

impl GameEngine {
    pub fn new() -> Self {
        Self {
            active_games: dashmap::DashMap::new(),
        }
    }
    
    fn create_game_instance(game_type: &GameType) -> Box<dyn Game> {
        match game_type {
            GameType::MinorityGame => Box::new(minority_game::MinorityGame::new()),
            GameType::ByzantineGenerals => Box::new(byzantine_generals::ByzantineGenerals::new()),
            GameType::CollectiveMaze => Box::new(collective_maze::CollectiveMaze::new()),
            GameType::RecursiveReasoning => Box::new(recursive_reasoning::RecursiveReasoning::new()),
            GameType::SwarmOptimization => Box::new(swarm_optimization::SwarmOptimization::new()),
            _ => Box::new(minority_game::MinorityGame::new()), // Default fallback
        }
    }
    
    pub async fn create_game(&self, config: GameConfig) -> anyhow::Result<Uuid> {
        let game_type = config.game_type.clone();
        let mut game = Self::create_game_instance(&game_type);
        let state = game.initialize(config).await?;
        let game_id = state.game_id;
        self.active_games.insert(game_id, (state, game));
        Ok(game_id)
    }
    
    pub async fn process_turn(&self, game_id: Uuid, actions: HashMap<String, Action>) -> anyhow::Result<RoundResult> {
        let mut game_data = self.active_games.get_mut(&game_id)
            .ok_or_else(|| anyhow::anyhow!("Game not found"))?;
        
        let (state, game) = game_data.value_mut();
        let result = game.process_round(state, actions).await?;
        
        // Update state
        state.round += 1;
        state.history.push(result.clone());
        
        // Update scores
        for (player, delta) in &result.scores_delta {
            *state.scores.entry(player.clone()).or_insert(0) += delta;
        }
        
        Ok(result)
    }
    
    pub async fn get_game_state(&self, game_id: Uuid) -> Option<GameState> {
        self.active_games.get(&game_id).map(|game_data| game_data.0.clone())
    }
    
    pub async fn is_game_finished(&self, game_id: Uuid) -> bool {
        if let Some(game_data) = self.active_games.get(&game_id) {
            let (state, game) = game_data.value();
            return game.is_game_over(state).await;
        }
        true
    }
    
    pub async fn finalize_game(&self, game_id: Uuid) -> anyhow::Result<GameResult> {
        let game_data = self.active_games.get(&game_id)
            .ok_or_else(|| anyhow::anyhow!("Game not found"))?;
        
        let (state, game) = game_data.value();
        let result = game.calculate_final_result(state).await;
        drop(game_data);
        
        self.active_games.remove(&game_id);
        Ok(result)
    }
}