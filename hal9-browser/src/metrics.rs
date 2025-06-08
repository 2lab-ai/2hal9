//! Browser automation metrics collection

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use serde::Serialize;

use crate::controller::{BrowserAction, ActionResult};

/// Browser automation metrics
pub struct BrowserMetrics {
    /// Total actions executed
    total_actions: AtomicU64,
    
    /// Successful actions
    successful_actions: AtomicU64,
    
    /// Failed actions
    failed_actions: AtomicU64,
    
    /// Action counts by type
    action_counts: DashMap<String, u64>,
    
    /// Average duration by action type (microseconds)
    action_durations: DashMap<String, DurationStats>,
    
    /// Current active contexts
    active_contexts: AtomicUsize,
    
    /// Total contexts created
    total_contexts_created: AtomicU64,
    
    /// Navigation statistics
    navigation_stats: NavigationStats,
    
    /// Error counts by type
    error_counts: DashMap<String, u64>,
    
    /// Start time for uptime calculation
    start_time: Instant,
}

impl BrowserMetrics {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            total_actions: AtomicU64::new(0),
            successful_actions: AtomicU64::new(0),
            failed_actions: AtomicU64::new(0),
            action_counts: DashMap::new(),
            action_durations: DashMap::new(),
            active_contexts: AtomicUsize::new(0),
            total_contexts_created: AtomicU64::new(0),
            navigation_stats: NavigationStats::new(),
            error_counts: DashMap::new(),
            start_time: Instant::now(),
        }
    }
    
    /// Record action start
    pub fn record_action_start(&self, action: &BrowserAction) {
        self.total_actions.fetch_add(1, Ordering::Relaxed);
        
        let action_type = self.action_type_name(action);
        self.action_counts
            .entry(action_type)
            .and_modify(|c| *c += 1)
            .or_insert(1);
    }
    
    /// Record action completion
    pub fn record_action_complete(
        &self,
        action: &BrowserAction,
        result: &Result<ActionResult, crate::BrowserError>,
        duration: Duration,
    ) {
        match result {
            Ok(_) => {
                self.successful_actions.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                self.failed_actions.fetch_add(1, Ordering::Relaxed);
                self.record_error(e);
            }
        }
        
        // Record duration
        let action_type = self.action_type_name(action);
        let duration_micros = duration.as_micros() as u64;
        
        self.action_durations
            .entry(action_type)
            .and_modify(|stats| stats.add_sample(duration_micros))
            .or_insert_with(|| {
                let mut stats = DurationStats::new();
                stats.add_sample(duration_micros);
                stats
            });
        
        // Update navigation stats if applicable
        if let BrowserAction::Navigate { .. } = action {
            if result.is_ok() {
                self.navigation_stats.record_success();
            } else {
                self.navigation_stats.record_failure();
            }
        }
    }
    
    /// Record context creation
    pub fn record_context_created(&self) {
        self.total_contexts_created.fetch_add(1, Ordering::Relaxed);
        self.active_contexts.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record context destruction
    pub fn record_context_destroyed(&self) {
        self.active_contexts.fetch_sub(1, Ordering::Relaxed);
    }
    
    /// Record error
    fn record_error(&self, error: &crate::BrowserError) {
        let error_type = format!("{:?}", error).split('(').next().unwrap_or("Unknown").to_string();
        self.error_counts
            .entry(error_type)
            .and_modify(|c| *c += 1)
            .or_insert(1);
    }
    
    /// Get action type name
    fn action_type_name(&self, action: &BrowserAction) -> String {
        match action {
            BrowserAction::Navigate { .. } => "navigate",
            BrowserAction::Click { .. } => "click",
            BrowserAction::Type { .. } => "type",
            BrowserAction::Extract { .. } => "extract",
            BrowserAction::Screenshot { .. } => "screenshot",
            BrowserAction::WaitFor { .. } => "wait_for",
        }.to_string()
    }
    
    /// Get metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        let uptime_seconds = self.start_time.elapsed().as_secs();
        
        let action_stats: Vec<ActionStats> = self.action_counts.iter()
            .map(|entry| {
                let action_type = entry.key().clone();
                let count = *entry.value();
                let duration_stats = self.action_durations.get(&action_type)
                    .map(|s| s.value().clone())
                    .unwrap_or_default();
                
                ActionStats {
                    action_type,
                    count,
                    avg_duration_ms: duration_stats.average() as f64 / 1000.0,
                    min_duration_ms: duration_stats.min as f64 / 1000.0,
                    max_duration_ms: duration_stats.max as f64 / 1000.0,
                }
            })
            .collect();
        
        let error_stats: Vec<ErrorStats> = self.error_counts.iter()
            .map(|entry| ErrorStats {
                error_type: entry.key().clone(),
                count: *entry.value(),
            })
            .collect();
        
        MetricsSnapshot {
            uptime_seconds,
            total_actions: self.total_actions.load(Ordering::Relaxed),
            successful_actions: self.successful_actions.load(Ordering::Relaxed),
            failed_actions: self.failed_actions.load(Ordering::Relaxed),
            success_rate: self.calculate_success_rate(),
            active_contexts: self.active_contexts.load(Ordering::Relaxed),
            total_contexts_created: self.total_contexts_created.load(Ordering::Relaxed),
            navigation_success_rate: self.navigation_stats.success_rate(),
            action_stats,
            error_stats,
        }
    }
    
    /// Calculate overall success rate
    fn calculate_success_rate(&self) -> f64 {
        let total = self.total_actions.load(Ordering::Relaxed);
        let successful = self.successful_actions.load(Ordering::Relaxed);
        
        if total == 0 {
            0.0
        } else {
            (successful as f64 / total as f64) * 100.0
        }
    }
}

