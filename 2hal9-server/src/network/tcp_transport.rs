//! TCP transport layer for neuron communication

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, RwLock};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use dashmap::DashMap;

use twohal9_core::{Result, Error, NeuronSignal};
use crate::network::protocol::{NetworkMessage, MessageCodec};
use crate::metrics::Metrics;

/// Configuration for TCP transport
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Address to bind to
    pub bind_address: SocketAddr,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Read/write timeout
    pub io_timeout: Duration,
    /// Keep-alive interval
    pub keep_alive_interval: Duration,
    /// Buffer size for network operations
    pub buffer_size: usize,
    /// Enable TLS encryption
    pub tls_enabled: bool,
    /// TLS certificate path
    pub tls_cert_path: Option<String>,
    /// TLS key path
    pub tls_key_path: Option<String>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:9000".parse().unwrap(),
            max_connections: 1000,
            connection_timeout: Duration::from_secs(10),
            io_timeout: Duration::from_secs(30),
            keep_alive_interval: Duration::from_secs(60),
            buffer_size: 65536,
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

/// TCP transport for distributed neuron communication
pub struct TcpTransport {
    config: TransportConfig,
    server_id: String,
    listener: Option<TcpListener>,
    connections: Arc<DashMap<String, Arc<Connection>>>,
    signal_tx: mpsc::Sender<(String, NeuronSignal)>,
    signal_rx: RwLock<Option<mpsc::Receiver<(String, NeuronSignal)>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    metrics: Option<Arc<Metrics>>,
}

/// Active connection to a remote server
struct Connection {
    peer_id: String,
    stream: Arc<RwLock<TcpStream>>,
    last_activity: RwLock<std::time::Instant>,
}

impl TcpTransport {
    /// Create a new TCP transport
    pub fn new(config: TransportConfig, server_id: String) -> Self {
        let (signal_tx, signal_rx) = mpsc::channel(10000);
        
        Self {
            config,
            server_id,
            listener: None,
            connections: Arc::new(DashMap::new()),
            signal_tx,
            signal_rx: RwLock::new(Some(signal_rx)),
            shutdown_tx: None,
            metrics: None,
        }
    }
    
    /// Set metrics collector
    pub fn set_metrics(&mut self, metrics: Arc<Metrics>) {
        self.metrics = Some(metrics);
    }
    
    /// Start the transport layer
    pub async fn start(&mut self) -> Result<()> {
        // Bind to the configured address
        let listener = TcpListener::bind(&self.config.bind_address).await
            .map_err(|e| Error::Network(format!("Failed to bind to {}: {}", self.config.bind_address, e)))?;
            
        info!("TCP transport listening on {}", self.config.bind_address);
        self.listener = Some(listener);
        
        // Start connection keeper
        self.start_connection_keeper().await;
        
        // Start accept loop
        self.start_accept_loop().await?;
        
        Ok(())
    }
    
