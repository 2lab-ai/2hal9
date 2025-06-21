// E2E Test Framework for HAL9 Server
// Provides utilities for comprehensive end-to-end testing

use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub base_url: String,
    pub ws_url: String,
    pub timeout: Duration,
    pub auth_enabled: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3000".to_string(),
            ws_url: "ws://localhost:3000".to_string(),
            timeout: Duration::from_secs(30),
            auth_enabled: false,
        }
    }
}

/// E2E Test Client with HTTP and WebSocket support
pub struct E2ETestClient {
    http_client: Client,
    config: TestConfig,
    auth_token: Arc<Mutex<Option<String>>>,
}

impl E2ETestClient {
    pub fn new(config: TestConfig) -> Self {
        let http_client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            config,
            auth_token: Arc::new(Mutex::new(None)),
        }
    }

    /// Perform HTTP GET request
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.config.base_url, path);
        let mut req = self.http_client.get(&url);
        
        if let Some(token) = self.auth_token.lock().await.as_ref() {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = req.send().await.context("Failed to send GET request")?;
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("GET {} failed with status {}: {}", path, status, error_text);
        }
        
        response.json().await.context("Failed to parse response")
    }

    /// Perform HTTP POST request
    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.config.base_url, path);
        let mut req = self.http_client.post(&url).json(body);
        
        if let Some(token) = self.auth_token.lock().await.as_ref() {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = req.send().await.context("Failed to send POST request")?;
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("POST {} failed with status {}: {}", path, status, error_text);
        }
        
        response.json().await.context("Failed to parse response")
    }

    /// Perform HTTP PUT request
    pub async fn put<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.config.base_url, path);
        let mut req = self.http_client.put(&url).json(body);
        
        if let Some(token) = self.auth_token.lock().await.as_ref() {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = req.send().await.context("Failed to send PUT request")?;
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("PUT {} failed with status {}: {}", path, status, error_text);
        }
        
        response.json().await.context("Failed to parse response")
    }

    /// Perform HTTP DELETE request
    pub async fn delete(&self, path: &str) -> Result<StatusCode> {
        let url = format!("{}{}", self.config.base_url, path);
        let mut req = self.http_client.delete(&url);
        
        if let Some(token) = self.auth_token.lock().await.as_ref() {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = req.send().await.context("Failed to send DELETE request")?;
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("DELETE {} failed with status {}: {}", path, status, error_text);
        }
        
        Ok(status)
    }

    /// Register a new user (when auth is enabled)
    pub async fn register(&self, username: &str, password: &str) -> Result<Value> {
        let body = serde_json::json!({
            "username": username,
            "password": password
        });
        
        self.post("/api/v1/auth/register", &body).await
    }

    /// Login and store auth token
    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        let body = serde_json::json!({
            "username": username,
            "password": password
        });
        
        let response: Value = self.post("/api/v1/auth/login", &body).await?;
        
        if let Some(token) = response.get("token").and_then(|t| t.as_str()) {
            *self.auth_token.lock().await = Some(token.to_string());
            Ok(())
        } else {
            anyhow::bail!("Login response missing token")
        }
    }

    /// Connect to WebSocket endpoint
    pub async fn connect_websocket(&self, path: &str) -> Result<WebSocketTestClient> {
        let url = format!("{}{}", self.config.ws_url, path);
        let (ws_stream, _) = connect_async(&url)
            .await
            .context("Failed to connect to WebSocket")?;
        
        Ok(WebSocketTestClient::new(ws_stream))
    }

    /// Wait for server to be ready
    pub async fn wait_for_server(&self) -> Result<()> {
        let start = tokio::time::Instant::now();
        let max_wait = Duration::from_secs(30);
        
        loop {
            match self.http_client.get(&format!("{}/health", self.config.base_url)).send().await {
                Ok(response) if response.status().is_success() => {
                    println!("Server ready after {:?}", start.elapsed());
                    return Ok(());
                }
                _ => {
                    if start.elapsed() > max_wait {
                        anyhow::bail!("Server failed to start within {:?}", max_wait);
                    }
                    sleep(Duration::from_millis(500)).await;
                }
            }
        }
    }
}

/// WebSocket test client
pub struct WebSocketTestClient {
    ws_stream: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
}

impl WebSocketTestClient {
    fn new(ws_stream: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>) -> Self {
        Self { ws_stream }
    }

    /// Send a message
    pub async fn send(&mut self, message: &Value) -> Result<()> {
        let msg = Message::Text(serde_json::to_string(message)?);
        self.ws_stream.send(msg).await.context("Failed to send WebSocket message")?;
        Ok(())
    }

    /// Receive a message with timeout
    pub async fn receive(&mut self) -> Result<Value> {
        let msg = timeout(Duration::from_secs(5), self.ws_stream.next())
            .await
            .context("WebSocket receive timeout")?
            .context("WebSocket stream ended")?
            .context("Failed to receive WebSocket message")?;
        
        match msg {
            Message::Text(text) => serde_json::from_str(&text).context("Failed to parse WebSocket message"),
            _ => anyhow::bail!("Unexpected WebSocket message type"),
        }
    }

    /// Close the connection
    pub async fn close(mut self) -> Result<()> {
        self.ws_stream.close(None).await.context("Failed to close WebSocket")?;
        Ok(())
    }
}

