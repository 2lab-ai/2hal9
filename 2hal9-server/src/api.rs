//! HTTP API endpoints for 2HAL9 server

use axum::{
    extract::{State, Json, Path, Query},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use crate::{server::HAL9Server, error::ServerError};
use twohal9_core::NeuronSignal;

/// API response wrapper
#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

/// Signal submission request
#[derive(Debug, Deserialize)]
struct SubmitSignalRequest {
    content: String,
    layer: String,
    neuron_id: Option<String>,
}

/// Server status response
#[derive(Debug, Serialize)]
struct ServerStatus {
    running: bool,
    uptime_seconds: u64,
    neurons: Vec<NeuronStatus>,
    metrics: MetricsSummary,
}

/// Individual neuron status
#[derive(Debug, Serialize)]
struct NeuronStatus {
    id: String,
    layer: String,
    state: String,
    health: String,
}

/// Metrics summary
#[derive(Debug, Serialize)]
struct MetricsSummary {
    signals_sent: u64,
    signals_processed: u64,
    signals_failed: u64,
    average_latency_ms: f64,
}

/// Signal trace response
#[derive(Debug, Serialize)]
struct SignalTrace {
    signal_id: String,
    trace: Vec<TraceStep>,
}

#[derive(Debug, Serialize)]
struct TraceStep {
    timestamp: String,
    neuron_id: String,
    layer: String,
    action: String,
    response: Option<String>,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    SignalUpdate {
        signal_id: String,
        neuron_id: String,
        status: String,
    },
    NeuronStateChange {
        neuron_id: String,
        old_state: String,
        new_state: String,
    },
    ServerEvent {
        event: String,
        details: String,
    },
}

/// Create the HTTP API router
pub fn create_api_router(server: Arc<HAL9Server>) -> Router {
    Router::new()
        // Core endpoints
        .route("/api/v1/status", get(get_status))
        .route("/api/v1/signal", post(submit_signal))
        .route("/api/v1/signal/:id", get(get_signal_trace))
        
        // Neuron management
        .route("/api/v1/neurons", get(list_neurons))
        .route("/api/v1/neurons/:id", get(get_neuron))
        .route("/api/v1/neurons/:id/health", get(get_neuron_health))
        
        // Metrics
        .route("/api/v1/metrics", get(get_metrics))
        .route("/api/v1/metrics/export", get(export_metrics))
        
        // Network status
        .route("/api/v1/network/status", get(get_network_status))
        
        // WebSocket endpoint for real-time updates
        .route("/api/v1/ws", get(websocket_handler))
        
        // Health check
        .route("/health", get(health_check))
        
        // Add CORS support
        .layer(CorsLayer::permissive())
        .with_state(server)
}

// Handler implementations

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "2hal9-server",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn get_status(
    State(server): State<Arc<HAL9Server>>,
) -> Result<impl IntoResponse, ServerError> {
    let status = server.get_status().await?;
    
    let response = ServerStatus {
        running: status.running,
        uptime_seconds: status.uptime.as_secs(),
        neurons: status.neurons.into_iter().map(|n| NeuronStatus {
            id: n.id,
            layer: format!("{:?}", n.layer),
            state: format!("{:?}", n.state),
            health: if n.is_healthy { "healthy".to_string() } else { "unhealthy".to_string() },
        }).collect(),
        metrics: MetricsSummary {
            signals_sent: status.metrics.signals_sent,
            signals_processed: status.metrics.signals_processed,
            signals_failed: status.metrics.signals_failed,
            average_latency_ms: calculate_average_latency(&status.metrics),
        },
    };
    
    Ok(Json(ApiResponse::success(response)))
}

async fn submit_signal(
    State(server): State<Arc<HAL9Server>>,
    Json(req): Json<SubmitSignalRequest>,
) -> Result<impl IntoResponse, ServerError> {
    // Parse layer
    let layer_str = match req.layer.to_lowercase().as_str() {
        "l4" | "strategic" => "L4",
        "l3" | "design" => "L3",
        "l2" | "implementation" => "L2",
        "l1" | "execution" => "L1",
        _ => return Ok(Json(ApiResponse::error("Invalid layer specified"))),
    };
    
    // Create signal
    let signal = NeuronSignal::forward(
        &"api-client",
        &req.neuron_id.unwrap_or_else(|| format!("neuron-{}", layer_str.to_lowercase())),
        "API",
        layer_str,
        req.content,
    );
    
    // Submit to server
    match server.submit_signal(signal).await {
        Ok(signal_id) => Ok(Json(ApiResponse::success(serde_json::json!({
            "signal_id": signal_id,
            "message": "Signal submitted successfully"
        })))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to submit signal: {}", e)))),
    }
}

async fn get_signal_trace(
    State(_server): State<Arc<HAL9Server>>,
    Path(_signal_id): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    // This would require implementing signal tracing in the server
    // For now, return a placeholder
    Ok(Json(ApiResponse::<SignalTrace>::error("Signal tracing not yet implemented")))
}

async fn list_neurons(
    State(server): State<Arc<HAL9Server>>,
) -> Result<impl IntoResponse, ServerError> {
    let neurons = server.list_neurons().await?;
    Ok(Json(ApiResponse::success(neurons)))
}

async fn get_neuron(
    State(server): State<Arc<HAL9Server>>,
    Path(neuron_id): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    match server.get_neuron_info(&neuron_id).await {
        Ok(info) => Ok(Json(ApiResponse::success(info))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Neuron not found: {}", e)))),
    }
}

async fn get_neuron_health(
    State(server): State<Arc<HAL9Server>>,
    Path(neuron_id): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    match server.get_neuron_health(&neuron_id).await {
        Ok(health) => Ok(Json(ApiResponse::success(health))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to get health: {}", e)))),
    }
}

async fn get_metrics(
    State(server): State<Arc<HAL9Server>>,
) -> Result<impl IntoResponse, ServerError> {
    let metrics = server.get_metrics().await?;
    Ok(Json(ApiResponse::success(metrics)))
}

async fn export_metrics(
    State(server): State<Arc<HAL9Server>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, ServerError> {
    let format = params.get("format").map(|s| s.as_str()).unwrap_or("json");
    
    match format {
        "json" => {
            let metrics = server.get_metrics().await?;
            Ok(Json(metrics).into_response())
        }
        "prometheus" => {
            // TODO: Implement Prometheus format export
            Ok(Json(ApiResponse::<String>::error("Prometheus format not yet implemented")).into_response())
        }
        _ => Ok(Json(ApiResponse::<String>::error("Unsupported format")).into_response()),
    }
}

async fn get_network_status(
    State(server): State<Arc<HAL9Server>>,
) -> Result<impl IntoResponse, ServerError> {
    match server.network_status().await {
        Some(status) => Ok(Json(ApiResponse::success(status))),
        None => {
            let disabled_status = crate::server::NetworkStatus {
                enabled: false,
                server_id: String::new(),
                connected_servers: vec![],
                remote_neurons: 0,
            };
            Ok(Json(ApiResponse::success(disabled_status)))
        }
    }
}

async fn websocket_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    State(server): State<Arc<HAL9Server>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, server))
}

async fn handle_websocket(
    mut socket: axum::extract::ws::WebSocket,
    server: Arc<HAL9Server>,
) {
    use axum::extract::ws::Message;
    
    // Subscribe to server events
    let mut event_rx = server.subscribe_to_events().await;
    
    // Send initial connection message
    let welcome_msg = WsMessage::ServerEvent {
        event: "connected".to_string(),
        details: "Connected to 2HAL9 server".to_string(),
    };
    
    if let Ok(msg_json) = serde_json::to_string(&welcome_msg) {
        let _ = socket.send(Message::Text(msg_json)).await;
    }
    
    // Handle bidirectional communication
    loop {
        tokio::select! {
            // Handle incoming messages from client
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(msg) => match msg {
                    Message::Text(text) => {
                        // Handle text messages if needed
                        tracing::debug!("Received WebSocket message: {}", text);
                    }
                    Message::Close(_) => {
                        tracing::info!("WebSocket client disconnected");
                        break;
                    }
                    _ => {}
                    },
                    Err(_) => break,
                }
            }
            
            // Forward server events to client
            Ok(event) = event_rx.recv() => {
                if let Ok(msg_json) = serde_json::to_string(&event) {
                    if socket.send(Message::Text(msg_json)).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
}

// Error handling
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ServerError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ServerError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            ServerError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        
        let response = ApiResponse::<()>::error(message);
        (status, Json(response)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_response_serialization() {
        let response = ApiResponse::success("test data");
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"data\":\"test data\""));
    }
    
    #[test]
    fn test_error_response() {
        let response = ApiResponse::<String>::error("Something went wrong");
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"error\":\"Something went wrong\""));
    }
}

// Helper functions

fn calculate_average_latency(metrics: &crate::metrics::MetricsSnapshot) -> f64 {
    if metrics.layer_latencies.is_empty() {
        return 0.0;
    }
    
    let total_count: u64 = metrics.layer_latencies.values().map(|s| s.count).sum();
    let weighted_sum: f64 = metrics.layer_latencies.values()
        .map(|s| s.avg_ms * s.count as f64)
        .sum();
    
    if total_count > 0 {
        weighted_sum / total_count as f64
    } else {
        0.0
    }
}