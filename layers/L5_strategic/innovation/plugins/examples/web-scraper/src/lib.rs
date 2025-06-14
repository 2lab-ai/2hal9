use hal9_plugin_sdk::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Define plugin metadata
hal9_plugin! {
    metadata: {
        name: "Web Scraper",
        version: "0.1.0",
        author: "HAL9 Developers",
        description: "Extract structured data from web pages",
        license: "MIT",
    },
    capabilities: [
        PluginCapability::ToolProvider {
            tool_name: "scrape_url".to_string(),
            tool_description: "Extract data from a web page".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "url".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "URL to scrape".to_string(),
                    default: None,
                },
                ToolParameter {
                    name: "selector".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    description: "CSS selector or regex pattern".to_string(),
                    default: Some(serde_json::json!("body")),
                },
                ToolParameter {
                    name: "extract_links".to_string(),
                    param_type: "boolean".to_string(),
                    required: false,
                    description: "Extract all links from page".to_string(),
                    default: Some(serde_json::json!(false)),
                },
                ToolParameter {
                    name: "extract_images".to_string(),
                    param_type: "boolean".to_string(),
                    required: false,
                    description: "Extract all image URLs".to_string(),
                    default: Some(serde_json::json!(false)),
                },
            ],
        },
        PluginCapability::ToolProvider {
            tool_name: "extract_emails".to_string(),
            tool_description: "Extract email addresses from text".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "text".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Text to search for emails".to_string(),
                    default: None,
                },
            ],
        },
    ],
    permissions: [
        Permission::NetworkHttps,
        Permission::Hal9Memory,
    ]
}

// Scraping result
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScrapeResult {
    url: String,
    title: Option<String>,
    content: String,
    links: Vec<String>,
    images: Vec<String>,
    metadata: HashMap<String, String>,
}

// Main plugin struct
pub struct WebScraperTool {
    config: ToolConfig,
    email_regex: Regex,
    url_regex: Regex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolConfig {
    timeout_ms: u64,
    max_content_length: usize,
    user_agent: String,
}

impl Default for WebScraperTool {
    fn default() -> Self {
        Self {
            config: ToolConfig {
                timeout_ms: 5000,
                max_content_length: 1_000_000, // 1MB
                user_agent: "HAL9 Web Scraper/1.0".to_string(),
            },
            email_regex: Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
            url_regex: Regex::new(r"https?://[^\s<>"']+").unwrap(),
        }
    }
}

impl WebScraperTool {
    fn scrape_url(&self, url: &str, params: &HashMap<String, serde_json::Value>) -> Result<ScrapeResult, PluginError> {
        // NOTE: In a real implementation, we would use an HTTP client
        // For this example, we'll simulate the scraping
        log_info(&format!("Scraping URL: {}", url));
        
        // Simulate network delay
        // In real WASM, we'd need to use host functions for network access
        
        // For demo, return mock data
        let mut result = ScrapeResult {
            url: url.to_string(),
            title: Some("Example Page Title".to_string()),
            content: "This is the page content. It contains some text and links.".to_string(),
            links: vec![],
            images: vec![],
            metadata: HashMap::new(),
        };
        
        // Check if we should extract links
        if params.get("extract_links")
            .and_then(|v| v.as_bool())
            .unwrap_or(false) {
            result.links = vec![
                "https://example.com/page1".to_string(),
                "https://example.com/page2".to_string(),
            ];
        }
        
        // Check if we should extract images
        if params.get("extract_images")
            .and_then(|v| v.as_bool())
            .unwrap_or(false) {
            result.images = vec![
                "https://example.com/image1.jpg".to_string(),
                "https://example.com/image2.png".to_string(),
            ];
        }
        
        // Add some metadata
        result.metadata.insert("scraped_at".to_string(), current_timestamp().to_string());
        result.metadata.insert("content_length".to_string(), result.content.len().to_string());
        
        Ok(result)
    }
    
