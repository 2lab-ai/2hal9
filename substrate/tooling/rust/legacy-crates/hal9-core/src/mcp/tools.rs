//! Tool definitions for MCP

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::{Result, Error};

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

/// Tool registry for managing available tools
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }
    
    /// Register a tool
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }
    
    /// Get available tool definitions
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values()
            .map(|tool| tool.definition())
            .collect()
    }
    
    /// Execute a tool
    pub async fn execute(&self, tool_name: &str, params: Value) -> Result<Value> {
        match self.tools.get(tool_name) {
            Some(tool) => tool.execute(params).await,
            None => Err(Error::ToolExecution(format!("Unknown tool: {}", tool_name))),
        }
    }
}

/// Filesystem read tool
pub struct FilesystemReadTool {
    allowed_paths: Vec<String>,
}

impl FilesystemReadTool {
    pub fn new(allowed_paths: Vec<String>) -> Self {
        Self { allowed_paths }
    }
    
    fn is_allowed_path(&self, path: &str) -> bool {
        // Check if path is under any allowed directory
        self.allowed_paths.iter().any(|allowed| path.starts_with(allowed))
    }
}

#[async_trait]
impl Tool for FilesystemReadTool {
    fn name(&self) -> &str {
        "filesystem_read"
    }
    
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "filesystem_read".to_string(),
            description: "Read a file from the filesystem".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path to read"
                    }
                },
                "required": ["path"]
            }),
        }
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        let path = params.get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| Error::ToolExecution("Missing 'path' parameter".to_string()))?;
        
        // Security check
        if !self.is_allowed_path(path) {
            return Err(Error::ToolExecution("Path not in allowed directories".to_string()));
        }
        
        // Read file
        match tokio::fs::read_to_string(path).await {
            Ok(content) => Ok(serde_json::json!({
                "content": [{
                    "type": "text",
                    "text": content
                }],
                "metadata": {
                    "path": path,
                    "size": content.len()
                }
            })),
            Err(e) => Err(Error::ToolExecution(format!("Failed to read file: {}", e))),
        }
    }
}

/// Filesystem write tool
pub struct FilesystemWriteTool {
    allowed_paths: Vec<String>,
}

impl FilesystemWriteTool {
    pub fn new(allowed_paths: Vec<String>) -> Self {
        Self { allowed_paths }
    }
    
    fn is_allowed_path(&self, path: &str) -> bool {
        self.allowed_paths.iter().any(|allowed| path.starts_with(allowed))
    }
}

#[async_trait]
impl Tool for FilesystemWriteTool {
    fn name(&self) -> &str {
        "filesystem_write"
    }
    
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "filesystem_write".to_string(),
            description: "Write content to a file".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write"
                    }
                },
                "required": ["path", "content"]
            }),
        }
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        let path = params.get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| Error::ToolExecution("Missing 'path' parameter".to_string()))?;
            
        let content = params.get("content")
            .and_then(|c| c.as_str())
            .ok_or_else(|| Error::ToolExecution("Missing 'content' parameter".to_string()))?;
        
        // Security check
        if !self.is_allowed_path(path) {
            return Err(Error::ToolExecution("Path not in allowed directories".to_string()));
        }
        
        // Write file
        match tokio::fs::write(path, content).await {
            Ok(_) => Ok(serde_json::json!({
                "content": [{
                    "type": "text",
                    "text": format!("Successfully wrote {} bytes to {}", content.len(), path)
                }],
                "metadata": {
                    "path": path,
                    "bytes_written": content.len()
                }
            })),
            Err(e) => Err(Error::ToolExecution(format!("Failed to write file: {}", e))),
        }
    }
}

/// Shell command execution tool
pub struct ShellTool {
    allowed_commands: Vec<String>,
}

impl ShellTool {
    pub fn new(allowed_commands: Vec<String>) -> Self {
        Self { allowed_commands }
    }
}

#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str {
        "shell_execute"
    }
    
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "shell_execute".to_string(),
            description: "Execute a shell command".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Command to execute"
                    },
                    "args": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Command arguments"
                    }
                },
                "required": ["command"]
            }),
        }
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        let command = params.get("command")
            .and_then(|c| c.as_str())
            .ok_or_else(|| Error::ToolExecution("Missing 'command' parameter".to_string()))?;
        
        // Security check
        if !self.allowed_commands.contains(&command.to_string()) {
            return Err(Error::ToolExecution(format!("Command '{}' not allowed", command)));
        }
        
        let args = params.get("args")
            .and_then(|a| a.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();
        
        // Execute command
        match tokio::process::Command::new(command)
            .args(&args)
            .output()
            .await
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);
                
                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": if exit_code == 0 { &stdout } else { &stderr }
                    }],
                    "metadata": {
                        "command": command,
                        "args": args,
                        "exit_code": exit_code,
                        "stdout": stdout,
                        "stderr": stderr
                    }
                }))
            }
            Err(e) => Err(Error::ToolExecution(format!("Failed to execute command: {}", e))),
        }
    }
}

/// Web fetch tool
pub struct WebFetchTool {
    allowed_domains: Option<Vec<String>>,
}

impl WebFetchTool {
    pub fn new(allowed_domains: Option<Vec<String>>) -> Self {
        Self { allowed_domains }
    }
    
    fn is_allowed_url(&self, url: &str) -> bool {
        match &self.allowed_domains {
            Some(domains) => domains.iter().any(|domain| url.contains(domain)),
            None => true, // No restrictions
        }
    }
}

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }
    
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "web_fetch".to_string(),
            description: "Fetch content from a web URL".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to fetch"
                    },
                    "headers": {
                        "type": "object",
                        "description": "Optional HTTP headers"
                    }
                },
                "required": ["url"]
            }),
        }
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        let url = params.get("url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| Error::ToolExecution("Missing 'url' parameter".to_string()))?;
        
        // Security check
        if !self.is_allowed_url(url) {
            return Err(Error::ToolExecution("URL not in allowed domains".to_string()));
        }
        
        // Create HTTP client
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| Error::ToolExecution(format!("Failed to create HTTP client: {}", e)))?;
            
        let mut request = client.get(url);
        
        // Add headers if provided
        if let Some(headers) = params.get("headers").and_then(|h| h.as_object()) {
            for (key, value) in headers {
                if let Some(val_str) = value.as_str() {
                    request = request.header(key, val_str);
                }
            }
        }
        
        // Execute request
        match request.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let content = response.text().await
                    .unwrap_or_else(|e| format!("Failed to read response: {}", e));
                
                Ok(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": content
                    }],
                    "metadata": {
                        "url": url,
                        "status": status
                    }
                }))
            }
            Err(e) => Err(Error::ToolExecution(format!("Failed to fetch URL: {}", e))),
        }
    }
}