//! Main browser controller implementation

use std::sync::Arc;
use tokio::sync::Mutex;
use crate::playwright_stub::{Browser, BrowserType, Playwright};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::{
    BrowserConfig, BrowserError, Result,
    context_pool::{ContextPool, PooledContext},
    security::SecuritySandbox,
    metrics::BrowserMetrics,
};

/// Main browser controller managing all automation operations
pub struct BrowserController {
    /// Playwright instance
    playwright: Playwright,
    
    /// Browser instance
    browser: Browser,
    
    /// Context pool for efficient reuse
    context_pool: Arc<Mutex<ContextPool>>,
    
    /// Security sandbox
    security_sandbox: Arc<SecuritySandbox>,
    
    /// Metrics collection
    metrics: Arc<BrowserMetrics>,
    
    /// Configuration
    config: BrowserConfig,
}

impl BrowserController {
    /// Create a new browser controller
    pub async fn new(config: BrowserConfig) -> Result<Self> {
        info!("Initializing browser controller");
        
        // Initialize playwright
        let playwright = Playwright::initialize().await
            .map_err(|e| BrowserError::Playwright(e.to_string()))?;
        
        // Launch browser
        let browser = Self::launch_browser(&playwright, &config).await?;
        
        // Create context pool
        let context_pool = Arc::new(Mutex::new(
            ContextPool::new(config.max_contexts, browser.clone(), config.clone())
        ));
        
        // Initialize security sandbox
        let security_sandbox = Arc::new(SecuritySandbox::new(config.security.clone()));
        
        // Initialize metrics
        let metrics = Arc::new(BrowserMetrics::new());
        
        Ok(Self {
            playwright,
            browser,
            context_pool,
            security_sandbox,
            metrics,
            config,
        })
    }
    
    /// Launch browser with configuration
    async fn launch_browser(playwright: &Playwright, config: &BrowserConfig) -> Result<Browser> {
        let browser_type = match config.browser_type.as_str() {
            "firefox" => playwright.firefox(),
            "webkit" => playwright.webkit(),
            _ => playwright.chromium(),
        };
        
        let browser = browser_type.launcher()
            .headless(config.headless)
            .launch()
            .await?;
        
        info!("Browser launched: {}", config.browser_type);
        Ok(browser)
    }
    
    /// Execute a browser action
    pub async fn execute_action(&self, action: BrowserAction) -> Result<ActionResult> {
        // Record metrics
        let start = std::time::Instant::now();
        self.metrics.record_action_start(&action);
        
        // Validate action security
        self.security_sandbox.validate_action(&action)?;
        
        // Acquire context from pool
        let mut pool = self.context_pool.lock().await;
        let context = pool.acquire().await?;
        drop(pool); // Release lock early
        
        // Execute action
        let result = match &action {
            BrowserAction::Navigate { url } => {
                self.navigate(context, url.clone()).await
            }
            BrowserAction::Click { selector } => {
                self.click(context, selector.clone()).await
            }
            BrowserAction::Type { selector, text } => {
                self.type_text(context, selector.clone(), text.clone()).await
            }
            BrowserAction::Extract { selector, extract_type } => {
                self.extract(context, selector.clone(), extract_type.clone()).await
            }
            BrowserAction::Screenshot { full_page } => {
                self.screenshot(context, *full_page).await
            }
            BrowserAction::WaitFor { condition } => {
                self.wait_for(context, condition.clone()).await
            }
        };
        
        // Record completion metrics
        let duration = start.elapsed();
        self.metrics.record_action_complete(&action, &result, duration);
        
        result
    }
    
    /// Navigate to URL
    async fn navigate(&self, context: PooledContext, url: String) -> Result<ActionResult> {
        debug!("Navigating to: {}", url);
        
        // Security check
        self.security_sandbox.validate_url(&url)?;
        
        let page = context.page();
        page.goto(&url)
            .await
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;
        
        let title = page.title().await.unwrap_or_default();
        
        Ok(ActionResult::Navigate { 
            url,
            title,
            status_code: 200, // TODO: Get actual status
        })
    }
    
    /// Click element
    async fn click(&self, context: PooledContext, selector: String) -> Result<ActionResult> {
        debug!("Clicking: {}", selector);
        
        let page = context.page();
        
        // Wait for element and click
        page.wait_for_selector(&selector)
            .await
            .map_err(|_| BrowserError::ElementNotFound(selector.clone()))?;
        
        page.click(&selector)
            .await
            .map_err(|e| BrowserError::Playwright(e.to_string()))?;
        
        Ok(ActionResult::Click { selector })
    }
    
