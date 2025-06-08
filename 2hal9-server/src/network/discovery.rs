//! Service discovery for distributed neuron servers

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::net::UdpSocket;
use tracing::{debug, error, info, warn};
use dashmap::DashMap;
use serde::{Serialize, Deserialize};

use twohal9_core::{Result, Error};
use crate::network::protocol::NeuronInfo;

/// Discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Multicast address for discovery
    pub multicast_addr: SocketAddr,
    /// Broadcast interval
    pub broadcast_interval: Duration,
    /// Server timeout (remove if not seen)
    pub server_timeout: Duration,
    /// Enable discovery
    pub enabled: bool,
    /// Discovery group (for multi-tenancy)
    pub discovery_group: String,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            multicast_addr: "239.255.42.99:8888".parse().unwrap(),
            broadcast_interval: Duration::from_secs(10),
            server_timeout: Duration::from_secs(60),
            enabled: true,
            discovery_group: "default".to_string(),
        }
    }
}

/// Information about a discovered server
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub server_id: String,
    pub address: SocketAddr,
    pub neurons: Vec<NeuronInfo>,
    pub last_seen: Instant,
    pub version: String,
}

/// Discovery message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveryMessage {
    /// Message type
    msg_type: DiscoveryMessageType,
    /// Discovery group
    group: String,
    /// Server ID
    server_id: String,
    /// Server address
    address: String,
    /// Available neurons
    neurons: Vec<NeuronInfo>,
    /// Server version
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum DiscoveryMessageType {
    Announce,
    Request,
    Response,
    Goodbye,
}

/// Service discovery implementation
pub struct ServiceDiscovery {
    config: DiscoveryConfig,
    local_server_id: String,
    local_address: SocketAddr,
    local_neurons: Arc<RwLock<Vec<NeuronInfo>>>,
    discovered_servers: Arc<DashMap<String, ServerInfo>>,
    socket: Option<Arc<UdpSocket>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    update_tx: mpsc::Sender<DiscoveryEvent>,
    update_rx: Arc<RwLock<Option<mpsc::Receiver<DiscoveryEvent>>>>,
}

/// Discovery events
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    ServerDiscovered(ServerInfo),
    ServerUpdated(ServerInfo),
    ServerLost(String),
}

impl ServiceDiscovery {
    /// Create a new service discovery instance
    pub fn new(
        config: DiscoveryConfig,
        server_id: String,
        bind_address: SocketAddr,
    ) -> Self {
        let (update_tx, update_rx) = mpsc::channel(100);
        
        Self {
            config,
            local_server_id: server_id,
            local_address: bind_address,
            local_neurons: Arc::new(RwLock::new(Vec::new())),
            discovered_servers: Arc::new(DashMap::new()),
            socket: None,
            shutdown_tx: None,
            update_tx,
            update_rx: Arc::new(RwLock::new(Some(update_rx))),
        }
    }
    
    /// Update local neuron list
    pub async fn update_neurons(&self, neurons: Vec<NeuronInfo>) {
        *self.local_neurons.write().await = neurons;
    }
    
    /// Start discovery service
    pub async fn start(&mut self) -> Result<()> {
        if !self.config.enabled {
            info!("Service discovery is disabled");
            return Ok(());
        }
        
        // Bind UDP socket
        let socket = UdpSocket::bind("0.0.0.0:0").await
            .map_err(|e| Error::Network(format!("Failed to bind discovery socket: {}", e)))?;
            
        // For now, skip multicast setup as it requires platform-specific handling
        // In production, use a proper multicast library or implement platform-specific code
        
        // socket.join_multicast_v4(...) - TODO: Implement proper multicast
        
        info!("Service discovery started on {:?}", socket.local_addr());
        
        self.socket = Some(Arc::new(socket));
        
        // Start background tasks
        self.start_broadcast_task().await;
        self.start_receive_task().await;
        self.start_cleanup_task().await;
        
        // Send initial announcement
        self.send_announcement().await?;
        
        Ok(())
    }
    
