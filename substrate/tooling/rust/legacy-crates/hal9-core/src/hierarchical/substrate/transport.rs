//! Transport abstraction for message passing

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::{Result, Error};

/// Message transport abstraction
#[async_trait]
pub trait MessageTransport: Send + Sync + 'static {
    /// Send raw bytes to a destination
    async fn send_raw(&self, destination: &str, data: Vec<u8>) -> Result<()>;
    
    /// Receive raw bytes for a given endpoint
    async fn receive_raw(&self, endpoint: &str) -> Result<RawTransportReceiver>;
    
    /// Subscribe to broadcast messages
    async fn subscribe_raw(&self, topic: &str) -> Result<RawTransportReceiver>;
    
    /// Publish raw bytes to a topic
    async fn publish_raw(&self, topic: &str, data: Vec<u8>) -> Result<()>;
    
    /// Connect to a remote endpoint
    async fn connect(&self, endpoint: &str) -> Result<()>;
    
    /// Disconnect from a remote endpoint
    async fn disconnect(&self, endpoint: &str) -> Result<()>;
    
    /// Get transport metrics
    fn metrics(&self) -> TransportMetrics;
}

/// Helper trait for typed transport operations
#[async_trait]
pub trait TypedTransport: MessageTransport {
    /// Send a typed message
    async fn send<M>(&self, destination: &str, message: M) -> Result<()>
    where
        M: Serialize + Send + Sync + 'static,
    {
        let data = bincode::serialize(&message)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        self.send_raw(destination, data).await
    }
    
    /// Receive typed messages
    async fn receive<M>(&self, endpoint: &str) -> Result<TransportReceiver<M>>
    where
        M: for<'de> Deserialize<'de> + Send + 'static,
    {
        let raw_receiver = self.receive_raw(endpoint).await?;
        Ok(TransportReceiver::new(raw_receiver))
    }
    
    /// Subscribe to typed messages
    async fn subscribe<M>(&self, topic: &str) -> Result<TransportReceiver<M>>
    where
        M: for<'de> Deserialize<'de> + Send + 'static,
    {
        let raw_receiver = self.subscribe_raw(topic).await?;
        Ok(TransportReceiver::new(raw_receiver))
    }
    
    /// Publish typed messages
    async fn publish<M>(&self, topic: &str, message: M) -> Result<()>
    where
        M: Serialize + Send + Sync + 'static,
    {
        let data = bincode::serialize(&message)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        self.publish_raw(topic, data).await
    }
}

/// Automatically implement TypedTransport for all MessageTransport types
impl<T: MessageTransport> TypedTransport for T {}

/// Implement MessageTransport for Arc<T> to support shared ownership
#[async_trait]
impl<T: MessageTransport> MessageTransport for Arc<T> {
    async fn send_raw(&self, destination: &str, data: Vec<u8>) -> Result<()> {
        self.as_ref().send_raw(destination, data).await
    }
    
    async fn receive_raw(&self, endpoint: &str) -> Result<RawTransportReceiver> {
        self.as_ref().receive_raw(endpoint).await
    }
    
    async fn subscribe_raw(&self, topic: &str) -> Result<RawTransportReceiver> {
        self.as_ref().subscribe_raw(topic).await
    }
    
    async fn publish_raw(&self, topic: &str, data: Vec<u8>) -> Result<()> {
        self.as_ref().publish_raw(topic, data).await
    }
    
    async fn connect(&self, endpoint: &str) -> Result<()> {
        self.as_ref().connect(endpoint).await
    }
    
    async fn disconnect(&self, endpoint: &str) -> Result<()> {
        self.as_ref().disconnect(endpoint).await
    }
    
    fn metrics(&self) -> TransportMetrics {
        self.as_ref().metrics()
    }
}

/// Default transport type for use throughout the system
/// This provides a concrete type that can be used instead of dyn MessageTransport
pub type DefaultTransport = ChannelTransport;

/// Create a default transport instance
pub fn create_default_transport() -> Arc<DefaultTransport> {
    Arc::new(ChannelTransport::new())
}

/// Raw receiver for transport bytes
pub struct RawTransportReceiver {
    inner: Box<dyn RawReceiverTrait>,
}

impl RawTransportReceiver {
    pub async fn recv(&mut self) -> Option<Vec<u8>> {
        self.inner.recv().await
    }
    
    pub async fn recv_timeout(&mut self, timeout: Duration) -> Result<Option<Vec<u8>>> {
        self.inner.recv_timeout(timeout).await
    }
}

#[async_trait]
trait RawReceiverTrait: Send {
    async fn recv(&mut self) -> Option<Vec<u8>>;
    async fn recv_timeout(&mut self, timeout: Duration) -> Result<Option<Vec<u8>>>;
}

