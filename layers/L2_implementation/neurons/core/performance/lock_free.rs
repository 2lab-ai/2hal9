//! Lock-free data structures for concurrent access
//!
//! Reduces lock contention in hot paths

use super::NeuronId;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::collections::HashMap;
use std::sync::RwLock;

/// Lock-free neuron map using sharded RwLock HashMap
pub struct LockFreeNeuronMap<T> {
    shards: Vec<RwLock<HashMap<NeuronId, Arc<T>>>>,
    shard_count: usize,
    size: AtomicUsize,
}

impl<T> LockFreeNeuronMap<T> {
    /// Create a new lock-free map
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }
    
    /// Create with capacity hint
    pub fn with_capacity(capacity: usize) -> Self {
        let shard_count = 16; // Good default for concurrency
        let mut shards = Vec::with_capacity(shard_count);
        let capacity_per_shard = capacity / shard_count;
        
        for _ in 0..shard_count {
            shards.push(RwLock::new(HashMap::with_capacity(capacity_per_shard)));
        }
        
        Self {
            shards,
            shard_count,
            size: AtomicUsize::new(0),
        }
    }
    
    /// Get shard index for a neuron ID
    fn shard_index(&self, id: &NeuronId) -> usize {
        (id.value() as usize) % self.shard_count
    }
    
    /// Insert a neuron
    pub fn insert(&self, id: NeuronId, neuron: Arc<T>) -> Option<Arc<T>> {
        let shard_idx = self.shard_index(&id);
        let mut shard = self.shards[shard_idx].write().unwrap();
        let prev = shard.insert(id, neuron);
        if prev.is_none() {
            self.size.fetch_add(1, Ordering::Relaxed);
        }
        prev
    }
    
    /// Get a neuron
    pub fn get(&self, id: &NeuronId) -> Option<Arc<T>> {
        let shard_idx = self.shard_index(id);
        let shard = self.shards[shard_idx].read().unwrap();
        shard.get(id).cloned()
    }
    
    /// Remove a neuron
    pub fn remove(&self, id: &NeuronId) -> Option<Arc<T>> {
        let shard_idx = self.shard_index(id);
        let mut shard = self.shards[shard_idx].write().unwrap();
        let removed = shard.remove(id);
        if removed.is_some() {
            self.size.fetch_sub(1, Ordering::Relaxed);
        }
        removed
    }
    
    /// Check if contains a neuron
    pub fn contains(&self, id: &NeuronId) -> bool {
        let shard_idx = self.shard_index(id);
        let shard = self.shards[shard_idx].read().unwrap();
        shard.contains_key(id)
    }
    
    /// Get the number of neurons
    pub fn len(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }
    
    /// Clear all neurons
    pub fn clear(&self) {
        for shard in &self.shards {
            shard.write().unwrap().clear();
        }
        self.size.store(0, Ordering::Relaxed);
    }
    
    /// Iterate over all neurons
    pub fn iter(&self) -> Vec<(NeuronId, Arc<T>)> {
        let mut result = Vec::new();
        for shard in &self.shards {
            let shard = shard.read().unwrap();
            for (k, v) in shard.iter() {
                result.push((*k, v.clone()));
            }
        }
        result
    }
}

/// Lock-free metrics using atomic counters
#[derive(Default)]
pub struct LockFreeMetrics {
    /// Total neurons processed
    pub neurons_processed: AtomicU64,
    
    /// Total signals sent
    pub signals_sent: AtomicU64,
    
    /// Total connections made
    pub connections_made: AtomicU64,
    
    /// Discovery cycles completed
    pub discovery_cycles: AtomicU64,
    
    /// Total processing time in microseconds
    pub processing_time_us: AtomicU64,
}

