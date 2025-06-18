//! Rate limiting middleware for DDoS protection and API usage control

use axum::{
    extract::{Request, ConnectInfo},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{warn, info};

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Time window duration
    pub window_duration: Duration,
    /// Whether to enable rate limiting
    pub enabled: bool,
    /// Burst size for token bucket
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 60,
            window_duration: Duration::from_secs(60),
            enabled: true,
            burst_size: 10,
        }
    }
}

/// Token bucket for rate limiting
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

/// Rate limiter state
pub struct RateLimiter {
    config: RateLimitConfig,
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    cleanup_interval: Duration,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        let limiter = Self {
            config,
            buckets: Arc::new(RwLock::new(HashMap::new())),
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        };

        // Start cleanup task
        limiter.start_cleanup_task();
        
        limiter
    }

    /// Start background task to clean up old buckets
    fn start_cleanup_task(&self) {
        let buckets = Arc::clone(&self.buckets);
        let cleanup_interval = self.cleanup_interval;
        let window_duration = self.config.window_duration;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            
            loop {
                interval.tick().await;
                
                let mut buckets = buckets.write().await;
                let now = Instant::now();
                
                // Remove buckets that haven't been used for 2x window duration
                buckets.retain(|_, bucket| {
                    now.duration_since(bucket.last_refill) < window_duration * 2
                });
                
                info!(
                    "Rate limiter cleanup: {} active buckets",
                    buckets.len()
                );
            }
        });
    }

    /// Check if request should be allowed
    pub async fn check_rate_limit(&self, key: &str) -> Result<(), RateLimitError> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut buckets = self.buckets.write().await;
        
        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| {
                TokenBucket::new(
                    self.config.max_requests + self.config.burst_size,
                    self.config.window_duration,
                )
            });

        if bucket.try_consume(1.0) {
            Ok(())
        } else {
            Err(RateLimitError::TooManyRequests {
                retry_after: self.calculate_retry_after(bucket),
            })
        }
    }

    /// Calculate retry-after duration
    fn calculate_retry_after(&self, bucket: &TokenBucket) -> Duration {
        let tokens_needed = 1.0 - bucket.tokens;
        let seconds_until_refill = tokens_needed / bucket.refill_rate;
        Duration::from_secs_f64(seconds_until_refill.max(1.0))
    }
}

/// Rate limit error types
#[derive(Debug)]
pub enum RateLimitError {
    TooManyRequests { retry_after: Duration },
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        match self {
            RateLimitError::TooManyRequests { retry_after } => {
                let mut response = StatusCode::TOO_MANY_REQUESTS.into_response();
                response.headers_mut().insert(
                    "Retry-After",
                    retry_after.as_secs().to_string().parse().unwrap(),
                );
                response.headers_mut().insert(
                    "X-RateLimit-Limit",
                    "60".parse().unwrap(),
                );
                response
            }
        }
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    // Get rate limiter from app state
    let rate_limiter = req
        .extensions()
        .get::<Arc<RateLimiter>>()
        .cloned()
        .ok_or_else(|| {
            warn!("Rate limiter not found in app state");
            RateLimitError::TooManyRequests {
                retry_after: Duration::from_secs(60),
            }
        })?;

    // Use IP address as rate limit key
    let key = addr.ip().to_string();
    
    // Check rate limit
    rate_limiter.check_rate_limit(&key).await?;
    
    // Process request
    Ok(next.run(req).await)
}

/// Per-user rate limiting middleware (requires authentication)
pub async fn user_rate_limit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    // Get rate limiter from app state
    let rate_limiter = req
        .extensions()
        .get::<Arc<RateLimiter>>()
        .cloned()
        .ok_or_else(|| {
            warn!("Rate limiter not found in app state");
            RateLimitError::TooManyRequests {
                retry_after: Duration::from_secs(60),
            }
        })?;

    // Get user ID from request (assumes auth middleware has run)
    let user_id = req
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_else(|| "anonymous".to_string());

    // Use user ID as rate limit key
    let key = format!("user:{}", user_id);
    
    // Check rate limit
    rate_limiter.check_rate_limit(&key).await?;
    
    // Process request
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_bucket() {
        let mut bucket = TokenBucket::new(10, Duration::from_secs(1));
        
        // Should allow initial requests
        assert!(bucket.try_consume(5.0));
        assert!(bucket.try_consume(5.0));
        
        // Should deny when out of tokens
        assert!(!bucket.try_consume(1.0));
        
        // Wait for refill
        tokio::time::sleep(Duration::from_millis(200)).await;
        bucket.refill();
        
        // Should have some tokens refilled
        assert!(bucket.try_consume(1.0));
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimitConfig {
            max_requests: 5,
            window_duration: Duration::from_secs(1),
            enabled: true,
            burst_size: 0,
        };
        
        let limiter = RateLimiter::new(config);
        
        // Should allow requests up to limit
        for _ in 0..5 {
            assert!(limiter.check_rate_limit("test").await.is_ok());
        }
        
        // Should deny after limit
        assert!(limiter.check_rate_limit("test").await.is_err());
        
        // Different key should have separate limit
        assert!(limiter.check_rate_limit("other").await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_disabled() {
        let config = RateLimitConfig {
            max_requests: 0,
            window_duration: Duration::from_secs(1),
            enabled: false,
            burst_size: 0,
        };
        
        let limiter = RateLimiter::new(config);
        
        // Should allow all requests when disabled
        for _ in 0..100 {
            assert!(limiter.check_rate_limit("test").await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_burst_allowance() {
        let config = RateLimitConfig {
            max_requests: 5,
            window_duration: Duration::from_secs(1),
            enabled: true,
            burst_size: 5,
        };
        
        let limiter = RateLimiter::new(config);
        
        // Should allow burst
        for _ in 0..10 {
            assert!(limiter.check_rate_limit("test").await.is_ok());
        }
        
        // Should deny after burst
        assert!(limiter.check_rate_limit("test").await.is_err());
    }
}