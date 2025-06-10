use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

use super::api::*;
use super::sandbox::SecurityPolicy;

// ============ Runtime Configuration ============

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub max_memory_pages: u32,
    pub enable_fuel: bool,
    pub initial_fuel: u64,
    pub enable_epoch_interruption: bool,
    pub epoch_deadline_ms: u64,
    pub enable_cache: bool,
    pub cache_dir: Option<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_memory_pages: 1024, // 64MB max memory
            enable_fuel: true,
            initial_fuel: 1_000_000_000,
            enable_epoch_interruption: true,
            epoch_deadline_ms: 5000,
            enable_cache: true,
            cache_dir: Some("/tmp/hal9-wasm-cache".to_string()),
        }
    }
}

// ============ WASM Runtime ============

pub struct WasmRuntime {
    engine: Engine,
    config: RuntimeConfig,
    instances: Arc<RwLock<HashMap<String, PluginInstance>>>,
}

struct PluginInstance {
    instance: Instance,
    store: Store<PluginStore>,
    metadata: PluginMetadata,
    exports: HashMap<String, Func>,
}

struct PluginStore {
    wasi: WasiCtx,
    host_functions: HostFunctions,
    plugin_context: PluginContext,
    fuel_consumed: u64,
}

struct HostFunctions {
    log_func: Option<Func>,
    signal_send: Option<Func>,
    memory_get: Option<Func>,
    memory_set: Option<Func>,
}

