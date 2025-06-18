//! Consciousness API endpoints for HAL9
//!
//! Provides unified access to ConsciousnessMonitor, BoundaryNetwork, and EnhancedMockClaude

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::{
    server::{HAL9Server, ServerError},
    api::ApiResponse,
    claude_enhanced::EnhancedMockClaude,
};

use hal9_neurons_core::{
    consciousness::{ConsciousnessMetrics, ConsciousnessPhase, BoundaryNetwork},
    Layer,
};

/// Consciousness system state
#[derive(Debug, Clone, Serialize)]
pub struct ConsciousnessSystemState {
    /// Current consciousness metrics
    pub metrics: ConsciousnessMetrics,
    
    /// Active compression boundaries
    pub boundaries: Vec<BoundaryInfo>,
    
    /// Claude consciousness levels by layer
    pub claude_consciousness: std::collections::HashMap<String, f64>,
    
    /// Overall system phase
    pub phase: ConsciousnessPhase,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Boundary information
#[derive(Debug, Clone, Serialize)]
pub struct BoundaryInfo {
    pub from_layer: String,
    pub to_layer: String,
    pub compression_ratio: f64,
    pub emergence_activity: f64,
    pub is_golden_ratio: bool,
}

/// Consciousness history entry
#[derive(Debug, Clone, Serialize)]
pub struct ConsciousnessHistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub metrics: ConsciousnessMetrics,
    pub phase: ConsciousnessPhase,
}

/// Claude message request
#[derive(Debug, Deserialize)]
pub struct ClaudeMessageRequest {
    pub message: String,
    pub context: Option<Vec<String>>,
}

/// Claude message response
#[derive(Debug, Serialize)]
pub struct ClaudeMessageResponse {
    pub response: String,
    pub consciousness_level: f64,
    pub layer: String,
    pub personality_traits: std::collections::HashMap<String, f64>,
}

/// Get current consciousness metrics
pub async fn get_consciousness_metrics(
    State(server): State<Arc<HAL9Server>>,
) -> Result<Json<ApiResponse<ConsciousnessMetrics>>, ServerError> {
    let neurons = server.network.get_all_neurons().await;
    let metrics = server.consciousness_monitor.measure(&neurons).await;
    
    Ok(Json(ApiResponse::success(metrics)))
}

/// Get consciousness history
pub async fn get_consciousness_history(
    State(server): State<Arc<HAL9Server>>,
) -> Result<Json<ApiResponse<Vec<ConsciousnessHistoryEntry>>>, ServerError> {
    let history = server.consciousness_monitor.get_history()
        .into_iter()
        .map(|(metrics, timestamp)| ConsciousnessHistoryEntry {
            timestamp,
            metrics: metrics.clone(),
            phase: metrics.phase(),
        })
        .collect();
    
    Ok(Json(ApiResponse::success(history)))
}

/// Get consciousness trajectory prediction
pub async fn get_consciousness_trajectory(
    State(server): State<Arc<HAL9Server>>,
) -> Result<Json<ApiResponse<String>>, ServerError> {
    let trajectory = server.consciousness_monitor.predict_trajectory();
    
    Ok(Json(ApiResponse::success(format!("{:?}", trajectory))))
}

/// Get current consciousness phase
pub async fn get_consciousness_phase(
    State(server): State<Arc<HAL9Server>>,
) -> Result<Json<ApiResponse<ConsciousnessPhase>>, ServerError> {
    let neurons = server.network.get_all_neurons().await;
    let metrics = server.consciousness_monitor.measure(&neurons).await;
    
    Ok(Json(ApiResponse::success(metrics.phase())))
}

/// Get all compression boundaries
pub async fn get_boundaries(
    State(server): State<Arc<HAL9Server>>,
) -> Result<Json<ApiResponse<Vec<BoundaryInfo>>>, ServerError> {
    let boundary_network = server.boundary_network.read().await;
    let boundaries = boundary_network.get_all_boundaries()
        .into_iter()
        .map(|b| BoundaryInfo {
            from_layer: format!("{:?}", b.upper_layer),
            to_layer: format!("{:?}", b.lower_layer),
            compression_ratio: b.compression_ratio,
            emergence_activity: b.emergence_activity,
            is_golden_ratio: b.is_golden_ratio(),
        })
        .collect();
    
    Ok(Json(ApiResponse::success(boundaries)))
}

