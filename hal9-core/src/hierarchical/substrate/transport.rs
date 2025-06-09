//! Transport abstraction for message passing

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use tokio::sync::mpsc;
use crate::{Result, Error};

/// Message transport abstraction
#[async_trait]
pub trait MessageTransport: Send + Sync + 'static {
    /// Send a message to a destination
    async fn send<M>(&self, destination: &str, message: M) -> Result<()>
    where
        M: Serialize + Send + Sync + 'static;
    
    /// Receive messages for a given endpoint
    async fn receive<M>(&self, endpoint: &str) -> Result<TransportReceiver<M>>
    where
        M: for<'de> Deserialize<'de> + Send + 'static;
    
    /// Subscribe to broadcast messages
    async fn subscribe<M>(&self, topic: &str) -> Result<TransportReceiver<M>>
    where
        M: for<'de> Deserialize<'de> + Send + 'static;
    
    /// Publish to a topic
    async fn publish<M>(&self, topic: &str, message: M) -> Result<()>
    where
        M: Serialize + Send + Sync + 'static;
    
    /// Connect to a remote endpoint
    async fn connect(&self, endpoint: &str) -> Result<()>;
    
    /// Disconnect from a remote endpoint
    async fn disconnect(&self, endpoint: &str) -> Result<()>;
    
    /// Get transport metrics
    fn metrics(&self) -> TransportMetrics;
}

/// Receiver for transport messages
pub struct TransportReceiver<M> {
    inner: Box<dyn ReceiverTrait<M>>,
}

impl<M> TransportReceiver<M> {
    pub async fn recv(&mut self) -> Option<M> {
        self.inner.recv().await
    }
}

#[async_trait]
trait ReceiverTrait<M>: Send {
    async fn recv(&mut self) -> Option<M>;
}

/// Transport performance metrics
#[derive(Debug, Clone, Default)]
pub struct TransportMetrics {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub active_connections: usize,
}

/// Local channel-based transport for single process
pub struct ChannelTransport {
    senders: dashmap::DashMap<String, mpsc::UnboundedSender<Vec<u8>>>,
    metrics: std::sync::Arc<parking_lot::Mutex<TransportMetrics>>,
}

impl ChannelTransport {
    pub fn new() -> Self {
        Self {
            senders: dashmap::DashMap::new(),
            metrics: std::sync::Arc::new(parking_lot::Mutex::new(TransportMetrics::default())),
        }
    }
}

/// TCP-based transport for distributed deployment
pub struct TcpTransport {
    connections: dashmap::DashMap<String, tokio::net::TcpStream>,
    metrics: std::sync::Arc<parking_lot::Mutex<TransportMetrics>>,
}

impl TcpTransport {
    pub fn new() -> Self {
        Self {
            connections: dashmap::DashMap::new(),
            metrics: std::sync::Arc::new(parking_lot::Mutex::new(TransportMetrics::default())),
        }
    }
}

/// gRPC transport for cloud deployment
pub struct GrpcTransport {
    // Would use tonic or similar
    metrics: std::sync::Arc<parking_lot::Mutex<TransportMetrics>>,
}

impl GrpcTransport {
    pub fn new() -> Self {
        Self {
            metrics: std::sync::Arc::new(parking_lot::Mutex::new(TransportMetrics::default())),
        }
    }
}

/// Message envelope for transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportEnvelope {
    pub id: uuid::Uuid,
    pub source: String,
    pub destination: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub payload: Vec<u8>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Connection info for transport endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub endpoint: String,
    pub protocol: String,
    pub status: ConnectionStatus,
    pub latency_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Failed(String),
}