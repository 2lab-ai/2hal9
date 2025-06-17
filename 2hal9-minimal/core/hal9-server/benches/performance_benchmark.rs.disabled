//! Performance benchmarks for 2HAL9 signal processing

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use std::sync::Arc;
use tokio::runtime::Runtime;

use hal9_core::{NeuronSignal, NeuronConfig, ServerConfig, config::{ClaudeConfig, MockResponse}};
use hal9_server::{HAL9Server, MockClaude, ManagedNeuron};

/// Create a test configuration optimized for performance
fn create_performance_config() -> ServerConfig {
    use std::collections::HashMap;
    use hal9_core::config::{MonitoringConfig, CostControls};
    
    // Create fast mock responses
    let mut mock_responses = HashMap::new();
    
    mock_responses.insert("L4".to_string(), vec![
        MockResponse {
            trigger: "bench".to_string(),
            response: "FORWARD_TO: bench-l3-1\nCONTENT: fast".to_string(),
            delay_ms: 1, // Minimal delay for benchmarking
        },
    ]);
    
    mock_responses.insert("L3".to_string(), vec![
        MockResponse {
            trigger: "bench".to_string(),
            response: "FORWARD_TO: bench-l2-1\nCONTENT: fast".to_string(),
            delay_ms: 1,
        },
    ]);
    
    mock_responses.insert("L2".to_string(), vec![
        MockResponse {
            trigger: "bench".to_string(),
            response: "RESULT: Complete".to_string(),
            delay_ms: 1,
        },
    ]);
    
    ServerConfig {
        server_id: "bench-server".to_string(),
        neurons: vec![
            NeuronConfig {
                id: "bench-l4-1".to_string(),
                layer: "L4".to_string(),
                claude_command: "claude".to_string(),
                forward_connections: vec!["bench-l3-1".to_string()],
                backward_connections: vec![],
                settings: HashMap::new(),
            },
            NeuronConfig {
                id: "bench-l3-1".to_string(),
                layer: "L3".to_string(),
                claude_command: "claude".to_string(),
                forward_connections: vec!["bench-l2-1".to_string()],
                backward_connections: vec!["bench-l4-1".to_string()],
                settings: HashMap::new(),
            },
            NeuronConfig {
                id: "bench-l2-1".to_string(),
                layer: "L2".to_string(),
                claude_command: "claude".to_string(),
                forward_connections: vec![],
                backward_connections: vec!["bench-l3-1".to_string()],
                settings: HashMap::new(),
            },
        ],
        claude: ClaudeConfig {
            mode: "mock".to_string(),
            api_key: None,
            model: "bench-model".to_string(),
            temperature: 0.0,
            max_tokens: 100,
            rate_limit: 1000,
            mock_responses,
            fallback_to_mock: false,
            cost_controls: CostControls::default(),
        },
        monitoring: MonitoringConfig {
            enabled: false, // Disable monitoring for pure performance
            metrics_interval: 60,
            log_level: "error".to_string(),
        },
    }
}

fn benchmark_single_signal(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("single_signal_processing", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = create_performance_config();
                let server = Arc::new(HAL9Server::new(config));
                server.start().await.unwrap();
                
                let signal = NeuronSignal::forward(
                    "bench-client",
                    "bench-l4-1",
                    "client",
                    "L4",
                    "benchmark test signal".to_string(),
                );
                
                let start = std::time::Instant::now();
                server.send_signal(signal).await.unwrap();
                
                // Wait for processing
                tokio::time::sleep(Duration::from_millis(10)).await;
                
                let elapsed = start.elapsed();
                server.shutdown().await.unwrap();
                
                black_box(elapsed)
            })
        })
    });
}

