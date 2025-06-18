//! AI Genius Game 2025 - Commercial Grade Implementation
//! Real-time multiplayer consciousness emergence game

use axum::{
    extract::{ws::WebSocket, ws::Message, WebSocketUpgrade, State, Path},
    response::{Html, Response},
    routing::{get, post},
    middleware,
    Json, Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock, Mutex};
use tower_http::cors::CorsLayer;
use uuid::Uuid;
use tracing::info;

mod auth;
mod user_store;
use auth::{Claims, LoginRequest, LoginResponse, RegisterRequest, UserInfo, 
           create_jwt, verify_password};
use user_store::UserStore;

// Game Constants
const BOARD_SIZE: usize = 19;
const CONSCIOUSNESS_THRESHOLD: f32 = 0.8;
const MAX_NEURONS_PER_PLAYER: usize = 50;
const SIMULATION_TICK_MS: u64 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GameType {
    ConsciousnessEmergence,
    MinorityGame,
    SemanticShapeshifter,
    OracleParadox,
    CollectiveMaze,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PlayerType {
    #[serde(rename = "hal9_collective")]
    HAL9Collective {
        agent_count: u32,
    },
    #[serde(rename = "single_ai")]
    SingleAI {
        model: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub id: String,
    pub game_type: GameType,
    pub status: GameStatus,
    pub round: u32,
    pub max_rounds: u32,
    pub players: HashMap<String, Player>,
    pub board: Vec<Vec<Cell>>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub player_type: PlayerType,
    pub score: i32,
    pub neurons_placed: u32,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub owner: Option<String>,
    pub neuron_strength: f32,
    pub connections: Vec<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum GameAction {
    #[serde(rename = "place_neuron")]
    PlaceNeuron { x: usize, y: usize },
    #[serde(rename = "strengthen_connection")]
    StrengthenConnection { from: (usize, usize), to: (usize, usize) },
    #[serde(rename = "activate_special")]
    ActivateSpecial { ability: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "join_game")]
    JoinGame {
        player_name: String,
        player_type: PlayerType,
        game_id: Option<String>,
    },
    #[serde(rename = "game_action")]
    GameAction {
        action: GameAction,
    },
    #[serde(rename = "game_state")]
    GameState {
        state: GameState,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
    },
}

struct AppState {
    games: Arc<RwLock<HashMap<String, Arc<Mutex<GameState>>>>>,
    connections: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
    user_store: UserStore,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    // Initialize user store and create default admin
    let user_store = UserStore::new();
    user_store.init_default_admin().await;
    
    let state = Arc::new(AppState {
        games: Arc::new(RwLock::new(HashMap::new())),
        connections: Arc::new(RwLock::new(HashMap::new())),
        user_store,
    });
    
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/", get(index))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/games", get(list_games))
        .route("/api/games/:id", get(get_game));
    
    // Protected routes (auth required)
    let protected_routes = Router::new()
        .route("/api/games/create", post(create_game))
        .route("/api/auth/profile", get(get_profile))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));
    
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .route("/ws/:id", get(websocket_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3456")
        .await
        .unwrap();
    
    info!("🎮 AI Genius Game Server running on http://localhost:3456");
    info!("🔗 WebSocket endpoint: ws://localhost:3456/ws/:game_id");
    
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

async fn create_game(
    State(state): State<Arc<AppState>>,
    Json(params): Json<CreateGameParams>,
) -> Result<Json<GameInfo>, StatusCode> {
    let game_id = Uuid::new_v4().to_string();
    let game = GameState {
        id: game_id.clone(),
        game_type: params.game_type,
        status: GameStatus::Waiting,
        round: 0,
        max_rounds: params.max_rounds.unwrap_or(20),
        players: HashMap::new(),
        board: vec![vec![Cell {
            owner: None,
            neuron_strength: 0.0,
            connections: vec![],
        }; BOARD_SIZE]; BOARD_SIZE],
        consciousness_level: 0.0,
        events: vec![],
        winner: None,
    };
    
    state.games.write().await.insert(
        game_id.clone(),
        Arc::new(Mutex::new(game))
    );
    
    let (tx, _) = broadcast::channel(100);
    state.connections.write().await.insert(game_id.clone(), tx);
    
    Ok(Json(GameInfo {
        id: game_id,
        status: "created".to_string(),
    }))
}

async fn list_games(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<GameInfo>> {
    let games = state.games.read().await;
    let mut game_list = vec![];
    
    for (id, game) in games.iter() {
        let g = game.lock().await;
        game_list.push(GameInfo {
            id: id.clone(),
            status: format!("{:?}", g.status),
        });
    }
    
    Json(game_list)
}

async fn get_game(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<GameState>, StatusCode> {
    let games = state.games.read().await;
    
    if let Some(game) = games.get(&id) {
        let g = game.lock().await;
        Ok(Json(g.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(game_id): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, game_id))
}

async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    game_id: String,
) {
    let (mut sender, mut receiver) = socket.split();
    let player_id = Uuid::new_v4().to_string();
    
    // Subscribe to game broadcasts
    let connections = state.connections.read().await;
    let tx = if let Some(tx) = connections.get(&game_id) {
        tx.clone()
    } else {
        let _ = sender.send(Message::Text(
            serde_json::to_string(&WebSocketMessage::Error {
                message: "Game not found".to_string()
            }).unwrap()
        )).await;
        return;
    };
    drop(connections);
    
    let mut rx = tx.subscribe();
    
    // Handle incoming messages
    let state_clone = state.clone();
    let game_id_clone = game_id.clone();
    let player_id_clone = player_id.clone();
    let tx_clone = tx.clone();
    
    tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(Message::Text(text)) = msg {
                if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                    handle_game_message(
                        ws_msg,
                        &state_clone,
                        &game_id_clone,
                        &player_id_clone,
                        &tx_clone
                    ).await;
                }
            }
        }
    });
    
    // Send game updates to client
    while let Ok(msg) = rx.recv().await {
        if sender.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}

async fn handle_game_message(
    message: WebSocketMessage,
    state: &Arc<AppState>,
    game_id: &str,
    player_id: &str,
    tx: &broadcast::Sender<String>,
) {
    match message {
        WebSocketMessage::JoinGame { player_name, player_type, .. } => {
            let games = state.games.read().await;
            if let Some(game) = games.get(game_id) {
                let mut g = game.lock().await;
                
                let player = Player {
                    id: player_id.to_string(),
                    name: player_name,
                    player_type,
                    score: 0,
                    neurons_placed: 0,
                    color: generate_player_color(&g.players),
                };
                
                g.players.insert(player_id.to_string(), player.clone());
                
                g.events.push(GameEvent {
                    timestamp: chrono::Utc::now(),
                    event_type: "player_joined".to_string(),
                    description: format!("Player {} joined the game", player.name),
                });
                
                // Broadcast updated game state
                let _ = tx.send(serde_json::to_string(&WebSocketMessage::GameState {
                    state: g.clone()
                }).unwrap());
            }
        }
        WebSocketMessage::GameAction { action } => {
            let games = state.games.read().await;
            if let Some(game) = games.get(game_id) {
                let mut g = game.lock().await;
                
                if g.status != GameStatus::Running {
                    return;
                }
                
                // Process game action
                match action {
                    GameAction::PlaceNeuron { x, y } => {
                        if x < BOARD_SIZE && y < BOARD_SIZE {
                            g.board[y][x] = Cell {
                                owner: Some(player_id.to_string()),
                                neuron_strength: 1.0,
                                connections: vec![],
                            };
                            
                            if let Some(player) = g.players.get_mut(player_id) {
                                player.neurons_placed += 1;
                                player.score += 10;
                            }
                            
                            // Update consciousness level
                            g.consciousness_level = calculate_consciousness(&g.board);
                            
                            g.events.push(GameEvent {
                                timestamp: chrono::Utc::now(),
                                event_type: "neuron_placed".to_string(),
                                description: format!("Neuron placed at ({}, {})", x, y),
                            });
                        }
                    }
                    GameAction::StrengthenConnection { from: _, to: _ } => {
                        // Implement connection logic
                    }
                    GameAction::ActivateSpecial { ability: _ } => {
                        // Implement special abilities
                    }
                }
                
                // Check win conditions
                if g.consciousness_level >= CONSCIOUSNESS_THRESHOLD {
                    g.status = GameStatus::Finished;
                    g.winner = Some(player_id.to_string());
                }
                
                // Broadcast updated game state
                let _ = tx.send(serde_json::to_string(&WebSocketMessage::GameState {
                    state: g.clone()
                }).unwrap());
            }
        }
        _ => {}
    }
}

fn calculate_consciousness(board: &Vec<Vec<Cell>>) -> f32 {
    let mut total_strength = 0.0;
    let mut connections = 0;
    
    for row in board {
        for cell in row {
            total_strength += cell.neuron_strength;
            connections += cell.connections.len();
        }
    }
    
    // Simple consciousness calculation
    let density = total_strength / (BOARD_SIZE * BOARD_SIZE) as f32;
    let connectivity = connections as f32 / (total_strength * 4.0).max(1.0);
    
    (density * 0.5 + connectivity * 0.5).min(1.0)
}

fn generate_player_color(existing_players: &HashMap<String, Player>) -> String {
    let colors = vec![
        "#00ffff", "#ff00ff", "#ffff00", "#00ff00",
        "#ff6600", "#0066ff", "#ff0066", "#66ff00"
    ];
    
    let used_colors: Vec<&String> = existing_players.values()
        .map(|p| &p.color)
        .collect();
    
    colors.into_iter()
        .find(|&c| !used_colors.contains(&&c.to_string()))
        .unwrap_or("#ffffff")
        .to_string()
}

#[derive(Debug, Deserialize)]
struct CreateGameParams {
    game_type: GameType,
    max_rounds: Option<u32>,
}

#[derive(Debug, Serialize)]
struct GameInfo {
    id: String,
    status: String,
}

use axum::http::StatusCode;

// Authentication handlers

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<UserInfo>, StatusCode> {
    // Create new user
    let user = state.user_store.create_user(
        req.username,
        req.email,
        req.password,
    ).await
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    Ok(Json(user.into()))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Find user
    let user = state.user_store.get_user_by_username(&req.username).await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Verify password
    if !verify_password(&req.password, &user.password_hash).unwrap_or(false) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    // Create tokens
    let access_claims = auth::Claims::new(user.id.clone(), user.username.clone(), user.role.clone());
    let refresh_claims = auth::Claims::new_refresh(user.id.clone(), user.username.clone(), user.role.clone());
    
    let access_token = create_jwt(&access_claims)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let refresh_token = create_jwt(&refresh_claims)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 86400, // 24 hours
        user: user.into(),
    }))
}

async fn get_profile(claims: Claims) -> Result<Json<UserInfo>, StatusCode> {
    Ok(Json(UserInfo {
        id: claims.sub,
        username: claims.username,
        email: "".to_string(), // Would be fetched from user store in real app
        role: claims.role,
    }))
}

async fn auth_middleware(
    State(_state): State<Arc<AppState>>,
    mut request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    // Extract token from Authorization header
    let auth_header = request.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    
    let token = match auth_header {
        Some(value) if value.starts_with("Bearer ") => &value[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    
    // Validate token
    let claims = auth::validate_jwt(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Insert claims into request extensions so handlers can access them
    request.extensions_mut().insert(claims);
    
    Ok(next.run(request).await)
}