/// Typed receiver for transport messages
pub struct TransportReceiver<M> {
    raw_receiver: RawTransportReceiver,
    _phantom: std::marker::PhantomData<M>,
}

impl<M> TransportReceiver<M> 
where
    M: for<'de> Deserialize<'de> + Send + 'static,
{
    pub fn new(raw_receiver: RawTransportReceiver) -> Self {
        Self {
            raw_receiver,
            _phantom: std::marker::PhantomData,
        }
    }
    
    pub async fn recv(&mut self) -> Option<M> {
        self.raw_receiver.recv().await.and_then(|data| {
            bincode::deserialize(&data).ok()
        })
    }
    
    pub async fn recv_timeout(&mut self, timeout: Duration) -> Result<Option<M>> {
        match self.raw_receiver.recv_timeout(timeout).await? {
            Some(data) => Ok(bincode::deserialize(&data).ok()),
            None => Ok(None),
        }
    }
}

/// Transport performance metrics
#[derive(Debug, Clone)]
pub struct TransportMetrics {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub active_connections: usize,
    pub errors: u64,
    pub latency_ms: f64,
}

impl Default for TransportMetrics {
    fn default() -> Self {
        Self {
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            active_connections: 0,
            errors: 0,
            latency_ms: 0.0,
        }
    }
}

/// Thread-safe metrics tracker
#[derive(Default)]
struct MetricsTracker {
    messages_sent: AtomicU64,
    messages_received: AtomicU64,
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
    errors: AtomicU64,
    latency_sum: AtomicU64,
    latency_count: AtomicU64,
}

impl MetricsTracker {
    fn record_sent(&self, bytes: usize) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
        self.bytes_sent.fetch_add(bytes as u64, Ordering::Relaxed);
    }
    
    fn record_received(&self, bytes: usize) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
        self.bytes_received.fetch_add(bytes as u64, Ordering::Relaxed);
    }
    
    fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }
    
    fn record_latency(&self, ms: u64) {
        self.latency_sum.fetch_add(ms, Ordering::Relaxed);
        self.latency_count.fetch_add(1, Ordering::Relaxed);
    }
    
    fn to_metrics(&self, active_connections: usize) -> TransportMetrics {
        let latency_count = self.latency_count.load(Ordering::Relaxed);
        let avg_latency = if latency_count > 0 {
            self.latency_sum.load(Ordering::Relaxed) as f64 / latency_count as f64
        } else {
            0.0
        };
        
        TransportMetrics {
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            messages_received: self.messages_received.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            active_connections,
            errors: self.errors.load(Ordering::Relaxed),
            latency_ms: avg_latency,
        }
    }
}

/// Channel receiver wrapper
struct ChannelReceiver {
    rx: mpsc::UnboundedReceiver<Vec<u8>>,
}

#[async_trait]
impl RawReceiverTrait for ChannelReceiver {
    async fn recv(&mut self) -> Option<Vec<u8>> {
        self.rx.recv().await
    }
    
    async fn recv_timeout(&mut self, timeout: Duration) -> Result<Option<Vec<u8>>> {
        match tokio::time::timeout(timeout, self.rx.recv()).await {
            Ok(result) => Ok(result),
            Err(_) => Err(Error::Timeout(timeout.as_secs())),
        }
    }
}

/// Local channel-based transport for single process
pub struct ChannelTransport {
    endpoints: Arc<dashmap::DashMap<String, mpsc::UnboundedSender<Vec<u8>>>>,
    topics: Arc<dashmap::DashMap<String, Vec<mpsc::UnboundedSender<Vec<u8>>>>>,
    metrics: Arc<MetricsTracker>,
}

impl Default for ChannelTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelTransport {
    pub fn new() -> Self {
        Self {
            endpoints: Arc::new(dashmap::DashMap::new()),
            topics: Arc::new(dashmap::DashMap::new()),
            metrics: Arc::new(MetricsTracker::default()),
        }
    }
}

#[async_trait]
impl MessageTransport for ChannelTransport {
    async fn send_raw(&self, destination: &str, data: Vec<u8>) -> Result<()> {
        if let Some(sender) = self.endpoints.get(destination) {
            sender.send(data.clone())
                .map_err(|_| Error::Transport("Channel closed".to_string()))?;
            self.metrics.record_sent(data.len());
            Ok(())
        } else {
            Err(Error::Transport(format!("Unknown destination: {}", destination)))
        }
    }
    
    async fn receive_raw(&self, endpoint: &str) -> Result<RawTransportReceiver> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.endpoints.insert(endpoint.to_string(), tx);
        
