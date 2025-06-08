//! Performance optimization utilities

use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use parking_lot::RwLock;
use tokio::sync::{Semaphore, mpsc};
use tracing::debug;

/// Connection pool for reusing resources
pub struct ConnectionPool<T: Clone + Send + Sync + 'static> {
    pool: Vec<T>,
    available: Arc<Semaphore>,
    max_size: usize,
}

impl<T: Clone + Send + Sync + 'static> ConnectionPool<T> {
    pub fn new(initial: Vec<T>, max_size: usize) -> Self {
        let available = Arc::new(Semaphore::new(initial.len()));
        Self {
            pool: initial,
            available,
            max_size,
        }
    }
    
    pub async fn acquire(&self) -> Option<T> {
        let _permit = self.available.acquire().await.ok()?;
        self.pool.first().cloned()
    }
}

/// Response cache for frequently used queries
pub struct ResponseCache {
    cache: Arc<DashMap<String, CachedResponse>>,
    ttl: Duration,
    max_entries: usize,
}

#[derive(Clone)]
struct CachedResponse {
    response: String,
    timestamp: Instant,
    hit_count: u64,
}

impl ResponseCache {
    pub fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            ttl,
            max_entries,
        }
    }
    
    /// Get cached response if available and not expired
    pub fn get(&self, key: &str) -> Option<String> {
        self.cache.get_mut(key).and_then(|mut entry| {
            if entry.timestamp.elapsed() < self.ttl {
                entry.hit_count += 1;
                debug!("Cache hit for key: {} (hits: {})", key, entry.hit_count);
                Some(entry.response.clone())
            } else {
                // Expired entry
                drop(entry);
                self.cache.remove(key);
                None
            }
        })
    }
    
    /// Store response in cache
    pub fn put(&self, key: String, response: String) {
        // Check cache size limit
        if self.cache.len() >= self.max_entries {
            // Remove least recently used entries
            self.evict_lru();
        }
        
        self.cache.insert(key, CachedResponse {
            response,
            timestamp: Instant::now(),
            hit_count: 0,
        });
    }
    
    /// Evict least recently used entries
    fn evict_lru(&self) {
        let mut entries: Vec<_> = self.cache.iter()
            .map(|entry| (entry.key().clone(), entry.timestamp))
            .collect();
            
        entries.sort_by_key(|(_, timestamp)| *timestamp);
        
        // Remove oldest 10% of entries
        let remove_count = self.max_entries / 10;
        for (key, _) in entries.into_iter().take(remove_count) {
            self.cache.remove(&key);
        }
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut total_hits = 0u64;
        let mut total_entries = 0usize;
        
        for entry in self.cache.iter() {
            total_hits += entry.hit_count;
            total_entries += 1;
        }
        
        CacheStats {
            entries: total_entries,
            total_hits,
            size_bytes: total_entries * std::mem::size_of::<CachedResponse>(),
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub entries: usize,
    pub total_hits: u64,
    pub size_bytes: usize,
}

/// Batch processor for aggregating multiple signals
pub struct BatchProcessor<T> {
    batch_size: usize,
    batch_timeout: Duration,
    sender: mpsc::Sender<Vec<T>>,
}

impl<T: Send + 'static> BatchProcessor<T> {
    pub fn new(batch_size: usize, batch_timeout: Duration) -> (Self, mpsc::Receiver<Vec<T>>) {
        let (sender, receiver) = mpsc::channel(100);
        (
            Self {
                batch_size,
                batch_timeout,
                sender,
            },
            receiver
        )
    }
    
    /// Start batch processing task
    pub fn start<F>(
        mut receiver: mpsc::Receiver<Vec<T>>,
        processor: F
    ) 
    where
        F: Fn(Vec<T>) + Send + Sync + 'static,
        T: Send + 'static,
    {
        tokio::spawn(async move {
            while let Some(batch) = receiver.recv().await {
                let start = Instant::now();
                processor(batch);
                debug!("Batch processed in {:?}", start.elapsed());
            }
        });
    }
    
    /// Add item to batch
    pub async fn add(&self, item: T) -> Result<(), Box<dyn std::error::Error>> {
        self.sender.send(vec![item]).await?;
        Ok(())
    }
}

/// Parallel executor for concurrent processing
pub struct ParallelExecutor {
    max_concurrency: usize,
    semaphore: Arc<Semaphore>,
}

impl ParallelExecutor {
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            max_concurrency,
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
        }
    }
    
    /// Execute tasks in parallel with controlled concurrency
    pub async fn execute_all<F, T, R>(&self, tasks: Vec<T>, executor: F) -> Vec<R>
    where
        F: Fn(T) -> R + Send + Sync + Clone + 'static,
        T: Send + 'static,
        R: Send + 'static,
    {
        let mut handles = Vec::new();
        
        for task in tasks {
            let semaphore = self.semaphore.clone();
            let executor = executor.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                executor(task)
            });
            
            handles.push(handle);
        }
        
        // Collect results
        let mut results = Vec::new();
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }
        
        results
    }
}

/// Performance monitor for tracking key metrics
pub struct PerformanceMonitor {
    metrics: Arc<DashMap<String, PerformanceMetric>>,
}

