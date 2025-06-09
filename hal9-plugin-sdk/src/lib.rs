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

// Re-export types from hal9-server
// In a real implementation, these would be in a shared crate

pub use serde::{Deserialize, Serialize};
pub use std::collections::HashMap;
pub use uuid::Uuid;

// Include the SDK types directly
include!("../../hal9-server/src/plugins/api.rs");
include!("../../hal9-server/src/plugins/sdk.rs");