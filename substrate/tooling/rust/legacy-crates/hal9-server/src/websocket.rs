//! WebSocket handling for real-time communication

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::server::HAL9Server;

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Client subscribes to a channel
    Subscribe {
        channel: String,
        filter: Option<serde_json::Value>,
    },
    /// Server confirms subscription
    Subscribed {
        channel: String,
        subscription_id: String,
    },
    /// Client unsubscribes from a channel
    Unsubscribe {
        subscription_id: String,
    },
    /// Server confirms unsubscription
    Unsubscribed {
        subscription_id: String,
    },
    /// Echo request for testing
    Echo {
        payload: String,
    },
    /// Ping for keepalive
    Ping {
        id: Option<u64>,
        timestamp: Option<i64>,
    },
    /// Pong response
    Pong {
        id: Option<u64>,
        timestamp: Option<i64>,
    },
    /// Signal event
    Signal {
        signal_id: String,
        layer: String,
        content: String,
        timestamp: i64,
    },
    /// Neuron state change
    NeuronStateChange {
        neuron_id: String,
        old_state: String,
        new_state: String,
    },
    /// Error message
    Error {
        message: String,
        code: Option<String>,
    },
}

/// WebSocket connection state
struct WsConnection {
    id: Uuid,
    subscriptions: RwLock<Vec<Subscription>>,
}

/// Subscription information
#[derive(Clone)]
struct Subscription {
    id: String,
    channel: String,
    filter: Option<serde_json::Value>,
}

#[allow(dead_code)]
impl Subscription {
    fn matches(&self, channel: &str) -> bool {
        self.channel == channel
    }
}

/// WebSocket handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(server): State<Arc<HAL9Server>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, server))
}

async fn handle_socket(socket: WebSocket, server: Arc<HAL9Server>) {
    let connection_id = Uuid::new_v4();
    info!("New WebSocket connection: {}", connection_id);

    let (mut sender, mut receiver) = socket.split();
    let connection = Arc::new(WsConnection {
        id: connection_id,
        subscriptions: RwLock::new(Vec::new()),
    });

    // Create broadcast channel for this connection
    let (tx, mut rx) = broadcast::channel::<WsMessage>(100);

    // Spawn task to handle outgoing messages
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(text) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(text)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages
    let tx_clone = tx.clone();
    let connection_clone = connection.clone();
    let server_clone = server.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Err(e) = handle_message(msg, &tx_clone, &connection_clone, &server_clone).await {
                error!("Error handling message: {}", e);
                let _ = tx_clone.send(WsMessage::Error {
                    message: e.to_string(),
                    code: None,
                });
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    // Clean up
    let subscriptions = connection.subscriptions.read().await;
    info!(
        "WebSocket connection {} closed with {} subscriptions",
        connection_id,
        subscriptions.len()
    );
}

async fn handle_message(
    msg: Message,
    tx: &broadcast::Sender<WsMessage>,
    connection: &Arc<WsConnection>,
    _server: &Arc<HAL9Server>,
) -> anyhow::Result<()> {
    match msg {
        Message::Text(text) => {
            let ws_msg: WsMessage = serde_json::from_str(&text)?;
            match ws_msg {
                WsMessage::Subscribe { channel, filter } => {
                    handle_subscribe(tx, connection, channel, filter).await?;
                }
                WsMessage::Unsubscribe { subscription_id } => {
                    handle_unsubscribe(tx, connection, subscription_id).await?;
                }
                WsMessage::Echo { payload } => {
                    tx.send(WsMessage::Echo { payload })?;
                }
                WsMessage::Ping { id, timestamp } => {
                    tx.send(WsMessage::Pong { id, timestamp })?;
                }
                _ => {
                    warn!("Unhandled message type: {:?}", ws_msg);
                }
            }
        }
        Message::Binary(_) => {
            return Err(anyhow::anyhow!("Binary messages not supported"));
        }
        Message::Ping(data) => {
            // Axum handles ping/pong automatically
            debug!("Received ping: {:?}", data);
        }
        Message::Pong(data) => {
            debug!("Received pong: {:?}", data);
        }
        Message::Close(_) => {
            debug!("Received close message");
        }
    }
    Ok(())
}

async fn handle_subscribe(
    tx: &broadcast::Sender<WsMessage>,
    connection: &Arc<WsConnection>,
    channel: String,
    filter: Option<serde_json::Value>,
) -> anyhow::Result<()> {
    let subscription_id = Uuid::new_v4().to_string();
    let subscription = Subscription {
        id: subscription_id.clone(),
        channel: channel.clone(),
        filter,
    };

    // Add subscription
    let mut subscriptions = connection.subscriptions.write().await;
    subscriptions.push(subscription);

    // Send confirmation
    tx.send(WsMessage::Subscribed {
        channel,
        subscription_id,
    })?;

    info!("Connection {} subscribed to channel", connection.id);
    Ok(())
}

async fn handle_unsubscribe(
    tx: &broadcast::Sender<WsMessage>,
    connection: &Arc<WsConnection>,
    subscription_id: String,
) -> anyhow::Result<()> {
    let mut subscriptions = connection.subscriptions.write().await;
    subscriptions.retain(|s| s.id != subscription_id);

    // Send confirmation
    tx.send(WsMessage::Unsubscribed {
        subscription_id: subscription_id.clone(),
    })?;

    info!("Connection {} unsubscribed from {}", connection.id, subscription_id);
    Ok(())
}

/// Broadcast a message to all WebSocket connections
pub async fn broadcast_message(_server: &HAL9Server, message: WsMessage) {
    // This would need to be implemented with a global connection manager
    // For now, just log it
    debug!("Broadcasting message: {:?}", message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = WsMessage::Echo {
            payload: "test".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"echo\""));
        
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        matches!(parsed, WsMessage::Echo { payload } if payload == "test");
    }

    #[test]
    fn test_signal_message() {
        let msg = WsMessage::Signal {
            signal_id: "sig-123".to_string(),
            layer: "L2".to_string(),
            content: "Test signal".to_string(),
            timestamp: 1234567890,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"signal\""));
        assert!(json.contains("\"signal_id\":\"sig-123\""));
    }
}