fn benchmark_parallel_signals(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("parallel_signal_processing");
    
    for signal_count in [1, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(signal_count),
            signal_count,
            |b, &signal_count| {
                b.iter(|| {
                    rt.block_on(async {
                        let config = create_performance_config();
                        let server = Arc::new(HAL9Server::new(config));
                        server.start().await.unwrap();
                        
                        let start = std::time::Instant::now();
                        
                        // Send multiple signals in parallel
                        let mut handles = vec![];
                        for i in 0..signal_count {
                            let server_clone = server.clone();
                            let handle = tokio::spawn(async move {
                                let signal = NeuronSignal::forward(
                                    &format!("bench-client-{}", i),
                                    "bench-l4-1",
                                    "client",
                                    "L4",
                                    format!("benchmark test signal {}", i),
                                );
                                server_clone.send_signal(signal).await
                            });
                            handles.push(handle);
                        }
                        
                        // Wait for all signals to be sent
                        for handle in handles {
                            handle.await.unwrap().unwrap();
                        }
                        
                        // Wait for processing
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        
                        let elapsed = start.elapsed();
                        server.shutdown().await.unwrap();
                        
                        black_box(elapsed)
                    })
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_cached_responses(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("cached_signal_processing", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = create_performance_config();
                let server = Arc::new(HAL9Server::new(config));
                server.start().await.unwrap();
                
                // Warm up cache with first signal
                let warmup_signal = NeuronSignal::forward(
                    "bench-client",
                    "bench-l2-1", // L2 has caching enabled
                    "L3",
                    "L2",
                    "benchmark cached signal".to_string(),
                );
                server.send_signal(warmup_signal).await.unwrap();
                tokio::time::sleep(Duration::from_millis(10)).await;
                
                // Measure cached response time
                let signal = NeuronSignal::forward(
                    "bench-client",
                    "bench-l2-1",
                    "L3",
                    "L2",
                    "benchmark cached signal".to_string(),
                );
                
                let start = std::time::Instant::now();
                server.send_signal(signal).await.unwrap();
                tokio::time::sleep(Duration::from_millis(5)).await;
                let elapsed = start.elapsed();
                
                server.shutdown().await.unwrap();
                
                black_box(elapsed)
            })
        })
    });
}

fn benchmark_batch_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("batch_signal_processing", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = create_performance_config();
                let server = Arc::new(HAL9Server::new(config));
                server.start().await.unwrap();
                
                let start = std::time::Instant::now();
                
                // Send signals that will be batched
                for i in 0..10 {
                    let signal = NeuronSignal::forward(
                        &format!("bench-client-{}", i),
                        "bench-l4-1",
                        "client",
                        "L4",
                        format!("batch signal {}", i),
                    );
                    server.send_signal(signal).await.unwrap();
                }
                
                // Wait for batch processing
                tokio::time::sleep(Duration::from_millis(100)).await;
                
                let elapsed = start.elapsed();
                server.shutdown().await.unwrap();
                
                black_box(elapsed)
            })
        })
    });
}

fn benchmark_layer_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("layer_latency");
    
    for layer in ["L4", "L3", "L2"].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(layer),
            layer,
            |b, &layer| {
                b.iter(|| {
                    rt.block_on(async {
                        let config = create_performance_config();
                        let neuron_config = config.neurons.iter()
                            .find(|n| n.layer == layer)
                            .unwrap()
                            .clone();
                            
                        let claude = Box::new(MockClaude::new(layer, &config.claude));
                        let neuron = ManagedNeuron::new(neuron_config, claude).unwrap();
                        
                        let signal = NeuronSignal::forward(
                            "bench-client",
                            &neuron.id,
                            "client",
                            layer,
                            "benchmark layer test".to_string(),
                        );
                        
                        let start = std::time::Instant::now();
                        neuron.process_signal(&signal).await.unwrap();
                        let elapsed = start.elapsed();
                        
                        black_box(elapsed)
                    })
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_single_signal,
    benchmark_parallel_signals,
    benchmark_cached_responses,
    benchmark_batch_processing,
    benchmark_layer_latency
);

criterion_main!(benches);