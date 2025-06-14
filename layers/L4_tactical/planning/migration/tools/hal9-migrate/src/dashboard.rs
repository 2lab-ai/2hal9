use anyhow::Result;
use axum::{
    extract::State,
    response::{Html, Json},
    routing::{get, get_service},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tracing::info;

/// Migration dashboard server
pub struct DashboardServer {
    client: Arc<crate::client::MigrationClient>,
    state: Arc<RwLock<DashboardState>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct DashboardState {
    phase: String,
    progress: f32,
    metrics: Metrics,
    events: Vec<Event>,
    feature_flags: Vec<FeatureFlag>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Metrics {
    error_rate: f32,
    latency_p99: f32,
    throughput_rps: f32,
    cpu_usage: f32,
    memory_usage: f32,
    migrated_neurons: u32,
    total_neurons: u32,
}

#[derive(Clone, Serialize, Deserialize)]
struct Event {
    timestamp: String,
    level: String,
    message: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct FeatureFlag {
    name: String,
    enabled: bool,
    percentage: Option<u8>,
}

impl DashboardServer {
    pub fn new(server_url: &str) -> Result<Self> {
        let client = Arc::new(crate::client::MigrationClient::new(server_url)?);
        let state = Arc::new(RwLock::new(DashboardState::default()));
        
        Ok(Self { client, state })
    }
    
    pub async fn run(self, port: u16) -> Result<()> {
        // Start background updater
        let state_clone = Arc::clone(&self.state);
        let client_clone = Arc::clone(&self.client);
        tokio::spawn(async move {
            loop {
                if let Err(e) = update_state(&client_clone, &state_clone).await {
                    tracing::error!("Failed to update state: {}", e);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });
        
        // Build router
        let app = Router::new()
            .route("/", get(index_handler))
            .route("/api/status", get(status_handler))
            .route("/api/metrics", get(metrics_handler))
            .route("/api/events", get(events_handler))
            .route("/api/features", get(features_handler))
            .nest_service("/static", get_service(ServeDir::new("static")))
            .with_state(Arc::clone(&self.state));
        
        let addr = format!("0.0.0.0:{}", port);
        info!("Dashboard server running at http://{}", addr);
        
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
}

async fn update_state(
    _client: &crate::client::MigrationClient,
    state: &Arc<RwLock<DashboardState>>,
) -> Result<()> {
    // Mock update - in real implementation would fetch from API
    let mut state = state.write().await;
    
    // Simulate progress
    if state.progress < 1.0 {
        state.progress += 0.01;
    }
    
    // Update metrics
    state.metrics.error_rate = (rand::random::<f32>() * 0.02).max(0.001);
    state.metrics.latency_p99 = 8.0 + (rand::random::<f32>() * 2.0);
    state.metrics.throughput_rps = 1200.0 + (rand::random::<f32>() * 100.0);
    state.metrics.cpu_usage = 0.4 + (rand::random::<f32>() * 0.2);
    state.metrics.memory_usage = 0.5 + (rand::random::<f32>() * 0.1);
    
    if state.metrics.migrated_neurons < state.metrics.total_neurons {
        state.metrics.migrated_neurons += 1;
    }
    
    Ok(())
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("dashboard.html"))
}

async fn status_handler(
    State(state): State<Arc<RwLock<DashboardState>>>,
) -> Json<serde_json::Value> {
    let state = state.read().await;
    Json(serde_json::json!({
        "phase": state.phase,
        "progress": state.progress,
        "health": state.metrics.error_rate < 0.05,
        "migrated": state.metrics.migrated_neurons,
        "total": state.metrics.total_neurons,
    }))
}

async fn metrics_handler(
    State(state): State<Arc<RwLock<DashboardState>>>,
) -> Json<Metrics> {
    let state = state.read().await;
    Json(state.metrics.clone())
}

async fn events_handler(
    State(state): State<Arc<RwLock<DashboardState>>>,
) -> Json<Vec<Event>> {
    let state = state.read().await;
    Json(state.events.clone())
}

async fn features_handler(
    State(state): State<Arc<RwLock<DashboardState>>>,
) -> Json<Vec<FeatureFlag>> {
    let state = state.read().await;
    Json(state.feature_flags.clone())
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            phase: "canary".to_string(),
            progress: 0.35,
            metrics: Metrics {
                error_rate: 0.001,
                latency_p99: 8.5,
                throughput_rps: 1250.0,
                cpu_usage: 0.45,
                memory_usage: 0.62,
                migrated_neurons: 35,
                total_neurons: 100,
            },
            events: vec![
                Event {
                    timestamp: "14:32:01".to_string(),
                    level: "INFO".to_string(),
                    message: "Migration controller started".to_string(),
                },
                Event {
                    timestamp: "14:32:05".to_string(),
                    level: "INFO".to_string(),
                    message: "Canary phase initialized at 35%".to_string(),
                },
                Event {
                    timestamp: "14:32:30".to_string(),
                    level: "INFO".to_string(),
                    message: "Health check passed".to_string(),
                },
            ],
            feature_flags: vec![
                FeatureFlag {
                    name: "hierarchical_neurons".to_string(),
                    enabled: true,
                    percentage: Some(100),
                },
                FeatureFlag {
                    name: "substrate_abstraction".to_string(),
                    enabled: true,
                    percentage: Some(100),
                },
                FeatureFlag {
                    name: "protocol_negotiation".to_string(),
                    enabled: true,
                    percentage: Some(35),
                },
            ],
        }
    }
}