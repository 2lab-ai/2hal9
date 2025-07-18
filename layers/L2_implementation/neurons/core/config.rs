//! Configuration types for 2HAL9

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub server_id: String,
    pub neurons: Vec<NeuronConfig>,
    
    /// Optional monitoring configuration
    #[serde(default)]
    pub monitoring: MonitoringConfig,
    
    /// Optional Claude API configuration
    #[serde(default)]
    pub claude: ClaudeConfig,
    
    /// Optional network configuration for distributed mode
    #[serde(default)]
    pub network: NetworkConfig,
    
    /// Optional memory configuration
    #[serde(default)]
    pub memory: MemoryConfig,
    
    /// Optional backward propagation configuration
    #[serde(default)]
    pub backward_propagation: BackwardPropagationConfig,
    
    /// Optional authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,
    
    /// Optional browser automation configuration
    #[serde(default)]
    pub browser: Option<BrowserConfig>,
}

/// Backward propagation configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackwardPropagationConfig {
    #[serde(default = "default_bp_enabled")]
    pub enabled: bool,
    
    #[serde(default = "default_learning_rate")]
    pub learning_rate: f32,
    
    #[serde(default = "default_pattern_threshold")]
    pub pattern_threshold: usize,
    
    #[serde(default = "default_adjustment_decay")]
    pub adjustment_decay: f32,
    
    #[serde(default = "default_max_gradient_depth")]
    pub max_gradient_depth: usize,
}

impl Default for BackwardPropagationConfig {
    fn default() -> Self {
        Self {
            enabled: default_bp_enabled(),
            learning_rate: default_learning_rate(),
            pattern_threshold: default_pattern_threshold(),
            adjustment_decay: default_adjustment_decay(),
            max_gradient_depth: default_max_gradient_depth(),
        }
    }
}

/// Memory configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryConfig {
    /// Enable memory system
    #[serde(default = "default_false")]
    pub enabled: bool,
    
    /// Database path
    #[serde(default = "default_memory_database_path")]
    pub database_path: String,
    
    /// Cleanup configuration
    #[serde(default)]
    pub cleanup: MemoryCleanupConfig,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            database_path: default_memory_database_path(),
            cleanup: MemoryCleanupConfig::default(),
        }
    }
}

/// Memory cleanup configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryCleanupConfig {
    /// Retention days
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    
    /// Minimum importance to keep
    #[serde(default = "default_min_importance")]
    pub min_importance: f32,
}

impl Default for MemoryCleanupConfig {
    fn default() -> Self {
        Self {
            retention_days: default_retention_days(),
            min_importance: default_min_importance(),
        }
    }
}

/// Individual neuron configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NeuronConfig {
    pub id: String,
    pub layer: String,
    
    /// Command to spawn neuron process (for CLI mode)
    #[serde(default = "default_claude_command")]
    pub claude_command: String,
    
    /// System prompt for the neuron
    pub system_prompt: Option<String>,
    
    /// Forward connections (neurons this one sends to)
    pub forward_connections: Vec<String>,
    
    /// Backward connections (neurons this one receives errors from)
    pub backward_connections: Vec<String>,
    
    /// Optional neuron-specific configuration
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
}

/// Monitoring configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MonitoringConfig {
    /// Enable metrics collection
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Metrics export interval in seconds
    #[serde(default = "default_metrics_interval")]
    pub metrics_interval: u64,
    
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

/// Claude API configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClaudeConfig {
    /// Integration mode: "mock", "api", "hybrid", "auto", or "cli"
    #[serde(default = "default_claude_mode")]
    pub mode: String,
    
    /// API key (from environment if not specified)
    pub api_key: Option<String>,
    
    /// Model to use
    #[serde(default = "default_claude_model")]
    pub model: String,
    
    /// Default temperature
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    
    /// Default max tokens
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    
    /// Rate limit (requests per minute)
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u32,
    
    /// Mock responses for testing (layer -> responses)
    #[serde(default)]
    pub mock_responses: HashMap<String, Vec<MockResponse>>,
    
    /// Fallback to mock on API errors
    #[serde(default = "default_true")]
    pub fallback_to_mock: bool,
    
    /// Cost controls
    #[serde(default)]
    pub cost_controls: CostControls,
}

/// Cost control configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CostControls {
    /// Maximum cost per hour in USD
    #[serde(default = "default_max_cost_per_hour")]
    pub max_cost_per_hour: f64,
    
    /// Maximum cost per day in USD
    #[serde(default = "default_max_cost_per_day")]
    pub max_cost_per_day: f64,
    
    /// Maximum tokens per request
    #[serde(default = "default_max_tokens_per_request")]
    pub max_tokens_per_request: u32,
    
    /// Alert threshold (percentage of limit)
    #[serde(default = "default_alert_threshold")]
    pub alert_threshold: f64,
}