impl LockFreeMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Increment neurons processed
    pub fn inc_neurons(&self, count: u64) {
        self.neurons_processed.fetch_add(count, Ordering::Relaxed);
    }
    
    /// Increment signals sent
    pub fn inc_signals(&self, count: u64) {
        self.signals_sent.fetch_add(count, Ordering::Relaxed);
    }
    
    /// Increment connections made
    pub fn inc_connections(&self, count: u64) {
        self.connections_made.fetch_add(count, Ordering::Relaxed);
    }
    
    /// Increment discovery cycles
    pub fn inc_discovery_cycles(&self) {
        self.discovery_cycles.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Add processing time
    pub fn add_processing_time(&self, microseconds: u64) {
        self.processing_time_us.fetch_add(microseconds, Ordering::Relaxed);
    }
    
    /// Get snapshot of metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            neurons_processed: self.neurons_processed.load(Ordering::Relaxed),
            signals_sent: self.signals_sent.load(Ordering::Relaxed),
            connections_made: self.connections_made.load(Ordering::Relaxed),
            discovery_cycles: self.discovery_cycles.load(Ordering::Relaxed),
            processing_time_us: self.processing_time_us.load(Ordering::Relaxed),
        }
    }
    
    /// Reset all metrics
    pub fn reset(&self) {
        self.neurons_processed.store(0, Ordering::Relaxed);
        self.signals_sent.store(0, Ordering::Relaxed);
        self.connections_made.store(0, Ordering::Relaxed);
        self.discovery_cycles.store(0, Ordering::Relaxed);
        self.processing_time_us.store(0, Ordering::Relaxed);
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub neurons_processed: u64,
    pub signals_sent: u64,
    pub connections_made: u64,
    pub discovery_cycles: u64,
    pub processing_time_us: u64,
}

impl MetricsSnapshot {
    /// Calculate average processing time per neuron
    pub fn avg_time_per_neuron(&self) -> f64 {
        if self.neurons_processed == 0 {
            0.0
        } else {
            self.processing_time_us as f64 / self.neurons_processed as f64
        }
    }
    
    /// Calculate signals per neuron
    pub fn signals_per_neuron(&self) -> f64 {
        if self.neurons_processed == 0 {
            0.0
        } else {
            self.signals_sent as f64 / self.neurons_processed as f64
        }
    }
}

/// Thread-safe priority queue using BinaryHeap with RwLock
pub struct LockFreePriorityQueue<T> {
    /// Priority queue sorted by priority (higher = more important)
    queue: RwLock<std::collections::BinaryHeap<PriorityItem<T>>>,
    
    /// Counter for unique IDs
    counter: AtomicU64,
}

#[derive(Debug)]
struct PriorityItem<T> {
    priority: u64,
    id: u64,
    value: T,
}

impl<T> PartialEq for PriorityItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.id == other.id
    }
}

impl<T> Eq for PriorityItem<T> {}

impl<T> PartialOrd for PriorityItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for PriorityItem<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then by ID
        self.priority.cmp(&other.priority)
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl<T> LockFreePriorityQueue<T> {
    /// Create a new priority queue
    pub fn new() -> Self {
        Self {
            queue: RwLock::new(std::collections::BinaryHeap::new()),
            counter: AtomicU64::new(0),
        }
    }
    
    /// Insert with priority (higher = more important)
    pub fn insert(&self, priority: u64, value: T) {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        let item = PriorityItem { priority, id, value };
        self.queue.write().unwrap().push(item);
    }
    
    /// Pop the highest priority item
    pub fn pop(&self) -> Option<T> {
        self.queue.write().unwrap().pop().map(|item| item.value)
    }
    
    /// Peek at the highest priority item
    pub fn peek(&self) -> Option<u64> {
        self.queue.read().unwrap().peek().map(|item| item.priority)
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.queue.read().unwrap().is_empty()
    }
    
    /// Get approximate size
    pub fn len(&self) -> usize {
        self.queue.read().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lock_free_neuron_map() {
        let map = LockFreeNeuronMap::new();
        
        let id1 = NeuronId::new(1);
        let id2 = NeuronId::new(2);
        
        map.insert(id1, Arc::new("neuron1"));
        map.insert(id2, Arc::new("neuron2"));
        
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&id1).as_deref(), Some(&"neuron1"));
        
        map.remove(&id1);
        assert_eq!(map.len(), 1);
        assert!(!map.contains(&id1));
    }
    
    #[test]
    fn test_lock_free_metrics() {
        let metrics = LockFreeMetrics::new();
        
        metrics.inc_neurons(10);
        metrics.inc_signals(50);
        metrics.add_processing_time(1000);
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.neurons_processed, 10);
        assert_eq!(snapshot.signals_sent, 50);
        assert_eq!(snapshot.avg_time_per_neuron(), 100.0);
        assert_eq!(snapshot.signals_per_neuron(), 5.0);
    }
    
    #[test]
    fn test_priority_queue() {
        let queue = LockFreePriorityQueue::new();
        
        queue.insert(5, "medium");
        queue.insert(10, "high");
        queue.insert(1, "low");
        
        assert_eq!(queue.pop(), Some("high"));
        assert_eq!(queue.pop(), Some("medium"));
        assert_eq!(queue.pop(), Some("low"));
        assert!(queue.pop().is_none());
    }
}