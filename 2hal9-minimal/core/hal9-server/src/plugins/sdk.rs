//! HAL9 Plugin SDK
//!
//! This module provides the SDK for developing HAL9 plugins in Rust.
//! Plugins can extend HAL9 with custom neurons, tools, memory providers, and more.

use std::collections::HashMap;
use uuid::Uuid;

use super::api::*;

// ============ Plugin Macros ============

/// Main macro to define a HAL9 plugin
#[macro_export]
macro_rules! hal9_plugin {
    (
        metadata: {
            name: $name:expr,
            version: $version:expr,
            author: $author:expr,
            description: $description:expr,
            license: $license:expr,
        },
        capabilities: [
            $($capability:expr),*
        ],
        permissions: [
            $($permission:expr),*
        ]
    ) => {
        use $crate::plugins::sdk::*;

        #[no_mangle]
        pub extern "C" fn _get_plugin_metadata() -> *const u8 {
            let metadata = PluginMetadata {
                id: Uuid::parse_str(env!("CARGO_PKG_NAME")).unwrap_or_else(|_| Uuid::new_v4()),
                name: $name.to_string(),
                version: $version.to_string(),
                author: $author.to_string(),
                description: $description.to_string(),
                license: $license.to_string(),
                repository: option_env!("CARGO_PKG_REPOSITORY").map(|s| s.to_string()),
                homepage: option_env!("CARGO_PKG_HOMEPAGE").map(|s| s.to_string()),
                capabilities: vec![$($capability),*],
                requirements: PluginRequirements {
                    min_hal9_version: "0.1.0".to_string(),
                    max_memory_mb: 64,
                    required_permissions: vec![$($permission),*],
                    dependencies: vec![],
                },
            };

            let json = serde_json::to_string(&metadata).unwrap();
            json.as_ptr()
        }

        #[no_mangle]
        pub extern "C" fn _get_metadata_len() -> usize {
            let metadata = PluginMetadata {
                id: Uuid::parse_str(env!("CARGO_PKG_NAME")).unwrap_or_else(|_| Uuid::new_v4()),
                name: $name.to_string(),
                version: $version.to_string(),
                author: $author.to_string(),
                description: $description.to_string(),
                license: $license.to_string(),
                repository: option_env!("CARGO_PKG_REPOSITORY").map(|s| s.to_string()),
                homepage: option_env!("CARGO_PKG_HOMEPAGE").map(|s| s.to_string()),
                capabilities: vec![$($capability),*],
                requirements: PluginRequirements {
                    min_hal9_version: "0.1.0".to_string(),
                    max_memory_mb: 64,
                    required_permissions: vec![$($permission),*],
                    dependencies: vec![],
                },
            };

            let json = serde_json::to_string(&metadata).unwrap();
            json.len()
        }
    };
}

/// Macro to implement a neuron plugin
#[macro_export]
macro_rules! neuron_plugin {
    ($struct_name:ident) => {
        #[no_mangle]
        pub extern "C" fn process_signal(signal_ptr: *const u8, signal_len: usize) -> *mut u8 {
            unsafe {
                let signal_bytes = std::slice::from_raw_parts(signal_ptr, signal_len);
                let signal: PluginSignal = match serde_json::from_slice(signal_bytes) {
                    Ok(s) => s,
                    Err(_) => return std::ptr::null_mut(),
                };

                let mut plugin = $struct_name::default();
                match plugin.process_signal(signal) {
                    Ok(result) => {
                        let json = serde_json::to_string(&result).unwrap();
                        let bytes = json.into_bytes();
                        let ptr = bytes.as_ptr() as *mut u8;
                        std::mem::forget(bytes);
                        ptr
                    }
                    Err(_) => std::ptr::null_mut(),
                }
            }
        }
    };
}

// ============ Plugin SDK Base Traits ============

pub trait PluginBase {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Initialize the plugin
    fn initialize(&mut self, context: PluginContext) -> Result<(), PluginError> {
        Ok(())
    }

    /// Cleanup when plugin is unloaded
    fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

// ============ Memory Management ============

#[no_mangle]
pub extern "C" fn allocate(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn deallocate(ptr: *mut u8, size: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr, size, size);
    }
}

// ============ Host Function Bindings ============

extern "C" {
    fn hal9_log_debug(ptr: *const u8, len: usize);
    fn hal9_log_info(ptr: *const u8, len: usize);
    fn hal9_log_warn(ptr: *const u8, len: usize);
    fn hal9_log_error(ptr: *const u8, len: usize);
    fn hal9_current_timestamp() -> i64;
    fn hal9_memory_get(key_ptr: *const u8, key_len: usize, value_ptr: *mut u8) -> i32;
    fn hal9_memory_set(
        key_ptr: *const u8,
        key_len: usize,
        value_ptr: *const u8,
        value_len: usize,
    ) -> i32;
}

// ============ Logging Functions ============

