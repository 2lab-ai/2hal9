use anyhow::{Context, Result};
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{
    api::*,
    loader::{PluginLoader, LoadedPlugin},
    registry::PluginRegistry,
    runtime::{WasmRuntime, RuntimeConfig},
};
use crate::signal::Signal;

// ============ Plugin State ============

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginState {
    Loaded,
    Active,
    Inactive,
    Failed(String),
    Unloading,
}

#[derive(Clone)]
struct ManagedPlugin {
    loaded: LoadedPlugin,
    state: PluginState,
    instances: Vec<PluginInstanceInfo>,
}

#[derive(Clone)]
struct PluginInstanceInfo {
    instance_id: Uuid,
    layer: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

// ============ Plugin Manager ============

pub struct PluginManager {
    runtime: Arc<WasmRuntime>,
    loader: Arc<PluginLoader>,
    registry: Arc<PluginRegistry>,
    plugins: Arc<DashMap<Uuid, Arc<RwLock<ManagedPlugin>>>>,
    capabilities: Arc<DashMap<String, Vec<Uuid>>>,
    config: PluginManagerConfig,
}

#[derive(Debug, Clone)]
pub struct PluginManagerConfig {
    pub plugins_dir: PathBuf,
    pub auto_load: bool,
    pub auto_activate: bool,
    pub max_plugins: usize,
    pub enable_hot_reload: bool,
}

impl Default for PluginManagerConfig {
    fn default() -> Self {
        Self {
            plugins_dir: PathBuf::from("./plugins"),
            auto_load: true,
            auto_activate: true,
            max_plugins: 100,
            enable_hot_reload: false,
        }
    }
}

impl PluginManager {
    pub async fn new(config: PluginManagerConfig) -> Result<Self> {
        // Create runtime
        let runtime = Arc::new(WasmRuntime::new(RuntimeConfig::default())?)
            
        // Create loader
        let loader = Arc::new(PluginLoader::new(
            config.plugins_dir.clone(),
            runtime.clone(),
        ));
        
        // Create registry
        let registry = Arc::new(PluginRegistry::new());
        
        let manager = Self {
            runtime,
            loader,
            registry,
            plugins: Arc::new(DashMap::new()),
            capabilities: Arc::new(DashMap::new()),
            config,
        };
        
        // Auto-load plugins if enabled
        if config.auto_load {
            manager.load_all_plugins().await?;
        }
        
        Ok(manager)
    }
    
    /// Load all plugins from the plugins directory
    pub async fn load_all_plugins(&self) -> Result<Vec<Uuid>> {
        let plugins = self.loader.scan_and_load_all().await?;
        let mut loaded_ids = Vec::new();
        
        for plugin in plugins {
            let plugin_id = plugin.id;
            
            // Check plugin limit
            if self.plugins.len() >= self.config.max_plugins {
                tracing::warn!("Plugin limit reached, skipping {}", plugin.metadata.name);
                continue;
            }
            
            // Register capabilities
            self.register_capabilities(&plugin);
            
            // Store plugin
            let managed = ManagedPlugin {
                loaded: plugin,
                state: PluginState::Loaded,
                instances: Vec::new(),
            };
            
            self.plugins.insert(plugin_id, Arc::new(RwLock::new(managed)));
            loaded_ids.push(plugin_id);
            
            // Auto-activate if enabled
            if self.config.auto_activate {
                if let Err(e) = self.activate_plugin(plugin_id).await {
                    tracing::error!("Failed to activate plugin {}: {}", plugin_id, e);
                }
            }
        }
        
        Ok(loaded_ids)
    }
    
    /// Install a plugin from a package
    pub async fn install_plugin(&self, package_path: &PathBuf) -> Result<Uuid> {
        // Load the plugin
        let plugin = self.loader.load_from_package(package_path).await?;
        let plugin_id = plugin.id;
        
        // Check if already installed
        if self.plugins.contains_key(&plugin_id) {
            return Err(anyhow::anyhow!("Plugin already installed: {}", plugin_id));
        }
        
        // Register capabilities
        self.register_capabilities(&plugin);
        
        // Store plugin
        let managed = ManagedPlugin {
            loaded: plugin,
            state: PluginState::Loaded,
            instances: Vec::new(),
        };
        
        self.plugins.insert(plugin_id, Arc::new(RwLock::new(managed)));
        
        // Register in registry
        self.registry.register_plugin(plugin_id).await?;
        
        Ok(plugin_id)
    }
    