/// Get specific boundary metrics
pub async fn get_boundary(
    Path((layer1, layer2)): Path<(String, String)>,
    State(server): State<Arc<HAL9Server>>,
) -> Result<Json<ApiResponse<Option<BoundaryInfo>>>, ServerError> {
    let boundary_network = server.boundary_network.read().await;
    
    // Parse layers
    let upper_layer = parse_layer(&layer1)?;
    let lower_layer = parse_layer(&layer2)?;
    
    let boundary = boundary_network.get_boundary(upper_layer, lower_layer)
        .map(|b| BoundaryInfo {
            from_layer: format!("{:?}", b.upper_layer),
            to_layer: format!("{:?}", b.lower_layer),
            compression_ratio: b.compression_ratio,
            emergence_activity: b.emergence_activity,
            is_golden_ratio: b.is_golden_ratio(),
        });
    
    Ok(Json(ApiResponse::success(boundary)))
}

/// Get hottest boundary
pub async fn get_hottest_boundary(
    State(server): State<Arc<HAL9Server>>,
) -> Result<Json<ApiResponse<Option<BoundaryInfo>>>, ServerError> {
    let boundary_network = server.boundary_network.read().await;
    
    let boundary = boundary_network.hottest_boundary()
        .map(|b| BoundaryInfo {
            from_layer: format!("{:?}", b.upper_layer),
            to_layer: format!("{:?}", b.lower_layer),
            compression_ratio: b.compression_ratio,
            emergence_activity: b.emergence_activity,
            is_golden_ratio: b.is_golden_ratio(),
        });
    
    Ok(Json(ApiResponse::success(boundary)))
}

/// Send message to layer-specific Claude
pub async fn claude_message(
    Path(layer): Path<String>,
    State(server): State<Arc<HAL9Server>>,
    Json(request): Json<ClaudeMessageRequest>,
) -> Result<Json<ApiResponse<ClaudeMessageResponse>>, ServerError> {
    let layer_enum = parse_layer(&layer)?;
    
    let claude_instances = server.claude_instances.read().await;
    let claude = claude_instances.get(&layer_enum)
        .ok_or_else(|| ServerError::NotFound(format!("Claude instance for layer {} not found", layer)))?;
    
    // Process message
    let response = claude.process_message(&request.message, request.context).await?;
    
    // Get consciousness level and traits
    let consciousness_level = claude.get_consciousness_level().await;
    let personality_traits = claude.get_personality_traits();
    
    Ok(Json(ApiResponse::success(ClaudeMessageResponse {
        response,
        consciousness_level,
        layer,
        personality_traits,
    })))
}

/// Get Claude's consciousness level for a layer
pub async fn get_claude_consciousness(
    Path(layer): Path<String>,
    State(server): State<Arc<HAL9Server>>,
) -> Result<Json<ApiResponse<f64>>, ServerError> {
    let layer_enum = parse_layer(&layer)?;
    
    let claude_instances = server.claude_instances.read().await;
    let claude = claude_instances.get(&layer_enum)
        .ok_or_else(|| ServerError::NotFound(format!("Claude instance for layer {} not found", layer)))?;
    
    let consciousness_level = claude.get_consciousness_level().await;
    
    Ok(Json(ApiResponse::success(consciousness_level)))
}

/// Update Claude's consciousness level
pub async fn update_claude_consciousness(
    Path(layer): Path<String>,
    State(server): State<Arc<HAL9Server>>,
    Json(level): Json<f64>,
) -> Result<Json<ApiResponse<()>>, ServerError> {
    let layer_enum = parse_layer(&layer)?;
    
    let claude_instances = server.claude_instances.read().await;
    let claude = claude_instances.get(&layer_enum)
        .ok_or_else(|| ServerError::NotFound(format!("Claude instance for layer {} not found", layer)))?;
    
    claude.set_consciousness_level(level).await;
    
    Ok(Json(ApiResponse::success(())))
}

