use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;
use zip::ZipArchive;

use super::api::*;
use super::runtime::WasmRuntime;
use super::sandbox::SecurityPolicy;

// ============ Plugin Package Format ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub api_version: u32,
    pub metadata: PluginMetadata,
    pub files: PluginFiles,
    pub signature: Option<PluginSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginFiles {
    pub wasm: String,
    pub icon: Option<String>,
    pub readme: Option<String>,
    pub license: Option<String>,
    pub examples: Vec<String>,
    pub assets: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSignature {
    pub algorithm: String,
    pub public_key: String,
    pub signature: String,
}

// ============ Loaded Plugin ============

pub struct LoadedPlugin {
    pub id: Uuid,
    pub metadata: PluginMetadata,
    pub wasm_bytes: Vec<u8>,
    pub manifest: PluginManifest,
    pub install_path: PathBuf,
    pub context: PluginContext,
}

// ============ Plugin Loader ============

pub struct PluginLoader {
    plugins_dir: PathBuf,
    runtime: Arc<WasmRuntime>,
    security_policies: HashMap<String, SecurityPolicy>,
}

impl PluginLoader {
    pub fn new(plugins_dir: PathBuf, runtime: Arc<WasmRuntime>) -> Self {
        Self {
            plugins_dir,
            runtime,
            security_policies: Self::default_security_policies(),
        }
    }

    /// Load plugin from a .hal9 package file
    pub async fn load_from_package(&self, package_path: &Path) -> Result<LoadedPlugin> {
        // Open the package (ZIP file)
        let file = std::fs::File::open(package_path).context("Failed to open plugin package")?;

        let mut archive = ZipArchive::new(file).context("Failed to read plugin package")?;

        // Read manifest
        let manifest = self.read_manifest(&mut archive)?;

        // Validate API version
        if manifest.api_version != PLUGIN_ABI_VERSION {
            return Err(anyhow::anyhow!(
                "Incompatible plugin API version: {} (expected {})",
                manifest.api_version,
                PLUGIN_ABI_VERSION
            ));
        }

        // Verify signature if present
        if let Some(ref signature) = manifest.signature {
            self.verify_signature(&manifest, signature)?;
        }

        // Extract plugin files
        let plugin_id = manifest.metadata.id;
        let install_path = self.plugins_dir.join(plugin_id.to_string());

        // Create plugin directory
        fs::create_dir_all(&install_path).await?;

        // Extract WASM file
        let wasm_bytes = self.extract_wasm(&mut archive, &manifest.files.wasm)?;

        // Extract other files
        self.extract_assets(&mut archive, &manifest.files, &install_path)
            .await?;

        // Create plugin context
        let context = self.create_plugin_context(&manifest.metadata)?;

        Ok(LoadedPlugin {
            id: plugin_id,
            metadata: manifest.metadata.clone(),
            wasm_bytes,
            manifest,
            install_path,
            context,
        })
    }

    /// Load plugin from directory (development mode)
    pub async fn load_from_directory(&self, dir_path: &Path) -> Result<LoadedPlugin> {
        // Read manifest.json
        let manifest_path = dir_path.join("manifest.json");
        let manifest_content = fs::read_to_string(&manifest_path)
            .await
            .context("Failed to read manifest.json")?;

        let manifest: PluginManifest =
            serde_json::from_str(&manifest_content).context("Failed to parse manifest.json")?;

        // Read WASM file
        let wasm_path = dir_path.join(&manifest.files.wasm);
        let wasm_bytes = fs::read(&wasm_path)
            .await
            .context("Failed to read WASM file")?;

        // Create plugin context
        let context = self.create_plugin_context(&manifest.metadata)?;

        Ok(LoadedPlugin {
            id: manifest.metadata.id,
            metadata: manifest.metadata.clone(),
            wasm_bytes,
            manifest,
            install_path: dir_path.to_path_buf(),
            context,
        })
    }

    /// Scan plugins directory and load all plugins
    pub async fn scan_and_load_all(&self) -> Result<Vec<LoadedPlugin>> {
        let mut plugins = Vec::new();

        // Ensure plugins directory exists
        fs::create_dir_all(&self.plugins_dir).await?;

        // Scan for .hal9 packages
        let mut entries = fs::read_dir(&self.plugins_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Load .hal9 packages
            if path.extension().and_then(|s| s.to_str()) == Some("hal9") {
                match self.load_from_package(&path).await {
                    Ok(plugin) => {
                        tracing::info!(
                            "Loaded plugin: {} v{}",
                            plugin.metadata.name,
                            plugin.metadata.version
                        );
                        plugins.push(plugin);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load plugin {:?}: {}", path, e);
                    }
                }
            }

            // Load directories (development mode)
            if path.is_dir() && path.join("manifest.json").exists() {
                match self.load_from_directory(&path).await {
                    Ok(plugin) => {
                        tracing::info!(
                            "Loaded development plugin: {} v{}",
                            plugin.metadata.name,
                            plugin.metadata.version
                        );
                        plugins.push(plugin);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load plugin {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// Activate a loaded plugin
    pub async fn activate_plugin(&self, plugin: &LoadedPlugin) -> Result<()> {
        // Determine security policy
        let security_policy = self.get_security_policy(&plugin.metadata);

        // Load into runtime
        self.runtime
            .load_plugin(
                &plugin.id.to_string(),
                &plugin.wasm_bytes,
                plugin.metadata.clone(),
                plugin.context.clone(),
                &security_policy,
            )
            .await?;

        // Call activation hook
        self.runtime
            .call_function(&plugin.id.to_string(), "on_activate", &[])
            .await?;

        Ok(())
    }

    /// Deactivate a plugin
    pub async fn deactivate_plugin(&self, plugin_id: &str) -> Result<()> {
        // Call deactivation hook
        let _ = self
            .runtime
            .call_function(plugin_id, "on_deactivate", &[])
            .await;

        // Unload from runtime
        self.runtime.unload_plugin(plugin_id).await?;

        Ok(())
    }

    // ============ Helper Methods ============

    fn read_manifest(&self, archive: &mut ZipArchive<std::fs::File>) -> Result<PluginManifest> {
        let mut manifest_file = archive
            .by_name("manifest.json")
            .context("manifest.json not found in package")?;

        let mut manifest_content = String::new();
        std::io::Read::read_to_string(&mut manifest_file, &mut manifest_content)?;

        serde_json::from_str(&manifest_content).context("Failed to parse manifest.json")
    }

    fn extract_wasm(
        &self,
        archive: &mut ZipArchive<std::fs::File>,
        wasm_filename: &str,
    ) -> Result<Vec<u8>> {
        let mut wasm_file = archive
            .by_name(wasm_filename)
            .context("WASM file not found in package")?;

        let mut wasm_bytes = Vec::new();
        std::io::Read::read_to_end(&mut wasm_file, &mut wasm_bytes)?;

        Ok(wasm_bytes)
    }

    async fn extract_assets(
        &self,
        archive: &mut ZipArchive<std::fs::File>,
        files: &PluginFiles,
        install_path: &Path,
    ) -> Result<()> {
        // Extract README
        if let Some(ref readme) = files.readme {
            if let Ok(mut file) = archive.by_name(readme) {
                let path = install_path.join("README.md");
                let mut content = String::new();
                std::io::Read::read_to_string(&mut file, &mut content)?;
                fs::write(path, content).await?;
            }
        }

        // Extract examples
        let examples_dir = install_path.join("examples");
        if !files.examples.is_empty() {
            fs::create_dir_all(&examples_dir).await?;
        }

        for example in &files.examples {
            if let Ok(mut file) = archive.by_name(example) {
                let filename = Path::new(example)
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("Invalid example filename"))?;
                let path = examples_dir.join(filename);

                let mut content = Vec::new();
                std::io::Read::read_to_end(&mut file, &mut content)?;
                fs::write(path, content).await?;
            }
        }

        Ok(())
    }

    fn create_plugin_context(&self, metadata: &PluginMetadata) -> Result<PluginContext> {
        Ok(PluginContext {
            plugin_id: metadata.id,
            config: serde_json::json!({}),
            permissions: metadata.requirements.required_permissions.clone(),
            resource_limits: ResourceLimits {
                max_memory_bytes: (metadata.requirements.max_memory_mb as u64) * 1024 * 1024,
                max_cpu_percent: 25.0,
                max_execution_time_ms: 5000,
                max_file_size_bytes: 10 * 1024 * 1024, // 10MB
                max_network_connections: 10,
            },
            host_version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    fn get_security_policy(&self, metadata: &PluginMetadata) -> SecurityPolicy {
        // Check for predefined policies
        for capability in &metadata.capabilities {
            match capability {
                PluginCapability::NeuronType { .. } => {
                    return self.security_policies["neuron"].clone();
                }
                PluginCapability::ToolProvider { .. } => {
                    return self.security_policies["tool"].clone();
                }
                _ => {}
            }
        }

        // Default restrictive policy
        self.security_policies["default"].clone()
    }

    fn verify_signature(
        &self,
        manifest: &PluginManifest,
        signature: &PluginSignature,
    ) -> Result<()> {
        // TODO: Implement signature verification
        tracing::warn!("Plugin signature verification not yet implemented");
        Ok(())
    }

    fn default_security_policies() -> HashMap<String, SecurityPolicy> {
        let mut policies = HashMap::new();

        // Default restrictive policy
        policies.insert(
            "default".to_string(),
            SecurityPolicy {
                allow_network: false,
                allow_filesystem: false,
                allow_system_time: true,
                allow_random: true,
                allow_environment: false,
                max_memory_bytes: 64 * 1024 * 1024,
                max_cpu_percent: 10.0,
                allowed_hosts: vec![],
                allowed_paths: vec![],
            },
        );

        // Neuron plugin policy
        policies.insert(
            "neuron".to_string(),
            SecurityPolicy {
                allow_network: false,
                allow_filesystem: false,
                allow_system_time: true,
                allow_random: true,
                allow_environment: false,
                max_memory_bytes: 128 * 1024 * 1024,
                max_cpu_percent: 25.0,
                allowed_hosts: vec![],
                allowed_paths: vec![],
            },
        );

        // Tool provider policy
        policies.insert(
            "tool".to_string(),
            SecurityPolicy {
                allow_network: true,
                allow_filesystem: true,
                allow_system_time: true,
                allow_random: true,
                allow_environment: true,
                max_memory_bytes: 256 * 1024 * 1024,
                max_cpu_percent: 50.0,
                allowed_hosts: vec!["*".to_string()],
                allowed_paths: vec!["/tmp".to_string()],
            },
        );

        policies
    }
}

use std::sync::Arc;
