//! MCP tools for browser automation

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use hal9_core::mcp::tools::{Tool, ToolDefinition};
use hal9_core::{Result, Error};
use crate::BrowserController;
use crate::controller::{BrowserAction, WaitCondition, ExtractType};

/// Base trait for browser tools
#[async_trait]
trait BrowserTool: Tool {
    /// Get browser controller
    async fn get_controller(&self) -> Arc<BrowserController>;
}

/// Navigate to URL tool
pub struct NavigateTool {
    controller: Arc<BrowserController>,
}

impl NavigateTool {
    pub fn new(controller: Arc<BrowserController>) -> Self {
        Self { controller }
    }
}

#[async_trait]
impl Tool for NavigateTool {
    fn name(&self) -> &str {
        "browser_navigate"
    }
    
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "browser_navigate".to_string(),
            description: "Navigate browser to a URL".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to navigate to"
                    }
                },
                "required": ["url"]
            })
        }
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        let url = params["url"].as_str()
            .ok_or_else(|| Error::InvalidInput("URL parameter is required".to_string()))?;
        
        let action = BrowserAction::Navigate { 
            url: url.to_string() 
        };
        
        match self.controller.execute_action(action).await {
            Ok(result) => Ok(json!({
                "status": "success",
                "result": result
            })),
            Err(e) => Err(Error::ToolExecution(format!("Navigation failed: {}", e))),
        }
    }
}

/// Click element tool
pub struct ClickTool {
    controller: Arc<BrowserController>,
}

impl ClickTool {
    pub fn new(controller: Arc<BrowserController>) -> Self {
        Self { controller }
    }
}

#[async_trait]
impl Tool for ClickTool {
    fn name(&self) -> &str {
        "browser_click"
    }
    
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "browser_click".to_string(),
            description: "Click an element on the page".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "selector": {
                        "type": "string",
                        "description": "CSS selector or XPath of the element to click"
                    }
                },
                "required": ["selector"]
            })
        }
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        let selector = params["selector"].as_str()
            .ok_or_else(|| Error::InvalidInput("Selector parameter is required".to_string()))?;
        
        let action = BrowserAction::Click { 
            selector: selector.to_string() 
        };
        
        match self.controller.execute_action(action).await {
            Ok(result) => Ok(json!({
                "status": "success",
                "result": result
            })),
            Err(e) => Err(Error::ToolExecution(format!("Click failed: {}", e))),
        }
    }
}

/// Type text tool
pub struct TypeTool {
    controller: Arc<BrowserController>,
}

impl TypeTool {
    pub fn new(controller: Arc<BrowserController>) -> Self {
        Self { controller }
    }
}

#[async_trait]
impl Tool for TypeTool {
    fn name(&self) -> &str {
        "browser_type"
    }
    
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "browser_type".to_string(),
            description: "Type text into an input field".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "selector": {
                        "type": "string",
                        "description": "CSS selector or XPath of the input field"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text to type"
                    }
                },
                "required": ["selector", "text"]
            })
        }
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        let selector = params["selector"].as_str()
            .ok_or_else(|| Error::InvalidInput("Selector parameter is required".to_string()))?;
        let text = params["text"].as_str()
            .ok_or_else(|| Error::InvalidInput("Text parameter is required".to_string()))?;
        
        let action = BrowserAction::Type { 
            selector: selector.to_string(),
            text: text.to_string(),
        };
        
        match self.controller.execute_action(action).await {
            Ok(result) => Ok(json!({
                "status": "success",
                "result": result
            })),
            Err(e) => Err(Error::ToolExecution(format!("Type failed: {}", e))),
        }
    }
}

/// Extract data tool
pub struct ExtractTool {
    controller: Arc<BrowserController>,
}

impl ExtractTool {
    pub fn new(controller: Arc<BrowserController>) -> Self {
        Self { controller }
    }
}

#[async_trait]
impl Tool for ExtractTool {
    fn name(&self) -> &str {
        "browser_extract"
    }
    
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "browser_extract".to_string(),
            description: "Extract data from page elements".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "selector": {
                        "type": "string",
                        "description": "CSS selector or XPath of elements to extract from"
                    },
                    "type": {
                        "type": "string",
                        "enum": ["text", "html", "attribute", "all_text"],
                        "description": "Type of data to extract",
                        "default": "text"
                    },
                    "attribute": {
                        "type": "string",
                        "description": "Attribute name (required when type is 'attribute')"
                    }
                },
                "required": ["selector"]
            })
        }
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        let selector = params["selector"].as_str()
            .ok_or_else(|| Error::InvalidInput("Selector parameter is required".to_string()))?;
        
        let extract_type = match params["type"].as_str() {
            Some("html") => ExtractType::Html,
            Some("attribute") => {
                let attr = params["attribute"].as_str()
                    .ok_or_else(|| Error::InvalidInput("Attribute parameter required for type 'attribute'".to_string()))?;
                ExtractType::Attribute(attr.to_string())
            }
            Some("all_text") => ExtractType::AllText,
            _ => ExtractType::Text,
        };
        
        let action = BrowserAction::Extract { 
            selector: selector.to_string(),
            extract_type,
        };
        
        match self.controller.execute_action(action).await {
            Ok(result) => Ok(json!({
                "status": "success",
                "result": result
            })),
            Err(e) => Err(Error::ToolExecution(format!("Extract failed: {}", e))),
        }
    }
}