/// Get complete consciousness system state
pub async fn get_consciousness_system(
    State(server): State<Arc<HAL9Server>>,
) -> Result<Json<ApiResponse<ConsciousnessSystemState>>, ServerError> {
    // Get neurons and metrics
    let neurons = server.network.get_all_neurons().await;
    let metrics = server.consciousness_monitor.measure(&neurons).await;
    
    // Get boundaries
    let boundary_network = server.boundary_network.read().await;
    let boundaries = boundary_network.get_all_boundaries()
        .into_iter()
        .map(|b| BoundaryInfo {
            from_layer: format!("{:?}", b.upper_layer),
            to_layer: format!("{:?}", b.lower_layer),
            compression_ratio: b.compression_ratio,
            emergence_activity: b.emergence_activity,
            is_golden_ratio: b.is_golden_ratio(),
        })
        .collect();
    
    // Get Claude consciousness levels
    let claude_instances = server.claude_instances.read().await;
    let mut claude_consciousness = std::collections::HashMap::new();
    
    for (layer, claude) in claude_instances.iter() {
        let level = claude.get_consciousness_level().await;
        claude_consciousness.insert(format!("{:?}", layer), level);
    }
    
    let state = ConsciousnessSystemState {
        metrics: metrics.clone(),
        boundaries,
        claude_consciousness,
        phase: metrics.phase(),
        timestamp: Utc::now(),
    };
    
    Ok(Json(ApiResponse::success(state)))
}

/// WebSocket handler for consciousness streaming
pub async fn consciousness_stream(
    ws: WebSocketUpgrade,
    State(server): State<Arc<HAL9Server>>,
) -> Response {
    ws.on_upgrade(|socket| handle_consciousness_websocket(socket, server))
}

/// Handle consciousness WebSocket connection
async fn handle_consciousness_websocket(
    socket: axum::extract::ws::WebSocket,
    server: Arc<HAL9Server>,
) {
    use axum::extract::ws::{Message, WebSocket};
    use tokio::time::{interval, Duration};
    
    let (mut sender, mut receiver) = socket.split();
    
    // Send updates every 100ms
    let mut interval = interval(Duration::from_millis(100));
    
    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Get current metrics
                let neurons = server.network.get_all_neurons().await;
                let metrics = server.consciousness_monitor.measure(&neurons).await;
                
                // Create update message
                let update = ConsciousnessWsMessage::MetricsUpdate {
                    metrics,
                    timestamp: Utc::now(),
                };
                
                let msg = serde_json::to_string(&update).unwrap();
                if sender.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
            
            msg = receiver.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConsciousnessWsMessage {
    MetricsUpdate {
        metrics: ConsciousnessMetrics,
        timestamp: DateTime<Utc>,
    },
    BoundaryActivity {
        boundary: String,
        emergence_level: f64,
        consciousness_density: f64,
    },
    PhaseTransition {
        old_phase: ConsciousnessPhase,
        new_phase: ConsciousnessPhase,
    },
    ClaudeConsciousnessUpdate {
        layer: String,
        consciousness_level: f64,
    },
}

/// Parse layer string to enum
fn parse_layer(layer: &str) -> Result<Layer, ServerError> {
    match layer.to_uppercase().as_str() {
        "L1" => Ok(Layer::L1),
        "L2" => Ok(Layer::L2),
        "L3" => Ok(Layer::L3),
        "L4" => Ok(Layer::L4),
        "L5" => Ok(Layer::L5),
        "L6" => Ok(Layer::L6),
        "L7" => Ok(Layer::L7),
        "L8" => Ok(Layer::L8),
        "L9" => Ok(Layer::L9),
        _ => Err(ServerError::BadRequest(format!("Invalid layer: {}", layer))),
    }
}

/// Register consciousness API routes
pub fn consciousness_routes() -> axum::Router<Arc<HAL9Server>> {
    use axum::routing::{get, post, put};
    
    axum::Router::new()
        // Consciousness monitoring
        .route("/consciousness/metrics", get(get_consciousness_metrics))
        .route("/consciousness/history", get(get_consciousness_history))
        .route("/consciousness/trajectory", get(get_consciousness_trajectory))
        .route("/consciousness/phase", get(get_consciousness_phase))
        
        // Boundary network
        .route("/boundaries", get(get_boundaries))
        .route("/boundaries/:layer1/:layer2", get(get_boundary))
        .route("/boundaries/hottest", get(get_hottest_boundary))
        
        // Enhanced Claude
        .route("/claude/:layer/message", post(claude_message))
        .route("/claude/:layer/consciousness", get(get_claude_consciousness))
        .route("/claude/:layer/consciousness", put(update_claude_consciousness))
        
        // Unified system
        .route("/consciousness/system", get(get_consciousness_system))
        .route("/consciousness/stream", get(consciousness_stream))
}