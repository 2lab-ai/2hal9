//! Runtime abstraction for async execution

use crate::Result;
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Async runtime abstraction
#[async_trait]
pub trait AsyncRuntime: Send + Sync + 'static {
    /// Spawn a new async task
    fn spawn<F>(&self, future: F) -> TaskHandle
    where
        F: Future<Output = ()> + Send + 'static;

    /// Spawn a task with priority
    fn spawn_with_priority<F>(&self, priority: TaskPriority, future: F) -> TaskHandle
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

    /// Create an interval timer
    fn interval(&self, period: Duration) -> Pin<Box<dyn futures::Stream<Item = Instant> + Send>>;

    /// Get runtime metrics
    fn metrics(&self) -> RuntimeMetrics;

    /// Create a cancellation token
    fn cancellation_token(&self) -> CancellationToken;

    /// Shutdown the runtime gracefully
    async fn shutdown(&self, timeout: Duration) -> Result<()>;
}

/// Handle to a spawned task
pub struct TaskHandle {
    id: uuid::Uuid,
    abort_handle: Option<tokio::task::AbortHandle>,
    priority: TaskPriority,
    spawned_at: Instant,
}

impl TaskHandle {
    /// Get task ID
    pub fn id(&self) -> uuid::Uuid {
        self.id
    }

    /// Get task priority
    pub fn priority(&self) -> TaskPriority {
        self.priority
    }

    /// Get elapsed time since spawn
    pub fn elapsed(&self) -> Duration {
        self.spawned_at.elapsed()
    }

    /// Abort the task
    pub fn abort(&self) {
        if let Some(handle) = &self.abort_handle {
            handle.abort();
        }
    }

    /// Check if task is finished
    pub fn is_finished(&self) -> bool {
        self.abort_handle.as_ref().is_none_or(|h| h.is_finished())
    }
}

/// Runtime performance metrics
#[derive(Debug, Clone)]
pub struct RuntimeMetrics {
    pub active_tasks: usize,
    pub total_spawned: u64,
    pub total_completed: u64,
    pub blocked_threads: usize,
    pub worker_threads: usize,
    pub task_queue_depth: usize,
    pub avg_task_duration_ms: f64,
}

/// Runtime statistics tracker
#[derive(Default)]
struct RuntimeStats {
    total_spawned: AtomicU64,
    total_completed: AtomicU64,
    task_durations: Arc<parking_lot::Mutex<Vec<Duration>>>,
}

/// Tokio-based runtime implementation
pub struct TokioRuntime {
    handle: tokio::runtime::Handle,
    stats: Arc<RuntimeStats>,
    shutdown_token: CancellationToken,
}

impl Default for TokioRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl TokioRuntime {
    pub fn new() -> Self {
        Self {
            handle: tokio::runtime::Handle::current(),
            stats: Arc::new(RuntimeStats::default()),
            shutdown_token: CancellationToken::new(),
        }
    }

    pub fn with_handle(handle: tokio::runtime::Handle) -> Self {
        Self {
            handle,
            stats: Arc::new(RuntimeStats::default()),
            shutdown_token: CancellationToken::new(),
        }
    }

    #[allow(dead_code)]
    fn track_task_completion(&self, duration: Duration) {
        self.stats.total_completed.fetch_add(1, Ordering::Relaxed);

        let mut durations = self.stats.task_durations.lock();
        durations.push(duration);

        // Keep only last 1000 task durations for averaging
        if durations.len() > 1000 {
            let drain_count = durations.len() - 1000;
            durations.drain(0..drain_count);
        }
    }
}

