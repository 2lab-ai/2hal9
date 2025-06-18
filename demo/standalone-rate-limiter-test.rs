// Standalone test for rate limiter functionality
use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(max_tokens: u32, window_duration: Duration) -> Self {
        let max_tokens = max_tokens as f64;
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate: max_tokens / window_duration.as_secs_f64(),
            last_refill: Instant::now(),
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
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let tokens_to_add = elapsed * self.refill_rate;
        
        self.tokens = (self.tokens + tokens_to_add).min(self.max_tokens);
        self.last_refill = now;
    }
}

fn main() {
    println!("Testing Rate Limiter Implementation");
    println!("==================================");
    
    // Test 1: Basic token consumption
    println!("\nTest 1: Basic token consumption");
    let mut bucket = TokenBucket::new(10, Duration::from_secs(1));
    
    // Should allow 10 requests
    let mut allowed = 0;
    for i in 0..15 {
        if bucket.try_consume(1.0) {
            allowed += 1;
            println!("Request {} - ALLOWED (tokens: {:.2})", i + 1, bucket.tokens);
        } else {
            println!("Request {} - DENIED (tokens: {:.2})", i + 1, bucket.tokens);
        }
    }
    println!("Allowed: {}/15 requests", allowed);
    
    // Test 2: Token refill
    println!("\nTest 2: Token refill after 500ms");
    std::thread::sleep(Duration::from_millis(500));
    bucket.refill();
    println!("Tokens after 500ms: {:.2} (should be ~5)", bucket.tokens);
    
    // Test 3: Burst handling
    println!("\nTest 3: Burst handling");
    let mut burst_bucket = TokenBucket::new(5, Duration::from_secs(1));
    
    // Consume all tokens at once
    if burst_bucket.try_consume(5.0) {
        println!("Burst of 5 requests - ALLOWED");
    }
    
    // Next request should fail
    if !burst_bucket.try_consume(1.0) {
        println!("Next request - DENIED (as expected)");
    }
    
    // Test 4: Multiple clients
    println!("\nTest 4: Multiple clients");
    let mut clients: HashMap<String, TokenBucket> = HashMap::new();
    
    // Simulate 3 clients
    for client_id in ["client1", "client2", "client3"] {
        clients.insert(
            client_id.to_string(),
            TokenBucket::new(3, Duration::from_secs(1))
        );
    }
    
    // Each client makes 5 requests
    for i in 0..5 {
        println!("\nRound {}", i + 1);
        for (client_id, bucket) in &mut clients {
            if bucket.try_consume(1.0) {
                println!("  {} - ALLOWED", client_id);
            } else {
                println!("  {} - DENIED", client_id);
            }
        }
    }
    
    println!("\nRate limiter tests completed!");
}