/// Test server runner
pub struct TestServer {
    port: u16,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl TestServer {
    /// Start a test server on a random port
    pub async fn start() -> Result<(Self, TestConfig)> {
        // Find available port
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        drop(listener);

        // Start server in background
        let handle = tokio::spawn(async move {
            // This would normally start your actual server
            // For testing, we'll simulate it
            std::env::set_var("HAL9_PORT", port.to_string());
            // hal9_server::run_server().await;
        });

        let config = TestConfig {
            base_url: format!("http://localhost:{}", port),
            ws_url: format!("ws://localhost:{}", port),
            ..Default::default()
        };

        Ok((Self { port, handle: Some(handle) }, config))
    }

    /// Stop the test server
    pub async fn stop(mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

/// Test assertions
pub struct Assertions;

impl Assertions {
    /// Assert response contains expected fields
    pub fn assert_fields(response: &Value, required_fields: &[&str]) -> Result<()> {
        for field in required_fields {
            if response.get(field).is_none() {
                anyhow::bail!("Response missing required field: {}", field);
            }
        }
        Ok(())
    }

    /// Assert response matches schema
    pub fn assert_schema(response: &Value, expected_type: &str) -> Result<()> {
        match expected_type {
            "neuron" => Self::assert_fields(response, &["id", "layer", "position", "processing_speed"]),
            "signal" => Self::assert_fields(response, &["id", "source_id", "pattern", "intensity"]),
            "consciousness" => Self::assert_fields(response, &["integration", "coherence", "resonance", "emergence"]),
            _ => anyhow::bail!("Unknown schema type: {}", expected_type),
        }
    }

    /// Assert array length
    pub fn assert_array_length(response: &Value, min: Option<usize>, max: Option<usize>) -> Result<()> {
        if let Some(array) = response.as_array() {
            if let Some(min_len) = min {
                if array.len() < min_len {
                    anyhow::bail!("Array length {} is less than minimum {}", array.len(), min_len);
                }
            }
            if let Some(max_len) = max {
                if array.len() > max_len {
                    anyhow::bail!("Array length {} is greater than maximum {}", array.len(), max_len);
                }
            }
            Ok(())
        } else {
            anyhow::bail!("Response is not an array");
        }
    }
}

/// Test fixtures
pub struct Fixtures;

impl Fixtures {
    /// Create test neuron data
    pub fn neuron() -> Value {
        serde_json::json!({
            "layer": 3,
            "position": [0.5, 0.5, 0.5],
            "processing_speed": 1.0,
            "complexity_threshold": 0.7
        })
    }

    /// Create test signal data
    pub fn signal() -> Value {
        serde_json::json!({
            "pattern": [0.1, 0.2, 0.3, 0.4, 0.5],
            "intensity": 0.8
        })
    }

    /// Create test user data
    pub fn user(username: &str) -> Value {
        serde_json::json!({
            "username": username,
            "password": "test_password_123"
        })
    }
}

/// Performance test utilities
pub struct PerfTest {
    pub name: String,
    pub start: tokio::time::Instant,
    pub measurements: Vec<Duration>,
}

impl PerfTest {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start: tokio::time::Instant::now(),
            measurements: Vec::new(),
        }
    }

    pub fn record(&mut self) {
        self.measurements.push(self.start.elapsed());
        self.start = tokio::time::Instant::now();
    }

    pub fn summary(&self) -> String {
        if self.measurements.is_empty() {
            return format!("{}: No measurements", self.name);
        }

        let total: Duration = self.measurements.iter().sum();
        let avg = total / self.measurements.len() as u32;
        let min = self.measurements.iter().min().unwrap();
        let max = self.measurements.iter().max().unwrap();

        format!(
            "{}: {} ops, avg={:?}, min={:?}, max={:?}, total={:?}",
            self.name,
            self.measurements.len(),
            avg,
            min,
            max,
            total
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_framework_initialization() {
        let config = TestConfig::default();
        let client = E2ETestClient::new(config);
        assert!(client.auth_token.lock().await.is_none());
    }

    #[tokio::test]
    async fn test_assertions() {
        let response = serde_json::json!({
            "id": "test-123",
            "layer": 3,
            "position": [0.5, 0.5, 0.5],
            "processing_speed": 1.0
        });

        assert!(Assertions::assert_fields(&response, &["id", "layer"]).is_ok());
        assert!(Assertions::assert_fields(&response, &["missing_field"]).is_err());
        assert!(Assertions::assert_schema(&response, "neuron").is_ok());
    }

    #[tokio::test]
    async fn test_fixtures() {
        let neuron = Fixtures::neuron();
        assert!(neuron.get("layer").is_some());
        assert!(neuron.get("position").is_some());

        let signal = Fixtures::signal();
        assert!(signal.get("pattern").is_some());
        assert!(signal.get("intensity").is_some());
    }

    #[tokio::test]
    async fn test_perf_measurements() {
        let mut perf = PerfTest::new("test_operation");
        
        sleep(Duration::from_millis(10)).await;
        perf.record();
        
        sleep(Duration::from_millis(20)).await;
        perf.record();
        
        assert_eq!(perf.measurements.len(), 2);
        let summary = perf.summary();
        assert!(summary.contains("test_operation"));
        assert!(summary.contains("2 ops"));
    }
}