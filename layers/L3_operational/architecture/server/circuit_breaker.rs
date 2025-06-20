//! Circuit breaker pattern for fault tolerance

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Normal operation
    Closed,
    /// Circuit open, rejecting requests
    Open,
    /// Testing if service recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Success threshold to close circuit from half-open
    pub success_threshold: u32,
    /// Duration to wait before trying half-open
    pub timeout: Duration,
    /// Time window for counting failures
    pub window: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            window: Duration::from_secs(60),
        }
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    failures: Arc<RwLock<Vec<Instant>>>,
    successes: Arc<RwLock<u32>>,
    last_failure: Arc<RwLock<Option<Instant>>>,
    service_name: String,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(service_name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failures: Arc::new(RwLock::new(Vec::new())),
            successes: Arc::new(RwLock::new(0)),
            last_failure: Arc::new(RwLock::new(None)),
            service_name,
        }
    }
    
    /// Check if request is allowed
    pub async fn allow_request(&self) -> bool {
        let mut state = self.state.write().await;
        
        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = *self.last_failure.read().await {
                    if last_failure.elapsed() >= self.config.timeout {
                        *state = CircuitState::HalfOpen;
                        *self.successes.write().await = 0;
                        info!("Circuit breaker for {} transitioning to half-open", self.service_name);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }
    
    /// Record a successful request
    pub async fn record_success(&self) {
        let state = *self.state.read().await;
        
        if state == CircuitState::HalfOpen {
            let mut successes = self.successes.write().await;
            *successes += 1;
            
            if *successes >= self.config.success_threshold {
                *self.state.write().await = CircuitState::Closed;
                *self.failures.write().await = Vec::new();
                info!("Circuit breaker for {} closed after {} successes", 
                    self.service_name, successes);
            }
        }
    }
    
    /// Record a failed request
    pub async fn record_failure(&self) {
        let now = Instant::now();
        let mut failures = self.failures.write().await;
        
        // Remove old failures outside the window
        failures.retain(|&failure| now.duration_since(failure) < self.config.window);
        
        // Add new failure
        failures.push(now);
        *self.last_failure.write().await = Some(now);
        
        let failure_count = failures.len() as u32;
        
        // Check if we should open the circuit
        if failure_count >= self.config.failure_threshold {
            let mut state = self.state.write().await;
            if *state != CircuitState::Open {
                *state = CircuitState::Open;
                warn!("Circuit breaker for {} opened after {} failures", 
                    self.service_name, failure_count);
            }
        }
        
        // If in half-open state, immediately go back to open
        if *self.state.read().await == CircuitState::HalfOpen {
            *self.state.write().await = CircuitState::Open;
            warn!("Circuit breaker for {} reopened due to failure in half-open state", 
                self.service_name);
        }
    }
    
    /// Get current state
    pub async fn state(&self) -> CircuitState {
        *self.state.read().await
    }
    
    /// Reset the circuit breaker
    pub async fn reset(&self) {
        *self.state.write().await = CircuitState::Closed;
        *self.failures.write().await = Vec::new();
        *self.successes.write().await = 0;
        *self.last_failure.write().await = None;
        info!("Circuit breaker for {} reset", self.service_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_circuit_breaker_opens_after_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Should start closed
        assert_eq!(cb.state().await, CircuitState::Closed);
        assert!(cb.allow_request().await);
        
        // Record failures
        for _ in 0..3 {
            cb.record_failure().await;
        }
        
        // Should be open now
        assert_eq!(cb.state().await, CircuitState::Open);
        assert!(!cb.allow_request().await);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_half_open_transition() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            timeout: Duration::from_millis(100),
            ..Default::default()
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Open the circuit
        cb.record_failure().await;
        assert_eq!(cb.state().await, CircuitState::Open);
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should transition to half-open
        assert!(cb.allow_request().await);
        assert_eq!(cb.state().await, CircuitState::HalfOpen);
    }
}