impl WasmRuntime {
    pub fn new(config: RuntimeConfig) -> Result<Self> {
        let mut engine_config = Config::new();

        // Configure memory limits
        engine_config.memory_guaranteed_dense_image_size(config.max_memory_pages as u64 * 65536);
        engine_config.static_memory_maximum_size(config.max_memory_pages as u64 * 65536);

        // Configure fuel metering
        if config.enable_fuel {
            engine_config.consume_fuel(true);
        }

        // Configure epoch interruption
        if config.enable_epoch_interruption {
            engine_config.epoch_interruption(true);
        }

        // Enable caching
        if config.enable_cache {
            if let Some(ref cache_dir) = config.cache_dir {
                engine_config.cache_config_load(cache_dir)?;
            }
        }

        // Security hardening
        engine_config.wasm_backtrace_details(WasmBacktraceDetails::Disable);
        engine_config.wasm_reference_types(false);
        engine_config.wasm_simd(true);
        engine_config.wasm_threads(false);
        engine_config.wasm_multi_value(true);

        let engine = Engine::new(&engine_config)?;

        Ok(Self {
            engine,
            config,
            instances: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Load a plugin from WASM bytes
    pub async fn load_plugin(
        &self,
        plugin_id: &str,
        wasm_bytes: &[u8],
        metadata: PluginMetadata,
        context: PluginContext,
        security_policy: &SecurityPolicy,
    ) -> Result<()> {
        // Compile the module
        let module =
            Module::new(&self.engine, wasm_bytes).context("Failed to compile WASM module")?;

        // Create WASI context based on security policy
        let wasi_ctx = self.create_wasi_context(&context, security_policy)?;

        // Create store with plugin context
        let mut store = Store::new(
            &self.engine,
            PluginStore {
                wasi: wasi_ctx,
                host_functions: HostFunctions::default(),
                plugin_context: context,
                fuel_consumed: 0,
            },
        );

        // Add initial fuel
        if self.config.enable_fuel {
            store.add_fuel(self.config.initial_fuel)?;
        }

        // Create linker and add WASI
        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |state: &mut PluginStore| &mut state.wasi)?;

        // Add host functions
        self.link_host_functions(&mut linker)?;

        // Instantiate the module
        let instance = linker
            .instantiate(&mut store, &module)
            .context("Failed to instantiate WASM module")?;

        // Extract exports
        let exports = self.extract_exports(&instance, &mut store)?;

        // Call plugin initialization if available
        if let Some(init_func) = instance.get_func(&mut store, "_initialize") {
            init_func.call(&mut store, &[], &mut [])?;
        }

        // Store the instance
        let mut instances = self.instances.write().await;
        instances.insert(
            plugin_id.to_string(),
            PluginInstance {
                instance,
                store,
                metadata,
                exports,
            },
        );

        Ok(())
    }

    /// Call a plugin function
    pub async fn call_function(
        &self,
        plugin_id: &str,
        function_name: &str,
        args: &[Val],
    ) -> Result<Vec<Val>> {
        let mut instances = self.instances.write().await;
        let instance = instances
            .get_mut(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", plugin_id))?;

        let func = instance
            .exports
            .get(function_name)
            .ok_or_else(|| anyhow::anyhow!("Function not found: {}", function_name))?;

        // Set epoch deadline
        if self.config.enable_epoch_interruption {
            instance
                .store
                .set_epoch_deadline(self.config.epoch_deadline_ms);
        }

        // Prepare results buffer
        let func_ty = func.ty(&instance.store);
        let mut results = vec![Val::I32(0); func_ty.results().len()];

        // Call the function
        func.call(&mut instance.store, args, &mut results)
            .context("Function call failed")?;

        // Check fuel consumption
        if self.config.enable_fuel {
            let fuel_consumed = instance.store.fuel_consumed().unwrap_or(0);
            instance.store.data_mut().fuel_consumed = fuel_consumed;
        }

        Ok(results)
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut instances = self.instances.write().await;

        if let Some(mut instance) = instances.remove(plugin_id) {
            // Call plugin cleanup if available
            if let Some(cleanup_func) = instance.exports.get("_cleanup") {
                let _ = cleanup_func.call(&mut instance.store, &[], &mut []);
            }
        }

        Ok(())
    }

    /// Get plugin metadata
    pub async fn get_plugin_metadata(&self, plugin_id: &str) -> Result<PluginMetadata> {
        let instances = self.instances.read().await;
        let instance = instances
            .get(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", plugin_id))?;

        Ok(instance.metadata.clone())
    }

    /// Create WASI context based on security policy
    fn create_wasi_context(
        &self,
        context: &PluginContext,
        security_policy: &SecurityPolicy,
    ) -> Result<WasiCtx> {
        let mut builder = WasiCtxBuilder::new();

        // Set environment variables if permitted
        for perm in &context.permissions {
            if let Permission::SystemEnv(var) = perm {
                if let Ok(value) = std::env::var(var) {
                    builder = builder.env(var, value)?;
                }
            }
        }

        // Configure file system access
        if security_policy.allow_filesystem {
            for perm in &context.permissions {
                match perm {
                    Permission::FileRead(path) => {
                        if let Ok(dir) = builder.preopened_dir(path, path) {
                            builder = dir;
                        }
                    }
                    Permission::FileWrite(path) | Permission::FileCreate(path) => {
                        if let Ok(dir) = builder.preopened_dir(path, path) {
                            builder = dir;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Configure standard I/O
        builder = builder.inherit_stdout().inherit_stderr();

        Ok(builder.build())
    }

    /// Link host functions to the WASM module
    fn link_host_functions(&self, linker: &mut Linker<PluginStore>) -> Result<()> {
        // Logging functions
        linker.func_wrap(
            "hal9",
            "log_debug",
            |mut caller: Caller<'_, PluginStore>, ptr: i32, len: i32| {
                let message = read_string_from_memory(&mut caller, ptr, len)?;
                tracing::debug!("[Plugin] {}", message);
                Ok(())
            },
        )?;

        linker.func_wrap(
            "hal9",
            "log_info",
            |mut caller: Caller<'_, PluginStore>, ptr: i32, len: i32| {
                let message = read_string_from_memory(&mut caller, ptr, len)?;
                tracing::info!("[Plugin] {}", message);
                Ok(())
            },
        )?;

        linker.func_wrap(
            "hal9",
            "log_error",
            |mut caller: Caller<'_, PluginStore>, ptr: i32, len: i32| {
                let message = read_string_from_memory(&mut caller, ptr, len)?;
                tracing::error!("[Plugin] {}", message);
                Ok(())
            },
        )?;

        // Time functions
        linker.func_wrap(
            "hal9",
            "current_timestamp",
            |_caller: Caller<'_, PluginStore>| -> i64 { chrono::Utc::now().timestamp_millis() },
        )?;

        // Memory functions
        linker.func_wrap(
            "hal9",
            "memory_get",
            |mut caller: Caller<'_, PluginStore>,
             key_ptr: i32,
             key_len: i32,
             value_ptr: i32|
             -> i32 {
                // Check permissions
                let has_perm = caller
                    .data()
                    .plugin_context
                    .permissions
                    .contains(&Permission::Hal9Memory);
                if !has_perm {
                    return -1; // Permission denied
                }

                let key = match read_string_from_memory(&mut caller, key_ptr, key_len) {
                    Ok(k) => k,
                    Err(_) => return -2, // Invalid memory access
                };

                // TODO: Implement actual memory retrieval
                0 // Success but no value found
            },
        )?;

        Ok(())
    }

    /// Extract exported functions from instance
    fn extract_exports(
        &self,
        instance: &Instance,
        store: &mut Store<PluginStore>,
    ) -> Result<HashMap<String, Func>> {
        let mut exports = HashMap::new();

        for export in instance.exports(store) {
            if let Some(func) = export.into_func() {
                exports.insert(export.name().to_string(), func);
            }
        }

        Ok(exports)
    }
}

// ============ Helper Functions ============

fn read_string_from_memory(
    caller: &mut Caller<'_, PluginStore>,
    ptr: i32,
    len: i32,
) -> Result<String> {
    let memory = caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or_else(|| anyhow::anyhow!("Failed to get memory export"))?;

    let data = memory.data(caller);
    let start = ptr as usize;
    let end = start + len as usize;

    if end > data.len() {
        return Err(anyhow::anyhow!("Memory access out of bounds"));
    }

    let bytes = &data[start..end];
    String::from_utf8(bytes.to_vec()).context("Invalid UTF-8 string")
}

impl Default for HostFunctions {
    fn default() -> Self {
        Self {
            log_func: None,
            signal_send: None,
            memory_get: None,
            memory_set: None,
        }
    }
}
