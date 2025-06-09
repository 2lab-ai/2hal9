//! HTTP API endpoints for HAL9 server

use axum::{
    extract::{State, Json, Path, Query},
    response::{IntoResponse, Response},
    routing::{get, post, put, delete},
    Router,
    http::StatusCode,
    middleware,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use crate::{
    server::HAL9Server, 
    error::ServerError,
    auth_middleware::{auth_middleware as auth_mw, AuthState},
    api_auth,
    api_codegen,
};
use hal9_core::NeuronSignal;

pub mod graphql;

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
    network_status: Option<crate::server::NetworkStatus>,
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
    let mut router = Router::new()
        // Health check (no auth)
        .route("/health", get(health_check))
        
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
        
        // Prometheus metrics endpoint
        .route("/metrics", get(prometheus_metrics))
        
        // Network status
        .route("/api/v1/network/status", get(get_network_status))
        
        // WebSocket endpoint for real-time updates
        .route("/api/v1/ws", get(websocket_handler))
        
        // Add CORS support
        .layer(CorsLayer::permissive())
        .with_state(server.clone());
    
    // Add authentication routes if enabled
    if server.jwt_manager.is_some() {
        let auth_state = AuthState {
            jwt_manager: server.jwt_manager.clone().unwrap(),
            api_key_manager: server.api_key_manager.clone().unwrap(),
        };
        
        let api_auth_state = Arc::new(api_auth::AuthApiState {
            user_manager: server.user_manager.clone().unwrap(),
            jwt_manager: server.jwt_manager.clone().unwrap(),
            api_key_manager: server.api_key_manager.clone().unwrap(),
        });
        
        // Create auth router with public endpoints
        let auth_router = Router::new()
            .route("/api/v1/auth/register", post(api_auth::register))
            .route("/api/v1/auth/login", post(api_auth::login))
            .route("/api/v1/auth/refresh", post(api_auth::refresh_token))
            .with_state(api_auth_state.clone());
        
        // Create protected auth router
        let protected_auth_router = Router::new()
            .route("/api/v1/auth/profile", get(api_auth::get_profile))
            .route("/api/v1/auth/profile", put(api_auth::update_profile))
            .route("/api/v1/auth/api-keys", post(api_auth::create_api_key))
            .route("/api/v1/auth/api-keys", get(api_auth::list_api_keys))
            .route("/api/v1/auth/api-keys/:id", put(api_auth::revoke_api_key))
            .route("/api/v1/auth/api-keys/:id", delete(api_auth::delete_api_key))
            .layer(middleware::from_fn_with_state(auth_state.clone(), auth_mw))
            .with_state(api_auth_state);
        
        router = router.merge(auth_router).merge(protected_auth_router);
    }
    
    // Add code generation routes if configured
    let codegen_state = Arc::new(api_codegen::CodegenApiState {
        server: server.clone(),
    });
    
    let codegen_router = Router::new()
        .route("/api/v1/codegen/health", get(api_codegen::codegen_health))
        .route("/api/v1/codegen/templates", get(api_codegen::list_templates))
        .route("/api/v1/codegen/project", post(api_codegen::generate_project))
        .route("/api/v1/codegen/project/:id", get(api_codegen::get_project_status))
        .route("/api/v1/codegen/complete", post(api_codegen::code_completion))
        .route("/api/v1/codegen/review", post(api_codegen::review_code))
        .route("/api/v1/codegen/refactor", post(api_codegen::refactor_code))
        .with_state(codegen_state);
    
    router = router.merge(codegen_router);
    
    // Add GraphQL endpoints if enabled
    #[cfg(feature = "graphql")]
    {
        use crate::api::graphql::{create_graphql_schema, graphql_routes, EventBus};
        use tokio::sync::RwLock;
        
        // Create GraphQL schema with all required services
        let event_bus = Arc::new(EventBus::new(10000));
        let graphql_schema = create_graphql_schema(
            server.db.clone(),
            auth_state.auth_service.clone(),
            server.org_service.clone(),
            server.team_service.clone(),
            Arc::new(RwLock::new(server.neuron_manager.clone())),
            server.router.clone(),
            server.memory_manager.clone(),
            server.metrics.clone(),
            event_bus,
        );
        
        // Add GraphQL routes
        router = router.merge(graphql_routes(graphql_schema));
    }
    
    router
}

// Handler implementations

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "hal9-server",
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
        network_status: status.network_status,
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
            let prometheus_data = crate::prometheus_exporter::export_metrics(server).await;
            Ok((StatusCode::OK, prometheus_data).into_response())
        }
        _ => Ok(Json(ApiResponse::<String>::error("Unsupported format")).into_response()),
    }
}

async fn prometheus_metrics(
    State(server): State<Arc<HAL9Server>>,
) -> Result<impl IntoResponse, ServerError> {
    let prometheus_data = crate::prometheus_exporter::export_metrics(server).await;
    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        prometheus_data,
    ))
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