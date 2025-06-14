use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::{
    games::GameState,
    collective::CollectiveDecision,
    sota::SOTADecision,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamMessage {
    GameStarted {
        game_id: Uuid,
        collective_players: Vec<String>,
        sota_players: Vec<String>,
    },
    GameStateUpdate {
        game_id: Uuid,
        state: GameState,
    },
    CollectiveUpdate {
        game_id: Uuid,
        player_id: String,
        decision: CollectiveDecision,
        visualization: CollectiveVisualization,
    },
    SOTAUpdate {
        game_id: Uuid,
        player_id: String,
        decision: SOTADecision,
        reasoning_viz: ReasoningVisualization,
    },
    GameEnded {
        game_id: Uuid,
        result: crate::games::GameResult,
    },
    AnalyticsUpdate {
        game_id: Uuid,
        metrics: RealTimeMetrics,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveVisualization {
    pub active_nodes: u32,
    pub consensus_graph: serde_json::Value,
    pub decision_distribution: Vec<(String, u32)>,
    pub emergence_indicator: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningVisualization {
    pub thought_chain: Vec<String>,
    pub confidence_graph: Vec<f32>,
    pub strategy_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeMetrics {
    pub round: u32,
    pub collective_performance: f32,
    pub sota_performance: f32,
    pub emergence_events: u32,
    pub decision_time_ms: u64,
}

pub struct StreamingEngine {
    broadcast_sender: broadcast::Sender<StreamMessage>,
    active_connections: Arc<RwLock<u32>>,
}

impl StreamingEngine {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            broadcast_sender: tx,
            active_connections: Arc::new(RwLock::new(0)),
        }
    }
    
    pub async fn handle_connection(&self, ws: WebSocket) {
        let mut rx = self.broadcast_sender.subscribe();
        let mut connections = self.active_connections.write().await;
        *connections += 1;
        drop(connections);
        
        let (mut sender, mut receiver) = ws.split();
        
        // Spawn task to forward broadcast messages to websocket
        let send_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                let json = serde_json::to_string(&msg).unwrap();
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        });
        
        // Handle incoming messages (if any)
        let recv_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                match msg {
                    Message::Text(text) => {
                        tracing::debug!("Received: {}", text);
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        });
        
        // Wait for either task to finish
        tokio::select! {
            _ = send_task => {},
            _ = recv_task => {},
        }
        
        let mut connections = self.active_connections.write().await;
        *connections -= 1;
    }
    
    pub async fn game_started(
        &self,
        game_id: Uuid,
        collective_players: Vec<String>,
        sota_players: Vec<String>,
    ) {
        let _ = self.broadcast_sender.send(StreamMessage::GameStarted {
            game_id,
            collective_players,
            sota_players,
        });
    }
    
    pub async fn update_game_state(&self, game_id: Uuid, state: GameState) {
        let _ = self.broadcast_sender.send(StreamMessage::GameStateUpdate {
            game_id,
            state,
        });
    }
    
    pub async fn update_collective_state(
        &self,
        game_id: Uuid,
        player_id: String,
        decision: CollectiveDecision,
    ) {
        let visualization = CollectiveVisualization {
            active_nodes: decision.individual_decisions.len() as u32,
            consensus_graph: serde_json::json!({
                "nodes": decision.individual_decisions.len(),
                "method": decision.consensus_method,
            }),
            decision_distribution: vec![], // TODO: Calculate from decisions
            emergence_indicator: if decision.emergence_detected { 1.0 } else { 0.0 },
        };
        
        let _ = self.broadcast_sender.send(StreamMessage::CollectiveUpdate {
            game_id,
            player_id,
            decision,
            visualization,
        });
    }
    
    pub async fn update_sota_state(
        &self,
        game_id: Uuid,
        player_id: String,
        decision: SOTADecision,
    ) {
        let reasoning_viz = ReasoningVisualization {
            thought_chain: decision.reasoning_chain.clone(),
            confidence_graph: vec![decision.confidence],
            strategy_state: decision.strategy.clone(),
        };
        
        let _ = self.broadcast_sender.send(StreamMessage::SOTAUpdate {
            game_id,
            player_id,
            decision,
            reasoning_viz,
        });
    }
    
    pub async fn game_ended(&self, game_id: Uuid, result: crate::games::GameResult) {
        let _ = self.broadcast_sender.send(StreamMessage::GameEnded {
            game_id,
            result,
        });
    }
    
    pub async fn update_analytics(&self, game_id: Uuid, metrics: RealTimeMetrics) {
        let _ = self.broadcast_sender.send(StreamMessage::AnalyticsUpdate {
            game_id,
            metrics,
        });
    }
}