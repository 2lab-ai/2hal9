use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::{Mutex, RwLock},
    time::interval,
};
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

#[derive(Clone)]
struct DashboardState {
    metrics: Arc<RwLock<SystemMetrics>>,
    connections: Arc<Mutex<HashMap<String, WebSocket>>>,
    start_time: Instant,
}

#[derive(Clone, Serialize, Default)]
struct SystemMetrics {
    system: SystemInfo,
    neurons: NeuronStatus,
    consciousness: ConsciousnessMetrics,
    games: GameStats,
    performance: PerformanceMetrics,
}

#[derive(Clone, Serialize, Default)]
struct SystemInfo {
    cpu: f32,
    memory: f32,
    latency: u32,
    uptime: u64,
}

#[derive(Clone, Serialize, Default)]
struct NeuronStatus {
    active: usize,
    layers: usize,
    speed: f32,
    grid: Vec<bool>,
}

#[derive(Clone, Serialize, Default)]
struct ConsciousnessMetrics {
    phi: f64,
    ratio: String,
}

#[derive(Clone, Serialize, Default)]
struct GameStats {
    active: usize,
    #[serde(rename = "winRate")]
    win_rate: u8,
}

#[derive(Clone, Serialize, Default)]
struct PerformanceMetrics {
    ops: u64,
    #[serde(rename = "avgResponse")]
    avg_response: f32,
    fps: u32,
}

#[derive(Deserialize)]
struct DashboardCommand {
    action: String,
    #[serde(default)]
    count: Option<usize>,
}

impl DashboardState {
    fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            connections: Arc::new(Mutex::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    async fn update_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        
        // Update system info
        metrics.system.cpu = get_cpu_usage();
        metrics.system.memory = get_memory_usage();
        metrics.system.latency = measure_latency().await;
        metrics.system.uptime = self.start_time.elapsed().as_secs();
        
        // Update neuron status
        metrics.neurons.active = rand::random::<usize>() % 10000;
        metrics.neurons.layers = (rand::random::<usize>() % 6) + 2;
        metrics.neurons.speed = rand::random::<f32>() * 100.0;
        metrics.neurons.grid = (0..100).map(|_| rand::random::<bool>()).collect();
        
        // Update consciousness metrics
        metrics.consciousness.phi = rand::random::<f64>() * 2.0;
        metrics.consciousness.ratio = format!("{:.1}:1", rand::random::<f32>() * 10.0);
        
        // Update game stats
        metrics.games.active = rand::random::<usize>() % 10;
        metrics.games.win_rate = rand::random::<u8>() % 100;
        
        // Update performance
        metrics.performance.ops = rand::random::<u64>() % 100_000_000;
        metrics.performance.avg_response = rand::random::<f32>() * 100.0;
        metrics.performance.fps = rand::random::<u32>() % 120;
    }

    async fn broadcast_metrics(&self) {
        let metrics = self.metrics.read().await.clone();
        let message = serde_json::to_string(&metrics).unwrap();
        
        let mut connections = self.connections.lock().await;
        connections.retain(|id, ws| {
            // In real implementation, send through WebSocket
            // For now, just log
            info!("Broadcasting to connection {}: {}", id, message);
            true
        });
    }
}

async fn serve_dashboard() -> Html<&'static str> {
    Html(include_str!("integrated-dashboard.html"))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<DashboardState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: DashboardState) {
    let id = uuid::Uuid::new_v4().to_string();
    info!("New dashboard connection: {}", id);
    
    // Store connection
    state.connections.lock().await.insert(id.clone(), socket);
    
    // Handle incoming messages
    // In real implementation, process commands from dashboard
}

async fn get_metrics(State(state): State<DashboardState>) -> Json<SystemMetrics> {
    Json(state.metrics.read().await.clone())
}

async fn handle_command(
    State(state): State<DashboardState>,
    Json(command): Json<DashboardCommand>,
) -> Json<serde_json::Value> {
    match command.action.as_str() {
        "reorganize" => {
            info!("Reorganizing neurons...");
            // Trigger neuron reorganization
            Json(serde_json::json!({ "status": "success" }))
        }
        "add_neurons" => {
            let count = command.count.unwrap_or(10);
            info!("Adding {} neurons...", count);
            // Add neurons to the system
            Json(serde_json::json!({ "status": "success", "added": count }))
        }
        _ => Json(serde_json::json!({ "status": "error", "message": "Unknown command" })),
    }
}

// Mock functions - replace with real implementations
fn get_cpu_usage() -> f32 {
    rand::random::<f32>() * 100.0
}

fn get_memory_usage() -> f32 {
    rand::random::<f32>() * 100.0
}

async fn measure_latency() -> u32 {
    rand::random::<u32>() % 100
}

pub async fn run_dashboard_server() {
    let state = DashboardState::new();

    // Spawn metrics update task
    let metrics_state = state.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            metrics_state.update_metrics().await;
            metrics_state.broadcast_metrics().await;
        }
    });

    // Build router
    let app = Router::new()
        .route("/", get(serve_dashboard))
        .route("/dashboard", get(serve_dashboard))
        .route("/ws/dashboard", get(websocket_handler))
        .route("/api/dashboard/metrics", get(get_metrics))
        .route("/api/dashboard/command", post(handle_command))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Dashboard server running on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dashboard_state() {
        let state = DashboardState::new();
        state.update_metrics().await;
        
        let metrics = state.metrics.read().await;
        assert!(metrics.system.cpu >= 0.0 && metrics.system.cpu <= 100.0);
        assert!(metrics.neurons.active > 0);
    }
}