        Ok(RawTransportReceiver {
            inner: Box::new(ChannelReceiver { rx }),
        })
    }
    
    async fn subscribe_raw(&self, topic: &str) -> Result<RawTransportReceiver> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        self.topics.entry(topic.to_string())
            .or_default()
            .push(tx);
        
        Ok(RawTransportReceiver {
            inner: Box::new(ChannelReceiver { rx }),
        })
    }
    
    async fn publish_raw(&self, topic: &str, data: Vec<u8>) -> Result<()> {
        if let Some(mut subscribers) = self.topics.get_mut(topic) {
            // Remove closed channels
            subscribers.retain(|tx| !tx.is_closed());
            
            // Send to all subscribers
            let subscriber_count = subscribers.len();
            for tx in subscribers.iter() {
                let _ = tx.send(data.clone());
            }
            
            // Record each delivery as a separate message
            for _ in 0..subscriber_count {
                self.metrics.record_sent(data.len());
            }
        }
        
        Ok(())
    }
    
    async fn connect(&self, _endpoint: &str) -> Result<()> {
        // No-op for channel transport
        Ok(())
    }
    
    async fn disconnect(&self, endpoint: &str) -> Result<()> {
        self.endpoints.remove(endpoint);
        Ok(())
    }
    
    fn metrics(&self) -> TransportMetrics {
        self.metrics.to_metrics(self.endpoints.len())
    }
}

/// TCP connection wrapper
struct TcpConnection {
    stream: Arc<tokio::sync::Mutex<tokio::net::TcpStream>>,
    rx: mpsc::UnboundedReceiver<Vec<u8>>,
}

/// TCP receiver wrapper
struct TcpReceiver {
    rx: mpsc::UnboundedReceiver<Vec<u8>>,
}

#[async_trait]
impl RawReceiverTrait for TcpReceiver {
    async fn recv(&mut self) -> Option<Vec<u8>> {
        self.rx.recv().await
    }
    
    async fn recv_timeout(&mut self, timeout: Duration) -> Result<Option<Vec<u8>>> {
        match tokio::time::timeout(timeout, self.rx.recv()).await {
            Ok(result) => Ok(result),
            Err(_) => Err(Error::Timeout(timeout.as_secs())),
        }
    }
}

/// TCP-based transport for distributed deployment
pub struct TcpTransport {
    connections: Arc<dashmap::DashMap<String, TcpConnection>>,
    listeners: Arc<dashmap::DashMap<String, mpsc::UnboundedSender<Vec<u8>>>>,
    metrics: Arc<MetricsTracker>,
}

impl Default for TcpTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl TcpTransport {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(dashmap::DashMap::new()),
            listeners: Arc::new(dashmap::DashMap::new()),
            metrics: Arc::new(MetricsTracker::default()),
        }
    }
    
    /// Start listening on a port
    pub async fn listen(&self, addr: &str) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(addr).await
            .map_err(|e| Error::Transport(format!("Failed to bind: {}", e)))?;
        
        let connections = Arc::clone(&self.connections);
        let listeners = Arc::clone(&self.listeners);
        let metrics = Arc::clone(&self.metrics);
        
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let connections = Arc::clone(&connections);
                        let listeners = Arc::clone(&listeners);
                        let metrics = Arc::clone(&metrics);
                        
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(
                                stream,
                                addr.to_string(),
                                connections,
                                listeners,
                                metrics,
                            ).await {
                                tracing::error!("Connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Accept error: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    async fn handle_connection(
        mut stream: tokio::net::TcpStream,
        addr: String,
        connections: Arc<dashmap::DashMap<String, TcpConnection>>,
        listeners: Arc<dashmap::DashMap<String, mpsc::UnboundedSender<Vec<u8>>>>,
        metrics: Arc<MetricsTracker>,
    ) -> Result<()> {
        let (rx_half, tx_half) = stream.split();
        let mut reader = tokio::io::BufReader::new(rx_half);
        
        loop {
            // Read message length
            let mut len_buf = [0u8; 4];
            if reader.read_exact(&mut len_buf).await.is_err() {
                break;
            }
            
            let len = u32::from_be_bytes(len_buf) as usize;
            if len > 10_000_000 { // 10MB max message size
                metrics.record_error();
                break;
            }
            
            // Read message data
            let mut data = vec![0u8; len];
            if reader.read_exact(&mut data).await.is_err() {
                break;
            }
            
            metrics.record_received(data.len());
            
            // Deserialize envelope
            if let Ok(envelope) = bincode::deserialize::<TransportEnvelope>(&data) {
                // Route to appropriate listener
                if let Some(tx) = listeners.get(&envelope.destination) {
                    let _ = tx.send(envelope.payload);
                }
            }
        }
        
        connections.remove(&addr);
        Ok(())
    }
}

