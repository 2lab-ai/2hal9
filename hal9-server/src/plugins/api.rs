use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============ Plugin ABI Version ============

pub const PLUGIN_ABI_VERSION: u32 = 1;

// ============ Plugin Metadata ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub license: String,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub capabilities: Vec<PluginCapability>,
    pub requirements: PluginRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequirements {
    pub min_hal9_version: String,
    pub max_memory_mb: u32,
    pub required_permissions: Vec<Permission>,
    pub dependencies: Vec<PluginDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub plugin_id: String,
    pub min_version: String,
    pub optional: bool,
}

// ============ Plugin Capabilities ============

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "config")]
pub enum PluginCapability {
    // Neuron types that can be implemented
    NeuronType {
        layer: String,
        neuron_type: String,
        description: String,
    },
    // Signal processors
    SignalProcessor {
        signal_types: Vec<String>,
        priority: i32,
    },
    // Memory providers
    MemoryProvider {
        storage_type: String,
        features: Vec<String>,
    },
    // Learning algorithms
    LearningAlgorithm {
        algorithm_name: String,
        supported_layers: Vec<String>,
    },
    // Tool providers (MCP-like)
    ToolProvider {
        tool_name: String,
        tool_description: String,
        parameters: Vec<ToolParameter>,
    },
    // API extensions
    ApiExtension {
        endpoint: String,
        methods: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
    pub default: Option<serde_json::Value>,
}

// ============ Permissions ============

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Permission {
    // Network access
    NetworkHttp,
    NetworkHttps,
    NetworkTcp,
    NetworkUdp,
    
    // File system
    FileRead(String),   // Path pattern
    FileWrite(String),  // Path pattern
    FileCreate(String), // Path pattern
    
    // System resources
    SystemTime,
    SystemRandom,
    SystemEnv(String), // Environment variable
    
    // HAL9 APIs
    Hal9Signal,
    Hal9Memory,
    Hal9Metrics,
    Hal9Learning,
    
    // Inter-plugin communication
    PluginCall(String), // Target plugin ID
}

// ============ Plugin API ============

/// Core API that plugins must implement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginApi {
    pub abi_version: u32,
    pub metadata: PluginMetadata,
    pub exports: Vec<ExportedFunction>,
    pub imports: Vec<ImportedFunction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedFunction {
    pub name: String,
    pub description: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: ValueType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedFunction {
    pub module: String,
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: ValueType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParameter {
    pub name: String,
    pub value_type: ValueType,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
    String,
    Bytes,
    Json,
    Void,
}

// ============ Plugin Interfaces ============

/// Standard plugin lifecycle interface
pub trait PluginLifecycle {
    /// Called when plugin is loaded
    fn on_load(&mut self, context: PluginContext) -> Result<(), PluginError>;
    
    /// Called when plugin is activated
    fn on_activate(&mut self) -> Result<(), PluginError>;
    
    /// Called when plugin is deactivated
    fn on_deactivate(&mut self) -> Result<(), PluginError>;
    
    /// Called when plugin is unloaded
    fn on_unload(&mut self) -> Result<(), PluginError>;
}

/// Neuron plugin interface
pub trait NeuronPlugin: PluginLifecycle {
    /// Process a signal
    fn process_signal(&mut self, signal: PluginSignal) -> Result<PluginSignal, PluginError>;
    
    /// Get neuron state
    fn get_state(&self) -> NeuronState;
    
    /// Update configuration
    fn update_config(&mut self, config: serde_json::Value) -> Result<(), PluginError>;
}

/// Tool provider plugin interface
pub trait ToolPlugin: PluginLifecycle {
    /// Execute tool with parameters
    fn execute(&mut self, params: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, PluginError>;
    
    /// Validate parameters before execution
    fn validate_params(&self, params: &HashMap<String, serde_json::Value>) -> Result<(), PluginError>;
}

/// Memory provider plugin interface
pub trait MemoryPlugin: PluginLifecycle {
    /// Store memory entry
    fn store(&mut self, key: String, value: Vec<u8>, metadata: HashMap<String, String>) -> Result<(), PluginError>;
    
    /// Retrieve memory entry
    fn retrieve(&self, key: &str) -> Result<Option<(Vec<u8>, HashMap<String, String>)>, PluginError>;
    
    /// Search memory entries
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, PluginError>;
    
    /// Delete memory entry
    fn delete(&mut self, key: &str) -> Result<bool, PluginError>;
}

// ============ Plugin Context ============

#[derive(Debug, Clone)]
pub struct PluginContext {
    pub plugin_id: Uuid,
    pub config: serde_json::Value,
    pub permissions: Vec<Permission>,
    pub resource_limits: ResourceLimits,
    pub host_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_bytes: u64,
    pub max_cpu_percent: f32,
    pub max_execution_time_ms: u64,
    pub max_file_size_bytes: u64,
    pub max_network_connections: u32,
}

// ============ Plugin Communication ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSignal {
    pub id: Uuid,
    pub content: String,
    pub signal_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronState {
    pub state: String,
    pub health: f32,
    pub processed_count: u64,
    pub error_count: u64,
    pub last_activity: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub key: String,
    pub score: f32,
    pub metadata: HashMap<String, String>,
}

// ============ Plugin Errors ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginError {
    pub code: ErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidInput,
    PermissionDenied,
    ResourceExhausted,
    NotImplemented,
    InternalError,
    Timeout,
    NetworkError,
    ConfigError,
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for PluginError {}

// ============ Host Functions ============

/// Functions provided by the host to plugins
pub mod host_functions {
    use super::*;
    
    /// Logging functions
    pub fn log_debug(message: &str) {}
    pub fn log_info(message: &str) {}
    pub fn log_warn(message: &str) {}
    pub fn log_error(message: &str) {}
    
    /// Signal functions
    pub fn send_signal(signal: PluginSignal) -> Result<Uuid, PluginError> { todo!() }
    pub fn receive_signal() -> Result<Option<PluginSignal>, PluginError> { todo!() }
    
    /// Memory functions
    pub fn memory_get(key: &str) -> Result<Option<Vec<u8>>, PluginError> { todo!() }
    pub fn memory_set(key: &str, value: Vec<u8>) -> Result<(), PluginError> { todo!() }
    
    /// Metrics functions
    pub fn metric_increment(name: &str, value: f64, labels: HashMap<String, String>) { }
    pub fn metric_gauge(name: &str, value: f64, labels: HashMap<String, String>) { }
    
    /// Time functions
    pub fn current_timestamp() -> i64 { 0 }
    pub fn sleep_ms(ms: u64) { }
}