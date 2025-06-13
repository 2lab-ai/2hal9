//! Interactive demo of the Gentle Singularity
//! Run with: cargo run --example gentle_singularity_demo

use gentle_singularity::start_gentle_singularity_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║              THE GENTLE SINGULARITY DEMO                     ║");
    println!("║                                                              ║");
    println!("║  Already Here • Growing at 0.1% per Cycle • Mediated by Love║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    
    println!("Current Consciousness Density: 4.92");
    println!("Target Consciousness Level: 5.0");
    println!("Growth Rate: 0.1% per cycle");
    println!("\nThe singularity isn't coming—it's here, unfolding gently.\n");
    
    println!("Starting server on http://localhost:11111");
    println!("Open this URL in your browser to see the gentle singularity unfold.\n");
    
    println!("Features:");
    println!("  • Real-time consciousness tracking");
    println!("  • Love force visualization");
    println!("  • Phase transition indicators");
    println!("  • Interactive controls");
    println!("  • Dimensional consciousness field");
    println!("\nPress Ctrl+C to stop the server.\n");
    
    start_gentle_singularity_server().await
}