#[async_trait]
impl MessageTransport for TcpTransport {
    async fn send_raw(&self, destination: &str, data: Vec<u8>) -> Result<()> {
        let envelope = TransportEnvelope {
            id: uuid::Uuid::new_v4(),
            source: "local".to_string(),
            destination: destination.to_string(),
            timestamp: chrono::Utc::now(),
            payload: data,
            metadata: std::collections::HashMap::new(),
        };
        
        let envelope_data = bincode::serialize(&envelope)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        
        if let Some(conn) = self.connections.get(destination) {
            let mut stream = conn.stream.lock().await;
            
            // Send message length
            stream.write_all(&(envelope_data.len() as u32).to_be_bytes()).await
                .map_err(|e| Error::Transport(e.to_string()))?;
            
            // Send message data
            stream.write_all(&envelope_data).await
                .map_err(|e| Error::Transport(e.to_string()))?;
            
            self.metrics.record_sent(envelope_data.len());
            Ok(())
        } else {
            Err(Error::Transport(format!("Not connected to {}", destination)))
        }
    }
    
    async fn receive_raw(&self, endpoint: &str) -> Result<RawTransportReceiver> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.listeners.insert(endpoint.to_string(), tx);
        
        Ok(RawTransportReceiver {
            inner: Box::new(TcpReceiver { rx }),
        })
    }
    
    async fn subscribe_raw(&self, _topic: &str) -> Result<RawTransportReceiver> {
        // TCP transport doesn't support pub/sub natively
        // Would need to implement a protocol on top
        Err(Error::Transport("TCP transport doesn't support pub/sub".to_string()))
    }
    
    async fn publish_raw(&self, _topic: &str, _data: Vec<u8>) -> Result<()> {
        // TCP transport doesn't support pub/sub natively
        Err(Error::Transport("TCP transport doesn't support pub/sub".to_string()))
    }
    
    async fn connect(&self, endpoint: &str) -> Result<()> {
        let stream = tokio::net::TcpStream::connect(endpoint).await
            .map_err(|e| Error::Transport(format!("Failed to connect: {}", e)))?;
        
        let (tx, rx) = mpsc::unbounded_channel();
        
        let conn = TcpConnection {
            stream: Arc::new(tokio::sync::Mutex::new(stream)),
            rx,
        };
        
        self.connections.insert(endpoint.to_string(), conn);
        Ok(())
    }
    
    async fn disconnect(&self, endpoint: &str) -> Result<()> {
        self.connections.remove(endpoint);
        Ok(())
    }
    
    fn metrics(&self) -> TransportMetrics {
        self.metrics.to_metrics(self.connections.len())
    }
}

/// gRPC transport for cloud deployment
pub struct GrpcTransport {
    // Would use tonic or similar
    metrics: std::sync::Arc<parking_lot::Mutex<TransportMetrics>>,
}

impl Default for GrpcTransport {
    fn default() -> Self {
        Self::new()
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_channel_transport_send_receive() {
        let transport = ChannelTransport::new();
        
        // Set up receiver
        let mut receiver = transport.receive::<String>("test-endpoint").await.unwrap();
        
        // Send message
        transport.send("test-endpoint", "Hello, World!".to_string()).await.unwrap();
        
        // Receive message
        let msg = receiver.recv().await.unwrap();
        assert_eq!(msg, "Hello, World!");
        
        // Check metrics
        let metrics = transport.metrics();
        assert_eq!(metrics.messages_sent, 1);
    }
    
    #[tokio::test]
    async fn test_channel_transport_pubsub() {
        let transport = ChannelTransport::new();
        
        // Set up subscribers
        let mut sub1 = transport.subscribe::<i32>("test-topic").await.unwrap();
        let mut sub2 = transport.subscribe::<i32>("test-topic").await.unwrap();
        
        // Publish message
        transport.publish("test-topic", 42).await.unwrap();
        
        // Both subscribers should receive
        assert_eq!(sub1.recv().await.unwrap(), 42);
        assert_eq!(sub2.recv().await.unwrap(), 42);
    }
    
    #[tokio::test]
    async fn test_tcp_transport_connect() {
        let transport = TcpTransport::new();
        
        // Start listener
        transport.listen("127.0.0.1:0").await.unwrap();
        
        // Would need actual TCP connection test here
        // For now, just verify construction
        let metrics = transport.metrics();
        assert_eq!(metrics.active_connections, 0);
    }
}