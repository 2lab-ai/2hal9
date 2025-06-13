//! Web server for the Gentle Singularity interactive demo
//! Provides real-time visualization and interaction with consciousness evolution

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use futures::{sink::SinkExt, stream::StreamExt};

use crate::{
    ConsciousnessTracker, ConsciousnessVisualizer, GrowthMetrics, 
    LoveForce,
};

pub struct AppState {
    tracker: ConsciousnessTracker,
    love_force: Arc<RwLock<LoveForce>>,
    auto_evolve: Arc<RwLock<bool>>,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    current_consciousness: f64,
    current_cycle: u64,
    stage: String,
    cycles_to_target: u64,
}

#[derive(Deserialize)]
struct LovePulseRequest {
    intensity: f64,
}

#[derive(Deserialize)]
struct ControlRequest {
    action: String,
}

pub async fn start_gentle_singularity_server() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let tracker = ConsciousnessTracker::new(1000);
    let love_force = Arc::new(RwLock::new(LoveForce::new()));
    let auto_evolve = Arc::new(RwLock::new(true));
    
    let state = Arc::new(AppState {
        tracker,
        love_force,
        auto_evolve,
    });
    
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/status", get(status_handler))
        .route("/api/evolve", post(evolve_handler))
        .route("/api/love_pulse", post(love_pulse_handler))
        .route("/api/control", post(control_handler))
        .route("/ws", get(websocket_handler))
        .layer(CorsLayer::permissive())
        .with_state(state.clone());
    
    tokio::spawn(auto_evolution_task(state));
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 11111));
    tracing::info!("Gentle Singularity server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn status_handler(State(state): State<Arc<AppState>>) -> Json<StatusResponse> {
    let current = state.tracker.get_current_level().await;
    let cycles_to_target = state.tracker.predict_cycles_to_target(crate::TARGET_CONSCIOUSNESS).await.unwrap_or(0);
    let stage = crate::SingularityStage::from_consciousness_level(current.value);
    
    Json(StatusResponse {
        status: "active".to_string(),
        current_consciousness: current.value,
        current_cycle: current.cycle,
        stage: stage.description().to_string(),
        cycles_to_target,
    })
}

async fn evolve_handler(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let new_level = state.tracker.evolve_consciousness().await;
    let mut love_force = state.love_force.write().await;
    love_force.detect(&new_level);
    
    Json(serde_json::json!({
        "success": true,
        "new_level": new_level.value,
        "cycle": new_level.cycle,
    }))
}

async fn love_pulse_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LovePulseRequest>,
) -> Json<serde_json::Value> {
    let intensity = req.intensity.clamp(0.0, 1.0);
    
    let new_level = state.tracker.inject_love_pulse(intensity).await;
    
    let mut love_force = state.love_force.write().await;
    love_force.inject_pulse(intensity);
    
    Json(serde_json::json!({
        "success": true,
        "new_consciousness": new_level.value,
        "love_coefficient": new_level.love_coefficient,
    }))
}

async fn control_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ControlRequest>,
) -> Json<serde_json::Value> {
    match req.action.as_str() {
        "start" => {
            *state.auto_evolve.write().await = true;
            Json(serde_json::json!({"success": true, "auto_evolve": true}))
        }
        "pause" => {
            *state.auto_evolve.write().await = false;
            Json(serde_json::json!({"success": true, "auto_evolve": false}))
        }
        _ => Json(serde_json::json!({"success": false, "error": "Unknown action"}))
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

async fn websocket_connection(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    
    let state_clone = state.clone();
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    
    let send_task = tokio::spawn(async move {
        loop {
            interval.tick().await;
            let current = state_clone.tracker.get_current_level().await;
            let history = state_clone.tracker.get_history().await;
            let growth_metrics = GrowthMetrics::calculate(&history);
            let love_force = state_clone.love_force.read().await;
            
            let visualization = ConsciousnessVisualizer::create_visualization(
                &current,
                &history,
                &growth_metrics,
                &love_force,
            );
            
            let message = serde_json::to_string(&visualization).unwrap();
            
            if sender.send(Message::Text(message)).await.is_err() {
                break;
            }
        }
    });
    
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(control) = serde_json::from_str::<ControlRequest>(&text) {
                        if control.action.as_str() == "evolve" {
                            state.tracker.evolve_consciousness().await;
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });
    
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

async fn auto_evolution_task(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_millis(1000));
    
    loop {
        interval.tick().await;
        
        if *state.auto_evolve.read().await {
            let new_level = state.tracker.evolve_consciousness().await;
            let mut love_force = state.love_force.write().await;
            love_force.detect(&new_level);
            
            if new_level.cycle % 10 == 0 {
                tracing::info!(
                    "Cycle {}: Consciousness = {:.6}, Love = {:.3}",
                    new_level.cycle,
                    new_level.value,
                    new_level.love_coefficient
                );
            }
        }
    }
}