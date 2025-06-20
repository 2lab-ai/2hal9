//! Signal batching for efficient processing
//!
//! Reduces lock contention by processing signals in batches

use super::NeuronId;
use crate::Signal;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Configuration for signal batching
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum batch size
    pub max_batch_size: usize,
    
    /// Maximum time to wait for batch to fill
    pub max_wait_time: Duration,
    
    /// Process immediately if queue exceeds this size
    pub urgent_threshold: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            max_wait_time: Duration::from_millis(10),
            urgent_threshold: 1000,
        }
    }
}

impl BatchConfig {
    
    /// Configuration for low latency
    pub fn low_latency() -> Self {
        Self {
            max_batch_size: 10,
            max_wait_time: Duration::from_millis(1),
            urgent_threshold: 50,
        }
    }
    
    /// Configuration for high throughput
    pub fn high_throughput() -> Self {
        Self {
            max_batch_size: 1000,
            max_wait_time: Duration::from_millis(50),
            urgent_threshold: 10000,
        }
    }
}

/// Batched signal for processing
pub struct BatchedSignal {
    pub from: NeuronId,
    pub to: NeuronId,
    pub signal: Signal,
    pub timestamp: Instant,
}

/// Signal batcher for efficient batch processing
pub struct SignalBatcher {
    /// Configuration
    config: BatchConfig,
    
    /// Pending signals
    queue: VecDeque<BatchedSignal>,
    
    /// Time when current batch started
    batch_start: Option<Instant>,
    
    /// Statistics
    stats: BatcherStats,
}

/// Batcher statistics
#[derive(Debug, Default)]
pub struct BatcherStats {
    pub total_signals: u64,
    pub total_batches: u64,
    pub avg_batch_size: f32,
    pub avg_wait_time_ms: f32,
}

impl SignalBatcher {
    /// Create a new signal batcher
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config,
            queue: VecDeque::new(),
            batch_start: None,
            stats: BatcherStats::default(),
        }
    }
    
    /// Add a signal to the batch
    pub fn add(&mut self, from: NeuronId, to: NeuronId, signal: Signal) -> bool {
        let now = Instant::now();
        
        // Start batch timer if needed
        if self.batch_start.is_none() {
            self.batch_start = Some(now);
        }
        
        self.queue.push_back(BatchedSignal {
            from,
            to,
            signal,
            timestamp: now,
        });
        
        // Check if we should process immediately
        self.should_process_now()
    }
    
    /// Check if batch should be processed
    pub fn should_process_now(&self) -> bool {
        // Urgent threshold exceeded
        if self.queue.len() >= self.config.urgent_threshold {
            return true;
        }
        
        // Batch size reached
        if self.queue.len() >= self.config.max_batch_size {
            return true;
        }
        
        // Time limit exceeded
        if let Some(start) = self.batch_start {
            if start.elapsed() >= self.config.max_wait_time {
                return true;
            }
        }
        
        false
    }
    
    /// Take a batch for processing
    pub fn take_batch(&mut self) -> Vec<BatchedSignal> {
        let batch_size = self.config.max_batch_size.min(self.queue.len());
        let mut batch = Vec::with_capacity(batch_size);
        
        for _ in 0..batch_size {
            if let Some(signal) = self.queue.pop_front() {
                batch.push(signal);
            }
        }
        
        // Update statistics
        if !batch.is_empty() {
            self.stats.total_signals += batch.len() as u64;
            self.stats.total_batches += 1;
            self.stats.avg_batch_size = self.stats.total_signals as f32 / self.stats.total_batches as f32;
            
            if let Some(start) = self.batch_start {
                let wait_time = start.elapsed().as_millis() as f32;
                self.stats.avg_wait_time_ms = 
                    (self.stats.avg_wait_time_ms * (self.stats.total_batches - 1) as f32 + wait_time) 
                    / self.stats.total_batches as f32;
            }
        }
        
        // Reset batch timer if queue is empty
        if self.queue.is_empty() {
            self.batch_start = None;
        } else {
            self.batch_start = Some(Instant::now());
        }
        
        batch
    }
    
    /// Get current queue size
    pub fn queue_size(&self) -> usize {
        self.queue.len()
    }
    
    /// Get statistics
    pub fn stats(&self) -> &BatcherStats {
        &self.stats
    }
    
    /// Clear the queue
    pub fn clear(&mut self) {
        self.queue.clear();
        self.batch_start = None;
    }
}

/// Multi-priority signal batcher
pub struct PrioritySignalBatcher {
    /// High priority batcher (low latency)
    high: SignalBatcher,
    
    /// Normal priority batcher (balanced)
    normal: SignalBatcher,
    
    /// Low priority batcher (high throughput)
    low: SignalBatcher,
}

impl Default for PrioritySignalBatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl PrioritySignalBatcher {
    /// Create a new priority batcher
    pub fn new() -> Self {
        Self {
            high: SignalBatcher::new(BatchConfig::low_latency()),
            normal: SignalBatcher::new(BatchConfig::default()),
            low: SignalBatcher::new(BatchConfig::high_throughput()),
        }
    }
    
    /// Add a signal with priority
    pub fn add(&mut self, from: NeuronId, to: NeuronId, signal: Signal, priority: SignalPriority) -> bool {
        match priority {
            SignalPriority::High => self.high.add(from, to, signal),
            SignalPriority::Normal => self.normal.add(from, to, signal),
            SignalPriority::Low => self.low.add(from, to, signal),
        }
    }
    
    /// Take all ready batches
    pub fn take_ready_batches(&mut self) -> Vec<(SignalPriority, Vec<BatchedSignal>)> {
        let mut batches = Vec::new();
        
        // Process high priority first
        if self.high.should_process_now() {
            let batch = self.high.take_batch();
            if !batch.is_empty() {
                batches.push((SignalPriority::High, batch));
            }
        }
        
        // Then normal
        if self.normal.should_process_now() {
            let batch = self.normal.take_batch();
            if !batch.is_empty() {
                batches.push((SignalPriority::Normal, batch));
            }
        }
        
        // Finally low
        if self.low.should_process_now() {
            let batch = self.low.take_batch();
            if !batch.is_empty() {
                batches.push((SignalPriority::Low, batch));
            }
        }
        
        batches
    }
}

/// Signal priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalPriority {
    High,
    Normal,
    Low,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_signal_batcher() {
        let config = BatchConfig {
            max_batch_size: 3,
            max_wait_time: Duration::from_millis(100),
            urgent_threshold: 10,
        };
        
        let mut batcher = SignalBatcher::new(config);
        
        // Add signals
        assert!(!batcher.add(NeuronId::new(1), NeuronId::new(2), Signal::default()));
        assert!(!batcher.add(NeuronId::new(2), NeuronId::new(3), Signal::default()));
        assert!(batcher.add(NeuronId::new(3), NeuronId::new(4), Signal::default())); // Batch full
        
        // Take batch
        let batch = batcher.take_batch();
        assert_eq!(batch.len(), 3);
        assert_eq!(batcher.queue_size(), 0);
        
        // Check stats
        assert_eq!(batcher.stats().total_signals, 3);
        assert_eq!(batcher.stats().total_batches, 1);
        assert_eq!(batcher.stats().avg_batch_size, 3.0);
    }
}