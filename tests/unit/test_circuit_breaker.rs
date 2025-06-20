#[cfg(test)]
mod circuit_breaker_tests {
    use std::sync::atomic::{AtomicU32, AtomicI64, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum CircuitState {
        Closed,
        Open,
        HalfOpen,
    }
    
    struct CircuitBreaker {
        failure_threshold: u32,
        recovery_timeout: Duration,
        half_open_requests: u32,
        
        failure_count: AtomicU32,
        success_count: AtomicU32,
        last_failure_time: AtomicI64,
        state: parking_lot::RwLock<CircuitState>,
        half_open_count: AtomicU32,
    }
    
    impl CircuitBreaker {
        fn new(failure_threshold: u32, recovery_timeout: Duration, half_open_requests: u32) -> Self {
            Self {
                failure_threshold,
                recovery_timeout,
                half_open_requests,
                failure_count: AtomicU32::new(0),
                success_count: AtomicU32::new(0),
                last_failure_time: AtomicI64::new(0),
                state: parking_lot::RwLock::new(CircuitState::Closed),
                half_open_count: AtomicU32::new(0),
            }
        }
        
        fn record_success(&self) {
            self.success_count.fetch_add(1, Ordering::Relaxed);
            let mut state = self.state.write();
            
            match *state {
                CircuitState::HalfOpen => {
                    let count = self.half_open_count.fetch_add(1, Ordering::Relaxed) + 1;
                    if count >= self.half_open_requests {
                        *state = CircuitState::Closed;
                        self.failure_count.store(0, Ordering::Relaxed);
                        self.half_open_count.store(0, Ordering::Relaxed);
                    }
                }
                _ => {}
            }
        }
        
        fn record_failure(&self) {
            let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
            self.last_failure_time.store(
                Instant::now().elapsed().as_secs() as i64,
                Ordering::Relaxed
            );
            
            let mut state = self.state.write();
            if failures >= self.failure_threshold && *state == CircuitState::Closed {
                *state = CircuitState::Open;
            }
        }
        
        fn should_allow_request(&self) -> bool {
            let state = self.state.read();
            match *state {
                CircuitState::Closed => true,
                CircuitState::Open => {
                    // Check if we should transition to half-open
                    let last_failure = self.last_failure_time.load(Ordering::Relaxed);
                    let now = Instant::now().elapsed().as_secs() as i64;
                    
                    if now - last_failure > self.recovery_timeout.as_secs() as i64 {
                        drop(state);
                        let mut state = self.state.write();
                        *state = CircuitState::HalfOpen;
                        self.half_open_count.store(0, Ordering::Relaxed);
                        true
                    } else {
                        false
                    }
                }
                CircuitState::HalfOpen => true,
            }
        }
        
        fn get_state(&self) -> CircuitState {
            *self.state.read()
        }
    }
    
    #[test]
    fn test_circuit_breaker_closed_state() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(10), 2);
        
        assert_eq!(cb.get_state(), CircuitState::Closed);
        assert!(cb.should_allow_request());
        
        // Record some successes
        cb.record_success();
        cb.record_success();
        
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }
    
    #[test]
    fn test_circuit_breaker_opens_on_failures() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(10), 2);
        
        // Record failures up to threshold
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Closed);
        
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Closed);
        
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Open);
        assert!(!cb.should_allow_request());
    }
    
    #[test]
    fn test_circuit_breaker_half_open_transition() {
        let cb = Arc::new(CircuitBreaker::new(2, Duration::from_millis(100), 2));
        
        // Open the circuit
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Open);
        
        // Wait for recovery timeout
        std::thread::sleep(Duration::from_millis(150));
        
        // Should transition to half-open
        assert!(cb.should_allow_request());
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);
    }
    
    #[test]
    fn test_circuit_breaker_closes_after_success_in_half_open() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50), 2);
        
        // Open the circuit
        cb.record_failure();
        cb.record_failure();
        
        // Wait and transition to half-open
        std::thread::sleep(Duration::from_millis(100));
        assert!(cb.should_allow_request());
        
        // Record successes in half-open state
        cb.record_success();
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);
        
        cb.record_success();
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }
}