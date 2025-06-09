# HAL9 Testing Guide

Comprehensive guide for testing HAL9 components, from unit tests to production validation.

## 🧪 Testing Philosophy

HAL9 follows a multi-layered testing approach:
- **Unit Tests**: Individual component validation
- **Integration Tests**: Multi-component interaction
- **System Tests**: End-to-end scenarios
- **Performance Tests**: Scalability and efficiency
- **Chaos Tests**: Failure resilience

## 🏗️ Test Structure

### Directory Organization
```
2hal9/
├── hal9-core/src/
│   └── **/tests.rs        # Unit tests
├── hal9-server/tests/     # Integration tests
├── mvp/tests/             # System tests
├── benches/               # Performance benchmarks
└── tests/                 # Cross-crate tests
```

### Test Categories

#### 1. Unit Tests
Located alongside source code for quick feedback.

```rust
// In hal9-core/src/neuron.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_neuron_creation() {
        let config = NeuronConfig::builder()
            .id("test-neuron")
            .layer("L3")
            .build();
            
        let neuron = ClaudeNeuron::new(config).unwrap();
        assert_eq!(neuron.id(), "test-neuron");
    }
}
```

#### 2. Integration Tests
Test component interactions.

```rust
// In hal9-server/tests/integration_test.rs
#[tokio::test]
async fn test_signal_routing() {
    let server = create_test_server().await;
    
    let signal = Signal::new("Test signal");
    let result = server.route_signal(signal).await.unwrap();
    
    assert_eq!(result.target_neuron, "expected-neuron");
}
```

#### 3. System Tests
Full end-to-end scenarios.

```rust
// In mvp/tests/system_test.rs
#[tokio::test]
async fn test_complete_workflow() {
    let system = HAL9System::start_test().await;
    
    // Send complex query
    let response = system
        .process("Build a task management app")
        .await
        .unwrap();
    
    // Verify hierarchical processing
    assert_eq!(response.layers_processed, vec!["L4", "L3", "L2"]);
    assert!(response.has_implementation_details());
}
```

## 🎯 Testing Patterns

### Test Data Builders

```rust
pub struct SignalBuilder {
    content: String,
    layer: String,
    parent_id: Option<Uuid>,
}

impl SignalBuilder {
    pub fn new() -> Self {
        Self {
            content: "default".to_string(),
            layer: "L4".to_string(),
            parent_id: None,
        }
    }
    
    pub fn content(mut self, content: &str) -> Self {
        self.content = content.to_string();
        self
    }
    
    pub fn build(self) -> Signal {
        Signal {
            id: Uuid::new_v4(),
            content: self.content,
            layer: self.layer,
            parent_id: self.parent_id,
            timestamp: Utc::now(),
        }
    }
}

// Usage
let signal = SignalBuilder::new()
    .content("Test signal")
    .layer("L3")
    .build();
```

### Test Fixtures

```rust
pub struct TestFixture {
    pub server: HAL9Server,
    pub neurons: Vec<Box<dyn Neuron>>,
    pub temp_dir: TempDir,
}

impl TestFixture {
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let config = create_test_config(&temp_dir);
        
        let server = HAL9Server::new(config).await?;
        let neurons = create_test_neurons();
        
        Ok(Self { server, neurons, temp_dir })
    }
    
    pub async fn cleanup(self) {
        self.server.shutdown().await;
        // temp_dir auto-cleans on drop
    }
}
```

### Async Test Helpers

```rust
/// Test helper for timeout operations
pub async fn with_timeout<F, T>(duration: Duration, future: F) -> Result<T>
where
    F: Future<Output = T>,
{
    tokio::time::timeout(duration, future)
        .await
        .map_err(|_| anyhow!("Operation timed out"))
}

// Usage
let result = with_timeout(
    Duration::from_secs(5),
    neuron.process(signal)
).await?;
```

## 🧪 Test Types

### Unit Tests

```rust
#[cfg(test)]
mod unit_tests {
    use super::*;
    
    #[test]
    fn test_signal_validation() {
        let signal = Signal::new("");
        assert!(signal.validate().is_err());
    }
    
    #[test]
    fn test_layer_ordering() {
        assert!(Layer::L4 > Layer::L3);
        assert!(Layer::L3 > Layer::L2);
    }
    
    #[tokio::test]
    async fn test_async_processing() {
        let processor = Processor::new();
        let result = processor.process("test").await;
        assert!(result.is_ok());
    }
}
```

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_signal_serialization(content in any::<String>()) {
        let signal = Signal::new(&content);
        let serialized = serde_json::to_string(&signal).unwrap();
        let deserialized: Signal = serde_json::from_str(&serialized).unwrap();
        
        prop_assert_eq!(signal.id, deserialized.id);
        prop_assert_eq!(signal.content, deserialized.content);
    }
    
    #[test]
    fn test_uuid_uniqueness(count in 1..10000) {
        let mut ids = HashSet::new();
        for _ in 0..count {
            ids.insert(Uuid::new_v4());
        }
        prop_assert_eq!(ids.len(), count as usize);
    }
}
```

### Performance Tests

```rust
#[bench]
fn bench_signal_routing(b: &mut Bencher) {
    let router = create_test_router();
    let signal = create_test_signal();
    
    b.iter(|| {
        router.find_target(&signal)
    });
}