    /// Activate a plugin
    pub async fn activate_plugin(&self, plugin_id: Uuid) -> Result<()> {
        let plugin_arc = self.plugins.get(&plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", plugin_id))?;
        
        let mut plugin = plugin_arc.write().await;
        
        match plugin.state {
            PluginState::Active => {
                return Ok(()); // Already active
            }
            PluginState::Failed(ref error) => {
                tracing::warn!("Activating previously failed plugin: {}", error);
            }
            _ => {}
        }
        
        // Activate the plugin
        match self.loader.activate_plugin(&plugin.loaded).await {
            Ok(()) => {
                plugin.state = PluginState::Active;
                tracing::info!("Activated plugin: {} v{}", 
                    plugin.loaded.metadata.name,
                    plugin.loaded.metadata.version
                );
                Ok(())
            }
            Err(e) => {
                plugin.state = PluginState::Failed(e.to_string());
                Err(e)
            }
        }
    }
    
    /// Deactivate a plugin
    pub async fn deactivate_plugin(&self, plugin_id: Uuid) -> Result<()> {
        let plugin_arc = self.plugins.get(&plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", plugin_id))?;
        
        let mut plugin = plugin_arc.write().await;
        
        if plugin.state != PluginState::Active {
            return Ok(()); // Not active
        }
        
        // Deactivate the plugin
        self.loader.deactivate_plugin(&plugin_id.to_string()).await?;
        plugin.state = PluginState::Inactive;
        
        tracing::info!("Deactivated plugin: {}", plugin.loaded.metadata.name);
        
        Ok(())
    }
    
    /// Uninstall a plugin
    pub async fn uninstall_plugin(&self, plugin_id: Uuid) -> Result<()> {
        // Deactivate first if active
        if let Some(plugin_arc) = self.plugins.get(&plugin_id) {
            let plugin = plugin_arc.read().await;
            if plugin.state == PluginState::Active {
                drop(plugin);
                self.deactivate_plugin(plugin_id).await?;
            }
        }
        
        // Remove from manager
        self.plugins.remove(&plugin_id);
        
        // Remove capabilities
        self.capabilities.retain(|_, plugins| {
            plugins.retain(|id| *id != plugin_id);
            !plugins.is_empty()
        });
        
        // Unregister from registry
        self.registry.unregister_plugin(plugin_id).await?;
        
        Ok(())
    }
    
    /// Get plugin information
    pub async fn get_plugin_info(&self, plugin_id: Uuid) -> Result<PluginInfo> {
        let plugin_arc = self.plugins.get(&plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", plugin_id))?;
        
        let plugin = plugin_arc.read().await;
        
        Ok(PluginInfo {
            id: plugin_id,
            metadata: plugin.loaded.metadata.clone(),
            state: plugin.state.clone(),
            install_path: plugin.loaded.install_path.clone(),
            instances: plugin.instances.len(),
        })
    }
    
    /// List all plugins
    pub async fn list_plugins(&self) -> Vec<PluginInfo> {
        let mut plugins = Vec::new();
        
        for entry in self.plugins.iter() {
            let plugin = entry.value().read().await;
            plugins.push(PluginInfo {
                id: *entry.key(),
                metadata: plugin.loaded.metadata.clone(),
                state: plugin.state.clone(),
                install_path: plugin.loaded.install_path.clone(),
                instances: plugin.instances.len(),
            });
        }
        
        plugins
    }
    
    /// Find plugins by capability
    pub fn find_plugins_by_capability(&self, capability_type: &str) -> Vec<Uuid> {
        self.capabilities
            .get(capability_type)
            .map(|plugins| plugins.clone())
            .unwrap_or_default()
    }
    
    /// Process a signal through plugin neurons
    pub async fn process_signal_through_plugins(
        &self,
        signal: &Signal,
        layer: &str,
    ) -> Result<Vec<Signal>> {
        let mut results = Vec::new();
        
        // Find neuron plugins for this layer
        let neuron_plugins = self.find_plugins_by_capability(&format!("neuron:{}", layer));
        
        for plugin_id in neuron_plugins {
            if let Some(plugin_arc) = self.plugins.get(&plugin_id) {
                let plugin = plugin_arc.read().await;
                
                if plugin.state != PluginState::Active {
                    continue;
                }
                
                // Convert signal to plugin format
                let plugin_signal = PluginSignal {
                    id: signal.id,
                    content: signal.content.clone(),
                    signal_type: signal.signal_type.clone(),
                    metadata: Default::default(),
                    timestamp: chrono::Utc::now().timestamp_millis(),
                };
                
                // Call plugin
                match self.call_plugin_neuron(&plugin_id, plugin_signal).await {
                    Ok(result) => {
                        // Convert back to Signal
                        let output_signal = Signal {
                            id: result.id,
                            content: result.content,
                            source: format!("plugin:{}", plugin_id),
                            target: signal.target.clone(),
                            signal_type: result.signal_type,
                            priority: signal.priority,
                            metadata: Some(result.metadata),
                            created_at: signal.created_at,
                            processed_at: Some(chrono::Utc::now()),
                        };
                        results.push(output_signal);
                    }
                    Err(e) => {
                        tracing::error!("Plugin {} failed to process signal: {}", plugin_id, e);
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    /// Call a plugin neuron function
    async fn call_plugin_neuron(
        &self,
        plugin_id: &Uuid,
        signal: PluginSignal,
    ) -> Result<PluginSignal> {
        // Serialize signal
        let signal_json = serde_json::to_string(&signal)?;
        let signal_bytes = signal_json.as_bytes();
        
        // Allocate memory in plugin for signal
        let ptr = self.runtime.call_function(
            &plugin_id.to_string(),
            "allocate",
            &[wasmtime::Val::I32(signal_bytes.len() as i32)],
        ).await?;
        
        // Write signal to plugin memory
        // TODO: Implement memory write
        
        // Call process_signal
        let result = self.runtime.call_function(
            &plugin_id.to_string(),
            "process_signal",
            &[ptr[0].clone()],
        ).await?;
        
        // Read result from plugin memory
        // TODO: Implement memory read
        
        // For now, return the input signal
        Ok(signal)
    }
    
    /// Register plugin capabilities
    fn register_capabilities(&self, plugin: &LoadedPlugin) {
        for capability in &plugin.metadata.capabilities {
            match capability {
                PluginCapability::NeuronType { layer, neuron_type, .. } => {
                    let key = format!("neuron:{}", layer);
                    self.capabilities.entry(key)
                        .or_insert_with(Vec::new)
                        .push(plugin.id);
                    
                    let key = format!("neuron:{}:{}", layer, neuron_type);
                    self.capabilities.entry(key)
                        .or_insert_with(Vec::new)
                        .push(plugin.id);
                }
                PluginCapability::ToolProvider { tool_name, .. } => {
                    let key = format!("tool:{}", tool_name);
                    self.capabilities.entry(key)
                        .or_insert_with(Vec::new)
                        .push(plugin.id);
                }
                PluginCapability::MemoryProvider { storage_type, .. } => {
                    let key = format!("memory:{}", storage_type);
                    self.capabilities.entry(key)
                        .or_insert_with(Vec::new)
                        .push(plugin.id);
                }
                PluginCapability::LearningAlgorithm { algorithm_name, .. } => {
                    let key = format!("learning:{}", algorithm_name);
                    self.capabilities.entry(key)
                        .or_insert_with(Vec::new)
                        .push(plugin.id);
                }
                _ => {}
            }
        }
    }
}

// ============ Plugin Info ============

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: Uuid,
    pub metadata: PluginMetadata,
    pub state: PluginState,
    pub install_path: PathBuf,
    pub instances: usize,
}

// ============ Plugin Error ============

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(Uuid),
    
    #[error("Plugin already installed: {0}")]
    AlreadyInstalled(Uuid),
    
    #[error("Plugin state error: {0}")]
    InvalidState(String),
    
    #[error("Plugin execution error: {0}")]
    ExecutionError(String),
    
    #[error("Plugin limit exceeded")]
    LimitExceeded,
    
    #[error("Plugin API error: {0}")]
    ApiError(#[from] super::api::PluginError),
    
    #[error("Runtime error: {0}")]
    RuntimeError(#[from] anyhow::Error),
}