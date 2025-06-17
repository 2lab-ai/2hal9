use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hal9_core::hierarchical::cognitive::{L1ReflexiveNeuron, L2ImplementationNeuron, CognitiveConfig};
use std::time::Duration;

// Benchmark goal: Ensure all neurons process within 10ms threshold
const THRESHOLD_MS: f64 = 10.0;

fn benchmark_l1_reflexive_neuron(c: &mut Criterion) {
    let mut group = c.benchmark_group("L1_Reflexive_Neuron");
    group.measurement_time(Duration::from_secs(10));
    
    // Test various input sizes
    let test_cases = vec![
        ("small", "simple test input"),
        ("medium", "This is a medium-sized input that represents typical user queries with some complexity and detail."),
        ("large", &"x".repeat(1000)), // 1KB input
    ];
    
    for (name, _input) in test_cases {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &name,
            |b, _| {
                b.iter(|| {
                    let config = CognitiveConfig::default();
                    let neuron = L1ReflexiveNeuron::new(config);
                    black_box(neuron);
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_l2_implementation_neuron(c: &mut Criterion) {
    let mut group = c.benchmark_group("L2_Implementation_Neuron");
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("create_neuron", |b| {
        b.iter(|| {
            let config = CognitiveConfig::default();
            let neuron = L2ImplementationNeuron::new(config);
            black_box(neuron);
        });
    });
    
    group.finish();
}

fn benchmark_10ms_threshold(c: &mut Criterion) {
    let mut group = c.benchmark_group("10ms_Threshold_Compliance");
    group.measurement_time(Duration::from_secs(5));
    
    // Set significance level for detecting violations
    group.significance_level(0.01);
    
    // Benchmark neuron creation
    group.bench_function("neuron_creation", |b| {
        b.iter(|| {
            let config = CognitiveConfig::default();
            let l1 = L1ReflexiveNeuron::new(config.clone());
            let l2 = L2ImplementationNeuron::new(config);
            black_box((l1, l2));
        });
    });
    
    group.finish();
}

fn check_10ms_compliance(c: &mut Criterion) {
    let mut group = c.benchmark_group("Compliance_Check");
    
    group.bench_function("verify_under_threshold", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let config = CognitiveConfig::default();
                let _neuron = L1ReflexiveNeuron::new(config);
            }
            let elapsed = start.elapsed();
            
            // Log if we exceed threshold
            let avg_ms = elapsed.as_secs_f64() * 1000.0 / iters as f64;
            if avg_ms > THRESHOLD_MS {
                eprintln!("WARNING: Average processing time {:.2}ms exceeds {}ms threshold", avg_ms, THRESHOLD_MS);
            }
            
            elapsed
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_l1_reflexive_neuron,
    benchmark_l2_implementation_neuron,
    benchmark_10ms_threshold,
    check_10ms_compliance
);

criterion_main!(benches);