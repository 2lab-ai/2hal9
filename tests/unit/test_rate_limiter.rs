#[cfg(test)]
mod rate_limiter_tests {
    use std::time::Duration;
    use std::net::IpAddr;
    
    // Mock structures for testing
    #[derive(Debug)]
    struct TokenBucket {
        tokens: f64,
        max_tokens: f64,
        refill_rate: f64,
        last_refill: std::time::Instant,
    }
    
    impl TokenBucket {
        fn new(max_tokens: u32, window: Duration) -> Self {
            let max_tokens = max_tokens as f64;
            Self {
                tokens: max_tokens,
                max_tokens,
                refill_rate: max_tokens / window.as_secs_f64(),
                last_refill: std::time::Instant::now(),
            }
        }
        
        fn try_consume(&mut self, tokens: f64) -> bool {
            self.refill();
            
            if self.tokens >= tokens {
                self.tokens -= tokens;
                true
            } else {
                false
            }
        }
        
        fn refill(&mut self) {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(self.last_refill).as_secs_f64();
            let tokens_to_add = elapsed * self.refill_rate;
            
            self.tokens = (self.tokens + tokens_to_add).min(self.max_tokens);
            self.last_refill = now;
        }
    }
    
    #[test]
    fn test_token_bucket_creation() {
        let bucket = TokenBucket::new(10, Duration::from_secs(1));
        assert_eq!(bucket.max_tokens, 10.0);
        assert_eq!(bucket.tokens, 10.0);
        assert_eq!(bucket.refill_rate, 10.0);
    }
    
    #[test]
    fn test_token_consumption() {
        let mut bucket = TokenBucket::new(10, Duration::from_secs(1));
        
        // Should allow 10 requests
        for i in 0..10 {
            assert!(bucket.try_consume(1.0), "Request {} should succeed", i + 1);
        }
        
        // 11th request should fail
        assert!(!bucket.try_consume(1.0), "11th request should fail");
    }
    
    #[test]
    fn test_token_refill() {
        let mut bucket = TokenBucket::new(10, Duration::from_secs(1));
        
        // Consume all tokens
        assert!(bucket.try_consume(10.0));
        assert_eq!(bucket.tokens, 0.0);
        
        // Wait for refill
        std::thread::sleep(Duration::from_millis(500));
        bucket.refill();
        
        // Should have ~5 tokens
        assert!(bucket.tokens > 4.0 && bucket.tokens < 6.0);
    }
    
    #[test]
    fn test_burst_handling() {
        let mut bucket = TokenBucket::new(5, Duration::from_secs(1));
        
        // Should allow burst of 5
        assert!(bucket.try_consume(5.0));
        
        // Next request should fail
        assert!(!bucket.try_consume(1.0));
    }
    
    #[test]
    fn test_rate_limiter_per_ip() {
        use std::collections::HashMap;
        use std::str::FromStr;
        
        let mut limiters: HashMap<IpAddr, TokenBucket> = HashMap::new();
        
        let ip1 = IpAddr::from_str("192.168.1.1").unwrap();
        let ip2 = IpAddr::from_str("192.168.1.2").unwrap();
        
        // Create limiters for each IP
        limiters.insert(ip1, TokenBucket::new(5, Duration::from_secs(1)));
        limiters.insert(ip2, TokenBucket::new(5, Duration::from_secs(1)));
        
        // Each IP should have independent limits
        let limiter1 = limiters.get_mut(&ip1).unwrap();
        for _ in 0..5 {
            assert!(limiter1.try_consume(1.0));
        }
        assert!(!limiter1.try_consume(1.0));
        
        // IP2 should still have tokens
        let limiter2 = limiters.get_mut(&ip2).unwrap();
        assert!(limiter2.try_consume(1.0));
    }
}