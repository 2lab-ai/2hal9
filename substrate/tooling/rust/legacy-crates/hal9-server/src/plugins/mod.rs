pub mod api;
pub mod loader;
pub mod manager;
pub mod registry;
pub mod runtime;
pub mod sandbox;
pub mod sdk;

pub use api::{PluginApi, PluginCapability, PluginMetadata};
pub use loader::{LoadedPlugin, PluginLoader};
pub use manager::{PluginError, PluginManager};
pub use registry::{PluginPackage, PluginRegistry};
pub use runtime::{RuntimeConfig, WasmRuntime};
pub use sandbox::{ResourceLimits, SecurityPolicy};

#[cfg(test)]
mod tests;
