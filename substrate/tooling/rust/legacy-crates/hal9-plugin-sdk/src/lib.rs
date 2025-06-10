//! HAL9 Plugin SDK
//!
//! This crate provides everything needed to develop plugins for HAL9.
//!
//! # Quick Start
//!
//! ```rust
//! use hal9_plugin_sdk::*;
//!
//! // Define your plugin
//! hal9_plugin! {
//!     metadata: {
//!         name: "My Plugin",
//!         version: "0.1.0",
//!         author: "Your Name",
//!         description: "Description of your plugin",
//!         license: "MIT",
//!     },
//!     capabilities: [
//!         PluginCapability::NeuronType {
//!             layer: "L2".to_string(),
//!             neuron_type: "custom".to_string(),
//!             description: "Custom neuron".to_string(),
//!         },
//!     ],
//!     permissions: [
//!         Permission::Hal9Signal,
//!     ]
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub license: String,
}

/// Plugin capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PluginCapability {
    /// Custom neuron type
    NeuronType {
        layer: String,
        neuron_type: String,
        description: String,
    },
    /// Custom tool
    Tool {
        name: String,
        description: String,
    },
    /// Memory provider
    MemoryProvider {
        name: String,
        description: String,
    },
    /// Protocol handler
    ProtocolHandler {
        protocol: String,
        description: String,
    },
}

/// Plugin permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Can send HAL9 signals
    Hal9Signal,
    /// Can access network
    Network,
    /// Can access filesystem
    Filesystem,
    /// Can spawn processes
    Process,
    /// Can access system info
    SystemInfo,
}

/// Plugin API result
pub type PluginResult<T> = Result<T, PluginError>;

/// Plugin error
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Plugin initialization failed: {0}")]
    InitError(String),
    
    #[error("Permission denied: {0:?}")]
    PermissionDenied(Permission),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

/// Plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Get plugin capabilities
    fn capabilities(&self) -> &[PluginCapability];
    
    /// Get required permissions
    fn permissions(&self) -> &[Permission];
    
    /// Initialize the plugin
    fn initialize(&mut self, config: HashMap<String, serde_json::Value>) -> PluginResult<()>;
    
    /// Shutdown the plugin
    fn shutdown(&mut self) -> PluginResult<()>;
}

/// Macro for defining plugins
#[macro_export]
macro_rules! hal9_plugin {
    (
        metadata: {
            name: $name:expr,
            version: $version:expr,
            author: $author:expr,
            description: $description:expr,
            license: $license:expr $(,)?
        },
        capabilities: [$($capability:expr),* $(,)?],
        permissions: [$($permission:expr),* $(,)?]
    ) => {
        pub struct PluginImpl {
            metadata: $crate::PluginMetadata,
            capabilities: Vec<$crate::PluginCapability>,
            permissions: Vec<$crate::Permission>,
        }
        
        impl Default for PluginImpl {
            fn default() -> Self {
                Self {
                    metadata: $crate::PluginMetadata {
                        name: $name.to_string(),
                        version: $version.to_string(),
                        author: $author.to_string(),
                        description: $description.to_string(),
                        license: $license.to_string(),
                    },
                    capabilities: vec![$($capability),*],
                    permissions: vec![$($permission),*],
                }
            }
        }
        
        impl $crate::Plugin for PluginImpl {
            fn metadata(&self) -> &$crate::PluginMetadata {
                &self.metadata
            }
            
            fn capabilities(&self) -> &[$crate::PluginCapability] {
                &self.capabilities
            }
            
            fn permissions(&self) -> &[$crate::Permission] {
                &self.permissions
            }
            
            fn initialize(&mut self, _config: std::collections::HashMap<String, serde_json::Value>) -> $crate::PluginResult<()> {
                Ok(())
            }
            
            fn shutdown(&mut self) -> $crate::PluginResult<()> {
                Ok(())
            }
        }
        
        #[no_mangle]
        pub extern "C" fn hal9_plugin_create() -> Box<dyn $crate::Plugin> {
            Box::new(PluginImpl::default())
        }
    };
}