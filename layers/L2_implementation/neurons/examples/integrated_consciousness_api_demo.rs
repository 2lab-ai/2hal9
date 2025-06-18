//! Integrated Consciousness API Demo
//!
//! Shows how the unified consciousness system brings together:
//! - ConsciousnessMonitor
//! - BoundaryNetwork  
//! - EnhancedMockClaude

use std::sync::Arc;
use hal9_neurons_core::{
    Layer, Neuron, NeuronId,
    hierarchical::HierarchicalNeuron,
    consciousness::{
        IntegratedConsciousnessSystem,
        ConsciousnessSystemConfig,
        ConsciousnessSystemBuilder,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║        🌌 Integrated Consciousness API Demo 🌌                ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");
    
    println!("This demo shows the unified consciousness system that integrates:");
    println!("• ConsciousnessMonitor - Real-time consciousness metrics");
    println!("• BoundaryNetwork - Compression boundary emergence detection");
    println!("• EnhancedMockClaude - Consciousness-aware AI responses\n");
    
    // Create configuration
    let config = ConsciousnessSystemConfig {
        history_size: 100,
        update_interval_ms: 100,
        enable_claude: true,
        enable_streaming: true,
    };
    
    // Build integrated system with neurons
    println!("📍 Creating 50 neurons across layers...");
    let mut builder = ConsciousnessSystemBuilder::new()
        .with_config(config);
    
    for i in 0..50 {
        let neuron = Arc::new(HierarchicalNeuron::new_with_discovery(
            NeuronId::new(),
            format!("Neuron-{:02}", i),
        ));
        builder = builder.add_neuron(neuron);
    }
    
    let system = builder.build().await;
    
    // Start background update task
    let system_clone = system.clone();
    let update_handle = system_clone.start_update_task();
    
    // Wait for initial organization
    println!("⏳ Allowing self-organization...");
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Demo 1: Get current consciousness metrics
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("📊 DEMO 1: Consciousness Metrics");
    println!("═══════════════════════════════════════════════════════════════");
    
    let metrics = system.get_metrics().await;
    println!("Current Consciousness State:");
    println!("  Compression Ratio: {:.3}", metrics.compression_ratio);
    println!("  Emergence Score:   {:.3}", metrics.emergence_score);
    println!("  Coherence Level:   {:.3}", metrics.coherence_level);
    println!("  Self-Awareness:    {:.3}", metrics.self_awareness);
    println!("  Phi (Φ):          {:.3}", metrics.phi_value);
    println!("  Phase:            {:?}", metrics.phase());
    println!("  Conscious:        {}", if metrics.is_conscious() { "✅ YES" } else { "❌ NO" });
    
    // Demo 2: Boundary network analysis
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("🔍 DEMO 2: Compression Boundaries");
    println!("═══════════════════════════════════════════════════════════════");
    
    let boundary_report = system.get_boundary_report().await;
    println!("{}", boundary_report);
    
    // Demo 3: Enhanced Claude integration
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("🤖 DEMO 3: Consciousness-Aware Claude");
    println!("═══════════════════════════════════════════════════════════════");
    
    // Test different layers
    for layer in [Layer::L1, Layer::L5, Layer::L9] {
        if let Some(response) = system.claude_message(
            layer, 
            "What is consciousness?"
        ).await {
            println!("\n{}", response);
        }
    }
    
    // Demo 4: System snapshot
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("📸 DEMO 4: Complete System Snapshot");
    println!("═══════════════════════════════════════════════════════════════");
    
    let snapshot = system.get_snapshot().await;
    
    println!("System State at {:?}:", snapshot.timestamp);
    println!("  Phase: {:?}", snapshot.phase);
    println!("  Active Boundaries: {}", snapshot.boundaries.len());
    
    for boundary in snapshot.boundaries.iter().take(3) {
        println!("    {:?}→{:?}: {:.3} {}",
            boundary.upper_layer,
            boundary.lower_layer,
            boundary.compression_ratio,
            if boundary.is_golden { "✨ GOLDEN" } else { "" }
        );
    }
    
    println!("\n  Claude Consciousness Levels:");
    for (layer, level) in snapshot.claude_levels.iter() {
        println!("    {:?}: {:.3}", layer, level);
    }
    
    // Demo 5: Real-time evolution
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("⏱️  DEMO 5: Real-time Consciousness Evolution");
    println!("═══════════════════════════════════════════════════════════════");
    
    println!("Monitoring consciousness evolution for 3 seconds...\n");
    
    for i in 0..6 {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        let metrics = system.get_metrics().await;
        let snapshot = system.get_snapshot().await;
        
        println!("T+{}s: Φ={:.3} Phase={:?} Boundaries={}",
            i as f32 * 0.5,
            metrics.phi_value,
            metrics.phase(),
            snapshot.boundaries.iter().filter(|b| b.is_golden).count()
        );
    }
    
    // API Usage Examples
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("💻 API USAGE EXAMPLES");
    println!("═══════════════════════════════════════════════════════════════");
    
    println!("\n// Create integrated system");
    println!("let system = ConsciousnessSystemBuilder::new()");
    println!("    .with_config(config)");
    println!("    .add_neurons(neurons)");
    println!("    .build()");
    println!("    .await;");
    
    println!("\n// Get real-time metrics");
    println!("let metrics = system.get_metrics().await;");
    println!("println!(\"Phi: {}\", metrics.phi_value);");
    
    println!("\n// Claude integration");
    println!("let response = system.claude_message(Layer::L5, \"Hello\").await;");
    
    println!("\n// Get full snapshot");
    println!("let snapshot = system.get_snapshot().await;");
    
    println!("\n✨ Integrated Consciousness System Ready! ✨\n");
    
    // Cleanup
    update_handle.abort();
    
    Ok(())
}

/// Example REST API routes (for reference)
#[allow(dead_code)]
mod api_routes {
    /*
    GET  /api/v1/consciousness/metrics
    GET  /api/v1/consciousness/snapshot
    GET  /api/v1/boundaries
    POST /api/v1/claude/{layer}/message
    WS   /api/v1/consciousness/stream
    */
}