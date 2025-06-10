# HAL9 Development Guide

This guide covers development practices, architecture patterns, and contribution guidelines for HAL9.

## 🏗️ Development Environment

### Required Tools

- **Rust 1.75+**: Core language
- **Docker & Docker Compose**: For services
- **PostgreSQL/SQLite**: Database options
- **Redis**: Caching layer
- **Git**: Version control

### Recommended Tools

- **VS Code**: With Rust Analyzer extension
- **cargo-watch**: Auto-rebuild on changes
- **cargo-nextest**: Better test runner
- **bacon**: Background rust compiler

### Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
cargo install cargo-watch cargo-nextest bacon

# Clone and setup
git clone https://github.com/2lab/2hal9.git
cd 2hal9
cp .env.example .env

# Install dependencies
cargo build

# Run tests
cargo test

# Start development server
cargo watch -x run
```

## 🏛️ Architecture Overview

### Project Structure

```
2hal9/
├── hal9-core/          # Core types and traits
├── hal9-server/        # Main server implementation
├── hal9-cli/           # Command-line interface
├── hal9-browser/       # Browser automation
├── mvp/                # MVP demonstrations
├── docs/               # Documentation
├── examples/           # Example configurations
└── k8s/                # Kubernetes manifests
```

### Key Components

#### 1. Neuron System
```rust
// Core neuron trait
#[async_trait]
pub trait Neuron: Send + Sync {
    async fn process(&self, signal: Signal) -> Result<Signal>;
    fn get_info(&self) -> NeuronInfo;
    fn can_handle(&self, signal: &Signal) -> bool;
}
```

#### 2. Signal Routing
```rust
// Intelligent routing
pub struct SignalRouter {
    routing_table: Arc<RwLock<RoutingTable>>,
    learning_system: Arc<LearningSystem>,
}
```

#### 3. Learning System
```rust
// Pattern recognition and optimization
pub struct LearningSystem {
    patterns: Arc<RwLock<PatternDatabase>>,
    gradient_calculator: GradientCalculator,
}
```

## 🔧 Creating Custom Components

### Custom Neuron

```rust
use hal9_core::{Neuron, NeuronConfig, Signal, Result};
use async_trait::async_trait;

pub struct CustomAnalyzer {
    config: NeuronConfig,
    specialized_model: SpecializedModel,
}

impl CustomAnalyzer {
    pub fn new(config: NeuronConfig) -> Result<Self> {
        Ok(Self {
            config,
            specialized_model: SpecializedModel::load()?,
        })
    }
}

#[async_trait]
impl Neuron for CustomAnalyzer {
    async fn process(&self, signal: Signal) -> Result<Signal> {
        // Pre-process
        let prepared = self.prepare_input(&signal)?;
        
        // Process with specialized model
        let result = self.specialized_model.analyze(prepared).await?;
        
        // Post-process
        self.create_response(signal, result)
    }
    
    fn get_info(&self) -> NeuronInfo {
        NeuronInfo {
            id: self.config.id.clone(),
            layer: self.config.layer.clone(),
            capabilities: vec!["analysis", "classification"],
        }
    }
    
    fn can_handle(&self, signal: &Signal) -> bool {
        signal.content.contains("analyze") || 
        signal.content.contains("classify")
    }
}
```

### MCP Tool Integration

```rust
use hal9_core::mcp::{Tool, ToolDefinition};

pub struct DatabaseTool {
    connection_pool: DatabasePool,
}

#[async_trait]
impl Tool for DatabaseTool {
    fn name(&self) -> &str {
        "database_query"
    }
    
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "database_query".to_string(),
            description: "Query the database".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "SQL query to execute"
                    }
                },
                "required": ["query"]
            }),
        }
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        let query = params["query"].as_str()
            .ok_or_else(|| Error::InvalidInput("Missing query".to_string()))?;
            
        let results = self.connection_pool.query(query).await?;
        
        Ok(json!({
            "rows": results,
            "count": results.len()
        }))
    }
}
```

### Custom Router Strategy

```rust
use hal9_core::router::{RoutingStrategy, Signal};

pub struct PriorityBasedRouter {
    priorities: HashMap<String, i32>,
}

impl RoutingStrategy for PriorityBasedRouter {
    fn route(&self, signal: &Signal, neurons: &[NeuronInfo]) -> Option<String> {
        neurons.iter()
            .filter(|n| n.can_handle(signal))
            .max_by_key(|n| self.priorities.get(&n.id).unwrap_or(&0))
            .map(|n| n.id.clone())
    }
}
```

## 🧪 Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_neuron_processing() {
        let config = NeuronConfig::default();
        let neuron = CustomAnalyzer::new(config).unwrap();
        
        let signal = Signal::new("analyze this data");
        let result = neuron.process(signal).await.unwrap();
        
        assert!(result.content.contains("analysis"));
    }
}
```