    fn extract_emails(&self, text: &str) -> Vec<String> {
        self.email_regex
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }
}

// Implement ToolPlugin trait
impl ToolPlugin for WebScraperTool {
    fn execute(&mut self, params: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, PluginError> {
        // Determine which tool to execute
        let tool_name = params.get("_tool_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| plugin_error(ErrorCode::InvalidInput, "Missing tool name"))?;
        
        match tool_name {
            "scrape_url" => {
                let url = params.get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| plugin_error(ErrorCode::InvalidInput, "Missing URL parameter"))?;
                
                let result = self.scrape_url(url, &params)?;
                serde_json::to_value(result)
                    .map_err(|e| plugin_error(ErrorCode::InternalError, format!("Serialization error: {}", e)))
            }
            "extract_emails" => {
                let text = params.get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| plugin_error(ErrorCode::InvalidInput, "Missing text parameter"))?;
                
                let emails = self.extract_emails(text);
                Ok(serde_json::json!({
                    "emails": emails,
                    "count": emails.len()
                }))
            }
            _ => err(ErrorCode::InvalidInput, format!("Unknown tool: {}", tool_name)),
        }
    }
    
    fn validate_params(&self, params: &HashMap<String, serde_json::Value>) -> Result<(), PluginError> {
        let tool_name = params.get("_tool_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| plugin_error(ErrorCode::InvalidInput, "Missing tool name"))?;
        
        match tool_name {
            "scrape_url" => {
                // Validate URL
                let url = params.get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| plugin_error(ErrorCode::InvalidInput, "Missing URL parameter"))?;
                
                if !url.starts_with("https://") && !url.starts_with("http://") {
                    return err(ErrorCode::InvalidInput, "URL must start with http:// or https://");
                }
                
                Ok(())
            }
            "extract_emails" => {
                // Validate text exists
                if !params.contains_key("text") {
                    return err(ErrorCode::InvalidInput, "Missing text parameter");
                }
                Ok(())
            }
            _ => err(ErrorCode::InvalidInput, format!("Unknown tool: {}", tool_name)),
        }
    }
}

// Implement lifecycle hooks
impl PluginLifecycle for WebScraperTool {
    fn on_load(&mut self, context: PluginContext) -> Result<(), PluginError> {
        log_info(&format!("Web Scraper Tool loaded with permissions: {:?}", context.permissions));
        
        // Check we have network permission
        if !context.permissions.contains(&Permission::NetworkHttps) {
            return err(ErrorCode::PermissionDenied, "Network access permission required");
        }
        
        Ok(())
    }
    
    fn on_activate(&mut self) -> Result<(), PluginError> {
        log_info("Web Scraper Tool activated");
        Ok(())
    }
    
    fn on_deactivate(&mut self) -> Result<(), PluginError> {
        log_info("Web Scraper Tool deactivated");
        Ok(())
    }
    
    fn on_unload(&mut self) -> Result<(), PluginError> {
        log_info("Web Scraper Tool unloaded");
        Ok(())
    }
}

// Export tool execution function
#[no_mangle]
pub extern "C" fn execute_tool(params_ptr: *const u8, params_len: usize) -> *mut u8 {
    unsafe {
        let params_bytes = std::slice::from_raw_parts(params_ptr, params_len);
        let params: HashMap<String, serde_json::Value> = match serde_json::from_slice(params_bytes) {
            Ok(p) => p,
            Err(_) => return std::ptr::null_mut(),
        };
        
        let mut tool = WebScraperTool::default();
        match tool.execute(params) {
            Ok(result) => {
                let json = serde_json::to_string(&result).unwrap();
                let bytes = json.into_bytes();
                let ptr = bytes.as_ptr() as *mut u8;
                std::mem::forget(bytes);
                ptr
            }
            Err(e) => {
                let error_json = serde_json::json!({
                    "error": {
                        "code": format!("{:?}", e.code),
                        "message": e.message
                    }
                });
                let json = serde_json::to_string(&error_json).unwrap();
                let bytes = json.into_bytes();
                let ptr = bytes.as_ptr() as *mut u8;
                std::mem::forget(bytes);
                ptr
            }
        }
    }
}