#[derive(Clone)]
struct PerformanceMetric {
    count: u64,
    total_duration: Duration,
    min_duration: Duration,
    max_duration: Duration,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(DashMap::new()),
        }
    }
    
    /// Record a performance measurement
    pub fn record(&self, operation: &str, duration: Duration) {
        self.metrics.entry(operation.to_string())
            .and_modify(|metric| {
                metric.count += 1;
                metric.total_duration += duration;
                metric.min_duration = metric.min_duration.min(duration);
                metric.max_duration = metric.max_duration.max(duration);
            })
            .or_insert(PerformanceMetric {
                count: 1,
                total_duration: duration,
                min_duration: duration,
                max_duration: duration,
            });
    }
    
    /// Get performance report
    pub fn report(&self) -> PerformanceReport {
        let mut operations = Vec::new();
        
        for entry in self.metrics.iter() {
            let avg_duration = entry.total_duration / entry.count as u32;
            operations.push(OperationStats {
                name: entry.key().clone(),
                count: entry.count,
                avg_duration_ms: avg_duration.as_millis() as f64,
                min_duration_ms: entry.min_duration.as_millis() as f64,
                max_duration_ms: entry.max_duration.as_millis() as f64,
            });
        }
        
        operations.sort_by(|a, b| b.avg_duration_ms.partial_cmp(&a.avg_duration_ms).unwrap());
        
        PerformanceReport { operations }
    }
}

#[derive(Debug)]
pub struct PerformanceReport {
    pub operations: Vec<OperationStats>,
}

#[derive(Debug)]
pub struct OperationStats {
    pub name: String,
    pub count: u64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: f64,
    pub max_duration_ms: f64,
}

/// Optimized signal buffer for batch processing
pub struct SignalBuffer<T> {
    buffer: Arc<RwLock<Vec<T>>>,
    capacity: usize,
    flush_interval: Duration,
    last_flush: Arc<RwLock<Instant>>,
}

impl<T: Send + Sync + 'static> SignalBuffer<T> {
    pub fn new(capacity: usize, flush_interval: Duration) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(Vec::with_capacity(capacity))),
            capacity,
            flush_interval,
            last_flush: Arc::new(RwLock::new(Instant::now())),
        }
    }
    
    /// Add item to buffer, returns items to flush if buffer is full
    pub fn add(&self, item: T) -> Option<Vec<T>> {
        let mut buffer = self.buffer.write();
        buffer.push(item);
        
        // Check if we should flush
        let should_flush = buffer.len() >= self.capacity || 
            self.last_flush.read().elapsed() > self.flush_interval;
            
        if should_flush {
            let items = std::mem::replace(&mut *buffer, Vec::with_capacity(self.capacity));
            *self.last_flush.write() = Instant::now();
            Some(items)
        } else {
            None
        }
    }
    
    /// Force flush the buffer
    pub fn flush(&self) -> Vec<T> {
        let mut buffer = self.buffer.write();
        let items = std::mem::replace(&mut *buffer, Vec::with_capacity(self.capacity));
        *self.last_flush.write() = Instant::now();
        items
    }
    
    /// Get current buffer size
    pub fn len(&self) -> usize {
        self.buffer.read().len()
    }
}

/// Zero-copy string interner for reducing memory usage
pub struct StringInterner {
    strings: Arc<DashMap<String, Arc<str>>>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: Arc::new(DashMap::new()),
        }
    }
    
    /// Intern a string, returning a shared reference
    pub fn intern(&self, s: &str) -> Arc<str> {
        if let Some(entry) = self.strings.get(s) {
            entry.clone()
        } else {
            let arc_str: Arc<str> = s.into();
            self.strings.insert(s.to_string(), arc_str.clone());
            arc_str
        }
    }
    
    /// Get statistics about interned strings
    pub fn stats(&self) -> InternerStats {
        InternerStats {
            unique_strings: self.strings.len(),
            total_bytes: self.strings.iter()
                .map(|entry| entry.key().len())
                .sum(),
        }
    }
}

#[derive(Debug)]
pub struct InternerStats {
    pub unique_strings: usize,
    pub total_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_response_cache() {
        let cache = ResponseCache::new(Duration::from_secs(60), 100);
        
        // Test put and get
        cache.put("key1".to_string(), "response1".to_string());
        assert_eq!(cache.get("key1"), Some("response1".to_string()));
        
        // Test cache miss
        assert_eq!(cache.get("key2"), None);
        
        // Test stats
        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
    }
    
    #[tokio::test]
    async fn test_signal_buffer() {
        let buffer = SignalBuffer::new(3, Duration::from_secs(60));
        
        // Add items without triggering flush
        assert!(buffer.add(1).is_none());
        assert!(buffer.add(2).is_none());
        assert_eq!(buffer.len(), 2);
        
        // Add item that triggers flush
        let flushed = buffer.add(3);
        assert!(flushed.is_some());
        assert_eq!(flushed.unwrap(), vec![1, 2, 3]);
        assert_eq!(buffer.len(), 0);
    }
    
    #[test]
    fn test_string_interner() {
        let interner = StringInterner::new();
        
        let s1 = interner.intern("hello");
        let s2 = interner.intern("hello");
        
        // Should return the same Arc
        assert!(Arc::ptr_eq(&s1, &s2));
        
        let stats = interner.stats();
        assert_eq!(stats.unique_strings, 1);
    }
}