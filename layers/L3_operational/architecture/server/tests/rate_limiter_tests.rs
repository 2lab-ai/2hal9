//! Integration tests for rate limiting

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
    routing::get,
};
use std::net::SocketAddr;
use tower::ServiceExt;
use hal9_server::rate_limiter::{RateLimiter, RateLimitConfig};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_rate_limiter_integration() {
    // Create rate limiter with low limits for testing
    let config = RateLimitConfig {
        max_requests: 3,
        window_duration: Duration::from_secs(1),
        enabled: true,
        burst_size: 0,
    };
    
    let rate_limiter = Arc::new(RateLimiter::new(config));
    
    // Create a simple test router
    let app = Router::new()
        .route("/test", get(|| async { "OK" }))
        .layer(axum::Extension(rate_limiter));
    
    // Test IP address
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    
    // Make requests up to the limit
    for i in 0..3 {
        let request = Request::builder()
            .uri("/test")
            .extension(axum::extract::ConnectInfo(addr))
            .body(Body::empty())
            .unwrap();
            
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK, "Request {} should succeed", i + 1);
    }
    
    // Next request should be rate limited
    let request = Request::builder()
        .uri("/test")
        .extension(axum::extract::ConnectInfo(addr))
        .body(Body::empty())
        .unwrap();
        
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    
    // Check retry-after header
    assert!(response.headers().contains_key("Retry-After"));
}

#[tokio::test]
async fn test_rate_limiter_different_ips() {
    // Create rate limiter
    let config = RateLimitConfig {
        max_requests: 2,
        window_duration: Duration::from_secs(1),
        enabled: true,
        burst_size: 0,
    };
    
    let rate_limiter = Arc::new(RateLimiter::new(config));
    
    // Create test router
    let app = Router::new()
        .route("/test", get(|| async { "OK" }))
        .layer(axum::Extension(rate_limiter));
    
    // Test with different IPs
    let addr1: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let addr2: SocketAddr = "127.0.0.2:8080".parse().unwrap();
    
    // Both IPs should be able to make requests
    for addr in [addr1, addr2] {
        for i in 0..2 {
            let request = Request::builder()
                .uri("/test")
                .extension(axum::extract::ConnectInfo(addr))
                .body(Body::empty())
                .unwrap();
                
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(
                response.status(), 
                StatusCode::OK, 
                "Request {} from {:?} should succeed", 
                i + 1, 
                addr
            );
        }
    }
}

#[tokio::test]
async fn test_rate_limiter_disabled() {
    // Create disabled rate limiter
    let config = RateLimitConfig {
        max_requests: 1,
        window_duration: Duration::from_secs(1),
        enabled: false,
        burst_size: 0,
    };
    
    let rate_limiter = Arc::new(RateLimiter::new(config));
    
    // Create test router
    let app = Router::new()
        .route("/test", get(|| async { "OK" }))
        .layer(axum::Extension(rate_limiter));
    
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    
    // Should allow unlimited requests when disabled
    for i in 0..10 {
        let request = Request::builder()
            .uri("/test")
            .extension(axum::extract::ConnectInfo(addr))
            .body(Body::empty())
            .unwrap();
            
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK, "Request {} should succeed when rate limiting is disabled", i + 1);
    }
}

#[tokio::test]
async fn test_rate_limiter_token_refill() {
    // Create rate limiter with short window
    let config = RateLimitConfig {
        max_requests: 2,
        window_duration: Duration::from_millis(500),
        enabled: true,
        burst_size: 0,
    };
    
    let rate_limiter = Arc::new(RateLimiter::new(config));
    
    // Create test router
    let app = Router::new()
        .route("/test", get(|| async { "OK" }))
        .layer(axum::Extension(rate_limiter));
    
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    
    // Use up the tokens
    for _ in 0..2 {
        let request = Request::builder()
            .uri("/test")
            .extension(axum::extract::ConnectInfo(addr))
            .body(Body::empty())
            .unwrap();
            
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    // Should be rate limited
    let request = Request::builder()
        .uri("/test")
        .extension(axum::extract::ConnectInfo(addr))
        .body(Body::empty())
        .unwrap();
        
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    
    // Wait for tokens to refill
    tokio::time::sleep(Duration::from_millis(600)).await;
    
    // Should be able to make requests again
    let request = Request::builder()
        .uri("/test")
        .extension(axum::extract::ConnectInfo(addr))
        .body(Body::empty())
        .unwrap();
        
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}