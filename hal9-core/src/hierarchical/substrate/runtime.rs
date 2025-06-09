//! Runtime abstraction for async execution

use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use crate::Result;

/// Async runtime abstraction
#[async_trait]
pub trait AsyncRuntime: Send + Sync + 'static {
    /// Spawn a new async task
    fn spawn<F>(&self, future: F) -> TaskHandle
    where
        F: Future<Output = ()> + Send + 'static;
    
    /// Spawn a blocking task
    fn spawn_blocking<F, R>(&self, f: F) -> Pin<Box<dyn Future<Output = Result<R>> + Send>>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static;
    
    /// Sleep for a duration
    async fn sleep(&self, duration: Duration);
    
    /// Create a timer that fires after duration
    fn timer(&self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send>>;
    
    /// Get runtime metrics
    fn metrics(&self) -> RuntimeMetrics;
}

/// Handle to a spawned task
pub struct TaskHandle {
    id: uuid::Uuid,
    abort_handle: Option<tokio::task::AbortHandle>,
}

impl TaskHandle {
    /// Abort the task
    pub fn abort(&self) {
        if let Some(handle) = &self.abort_handle {
            handle.abort();
        }
    }
    
    /// Check if task is finished
    pub fn is_finished(&self) -> bool {
        self.abort_handle.as_ref().map_or(true, |h| h.is_finished())
    }
}

/// Runtime performance metrics
#[derive(Debug, Clone)]
pub struct RuntimeMetrics {
    pub active_tasks: usize,
    pub total_spawned: u64,
    pub blocked_threads: usize,
    pub worker_threads: usize,
}

/// Tokio-based runtime implementation
pub struct TokioRuntime {
    handle: tokio::runtime::Handle,
}

impl TokioRuntime {
    pub fn new() -> Self {
        Self {
            handle: tokio::runtime::Handle::current(),
        }
    }
}

#[async_trait]
impl AsyncRuntime for TokioRuntime {
    fn spawn<F>(&self, future: F) -> TaskHandle
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let handle = self.handle.spawn(future);
        TaskHandle {
            id: uuid::Uuid::new_v4(),
            abort_handle: Some(handle.abort_handle()),
        }
    }
    
    fn spawn_blocking<F, R>(&self, f: F) -> Pin<Box<dyn Future<Output = Result<R>> + Send>>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let handle = self.handle.clone();
        Box::pin(async move {
            handle.spawn_blocking(f)
                .await
                .map_err(|e| crate::Error::Runtime(e.to_string()))
        })
    }
    
    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await
    }
    
    fn timer(&self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(tokio::time::sleep(duration))
    }
    
    fn metrics(&self) -> RuntimeMetrics {
        RuntimeMetrics {
            active_tasks: self.handle.metrics().num_alive_tasks(),
            total_spawned: 0, // Would need custom tracking
            blocked_threads: self.handle.metrics().num_blocking_threads(),
            worker_threads: self.handle.metrics().num_workers(),
        }
    }
}