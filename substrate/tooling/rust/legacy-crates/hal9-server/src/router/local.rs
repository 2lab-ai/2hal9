//! Signal routing and processing

use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::neuron::NeuronRegistry;
use crate::performance::{ParallelExecutor, SignalBuffer};
use hal9_core::{Error, NeuronConfig, NeuronInterface, NeuronSignal, Result};

/// Routing table for signal delivery
pub struct RoutingTable {
    routes: Arc<DashMap<String, Vec<String>>>,
}

impl Default for RoutingTable {
    fn default() -> Self {
        Self::new()
    }
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
            self.routes
                .insert(config.id.clone(), config.forward_connections.clone());
        }

        info!("Built routing table with {} entries", self.routes.len());
    }

    /// Get forward connections for a neuron
    pub fn get_forwards(&self, neuron_id: &str) -> Vec<String> {
        self.routes
            .get(neuron_id)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }

    /// Check if a route exists
    pub fn has_route(&self, from: &str, to: &str) -> bool {
        self.routes
            .get(from)
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
    signal_buffer: Arc<SignalBuffer<NeuronSignal>>,
    parallel_executor: Arc<ParallelExecutor>,
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
            signal_buffer: Arc::new(SignalBuffer::new(
                10,                                   // buffer up to 10 signals
                std::time::Duration::from_millis(50), // flush every 50ms
            )),
            parallel_executor: Arc::new(ParallelExecutor::new(8)), // 8 parallel workers
        }
    }

    /// Start the signal processing loop
    pub async fn start(&mut self) -> Result<()> {
        let mut signal_rx = self
            .signal_rx
            .take()
            .ok_or_else(|| Error::InvalidState("Router already started".to_string()))?;

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let registry = self.registry.clone();
        let routing_table = self.routing_table.clone();
        let signal_tx = self.signal_tx.clone();
        let signal_buffer = self.signal_buffer.clone();

        info!("Starting signal router");

        tokio::spawn(async move {
            // Create a timer for periodic buffer flushing
            let mut flush_interval = tokio::time::interval(std::time::Duration::from_millis(50));
            flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    Some(signal) = signal_rx.recv() => {
                        debug!("Processing signal: {} -> {}", signal.from_neuron, signal.to_neuron);

                        // Buffer signals for batch processing
                        let signal_buffer = signal_buffer.clone();
                        let registry_clone = registry.clone();
                        let routing_table_clone = routing_table.clone();
                        let signal_tx_clone = signal_tx.clone();

                        if let Some(batch) = signal_buffer.add(signal) {
                            // Process batch in parallel
                            tokio::spawn(async move {
                                Self::process_signal_batch(
                                    &registry_clone,
                                    &routing_table_clone,
                                    &signal_tx_clone,
                                    batch
                                ).await;
                            });
                        }
                    }
                    _ = flush_interval.tick() => {
                        // Periodic flush of any buffered signals
                        let buffered = signal_buffer.flush();
                        if !buffered.is_empty() {
                            debug!("Flushing {} buffered signals", buffered.len());
                            let registry_clone = registry.clone();
                            let routing_table_clone = routing_table.clone();
                            let signal_tx_clone = signal_tx.clone();

                            tokio::spawn(async move {
                                Self::process_signal_batch(
                                    &registry_clone,
                                    &routing_table_clone,
                                    &signal_tx_clone,
                                    buffered
                                ).await;
                            });
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        // Flush any remaining signals
                        let remaining = signal_buffer.flush();
                        if !remaining.is_empty() {
                            Self::process_signal_batch(
                                &registry,
                                &routing_table,
                                &signal_tx,
                                remaining
                            ).await;
                        }
                        info!("Signal router shutting down");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Process a batch of signals in parallel
    async fn process_signal_batch(
        registry: &Arc<NeuronRegistry>,
        routing_table: &Arc<RoutingTable>,
        signal_tx: &mpsc::Sender<NeuronSignal>,
        signals: Vec<NeuronSignal>,
    ) {
        let start = std::time::Instant::now();
        debug!("Processing batch of {} signals", signals.len());

        // Process signals in parallel
        let tasks: Vec<_> = signals
            .into_iter()
            .map(|signal| {
                let registry = registry.clone();
                let routing_table = routing_table.clone();
                let signal_tx = signal_tx.clone();

                tokio::spawn(async move {
                    if let Err(e) =
                        Self::process_signal(&registry, &routing_table, &signal_tx, signal).await
                    {
                        error!("Failed to process signal: {}", e);
                    }
                })
            })
            .collect();

        // Wait for all tasks to complete
        for task in tasks {
            let _ = task.await;
        }

        debug!("Batch processed in {:?}", start.elapsed());
    }

    /// Process a single signal
    async fn process_signal(
        registry: &Arc<NeuronRegistry>,
        _routing_table: &Arc<RoutingTable>,
        signal_tx: &mpsc::Sender<NeuronSignal>,
        signal: NeuronSignal,
    ) -> Result<()> {
        // Get target neuron
        let neuron = registry
            .get(&signal.to_neuron)
            .ok_or_else(|| Error::Routing(format!("Neuron {} not found", signal.to_neuron)))?;

        // Process signal
        match neuron.process_signal(&signal).await {
            Ok(response) => {
                debug!("Neuron {} processed signal successfully", neuron.id());

                // Parse response for new signals
                let new_signals = neuron.parse_response(&response, &signal);

                // Queue new signals in parallel if multiple
                if new_signals.len() > 1 {
                    let signal_tx = signal_tx.clone();
                    tokio::spawn(async move {
                        for new_signal in new_signals {
                            if let Err(e) = signal_tx.send(new_signal).await {
                                error!("Failed to queue signal: {}", e);
                            }
                        }
                    });
                } else {
                    // Queue single signal directly
                    for new_signal in new_signals {
                        if let Err(e) = signal_tx.send(new_signal).await {
                            error!("Failed to queue signal: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Neuron failed to process signal: {}", e);

                // Generate error signal if appropriate
                if e.is_recoverable() {
                    let error_signal = NeuronSignal::backward(
                        &signal.to_neuron,
                        &signal.from_neuron,
                        &signal.layer_to,
                        &signal.layer_from,
                        hal9_core::Gradient::new(e.to_string(), 1.0),
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
        self.signal_tx
            .send(signal)
            .await
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
