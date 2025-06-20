//! Main 2HAL9 server implementation

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, broadcast};
use tracing::{info, error};

use hal9_core::{Error, Result, ServerConfig, NeuronSignal, neuron::NeuronHealth};
use hal9_core::auth::{UserManager, JwtManager, ApiKeyManager};
use crate::{
    api::WsMessage,
    claude::{ClaudeInterface, MockClaude, ClaudeAPIClient, FallbackClaude, HybridClaude},
    cost_tracker::CostTracker,
    error::{ServerError, ServerResult},
    neuron::{ManagedNeuron, NeuronRegistry},
    router::{SignalRouter, RoutingTable, DistributedRouter, DistributedConfig},
    metrics::Metrics,
    network::{TcpTransport, ServiceDiscovery},
};

/// Network status information
#[derive(Debug, Clone, serde::Serialize)]
pub struct NetworkStatus {
    pub enabled: bool,
    pub server_id: String,
    pub connected_servers: Vec<String>,
    pub remote_neurons: usize,
}

/// Main HAL9 server
pub struct HAL9Server {
    config: ServerConfig,
    registry: Arc<NeuronRegistry>,
    routing_table: Arc<RoutingTable>,
    router: RwLock<Option<SignalRouter>>,
    distributed_router: RwLock<Option<Arc<DistributedRouter>>>,
    transport: RwLock<Option<Arc<TcpTransport>>>,
    discovery: RwLock<Option<Arc<RwLock<ServiceDiscovery>>>>,
    metrics: Arc<Metrics>,
    cost_tracker: Arc<CostTracker>,
    event_tx: broadcast::Sender<WsMessage>,
    start_time: RwLock<Option<Instant>>,
    // Authentication components
    pub user_manager: Option<Arc<UserManager>>,
    pub jwt_manager: Option<Arc<JwtManager>>,
    pub api_key_manager: Option<Arc<ApiKeyManager>>,
}

impl HAL9Server {
    /// Create a new server instance
    pub fn new(config: ServerConfig) -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        
        // Create metrics first
        let metrics = Arc::new(Metrics::new());
        
        // Create cost tracker with configuration
        let mut cost_tracker = CostTracker::new(config.claude.cost_controls.clone());
        cost_tracker.set_metrics(metrics.clone());
        let cost_tracker = Arc::new(cost_tracker);
        
