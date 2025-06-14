//! Metrics collection and monitoring

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Server metrics
pub struct Metrics {
    // Signal metrics
    pub signals_sent: AtomicU64,
    pub signals_processed: AtomicU64,
    pub signals_failed: AtomicU64,
    
    // Neuron metrics
    pub neurons_active: AtomicU64,
    pub neurons_failed: AtomicU64,
    pub neurons_processing: AtomicU64,
    
    // Performance metrics
    pub signal_latencies: Arc<DashMap<String, Vec<Duration>>>,
    pub processing_times: Arc<DashMap<String, Vec<Duration>>>,
    
    // Token usage metrics (for Claude API)
    pub tokens_prompt: AtomicU64,
    pub tokens_completion: AtomicU64,
    pub tokens_total: AtomicU64,
    
    // Cost metrics
    pub cost_hourly: Arc<parking_lot::RwLock<f64>>,
    pub cost_daily: Arc<parking_lot::RwLock<f64>>,
    pub cost_total: Arc<parking_lot::RwLock<f64>>,
    
    // Error tracking
    pub errors_by_type: Arc<DashMap<String, AtomicU64>>,
    
    // Memory usage
    pub memory_usage_bytes: AtomicU64,
    
    // Start time
    start_time: Instant,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            signals_sent: AtomicU64::new(0),
            signals_processed: AtomicU64::new(0),
            signals_failed: AtomicU64::new(0),
            neurons_active: AtomicU64::new(0),
            neurons_failed: AtomicU64::new(0),
            neurons_processing: AtomicU64::new(0),
            signal_latencies: Arc::new(DashMap::new()),
            processing_times: Arc::new(DashMap::new()),
            tokens_prompt: AtomicU64::new(0),
            tokens_completion: AtomicU64::new(0),
            tokens_total: AtomicU64::new(0),
            cost_hourly: Arc::new(parking_lot::RwLock::new(0.0)),
            cost_daily: Arc::new(parking_lot::RwLock::new(0.0)),
            cost_total: Arc::new(parking_lot::RwLock::new(0.0)),
            errors_by_type: Arc::new(DashMap::new()),
            memory_usage_bytes: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }
}

impl Metrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self::default()
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
    
    /// Record a neuron starting processing
    pub fn record_neuron_processing_start(&self) {
        self.neurons_processing.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a neuron finishing processing
    pub fn record_neuron_processing_end(&self) {
        self.neurons_processing.fetch_sub(1, Ordering::Relaxed);
    }
    
    /// Record processing time for a neuron
    pub fn record_processing_time(&self, neuron_id: &str, duration: Duration) {
        self.processing_times
            .entry(neuron_id.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }
    
    /// Record token usage
    pub fn record_token_usage(&self, prompt_tokens: u32, completion_tokens: u32) {
        self.tokens_prompt.fetch_add(prompt_tokens as u64, Ordering::Relaxed);
        self.tokens_completion.fetch_add(completion_tokens as u64, Ordering::Relaxed);
        self.tokens_total.fetch_add((prompt_tokens + completion_tokens) as u64, Ordering::Relaxed);
    }
    
    /// Update cost metrics
    pub fn update_cost_metrics(&self, hourly: f64, daily: f64, total: f64) {
        *self.cost_hourly.write() = hourly;
        *self.cost_daily.write() = daily;
        *self.cost_total.write() = total;
    }
    
    /// Record an error
    pub fn record_error(&self, error_type: &str) {
        self.errors_by_type
            .entry(error_type.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }
    
    /// Update memory usage
    pub fn update_memory_usage(&self) {
        // Simple memory estimation - in production, use proper memory profiling
        if let Ok(mem_info) = sys_info::mem_info() {
            let used_kb = mem_info.total - mem_info.free;
            self.memory_usage_bytes.store(used_kb as u64 * 1024, Ordering::Relaxed);
        }
    }
    
    /// Get current metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        let mut layer_latencies = std::collections::HashMap::new();
        let mut processing_times = std::collections::HashMap::new();
        
        // Collect signal latencies
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
        
        // Collect processing times
        for entry in self.processing_times.iter() {
            let neuron = entry.key().clone();
            let times = entry.value();
            
            if !times.is_empty() {
                let sum: Duration = times.iter().sum();
                let avg = sum / times.len() as u32;
                
                processing_times.insert(neuron, LatencyStats {
                    count: times.len() as u64,
                    avg_ms: avg.as_millis() as f64,
                    min_ms: times.iter().min().unwrap().as_millis() as f64,
                    max_ms: times.iter().max().unwrap().as_millis() as f64,
                });
            }
        }
        
        // Collect error counts
        let mut errors_by_type = std::collections::HashMap::new();
        for entry in self.errors_by_type.iter() {
            errors_by_type.insert(
                entry.key().clone(),
                entry.value().load(Ordering::Relaxed)
            );
        }
        
        MetricsSnapshot {
            uptime_seconds: self.start_time.elapsed().as_secs(),
            signals_sent: self.signals_sent.load(Ordering::Relaxed),
            signals_processed: self.signals_processed.load(Ordering::Relaxed),
            signals_failed: self.signals_failed.load(Ordering::Relaxed),
            signals_per_second: self.calculate_rate(),
            neurons_active: self.neurons_active.load(Ordering::Relaxed),
            neurons_failed: self.neurons_failed.load(Ordering::Relaxed),
            neurons_processing: self.neurons_processing.load(Ordering::Relaxed),
            layer_latencies,
            processing_times,
            tokens_prompt: self.tokens_prompt.load(Ordering::Relaxed),
            tokens_completion: self.tokens_completion.load(Ordering::Relaxed),
            tokens_total: self.tokens_total.load(Ordering::Relaxed),
            cost_hourly: *self.cost_hourly.read(),
            cost_daily: *self.cost_daily.read(),
            cost_total: *self.cost_total.read(),
            errors_by_type,
            memory_usage_mb: self.memory_usage_bytes.load(Ordering::Relaxed) as f64 / (1024.0 * 1024.0),
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
    pub neurons_processing: u64,
    pub layer_latencies: std::collections::HashMap<String, LatencyStats>,
    pub processing_times: std::collections::HashMap<String, LatencyStats>,
    pub tokens_prompt: u64,
    pub tokens_completion: u64,
    pub tokens_total: u64,
    pub cost_hourly: f64,
    pub cost_daily: f64,
    pub cost_total: f64,
    pub errors_by_type: std::collections::HashMap<String, u64>,
    pub memory_usage_mb: f64,
}

/// Latency statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct LatencyStats {
    pub count: u64,
    pub avg_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
}