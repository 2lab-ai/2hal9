//! HTTP middleware for request/response logging and monitoring

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::time::Instant;
use tracing::{info, warn, error, Instrument};
use uuid::Uuid;
use http_body_util::BodyExt;
use crate::logging::{api_span, generate_trace_id};

/// HTTP header name for trace ID
pub const TRACE_ID_HEADER: &str = "X-Trace-Id";

/// Middleware for comprehensive request/response logging
pub async fn logging_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    
    // Extract or generate trace ID
    let trace_id = req
        .headers()
        .get(TRACE_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(generate_trace_id);
    
    // Extract request metadata
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().map(String::from);
    let headers = req.headers().clone();
    
    // Create span for this request
    let span = api_span(method.as_str(), &path, &trace_id);
    
    // Log request
    info!(
        target: "http.request",
        method = %method,
        path = %path,
        query = ?query,
        trace_id = %trace_id,
        user_agent = ?headers.get("user-agent").and_then(|v| v.to_str().ok()),
        content_type = ?headers.get("content-type").and_then(|v| v.to_str().ok()),
        "Incoming request"
    );
    
    // Process request within span
    let response = async move {
        // Add trace ID to request extensions for downstream use
        let mut req = req;
        req.extensions_mut().insert(trace_id.clone());
        
        // Call the next middleware/handler
        let response = next.run(req).await;
        
        // Log response
        let duration = start.elapsed();
        let status = response.status();
        
        if status.is_success() {
            info!(
                target: "http.response",
                status = status.as_u16(),
                duration_ms = duration.as_millis(),
                trace_id = %trace_id,
                "Request completed successfully"
            );
        } else if status.is_client_error() {
            warn!(
                target: "http.response",
                status = status.as_u16(),
                duration_ms = duration.as_millis(),
                trace_id = %trace_id,
                "Client error response"
            );
        } else if status.is_server_error() {
            error!(
                target: "http.response",
                status = status.as_u16(),
                duration_ms = duration.as_millis(),
                trace_id = %trace_id,
                "Server error response"
            );
        }
        
        // Add trace ID to response headers
        let mut response = response;
        response.headers_mut().insert(
            TRACE_ID_HEADER,
            trace_id.parse().unwrap_or_else(|_| {
                format!("{}", Uuid::new_v4()).parse().unwrap()
            })
        );
        
        response
    }
    .instrument(span)
    .await;
    
    Ok(response)
}

/// Middleware for logging request and response bodies (use with caution in production)
pub async fn body_logging_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let (parts, body) = req.into_parts();
    
    // Read request body
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => {
            return Ok(StatusCode::BAD_REQUEST.into_response());
        }
    };
    
    // Log request body if not too large
    if bytes.len() < 10_000 {  // 10KB limit
        if let Ok(body_str) = std::str::from_utf8(&bytes) {
            info!(
                target: "http.request.body",
                size = bytes.len(),
                body = %body_str,
                "Request body"
            );
        }
    } else {
        info!(
            target: "http.request.body",
            size = bytes.len(),
            "Request body too large to log"
        );
    }
    
    // Reconstruct request
    let req = Request::from_parts(parts, Body::from(bytes));
    
    // Process request
    let response = next.run(req).await;
    
    // For responses, we'd need to buffer the body which is more complex
    // and not recommended for production use
    
    Ok(response)
}

/// Extract trace ID from request extensions
pub fn extract_trace_id(req: &Request) -> Option<String> {
    req.extensions().get::<String>().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get, Router};
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_trace_id_generation() {
        // Create a test app with the middleware
        let app = Router::new()
            .route("/test", get(|| async move { 
                "OK" 
            }))
            .layer(axum::middleware::from_fn(logging_middleware));
            
        // Create request without trace ID
        let req = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
            
        let response = app.oneshot(req).await.unwrap();
        
        // Check that trace ID was added to response headers
        assert!(response.headers().contains_key(TRACE_ID_HEADER));
        
        // Verify it's a valid UUID-like format
        let trace_id = response.headers().get(TRACE_ID_HEADER).unwrap();
        assert!(trace_id.to_str().unwrap().contains('-'));
    }
    
    #[tokio::test]
    async fn test_existing_trace_id_preserved() {
        let existing_trace_id = "test-trace-id-123";
        let expected_trace_id = existing_trace_id.to_string();
        
        // Create a test app that can check the trace ID in extensions
        let app = Router::new()
            .route("/test", get(move |req: Request<Body>| {
                let expected = expected_trace_id.clone();
                async move {
                    // Verify trace ID was added to extensions
                    let trace_id = req.extensions().get::<String>()
                        .expect("Trace ID should be in extensions");
                    assert_eq!(trace_id, &expected);
                    "OK"
                }
            }))
            .layer(axum::middleware::from_fn(logging_middleware));
            
        // Create request with existing trace ID
        let req = Request::builder()
            .uri("/test")
            .header(TRACE_ID_HEADER, existing_trace_id)
            .body(Body::empty())
            .unwrap();
            
        let response = app.oneshot(req).await.unwrap();
        
        // Check response header matches the original trace ID
        assert_eq!(
            response.headers().get(TRACE_ID_HEADER).unwrap().to_str().unwrap(),
            existing_trace_id
        );
    }
    
    #[tokio::test]
    async fn test_body_logging_middleware() {
        // Create a test app that echoes the request body
        let app = Router::new()
            .route("/test", axum::routing::post(|body: String| async move {
                body
            }))
            .layer(axum::middleware::from_fn(body_logging_middleware));
            
        let test_body = "test request body";
        let req = Request::builder()
            .method("POST")
            .uri("/test")
            .header("content-type", "text/plain")
            .body(Body::from(test_body))
            .unwrap();
            
        let response = app.oneshot(req).await.unwrap();
        
        // Verify response is successful
        assert_eq!(response.status(), StatusCode::OK);
        
        // Read response body
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_body = String::from_utf8(body_bytes.to_vec()).unwrap();
        
        // Verify the body was processed correctly
        assert_eq!(response_body, test_body);
    }
}