        Self {
            config,
            registry: Arc::new(NeuronRegistry::new()),
            routing_table: Arc::new(RoutingTable::new()),
            router: RwLock::new(None),
            distributed_router: RwLock::new(None),
            transport: RwLock::new(None),
            discovery: RwLock::new(None),
            metrics,
            cost_tracker,
            event_tx,
            start_time: RwLock::new(None),
            user_manager: None,
            jwt_manager: None,
            api_key_manager: None,
        }
    }
    
    /// Initialize authentication if enabled
    pub async fn initialize_auth(&mut self, pool: sqlx::SqlitePool) -> Result<()> {
        if self.config.auth.enabled {
            info!("Initializing authentication system");
            
            // Create managers
            let user_manager = Arc::new(UserManager::new(pool.clone()));
            let jwt_manager = Arc::new(JwtManager::with_durations(
                self.config.auth.jwt_secret.clone(),
                self.config.auth.access_token_duration_minutes,
                self.config.auth.refresh_token_duration_days,
            ));
            let api_key_manager = Arc::new(ApiKeyManager::new(pool.clone()));
            
            // Initialize tables
            user_manager.initialize().await
                .map_err(|e| Error::Other(anyhow::anyhow!("Failed to initialize user tables: {}", e)))?;
            api_key_manager.initialize().await
                .map_err(|e| Error::Other(anyhow::anyhow!("Failed to initialize API key tables: {}", e)))?;
            
            self.user_manager = Some(user_manager);
            self.jwt_manager = Some(jwt_manager);
            self.api_key_manager = Some(api_key_manager);
            
            info!("Authentication system initialized");
        }
        
        Ok(())
    }
    
    /// Start the server
    pub async fn start(&self) -> Result<()> {
        info!("Starting 2HAL9 server: {}", self.config.server_id);
        
        // Record start time
        *self.start_time.write().await = Some(Instant::now());
        
        // Build routing table
        self.routing_table.build_from_configs(&self.config.neurons);
        
        // Set metrics for registry
        let registry_ref = unsafe { 
            // Safe because we're the only ones with mutable access during startup
            &mut *(Arc::as_ptr(&self.registry) as *mut NeuronRegistry) 
        };
        registry_ref.set_metrics(self.metrics.clone());
        
        // Initialize memory system if enabled
        let memory_store = if self.config.memory.enabled {
            info!("Initializing memory system");
            let memory_manager = crate::memory_manager::MemoryManager::new(&self.config.memory).await?;
            let store = memory_manager.get_store();
            
            // Start cleanup task if enabled
            if self.config.memory.cleanup.retention_days > 0 {
                let manager = Arc::new(memory_manager);
                let cleanup_config = self.config.memory.cleanup.clone();
                tokio::spawn(crate::memory_manager::cleanup_task(manager, cleanup_config));
            }
            
            Some(store)
        } else {
            None
        };
        
        // Spawn neurons
        for neuron_config in &self.config.neurons {
            let claude = self.create_claude_instance(&neuron_config.layer).await?;
            let mut neuron = ManagedNeuron::new(neuron_config.clone(), claude)?;
            
            // Set memory store if available
            if let Some(store) = &memory_store {
                neuron.set_memory_store(store.clone());
            }
            
            // Enable backward propagation if configured
            if self.config.backward_propagation.enabled {
                let base_prompt = neuron_config.system_prompt.clone()
                    .unwrap_or_else(|| format!("You are neuron {} on layer {}", neuron_config.id, neuron_config.layer));
                neuron.enable_backward_propagation(self.config.backward_propagation.clone(), base_prompt);
            }
            
            self.registry.register(neuron).await?;
        }
        
        // Update metrics
        self.metrics.set_active_neurons(self.config.neurons.len() as u64);
        
        // Start periodic metrics reporting if enabled
        if self.config.monitoring.enabled {
            self.start_metrics_reporter().await;
        }
        
        // Initialize network if enabled
        if self.config.network.enabled {
            info!("Initializing distributed networking");
            
            // Create TCP transport
            let bind_address = self.config.network.bind_address.parse()
                .map_err(|e| Error::Config(format!("Invalid bind address: {}", e)))?;
                
            let transport_config = crate::network::tcp_transport::TransportConfig {
                bind_address,
                max_connections: self.config.network.max_connections,
                ..Default::default()
            };
            
            let mut transport = TcpTransport::new(transport_config, self.config.server_id.clone());
            transport.set_metrics(self.metrics.clone());
            transport.start().await?;
            let transport = Arc::new(transport);
            *self.transport.write().await = Some(transport.clone());
            
            // Create service discovery if enabled
            if self.config.network.discovery_enabled {
                let discovery_addr = self.config.network.discovery_address.parse()
                    .map_err(|e| Error::Config(format!("Invalid discovery address: {}", e)))?;
                    
                let discovery_config = crate::network::discovery::DiscoveryConfig {
                    multicast_addr: discovery_addr,
                    discovery_group: self.config.network.discovery_group.clone(),
                    enabled: true,
                    ..Default::default()
                };
                
                let mut discovery = ServiceDiscovery::new(
                    discovery_config,
                    self.config.server_id.clone(),
                    self.config.network.bind_address.parse()
                        .unwrap_or_else(|_| "0.0.0.0:9000".parse().unwrap()),
                );
                
                // Get local neurons for announcement
                let local_neurons = self.config.neurons.iter()
                    .map(|n| crate::network::protocol::NeuronInfo {
                        id: n.id.clone(),
                        layer: n.layer.clone(),
                        server_id: self.config.server_id.clone(),
                    })
                    .collect();
                    
                discovery.update_neurons(local_neurons).await;
                discovery.start().await?;
                *self.discovery.write().await = Some(Arc::new(RwLock::new(discovery)));
            }
        }
        
        // Start signal router
        let mut router = SignalRouter::new(
            self.registry.clone(),
            self.routing_table.clone(),
        );
        router.start().await?;
        
        // Store the router for local use
        *self.router.write().await = Some(router);
        
        // Initialize distributed router if network is enabled
        if self.config.network.enabled {
            if let (Some(transport), Some(discovery)) = (
                self.transport.read().await.as_ref(),
                self.discovery.read().await.as_ref()
            ) {
                info!("Initializing distributed router");
                
                // Create distributed router config
                let dist_config = DistributedConfig {
                    enabled: true,
                    max_hops: 5,
                    remote_timeout: std::time::Duration::from_secs(30),
                    auto_discovery: self.config.network.discovery_enabled,
                };
                
                // Get a reference to the router we just stored
                let router_guard = self.router.read().await;
                if let Some(ref _router) = *router_guard {
                    // Create a new router for the distributed router
                    // This is necessary because distributed router needs ownership
                    let mut distributed_local_router = SignalRouter::new(
                        self.registry.clone(),
                        self.routing_table.clone(),
                    );
                    distributed_local_router.start().await?;
                    
                    // Create distributed router using the new started router
                    let mut distributed_router = DistributedRouter::new(
                        self.config.server_id.clone(),
                        Arc::new(distributed_local_router),
                        transport.clone(),
                        discovery.clone(),
                        dist_config,
                    );
                    
                    // Start the distributed router
                    distributed_router.start().await?;
                    
                    *self.distributed_router.write().await = Some(Arc::new(distributed_router));
                    
                    info!("Distributed router initialized and started");
                } else {
                    error!("Failed to get router reference for distributed routing");
                }
            } else {
                info!("Network components not ready for distributed routing");
            }
        }
        
        info!("Server started with {} neurons", self.config.neurons.len());
        Ok(())
    }
    
    /// Create Claude instance based on configuration
    async fn create_claude_instance(&self, layer: &str) -> Result<Box<dyn ClaudeInterface>> {
        match self.config.claude.mode.as_str() {
            "mock" => {
                info!("Creating mock Claude for layer {}", layer);
                Ok(Box::new(MockClaude::new(layer, &self.config.claude)))
            }
            "api" => {
                info!("Creating Claude API client for layer {}", layer);
                let api_key = self.config.claude.api_key.clone()
                    .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                    .ok_or_else(|| Error::Config("Claude API key not found".to_string()))?;
                    
                let mut api_client = ClaudeAPIClient::new(
                    api_key,
                    self.config.claude.model.clone(),
                    layer,
                    self.config.claude.temperature,
                    self.config.claude.max_tokens,
                );
                
                // Set cost tracker
                api_client.set_cost_tracker(self.cost_tracker.clone());
                
                // If fallback is enabled, wrap in FallbackClaude
                if self.config.claude.fallback_to_mock {
                    info!("Enabling fallback to mock mode for layer {}", layer);
                    let mock_client = Box::new(MockClaude::new(layer, &self.config.claude));
                    Ok(Box::new(FallbackClaude::new(Box::new(api_client), mock_client)))
                } else {
                    Ok(Box::new(api_client))
                }
            }
            "hybrid" | "auto" => {
                info!("Creating hybrid Claude for layer {} (mode: {})", layer, self.config.claude.mode);
                Ok(Box::new(HybridClaude::new(
                    layer,
                    &self.config.claude,
                    self.cost_tracker.clone(),
                )?))
            }
            mode => Err(Error::Config(format!("Unknown Claude mode: {}", mode))),
        }
    }
    
    /// Send a signal to the network
    pub async fn send_signal(&self, signal: NeuronSignal) -> Result<()> {
        self.metrics.record_signal_sent();
        
        // Use distributed router if available
        if let Some(distributed_router) = self.distributed_router.read().await.as_ref() {
            distributed_router.route_signal(signal).await
        } else if let Some(router) = self.router.read().await.as_ref() {
            router.send_signal(signal).await
        } else {
            Err(Error::InvalidState("Server not started".to_string()))
        }
    }
    
    /// Get server status
    pub async fn status(&self) -> ServerStatus {
        let health = self.registry.health_check().await;
        let metrics = self.metrics.snapshot();
        
        ServerStatus {
            server_id: self.config.server_id.clone(),
            running: self.router.read().await.is_some(),
            neurons: health,
            metrics,
        }
    }
    
    /// Shutdown the server
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down 2HAL9 server");
        
        // Stop distributed router first
        if let Some(_distributed_router) = self.distributed_router.write().await.take() {
            // The Arc will be dropped when this scope ends
            info!("Distributed router stopped");
        }
        
        // Stop local router
        if let Some(mut router) = self.router.write().await.take() {
            router.shutdown().await?;
        }
        
        // Stop network components
        if let Some(_transport) = self.transport.write().await.take() {
            // Transport will be dropped when Arc goes out of scope
            // This will trigger cleanup in the Drop implementation
            info!("Network transport stopped");
        }
        
        if let Some(_discovery) = self.discovery.write().await.take() {
            // Discovery will be dropped when Arc goes out of scope
            info!("Service discovery stopped");
        }
        
        // Shutdown all neurons
        self.registry.shutdown_all().await?;
        
        info!("Server shutdown complete");
        Ok(())
    }
    
    /// Get metrics
    pub fn metrics(&self) -> Arc<Metrics> {
        self.metrics.clone()
    }
    
    /// Get neuron registry
    pub fn registry(&self) -> Arc<NeuronRegistry> {
        self.registry.clone()
    }
    
    /// Get server ID
    pub fn server_id(&self) -> &str {
        &self.config.server_id
    }
    
    /// Get network status
    pub async fn network_status(&self) -> Option<NetworkStatus> {
        if let Some(distributed_router) = self.distributed_router.read().await.as_ref() {
            let routing_info = distributed_router.get_routing_info();
            Some(NetworkStatus {
                enabled: true,
                server_id: routing_info.server_id,
                connected_servers: routing_info.connected_servers,
                remote_neurons: routing_info.remote_neurons.len(),
            })
        } else {
            None
        }
    }
    
    
    // API-specific methods
    
    /// Get server status (API wrapper)
    pub async fn get_status(&self) -> ServerResult<ExtendedServerStatus> {
        let uptime = self.start_time.read().await
            .map(|start| start.elapsed())
            .unwrap_or_default();
            
        let neurons = self.registry.list_all().await;
        let metrics = self.metrics.snapshot();
        let network_status = self.network_status().await;
        
        Ok(ExtendedServerStatus {
            running: self.router.read().await.is_some(),
            uptime,
            neurons,
            metrics,
            network_status,
        })
    }
    
    /// Submit a signal to the network
    pub async fn submit_signal(&self, signal: NeuronSignal) -> ServerResult<String> {
        let signal_id = signal.signal_id.to_string();
        
        // Send signal
        self.send_signal(signal.clone()).await
            .map_err(|e| ServerError::RoutingError(e.to_string()))?;
            
        // Broadcast event
        let _ = self.event_tx.send(WsMessage::SignalUpdate {
            signal_id: signal_id.clone(),
            neuron_id: signal.to_neuron.clone(),
            status: "submitted".to_string(),
        });
        
        Ok(signal_id)
    }
    
    /// List all neurons
    pub async fn list_neurons(&self) -> ServerResult<Vec<NeuronInfo>> {
        Ok(self.registry.list_all().await)
    }
    
    /// Get specific neuron info
    pub async fn get_neuron_info(&self, neuron_id: &str) -> ServerResult<NeuronInfo> {
        self.registry.get_info(neuron_id).await
            .ok_or_else(|| ServerError::NotFound(format!("Neuron {} not found", neuron_id)))
    }
    
    /// Get neuron health
    pub async fn get_neuron_health(&self, neuron_id: &str) -> ServerResult<NeuronHealth> {
        let health_map = self.registry.health_check().await;
        health_map.get(neuron_id)
            .cloned()
            .ok_or_else(|| ServerError::NotFound(format!("Neuron {} not found", neuron_id)))
    }
    
    /// Get metrics
    pub async fn get_metrics(&self) -> ServerResult<crate::metrics::MetricsSnapshot> {
        Ok(self.metrics.snapshot())
    }
    
    /// Subscribe to server events
    pub async fn subscribe_to_events(&self) -> broadcast::Receiver<WsMessage> {
        self.event_tx.subscribe()
    }
    
    /// Broadcast an event
    pub fn broadcast_event(&self, event: WsMessage) {
        let _ = self.event_tx.send(event);
    }
    
    
    /// Get active user count
    pub async fn get_active_user_count(&self) -> Result<usize> {
        if let Some(ref _user_manager) = self.user_manager {
            // For now, return a placeholder count
            // In production, query the database for active sessions
            Ok(1)
        } else {
            Ok(0)
        }
    }
    
    /// Get MCP tool metrics
    pub async fn get_mcp_tool_metrics(&self) -> Option<McpToolMetrics> {
        // Placeholder for MCP tool metrics
        // TODO: Implement actual metric collection from MCP tools
        Some(McpToolMetrics {
            invocations: std::collections::HashMap::new(),
        })
    }
    
    /// Get memory system metrics
    pub async fn get_memory_metrics(&self) -> Option<MemoryMetrics> {
        // TODO: Integrate with memory manager when available
        // For now, return None as memory system integration is pending
        None
    }
    
    /// Get learning system metrics
    pub async fn get_learning_metrics(&self) -> Option<LearningMetrics> {
        if self.config.backward_propagation.enabled {
            // TODO: Get actual metrics from learning system
            Some(LearningMetrics {
                cycles_completed: 0,
                adjustments_success: 0,
                adjustments_failure: 0,
            })
        } else {
            None
        }
    }
    
    /// Start periodic metrics reporting
    async fn start_metrics_reporter(&self) {
        let metrics = self.metrics.clone();
        let interval = self.config.monitoring.metrics_interval;
        let server_id = self.config.server_id.clone();
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(Duration::from_secs(interval));
            loop {
                interval_timer.tick().await;
                
                // Update memory usage
                metrics.update_memory_usage();
                
                // Get snapshot
                let snapshot = metrics.snapshot();
                
                // Log metrics
                info!(
                    "[METRICS] {} - Signals: sent={}, processed={}, failed={}, rate={:.2}/s | Neurons: active={}, failed={}, processing={} | Tokens: total={} | Cost: hour=${:.2}, day=${:.2}, total=${:.2} | Memory: {:.2}MB",
                    server_id,
                    snapshot.signals_sent,
                    snapshot.signals_processed,
                    snapshot.signals_failed,
                    snapshot.signals_per_second,
                    snapshot.neurons_active,
                    snapshot.neurons_failed,
                    snapshot.neurons_processing,
                    snapshot.tokens_total,
                    snapshot.cost_hourly,
                    snapshot.cost_daily,
                    snapshot.cost_total,
                    snapshot.memory_usage_mb
                );
                
                // Log layer latencies if available
                for (layer, stats) in &snapshot.layer_latencies {
                    info!(
                        "[METRICS] {} - Layer {} latency: avg={:.2}ms, min={:.2}ms, max={:.2}ms (n={})",
                        server_id, layer, stats.avg_ms, stats.min_ms, stats.max_ms, stats.count
                    );
                }
                
                // Log errors if any
                if !snapshot.errors_by_type.is_empty() {
                    for (error_type, count) in &snapshot.errors_by_type {
                        info!("[METRICS] {} - Error '{}': {} occurrences", server_id, error_type, count);
                    }
                }
            }
        });
    }
}

/// Server status information
#[derive(Debug, serde::Serialize)]
pub struct ServerStatus {
    pub server_id: String,
    pub running: bool,
    pub neurons: std::collections::HashMap<String, NeuronHealth>,
    pub metrics: crate::metrics::MetricsSnapshot,
}

/// Extended server status for API
#[derive(Debug, serde::Serialize)]
pub struct ExtendedServerStatus {
    pub running: bool,
    pub uptime: Duration,
    pub neurons: Vec<NeuronInfo>,
    pub metrics: crate::metrics::MetricsSnapshot,
    pub network_status: Option<NetworkStatus>,
}

/// Neuron information
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeuronInfo {
    pub id: String,
    pub layer: String,
    pub state: String,
    pub is_healthy: bool,
}

/// MCP tool metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct McpToolMetrics {
    pub invocations: std::collections::HashMap<String, ToolInvocations>,
}

/// Tool invocation counts
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolInvocations {
    pub success: u64,
    pub error: u64,
}

/// Memory system metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct MemoryMetrics {
    pub total_entries: u64,
    pub database_size_bytes: u64,
}

/// Learning system metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct LearningMetrics {
    pub cycles_completed: u64,
    pub adjustments_success: u64,
    pub adjustments_failure: u64,
}