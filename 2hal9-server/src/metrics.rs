//! Metrics collection and monitoring

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Server metrics
#[derive(Default)]
pub struct Metrics {
    // Signal metrics
    pub signals_sent: AtomicU64,
    pub signals_processed: AtomicU64,
    pub signals_failed: AtomicU64,
    
    // Neuron metrics
    pub neurons_active: AtomicU64,
    pub neurons_failed: AtomicU64,
    
    // Performance metrics
    pub signal_latencies: Arc<DashMap<String, Vec<Duration>>>,
    
    // Start time
    start_time: Instant,
}

impl Metrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            signal_latencies: Arc::new(DashMap::new()),
            ..Default::default()
        }
    }
    
    /// Record a signal sent
    pub fn record_signal_sent(&self) {
        self.signals_sent.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a signal processed
    pub fn record_signal_processed(&self) {
        self.signals_processed.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a signal failure
    pub fn record_signal_failed(&self) {
        self.signals_failed.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record signal processing latency
    pub fn record_latency(&self, layer: &str, latency: Duration) {
        self.signal_latencies
            .entry(layer.to_string())
            .or_insert_with(Vec::new)
            .push(latency);
    }
    
    /// Update active neuron count
    pub fn set_active_neurons(&self, count: u64) {
        self.neurons_active.store(count, Ordering::Relaxed);
    }
    
    /// Record neuron failure
    pub fn record_neuron_failure(&self) {
        self.neurons_failed.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get current metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        let mut layer_latencies = std::collections::HashMap::new();
        
        for entry in self.signal_latencies.iter() {
            let layer = entry.key().clone();
            let latencies = entry.value();
            
            if !latencies.is_empty() {
                let sum: Duration = latencies.iter().sum();
                let avg = sum / latencies.len() as u32;
                
                layer_latencies.insert(layer, LatencyStats {
                    count: latencies.len() as u64,
                    avg_ms: avg.as_millis() as f64,
                    min_ms: latencies.iter().min().unwrap().as_millis() as f64,
                    max_ms: latencies.iter().max().unwrap().as_millis() as f64,
                });
            }
        }
        
        MetricsSnapshot {
            uptime_seconds: self.start_time.elapsed().as_secs(),
            signals_sent: self.signals_sent.load(Ordering::Relaxed),
            signals_processed: self.signals_processed.load(Ordering::Relaxed),
            signals_failed: self.signals_failed.load(Ordering::Relaxed),
            signals_per_second: self.calculate_rate(),
            neurons_active: self.neurons_active.load(Ordering::Relaxed),
            neurons_failed: self.neurons_failed.load(Ordering::Relaxed),
            layer_latencies,
        }
    }
    
    /// Calculate signals per second
    fn calculate_rate(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.signals_processed.load(Ordering::Relaxed) as f64 / elapsed
        } else {
            0.0
        }
    }
}

/// Metrics snapshot for reporting
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub uptime_seconds: u64,
    pub signals_sent: u64,
    pub signals_processed: u64,
    pub signals_failed: u64,
    pub signals_per_second: f64,
    pub neurons_active: u64,
    pub neurons_failed: u64,
    pub layer_latencies: std::collections::HashMap<String, LatencyStats>,
}

/// Latency statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct LatencyStats {
    pub count: u64,
    pub avg_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
}