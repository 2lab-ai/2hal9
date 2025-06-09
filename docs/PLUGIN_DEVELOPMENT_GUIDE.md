# HAL9 Plugin Development Guide

## Overview

HAL9's WebAssembly plugin system allows developers to extend the platform with custom neurons, tools, memory providers, and learning algorithms. Plugins run in a secure sandbox with controlled access to system resources.

## Architecture

```
┌─────────────────────────────────────────────────┐
│                Plugin (.wasm)                    │
│  • Custom logic                                  │
│  • HAL9 SDK integration                          │
│  • Exported functions                            │
└─────────────────────┬───────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────┐
│              WASM Runtime (Wasmtime)             │
│  • Sandboxed execution                           │
│  • Resource limits                               │
│  • Host function bindings                        │
└─────────────────────┬───────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────┐
│               Plugin Manager                      │
│  • Plugin lifecycle                              │
│  • Capability registry                           │
│  • Security policies                             │
└─────────────────────┬───────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────┐
│                HAL9 Core                         │
│  • Signal routing                                │
│  • Memory system                                 │
│  • Neuron orchestration                          │
└─────────────────────────────────────────────────┘
```

## Getting Started

### 1. Prerequisites

- Rust 1.75 or later
- `wasm32-wasi` target: `rustup target add wasm32-wasi`
- HAL9 Plugin SDK

### 2. Create a New Plugin

```bash
# Create a new plugin project
cargo new --lib my-hal9-plugin
cd my-hal9-plugin

# Add dependencies
cat >> Cargo.toml << EOF
[lib]
crate-type = ["cdylib"]

[dependencies]
hal9-plugin-sdk = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.10", features = ["v4", "serde"] }
EOF
```

### 3. Define Your Plugin

```rust
use hal9_plugin_sdk::*;

// Define plugin metadata
hal9_plugin! {
    metadata: {
        name: "My Custom Neuron",
        version: "0.1.0",
        author: "Your Name",
        description: "A custom neuron that processes signals",
        license: "MIT",
    },
    capabilities: [
        PluginCapability::NeuronType {
            layer: "L2".to_string(),
            neuron_type: "custom_processor".to_string(),
            description: "Processes signals with custom logic".to_string(),
        },
    ],
    permissions: [
        Permission::Hal9Signal,
        Permission::Hal9Memory,
        Permission::SystemTime,
    ]
}
```

## Plugin Types

### 1. Neuron Plugin

Extends HAL9 with custom neuron types for signal processing.

```rust
pub struct MyNeuron {
    config: NeuronConfig,
    state: NeuronState,
}

impl NeuronPlugin for MyNeuron {
    fn process_signal(&mut self, signal: PluginSignal) -> Result<PluginSignal, PluginError> {
        // Process the signal
        let processed = format!("Processed: {}", signal.content);
        
        Ok(PluginSignal {
            id: signal.id,
            content: processed,
            signal_type: "processed".to_string(),
            metadata: signal.metadata,
            timestamp: current_timestamp(),
        })
    }
    
    fn get_state(&self) -> NeuronState {
        self.state.clone()
    }
    
    fn update_config(&mut self, config: serde_json::Value) -> Result<(), PluginError> {
        // Update configuration
        Ok(())
    }
}

// Export the neuron
neuron_plugin!(MyNeuron);
```

### 2. Tool Provider Plugin

Adds custom tools that can be called by neurons.

```rust
pub struct MyTool;

impl ToolPlugin for MyTool {
    fn execute(&mut self, params: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, PluginError> {
        // Execute tool logic
        let result = perform_operation(&params)?;
        Ok(serde_json::to_value(result)?)
    }
    
    fn validate_params(&self, params: &HashMap<String, serde_json::Value>) -> Result<(), PluginError> {
        // Validate parameters
        if !params.contains_key("required_param") {
            return err(ErrorCode::InvalidInput, "Missing required parameter");
        }
        Ok(())
    }
}
```

### 3. Memory Provider Plugin

Implements custom memory storage backends.

```rust
pub struct MyMemoryProvider {
    storage: HashMap<String, Vec<u8>>,
}

impl MemoryPlugin for MyMemoryProvider {
    fn store(&mut self, key: String, value: Vec<u8>, metadata: HashMap<String, String>) -> Result<(), PluginError> {
        self.storage.insert(key, value);
        Ok(())
    }
    
    fn retrieve(&self, key: &str) -> Result<Option<(Vec<u8>, HashMap<String, String>)>, PluginError> {
        Ok(self.storage.get(key).cloned().map(|v| (v, HashMap::new())))
    }
    
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, PluginError> {
        // Implement search logic
        Ok(vec![])
    }
    
    fn delete(&mut self, key: &str) -> Result<bool, PluginError> {
        Ok(self.storage.remove(key).is_some())
    }
}
```

## Plugin API

### Host Functions

Plugins can call these functions provided by the host:

```rust
// Logging
log_debug("Debug message");
log_info("Info message");
log_warn("Warning message");
log_error("Error message");

// Time
let timestamp = current_timestamp();

// Memory access
let value = memory_get("key")?;
memory_set("key", b"value")?;

// Metrics (coming soon)
metric_increment("my_metric", 1.0);
metric_gauge("queue_size", 42.0);
```

### Permissions

Plugins must declare required permissions:

