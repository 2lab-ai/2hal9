//! Signal routing and processing

use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use 2hal9_core::{Error, Result, NeuronSignal, NeuronConfig};
use crate::neuron::{ManagedNeuron, NeuronRegistry};

/// Routing table for signal delivery
pub struct RoutingTable {
    routes: Arc<DashMap<String, Vec<String>>>,
}

impl RoutingTable {
    /// Create a new routing table
    pub fn new() -> Self {
        Self {
            routes: Arc::new(DashMap::new()),
        }
    }
    
    /// Build routing table from neuron configurations
    pub fn build_from_configs(&self, configs: &[NeuronConfig]) {
        for config in configs {
            // Add forward routes
            self.routes.insert(
                config.id.clone(),
                config.forward_connections.clone()
            );
        }
        
        info!("Built routing table with {} entries", self.routes.len());
    }
    
    /// Get forward connections for a neuron
    pub fn get_forwards(&self, neuron_id: &str) -> Vec<String> {
        self.routes.get(neuron_id)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }
    
    /// Check if a route exists
    pub fn has_route(&self, from: &str, to: &str) -> bool {
        self.routes.get(from)
            .map(|entry| entry.contains(&to.to_string()))
            .unwrap_or(false)
    }
}

/// Signal router for processing and distributing signals
pub struct SignalRouter {
    registry: Arc<NeuronRegistry>,
    routing_table: Arc<RoutingTable>,
    signal_tx: mpsc::Sender<NeuronSignal>,
    signal_rx: Option<mpsc::Receiver<NeuronSignal>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl SignalRouter {
    /// Create a new signal router
    pub fn new(registry: Arc<NeuronRegistry>, routing_table: Arc<RoutingTable>) -> Self {
        let (signal_tx, signal_rx) = mpsc::channel(1000);
        
        Self {
            registry,
            routing_table,
            signal_tx,
            signal_rx: Some(signal_rx),
            shutdown_tx: None,
        }
    }
    
    /// Start the signal processing loop
    pub async fn start(&mut self) -> Result<()> {
        let mut signal_rx = self.signal_rx.take()
            .ok_or_else(|| Error::InvalidState("Router already started".to_string()))?;
            
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);
        
        let registry = self.registry.clone();
        let routing_table = self.routing_table.clone();
        let signal_tx = self.signal_tx.clone();
        
        info!("Starting signal router");
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(signal) = signal_rx.recv() => {
                        debug!("Processing signal: {} -> {}", signal.from_neuron, signal.to_neuron);
                        
                        if let Err(e) = Self::process_signal(
                            &registry,
                            &routing_table,
                            &signal_tx,
                            signal
                        ).await {
                            error!("Failed to process signal: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Signal router shutting down");
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Process a single signal
    async fn process_signal(
        registry: &Arc<NeuronRegistry>,
        routing_table: &Arc<RoutingTable>,
        signal_tx: &mpsc::Sender<NeuronSignal>,
        signal: NeuronSignal,
    ) -> Result<()> {
        // Get target neuron
        let neuron = registry.get(&signal.to_neuron)
            .ok_or_else(|| Error::Routing(format!("Neuron {} not found", signal.to_neuron)))?;
            
        // Process signal
        match neuron.process_signal(&signal).await {
            Ok(response) => {
                debug!("Neuron {} processed signal successfully", neuron.id());
                
                // Parse response for new signals
                let new_signals = neuron.parse_response(&response, &signal);
                
                // Queue new signals
                for new_signal in new_signals {
                    if let Err(e) = signal_tx.send(new_signal).await {
                        error!("Failed to queue signal: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Neuron {} failed to process signal: {}", neuron.id(), e);
                
                // Generate error signal if appropriate
                if e.is_recoverable() && !signal.backward_connections.is_empty() {
                    let error_signal = NeuronSignal::backward(
                        &signal.to_neuron,
                        &signal.from_neuron,
                        &signal.layer_to,
                        &signal.layer_from,
                        2hal9_core::Gradient::new(e.to_string(), 1.0),
                    );
                    
                    if let Err(e) = signal_tx.send(error_signal).await {
                        error!("Failed to queue error signal: {}", e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Send a signal
    pub async fn send_signal(&self, signal: NeuronSignal) -> Result<()> {
        self.signal_tx.send(signal).await
            .map_err(|_| Error::Communication("Failed to send signal".to_string()))
    }
    
    /// Get the signal sender for external use
    pub fn get_sender(&self) -> mpsc::Sender<NeuronSignal> {
        self.signal_tx.clone()
    }
    
    /// Shutdown the router
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
        }
        Ok(())
    }
}

// Fix: Add backward_connections to signal parsing
impl NeuronSignal {
    fn backward_connections(&self) -> &[String] {
        // This would be looked up from config in real implementation
        &[]
    }
}