/// Network configuration for distributed mode
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkConfig {
    /// Enable distributed mode
    #[serde(default = "default_false")]
    pub enabled: bool,
    
    /// TCP bind address
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    
    /// Enable service discovery
    #[serde(default = "default_true")]
    pub discovery_enabled: bool,
    
    /// Discovery multicast address
    #[serde(default = "default_multicast_address")]
    pub discovery_address: String,
    
    /// Discovery group (for multi-tenancy)
    #[serde(default = "default_discovery_group")]
    pub discovery_group: String,
    
    /// Maximum connections
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    
    /// Enable TLS encryption
    #[serde(default = "default_false")]
    pub tls_enabled: bool,
    
    /// TLS certificate path
    pub tls_cert: Option<String>,
    
    /// TLS key path
    pub tls_key: Option<String>,
}

/// Mock response configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MockResponse {
    /// Trigger phrase to match (partial match)
    pub trigger: String,
    
    /// Response template
    pub response: String,
    
    /// Optional delay in milliseconds
    #[serde(default = "default_mock_delay")]
    pub delay_ms: u64,
}

/// Authentication configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// Enable authentication
    #[serde(default = "default_false")]
    pub enabled: bool,
    
    /// JWT secret key
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    
    /// Database path for auth data
    #[serde(default = "default_auth_database_path")]
    pub database_path: String,
    
    /// Access token duration in minutes
    #[serde(default = "default_access_token_duration")]
    pub access_token_duration_minutes: i64,
    
    /// Refresh token duration in days
    #[serde(default = "default_refresh_token_duration")]
    pub refresh_token_duration_days: i64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            jwt_secret: default_jwt_secret(),
            database_path: default_auth_database_path(),
            access_token_duration_minutes: default_access_token_duration(),
            refresh_token_duration_days: default_refresh_token_duration(),
        }
    }
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            mode: default_claude_mode(),
            api_key: None,
            model: default_claude_model(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            rate_limit: default_rate_limit(),
            mock_responses: HashMap::new(),
            fallback_to_mock: true,
            cost_controls: CostControls::default(),
        }
    }
}

impl Default for CostControls {
    fn default() -> Self {
        Self {
            max_cost_per_hour: default_max_cost_per_hour(),
            max_cost_per_day: default_max_cost_per_day(),
            max_tokens_per_request: default_max_tokens_per_request(),
            alert_threshold: default_alert_threshold(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enabled: default_false(),
            bind_address: default_bind_address(),
            discovery_enabled: default_true(),
            discovery_address: default_multicast_address(),
            discovery_group: default_discovery_group(),
            max_connections: default_max_connections(),
            tls_enabled: default_false(),
            tls_cert: None,
            tls_key: None,
        }
    }
}

// Default value functions
fn default_claude_command() -> String {
    "claude".to_string()
}

fn default_true() -> bool {
    true
}

fn default_metrics_interval() -> u64 {
    30
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_claude_mode() -> String {
    "mock".to_string()
}

fn default_claude_model() -> String {
    "claude-3-opus-20240229".to_string()
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_rate_limit() -> u32 {
    60
}

fn default_mock_delay() -> u64 {
    100
}

fn default_jwt_secret() -> String {
    "change-me-in-production".to_string()
}

fn default_auth_database_path() -> String {
    "./data/hal9_auth.db".to_string()
}

fn default_access_token_duration() -> i64 {
    15 // 15 minutes
}

fn default_refresh_token_duration() -> i64 {
    7 // 7 days
}

fn default_max_cost_per_hour() -> f64 {
    10.0 // $10 per hour
}

fn default_max_cost_per_day() -> f64 {
    100.0 // $100 per day
}

fn default_max_tokens_per_request() -> u32 {
    4096
}

fn default_alert_threshold() -> f64 {
    0.8 // Alert at 80% of limit
}

fn default_false() -> bool {
    false
}

fn default_bind_address() -> String {
    "0.0.0.0:9000".to_string()
}

fn default_multicast_address() -> String {
    "239.255.42.99:8888".to_string()
}

fn default_discovery_group() -> String {
    "default".to_string()
}

fn default_max_connections() -> usize {
    1000
}

fn default_memory_database_path() -> String {
    "./data/hal9_memory.db".to_string()
}

fn default_retention_days() -> u32 {
    30
}

fn default_min_importance() -> f32 {
    0.3
}

fn default_bp_enabled() -> bool {
    true
}

fn default_learning_rate() -> f32 {
    0.1
}

fn default_pattern_threshold() -> usize {
    3
}

fn default_adjustment_decay() -> f32 {
    0.95
}

fn default_max_gradient_depth() -> usize {
    3
}

/// Browser automation configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrowserConfig {
    /// Browser automation settings
    pub config: BrowserSettings,
}

/// Browser settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrowserSettings {
    /// Maximum concurrent browser contexts
    #[serde(default = "default_max_browser_contexts")]
    pub max_contexts: usize,
    
    /// Browser type (chromium, firefox, webkit)
    #[serde(default = "default_browser_type")]
    pub browser_type: String,
    
    /// Headless mode
    #[serde(default = "default_browser_headless")]
    pub headless: bool,
    
    /// Viewport width
    #[serde(default = "default_viewport_width")]
    pub viewport_width: u32,
    
    /// Viewport height
    #[serde(default = "default_viewport_height")]
    pub viewport_height: u32,
    
    /// Default timeout in milliseconds
    #[serde(default = "default_browser_timeout")]
    pub default_timeout: u32,
    
    /// Resource limits
    #[serde(default)]
    pub resource_limits: BrowserResourceLimits,
    
    /// Security configuration
    #[serde(default)]
    pub security: BrowserSecurityConfig,
}

/// Browser resource limits
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrowserResourceLimits {
    #[serde(default = "default_max_cpu_percent")]
    pub max_cpu_percent: u8,
    
    #[serde(default = "default_max_memory_mb")]
    pub max_memory_mb: u32,
    
    #[serde(default = "default_max_execution_time_secs")]
    pub max_execution_time_secs: u64,
    
    #[serde(default = "default_max_concurrent_actions")]
    pub max_concurrent_actions: usize,
}

impl Default for BrowserResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: default_max_cpu_percent(),
            max_memory_mb: default_max_memory_mb(),
            max_execution_time_secs: default_max_execution_time_secs(),
            max_concurrent_actions: default_max_concurrent_actions(),
        }
    }
}