    /// Start accepting incoming connections
    async fn start_accept_loop(&mut self) -> Result<()> {
        let listener = self.listener.take()
            .ok_or_else(|| Error::InvalidState("Listener not initialized".to_string()))?;
            
        let connections = self.connections.clone();
        let signal_tx = self.signal_tx.clone();
        let config = self.config.clone();
        let metrics = self.metrics.clone();
        let server_id = self.server_id.clone();
        
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((mut stream, peer_addr)) => {
                                debug!("Accepted connection from {}", peer_addr);
                                
                                // Check connection limit
                                if connections.len() >= config.max_connections {
                                    warn!("Connection limit reached, rejecting {}", peer_addr);
                                    let _ = stream.shutdown().await;
                                    continue;
                                }
                                
                                // Handle connection
                                let connections = connections.clone();
                                let signal_tx = signal_tx.clone();
                                let config = config.clone();
                                let metrics = metrics.clone();
                                let server_id = server_id.clone();
                                
                                tokio::spawn(async move {
                                    if let Err(e) = Self::handle_connection(
                                        stream,
                                        peer_addr,
                                        connections,
                                        signal_tx,
                                        config,
                                        metrics,
                                        server_id
                                    ).await {
                                        error!("Connection handler error: {}", e);
                                    }
                                });
                            }
                            Err(e) => {
                                error!("Accept error: {}", e);
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Shutting down accept loop");
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Handle an incoming connection
    async fn handle_connection(
        mut stream: TcpStream,
        _peer_addr: SocketAddr,
        connections: Arc<DashMap<String, Arc<Connection>>>,
        signal_tx: mpsc::Sender<(String, NeuronSignal)>,
        config: TransportConfig,
        metrics: Option<Arc<Metrics>>,
        server_id: String,
    ) -> Result<()> {
        // Set TCP options
        stream.set_nodelay(true)
            .map_err(|e| Error::Network(format!("Failed to set TCP nodelay: {}", e)))?;
        
        // Perform handshake
        let peer_id = Self::perform_handshake(&mut stream, &config, &server_id).await?;
        info!("Handshake completed with peer {}", peer_id);
        
        // Create connection
        let connection = Arc::new(Connection {
            peer_id: peer_id.clone(),
            stream: Arc::new(RwLock::new(stream)),
            last_activity: RwLock::new(std::time::Instant::now()),
        });
        
        // Store connection
        connections.insert(peer_id.clone(), connection.clone());
        
        // Start read loop
        Self::connection_read_loop(
            connection,
            connections,
            signal_tx,
            config,
            metrics
        ).await
    }
    
    /// Perform handshake with remote peer
    async fn perform_handshake(stream: &mut TcpStream, config: &TransportConfig, server_id: &str) -> Result<String> {
        // Send hello message
        let hello = NetworkMessage::Hello {
            version: "1.0".to_string(),
            server_id: server_id.to_string(),
            capabilities: vec!["signal".to_string(), "metrics".to_string()],
        };
        
        let encoded = MessageCodec::encode(&hello)?;
        
        timeout(config.connection_timeout, async {
            stream.write_all(&encoded).await
        }).await
            .map_err(|_| Error::Network("Handshake timeout".to_string()))?
            .map_err(|e| Error::Network(format!("Handshake write error: {}", e)))?;
            
        // Read response
        let mut buffer = vec![0u8; config.buffer_size];
        let n = timeout(config.connection_timeout, async {
            stream.read(&mut buffer).await
        }).await
            .map_err(|_| Error::Network("Handshake response timeout".to_string()))?
            .map_err(|e| Error::Network(format!("Handshake read error: {}", e)))?;
            
        if n == 0 {
            return Err(Error::Network("Connection closed during handshake".to_string()));
        }
        
        // Decode response
        let response = MessageCodec::decode(&buffer[..n])?;
        
        match response {
            NetworkMessage::Hello { server_id, .. } => Ok(server_id),
            _ => Err(Error::Network("Invalid handshake response".to_string())),
        }
    }
    
    /// Connection read loop
    async fn connection_read_loop(
        connection: Arc<Connection>,
        connections: Arc<DashMap<String, Arc<Connection>>>,
        signal_tx: mpsc::Sender<(String, NeuronSignal)>,
        config: TransportConfig,
        metrics: Option<Arc<Metrics>>,
    ) -> Result<()> {
        let mut buffer = vec![0u8; config.buffer_size];
        let peer_id = connection.peer_id.clone();
        
        loop {
            let stream = connection.stream.read().await;
            
            // Read with timeout
            let read_result = timeout(config.io_timeout, async {
                stream.try_read(&mut buffer)
            }).await;
            
            drop(stream); // Release read lock
            
            match read_result {
                Ok(Ok(0)) => {
                    info!("Connection closed by peer {}", peer_id);
                    break;
                }
                Ok(Ok(n)) => {
                    // Update last activity
                    *connection.last_activity.write().await = std::time::Instant::now();
                    
                    // Decode message
                    match MessageCodec::decode(&buffer[..n]) {
                        Ok(msg) => {
                            if let Err(e) = Self::handle_message(
                                msg,
                                &peer_id,
                                &signal_tx,
                                &metrics
                            ).await {
                                error!("Failed to handle message: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to decode message from {}: {}", peer_id, e);
                        }
                    }
                }
                Ok(Err(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available, sleep briefly
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Ok(Err(e)) => {
                    error!("Read error from {}: {}", peer_id, e);
                    break;
                }
                Err(_) => {
                    warn!("Read timeout from {}", peer_id);
                    // Don't break on timeout, connection might still be alive
                }
            }
        }
        
        // Remove connection
        connections.remove(&peer_id);
        info!("Removed connection to {}", peer_id);
        
        Ok(())
    }
    
    /// Handle incoming network message
    async fn handle_message(
        msg: NetworkMessage,
        peer_id: &str,
        signal_tx: &mpsc::Sender<(String, NeuronSignal)>,
        metrics: &Option<Arc<Metrics>>,
    ) -> Result<()> {
        match msg {
            NetworkMessage::Signal(signal) => {
                debug!("Received signal from {}: {:?}", peer_id, signal.signal_id);
                
                if let Some(metrics) = metrics {
                    metrics.record_signal_sent(); // Track as received from network
                }
                
                signal_tx.send((peer_id.to_string(), signal)).await
                    .map_err(|_| Error::Communication("Failed to queue signal".to_string()))?;
            }
            NetworkMessage::Ping => {
                debug!("Received ping from {}", peer_id);
                // TODO: Send pong
            }
            NetworkMessage::Metrics { .. } => {
                debug!("Received metrics from {}", peer_id);
                // TODO: Process metrics
            }
            _ => {
                warn!("Unexpected message type from {}", peer_id);
            }
        }
        
        Ok(())
    }
    
    /// Connect to a remote server
    pub async fn connect(&self, address: SocketAddr, server_id: &str) -> Result<()> {
        // Check if already connected
        if self.connections.contains_key(server_id) {
            debug!("Already connected to {}", server_id);
            return Ok(());
        }
        
        info!("Connecting to {} at {}", server_id, address);
        
        // Connect with timeout
        let mut stream = timeout(self.config.connection_timeout, async {
            TcpStream::connect(address).await
        }).await
            .map_err(|_| Error::Network(format!("Connection timeout to {}", address)))?
            .map_err(|e| Error::Network(format!("Failed to connect to {}: {}", address, e)))?;
            
        // Set TCP options
        stream.set_nodelay(true)
            .map_err(|e| Error::Network(format!("Failed to set TCP nodelay: {}", e)))?;
        
        // Perform handshake before creating connection
        let peer_id = Self::perform_handshake(&mut stream, &self.config, &self.server_id).await?;
        
        // Create connection with the stream
        let connection = Arc::new(Connection {
            peer_id: peer_id.clone(),
            stream: Arc::new(RwLock::new(stream)),
            last_activity: RwLock::new(std::time::Instant::now()),
        });
        
        // Store connection
        self.connections.insert(peer_id, connection.clone());
        
        // Start read loop
        let connections = self.connections.clone();
        let signal_tx = self.signal_tx.clone();
        let config = self.config.clone();
        let metrics = self.metrics.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::connection_read_loop(
                connection,
                connections,
                signal_tx,
                config,
                metrics
            ).await {
                error!("Connection read loop error: {}", e);
            }
        });
        
        Ok(())
    }
    
    /// Send a signal to a remote server
    pub async fn send_signal(&self, server_id: &str, signal: NeuronSignal) -> Result<()> {
        let connection = self.connections.get(server_id)
            .ok_or_else(|| Error::Network(format!("Not connected to {}", server_id)))?;
            
        let msg = NetworkMessage::Signal(signal);
        let encoded = MessageCodec::encode(&msg)?;
        
        let stream = connection.stream.write().await;
        
        timeout(self.config.io_timeout, async {
            stream.try_write(&encoded)
        }).await
            .map_err(|_| Error::Network("Send timeout".to_string()))?
            .map_err(|e| Error::Network(format!("Send error: {}", e)))?;
            
        // Update last activity
        *connection.last_activity.write().await = std::time::Instant::now();
        
        if let Some(metrics) = &self.metrics {
            metrics.record_signal_sent();
        }
        
        Ok(())
    }
    
    /// Get signal receiver
    pub async fn signal_receiver(&self) -> Option<mpsc::Receiver<(String, NeuronSignal)>> {
        self.signal_rx.write().await.take()
    }
    
    /// Start connection keeper to maintain alive connections
    async fn start_connection_keeper(&self) {
        let connections = self.connections.clone();
        let keep_alive_interval = self.config.keep_alive_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(keep_alive_interval);
            
            loop {
                interval.tick().await;
                
                let now = std::time::Instant::now();
                let mut dead_connections = Vec::new();
                
                // Check all connections
                for entry in connections.iter() {
                    let peer_id = entry.key().clone();
                    let connection = entry.value();
                    
                    let last_activity = *connection.last_activity.read().await;
                    if now.duration_since(last_activity) > keep_alive_interval * 2 {
                        warn!("Connection to {} appears dead", peer_id);
                        dead_connections.push(peer_id);
                    }
                }
                
                // Remove dead connections
                for peer_id in dead_connections {
                    connections.remove(&peer_id);
                    info!("Removed dead connection to {}", peer_id);
                }
            }
        });
    }
    
    /// Shutdown the transport
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down TCP transport");
        
        // Signal shutdown
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
        }
        
        // Close all connections
        for entry in self.connections.iter() {
            let connection = entry.value();
            if let Ok(_) = connection.stream.write().await.shutdown().await {
                debug!("Closed connection to {}", entry.key());
            }
        }
        
        self.connections.clear();
        
        Ok(())
    }
    
    /// Get list of connected peers
    pub fn connected_peers(&self) -> Vec<String> {
        self.connections.iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
    
    /// Check if connected to a specific peer
    pub fn is_connected(&self, server_id: &str) -> bool {
        self.connections.contains_key(server_id)
    }
}