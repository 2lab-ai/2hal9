//! Web interface for HAL9 MVP

use anyhow::Result;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use crate::{Orchestrator, Signal};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
enum WebSocketMessage {
    Signal {
        signal: Signal,
        event: SignalEvent,
    },
    Status {
        message: String,
    },
    Hierarchy {
        signals: Vec<Signal>,
    },
    CodeOutput {
        layer: String,
        content: String,
    },
}

#[derive(Debug, Clone, Serialize)]
enum SignalEvent {
    Created,
    Processing,
    Completed,
}

pub struct WebServer {
    orchestrator: Arc<Orchestrator>,
    broadcast_tx: broadcast::Sender<WebSocketMessage>,
}

impl WebServer {
    pub fn new(orchestrator: Arc<Orchestrator>) -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        Self {
            orchestrator,
            broadcast_tx,
        }
    }

    pub async fn start(self: Arc<Self>) -> Result<()> {
        let app = Router::new()
            .route("/", get(serve_index))
            .route("/ws", get(ws_handler))
            .layer(CorsLayer::permissive())
            .with_state(self);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
        println!("\n🌐 Web UI available at http://localhost:3000");
        
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn serve_index() -> Html<String> {
    // For now, serve a simple HTML page inline
    // In production, this would serve from static/index.html
    Html(get_index_html())
}

fn get_index_html() -> String {
    std::fs::read_to_string("mvp/static/index.html")
        .unwrap_or_else(|_| {
            // Fallback HTML if file not found
            r#"<!DOCTYPE html>
<html>
<head>
    <title>2HAL9 Web UI</title>
</head>
<body>
    <h1>2HAL9 Web Interface</h1>
    <p>Could not load UI. Please ensure mvp/static/index.html exists.</p>
</body>
</html>"#.to_string()
        })
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(server): axum::extract::State<Arc<WebServer>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, server))
}

async fn handle_socket(mut socket: WebSocket, server: Arc<WebServer>) {
    let mut rx = server.broadcast_tx.subscribe();

    loop {
        tokio::select! {
            // Handle outgoing messages
            Ok(msg) = rx.recv() => {
                if let Ok(text) = serde_json::to_string(&msg) {
                    if socket.send(Message::Text(text)).await.is_err() {
                        break;
                    }
                }
            }
            
            // Handle incoming messages
            Some(msg) = socket.recv() => {
                if let Ok(msg) = msg {
                    if let Message::Text(text) = msg {
                        if let Ok(request) = serde_json::from_str::<ClientRequest>(&text) {
                            match request {
                                ClientRequest::StartDemo { scenario } => {
                                    let _ = start_demo(server.clone(), scenario).await;
                                }
                                ClientRequest::GetStatus => {
                                    // Send current status
                                    let _ = server.broadcast_tx.send(WebSocketMessage::Status {
                                        message: "Ready".to_string(),
                                    });
                                }
                            }
                        }
                    }
                } else {
                    break;
                }
            }
            
            else => break,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type")]
enum ClientRequest {
    StartDemo { scenario: String },
    GetStatus,
}

async fn start_demo(server: Arc<WebServer>, scenario: String) -> Result<()> {
    // Send status
    let _ = server.broadcast_tx.send(WebSocketMessage::Status {
        message: format!("Starting demo: {}", scenario),
    });

    // Create initial signal
    let signal = Signal {
        id: Uuid::new_v4(),
        parent_id: None,
        from: "user".to_string(),
        to: "neuron-1".to_string(),
        content: scenario,
        layer: "Input".to_string(),
        timestamp: chrono::Utc::now(),
    };

    // Send signal created event
    let _ = server.broadcast_tx.send(WebSocketMessage::Signal {
        signal: signal.clone(),
        event: SignalEvent::Created,
    });

    // Send to orchestrator
    server.orchestrator.send_signal(signal).await?;

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Send hierarchy
    let signals = server.orchestrator.get_signals().await;
    let _ = server.broadcast_tx.send(WebSocketMessage::Hierarchy { signals });

    // Send completion status
    let _ = server.broadcast_tx.send(WebSocketMessage::Status {
        message: "Demo completed!".to_string(),
    });

    Ok(())
}

