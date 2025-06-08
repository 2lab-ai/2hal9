//! Main 2HAL9 server implementation

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, broadcast};
use tracing::info;

use twohal9_core::{Error, Result, ServerConfig, NeuronSignal, neuron::NeuronHealth};
use crate::{
    api::WsMessage,
    claude::{ClaudeInterface, MockClaude, ClaudeAPIClient, FallbackClaude},
    error::{ServerError, ServerResult},
    neuron::{ManagedNeuron, NeuronRegistry},
    router::{SignalRouter, RoutingTable},
    metrics::Metrics,
};

/// Main HAL9 server
pub struct HAL9Server {
    config: ServerConfig,
    registry: Arc<NeuronRegistry>,
    routing_table: Arc<RoutingTable>,
    router: RwLock<Option<SignalRouter>>,
    metrics: Arc<Metrics>,
    event_tx: broadcast::Sender<WsMessage>,
    start_time: RwLock<Option<Instant>>,
}

impl HAL9Server {
    /// Create a new server instance
    pub fn new(config: ServerConfig) -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        Self {
            config,
            registry: Arc::new(NeuronRegistry::new()),
            routing_table: Arc::new(RoutingTable::new()),
            router: RwLock::new(None),
            metrics: Arc::new(Metrics::new()),
            event_tx,
            start_time: RwLock::new(None),
        }
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
        
        // Spawn neurons
        for neuron_config in &self.config.neurons {
            let claude = self.create_claude_instance(&neuron_config.layer).await?;
            let neuron = ManagedNeuron::new(neuron_config.clone(), claude)?;
            self.registry.register(neuron).await?;
        }
        
        // Update metrics
        self.metrics.set_active_neurons(self.config.neurons.len() as u64);
        
        // Start periodic metrics reporting if enabled
        if self.config.monitoring.enabled {
            self.start_metrics_reporter().await;
        }
        
        // Start signal router
        let mut router = SignalRouter::new(
            self.registry.clone(),
            self.routing_table.clone(),
        );
        router.start().await?;
        
        *self.router.write().await = Some(router);
        
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
                    
                let api_client = Box::new(ClaudeAPIClient::new(
                    api_key,
                    self.config.claude.model.clone(),
                    layer,
                    self.config.claude.temperature,
                    self.config.claude.max_tokens,
                ));
                
                // If fallback is enabled, wrap in FallbackClaude
                if self.config.claude.fallback_to_mock {
                    info!("Enabling fallback to mock mode for layer {}", layer);
                    let mock_client = Box::new(MockClaude::new(layer, &self.config.claude));
                    Ok(Box::new(FallbackClaude::new(api_client, mock_client)))
                } else {
                    Ok(api_client)
                }
            }
            mode => Err(Error::Config(format!("Unknown Claude mode: {}", mode))),
        }
    }
    
    /// Send a signal to the network
    pub async fn send_signal(&self, signal: NeuronSignal) -> Result<()> {
        if let Some(router) = self.router.read().await.as_ref() {
            self.metrics.record_signal_sent();
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
        
        // Stop router
        if let Some(mut router) = self.router.write().await.take() {
            router.shutdown().await?;
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
    
    // API-specific methods
    
    /// Get server status (API wrapper)
    pub async fn get_status(&self) -> ServerResult<ExtendedServerStatus> {
        let uptime = self.start_time.read().await
            .map(|start| start.elapsed())
            .unwrap_or_default();
            
        let neurons = self.registry.list_all().await;
        let metrics = self.metrics.snapshot();
        
        Ok(ExtendedServerStatus {
            running: self.router.read().await.is_some(),
            uptime,
            neurons,
            metrics,
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
                    "[METRICS] {} - Signals: sent={}, processed={}, failed={}, rate={:.2}/s | Neurons: active={}, failed={}, processing={} | Tokens: total={} | Memory: {:.2}MB",
                    server_id,
                    snapshot.signals_sent,
                    snapshot.signals_processed,
                    snapshot.signals_failed,
                    snapshot.signals_per_second,
                    snapshot.neurons_active,
                    snapshot.neurons_failed,
                    snapshot.neurons_processing,
                    snapshot.tokens_total,
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
}

/// Neuron information
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeuronInfo {
    pub id: String,
    pub layer: String,
    pub state: String,
    pub is_healthy: bool,
}