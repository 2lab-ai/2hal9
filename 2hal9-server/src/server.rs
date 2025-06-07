//! Main 2HAL9 server implementation

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use 2hal9_core::{Error, Result, ServerConfig, NeuronSignal};
use crate::{
    claude::{ClaudeInterface, MockClaude, ClaudeAPIClient},
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
}

impl HAL9Server {
    /// Create a new server instance
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            registry: Arc::new(NeuronRegistry::new()),
            routing_table: Arc::new(RoutingTable::new()),
            router: RwLock::new(None),
            metrics: Arc::new(Metrics::new()),
        }
    }
    
    /// Start the server
    pub async fn start(&self) -> Result<()> {
        info!("Starting 2HAL9 server: {}", self.config.server_id);
        
        // Build routing table
        self.routing_table.build_from_configs(&self.config.neurons);
        
        // Spawn neurons
        for neuron_config in &self.config.neurons {
            let claude = self.create_claude_instance(&neuron_config.layer).await?;
            let neuron = ManagedNeuron::new(neuron_config.clone(), claude)?;
            self.registry.register(neuron).await?;
        }
        
        // Update metrics
        self.metrics.set_active_neurons(self.config.neurons.len() as u64);
        
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
                Ok(Box::new(MockClaude::new(layer)))
            }
            "api" => {
                info!("Creating Claude API client for layer {}", layer);
                let api_key = self.config.claude.api_key.clone()
                    .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                    .ok_or_else(|| Error::Config("Claude API key not found".to_string()))?;
                    
                Ok(Box::new(ClaudeAPIClient::new(
                    api_key,
                    self.config.claude.model.clone(),
                    layer,
                    self.config.claude.temperature,
                    self.config.claude.max_tokens,
                )))
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
}

/// Server status information
#[derive(Debug, serde::Serialize)]
pub struct ServerStatus {
    pub server_id: String,
    pub running: bool,
    pub neurons: std::collections::HashMap<String, 2hal9_core::NeuronHealth>,
    pub metrics: crate::metrics::MetricsSnapshot,
}