    /// Type text into element
    async fn type_text(&self, context: PooledContext, selector: String, text: String) -> Result<ActionResult> {
        debug!("Typing into: {}", selector);
        
        let page = context.page();
        
        // Clear existing text and type new
        page.fill(&selector, &text)
            .await
            .map_err(|e| BrowserError::Playwright(e.to_string()))?;
        
        Ok(ActionResult::Type { selector, text })
    }
    
    /// Extract data from page
    async fn extract(&self, context: PooledContext, selector: String, extract_type: ExtractType) -> Result<ActionResult> {
        debug!("Extracting {} from: {}", extract_type, selector);
        
        let page = context.page();
        
        let data = match extract_type {
            ExtractType::Text => {
                page.text_content(&selector)
                    .await
                    .map_err(|e| BrowserError::Playwright(e.to_string()))?
                    .unwrap_or_default()
            }
            ExtractType::Html => {
                page.inner_html(&selector)
                    .await
                    .map_err(|e| BrowserError::Playwright(e.to_string()))?
            }
            ExtractType::Attribute(attr) => {
                page.get_attribute(&selector, &attr)
                    .await
                    .map_err(|e| BrowserError::Playwright(e.to_string()))?
                    .unwrap_or_default()
            }
            ExtractType::AllText => {
                // Extract all matching elements
                let elements = page.query_selector_all(&selector)
                    .await
                    .map_err(|e| BrowserError::Playwright(e.to_string()))?;
                
                let mut texts = Vec::new();
                for element in elements {
                    if let Ok(Some(text)) = element.text_content().await {
                        texts.push(text);
                    }
                }
                texts.join("\n")
            }
        };
        
        Ok(ActionResult::Extract { selector, data })
    }
    
    /// Take screenshot
    async fn screenshot(&self, context: PooledContext, full_page: bool) -> Result<ActionResult> {
        debug!("Taking screenshot (full_page: {})", full_page);
        
        let page = context.page();
        
        let screenshot_builder = page.screenshot().await;
        let screenshot_data = screenshot_builder
            .full_page(full_page)
            .r#await()
            .await
            .map_err(|e| BrowserError::Playwright(e.to_string()))?;
        
        // Encode as base64
        let base64_data = BASE64.encode(&screenshot_data);
        
        Ok(ActionResult::Screenshot { 
            data: base64_data,
            mime_type: "image/png".to_string(),
        })
    }
    
    /// Wait for condition
    async fn wait_for(&self, context: PooledContext, condition: WaitCondition) -> Result<ActionResult> {
        debug!("Waiting for: {:?}", condition);
        
        let page = context.page();
        
        match condition {
            WaitCondition::Selector(selector) => {
                page.wait_for_selector(&selector)
                    .await
                    .map_err(|_| BrowserError::Timeout(format!("Waiting for selector: {}", selector)))?;
            }
            WaitCondition::Navigation => {
                page.wait_for_load_state(crate::playwright_stub::api::LoadState::Load)
                    .await
                    .map_err(|e| BrowserError::Timeout(format!("Waiting for navigation: {}", e)))?;
            }
            WaitCondition::Duration(ms) => {
                tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
            }
        }
        
        Ok(ActionResult::WaitComplete)
    }
    
    /// Get metrics snapshot
    pub fn metrics(&self) -> Arc<BrowserMetrics> {
        self.metrics.clone()
    }
    
    /// Shutdown controller
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down browser controller");
        
        // Clear context pool
        let mut pool = self.context_pool.lock().await;
        pool.clear().await;
        
        // Close browser
        self.browser.close().await
            .map_err(|e| BrowserError::Playwright(e.to_string()))?;
        
        Ok(())
    }
}

/// Browser actions that can be executed
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BrowserAction {
    Navigate { url: String },
    Click { selector: String },
    Type { selector: String, text: String },
    Extract { selector: String, extract_type: ExtractType },
    Screenshot { full_page: bool },
    WaitFor { condition: WaitCondition },
}

/// Types of data extraction
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractType {
    Text,
    Html,
    Attribute(String),
    AllText,
}

impl std::fmt::Display for ExtractType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Html => write!(f, "html"),
            Self::Attribute(attr) => write!(f, "attribute:{}", attr),
            Self::AllText => write!(f, "all_text"),
        }
    }
}

/// Wait conditions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaitCondition {
    Selector(String),
    Navigation,
    Duration(u64),
}

/// Results from browser actions
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionResult {
    Navigate { 
        url: String,
        title: String,
        status_code: u16,
    },
    Click { 
        selector: String,
    },
    Type { 
        selector: String,
        text: String,
    },
    Extract { 
        selector: String,
        data: String,
    },
    Screenshot { 
        data: String,
        mime_type: String,
    },
    WaitComplete,
}

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_browser_controller_creation() {
        let config = BrowserConfig::default();
        // Note: This test requires playwright to be installed
        // let controller = BrowserController::new(config).await;
        // assert!(controller.is_ok());
    }
}