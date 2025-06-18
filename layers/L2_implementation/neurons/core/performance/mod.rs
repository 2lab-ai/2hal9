//! Performance optimizations for large-scale neuron networks
//!
//! This module provides optimized data structures and algorithms
//! for scaling HAL9 to 10k+ neurons with real-time performance.

pub mod compact_id;
pub mod spatial_index;
pub mod lock_free;
pub mod signal_batcher;
pub mod memory_pool;

pub use compact_id::{NeuronId, NeuronIdGenerator};
pub use spatial_index::{SpatialIndex, NeuronPoint};
pub use lock_free::{LockFreeNeuronMap, LockFreeMetrics};
pub use signal_batcher::{SignalBatcher, BatchConfig};
pub use memory_pool::{NeuronPool, SignalPool};

/// Performance configuration for different network sizes
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Use spatial indexing for connection discovery
    pub use_spatial_index: bool,
    
    /// Use lock-free data structures
    pub use_lock_free: bool,
    
    /// Enable signal batching
    pub batch_signals: bool,
    
    /// Batch size for signal processing
    pub signal_batch_size: usize,
    
    /// Use memory pools
    pub use_memory_pools: bool,
    
    /// Pool sizes
    pub neuron_pool_size: usize,
    pub signal_pool_size: usize,
}

impl PerformanceConfig {
    /// Optimized for small networks (< 100 neurons)
    pub fn small() -> Self {
        Self {
            use_spatial_index: false,
            use_lock_free: false,
            batch_signals: false,
            signal_batch_size: 1,
            use_memory_pools: false,
            neuron_pool_size: 0,
            signal_pool_size: 0,
        }
    }
    
    /// Optimized for medium networks (100-1000 neurons)
    pub fn medium() -> Self {
        Self {
            use_spatial_index: true,
            use_lock_free: false,
            batch_signals: true,
            signal_batch_size: 10,
            use_memory_pools: true,
            neuron_pool_size: 1000,
            signal_pool_size: 10000,
        }
    }
    
    /// Optimized for large networks (1000-10k neurons)
    pub fn large() -> Self {
        Self {
            use_spatial_index: true,
            use_lock_free: true,
            batch_signals: true,
            signal_batch_size: 100,
            use_memory_pools: true,
            neuron_pool_size: 10000,
            signal_pool_size: 100000,
        }
    }
    
    /// Optimized for massive networks (10k+ neurons)
    pub fn massive() -> Self {
        Self {
            use_spatial_index: true,
            use_lock_free: true,
            batch_signals: true,
            signal_batch_size: 1000,
            use_memory_pools: true,
            neuron_pool_size: 100000,
            signal_pool_size: 1000000,
        }
    }
}

/// Performance metrics for monitoring
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    /// Time spent in connection discovery (microseconds)
    pub discovery_time_us: u64,
    
    /// Time spent processing signals (microseconds)
    pub signal_time_us: u64,
    
    /// Time spent in consciousness calculations (microseconds)
    pub consciousness_time_us: u64,
    
    /// Number of lock contentions
    pub lock_contentions: u64,
    
    /// Memory allocated (bytes)
    pub memory_bytes: usize,
    
    /// Number of active neurons
    pub active_neurons: usize,
    
    /// Signals processed per second
    pub signals_per_sec: f64,
}

impl PerformanceMetrics {
    /// Calculate efficiency score (0-1)
    pub fn efficiency(&self) -> f64 {
        if self.active_neurons == 0 {
            return 0.0;
        }
        
        // Ideal: < 1us per neuron per operation
        let time_per_neuron = (self.discovery_time_us + self.signal_time_us + self.consciousness_time_us) as f64 
            / self.active_neurons as f64;
        
        // Score decreases as time increases
        (1.0 / (1.0 + time_per_neuron / 1000.0)).min(1.0)
    }
    
    /// Check if performance is acceptable for real-time
    pub fn is_realtime(&self) -> bool {
        // 60 FPS = 16.67ms per frame
        let total_time_ms = (self.discovery_time_us + self.signal_time_us + self.consciousness_time_us) / 1000;
        total_time_ms < 16
    }
}