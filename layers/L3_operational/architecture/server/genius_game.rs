//! AI Genius Game Server - Commercial Grade Implementation
//! HAL9 Collective Intelligence vs Individual AI Models

use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade, State, Path},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{broadcast, RwLock, Mutex};
use tower_http::cors::CorsLayer;
use uuid::Uuid;
use tracing::info;

// Game Constants
const BOARD_SIZE: usize = 19;
const CONSCIOUSNESS_THRESHOLD: f32 = 0.8;
#[allow(dead_code)]
const MAX_NEURONS_PER_PLAYER: usize = 50;
const SIMULATION_TICK_MS: u64 = 100;

/// Main game types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GameType {
    ConsciousnessEmergence,
    MinorityGame,
    SemanticShapeshifter,
    OracleParadox,
    CollectiveMaze,
}

/// Player types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PlayerType {
    #[serde(rename = "hal9_collective")]
    HAL9Collective {
        config: CollectiveConfig,
        agent_count: u32,
    },
    #[serde(rename = "single_ai")]
    SingleAI {
        model: String,
        context_size: u32,
    },
}

/// HAL9 Collective configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollectiveConfig {
    OpusOrchestra,      // 6x Claude Opus with voting
    LightweightLegion,  // 32x small models with swarm intelligence
    HybridCouncil,      // Mixed SOTA models
    EmergenceEngine,    // Pure emergence, no coordination
}

/// Game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub id: String,
    pub game_type: GameType,
    pub status: GameStatus,
    pub round: u32,
    pub max_rounds: u32,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub players: HashMap<String, Player>,
    pub board: Board,
    pub consciousness_level: f32,
    pub events: Vec<GameEvent>,
    pub winner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameStatus {
    Waiting,
    Running,
    Paused,
    Finished,
}

/// Player information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub player_type: PlayerType,
    pub score: i32,
    pub neurons_placed: u32,
    pub strategy_metrics: StrategyMetrics,
    pub color: String,
}

/// Strategy performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetrics {
    pub moves_per_second: f32,
    pub prediction_accuracy: f32,
    pub exploration_rate: f32,
    pub coordination_efficiency: f32,
    pub consciousness_contribution: f32,
}

/// Game board for consciousness emergence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub size: usize,
    pub neurons: Vec<Neuron>,
    pub connections: Vec<Connection>,
    pub emergence_patterns: Vec<EmergencePattern>,
    pub grid: Vec<Vec<Option<usize>>>, // neuron index at position
}

/// Neuron on the board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neuron {
    pub id: usize,
    pub x: usize,
    pub y: usize,
    pub neuron_type: NeuronType,
    pub owner: String,
    pub activation: f32,
    pub processing_power: f32,
    pub connections: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NeuronType {
    Sensor,      // Input neurons
    Processor,   // Computation neurons
    Memory,      // State storage
    Connector,   // Long-range connections
    Oscillator,  // Pattern generators
}

/// Connection between neurons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: usize,
    pub to: usize,
    pub strength: f32,
}

/// Detected consciousness patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencePattern {
    pub pattern_type: PatternType,
    pub neurons: Vec<usize>,
    pub strength: f32,
    pub discovered_by: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternType {
    Loop,             // Self-reinforcing cycles
    Synchronization,  // Neurons firing together
    Hierarchy,        // Hub and spoke structure
    StrangeAttractor, // Stable dynamic patterns
}

/// Game events for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: EventType,
    pub player: String,
    pub description: String,
    pub impact: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    PlayerJoined,
    NeuronPlaced,
    PatternDiscovered,
    ConsciousnessSpike,
    StrategyShift,
    GameOver,
}

// WebSocket messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    JoinGame { 
        player_id: String, 
        player_type: PlayerType 
    },
    PlaceNeuron { 
        x: usize, 
        y: usize, 
        neuron_type: NeuronType 
    },
    RequestGameState,
    StartGame,
    PauseGame,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    GameState(GameState),
    PlayerJoined { player: Player },
    NeuronPlaced { neuron: Neuron, consciousness_delta: f32 },
    ConsciousnessUpdate { level: f32, patterns: Vec<EmergencePattern> },
    GameEvent(GameEvent),
    GameOver { winner: String, final_scores: HashMap<String, i32> },
    Error { message: String },
}