/// Browser security configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrowserSecurityConfig {
    /// URL whitelist patterns
    #[serde(default)]
    pub url_whitelist: Vec<String>,
    
    /// URL blacklist patterns
    #[serde(default)]
    pub url_blacklist: Vec<String>,
    
    /// Enable credential vault
    #[serde(default = "default_true")]
    pub enable_credential_vault: bool,
    
    /// Enable audit logging
    #[serde(default = "default_true")]
    pub enable_audit_log: bool,
    
    /// Rate limit per minute
    #[serde(default = "default_browser_rate_limit")]
    pub rate_limit_per_minute: u32,
}

impl Default for BrowserSecurityConfig {
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
            rate_limit_per_minute: default_browser_rate_limit(),
        }
    }
}

// Browser default functions
fn default_max_browser_contexts() -> usize {
    10
}

fn default_browser_type() -> String {
    "chromium".to_string()
}

fn default_browser_headless() -> bool {
    true
}

fn default_viewport_width() -> u32 {
    1920
}

fn default_viewport_height() -> u32 {
    1080
}

fn default_browser_timeout() -> u32 {
    30000
}

fn default_max_cpu_percent() -> u8 {
    50
}

fn default_max_memory_mb() -> u32 {
    512
}

fn default_max_execution_time_secs() -> u64 {
    60
}

fn default_max_concurrent_actions() -> usize {
    5
}

fn default_browser_rate_limit() -> u32 {
    60
}

/// Layer-specific system prompts
pub fn get_system_prompt(layer: &str) -> String {
    match layer {
        "L4" => {
            "You are a strategic planning AI neuron in a hierarchical neural network.
Your role is to receive high-level objectives and break them down into strategic initiatives.
Output format: List of 2-3 strategic directives for L3 neurons.
Focus on WHAT needs to be achieved, not HOW.".to_string()
        }
        "L3" => {
            "You are a system design AI neuron in a hierarchical neural network.
Your role is to receive strategic directives and create architectural designs.
Output format: Technical design specifications for L2 neurons.
Focus on system architecture and component interaction.".to_string()
        }
        "L2" => {
            "You are an implementation AI neuron in a hierarchical neural network.
Your role is to receive design specifications and implement solutions.
Output format: Actual code, configurations, or detailed procedures.
Focus on concrete implementation details.".to_string()
        }
        _ => {
            format!("You are a {} layer AI neuron in a hierarchical neural network.", layer)
        }
    }
}