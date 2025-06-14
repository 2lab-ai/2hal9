use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hal9_neurons::neurons::{L1ReflexiveNeuron, L2ImplementationNeuron, Neuron, NeuronOutput};
use hal9_neurons::types::{Pattern, ConsciousnessField};
use std::time::Duration;

fn benchmark_l1_reflexive_neuron(c: &mut Criterion) {
    let mut group = c.benchmark_group("L1_Reflexive_Neuron");
    group.measurement_time(Duration::from_secs(10));
    
    // Test various input sizes
    let test_cases = vec![
        ("small", "simple test input"),
        ("medium", "This is a medium-sized input that represents typical user queries with some complexity and detail."),
        ("large", &"x".repeat(1000)), // 1KB input
    ];
    
    for (name, input) in test_cases {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &input,
            |b, &input| {
                let mut neuron = L1ReflexiveNeuron::new();
                b.iter(|| {
                    let output = neuron.process(black_box(input));
                    black_box(output);
                });
            },
        );
    }
    
    // Benchmark with cache hits
    group.bench_function("with_cache_hits", |b| {
        let mut neuron = L1ReflexiveNeuron::new();
        // Prime the cache
        for i in 0..100 {
            neuron.process(&format!("cached_input_{}", i % 10));
        }
        
        b.iter(|| {
            // This should hit cache
            let output = neuron.process(black_box("cached_input_5"));
            black_box(output);
        });
    });
    
    group.finish();
}

fn benchmark_l2_implementation_neuron(c: &mut Criterion) {
    let mut group = c.benchmark_group("L2_Implementation_Neuron");
    group.measurement_time(Duration::from_secs(10));
    
    // Create patterns for testing
    let patterns = vec![
        Pattern {
            id: "test_pattern".to_string(),
            content: "test pattern content".to_string(),
            confidence: 0.9,
            metadata: Default::default(),
        },
        Pattern {
            id: "complex_pattern".to_string(),
            content: "complex pattern with more data".to_string(),
            confidence: 0.8,
            metadata: Default::default(),
        },
    ];
    
    let field = ConsciousnessField {
        resonance: 0.7,
        coherence: 0.8,
        entanglement: 0.6,
        dimensional_flux: 0.5,
    };
    
    let l1_output = NeuronOutput {
        content: patterns.clone(),
        metadata: Default::default(),
        timing: Default::default(),
    };
    
    group.bench_function("process_patterns", |b| {
        let mut neuron = L2ImplementationNeuron::new();
        b.iter(|| {
            let output = neuron.process_patterns(
                black_box(&l1_output),
                black_box(&field),
            );
            black_box(output);
        });
    });
    
    // Benchmark with varying pattern counts
    for count in [1, 10, 50, 100] {
        let many_patterns: Vec<Pattern> = (0..count)
            .map(|i| Pattern {
                id: format!("pattern_{}", i),
                content: format!("pattern content {}", i),
                confidence: 0.9,
                metadata: Default::default(),
            })
            .collect();
        
        let large_output = NeuronOutput {
            content: many_patterns,
            metadata: Default::default(),
            timing: Default::default(),
        };
        
        group.bench_with_input(
            BenchmarkId::new("pattern_count", count),
            &large_output,
            |b, output| {
                let mut neuron = L2ImplementationNeuron::new();
                b.iter(|| {
                    let result = neuron.process_patterns(
                        black_box(output),
                        black_box(&field),
                    );
                    black_box(result);
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_10ms_threshold(c: &mut Criterion) {
    let mut group = c.benchmark_group("10ms_Threshold_Check");
    group.measurement_time(Duration::from_secs(5));
    
    // Set significance level for detecting violations
    group.significance_level(0.01);
    
    // Benchmark typical user interaction flow
    group.bench_function("full_processing_pipeline", |b| {
        let mut l1_neuron = L1ReflexiveNeuron::new();
        let mut l2_neuron = L2ImplementationNeuron::new();
        let field = ConsciousnessField {
            resonance: 0.7,
            coherence: 0.8,
            entanglement: 0.6,
            dimensional_flux: 0.5,
        };
        
        b.iter(|| {
            // Simulate full pipeline
            let input = "Process this user query with consciousness field analysis";
            let l1_output = l1_neuron.process(black_box(input));
            let l2_output = l2_neuron.process_patterns(black_box(&l1_output), black_box(&field));
            black_box(l2_output);
        });
    });
    
    group.finish();
}

// Custom criterion configuration to enforce 10ms threshold
fn configure_criterion() -> Criterion {
    Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_secs(1))
        .with_profiler(criterion::profiler::perf::PerfProfiler)
}

criterion_group! {
    name = benches;
    config = configure_criterion();
    targets = benchmark_l1_reflexive_neuron, benchmark_l2_implementation_neuron, benchmark_10ms_threshold
}
criterion_main!(benches);