- `NetworkHttp/NetworkHttps`: HTTP/HTTPS access
- `FileRead/FileWrite`: File system access
- `SystemTime`: Current time access
- `SystemRandom`: Random number generation
- `SystemEnv`: Environment variables
- `Hal9Signal`: Signal manipulation
- `Hal9Memory`: Memory system access
- `Hal9Metrics`: Metrics reporting
- `Hal9Learning`: Learning system access

## Building and Packaging

### 1. Build the Plugin

```bash
# Build for WASM
cargo build --target wasm32-wasi --release

# The WASM file will be at:
# target/wasm32-wasi/release/my_hal9_plugin.wasm
```

### 2. Create Plugin Package

Create `manifest.json`:

```json
{
  "api_version": 1,
  "metadata": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "My HAL9 Plugin",
    "version": "0.1.0",
    "author": "Your Name",
    "description": "Description of your plugin",
    "license": "MIT",
    "repository": "https://github.com/you/my-plugin",
    "capabilities": [...],
    "requirements": {
      "min_hal9_version": "0.1.0",
      "max_memory_mb": 64,
      "required_permissions": [...],
      "dependencies": []
    }
  },
  "files": {
    "wasm": "my_hal9_plugin.wasm",
    "readme": "README.md",
    "license": "LICENSE",
    "examples": ["examples/basic.rs"]
  }
}
```

### 3. Package the Plugin

```bash
# Create .hal9 package
zip -r my-plugin.hal9 manifest.json *.wasm README.md LICENSE examples/
```

## Testing

### Local Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_signal_processing() {
        let mut neuron = MyNeuron::default();
        let signal = PluginSignal {
            id: Uuid::new_v4(),
            content: "Test signal".to_string(),
            signal_type: "test".to_string(),
            metadata: HashMap::new(),
            timestamp: 0,
        };
        
        let result = neuron.process_signal(signal).unwrap();
        assert!(result.content.contains("Processed"));
    }
}
```

### Integration Testing

```bash
# Load plugin in development mode
hal9 plugin load --dev ./target/wasm32-wasi/release/

# Test the plugin
hal9 signal send --layer L2 --content "Test message"

# Check plugin logs
hal9 plugin logs my-plugin
```

## Security Considerations

### Resource Limits

Plugins are subject to resource limits:

- **Memory**: Default 64MB (configurable)
- **CPU**: 25% of one core
- **Execution time**: 5 seconds per call
- **File size**: 10MB max
- **Network connections**: 10 concurrent

### Sandboxing

Plugins run in a WASM sandbox with:

- No direct file system access (unless permitted)
- No network access (unless permitted)
- No process spawning
- Limited system calls
- Memory isolation

### Best Practices

1. **Validate all inputs**: Check parameters thoroughly
2. **Handle errors gracefully**: Return proper error codes
3. **Respect timeouts**: Keep operations fast
4. **Use logging judiciously**: Don't spam logs
5. **Clean up resources**: Free memory and connections

## Publishing

### 1. Prepare for Publication

- Write comprehensive documentation
- Include usage examples
- Add proper error handling
- Test thoroughly
- Choose an appropriate license

### 2. Submit to Registry

```bash
# Login to HAL9 registry
hal9 auth login

# Publish plugin
hal9 plugin publish my-plugin.hal9
```

### 3. Version Management

Follow semantic versioning:
- MAJOR: Breaking API changes
- MINOR: New features (backward compatible)
- PATCH: Bug fixes

## Examples

### Complete Examples

1. **Sentiment Analyzer**: `/plugins/examples/sentiment-analyzer/`
   - Neuron plugin for text sentiment analysis
   - Demonstrates signal processing and caching

2. **Web Scraper**: `/plugins/examples/web-scraper/`
   - Tool provider for web scraping
   - Shows parameter validation and error handling

3. **Custom Memory**: `/plugins/examples/custom-memory/`
   - Memory provider with custom storage
   - Illustrates search implementation

## Troubleshooting

### Common Issues

1. **Plugin fails to load**
   - Check WASM target: `wasm32-wasi`
   - Verify manifest.json syntax
   - Ensure all permissions are declared

2. **Permission denied errors**
   - Add required permissions to manifest
   - Check security policy allows access

3. **Memory errors**
   - Reduce memory usage
   - Check for memory leaks
   - Increase memory limit if needed

4. **Performance issues**
   - Profile WASM execution
   - Optimize hot paths
   - Use caching appropriately

## Advanced Topics

### Inter-Plugin Communication

```rust
// Call another plugin
let result = call_plugin("other-plugin-id", "method", params)?;
```

### Custom Capabilities

Extend HAL9 with new capability types:

```rust
PluginCapability::Custom {
    capability_type: "my_custom_type".to_string(),
    config: serde_json::json!({
        "feature": "value"
    }),
}
```

### Plugin Composition

Combine multiple plugins:

```rust
// In manifest.json
"dependencies": [
    {
        "plugin_id": "base-plugin",
        "min_version": "1.0.0"
    }
]
```

## Resources

- [HAL9 Plugin SDK Documentation](https://docs.hal9.ai/plugin-sdk)
- [WebAssembly Reference](https://webassembly.org/)
- [Example Plugins](https://github.com/2lab/hal9/tree/main/plugins/examples)
- [Plugin Registry](https://plugins.hal9.ai)
- [Community Forum](https://forum.hal9.ai/c/plugins)

---

*Happy plugin development! 🚀*