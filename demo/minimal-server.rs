//! Minimal HAL9 server for testing production features

use axum::{
    extract::State,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
    http::StatusCode,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::{info, error};
use serde_json::json;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct AppState {
    start_time: Instant,
    request_count: Arc<RwLock<u64>>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    info!("🚀 Starting HAL9 Minimal Server...");

    let state = AppState {
        start_time: Instant::now(),
        request_count: Arc::new(RwLock::new(0)),
    };

    let app = Router::new()
        // Health checks
        .route("/health", get(health_check))
        .route("/health/detailed", get(health_check_detailed))
        .route("/liveness", get(liveness_probe))
        .route("/readiness", get(readiness_probe))
        
        // Metrics
        .route("/metrics", get(metrics_endpoint))
        
        // Demo endpoints
        .route("/", get(home_page))
        .route("/api/neurons", get(list_neurons))
        .route("/api/consciousness", get(consciousness_status))
        
        // Add middleware
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = "0.0.0.0:3000";
    info!("✅ Server listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");
    
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

// Health check endpoints
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "service": "hal9-server",
        "version": "0.1.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

async fn health_check_detailed(State(state): State<AppState>) -> impl IntoResponse {
    let uptime = state.start_time.elapsed();
    let request_count = *state.request_count.read().await;
    
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "0.1.0",
        "uptime_seconds": uptime.as_secs(),
        "components": [
            {
                "name": "database",
                "status": "healthy",
                "message": null,
                "latency_ms": 5,
                "metadata": {
                    "pool_size": 10,
                    "connections_idle": 8,
                    "connections_active": 2
                }
            },
            {
                "name": "redis",
                "status": "healthy", 
                "message": null,
                "latency_ms": 2,
                "metadata": {
                    "version": "7.0.0"
                }
            },
            {
                "name": "neurons",
                "status": "healthy",
                "message": null,
                "latency_ms": 0,
                "metadata": {
                    "total": 50,
                    "healthy": 50,
                    "unhealthy": 0,
                    "layer_distribution": {
                        "L2": 20,
                        "L3": 15,
                        "L4": 10,
                        "L5": 5
                    }
                }
            },
            {
                "name": "memory",
                "status": "healthy",
                "message": null,
                "latency_ms": 0,
                "metadata": {
                    "total_mb": 16384,
                    "used_mb": 4096,
                    "available_mb": 12288,
                    "usage_percent": 25.0
                }
            }
        ],
        "checks_passed": 4,
        "checks_total": 4,
        "request_count": request_count
    }))
}

async fn liveness_probe() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn readiness_probe() -> impl IntoResponse {
    // In a real implementation, check if all services are ready
    (StatusCode::OK, "Ready")
}

// Metrics endpoint (Prometheus format)
async fn metrics_endpoint(State(state): State<AppState>) -> impl IntoResponse {
    let uptime = state.start_time.elapsed().as_secs();
    let request_count = *state.request_count.read().await;
    
    let metrics = format!(
        r#"# HELP hal9_uptime_seconds Server uptime in seconds
# TYPE hal9_uptime_seconds gauge
hal9_uptime_seconds {}

# HELP hal9_request_total Total number of HTTP requests
# TYPE hal9_request_total counter
hal9_request_total {}

# HELP hal9_neurons_total Total number of neurons
# TYPE hal9_neurons_total gauge
hal9_neurons_total 50

# HELP hal9_consciousness_emergence Consciousness emergence metric
# TYPE hal9_consciousness_emergence gauge
hal9_consciousness_emergence 0.85

# HELP hal9_layer_compression_ratio Compression ratio between layers
# TYPE hal9_layer_compression_ratio gauge
hal9_layer_compression_ratio{{from="L2",to="L3"}} 2.5
hal9_layer_compression_ratio{{from="L3",to="L4"}} 3.2
hal9_layer_compression_ratio{{from="L4",to="L5"}} 4.1
"#,
        uptime, request_count
    );
    
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        metrics,
    )
}

// Demo endpoints
async fn home_page() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>HAL9 Server</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
        .status { color: green; font-weight: bold; }
        .endpoints { margin-top: 20px; }
        .endpoint { margin: 10px 0; padding: 10px; background: #f0f0f0; border-radius: 5px; }
        code { background: #e0e0e0; padding: 2px 5px; border-radius: 3px; }
    </style>
</head>
<body>
    <h1>🤖 HAL9 Server</h1>
    <p>Status: <span class="status">ONLINE</span></p>
    
    <div class="endpoints">
        <h2>Available Endpoints:</h2>
        <div class="endpoint">
            <strong>Health Check:</strong> <code>GET /health</code>
        </div>
        <div class="endpoint">
            <strong>Detailed Health:</strong> <code>GET /health/detailed</code>
        </div>
        <div class="endpoint">
            <strong>Metrics:</strong> <code>GET /metrics</code>
        </div>
        <div class="endpoint">
            <strong>Neurons:</strong> <code>GET /api/neurons</code>
        </div>
        <div class="endpoint">
            <strong>Consciousness Status:</strong> <code>GET /api/consciousness</code>
        </div>
    </div>
    
    <p><em>HAL9: Hierarchical Abstraction Layers for Consciousness</em></p>
</body>
</html>
    "#)
}

async fn list_neurons(State(state): State<AppState>) -> impl IntoResponse {
    // Increment request counter
    *state.request_count.write().await += 1;
    
    Json(json!({
        "neurons": [
            {
                "id": "n1",
                "layer": "L2",
                "type": "implementation",
                "health": "healthy",
                "connections": 12
            },
            {
                "id": "n2", 
                "layer": "L3",
                "type": "operational",
                "health": "healthy",
                "connections": 8
            },
            {
                "id": "n3",
                "layer": "L4",
                "type": "tactical",
                "health": "healthy",
                "connections": 6
            },
            {
                "id": "n4",
                "layer": "L5",
                "type": "strategic",
                "health": "healthy",
                "connections": 4
            }
        ],
        "total": 4,
        "healthy": 4,
        "unhealthy": 0
    }))
}

async fn consciousness_status(State(state): State<AppState>) -> impl IntoResponse {
    *state.request_count.write().await += 1;
    
    Json(json!({
        "emergence_level": 0.85,
        "compression_boundaries": {
            "L2_L3": {
                "ratio": 2.5,
                "stability": 0.92
            },
            "L3_L4": {
                "ratio": 3.2,
                "stability": 0.88
            },
            "L4_L5": {
                "ratio": 4.1,
                "stability": 0.85
            }
        },
        "self_organization": {
            "active": true,
            "patterns_detected": 42,
            "emergence_rate": 0.73
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}