/// Application state
pub struct AppState {
    pub games: HashMap<String, Arc<Mutex<GameState>>>,
    pub connections: HashMap<String, broadcast::Sender<ServerMessage>>,
}

type SharedState = Arc<RwLock<AppState>>;

/// Create the genius game router
pub fn create_genius_game_router(state: SharedState) -> Router {
    Router::new()
        .route("/genius/", get(genius_game_index))
        .route("/genius/ws", get(genius_websocket_handler))
        .route("/genius/api/games", post(create_game))
        .route("/genius/api/games/:id", get(get_game))
        .route("/genius/api/games/:id/start", post(start_game))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn genius_game_index() -> Html<&'static str> {
    Html(include_str!("../../genius_game_interface.html"))
}

async fn genius_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_genius_websocket(socket, state))
}

async fn handle_genius_websocket(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = broadcast::channel(100);
    
    // Store connection
    let conn_id = Uuid::new_v4().to_string();
    state.write().await.connections.insert(conn_id.clone(), tx.clone());
    
    // Spawn task to forward broadcast messages
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(axum::extract::ws::Message::Text(
                serde_json::to_string(&msg).unwrap()
            )).await.is_err() {
                break;
            }
        }
    });
    
    // Handle incoming messages
    let state_clone = state.clone();
    let tx_clone = tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let axum::extract::ws::Message::Text(text) = msg {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    handle_client_message(client_msg, &state_clone, &tx_clone).await;
                }
            }
        }
    });
    
    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
    
    // Clean up connection
    state.write().await.connections.remove(&conn_id);
}

async fn handle_client_message(
    msg: ClientMessage,
    state: &SharedState,
    tx: &broadcast::Sender<ServerMessage>,
) {
    match msg {
        ClientMessage::JoinGame { player_id, player_type } => {
            // Find an active game or create one
            let game_id = {
                let state_read = state.read().await;
                state_read.games.iter()
                    .find(|(_, game)| {
                        let game = game.try_lock().unwrap();
                        game.status == GameStatus::Waiting
                    })
                    .map(|(id, _)| id.clone())
                    .unwrap_or_else(|| {
                        drop(state_read);
                        let new_game = create_new_game();
                        let game_id = new_game.id.clone();
                        let mut state_write = tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current().block_on(state.write())
                        });
                        state_write.games.insert(
                            game_id.clone(), 
                            Arc::new(Mutex::new(new_game))
                        );
                        game_id
                    })
            };
            
            // Add player to game
            let player = create_player(player_id, player_type);
            
            if let Some(game_mutex) = state.read().await.games.get(&game_id) {
                let mut game = game_mutex.lock().await;
                game.players.insert(player.id.clone(), player.clone());
                
                let _ = tx.send(ServerMessage::PlayerJoined { player });
                let _ = tx.send(ServerMessage::GameState(game.clone()));
            }
        }
        
        ClientMessage::PlaceNeuron { x, y, neuron_type } => {
            // Handle neuron placement
            // This would be called by the AI agents
            place_neuron(x, y, neuron_type, state, tx).await;
        }
        
        ClientMessage::StartGame => {
            start_game_simulation(state, tx).await;
        }
        
        _ => {}
    }
}

