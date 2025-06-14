use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use crate::{
    games::{GameConfig, GameEngine, GameType, Action},
    collective::{CollectiveIntelligence, CollectiveConfig, CollectiveType},
    sota::{SOTAManager, SOTAConfig, ThinkingTime},
    streaming::StreamingEngine,
    analytics::AnalyticsEngine,
};
use std::collections::HashMap;

pub struct GeniusGameServer {
    game_engine: Arc<GameEngine>,
    collective_players: Arc<RwLock<HashMap<String, CollectiveIntelligence>>>,
    sota_players: Arc<RwLock<HashMap<String, SOTAManager>>>,
    streaming_engine: Arc<StreamingEngine>,
    analytics_engine: Arc<AnalyticsEngine>,
}

impl Default for GeniusGameServer {
    fn default() -> Self {
        Self {
            game_engine: Arc::new(GameEngine::new()),
            collective_players: Arc::new(RwLock::new(HashMap::new())),
            sota_players: Arc::new(RwLock::new(HashMap::new())),
            streaming_engine: Arc::new(StreamingEngine::new()),
            analytics_engine: Arc::new(AnalyticsEngine::new()),
        }
    }
}

impl GeniusGameServer {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub async fn run(self, addr: &str) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/", get(index))
            .route("/ws", get(ws_handler))
            .route("/api/game/create", post(create_game))
            .route("/api/game/:id/turn", post(process_turn))
            .route("/api/game/:id/state", get(get_game_state))
            .route("/api/player/collective/create", post(create_collective_player))
            .route("/api/player/sota/create", post(create_sota_player))
            .route("/api/analytics/:game_id", get(get_analytics))
            .layer(CorsLayer::permissive())
            .with_state(Arc::new(self));
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("Genius Game Server listening on {}", addr);
        
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn index() -> impl IntoResponse {
    "AI Genius Game Server v0.1.0"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(server): State<Arc<GeniusGameServer>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, server))
}

async fn handle_socket(socket: axum::extract::ws::WebSocket, server: Arc<GeniusGameServer>) {
    server.streaming_engine.handle_connection(socket).await;
}

#[derive(serde::Deserialize)]
struct CreateGameRequest {
    game_type: GameType,
    rounds: u32,
    time_limit_ms: u64,
    collective_players: Vec<String>,
    sota_players: Vec<String>,
}

