//! Monitoring and observability for migration process

use std::sync::Arc;
use std::collections::VecDeque;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};
use crate::{Result, Error};

/// Migration metrics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MigrationMetrics {
    // Performance metrics
    pub flat_p50_latency: f64,
    pub flat_p99_latency: f64,
    pub hierarchical_p50_latency: f64,
    pub hierarchical_p99_latency: f64,
    
    // Error metrics
    pub flat_error_rate: f32,
    pub hierarchical_error_rate: f32,
    pub divergence_count: u64,
    
    // Migration progress
    pub neurons_migrated: usize,
    pub migration_percentage: f32,
    
    // System health
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub active_connections: usize,
    
    // Comparison metrics
    pub output_parity: f32,
    pub performance_ratio: f32,
}

impl MigrationMetrics {
    pub fn is_healthy(&self) -> bool {
        self.hierarchical_error_rate < 0.05 &&
        self.cpu_usage < 80.0 &&
        self.memory_usage < 80.0 &&
        self.output_parity > 0.95
    }
}

/// Migration monitor for observability
pub struct MigrationMonitor {
    metrics: Arc<RwLock<MigrationMetrics>>,
    history: Arc<RwLock<MetricsHistory>>,
    alerts: Arc<RwLock<Vec<Alert>>>,
    health_checker: Arc<HealthChecker>,
}

impl MigrationMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(MigrationMetrics::default())),
            history: Arc::new(RwLock::new(MetricsHistory::new(1000))),
            alerts: Arc::new(RwLock::new(Vec::new())),
            health_checker: Arc::new(HealthChecker::new()),
        }
    }
    
    /// Start monitoring
    pub async fn start(&self) -> Result<()> {
        // Start metrics collection
        self.start_metrics_collection();
        
        // Start health monitoring
        self.start_health_monitoring();
        
        // Start alert monitoring
        self.start_alert_monitoring();
        
        Ok(())
    }
    
    fn start_metrics_collection(&self) {
        let metrics = self.metrics.clone();
        let history = self.history.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            
            loop {
                interval.tick().await;
                
                // Collect metrics (would integrate with actual systems)
                let current_metrics = Self::collect_metrics().await;
                
                // Update current metrics
                *metrics.write() = current_metrics.clone();
                
                // Add to history
                history.write().add(current_metrics);
            }
        });
    }
    
    fn start_health_monitoring(&self) {
        let metrics = self.metrics.clone();
        let health_checker = self.health_checker.clone();
        let alerts = self.alerts.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                let current_metrics = metrics.read().clone();
                let health_status = health_checker.check(&current_metrics);
                
                if !health_status.is_healthy {
                    let alert = Alert {
                        level: AlertLevel::Warning,
                        message: format!("System health degraded: {:?}", health_status.issues),
                        timestamp: chrono::Utc::now(),
                        metrics: current_metrics,
                    };
                    
                    alerts.write().push(alert);
                }
            }
        });
    }
    
    fn start_alert_monitoring(&self) {
        let alerts = self.alerts.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                // Process alerts
                let current_alerts = alerts.read().clone();
                for alert in &current_alerts {
                    match alert.level {
                        AlertLevel::Critical => {
                            tracing::error!("CRITICAL ALERT: {}", alert.message);
                            // TODO: Send notifications
                        }
                        AlertLevel::Warning => {
                            tracing::warn!("WARNING: {}", alert.message);
                        }
                        AlertLevel::Info => {
                            tracing::info!("INFO: {}", alert.message);
                        }
                    }
                }
                
                // Clear old alerts
                alerts.write().retain(|a| {
                    chrono::Utc::now() - a.timestamp < chrono::Duration::minutes(5)
                });
            }
        });
    }
    
    async fn collect_metrics() -> MigrationMetrics {
        // TODO: Integrate with actual monitoring systems
        // This is a placeholder implementation
        
        MigrationMetrics {
            flat_p50_latency: 25.0 + rand::random::<f64>() * 5.0,
            flat_p99_latency: 50.0 + rand::random::<f64>() * 10.0,
            hierarchical_p50_latency: 20.0 + rand::random::<f64>() * 5.0,
            hierarchical_p99_latency: 45.0 + rand::random::<f64>() * 10.0,
            
            flat_error_rate: 0.001 + rand::random::<f32>() * 0.002,
            hierarchical_error_rate: 0.002 + rand::random::<f32>() * 0.003,
            divergence_count: rand::random::<u64>() % 10,
            
            neurons_migrated: 500,
            migration_percentage: 50.0,
            
            cpu_usage: 40.0 + rand::random::<f32>() * 20.0,
            memory_usage: 50.0 + rand::random::<f32>() * 10.0,
            active_connections: 100 + rand::random::<usize>() % 50,
            
            output_parity: 0.98 + rand::random::<f32>() * 0.02,
            performance_ratio: 0.9 + rand::random::<f32>() * 0.2,
        }
    }
    
    /// Get current metrics
    pub async fn get_current_metrics(&self) -> MigrationMetrics {
        self.metrics.read().clone()
    }
    
    /// Get health score (0.0 - 1.0)
    pub async fn get_health_score(&self) -> Result<f32> {
        let metrics = self.metrics.read();
        let score = self.health_checker.calculate_score(&metrics);
        Ok(score)
    }
    
    /// Get output parity between systems
    pub async fn get_output_parity(&self) -> Result<f32> {
        Ok(self.metrics.read().output_parity)
    }
    
    /// Get performance ratio (hierarchical / flat)
    pub async fn get_performance_ratio(&self) -> Result<f32> {
        let metrics = self.metrics.read();
        let ratio = metrics.hierarchical_p99_latency / metrics.flat_p99_latency;
        Ok(ratio as f32)
    }
    
    /// Get error rate
    pub async fn get_error_rate(&self) -> Result<f32> {
        Ok(self.metrics.read().hierarchical_error_rate)
    }
    
    /// Get P99 latency
    pub async fn get_p99_latency(&self) -> Result<f64> {
        Ok(self.metrics.read().hierarchical_p99_latency)
    }
    
    /// Check if system is stable for duration
    pub async fn is_stable_for(&self, duration: std::time::Duration) -> Result<bool> {
        let history = self.history.read();
        let required_samples = duration.as_secs() as usize;
        
        if history.len() < required_samples {
            return Ok(false);
        }
        
        // Check last N samples are all healthy
        let stable = history.recent(required_samples)
            .all(|m| m.is_healthy());
        
        Ok(stable)
    }
    
    /// Check if resource usage is optimal
    pub async fn is_resource_usage_optimal(&self) -> Result<bool> {
        let metrics = self.metrics.read();
        let optimal = metrics.cpu_usage < 60.0 && 
                     metrics.memory_usage < 70.0 &&
                     metrics.hierarchical_p99_latency < metrics.flat_p99_latency * 1.1;
        Ok(optimal)
    }
    
    /// Check if all metrics are healthy
    pub async fn are_all_metrics_healthy(&self) -> Result<bool> {
        let metrics = self.metrics.read();
        let healthy = metrics.is_healthy() &&
                     metrics.divergence_count < 100 &&
                     metrics.performance_ratio < 1.2;
        Ok(healthy)
    }
}

