//! Circuit breaker unit tests

use twohal9_server::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
use std::time::Duration;

#[tokio::test]
async fn test_circuit_breaker_state_transitions() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout: Duration::from_millis(100),
        window: Duration::from_secs(60),
    };
    
    let cb = CircuitBreaker::new("test-service".to_string(), config);
    
    // Initial state should be closed
    assert_eq!(cb.state().await, CircuitState::Closed);
    assert!(cb.allow_request().await);
    
    // Record some successes - should remain closed
    cb.record_success().await;
    cb.record_success().await;
    assert_eq!(cb.state().await, CircuitState::Closed);
    
    // Record failures up to threshold
    cb.record_failure().await;
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitState::Closed); // Still closed
    
    // One more failure should open the circuit
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitState::Open);
    assert!(!cb.allow_request().await);
    
    // Wait for timeout to transition to half-open
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(cb.allow_request().await); // This transitions to half-open
    assert_eq!(cb.state().await, CircuitState::HalfOpen);
    
    // Success in half-open state
    cb.record_success().await;
    assert_eq!(cb.state().await, CircuitState::HalfOpen); // Still half-open
    
    // Another success should close the circuit
    cb.record_success().await;
    assert_eq!(cb.state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_failure_in_half_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        success_threshold: 3,
        timeout: Duration::from_millis(50),
        window: Duration::from_secs(60),
    };
    
    let cb = CircuitBreaker::new("test-service".to_string(), config);
    
    // Open the circuit
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitState::Open);
    
    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Transition to half-open
    assert!(cb.allow_request().await);
    assert_eq!(cb.state().await, CircuitState::HalfOpen);
    
    // Failure in half-open should immediately reopen
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitState::Open);
    assert!(!cb.allow_request().await);
}

#[tokio::test]
async fn test_circuit_breaker_window_expiry() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout: Duration::from_millis(100),
        window: Duration::from_millis(200), // Short window for testing
    };
    
    let cb = CircuitBreaker::new("test-service".to_string(), config);
    
    // Record two failures
    cb.record_failure().await;
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitState::Closed);
    
    // Wait for window to expire
    tokio::time::sleep(Duration::from_millis(250)).await;
    
    // Record another failure - should not open (old failures expired)
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitState::Closed);
    
    // But three new failures should open
    cb.record_failure().await;
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitState::Open);
}

#[tokio::test]
async fn test_circuit_breaker_reset() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        ..Default::default()
    };
    
    let cb = CircuitBreaker::new("test-service".to_string(), config);
    
    // Open the circuit
    cb.record_failure().await;
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitState::Open);
    
    // Reset
    cb.reset().await;
    assert_eq!(cb.state().await, CircuitState::Closed);
    assert!(cb.allow_request().await);
    
    // Should require new failures to open again
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_concurrent_access() {
    use std::sync::Arc;
    
    let config = CircuitBreakerConfig {
        failure_threshold: 10,
        ..Default::default()
    };
    
    let cb = Arc::new(CircuitBreaker::new("test-service".to_string(), config));
    
    // Spawn multiple tasks recording failures
    let mut handles = vec![];
    
    for _ in 0..10 {
        let cb_clone = cb.clone();
        let handle = tokio::spawn(async move {
            cb_clone.record_failure().await;
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Circuit should be open after 10 failures
    assert_eq!(cb.state().await, CircuitState::Open);
}