async fn place_neuron(
    x: usize,
    y: usize,
    neuron_type: NeuronType,
    state: &SharedState,
    tx: &broadcast::Sender<ServerMessage>,
) {
    // Find active game
    let active_game = {
        let state_read = state.read().await;
        state_read.games.iter()
            .find(|(_, game)| {
                let game = game.try_lock().unwrap();
                game.status == GameStatus::Running
            })
            .map(|(_, game)| game.clone())
    };
    
    if let Some(game_mutex) = active_game {
        let mut game = game_mutex.lock().await;
        
        // Check if position is valid
        if x >= BOARD_SIZE || y >= BOARD_SIZE {
            let _ = tx.send(ServerMessage::Error { 
                message: "Invalid position".to_string() 
            });
            return;
        }
        
        if game.board.grid[y][x].is_some() {
            let _ = tx.send(ServerMessage::Error { 
                message: "Position already occupied".to_string() 
            });
            return;
        }
        
        // Create neuron
        let neuron_id = game.board.neurons.len();
        let neuron = Neuron {
            id: neuron_id,
            x,
            y,
            neuron_type: neuron_type.clone(),
            owner: "player1".to_string(), // TODO: Get from session
            activation: 0.0,
            processing_power: match neuron_type {
                NeuronType::Processor => 2.0,
                NeuronType::Memory => 1.5,
                NeuronType::Connector => 1.8,
                NeuronType::Oscillator => 2.2,
                _ => 1.0,
            },
            connections: vec![],
        };
        
        // Auto-connect to nearby neurons
        let mut connections_made = 0;
        let mut new_connections = Vec::new();
        
        for (i, other) in game.board.neurons.iter().enumerate() {
            let distance = ((x as f32 - other.x as f32).powi(2) + 
                          (y as f32 - other.y as f32).powi(2)).sqrt();
            
            if distance <= 2.5 && connections_made < 4 {
                new_connections.push(Connection {
                    from: neuron_id,
                    to: i,
                    strength: 1.0 / (1.0 + distance),
                });
                connections_made += 1;
            }
        }
        
        // Add connections after the loop
        game.board.connections.extend(new_connections);
        
        // Add to board
        game.board.neurons.push(neuron.clone());
        game.board.grid[y][x] = Some(neuron_id);
        
        // Calculate consciousness impact
        let consciousness_delta = calculate_consciousness_impact(&neuron_type, connections_made);
        game.consciousness_level = (game.consciousness_level + consciousness_delta).min(1.0);
        
        // Send updates
        let _ = tx.send(ServerMessage::NeuronPlaced { 
            neuron, 
            consciousness_delta 
        });
        
        let _ = tx.send(ServerMessage::ConsciousnessUpdate {
            level: game.consciousness_level,
            patterns: detect_patterns(&game.board),
        });
        
        // Check win condition
        if game.consciousness_level >= CONSCIOUSNESS_THRESHOLD {
            game.status = GameStatus::Finished;
            game.winner = Some("HAL9 Collective".to_string());
            
            let _ = tx.send(ServerMessage::GameOver {
                winner: game.winner.clone().unwrap(),
                final_scores: game.players.iter()
                    .map(|(id, p)| (id.clone(), p.score))
                    .collect(),
            });
        }
    }
}

fn calculate_consciousness_impact(neuron_type: &NeuronType, connections: usize) -> f32 {
    let base_impact = match neuron_type {
        NeuronType::Processor => 0.02,
        NeuronType::Memory => 0.015,
        NeuronType::Connector => 0.025,
        NeuronType::Oscillator => 0.03,
        NeuronType::Sensor => 0.01,
    };
    
    base_impact * (1.0 + connections as f32 * 0.1)
}