pub fn log_debug(message: &str) {
    unsafe {
        hal9_log_debug(message.as_ptr(), message.len());
    }
}

pub fn log_info(message: &str) {
    unsafe {
        hal9_log_info(message.as_ptr(), message.len());
    }
}

pub fn log_warn(message: &str) {
    unsafe {
        hal9_log_warn(message.as_ptr(), message.len());
    }
}

pub fn log_error(message: &str) {
    unsafe {
        hal9_log_error(message.as_ptr(), message.len());
    }
}

// ============ Time Functions ============

pub fn current_timestamp() -> i64 {
    unsafe { hal9_current_timestamp() }
}

// ============ Memory Functions ============

pub fn memory_get(key: &str) -> Result<Option<Vec<u8>>, PluginError> {
    let mut buffer = vec![0u8; 1024]; // Start with 1KB buffer

    unsafe {
        let result = hal9_memory_get(key.as_ptr(), key.len(), buffer.as_mut_ptr());

        match result {
            0 => Ok(None), // Key not found
            n if n > 0 => {
                buffer.truncate(n as usize);
                Ok(Some(buffer))
            }
            -1 => Err(PluginError {
                code: ErrorCode::PermissionDenied,
                message: "Memory access denied".to_string(),
                details: None,
            }),
            _ => Err(PluginError {
                code: ErrorCode::InternalError,
                message: "Memory get failed".to_string(),
                details: None,
            }),
        }
    }
}

pub fn memory_set(key: &str, value: &[u8]) -> Result<(), PluginError> {
    unsafe {
        let result = hal9_memory_set(key.as_ptr(), key.len(), value.as_ptr(), value.len());

        match result {
            0 => Ok(()),
            -1 => Err(PluginError {
                code: ErrorCode::PermissionDenied,
                message: "Memory access denied".to_string(),
                details: None,
            }),
            _ => Err(PluginError {
                code: ErrorCode::InternalError,
                message: "Memory set failed".to_string(),
                details: None,
            }),
        }
    }
}

// ============ Plugin Development Helpers ============

/// Helper to create a plugin signal
pub fn create_signal(content: String, signal_type: String) -> PluginSignal {
    PluginSignal {
        id: Uuid::new_v4(),
        content,
        signal_type,
        metadata: HashMap::new(),
        timestamp: current_timestamp(),
    }
}

/// Helper to create a plugin error
pub fn plugin_error(code: ErrorCode, message: impl Into<String>) -> PluginError {
    PluginError {
        code,
        message: message.into(),
        details: None,
    }
}

/// Helper to create a success result
pub fn ok<T>(value: T) -> Result<T, PluginError> {
    Ok(value)
}

/// Helper to create an error result
pub fn err<T>(code: ErrorCode, message: impl Into<String>) -> Result<T, PluginError> {
    Err(plugin_error(code, message))
}

// ============ Example Plugin Template ============

/// Example neuron plugin implementation
#[cfg(feature = "example")]
pub mod example {
    use super::*;

    pub struct ExampleNeuron {
        config: serde_json::Value,
        state: NeuronState,
    }

    impl Default for ExampleNeuron {
        fn default() -> Self {
            Self {
                config: serde_json::json!({}),
                state: NeuronState {
                    state: "idle".to_string(),
                    health: 1.0,
                    processed_count: 0,
                    error_count: 0,
                    last_activity: 0,
                },
            }
        }
    }

    impl NeuronPlugin for ExampleNeuron {
        fn process_signal(&mut self, signal: PluginSignal) -> Result<PluginSignal, PluginError> {
            log_info(&format!("Processing signal: {}", signal.id));

            // Process the signal
            let processed_content = format!("Processed: {}", signal.content);

            // Update state
            self.state.processed_count += 1;
            self.state.last_activity = current_timestamp();

            // Return processed signal
            Ok(PluginSignal {
                id: signal.id,
                content: processed_content,
                signal_type: "processed".to_string(),
                metadata: signal.metadata,
                timestamp: current_timestamp(),
            })
        }

        fn get_state(&self) -> NeuronState {
            self.state.clone()
        }

        fn update_config(&mut self, config: serde_json::Value) -> Result<(), PluginError> {
            self.config = config;
            Ok(())
        }
    }

    impl PluginLifecycle for ExampleNeuron {
        fn on_load(&mut self, context: PluginContext) -> Result<(), PluginError> {
            log_info("Example neuron loaded");
            Ok(())
        }

        fn on_activate(&mut self) -> Result<(), PluginError> {
            log_info("Example neuron activated");
            self.state.state = "active".to_string();
            Ok(())
        }

        fn on_deactivate(&mut self) -> Result<(), PluginError> {
            log_info("Example neuron deactivated");
            self.state.state = "inactive".to_string();
            Ok(())
        }

        fn on_unload(&mut self) -> Result<(), PluginError> {
            log_info("Example neuron unloaded");
            Ok(())
        }
    }
}