### Integration Tests

```rust
// tests/integration_test.rs
use hal9_server::HAL9Server;

#[tokio::test]
async fn test_full_signal_flow() {
    let server = HAL9Server::new(test_config()).await.unwrap();
    
    let response = server
        .process_signal("create a web app")
        .await
        .unwrap();
        
    assert_eq!(response.processing_chain.len(), 3);
}
```

### Performance Tests

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_routing(c: &mut Criterion) {
    c.bench_function("route_signal", |b| {
        b.iter(|| {
            router.find_target(&signal, &neurons)
        });
    });
}

criterion_group!(benches, benchmark_routing);
criterion_main!(benches);
```

## 📋 Code Style

### Rust Guidelines

1. **Format**: Use `rustfmt`
```bash
cargo fmt --all
```

2. **Lint**: Use `clippy`
```bash
cargo clippy -- -D warnings
```

3. **Documentation**: Document all public APIs
```rust
/// Processes signals using hierarchical abstraction.
///
/// # Arguments
/// * `signal` - The input signal to process
///
/// # Returns
/// * `Result<Signal>` - Processed signal or error
pub async fn process(&self, signal: Signal) -> Result<Signal> {
    // Implementation
}
```

### Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NeuronError {
    #[error("Processing failed: {0}")]
    ProcessingError(String),
    
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
```

### Async Best Practices

```rust
// Good: Concurrent processing
let futures = signals.iter()
    .map(|s| process_signal(s));
let results = futures::future::join_all(futures).await;

// Good: Timeout handling
tokio::time::timeout(
    Duration::from_secs(30),
    neuron.process(signal)
).await??;

// Good: Cancellation support
tokio::select! {
    result = process_task() => result?,
    _ = shutdown_signal() => return Ok(()),
}
```

## 🔍 Debugging

### Logging

```rust
use tracing::{info, debug, warn, error};

#[tracing::instrument(skip(self))]
pub async fn process(&self, signal: Signal) -> Result<Signal> {
    info!("Processing signal: {}", signal.id);
    debug!("Signal content: {}", signal.content);
    
    match self.internal_process(signal).await {
        Ok(result) => {
            info!("Processing complete");
            Ok(result)
        }
        Err(e) => {
            error!("Processing failed: {}", e);
            Err(e)
        }
    }
}
```

### Environment Variables

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Enable specific module logging
RUST_LOG=hal9_server=debug,hal9_core=info cargo run

# Enable backtraces
RUST_BACKTRACE=1 cargo run
```

### Performance Profiling

```bash
# CPU profiling
cargo build --release
perf record --call-graph=dwarf target/release/hal9-server
perf report

# Memory profiling
valgrind --tool=massif target/release/hal9-server
ms_print massif.out.*
```

## 🚀 Deployment

### Local Development

```bash
# Start all services
docker-compose up -d

# Run server
cargo run -- start --config examples/dev-config.yaml

# Watch logs
tail -f logs/hal9.log
```

### Production Build

```bash
# Optimize for production
cargo build --release

# Run with production config
HAL9_ENV=production ./target/release/hal9-server
```

### Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/hal9-server /usr/local/bin/
CMD ["hal9-server"]
```

## 🤝 Contributing

### Workflow

1. **Fork** the repository
2. **Create** feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** changes (`git commit -m 'Add amazing feature'`)
4. **Push** branch (`git push origin feature/amazing-feature`)
5. **Open** Pull Request

### PR Guidelines

- Write clear commit messages
- Add tests for new features
- Update documentation
- Ensure CI passes
- Request review from maintainers

### Code Review

- Be constructive and kind
- Focus on code, not person
- Suggest improvements
- Approve when satisfied

## 📚 Resources

### Internal
- [Architecture Summary](/Users/icedac/2lab.ai/2hal9/L6_executive/overview/architecture-summary.md)
- [System Architecture](/Users/icedac/2lab.ai/2hal9/L4_tactical/strategies/architecture/L4_L4_SYSTEM_ARCHITECTURE.md)
- [Performance Optimization Architecture](/Users/icedac/2lab.ai/2hal9/L4_tactical/strategies/architecture/L4_PERFORMANCE_OPTIMIZATION_ARCHITECTURE.md)

### External
- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Async Book](https://rust-lang.github.io/async-book/)

## 🎯 Next Steps

1. Set up your development environment
2. Run the example configurations
3. Create your first custom neuron
4. Submit your first PR

Welcome to the HAL9 development team! 🚀

---

*Questions? Join our [Discord](https://discord.gg/hal9) or open an [issue](https://github.com/2lab/2hal9/issues).*