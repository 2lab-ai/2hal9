//! MCP Server implementation for neurons

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use super::protocol::*;
use super::tools::{Tool, ToolDefinition};
use crate::Result;

/// MCP Server trait for neurons
#[async_trait]
pub trait MCPServer: Send + Sync {
    /// Get server capabilities
    async fn get_capabilities(&self) -> Result<NeuronCapability>;

    /// List available tools
    async fn list_tools(&self) -> Result<Vec<ToolDefinition>>;

    /// Execute a tool
    async fn call_tool(&self, name: &str, params: Value) -> Result<Value>;

    /// List available resources
    async fn list_resources(&self) -> Result<Vec<Resource>>;

    /// Read a resource
    async fn read_resource(&self, uri: &str) -> Result<Value>;

    /// Process incoming MCP message
    async fn handle_message(&self, message: MCPMessage) -> Result<MCPMessage>;
}

/// Neuron-specific MCP server implementation
pub struct NeuronMCPServer {
    neuron_id: String,
    layer: String,
    tools: HashMap<String, Box<dyn Tool>>,
    resources: HashMap<String, Resource>,
}

impl NeuronMCPServer {
    pub fn new(neuron_id: String, layer: String) -> Self {
        Self {
            neuron_id,
            layer,
            tools: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register_tool(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Register a resource
    pub fn register_resource(&mut self, resource: Resource) {
        self.resources.insert(resource.uri.clone(), resource);
    }

    /// Process a task (main neuron function)
    async fn process_task(&self, request: ProcessTaskRequest) -> Result<ProcessTaskResponse> {
        // This would be implemented by specific neuron types
        // For now, return a mock response
        let subtasks = match self.layer.as_str() {
            "L4" => {
                // L4 generates 2 L3 tasks
                vec![
                    SubTask {
                        id: Uuid::new_v4(),
                        content: format!("Design architecture for: {}", request.content),
                        target_neuron: "neuron-2".to_string(),
                        target_layer: "L3".to_string(),
                    },
                    SubTask {
                        id: Uuid::new_v4(),
                        content: format!("Plan interface for: {}", request.content),
                        target_neuron: "neuron-3".to_string(),
                        target_layer: "L3".to_string(),
                    },
                ]
            }
            "L3" => {
                // L3 generates 2 L2 tasks
                let base = if request.content.contains("architecture") {
                    vec!["Implement data model", "Implement business logic"]
                } else {
                    vec!["Implement API endpoints", "Implement validation"]
                };

                base.iter()
                    .map(|&task| SubTask {
                        id: Uuid::new_v4(),
                        content: format!("{} for: {}", task, request.content),
                        target_neuron: "neuron-4".to_string(),
                        target_layer: "L2".to_string(),
                    })
                    .collect()
            }
            "L2" => {
                // L2 doesn't generate subtasks, just processes
                vec![]
            }
            _ => vec![],
        };

        Ok(ProcessTaskResponse {
            task_id: request.task_id,
            subtasks,
            status: TaskStatus::Success,
            processing_time_ms: 100,
        })
    }
}

#[async_trait]
impl MCPServer for NeuronMCPServer {
    async fn get_capabilities(&self) -> Result<NeuronCapability> {
        Ok(NeuronCapability {
            id: self.neuron_id.clone(),
            name: format!("Neuron {} ({})", self.neuron_id, self.layer),
            version: "1.0.0".to_string(),
            layer: self.layer.clone(),
            description: format!("{} layer neuron for task processing", self.layer),
            tools: self.tools.keys().cloned().collect(),
            resources: self.resources.keys().cloned().collect(),
            supports_batch: true,
            max_batch_size: Some(10),
        })
    }

    async fn list_tools(&self) -> Result<Vec<ToolDefinition>> {
        Ok(self.tools.values().map(|tool| tool.definition()).collect())
    }

    async fn call_tool(&self, name: &str, params: Value) -> Result<Value> {
        match self.tools.get(name) {
            Some(tool) => tool.execute(params).await,
            None => Err(crate::Error::NotFound(format!("Tool '{}' not found", name))),
        }
    }

    async fn list_resources(&self) -> Result<Vec<Resource>> {
        Ok(self.resources.values().cloned().collect())
    }

    async fn read_resource(&self, uri: &str) -> Result<Value> {
        match self.resources.get(uri) {
            Some(_resource) => {
                // Return resource data based on URI
                Ok(serde_json::json!({
                    "uri": uri,
                    "content": format!("Resource content for {}", uri),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }))
            }
            None => Err(crate::Error::NotFound(format!(
                "Resource '{}' not found",
                uri
            ))),
        }
    }

    async fn handle_message(&self, message: MCPMessage) -> Result<MCPMessage> {
        match message {
            MCPMessage::V2 {
                content: MCPContent::Request(req),
            } => match req.method.as_str() {
                "initialize" => Ok(MCPMessage::response(
                    req.id,
                    serde_json::json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": self.get_capabilities().await?,
                    }),
                )),
                "tools/list" => Ok(MCPMessage::response(
                    req.id,
                    serde_json::json!({
                        "tools": self.list_tools().await?,
                    }),
                )),
                "tools/call" => {
                    let name =
                        req.params
                            .get("name")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                crate::Error::InvalidInput("Missing tool name".to_string())
                            })?;
                    let params = req.params.get("arguments").cloned().unwrap_or(Value::Null);

                    match self.call_tool(name, params).await {
                        Ok(result) => Ok(MCPMessage::response(req.id, result)),
                        Err(e) => Ok(MCPMessage::error(
                            req.id,
                            error_codes::INTERNAL_ERROR,
                            e.to_string(),
                        )),
                    }
                }
                "resources/list" => Ok(MCPMessage::response(
                    req.id,
                    serde_json::json!({
                        "resources": self.list_resources().await?,
                    }),
                )),
                "resources/read" => {
                    let uri = req
                        .params
                        .get("uri")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            crate::Error::InvalidInput("Missing resource URI".to_string())
                        })?;

                    match self.read_resource(uri).await {
                        Ok(content) => Ok(MCPMessage::response(req.id, content)),
                        Err(e) => Ok(MCPMessage::error(
                            req.id,
                            error_codes::INTERNAL_ERROR,
                            e.to_string(),
                        )),
                    }
                }
                "neuron/processTask" => {
                    let task_req: ProcessTaskRequest = serde_json::from_value(req.params)
                        .map_err(|e| crate::Error::InvalidInput(e.to_string()))?;

                    match self.process_task(task_req).await {
                        Ok(result) => {
                            Ok(MCPMessage::response(req.id, serde_json::to_value(result)?))
                        }
                        Err(e) => Ok(MCPMessage::error(
                            req.id,
                            error_codes::INTERNAL_ERROR,
                            e.to_string(),
                        )),
                    }
                }
                _ => Ok(MCPMessage::error(
                    req.id,
                    error_codes::METHOD_NOT_FOUND,
                    format!("Method '{}' not found", req.method),
                )),
            },
            _ => {
                // Handle other message types if needed
                Ok(MCPMessage::error(
                    "0".to_string(),
                    error_codes::INVALID_REQUEST,
                    "Only request messages are supported".to_string(),
                ))
            }
        }
    }
}
