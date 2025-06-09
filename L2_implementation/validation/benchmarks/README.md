# HAL9 Performance Benchmarking Suite

This directory contains comprehensive performance benchmarks for the HAL9 hierarchical architecture system. The benchmarks measure various aspects of system performance to ensure optimal operation and track improvements over time.

## Overview

The benchmarking suite is divided into several categories:

1. **Hierarchical System Benchmarks** (`hierarchical_benchmark.rs`)
   - Signal propagation through layers
   - Layer processing performance
   - Neuron activation patterns
   - Consensus mechanisms
   - Learning and gradient calculations

2. **Memory System Benchmarks** (`memory_benchmark.rs`)
   - Embedding storage and retrieval
   - Similarity search performance
   - Pattern matching
   - Memory consolidation
   - Hierarchical memory operations

3. **Network Performance Benchmarks** (`network_benchmark.rs`)
   - Connection pool efficiency
   - Message throughput
   - Service discovery
   - Protocol serialization
   - Network latency and resilience

4. **Integration Benchmarks** (`performance_benchmark.rs`)
   - End-to-end request processing
   - System scalability
   - Real-world scenarios

## Running Benchmarks

### Quick Start

Run all benchmarks:
```bash
./run-benchmarks.sh
```

### Specific Benchmarks

Run individual benchmark suites:
```bash
# Hierarchical system benchmarks only
cargo bench --bench hierarchical_benchmark

# Memory benchmarks only
cargo bench --bench memory_benchmark

# Network benchmarks only
cargo bench --bench network_benchmark

# Specific test within a suite
cargo bench --bench hierarchical_benchmark signal_propagation
```

### Advanced Options

```bash
# Run with stress tests
./run-benchmarks.sh --stress

# Compare with previous results
./run-benchmarks.sh --compare 20250110-145623

# Run with profiling
cargo bench --bench hierarchical_benchmark -- --profile-time 10

# Save baseline for comparison
cargo bench -- --save-baseline my-baseline

# Compare against baseline
cargo bench -- --baseline my-baseline
```

## Benchmark Categories

### 1. Signal Propagation
Tests how efficiently signals flow through the hierarchical layers:
- 1, 3, and 5 layer configurations
- Measures routing overhead
- Tests parallel vs sequential processing

### 2. Layer Processing
Benchmarks each layer type individually:
- Substrate (L1): Low-level operations
- Protocol (L2): Communication protocols
- Cognitive (L3): Pattern recognition
- Orchestration (L4): Coordination logic
- Intelligence (L5): High-level reasoning

### 3. Neuron Activation
Measures neuron performance at scale:
- 10, 100, 1000 neuron configurations
- Activation patterns
- Connection management

### 4. Consensus Mechanisms
Tests distributed consensus performance:
- 3, 5, 10 participant scenarios
- Different proposal types
- Byzantine fault tolerance

### 5. Learning Performance
Benchmarks the learning system:
- Simple, moderate, complex patterns
- Gradient calculation efficiency
- Backpropagation through hierarchy

### 6. Memory Operations
Tests memory subsystem performance:
- Embedding dimensions: 64, 128, 256, 512, 1536
- Similarity search with 100 to 100k entries
- Pattern storage and retrieval
- Memory consolidation processes

### 7. Network Performance
Measures distributed system efficiency:
- Message sizes: 1KB to 1MB
- Connection pool scaling
- Service discovery with up to 1000 services
- Network resilience and failover

## Performance Targets

Based on the hierarchical architecture design, these are our performance targets:

| Metric | Target | Measurement |
|--------|--------|-------------|
| Signal Propagation (5 layers) | < 1ms | P50 latency |
| Layer Processing | < 0.1ms | Per layer P50 |
| Neuron Activation (1000) | < 10ms | Batch activation |
| Consensus (5 nodes) | < 50ms | Agreement time |
| Memory Search (100k) | < 5ms | Similarity search |
| Network Throughput | > 1GB/s | Large messages |
| End-to-end Request | < 10ms | P99 latency |

## Reading Results

### HTML Report
After running benchmarks, open the generated HTML report:
```bash
open benchmark-results/[timestamp]/report.html
```

### JSON Summary
Machine-readable results in:
```bash
benchmark-results/[timestamp]/summary.json
```

### Criterion Output
Detailed Criterion.rs output in:
```bash
target/criterion/*/report/index.html
```

## Continuous Benchmarking

### Integration with CI/CD

Add to your CI pipeline:
```yaml
benchmark:
  script:
    - cargo bench --bench hierarchical_benchmark -- --save-baseline $CI_COMMIT_SHA
    - ./run-benchmarks.sh --compare main
  artifacts:
    paths:
      - benchmark-results/
```

### Performance Regression Detection

The benchmarks automatically detect performance regressions:
- 5% threshold for warnings
- 10% threshold for failures
- Configurable per benchmark

### Historical Tracking

Track performance over time:
```bash
# Generate trend report
./scripts/benchmark-trends.sh --days 30

# Compare releases
./scripts/compare-releases.sh v1.0.0 v1.1.0
```

## Profiling

### CPU Profiling
```bash
# Linux with perf
perf record cargo bench --bench hierarchical_benchmark
perf report

# macOS with Instruments
cargo instruments -t "Time Profiler" --bench hierarchical_benchmark
```

### Memory Profiling
```bash
# With Valgrind (Linux)
valgrind --tool=massif cargo bench --bench memory_benchmark

# With heaptrack
heaptrack cargo bench --bench memory_benchmark
```

### Flame Graphs
```bash
# Generate flame graph
cargo flamegraph --bench hierarchical_benchmark -- --bench
```

## Optimization Guide

Based on benchmark results, consider these optimizations:

1. **Signal Propagation**
   - Use parallel processing for independent layers
   - Implement signal batching
   - Cache routing decisions

2. **Memory Performance**
   - Use appropriate embedding dimensions
   - Implement LRU caching
   - Optimize similarity algorithms

3. **Network Efficiency**
   - Tune connection pool sizes
   - Implement message batching
   - Use compression for large messages

## Contributing

When adding new benchmarks:

1. Follow the existing pattern in benchmark files
2. Include multiple input sizes/configurations
3. Document what the benchmark measures
4. Add performance targets
5. Update this README

## Troubleshooting

### Out of Memory
- Reduce benchmark iterations: `--sample-size 10`
- Run specific benchmarks instead of full suite
- Increase system memory or use swap

### Inconsistent Results
- Ensure system is idle during benchmarks
- Disable CPU frequency scaling
- Run multiple times and average results
- Use `--measurement-time 30` for longer runs

### Benchmark Compilation Errors
- Update dependencies: `cargo update`
- Clean build: `cargo clean`
- Check Rust version: `rustc --version`

## Related Documentation

- [Performance Optimization Guide](../docs/L4_architecture/L4_PERFORMANCE_OPTIMIZATION_ARCHITECTURE.md)
- [System Architecture](../docs/L4_architecture/L4_SYSTEM_ARCHITECTURE.md)
- [Monitoring Guide](../docs/L1_operational/L1_MONITORING_GUIDE.md)