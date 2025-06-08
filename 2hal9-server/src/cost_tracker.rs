//! Cost tracking and control for Claude API usage

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};
use twohal9_core::{Result, Error, config::CostControls};

/// Time-based cost window
#[derive(Debug, Clone)]
struct CostWindow {
    /// Window start time
    start: Instant,
    /// Total cost in this window
    cost: f64,
    /// Token count in this window
    tokens: u64,
}

impl CostWindow {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            cost: 0.0,
            tokens: 0,
        }
    }
    
    fn is_expired(&self, window_duration: Duration) -> bool {
        self.start.elapsed() > window_duration
    }
    
    fn add_cost(&mut self, cost: f64, tokens: u64) {
        self.cost += cost;
        self.tokens += tokens;
    }
}

/// Cost tracker for monitoring and controlling API costs
pub struct CostTracker {
    /// Cost control configuration
    config: CostControls,
    /// Hourly cost window
    hourly_window: Arc<RwLock<CostWindow>>,
    /// Daily cost window
    daily_window: Arc<RwLock<CostWindow>>,
    /// Total costs since start
    total_cost: Arc<RwLock<f64>>,
    /// Alert callback
    alert_callback: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

impl CostTracker {
    /// Create a new cost tracker
    pub fn new(config: CostControls) -> Self {
        Self {
            config,
            hourly_window: Arc::new(RwLock::new(CostWindow::new())),
            daily_window: Arc::new(RwLock::new(CostWindow::new())),
            total_cost: Arc::new(RwLock::new(0.0)),
            alert_callback: None,
        }
    }
    
    /// Set alert callback
    pub fn set_alert_callback<F>(&mut self, callback: F) 
    where 
        F: Fn(String) + Send + Sync + 'static 
    {
        self.alert_callback = Some(Arc::new(callback));
    }
    
    /// Check if a request with given tokens is allowed
    pub async fn check_request(&self, estimated_tokens: u32) -> Result<()> {
        // Check token limit
        if estimated_tokens > self.config.max_tokens_per_request {
            return Err(Error::CostLimit {
                reason: format!(
                    "Request tokens {} exceeds limit {}",
                    estimated_tokens, self.config.max_tokens_per_request
                ),
            });
        }
        
        // Update windows if expired
        self.update_windows().await;
        
        // Check hourly limit (rough estimate)
        let hourly_cost = self.hourly_window.read().await.cost;
        if hourly_cost >= self.config.max_cost_per_hour {
            return Err(Error::CostLimit {
                reason: format!(
                    "Hourly cost ${:.2} exceeds limit ${:.2}",
                    hourly_cost, self.config.max_cost_per_hour
                ),
            });
        }
        
        // Check daily limit
        let daily_cost = self.daily_window.read().await.cost;
        if daily_cost >= self.config.max_cost_per_day {
            return Err(Error::CostLimit {
                reason: format!(
                    "Daily cost ${:.2} exceeds limit ${:.2}",
                    daily_cost, self.config.max_cost_per_day
                ),
            });
        }
        
        // Check if we're approaching limits
        self.check_alerts(hourly_cost, daily_cost).await;
        
        Ok(())
    }
    
    /// Record actual cost after request
    pub async fn record_cost(&self, cost: f64, tokens: u64) {
        // Update windows
        self.update_windows().await;
        
        // Add to windows
        self.hourly_window.write().await.add_cost(cost, tokens);
        self.daily_window.write().await.add_cost(cost, tokens);
        *self.total_cost.write().await += cost;
        
        // Log cost
        info!(
            "API cost recorded: ${:.4} for {} tokens (hourly: ${:.2}, daily: ${:.2})",
            cost,
            tokens,
            self.hourly_window.read().await.cost,
            self.daily_window.read().await.cost
        );
        
        // Check alerts after recording
        let hourly_cost = self.hourly_window.read().await.cost;
        let daily_cost = self.daily_window.read().await.cost;
        self.check_alerts(hourly_cost, daily_cost).await;
    }
    
    /// Update windows if they've expired
    async fn update_windows(&self) {
        // Check hourly window
        let mut hourly = self.hourly_window.write().await;
        if hourly.is_expired(Duration::from_secs(3600)) {
            info!("Hourly cost window reset (was ${:.2})", hourly.cost);
            *hourly = CostWindow::new();
        }
        drop(hourly);
        
        // Check daily window
        let mut daily = self.daily_window.write().await;
        if daily.is_expired(Duration::from_secs(86400)) {
            info!("Daily cost window reset (was ${:.2})", daily.cost);
            *daily = CostWindow::new();
        }
    }
    
    /// Check if we should send alerts
    async fn check_alerts(&self, hourly_cost: f64, daily_cost: f64) {
        let hourly_ratio = hourly_cost / self.config.max_cost_per_hour;
        let daily_ratio = daily_cost / self.config.max_cost_per_day;
        
        if hourly_ratio >= self.config.alert_threshold {
            let msg = format!(
                "⚠️ Hourly cost alert: ${:.2} ({:.0}% of ${:.2} limit)",
                hourly_cost,
                hourly_ratio * 100.0,
                self.config.max_cost_per_hour
            );
            warn!("{}", msg);
            
            if let Some(callback) = &self.alert_callback {
                callback(msg);
            }
        }
        
        if daily_ratio >= self.config.alert_threshold {
            let msg = format!(
                "⚠️ Daily cost alert: ${:.2} ({:.0}% of ${:.2} limit)",
                daily_cost,
                daily_ratio * 100.0,
                self.config.max_cost_per_day
            );
            warn!("{}", msg);
            
            if let Some(callback) = &self.alert_callback {
                callback(msg);
            }
        }
    }
    
    /// Get current cost statistics
    pub async fn get_stats(&self) -> CostStats {
        CostStats {
            hourly_cost: self.hourly_window.read().await.cost,
            hourly_tokens: self.hourly_window.read().await.tokens,
            daily_cost: self.daily_window.read().await.cost,
            daily_tokens: self.daily_window.read().await.tokens,
            total_cost: *self.total_cost.read().await,
            hourly_limit: self.config.max_cost_per_hour,
            daily_limit: self.config.max_cost_per_day,
        }
    }
}

/// Cost statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct CostStats {
    pub hourly_cost: f64,
    pub hourly_tokens: u64,
    pub daily_cost: f64,
    pub daily_tokens: u64,
    pub total_cost: f64,
    pub hourly_limit: f64,
    pub daily_limit: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cost_limits() {
        let config = CostControls {
            max_cost_per_hour: 1.0,
            max_cost_per_day: 10.0,
            max_tokens_per_request: 1000,
            alert_threshold: 0.8,
        };
        
        let tracker = CostTracker::new(config);
        
        // Should allow initial request
        assert!(tracker.check_request(500).await.is_ok());
        
        // Record cost
        tracker.record_cost(0.5, 500).await;
        
        // Should still allow
        assert!(tracker.check_request(400).await.is_ok());
        
        // Record more cost to exceed hourly limit
        tracker.record_cost(0.6, 600).await;
        
        // Should now reject
        assert!(tracker.check_request(100).await.is_err());
    }
}