/// Duration statistics
#[derive(Debug, Clone, Default)]
struct DurationStats {
    count: u64,
    sum: u64,
    min: u64,
    max: u64,
}

impl DurationStats {
    fn new() -> Self {
        Self {
            count: 0,
            sum: 0,
            min: u64::MAX,
            max: 0,
        }
    }
    
    fn add_sample(&mut self, duration: u64) {
        self.count += 1;
        self.sum += duration;
        self.min = self.min.min(duration);
        self.max = self.max.max(duration);
    }
    
    fn average(&self) -> u64 {
        if self.count == 0 {
            0
        } else {
            self.sum / self.count
        }
    }
}

/// Navigation-specific statistics
struct NavigationStats {
    total: AtomicU64,
    successful: AtomicU64,
}

impl NavigationStats {
    fn new() -> Self {
        Self {
            total: AtomicU64::new(0),
            successful: AtomicU64::new(0),
        }
    }
    
    fn record_success(&self) {
        self.total.fetch_add(1, Ordering::Relaxed);
        self.successful.fetch_add(1, Ordering::Relaxed);
    }
    
    fn record_failure(&self) {
        self.total.fetch_add(1, Ordering::Relaxed);
    }
    
    fn success_rate(&self) -> f64 {
        let total = self.total.load(Ordering::Relaxed);
        let successful = self.successful.load(Ordering::Relaxed);
        
        if total == 0 {
            0.0
        } else {
            (successful as f64 / total as f64) * 100.0
        }
    }
}

/// Snapshot of current metrics
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshot {
    pub uptime_seconds: u64,
    pub total_actions: u64,
    pub successful_actions: u64,
    pub failed_actions: u64,
    pub success_rate: f64,
    pub active_contexts: usize,
    pub total_contexts_created: u64,
    pub navigation_success_rate: f64,
    pub action_stats: Vec<ActionStats>,
    pub error_stats: Vec<ErrorStats>,
}

/// Statistics per action type
#[derive(Debug, Clone, Serialize)]
pub struct ActionStats {
    pub action_type: String,
    pub count: u64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: f64,
    pub max_duration_ms: f64,
}

/// Error statistics
#[derive(Debug, Clone, Serialize)]
pub struct ErrorStats {
    pub error_type: String,
    pub count: u64,
}

/// Prometheus-compatible metrics export
impl BrowserMetrics {
    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();
        let snapshot = self.snapshot();
        
        // Basic counters
        output.push_str(&format!("# HELP browser_actions_total Total browser actions executed\n"));
        output.push_str(&format!("# TYPE browser_actions_total counter\n"));
        output.push_str(&format!("browser_actions_total {}\n", snapshot.total_actions));
        
        output.push_str(&format!("# HELP browser_actions_successful Total successful browser actions\n"));
        output.push_str(&format!("# TYPE browser_actions_successful counter\n"));
        output.push_str(&format!("browser_actions_successful {}\n", snapshot.successful_actions));
        
        output.push_str(&format!("# HELP browser_actions_failed Total failed browser actions\n"));
        output.push_str(&format!("# TYPE browser_actions_failed counter\n"));
        output.push_str(&format!("browser_actions_failed {}\n", snapshot.failed_actions));
        
        // Gauges
        output.push_str(&format!("# HELP browser_contexts_active Current active browser contexts\n"));
        output.push_str(&format!("# TYPE browser_contexts_active gauge\n"));
        output.push_str(&format!("browser_contexts_active {}\n", snapshot.active_contexts));
        
        output.push_str(&format!("# HELP browser_success_rate Overall success rate percentage\n"));
        output.push_str(&format!("# TYPE browser_success_rate gauge\n"));
        output.push_str(&format!("browser_success_rate {}\n", snapshot.success_rate));
        
        // Action-specific metrics
        output.push_str(&format!("# HELP browser_action_duration_ms Action duration in milliseconds\n"));
        output.push_str(&format!("# TYPE browser_action_duration_ms summary\n"));
        for stat in &snapshot.action_stats {
            output.push_str(&format!(
                "browser_action_duration_ms{{action=\"{}\",quantile=\"0.0\"}} {}\n",
                stat.action_type, stat.min_duration_ms
            ));
            output.push_str(&format!(
                "browser_action_duration_ms{{action=\"{}\",quantile=\"0.5\"}} {}\n",
                stat.action_type, stat.avg_duration_ms
            ));
            output.push_str(&format!(
                "browser_action_duration_ms{{action=\"{}\",quantile=\"1.0\"}} {}\n",
                stat.action_type, stat.max_duration_ms
            ));
            output.push_str(&format!(
                "browser_action_duration_ms_count{{action=\"{}\"}} {}\n",
                stat.action_type, stat.count
            ));
        }
        
        // Error metrics
        output.push_str(&format!("# HELP browser_errors_total Total errors by type\n"));
        output.push_str(&format!("# TYPE browser_errors_total counter\n"));
        for stat in &snapshot.error_stats {
            output.push_str(&format!(
                "browser_errors_total{{error=\"{}\"}} {}\n",
                stat.error_type, stat.count
            ));
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::BrowserAction;

    #[test]
    fn test_metrics_recording() {
        let metrics = BrowserMetrics::new();
        
        let action = BrowserAction::Navigate { 
            url: "https://example.com".to_string() 
        };
        
        metrics.record_action_start(&action);
        metrics.record_action_complete(&action, &Ok(crate::controller::ActionResult::WaitComplete), Duration::from_millis(100));
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_actions, 1);
        assert_eq!(snapshot.successful_actions, 1);
        assert_eq!(snapshot.failed_actions, 0);
        assert_eq!(snapshot.success_rate, 100.0);
    }
}