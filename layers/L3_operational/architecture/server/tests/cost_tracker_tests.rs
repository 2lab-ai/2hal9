//! Cost tracker unit tests

use hal9_server::cost_tracker::{CostTracker, CostStats};
use hal9_core::config::CostControls;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    
    // Record some cost
    tracker.record_cost(0.3, 500).await;
    
    // Should still allow
    assert!(tracker.check_request(400).await.is_ok());
    
    // Record more cost
    tracker.record_cost(0.5, 600).await;
    
    // Should still allow (0.8 < 1.0)
    assert!(tracker.check_request(100).await.is_ok());
    
    // Record cost to exceed limit
    tracker.record_cost(0.3, 300).await;
    
    // Should now reject (1.1 > 1.0)
    let result = tracker.check_request(100).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Hourly cost"));
}

#[tokio::test]
async fn test_token_limits() {
    let config = CostControls {
        max_cost_per_hour: 100.0,
        max_cost_per_day: 1000.0,
        max_tokens_per_request: 500,
        alert_threshold: 0.8,
    };
    
    let tracker = CostTracker::new(config);
    
    // Should allow request within token limit
    assert!(tracker.check_request(400).await.is_ok());
    
    // Should reject request exceeding token limit
    let result = tracker.check_request(600).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Request tokens"));
}

#[tokio::test]
async fn test_alert_callback() {
    let config = CostControls {
        max_cost_per_hour: 1.0,
        max_cost_per_day: 10.0,
        max_tokens_per_request: 1000,
        alert_threshold: 0.8,
    };
    
    let mut tracker = CostTracker::new(config);
    
    // Set up alert callback
    let alerts = Arc::new(Mutex::new(Vec::new()));
    let alerts_clone = alerts.clone();
    
    tracker.set_alert_callback(move |msg: String| {
        let alerts = alerts_clone.clone();
        tokio::spawn(async move {
            alerts.lock().await.push(msg);
        });
    });
    
    // Record cost below threshold - no alert
    tracker.record_cost(0.5, 500).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    assert_eq!(alerts.lock().await.len(), 0);
    
    // Record cost above threshold - should trigger alert
    tracker.record_cost(0.35, 350).await; // Total: 0.85 (85%)
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    let alert_messages = alerts.lock().await;
    assert!(alert_messages.len() > 0);
    assert!(alert_messages[0].contains("Hourly cost alert"));
}

#[tokio::test]
async fn test_daily_limits() {
    let config = CostControls {
        max_cost_per_hour: 100.0, // High hourly limit
        max_cost_per_day: 1.0,    // Low daily limit
        max_tokens_per_request: 1000,
        alert_threshold: 0.8,
    };
    
    let tracker = CostTracker::new(config);
    
    // Record cost within hourly but approaching daily
    tracker.record_cost(0.9, 900).await;
    
    // Should still allow (0.9 < 1.0)
    assert!(tracker.check_request(100).await.is_ok());
    
    // Record more to exceed daily
    tracker.record_cost(0.2, 200).await;
    
    // Should reject due to daily limit
    let result = tracker.check_request(100).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Daily cost"));
}

#[tokio::test]
async fn test_cost_stats() {
    let config = CostControls {
        max_cost_per_hour: 10.0,
        max_cost_per_day: 100.0,
        max_tokens_per_request: 1000,
        alert_threshold: 0.8,
    };
    
    let tracker = CostTracker::new(config);
    
    // Record some costs
    tracker.record_cost(1.5, 1500).await;
    tracker.record_cost(2.5, 2500).await;
    
    // Get stats
    let stats = tracker.get_stats().await;
    
    assert_eq!(stats.hourly_cost, 4.0);
    assert_eq!(stats.hourly_tokens, 4000);
    assert_eq!(stats.daily_cost, 4.0);
    assert_eq!(stats.daily_tokens, 4000);
    assert_eq!(stats.total_cost, 4.0);
    assert_eq!(stats.hourly_limit, 10.0);
    assert_eq!(stats.daily_limit, 100.0);
}

#[tokio::test]
async fn test_window_reset() {
    // This test would require mocking time or waiting for actual window expiry
    // For now, just verify the tracker can be created and used
    let config = CostControls::default();
    let tracker = CostTracker::new(config);
    
    // Basic operations should work
    assert!(tracker.check_request(100).await.is_ok());
    tracker.record_cost(0.1, 100).await;
    
    let stats = tracker.get_stats().await;
    assert!(stats.hourly_cost > 0.0);
}