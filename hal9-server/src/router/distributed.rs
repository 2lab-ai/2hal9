//! Distributed signal router for cross-server communication

use std::sync::Arc;
use std::net::SocketAddr;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use dashmap::DashMap;

use hal9_core::{Result, Error, NeuronSignal};
use crate::{
    router::local::SignalRouter,
    network::{
        TcpTransport, 
        discovery::{ServiceDiscovery, DiscoveryEvent, ServerInfo},
    },
};

/// Distributed routing configuration
#[derive(Debug, Clone)]
pub struct DistributedConfig {
    /// Enable distributed routing
    pub enabled: bool,
    /// Max hops for signal forwarding
    pub max_hops: u32,
    /// Remote routing timeout
    pub remote_timeout: std::time::Duration,
    /// Enable automatic peer discovery
    pub auto_discovery: bool,
}

impl Default for DistributedConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_hops: 5,
            remote_timeout: std::time::Duration::from_secs(30),
            auto_discovery: true,
        }
    }
}

/// Distributed signal router
pub struct DistributedRouter {
    /// Local server ID
    server_id: String,
    /// Local signal router
    local_router: Arc<SignalRouter>,
    /// TCP transport
    transport: Arc<TcpTransport>,
    /// Service discovery
    discovery: Arc<RwLock<ServiceDiscovery>>,
    /// Remote neuron mapping (neuron_id -> server_id)
    remote_neurons: Arc<DashMap<String, String>>,
    /// Configuration
    config: DistributedConfig,
    /// Shutdown signal
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl DistributedRouter {
    /// Create a new distributed router
    pub fn new(
        server_id: String,
        local_router: Arc<SignalRouter>,
        transport: Arc<TcpTransport>,
        discovery: Arc<RwLock<ServiceDiscovery>>,
        config: DistributedConfig,
    ) -> Self {
        Self {
            server_id,
            local_router,
            transport,
            discovery,
            remote_neurons: Arc::new(DashMap::new()),
            config,
            shutdown_tx: None,
        }
    }
    
    /// Start the distributed router
    pub async fn start(&mut self) -> Result<()> {
        if !self.config.enabled {
            info!("Distributed routing is disabled");
            return Ok(());
        }
        
        info!("Starting distributed router for server {}", self.server_id);
        
        // Start discovery if enabled
        if self.config.auto_discovery {
            self.start_discovery_handler().await?;
        }
        
        // Start signal receiver from transport
        self.start_network_receiver().await?;
        
        Ok(())
    }
    
    /// Start discovery event handler
    async fn start_discovery_handler(&mut self) -> Result<()> {
        let mut update_rx = self.discovery.read().await
            .update_receiver().await
            .ok_or_else(|| Error::InvalidState("Discovery not initialized".to_string()))?;
            
        let remote_neurons = self.remote_neurons.clone();
        let transport = self.transport.clone();
        
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(event) = update_rx.recv() => {
                        match event {
                            DiscoveryEvent::ServerDiscovered(server) | 
                            DiscoveryEvent::ServerUpdated(server) => {
                                Self::handle_server_update(server, &remote_neurons, &transport).await;
                            }
                            DiscoveryEvent::ServerLost(server_id) => {
                                Self::handle_server_lost(&server_id, &remote_neurons).await;
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Discovery handler shutting down");
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Handle server discovery/update
    async fn handle_server_update(
        server: ServerInfo,
        remote_neurons: &Arc<DashMap<String, String>>,
        transport: &Arc<TcpTransport>,
    ) {
        info!("Discovered server {} with {} neurons", server.server_id, server.neurons.len());
        
        // Update remote neuron mappings
        for neuron in &server.neurons {
            remote_neurons.insert(neuron.id.clone(), server.server_id.clone());
            debug!("Mapped remote neuron {} to server {}", neuron.id, server.server_id);
        }
        
        // Connect to the server if not already connected
        if !transport.is_connected(&server.server_id) {
            if let Err(e) = transport.connect(server.address, &server.server_id).await {
                error!("Failed to connect to {}: {}", server.server_id, e);
            } else {
                info!("Connected to remote server {}", server.server_id);
            }
        }
    }
    
    /// Handle server loss
    async fn handle_server_lost(
        server_id: &str,
        remote_neurons: &Arc<DashMap<String, String>>,
    ) {
        warn!("Lost connection to server {}", server_id);
        
        // Remove all neurons mapped to this server
        let mut removed = 0;
        remote_neurons.retain(|_, v| {
            if v == server_id {
                removed += 1;
                false
            } else {
                true
            }
        });
        
        if removed > 0 {
            debug!("Removed {} remote neuron mappings for server {}", removed, server_id);
        }
    }
    
    /// Start network signal receiver
    async fn start_network_receiver(&mut self) -> Result<()> {
        let mut signal_rx = self.transport.signal_receiver().await
            .ok_or_else(|| Error::InvalidState("Transport not initialized".to_string()))?;
            
        let local_router = self.local_router.clone();
        let _server_id = self.server_id.clone();
        
        tokio::spawn(async move {
            while let Some((from_server, signal)) = signal_rx.recv().await {
                debug!("Received signal from {}: {:?}", from_server, signal.signal_id);
                
                // Check hop count
                let hop_count = signal.metadata.get("hop_count")
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(0);
                    
                if hop_count >= 5 { // Max hops hardcoded for safety
                    warn!("Signal {} exceeded max hops", signal.signal_id);
                    continue;
                }
                
                // Add server trace
                let mut signal = signal;
                signal.metadata.insert("from_server".to_string(), from_server);
                signal.metadata.insert("hop_count".to_string(), (hop_count + 1).to_string());
                
                // Route locally
                if let Err(e) = local_router.send_signal(signal).await {
                    error!("Failed to route remote signal: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    /// Route a signal (local or remote)
    pub async fn route_signal(&self, signal: NeuronSignal) -> Result<()> {
        // First check if target is local
        if self.is_local_neuron(&signal.to_neuron).await {
            debug!("Routing signal {} locally", signal.signal_id);
            return self.local_router.send_signal(signal).await;
        }
        
        // Check if target is remote
        if let Some(server_id) = self.remote_neurons.get(&signal.to_neuron) {
            let server_id = server_id.clone();
            debug!("Routing signal {} to remote server {}", signal.signal_id, server_id);
            
            // Add metadata
            let mut signal = signal;
            signal.metadata.insert("via_server".to_string(), self.server_id.clone());
            
            // Send over network
            return self.transport.send_signal(&server_id, signal).await;
        }
        
        // Unknown target
        Err(Error::Routing(format!("Unknown neuron: {}", signal.to_neuron)))
    }
    
    /// Check if a neuron is local
    async fn is_local_neuron(&self, neuron_id: &str) -> bool {
        // This would check against local registry
        // For now, assume neurons not in remote_neurons are local
        !self.remote_neurons.contains_key(neuron_id)
    }
    
    /// Manually add a remote server
    pub async fn add_remote_server(
        &self,
        server_id: String,
        address: SocketAddr,
        neurons: Vec<String>,
    ) -> Result<()> {
        // Connect to server
        self.transport.connect(address, &server_id).await?;
        
        // Update neuron mappings
        for neuron_id in neurons {
            self.remote_neurons.insert(neuron_id, server_id.clone());
        }
        
        info!("Added remote server {} with address {}", server_id, address);
        Ok(())
    }
    
    /// Get routing information
    pub fn get_routing_info(&self) -> RoutingInfo {
        let remote_neurons: std::collections::HashMap<String, String> = 
            self.remote_neurons.iter()
                .map(|entry| (entry.key().clone(), entry.value().clone()))
                .collect();
                
        let connected_servers = self.transport.connected_peers();
        
        RoutingInfo {
            server_id: self.server_id.clone(),
            remote_neurons,
            connected_servers,
        }
    }
    
    /// Update local neurons in discovery
    pub async fn update_local_neurons(&self, neurons: Vec<String>) -> Result<()> {
        use crate::network::protocol::NeuronInfo;
        
        let neuron_infos: Vec<NeuronInfo> = neurons.into_iter()
            .map(|id| NeuronInfo {
                id: id.clone(),
                layer: "Unknown".to_string(), // TODO: Get actual layer
                server_id: self.server_id.clone(),
            })
            .collect();
            
        self.discovery.read().await.update_neurons(neuron_infos).await;
        Ok(())
    }
    
    /// Shutdown the distributed router
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down distributed router");
        
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
        }
        
        Ok(())
    }
}

/// Routing information
#[derive(Debug, Clone)]
pub struct RoutingInfo {
    pub server_id: String,
    pub remote_neurons: std::collections::HashMap<String, String>,
    pub connected_servers: Vec<String>,
}