#[tokio::test]
async fn test_high_load() {
    let system = create_test_system().await;
    
    let start = Instant::now();
    let mut handles = vec![];
    
    // Send 1000 concurrent requests
    for i in 0..1000 {
        let sys = system.clone();
        handles.push(tokio::spawn(async move {
            sys.process(&format!("Request {}", i)).await
        }));
    }
    
    let results = futures::future::join_all(handles).await;
    let duration = start.elapsed();
    
    // All should succeed
    assert!(results.iter().all(|r| r.is_ok()));
    
    // Should complete within 5 seconds
    assert!(duration < Duration::from_secs(5));
    
    // Calculate throughput
    let throughput = 1000.0 / duration.as_secs_f64();
    println!("Throughput: {:.2} requests/second", throughput);
}
```

### Error Handling Tests

```rust
#[tokio::test]
async fn test_neuron_timeout() {
    let neuron = create_slow_neuron(Duration::from_secs(10));
    let signal = Signal::new("test");
    
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        neuron.process(signal)
    ).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_circuit_breaker() {
    let service = create_flaky_service();
    let breaker = CircuitBreaker::new(service);
    
    // Fail 5 times to open circuit
    for _ in 0..5 {
        let _ = breaker.call("test").await;
    }
    
    // Circuit should be open
    let result = breaker.call("test").await;
    assert!(matches!(result, Err(Error::CircuitOpen)));
}
```

## 🛠️ Test Utilities

### Mock Implementations

```rust
pub struct MockClaude {
    responses: HashMap<String, String>,
    delay: Option<Duration>,
}

impl MockClaude {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            delay: None,
        }
    }
    
    pub fn with_response(mut self, input: &str, output: &str) -> Self {
        self.responses.insert(input.to_string(), output.to_string());
        self
    }
    
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }
}

#[async_trait]
impl ClaudeInterface for MockClaude {
    async fn complete(&self, prompt: &str) -> Result<String> {
        if let Some(delay) = self.delay {
            tokio::time::sleep(delay).await;
        }
        
        self.responses
            .get(prompt)
            .cloned()
            .ok_or_else(|| anyhow!("No mock response for: {}", prompt))
    }
}
```

### Test Database

```rust
pub async fn create_test_db() -> Result<DatabasePool> {
    let db = DatabasePool::new(":memory:").await?;
    db.run_migrations().await?;
    db.seed_test_data().await?;
    Ok(db)
}
```

## 🚀 Running Tests

### Command Line

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_signal_routing

# Run with output
cargo test -- --nocapture

# Run in release mode
cargo test --release

# Run with specific features
cargo test --features "enterprise blockchain"

# Run benchmarks
cargo bench

# Run with coverage
cargo tarpaulin --out Html
```

### Test Scripts

```bash
# Quick test suite
./scripts/test-quick.sh

# Full test suite
./scripts/test-full.sh

# Performance tests
./scripts/test-perf.sh

# Integration tests only
./scripts/test-integration.sh
```

### CI/CD Integration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run tests
        run: |
          cargo test --all-features
          cargo test --doc
          cargo test --release
          
      - name: Run clippy
        run: cargo clippy -- -D warnings
        
      - name: Check formatting
        run: cargo fmt -- --check
```

## 📊 Test Metrics

### Coverage Goals
- **Unit Tests**: 80% line coverage
- **Integration Tests**: All API endpoints
- **System Tests**: All user scenarios
- **Error Paths**: 100% of error types

### Performance Baselines
- **Signal Routing**: < 1ms
- **Neuron Processing**: < 100ms
- **End-to-end**: < 500ms
- **Throughput**: > 1000 req/s

### Quality Metrics
- **No flaky tests**: 100% deterministic
- **Fast execution**: < 30s for full suite
- **Clear failures**: Descriptive error messages
- **Maintainable**: < 50 lines per test

## 🐛 Debugging Tests

### Enable Logging

```rust
// In tests
use env_logger;

#[test]
fn test_with_logging() {
    env_logger::init();
    // Test code
}
```

### Use Test Output

```bash
# See println! output
cargo test -- --nocapture

# See specific test output
cargo test test_name -- --nocapture

# Enable debug logging
RUST_LOG=debug cargo test
```

### Debug Assertions

```rust
#[cfg(debug_assertions)]
debug_assert!(expensive_check());

#[cfg(test)]
assert_eq!(actual, expected, "Custom message: {:?}", context);
```

## 🏆 Best Practices

### Do's
- ✅ Write tests first (TDD)
- ✅ Test one thing per test
- ✅ Use descriptive test names
- ✅ Keep tests independent
- ✅ Mock external dependencies
- ✅ Test edge cases
- ✅ Use property-based testing

### Don'ts
- ❌ Test implementation details
- ❌ Use production data
- ❌ Rely on test order
- ❌ Skip error cases
- ❌ Ignore flaky tests
- ❌ Use hard-coded delays

## 📚 Resources

### Internal
- [MVP Testing Guide](../mvp/TESTING_GUIDE.md)
- [Performance Testing](../deployment/PERFORMANCE_TUNING.md)
- [CI/CD Setup](../deployment/CI_CD_GUIDE.md)

### External
- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)
- [PropTest Docs](https://proptest-rs.github.io/proptest/)

---

*Remember: Tests are documentation that compiles!*