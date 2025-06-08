//! Tool definitions for MCP

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Result;

/// Tool definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ToolContent>,
}

/// Tool content item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { 
        data: String,  // base64 encoded
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource { 
        uri: String,
        mime_type: String,
    },
}

/// Tool trait for implementing tools
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get tool name
    fn name(&self) -> &str;
    
    /// Get tool definition
    fn definition(&self) -> ToolDefinition;
    
    /// Execute the tool
    async fn execute(&self, params: Value) -> Result<Value>;
}

/// Process task tool for neurons
pub struct ProcessTaskTool {
    layer: String,
}

impl ProcessTaskTool {
    pub fn new(layer: String) -> Self {
        Self { layer }
    }
}

#[async_trait]
impl Tool for ProcessTaskTool {
    fn name(&self) -> &str {
        "process_task"
    }
    
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "process_task".to_string(),
            description: format!("Process a task at {} level", self.layer),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "task": {
                        "type": "string",
                        "description": "The task content to process"
                    },
                    "context": {
                        "type": "object",
                        "description": "Additional context for processing"
                    }
                },
                "required": ["task"]
            }),
        }
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        let task = params.get("task")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::Error::InvalidInput("Missing task parameter".to_string()))?;
        
        // Process based on layer
        let result = match self.layer.as_str() {
            "L4" => {
                // Strategic decomposition
                vec![
                    format!("Design architecture for: {}", task),
                    format!("Plan interface for: {}", task),
                ]
            }
            "L3" => {
                // Design decomposition
                if task.contains("architecture") {
                    vec![
                        format!("Implement data model for: {}", task),
                        format!("Implement business logic for: {}", task),
                    ]
                } else {
                    vec![
                        format!("Implement API for: {}", task),
                        format!("Implement validation for: {}", task),
                    ]
                }
            }
            "L2" => {
                // Implementation - return code or detailed steps
                vec![format!("Implementation code for: {}", task)]
            }
            _ => vec![format!("Processed: {}", task)]
        };
        
        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": result.join("\n")
            }],
            "subtasks": result
        }))
    }
}

/// Status tool for getting neuron status
pub struct StatusTool {
    neuron_id: String,
}

impl StatusTool {
    pub fn new(neuron_id: String) -> Self {
        Self { neuron_id }
    }
}

#[async_trait]
impl Tool for StatusTool {
    fn name(&self) -> &str {
        "get_status"
    }
    
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "get_status".to_string(),
            description: "Get current neuron status and metrics".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }
    
    async fn execute(&self, _params: Value) -> Result<Value> {
        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": format!("Neuron {} is operational", self.neuron_id)
            }],
            "status": {
                "neuron_id": self.neuron_id,
                "state": "active",
                "tasks_processed": 42,
                "average_processing_time_ms": 150,
                "memory_usage_mb": 32,
                "uptime_seconds": 3600
            }
        }))
    }
}