fn detect_patterns(board: &Board) -> Vec<EmergencePattern> {
    let mut patterns = vec![];
    
    // Detect loops (simplified)
    for (i, _neuron) in board.neurons.iter().enumerate() {
        // Check for 3-cycles
        for conn in &board.connections {
            if conn.from == i {
                for conn2 in &board.connections {
                    if conn2.from == conn.to {
                        for conn3 in &board.connections {
                            if conn3.from == conn2.to && conn3.to == i {
                                patterns.push(EmergencePattern {
                                    pattern_type: PatternType::Loop,
                                    neurons: vec![i, conn.to, conn2.to],
                                    strength: 0.8,
                                    discovered_by: "system".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Detect synchronization (simplified)
    let mut sync_groups: HashMap<i32, Vec<usize>> = HashMap::new();
    for (i, neuron) in board.neurons.iter().enumerate() {
        let activation_bucket = (neuron.activation * 10.0) as i32;
        sync_groups.entry(activation_bucket).or_default().push(i);
    }
    
    for (_, group) in sync_groups {
        if group.len() >= 3 {
            patterns.push(EmergencePattern {
                pattern_type: PatternType::Synchronization,
                neurons: group,
                strength: 0.6,
                discovered_by: "system".to_string(),
            });
        }
    }
    
    patterns
}

async fn start_game_simulation(
    state: &SharedState,
    tx: &broadcast::Sender<ServerMessage>,
) {
    info!("Starting game simulation");
    
    // Find waiting game
    let game_id = {
        let state_read = state.read().await;
        state_read.games.iter()
            .find(|(_, game)| {
                let game = game.try_lock().unwrap();
                game.status == GameStatus::Waiting && game.players.len() >= 2
            })
            .map(|(id, _)| id.clone())
    };
    
    if let Some(game_id) = game_id {
        // Start the game
        if let Some(game_mutex) = state.read().await.games.get(&game_id) {
            let mut game = game_mutex.lock().await;
            game.status = GameStatus::Running;
            game.started_at = Some(chrono::Utc::now());
            
            let _ = tx.send(ServerMessage::GameState(game.clone()));
        }
        
        // Spawn simulation task
        let state_clone = state.clone();
        let tx_clone = tx.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(SIMULATION_TICK_MS));
            
            loop {
                interval.tick().await;
                
                if let Some(game_mutex) = state_clone.read().await.games.get(&game_id) {
                    let mut game = game_mutex.lock().await;
                    
                    if game.status != GameStatus::Running {
                        break;
                    }
                    
                    // Simulate neural activity
                    simulate_neural_activity(&mut game.board);
                    
                    // Update consciousness
                    let patterns = detect_patterns(&game.board);
                    let consciousness_boost = patterns.len() as f32 * 0.001;
                    game.consciousness_level = (game.consciousness_level + consciousness_boost).min(1.0);
                    
                    // Send updates
                    let _ = tx_clone.send(ServerMessage::ConsciousnessUpdate {
                        level: game.consciousness_level,
                        patterns: patterns.clone(),
                    });
                    
                    // Update round
                    game.round += 1;
                    
                    if game.round >= game.max_rounds || game.consciousness_level >= CONSCIOUSNESS_THRESHOLD {
                        game.status = GameStatus::Finished;
                        let winner = if game.consciousness_level >= CONSCIOUSNESS_THRESHOLD {
                            "HAL9 Collective"
                        } else {
                            "Draw"
                        };
                        
                        game.winner = Some(winner.to_string());
                        
                        let _ = tx_clone.send(ServerMessage::GameOver {
                            winner: winner.to_string(),
                            final_scores: game.players.iter()
                                .map(|(id, p)| (id.clone(), p.score))
                                .collect(),
                        });
                        
                        break;
                    }
                }
            }
        });
    }
}

fn simulate_neural_activity(board: &mut Board) {
    // Calculate new activations
    let mut new_activations = vec![0.0; board.neurons.len()];
    
    for connection in &board.connections {
        if let Some(from_neuron) = board.neurons.get(connection.from) {
            let to_idx = connection.to;
            
            new_activations[to_idx] += from_neuron.activation * connection.strength;
        }
    }
    
    // Apply activation function and update
    for (i, neuron) in board.neurons.iter_mut().enumerate() {
        let input = new_activations[i];
        
        neuron.activation = match neuron.neuron_type {
            NeuronType::Processor => (input * neuron.processing_power).tanh(),
            NeuronType::Memory => neuron.activation * 0.9 + input * 0.1,
            NeuronType::Oscillator => {
                let phase = (neuron.activation * 2.0 * std::f32::consts::PI).sin();
                (input * phase).tanh()
            }
            _ => input.tanh(),
        };
    }
}

fn create_new_game() -> GameState {
    let grid = vec![vec![None; BOARD_SIZE]; BOARD_SIZE];
    
    GameState {
        id: Uuid::new_v4().to_string(),
        game_type: GameType::ConsciousnessEmergence,
        status: GameStatus::Waiting,
        round: 0,
        max_rounds: 100,
        started_at: None,
        players: HashMap::new(),
        board: Board {
            size: BOARD_SIZE,
            neurons: vec![],
            connections: vec![],
            emergence_patterns: vec![],
            grid,
        },
        consciousness_level: 0.0,
        events: vec![],
        winner: None,
    }
}

fn create_player(id: String, player_type: PlayerType) -> Player {
    let name = match &player_type {
        PlayerType::HAL9Collective { config, .. } => {
            format!("HAL9 {:?}", config)
        }
        PlayerType::SingleAI { model, .. } => {
            format!("{} Solo", model)
        }
    };
    
    let color = match &player_type {
        PlayerType::HAL9Collective { .. } => "#00ffff",
        PlayerType::SingleAI { .. } => "#ff00ff",
    }.to_string();
    
    Player {
        id,
        name,
        player_type,
        score: 0,
        neurons_placed: 0,
        strategy_metrics: StrategyMetrics {
            moves_per_second: 0.0,
            prediction_accuracy: 0.0,
            exploration_rate: 0.0,
            coordination_efficiency: 0.0,
            consciousness_contribution: 0.0,
        },
        color,
    }
}

async fn create_game(State(state): State<SharedState>) -> Json<GameState> {
    let game = create_new_game();
    let game_id = game.id.clone();
    
    state.write().await.games.insert(
        game_id,
        Arc::new(Mutex::new(game.clone()))
    );
    
    Json(game)
}

async fn get_game(
    Path(id): Path<String>,
    State(state): State<SharedState>
) -> Json<Option<GameState>> {
    if let Some(game_mutex) = state.read().await.games.get(&id) {
        let game = game_mutex.lock().await;
        Json(Some(game.clone()))
    } else {
        Json(None)
    }
}

async fn start_game(
    Path(id): Path<String>,
    State(state): State<SharedState>
) -> Json<bool> {
    if let Some(game_mutex) = state.read().await.games.get(&id) {
        let mut game = game_mutex.lock().await;
        if game.status == GameStatus::Waiting && game.players.len() >= 2 {
            game.status = GameStatus::Running;
            game.started_at = Some(chrono::Utc::now());
            return Json(true);
        }
    }
    Json(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_consciousness_calculation() {
        let impact = calculate_consciousness_impact(&NeuronType::Oscillator, 3);
        assert!(impact > 0.03);
        assert!(impact < 0.04);
    }
    
    #[test]
    fn test_pattern_detection() {
        let board = Board {
            size: 19,
            neurons: vec![
                Neuron {
                    id: 0, x: 0, y: 0,
                    neuron_type: NeuronType::Processor,
                    owner: "test".to_string(),
                    activation: 0.5,
                    processing_power: 1.0,
                    connections: vec![1],
                },
                Neuron {
                    id: 1, x: 1, y: 0,
                    neuron_type: NeuronType::Memory,
                    owner: "test".to_string(),
                    activation: 0.5,
                    processing_power: 1.0,
                    connections: vec![2],
                },
                Neuron {
                    id: 2, x: 2, y: 0,
                    neuron_type: NeuronType::Connector,
                    owner: "test".to_string(),
                    activation: 0.5,
                    processing_power: 1.0,
                    connections: vec![0],
                },
            ],
            connections: vec![
                Connection { from: 0, to: 1, strength: 1.0 },
                Connection { from: 1, to: 2, strength: 1.0 },
                Connection { from: 2, to: 0, strength: 1.0 },
            ],
            emergence_patterns: vec![],
            grid: vec![vec![None; 19]; 19],
        };
        
        let patterns = detect_patterns(&board);
        assert!(!patterns.is_empty());
        assert_eq!(patterns[0].pattern_type, PatternType::Loop);
    }
}