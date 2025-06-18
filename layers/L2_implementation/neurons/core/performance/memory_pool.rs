//! Memory pools for reducing allocations
//!
//! Pre-allocates commonly used objects to avoid allocation overhead

use super::NeuronId;
use crate::Signal;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Pool for reusing neuron objects
pub struct NeuronPool<T> {
    pool: Arc<Mutex<VecDeque<T>>>,
    capacity: usize,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
}

impl<T> NeuronPool<T> {
    /// Create a new pool with given capacity
    pub fn new<F>(capacity: usize, factory: F) -> Self 
    where
        F: Fn() -> T + Send + Sync + 'static
    {
        let pool = Arc::new(Mutex::new(VecDeque::with_capacity(capacity)));
        
        // Pre-allocate some objects
        let pre_alloc = capacity / 4;
        for _ in 0..pre_alloc {
            if let Ok(mut p) = pool.lock() {
                p.push_back(factory());
            }
        }
        
        Self {
            pool,
            capacity,
            factory: Arc::new(factory),
        }
    }
    
    /// Get an object from the pool
    pub fn get(&self) -> PooledObject<T> {
        let obj = if let Ok(mut pool) = self.pool.lock() {
            pool.pop_front()
        } else {
            None
        };
        
        let obj = obj.unwrap_or_else(|| (self.factory)());
        
        PooledObject {
            obj: Some(obj),
            pool: self.pool.clone(),
            capacity: self.capacity,
        }
    }
    
    /// Get current pool size
    pub fn size(&self) -> usize {
        self.pool.lock().map(|p| p.len()).unwrap_or(0)
    }
    
    /// Clear the pool
    pub fn clear(&self) {
        if let Ok(mut pool) = self.pool.lock() {
            pool.clear();
        }
    }
}

/// RAII wrapper for pooled objects
pub struct PooledObject<T> {
    obj: Option<T>,
    pool: Arc<Mutex<VecDeque<T>>>,
    capacity: usize,
}

impl<T> PooledObject<T> {
    /// Get reference to the object
    pub fn as_ref(&self) -> &T {
        self.obj.as_ref().expect("PooledObject already returned")
    }
    
    /// Get mutable reference to the object
    pub fn as_mut(&mut self) -> &mut T {
        self.obj.as_mut().expect("PooledObject already returned")
    }
    
    /// Take ownership of the object (won't be returned to pool)
    pub fn take(mut self) -> T {
        self.obj.take().expect("PooledObject already returned")
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.obj.take() {
            if let Ok(mut pool) = self.pool.lock() {
                if pool.len() < self.capacity {
                    pool.push_back(obj);
                }
                // Otherwise, let it be dropped
            }
        }
    }
}

impl<T> std::ops::Deref for PooledObject<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

/// Pool for Signal objects
pub struct SignalPool {
    pool: NeuronPool<Signal>,
}

impl SignalPool {
    /// Create a new signal pool
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: NeuronPool::new(capacity, Signal::default),
        }
    }
    
    /// Get a signal from the pool
    pub fn get(&self) -> PooledObject<Signal> {
        self.pool.get()
    }
    
    /// Create a signal with specific values
    pub fn create(&self, from: NeuronId, to: NeuronId, strength: f32) -> PooledObject<Signal> {
        let mut signal = self.get();
        // Set signal properties here
        // signal.from = from;
        // signal.to = to;
        // signal.strength = strength;
        signal
    }
}

/// Arena allocator for batch operations
pub struct Arena {
    chunks: Vec<Vec<u8>>,
    current: usize,
    position: usize,
    chunk_size: usize,
}

impl Arena {
    /// Create a new arena with given chunk size
    pub fn new(chunk_size: usize) -> Self {
        let mut chunks = Vec::new();
        chunks.push(vec![0; chunk_size]);
        
        Self {
            chunks,
            current: 0,
            position: 0,
            chunk_size,
        }
    }
    
    /// Allocate memory for an object
    pub fn alloc<T>(&mut self) -> &mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        
        // Align position
        let aligned_pos = (self.position + align - 1) & !(align - 1);
        
        // Check if we need a new chunk
        if aligned_pos + size > self.chunk_size {
            self.chunks.push(vec![0; self.chunk_size]);
            self.current += 1;
            self.position = 0;
        }
        
        let ptr = &mut self.chunks[self.current][self.position] as *mut u8 as *mut T;
        self.position += size;
        
        unsafe { &mut *ptr }
    }
    
    /// Reset the arena (invalidates all allocations)
    pub fn reset(&mut self) {
        self.current = 0;
        self.position = 0;
        // Keep the first chunk, clear others
        self.chunks.truncate(1);
    }
}

/// Memory statistics
#[derive(Debug, Default)]
pub struct MemoryStats {
    pub allocations: u64,
    pub deallocations: u64,
    pub bytes_allocated: u64,
    pub bytes_deallocated: u64,
    pub pool_hits: u64,
    pub pool_misses: u64,
}

impl MemoryStats {
    /// Calculate pool hit rate
    pub fn pool_hit_rate(&self) -> f64 {
        let total = self.pool_hits + self.pool_misses;
        if total == 0 {
            0.0
        } else {
            self.pool_hits as f64 / total as f64
        }
    }
    
    /// Calculate net memory usage
    pub fn net_bytes(&self) -> i64 {
        self.bytes_allocated as i64 - self.bytes_deallocated as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Default)]
    struct TestObject {
        value: i32,
    }
    
    #[test]
    fn test_neuron_pool() {
        let pool = NeuronPool::new(10, TestObject::default);
        
        // Get object from pool
        let mut obj1 = pool.get();
        obj1.value = 42;
        
        // Get another
        let obj2 = pool.get();
        assert_eq!(obj2.value, 0); // New object
        
        drop(obj1);
        drop(obj2);
        
        // Should have 2 objects in pool now
        assert!(pool.size() >= 2);
        
        // Get again - should reuse
        let obj3 = pool.get();
        // Note: value might be 42 if it reused obj1
    }
    
    #[test]
    fn test_arena() {
        let mut arena = Arena::new(1024);
        
        let n1: &mut i32 = arena.alloc();
        *n1 = 42;
        
        let n2: &mut i32 = arena.alloc();
        *n2 = 100;
        
        assert_eq!(*n1, 42);
        assert_eq!(*n2, 100);
        
        arena.reset();
        // n1 and n2 are now invalid!
    }
}