/// Metrics history for trend analysis
struct MetricsHistory {
    capacity: usize,
    history: VecDeque<MigrationMetrics>,
}

impl MetricsHistory {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            history: VecDeque::with_capacity(capacity),
        }
    }
    
    fn add(&mut self, metrics: MigrationMetrics) {
        if self.history.len() >= self.capacity {
            self.history.pop_front();
        }
        self.history.push_back(metrics);
    }
    
    fn len(&self) -> usize {
        self.history.len()
    }
    
    fn recent(&self, count: usize) -> impl Iterator<Item = &MigrationMetrics> {
        let start = self.history.len().saturating_sub(count);
        self.history.range(start..)
    }
}

/// Health checker
struct HealthChecker {
    thresholds: HealthThresholds,
}

impl HealthChecker {
    fn new() -> Self {
        Self {
            thresholds: HealthThresholds::default(),
        }
    }
    
    fn check(&self, metrics: &MigrationMetrics) -> HealthStatus {
        let mut issues = Vec::new();
        
        if metrics.hierarchical_error_rate > self.thresholds.max_error_rate {
            issues.push(format!(
                "Error rate {} exceeds threshold {}",
                metrics.hierarchical_error_rate,
                self.thresholds.max_error_rate
            ));
        }
        
        if metrics.cpu_usage > self.thresholds.max_cpu_usage {
            issues.push(format!(
                "CPU usage {}% exceeds threshold {}%",
                metrics.cpu_usage,
                self.thresholds.max_cpu_usage
            ));
        }
        
        if metrics.memory_usage > self.thresholds.max_memory_usage {
            issues.push(format!(
                "Memory usage {}% exceeds threshold {}%",
                metrics.memory_usage,
                self.thresholds.max_memory_usage
            ));
        }
        
        if metrics.output_parity < self.thresholds.min_output_parity {
            issues.push(format!(
                "Output parity {} below threshold {}",
                metrics.output_parity,
                self.thresholds.min_output_parity
            ));
        }
        
        HealthStatus {
            is_healthy: issues.is_empty(),
            issues,
        }
    }
    