#[async_trait]
impl AsyncRuntime for TokioRuntime {
    fn spawn<F>(&self, future: F) -> TaskHandle
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.spawn_with_priority(TaskPriority::Normal, future)
    }

    fn spawn_with_priority<F>(&self, priority: TaskPriority, future: F) -> TaskHandle
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let id = uuid::Uuid::new_v4();
        let spawned_at = Instant::now();
        let stats = Arc::clone(&self.stats);

        stats.total_spawned.fetch_add(1, Ordering::Relaxed);

        // Wrap the future to track completion
        let tracked_future = async move {
            future.await;
            let duration = spawned_at.elapsed();
            stats.total_completed.fetch_add(1, Ordering::Relaxed);

            let mut durations = stats.task_durations.lock();
            durations.push(duration);
            if durations.len() > 1000 {
                let drain_count = durations.len() - 1000;
                durations.drain(0..drain_count);
            }
        };

        let handle = match priority {
            TaskPriority::Critical | TaskPriority::High => {
                // For high priority tasks, spawn directly without yielding
                self.handle.spawn(tracked_future)
            }
            _ => {
                // For normal/low priority, yield first to allow higher priority tasks
                self.handle.spawn(async move {
                    tokio::task::yield_now().await;
                    tracked_future.await;
                })
            }
        };

        TaskHandle {
            id,
            abort_handle: Some(handle.abort_handle()),
            priority,
            spawned_at,
        }
    }

    fn spawn_blocking<F, R>(&self, f: F) -> Pin<Box<dyn Future<Output = Result<R>> + Send>>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let handle = self.handle.clone();
        Box::pin(async move {
            handle
                .spawn_blocking(f)
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

    fn interval(&self, period: Duration) -> Pin<Box<dyn futures::Stream<Item = Instant> + Send>> {
        use futures::StreamExt;

        let stream = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(period))
            .map(|_| Instant::now());

        Box::pin(stream)
    }

    fn metrics(&self) -> RuntimeMetrics {
        let durations = self.stats.task_durations.lock();
        let avg_duration_ms = if durations.is_empty() {
            0.0
        } else {
            let sum: Duration = durations.iter().sum();
            sum.as_millis() as f64 / durations.len() as f64
        };

        RuntimeMetrics {
            active_tasks: (self.stats.total_spawned.load(Ordering::Relaxed)
                - self.stats.total_completed.load(Ordering::Relaxed))
                as usize,
            total_spawned: self.stats.total_spawned.load(Ordering::Relaxed),
            total_completed: self.stats.total_completed.load(Ordering::Relaxed),
            blocked_threads: 0, // Not available in current tokio version
            worker_threads: self.handle.metrics().num_workers(),
            task_queue_depth: 0, // Not available in current tokio version
            avg_task_duration_ms: avg_duration_ms,
        }
    }

    fn cancellation_token(&self) -> CancellationToken {
        self.shutdown_token.child_token()
    }

    async fn shutdown(&self, timeout: Duration) -> Result<()> {
        // Signal shutdown
        self.shutdown_token.cancel();

        // Wait for timeout or all tasks to complete
        tokio::time::timeout(timeout, async {
            while self.handle.metrics().num_alive_tasks() > 0 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .map_err(|_| crate::Error::Runtime("Shutdown timeout".to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tokio_runtime_spawn() {
        let runtime = TokioRuntime::new();
        let (tx, rx) = tokio::sync::oneshot::channel();

        let handle = runtime.spawn(async move {
            tx.send(42).unwrap();
        });

        let result = rx.await.unwrap();
        assert_eq!(result, 42);
        assert!(handle.is_finished());
    }

    #[tokio::test]
    async fn test_priority_spawning() {
        let runtime = TokioRuntime::new();
        let results = Arc::new(parking_lot::Mutex::new(Vec::new()));

        // Spawn tasks with different priorities
        for i in 0..4 {
            let priority = match i {
                0 => TaskPriority::Low,
                1 => TaskPriority::Normal,
                2 => TaskPriority::High,
                3 => TaskPriority::Critical,
                _ => unreachable!(),
            };

            let results = Arc::clone(&results);
            runtime.spawn_with_priority(priority, async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                results.lock().push(i);
            });
        }

        // Wait for all tasks
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Higher priority tasks should generally complete first
        let final_results = results.lock().clone();
        assert_eq!(final_results.len(), 4);
    }

    #[tokio::test]
    async fn test_cancellation_token() {
        let runtime = TokioRuntime::new();
        let token = runtime.cancellation_token();

        let task_token = token.clone();
        let handle = runtime.spawn(async move {
            task_token.cancelled().await;
        });

        // Cancel and verify
        token.cancel();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(handle.is_finished());
    }

    #[tokio::test]
    async fn test_metrics() {
        let runtime = TokioRuntime::new();

        // Spawn some tasks
        for _ in 0..5 {
            runtime.spawn(async {
                tokio::time::sleep(Duration::from_millis(10)).await;
            });
        }

        let metrics = runtime.metrics();
        assert!(metrics.total_spawned >= 5);
        assert!(metrics.active_tasks > 0);
    }
}
