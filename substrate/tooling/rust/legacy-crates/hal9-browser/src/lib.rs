//! HAL9 Browser Automation Module
//!
//! Provides secure, scalable browser automation capabilities for HAL9 neurons.

pub mod context_pool;
pub mod controller;
pub mod error;
pub mod metrics;
pub mod playwright_stub;
pub mod security;
pub mod tools;

pub use context_pool::{ContextPool, PooledContext};
pub use controller::BrowserController;
pub use error::{BrowserError, Result};
pub use metrics::BrowserMetrics;
pub use security::{CredentialVault, SecuritySandbox, UrlPolicy};
pub use tools::{ClickTool, ExtractTool, NavigateTool, ScreenshotTool, TypeTool, WaitForTool};

// Re-export playwright types
// pub use playwright::{Browser, BrowserContext, Page};

/// Browser automation configuration
#[derive(Debug, Clone, serde::Deserialize)]
pub struct BrowserConfig {
    /// Maximum number of concurrent browser contexts
    pub max_contexts: usize,

    /// Browser type (chromium, firefox, webkit)
    pub browser_type: String,

    /// Headless mode
    pub headless: bool,

    /// Default viewport size
    pub viewport_width: u32,
    pub viewport_height: u32,

    /// Default timeout for operations (ms)
    pub default_timeout: u32,

    /// Resource limits
    pub resource_limits: ResourceLimits,

    /// Security configuration
    pub security: SecurityConfig,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            max_contexts: 10,
            browser_type: "chromium".to_string(),
            headless: true,
            viewport_width: 1920,
            viewport_height: 1080,
            default_timeout: 30000,
            resource_limits: ResourceLimits::default(),
            security: SecurityConfig::default(),
        }
    }
}

/// Resource limits for browser operations
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU percentage (0-100)
    pub max_cpu_percent: u8,

    /// Maximum memory in MB
    pub max_memory_mb: u32,

    /// Maximum execution time per action (seconds)
    pub max_execution_time_secs: u64,

    /// Maximum concurrent actions per context
    pub max_concurrent_actions: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: 50,
            max_memory_mb: 512,
            max_execution_time_secs: 60,
            max_concurrent_actions: 5,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SecurityConfig {
    /// URL whitelist patterns
    pub url_whitelist: Vec<String>,

    /// URL blacklist patterns
    pub url_blacklist: Vec<String>,

    /// Enable credential vault
    pub enable_credential_vault: bool,

    /// Enable activity auditing
    pub enable_audit_log: bool,

    /// Rate limiting per user
    pub rate_limit_per_minute: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            url_whitelist: vec!["*".to_string()],
            url_blacklist: vec![
                "*/admin/*".to_string(),
                "*/.git/*".to_string(),
                "*/api/internal/*".to_string(),
            ],
            enable_credential_vault: true,
            enable_audit_log: true,
            rate_limit_per_minute: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BrowserConfig::default();
        assert_eq!(config.max_contexts, 10);
        assert_eq!(config.browser_type, "chromium");
        assert!(config.headless);
    }
}