    fn calculate_score(&self, metrics: &MigrationMetrics) -> f32 {
        let mut score = 1.0;
        
        // Deduct points for issues
        score -= (metrics.hierarchical_error_rate / self.thresholds.max_error_rate).min(0.3);
        score -= ((metrics.cpu_usage - 50.0) / 50.0).max(0.0).min(0.2);
        score -= ((metrics.memory_usage - 50.0) / 50.0).max(0.0).min(0.2);
        score -= (1.0 - metrics.output_parity).min(0.3);
        
        score.max(0.0)
    }
}

/// Health thresholds
#[derive(Debug, Clone)]
struct HealthThresholds {
    max_error_rate: f32,
    max_cpu_usage: f32,
    max_memory_usage: f32,
    min_output_parity: f32,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            max_error_rate: 0.05,
            max_cpu_usage: 80.0,
            max_memory_usage: 80.0,
            min_output_parity: 0.95,
        }
    }
}

/// Health status
struct HealthStatus {
    is_healthy: bool,
    issues: Vec<String>,
}

/// Alert for monitoring
#[derive(Debug, Clone)]
struct Alert {
    level: AlertLevel,
    message: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    metrics: MigrationMetrics,
}

#[derive(Debug, Clone, PartialEq)]
enum AlertLevel {
    Info,
    Warning,
    Critical,
}

/// Dashboard metrics for visualization
#[derive(Debug, Clone, Serialize)]
pub struct DashboardMetrics {
    pub migration_progress: MigrationProgressView,
    pub performance_comparison: PerformanceComparison,
    pub system_health: SystemHealthView,
    pub alerts: Vec<AlertView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MigrationProgressView {
    pub percentage: f32,
    pub neurons_migrated: usize,
    pub neurons_total: usize,
    pub current_phase: String,
    pub estimated_completion: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceComparison {
    pub latency_chart: Vec<LatencyPoint>,
    pub error_rate_chart: Vec<ErrorRatePoint>,
    pub resource_usage_chart: Vec<ResourcePoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LatencyPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub flat_p50: f64,
    pub flat_p99: f64,
    pub hierarchical_p50: f64,
    pub hierarchical_p99: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorRatePoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub flat_error_rate: f32,
    pub hierarchical_error_rate: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourcePoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_usage: f32,
    pub memory_usage: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemHealthView {
    pub overall_score: f32,
    pub component_health: HashMap<String, f32>,
    pub active_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlertView {
    pub level: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// External dependency placeholder
mod rand {
    pub fn random<T>() -> T
    where
        T: Default,
    {
        T::default()
    }
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_health_check() {
        let mut metrics = MigrationMetrics::default();
        metrics.hierarchical_error_rate = 0.03;
        metrics.cpu_usage = 70.0;
        metrics.memory_usage = 75.0;
        metrics.output_parity = 0.97;
        
        assert!(metrics.is_healthy());
        
        metrics.hierarchical_error_rate = 0.06;
        assert!(!metrics.is_healthy());
    }
    
    #[test]
    fn test_health_score_calculation() {
        let checker = HealthChecker::new();
        
        let good_metrics = MigrationMetrics {
            hierarchical_error_rate: 0.01,
            cpu_usage: 40.0,
            memory_usage: 50.0,
            output_parity: 0.99,
            ..Default::default()
        };
        
        let score = checker.calculate_score(&good_metrics);
        assert!(score > 0.9);
        
        let bad_metrics = MigrationMetrics {
            hierarchical_error_rate: 0.05,
            cpu_usage: 85.0,
            memory_usage: 85.0,
            output_parity: 0.90,
            ..Default::default()
        };
        
        let bad_score = checker.calculate_score(&bad_metrics);
        assert!(bad_score < 0.5);
    }
}