async fn create_game(
    State(server): State<Arc<GeniusGameServer>>,
    Json(req): Json<CreateGameRequest>,
) -> impl IntoResponse {
    let config = GameConfig {
        game_type: req.game_type,
        rounds: req.rounds,
        time_limit_ms: req.time_limit_ms,
        special_rules: HashMap::new(),
    };
    
    match server.game_engine.create_game(config).await {
        Ok(game_id) => {
            // Initialize players for this game
            let _collective_players = server.collective_players.read().await;
            let _sota_players = server.sota_players.read().await;
            
            // Notify streaming engine
            server.streaming_engine.game_started(game_id, 
                req.collective_players.clone(), 
                req.sota_players.clone()
            ).await;
            
            Json(serde_json::json!({
                "game_id": game_id,
                "status": "created"
            }))
        }
        Err(e) => Json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[derive(serde::Deserialize)]
struct ProcessTurnRequest {
    actions: HashMap<String, serde_json::Value>,
}

async fn process_turn(
    State(server): State<Arc<GeniusGameServer>>,
    axum::extract::Path(game_id): axum::extract::Path<Uuid>,
    Json(req): Json<ProcessTurnRequest>,
) -> impl IntoResponse {
    // Get game state
    let _game_state = match server.game_engine.get_game_state(game_id).await {
        Some(state) => state,
        None => return Json(serde_json::json!({"error": "Game not found"})),
    };
    
    // Collect decisions from all players
    let mut all_actions = HashMap::new();
    
    // Collective players
    let mut collective_players = server.collective_players.write().await;
    for (player_id, action_context) in &req.actions {
        if player_id.starts_with("collective_") {
            if let Some(collective) = collective_players.get_mut(player_id) {
                match collective.make_decision(action_context.clone()).await {
                    Ok(decision) => {
                        let action = Action {
                            player_id: player_id.clone(),
                            action_type: "decision".to_string(),
                            data: decision.final_decision.clone(),
                            reasoning: Some(format!("Consensus: {}", decision.consensus_method)),
                            confidence: Some(1.0 - decision.dissent_rate),
                        };
                        all_actions.insert(player_id.clone(), action);
                        
                        // Stream collective internals
                        server.streaming_engine.update_collective_state(
                            game_id,
                            player_id.clone(),
                            decision
                        ).await;
                    }
                    Err(e) => tracing::error!("Collective decision error: {}", e),
                }
            }
        }
    }
    
    // SOTA players
    let mut sota_players = server.sota_players.write().await;
    for (player_id, action_context) in &req.actions {
        if player_id.starts_with("sota_") {
            if let Some(sota) = sota_players.get_mut(player_id) {
                match sota.make_decision(action_context.clone()).await {
                    Ok(decision) => {
                        let action = Action {
                            player_id: player_id.clone(),
                            action_type: "decision".to_string(),
                            data: decision.decision.clone(),
                            reasoning: Some(decision.reasoning_chain.join(" -> ")),
                            confidence: Some(decision.confidence),
                        };
                        all_actions.insert(player_id.clone(), action);
                        
                        // Stream SOTA reasoning
                        server.streaming_engine.update_sota_state(
                            game_id,
                            player_id.clone(),
                            decision
                        ).await;
                    }
                    Err(e) => tracing::error!("SOTA decision error: {}", e),
                }
            }
        }
    }
    
    // Process game turn
    match server.game_engine.process_turn(game_id, all_actions).await {
        Ok(round_result) => {
            // Update analytics
            server.analytics_engine.process_round(game_id, &round_result).await;
            
            // Stream game state update
            if let Some(new_state) = server.game_engine.get_game_state(game_id).await {
                server.streaming_engine.update_game_state(game_id, new_state).await;
            }
            
            // Check if game is over
            if server.game_engine.is_game_finished(game_id).await {
                match server.game_engine.finalize_game(game_id).await {
                    Ok(final_result) => {
                        server.streaming_engine.game_ended(game_id, final_result.clone()).await;
                        Json(serde_json::json!({
                            "status": "game_over",
                            "result": final_result
                        }))
                    }
                    Err(e) => Json(serde_json::json!({
                        "error": e.to_string()
                    }))
                }
            } else {
                Json(serde_json::json!({
                    "status": "turn_processed",
                    "round_result": round_result
                }))
            }
        }
        Err(e) => Json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

async fn get_game_state(
    State(server): State<Arc<GeniusGameServer>>,
    axum::extract::Path(game_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match server.game_engine.get_game_state(game_id).await {
        Some(state) => Json(serde_json::json!(state)),
        None => Json(serde_json::json!({"error": "Game not found"})),
    }
}

#[derive(serde::Deserialize)]
struct CreateCollectiveRequest {
    name: String,
    config_type: CollectiveType,
}

async fn create_collective_player(
    State(server): State<Arc<GeniusGameServer>>,
    Json(req): Json<CreateCollectiveRequest>,
) -> impl IntoResponse {
    let player_id = format!("collective_{}", Uuid::new_v4());
    
    let config = match req.config_type {
        CollectiveType::OpusOrchestra => CollectiveConfig {
            name: req.name,
            config_type: CollectiveType::OpusOrchestra,
            models: vec![],  // Would be populated with real configs
            coordination: crate::collective::CoordinationStrategy::HierarchicalDemocracy {
                master: "master_strategist".to_string()
            },
            cost_per_hour: 120.0,
        },
        CollectiveType::SwarmIntelligence => CollectiveConfig {
            name: req.name,
            config_type: CollectiveType::SwarmIntelligence,
            models: vec![],
            coordination: crate::collective::CoordinationStrategy::EmergentConsensus {
                communication: "local_only".to_string()
            },
            cost_per_hour: 2.0,
        },
        _ => return Json(serde_json::json!({"error": "Config type not implemented"})),
    };
    
    let collective = CollectiveIntelligence::new(player_id.clone(), config);
    server.collective_players.write().await.insert(player_id.clone(), collective);
    
    Json(serde_json::json!({
        "player_id": player_id,
        "type": "collective"
    }))
}

#[derive(serde::Deserialize)]
struct CreateSOTARequest {
    model_name: String,
    api_key: String,
}

async fn create_sota_player(
    State(server): State<Arc<GeniusGameServer>>,
    Json(req): Json<CreateSOTARequest>,
) -> impl IntoResponse {
    let player_id = format!("sota_{}", Uuid::new_v4());
    
    let config = SOTAConfig {
        model_name: req.model_name.clone(),
        api_key: req.api_key,
        context_window: 100000,
        thinking_time: ThinkingTime::Extended,
        temperature: 0.7,
        tools: vec![],
        cost_per_hour: 25.0,
    };
    
    let sota = SOTAManager::new(player_id.clone(), config);
    server.sota_players.write().await.insert(player_id.clone(), sota);
    
    Json(serde_json::json!({
        "player_id": player_id,
        "type": "sota"
    }))
}

async fn get_analytics(
    State(server): State<Arc<GeniusGameServer>>,
    axum::extract::Path(game_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    let analytics = server.analytics_engine.get_game_analytics(game_id).await;
    Json(analytics)
}