    /// Start broadcast task
    async fn start_broadcast_task(&mut self) {
        let socket = self.socket.as_ref().unwrap().clone();
        let interval = self.config.broadcast_interval;
        let multicast_addr = self.config.multicast_addr;
        let server_id = self.local_server_id.clone();
        let address = self.local_address;
        let neurons = self.local_neurons.clone();
        let group = self.config.discovery_group.clone();
        
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let msg = DiscoveryMessage {
                            msg_type: DiscoveryMessageType::Announce,
                            group: group.clone(),
                            server_id: server_id.clone(),
                            address: address.to_string(),
                            neurons: neurons.read().await.clone(),
                            version: "1.0".to_string(),
                        };
                        
                        if let Ok(data) = serde_json::to_vec(&msg) {
                            if let Err(e) = socket.send_to(&data, multicast_addr).await {
                                error!("Failed to send discovery announcement: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Broadcast task shutting down");
                        break;
                    }
                }
            }
        });
    }
    
    /// Start receive task
    async fn start_receive_task(&self) {
        let socket = self.socket.as_ref().unwrap().clone();
        let local_server_id = self.local_server_id.clone();
        let discovered_servers = self.discovered_servers.clone();
        let update_tx = self.update_tx.clone();
        let group = self.config.discovery_group.clone();
        
        tokio::spawn(async move {
            let mut buf = vec![0u8; 65536];
            
            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((len, addr)) => {
                        if let Ok(msg) = serde_json::from_slice::<DiscoveryMessage>(&buf[..len]) {
                            // Ignore our own messages
                            if msg.server_id == local_server_id {
                                continue;
                            }
                            
                            // Ignore different groups
                            if msg.group != group {
                                continue;
                            }
                            
                            // Handle message
                            Self::handle_discovery_message(
                                msg,
                                addr,
                                &discovered_servers,
                                &update_tx
                            ).await;
                        }
                    }
                    Err(e) => {
                        error!("Discovery receive error: {}", e);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        });
    }
    
    /// Handle discovery message
    async fn handle_discovery_message(
        msg: DiscoveryMessage,
        from_addr: SocketAddr,
        discovered_servers: &Arc<DashMap<String, ServerInfo>>,
        update_tx: &mpsc::Sender<DiscoveryEvent>,
    ) {
        match msg.msg_type {
            DiscoveryMessageType::Announce | DiscoveryMessageType::Response => {
                let server_addr: SocketAddr = msg.address.parse()
                    .unwrap_or(from_addr);
                    
                let server_info = ServerInfo {
                    server_id: msg.server_id.clone(),
                    address: server_addr,
                    neurons: msg.neurons,
                    last_seen: Instant::now(),
                    version: msg.version,
                };
                
                // Check if new or update
                let is_new = !discovered_servers.contains_key(&msg.server_id);
                discovered_servers.insert(msg.server_id.clone(), server_info.clone());
                
                // Send event
                let event = if is_new {
                    info!("Discovered new server: {} at {}", msg.server_id, server_addr);
                    DiscoveryEvent::ServerDiscovered(server_info)
                } else {
                    debug!("Updated server info: {}", msg.server_id);
                    DiscoveryEvent::ServerUpdated(server_info)
                };
                
                let _ = update_tx.send(event).await;
            }
            DiscoveryMessageType::Request => {
                // TODO: Send response
                debug!("Received discovery request from {}", from_addr);
            }
            DiscoveryMessageType::Goodbye => {
                if discovered_servers.remove(&msg.server_id).is_some() {
                    info!("Server {} said goodbye", msg.server_id);
                    let _ = update_tx.send(DiscoveryEvent::ServerLost(msg.server_id)).await;
                }
            }
        }
    }
    
    /// Start cleanup task
    async fn start_cleanup_task(&self) {
        let discovered_servers = self.discovered_servers.clone();
        let server_timeout = self.config.server_timeout;
        let update_tx = self.update_tx.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                let now = Instant::now();
                let mut expired = Vec::new();
                
                // Find expired servers
                for entry in discovered_servers.iter() {
                    if now.duration_since(entry.value().last_seen) > server_timeout {
                        expired.push(entry.key().clone());
                    }
                }
                
                // Remove expired servers
                for server_id in expired {
                    if discovered_servers.remove(&server_id).is_some() {
                        warn!("Server {} timed out", server_id);
                        let _ = update_tx.send(DiscoveryEvent::ServerLost(server_id)).await;
                    }
                }
            }
        });
    }
    
    /// Send announcement
    async fn send_announcement(&self) -> Result<()> {
        if let Some(socket) = &self.socket {
            let msg = DiscoveryMessage {
                msg_type: DiscoveryMessageType::Announce,
                group: self.config.discovery_group.clone(),
                server_id: self.local_server_id.clone(),
                address: self.local_address.to_string(),
                neurons: self.local_neurons.read().await.clone(),
                version: "1.0".to_string(),
            };
            
            let data = serde_json::to_vec(&msg)
                .map_err(|e| Error::Serialization(format!("Failed to serialize announcement: {}", e)))?;
                
            socket.send_to(&data, self.config.multicast_addr).await
                .map_err(|e| Error::Network(format!("Failed to send announcement: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Send goodbye message
    async fn send_goodbye(&self) -> Result<()> {
        if let Some(socket) = &self.socket {
            let msg = DiscoveryMessage {
                msg_type: DiscoveryMessageType::Goodbye,
                group: self.config.discovery_group.clone(),
                server_id: self.local_server_id.clone(),
                address: self.local_address.to_string(),
                neurons: vec![],
                version: "1.0".to_string(),
            };
            
            let data = serde_json::to_vec(&msg)
                .map_err(|e| Error::Serialization(format!("Failed to serialize goodbye: {}", e)))?;
                
            socket.send_to(&data, self.config.multicast_addr).await
                .map_err(|e| Error::Network(format!("Failed to send goodbye: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Get discovered servers
    pub fn get_servers(&self) -> Vec<ServerInfo> {
        self.discovered_servers.iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    /// Get specific server info
    pub fn get_server(&self, server_id: &str) -> Option<ServerInfo> {
        self.discovered_servers.get(server_id)
            .map(|entry| entry.value().clone())
    }
    
    /// Find servers with specific neuron
    pub fn find_servers_with_neuron(&self, neuron_id: &str) -> Vec<ServerInfo> {
        self.discovered_servers.iter()
            .filter(|entry| {
                entry.value().neurons.iter()
                    .any(|n| n.id == neuron_id)
            })
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    /// Get update receiver
    pub async fn update_receiver(&self) -> Option<mpsc::Receiver<DiscoveryEvent>> {
        self.update_rx.write().await.take()
    }
    
    /// Shutdown discovery
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down service discovery");
        
        // Send goodbye
        let _ = self.send_goodbye().await;
        
        // Signal shutdown
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
        }
        
        Ok(())
    }
}