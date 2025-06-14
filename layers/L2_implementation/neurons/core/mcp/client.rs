//! MCP Client implementation for wrapper/orchestrator

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::Result;
use super::protocol::*;

/// MCP Client trait
#[async_trait]
pub trait MCPClient: Send + Sync {
    /// Connect to an MCP server
    async fn connect(&mut self, server_id: &str) -> Result<()>;
    
    /// Disconnect from server
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Send a request and wait for response
    async fn request(&mut self, method: String, params: Value) -> Result<Value>;
    
    /// Send a notification (no response expected)
    async fn notify(&mut self, method: String, params: Value) -> Result<()>;
    
    /// Call a tool on the connected server
    async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value>;
    
    /// Read a resource from the server
    async fn read_resource(&mut self, uri: &str) -> Result<Value>;
}

/// Wrapper-side MCP client for connecting to neurons
pub struct WrapperMCPClient {
    server_id: Option<String>,
    request_id: u64,
    pending_requests: HashMap<String, mpsc::Sender<Result<Value>>>,
    capabilities: Option<NeuronCapability>,
}

impl WrapperMCPClient {
    pub fn new() -> Self {
        Self {
            server_id: None,
            request_id: 0,
            pending_requests: HashMap::new(),
            capabilities: None,
        }
    }
    
    /// Get next request ID
    fn next_request_id(&mut self) -> String {
        self.request_id += 1;
        self.request_id.to_string()
    }
    
    /// Process a task through the connected neuron
    pub async fn process_task(
        &mut self,
        content: String,
        context: TaskContext,
    ) -> Result<ProcessTaskResponse> {
        let params = serde_json::to_value(ProcessTaskRequest {
            task_id: Uuid::new_v4(),
            parent_task_id: None,
            content,
            context,
        })?;
        
        let result = self.request("neuron/processTask".to_string(), params).await?;
        let response: ProcessTaskResponse = serde_json::from_value(result)?;
        Ok(response)
    }
    
    /// Get capabilities of connected neuron
    pub async fn get_capabilities(&mut self) -> Result<NeuronCapability> {
        if let Some(cap) = &self.capabilities {
            return Ok(cap.clone());
        }
        
        let result = self.request("initialize".to_string(), Value::Null).await?;
        let init_response: serde_json::Value = result;
        
        if let Some(capabilities) = init_response.get("capabilities") {
            let cap: NeuronCapability = serde_json::from_value(capabilities.clone())?;
            self.capabilities = Some(cap.clone());
            Ok(cap)
        } else {
            Err(crate::Error::Protocol("No capabilities in initialize response".to_string()))
        }
    }
}

#[async_trait]
impl MCPClient for WrapperMCPClient {
    async fn connect(&mut self, server_id: &str) -> Result<()> {
        self.server_id = Some(server_id.to_string());
        
        // Initialize connection
        let _init_result = self.request("initialize".to_string(), serde_json::json!({
            "protocolVersion": "2024-11-05",
            "clientInfo": {
                "name": "2HAL9 Wrapper",
                "version": "1.0.0"
            }
        })).await?;
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        self.server_id = None;
        self.capabilities = None;
        self.pending_requests.clear();
        Ok(())
    }
    
    async fn request(&mut self, method: String, params: Value) -> Result<Value> {
        let id = self.next_request_id();
        let message = MCPMessage::request(id.clone(), method, params);
        
        // In a real implementation, this would send over a transport
        // For now, we'll simulate the response
        let response = self.simulate_response(message).await?;
        
        match response {
            MCPMessage::V2 { content: MCPContent::Response(resp) } => {
                match resp.result {
                    ResponseResult::Success { result } => Ok(result),
                    ResponseResult::Error { error } => {
                        Err(crate::Error::Protocol(format!("MCP Error {}: {}", error.code, error.message)))
                    }
                }
            }
            _ => Err(crate::Error::Protocol("Invalid response type".to_string())),
        }
    }
    
    async fn notify(&mut self, method: String, params: Value) -> Result<()> {
        let _message = MCPMessage::notification(method, params);
        // Send notification (no response expected)
        Ok(())
    }
    
    async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value> {
        self.request("tools/call".to_string(), serde_json::json!({
            "name": name,
            "arguments": arguments,
        })).await
    }
    
    async fn read_resource(&mut self, uri: &str) -> Result<Value> {
        self.request("resources/read".to_string(), serde_json::json!({
            "uri": uri,
        })).await
    }
}

impl WrapperMCPClient {
    /// Simulate server response for testing
    async fn simulate_response(&self, request: MCPMessage) -> Result<MCPMessage> {
        // In a real implementation, this would communicate with actual neuron servers
        // For now, return mock responses
        match request {
            MCPMessage::V2 { content: MCPContent::Request(req) } => {
                match req.method.as_str() {
                    "initialize" => {
                        Ok(MCPMessage::response(req.id, serde_json::json!({
                            "protocolVersion": "2024-11-05",
                            "capabilities": {
                                "id": "mock-neuron",
                                "name": "Mock Neuron",
                                "version": "1.0.0",
                                "layer": "L4",
                                "description": "Mock neuron for testing",
                                "tools": ["process_task"],
                                "resources": ["neuron://mock/status"],
                                "supports_batch": true,
                                "max_batch_size": 10
                            }
                        })))
                    }
                    _ => {
                        Ok(MCPMessage::error(req.id, error_codes::METHOD_NOT_FOUND,
                            format!("Method '{}' not found", req.method)))
                    }
                }
            }
            _ => Err(crate::Error::Protocol("Invalid request".to_string())),
        }
    }
}

/// Connection manager for multiple neuron connections
pub struct MCPConnectionManager {
    connections: HashMap<String, Box<dyn MCPClient>>,
}

impl MCPConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }
    
    /// Add a new connection
    pub async fn add_connection(&mut self, neuron_id: String, mut client: Box<dyn MCPClient>) -> Result<()> {
        client.connect(&neuron_id).await?;
        self.connections.insert(neuron_id, client);
        Ok(())
    }
    
    /// Get a connection by neuron ID
    pub fn get_connection(&mut self, neuron_id: &str) -> Option<&mut Box<dyn MCPClient>> {
        self.connections.get_mut(neuron_id)
    }
    
    /// Remove and disconnect a connection
    pub async fn remove_connection(&mut self, neuron_id: &str) -> Result<()> {
        if let Some(mut client) = self.connections.remove(neuron_id) {
            client.disconnect().await?;
        }
        Ok(())
    }
}