/// Screenshot tool
pub struct ScreenshotTool {
    controller: Arc<BrowserController>,
}

impl ScreenshotTool {
    pub fn new(controller: Arc<BrowserController>) -> Self {
        Self { controller }
    }
}

#[async_trait]
impl Tool for ScreenshotTool {
    fn name(&self) -> &str {
        "browser_screenshot"
    }
    
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "browser_screenshot".to_string(),
            description: "Take a screenshot of the current page".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "full_page": {
                        "type": "boolean",
                        "description": "Whether to capture the full page or just the viewport",
                        "default": false
                    }
                }
            })
        }
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        let full_page = params["full_page"].as_bool().unwrap_or(false);
        
        let action = BrowserAction::Screenshot { full_page };
        
        match self.controller.execute_action(action).await {
            Ok(result) => Ok(json!({
                "status": "success",
                "result": result
            })),
            Err(e) => Err(Error::ToolExecution(format!("Screenshot failed: {}", e))),
        }
    }
}

/// Wait for condition tool
pub struct WaitForTool {
    controller: Arc<BrowserController>,
}

impl WaitForTool {
    pub fn new(controller: Arc<BrowserController>) -> Self {
        Self { controller }
    }
}

#[async_trait]
impl Tool for WaitForTool {
    fn name(&self) -> &str {
        "browser_wait"
    }
    
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "browser_wait".to_string(),
            description: "Wait for a condition before proceeding".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "condition": {
                        "type": "string",
                        "enum": ["selector", "navigation", "duration"],
                        "description": "Type of condition to wait for"
                    },
                    "selector": {
                        "type": "string",
                        "description": "CSS selector (required when condition is 'selector')"
                    },
                    "duration": {
                        "type": "integer",
                        "description": "Duration in milliseconds (required when condition is 'duration')"
                    }
                },
                "required": ["condition"]
            })
        }
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        let condition_type = params["condition"].as_str()
            .ok_or_else(|| Error::InvalidInput("Condition parameter is required".to_string()))?;
        
        let condition = match condition_type {
            "selector" => {
                let selector = params["selector"].as_str()
                    .ok_or_else(|| Error::InvalidInput("Selector required for 'selector' condition".to_string()))?;
                WaitCondition::Selector(selector.to_string())
            }
            "navigation" => WaitCondition::Navigation,
            "duration" => {
                let duration = params["duration"].as_u64()
                    .ok_or_else(|| Error::InvalidInput("Duration required for 'duration' condition".to_string()))?;
                WaitCondition::Duration(duration)
            }
            _ => return Err(Error::InvalidInput("Invalid condition type".to_string())),
        };
        
        let action = BrowserAction::WaitFor { condition };
        
        match self.controller.execute_action(action).await {
            Ok(result) => Ok(json!({
                "status": "success",
                "result": result
            })),
            Err(e) => Err(Error::ToolExecution(format!("Wait failed: {}", e))),
        }
    }
}

/// Create all browser tools
pub fn create_browser_tools(controller: Arc<BrowserController>) -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(NavigateTool::new(controller.clone())),
        Box::new(ClickTool::new(controller.clone())),
        Box::new(TypeTool::new(controller.clone())),
        Box::new(ExtractTool::new(controller.clone())),
        Box::new(ScreenshotTool::new(controller.clone())),
        Box::new(WaitForTool::new(controller)),
    ]
}

#[cfg(test)]
mod tests {
    

    #[test]
    #[ignore = "Requires proper BrowserController mock"]
    fn test_navigate_tool_schema() {
        // TODO: Create proper mock for BrowserController
        // let controller = Arc::new(MockBrowserController::new());
        // let tool = NavigateTool::new(controller);
        // 
        // assert_eq!(tool.name(), "browser_navigate");
        // let definition = tool.definition();
        // assert!(definition.input_schema["properties"]["url"].is_object());
    }
}