#!/bin/bash
# A2A (Agent-to-Agent) Communication Demo
# Shows direct neuron-to-neuron communication for self-organization

set -e

echo "🔗 HAL9 A2A Communication Demo"
echo "=============================="
echo ""
echo "This demo shows how neurons communicate directly with each other"
echo "to self-organize without central coordination."
echo ""

# Create the A2A demo
cat > /tmp/a2a_demo.rs << 'EOF'
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;

// A2A Message types
#[derive(Debug, Clone)]
enum A2AMessage {
    Hello { from: String, properties: NeuronProperties },
    Compatibility { from: String, score: f32 },
    JoinLayer { from: String, layer: usize },
    Acknowledge { from: String },
}

#[derive(Debug, Clone)]
struct NeuronProperties {
    speed: f32,
    abstraction: f32,
    energy: f32,
}

// Simple neuron that can communicate A2A
struct A2ANeuron {
    id: String,
    properties: NeuronProperties,
    inbox: Arc<Mutex<Vec<A2AMessage>>>,
    discovered_layer: Arc<Mutex<Option<usize>>>,
    peers: Arc<Mutex<Vec<String>>>,
}

impl A2ANeuron {
    fn new(id: String) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        Self {
            id: id.clone(),
            properties: NeuronProperties {
                speed: rng.gen(),
                abstraction: rng.gen(),
                energy: rng.gen(),
            },
            inbox: Arc::new(Mutex::new(Vec::new())),
            discovered_layer: Arc::new(Mutex::new(None)),
            peers: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    // Send message directly to another neuron
    fn send_message(&self, recipient: &A2ANeuron, message: A2AMessage) {
        println!("  {} → {} : {:?}", self.id, recipient.id, 
                match &message {
                    A2AMessage::Hello { .. } => "HELLO",
                    A2AMessage::Compatibility { score, .. } => 
                        &format!("COMPAT({})", score),
                    A2AMessage::JoinLayer { layer, .. } => 
                        &format!("JOIN_L{}", layer),
                    A2AMessage::Acknowledge { .. } => "ACK",
                });
        
        recipient.inbox.lock().unwrap().push(message);
    }
    
    // Process incoming messages
    fn process_messages(&self, network: &HashMap<String, Arc<A2ANeuron>>) {
        let messages: Vec<_> = {
            let mut inbox = self.inbox.lock().unwrap();
            inbox.drain(..).collect()
        };
        
        for msg in messages {
            match msg {
                A2AMessage::Hello { from, properties } => {
                    // Calculate compatibility
                    let compat = self.calculate_compatibility(&properties);
                    
                    // Send compatibility score back
                    if let Some(sender) = network.get(&from) {
                        self.send_message(
                            sender,
                            A2AMessage::Compatibility { 
                                from: self.id.clone(), 
                                score: compat 
                            }
                        );
                    }
                    
                    // Remember peer if compatible
                    if compat > 0.7 {
                        self.peers.lock().unwrap().push(from);
                    }
                }
                
                A2AMessage::Compatibility { from, score } => {
                    if score > 0.7 {
                        self.peers.lock().unwrap().push(from.clone());
                        
                        // Share layer discovery if we have one
                        if let Some(layer) = *self.discovered_layer.lock().unwrap() {
                            if let Some(peer) = network.get(&from) {
                                self.send_message(
                                    peer,
                                    A2AMessage::JoinLayer { 
                                        from: self.id.clone(), 
                                        layer 
                                    }
                                );
                            }
                        }
                    }
                }
                
                A2AMessage::JoinLayer { from: _, layer } => {
                    // Consider joining suggested layer
                    let mut current = self.discovered_layer.lock().unwrap();
                    if current.is_none() {
                        *current = Some(layer);
                        println!("  {} discovered layer L{}", self.id, layer);
                    }
                }
                
                A2AMessage::Acknowledge { .. } => {
                    // Just acknowledgment
                }
            }
        }
    }
    
    fn calculate_compatibility(&self, other: &NeuronProperties) -> f32 {
        let speed_diff = (self.properties.speed - other.speed).abs();
        let abs_diff = (self.properties.abstraction - other.abstraction).abs();
        let energy_diff = (self.properties.energy - other.energy).abs();
        
        1.0 - (speed_diff + abs_diff + energy_diff) / 3.0
    }
    
    fn natural_layer(&self) -> usize {
        let score = self.properties.speed * 0.3 + 
                   self.properties.abstraction * 0.5 + 
                   self.properties.energy * 0.2;
        
        ((score * 8.0) as usize + 1).min(9)
    }
}

fn demonstrate_a2a_communication() {
    println!("🚀 Initializing A2A Network with 10 neurons...\n");
    
    // Create network
    let mut network: HashMap<String, Arc<A2ANeuron>> = HashMap::new();
    for i in 0..10 {
        let id = format!("N{:02}", i);
        network.insert(id.clone(), Arc::new(A2ANeuron::new(id)));
    }
    
    // Phase 1: Discovery - neurons say hello
    println!("📡 Phase 1: Discovery Protocol");
    println!("─────────────────────────────");
    
    let neurons: Vec<_> = network.values().cloned().collect();
    
    // Each neuron broadcasts hello to random peers
    for neuron in &neurons {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        
        // Pick 3 random peers
        let mut peers = neurons.clone();
        peers.shuffle(&mut rng);
        
        for peer in peers.iter().take(3) {
            if peer.id != neuron.id {
                neuron.send_message(
                    peer,
                    A2AMessage::Hello {
                        from: neuron.id.clone(),
                        properties: neuron.properties.clone(),
                    }
                );
            }
        }
    }
    
    // Process messages
    thread::sleep(Duration::from_millis(100));
    println!("\n🔄 Processing discovery messages...");
    
    for neuron in &neurons {
        neuron.process_messages(&network);
    }
    
    // Phase 2: Layer Formation
    println!("\n🌊 Phase 2: Layer Formation");
    println!("──────────────────────────");
    
    // Pioneer neurons discover their natural layers
    for (i, neuron) in neurons.iter().enumerate() {
        if i < 3 {  // First few neurons are pioneers
            let layer = neuron.natural_layer();
            *neuron.discovered_layer.lock().unwrap() = Some(layer);
            println!("  {} pioneers layer L{}", neuron.id, layer);
            
            // Tell compatible peers
            let peers = neuron.peers.lock().unwrap().clone();
            for peer_id in peers {
                if let Some(peer) = network.get(&peer_id) {
                    neuron.send_message(
                        peer,
                        A2AMessage::JoinLayer {
                            from: neuron.id.clone(),
                            layer,
                        }
                    );
                }
            }
        }
    }
    
    // Process layer messages
    thread::sleep(Duration::from_millis(100));
    println!("\n🔄 Processing layer formation messages...");
    
    for _ in 0..3 {  // Multiple rounds of propagation
        for neuron in &neurons {
            neuron.process_messages(&network);
        }
        thread::sleep(Duration::from_millis(50));
    }
    
    // Phase 3: Analyze Results
    println!("\n📊 Phase 3: Network Analysis");
    println!("───────────────────────────");
    
    let mut layer_distribution = vec![0; 10];
    
    for neuron in &neurons {
        let layer = neuron.discovered_layer.lock().unwrap()
            .unwrap_or_else(|| neuron.natural_layer());
        
        layer_distribution[layer] += 1;
        
        let peers_count = neuron.peers.lock().unwrap().len();
        println!("  {} → Layer L{} ({}P: S:{:.2} A:{:.2} E:{:.2}) [{} peers]",
                neuron.id, layer, 
                if neuron.discovered_layer.lock().unwrap().is_some() { "✓" } else { "?" },
                neuron.properties.speed,
                neuron.properties.abstraction,
                neuron.properties.energy,
                peers_count);
    }
    
    // Show emergence
    println!("\n✨ Emergent Structure:");
    println!("─────────────────────");
    
    for (layer, count) in layer_distribution.iter().enumerate() {
        if *count > 0 {
            let bar = "█".repeat(*count * 5);
            println!("  L{}: {} {}", layer, bar, count);
        }
    }
    
    // Communication stats
    println!("\n📈 Communication Statistics:");
    println!("──────────────────────────");
    
    let total_peers: usize = neurons.iter()
        .map(|n| n.peers.lock().unwrap().len())
        .sum();
    
    println!("  Total connections: {}", total_peers);
    println!("  Average connections per neuron: {:.1}", 
            total_peers as f32 / neurons.len() as f32);
    println!("  Network density: {:.1}%", 
            total_peers as f32 / (neurons.len() * (neurons.len() - 1)) as f32 * 100.0);
}

fn main() {
    demonstrate_a2a_communication();
    
    println!("\n🎯 Key A2A Principles:");
    println!("════════════════════");
    println!("1. No central coordinator - purely peer-to-peer");
    println!("2. Neurons discover layers through local interactions");
    println!("3. Compatible neurons naturally cluster together");
    println!("4. Information propagates through the network");
    println!("5. Structure emerges from simple communication rules");
    
    println!("\n✅ A2A Communication Demo Complete!");
}
EOF

# Try to compile and run
echo "Attempting to run A2A communication demo..."
echo ""

# Check if we can compile
if command -v rustc >/dev/null 2>&1; then
    cd /tmp
    if rustc a2a_demo.rs -O --edition 2021 2>/dev/null; then
        ./a2a_demo
        rm -f a2a_demo
    else
        echo "Note: For full demo, run from project directory with:"
        echo "  cargo run --example a2a_communication_demo"
        echo ""
        echo "Showing conceptual demonstration..."
        echo ""
    fi
else
    echo "Rust not found. Showing conceptual demonstration..."
    echo ""
fi

# Conceptual fallback
if [ ! -f /tmp/a2a_demo ]; then
    echo "🔗 A2A Communication Protocol Visualization"
    echo "=========================================="
    echo ""
    echo "Initial Network (10 neurons, no structure):"
    echo "  N00  N01  N02  N03  N04"
    echo "  N05  N06  N07  N08  N09"
    echo ""
    echo "Step 1: Discovery Phase 📡"
    echo "  N00 → N03: HELLO (properties: speed=0.8, abstract=0.2)"
    echo "  N03 → N00: COMPAT(0.75) [Compatible!]"
    echo "  N00 → N07: HELLO (properties: speed=0.8, abstract=0.2)"
    echo "  N07 → N00: COMPAT(0.23) [Not compatible]"
    echo "  ... (each neuron talks to ~3 random peers)"
    echo ""
    echo "Step 2: Layer Formation 🌊"
    echo "  N00 discovers it belongs in L2 (fast, concrete)"
    echo "  N00 → N03: JOIN_L2"
    echo "  N03 → peers: JOIN_L2 (propagation)"
    echo "  N08 discovers it belongs in L7 (slow, abstract)"
    echo "  N08 → N09: JOIN_L7"
    echo "  ... (layer information spreads)"
    echo ""
    echo "Step 3: Final Structure ✨"
    echo "  L1: ██ (N04, N06)"
    echo "  L2: ███ (N00, N03, N05)"
    echo "  L3: ██ (N01, N02)"
    echo "  L4: █ (N07)"
    echo "  L7: ██ (N08, N09)"
    echo ""
    echo "Communication Graph:"
    echo "  N00 ←→ N03 ←→ N05  (L2 cluster)"
    echo "   ↓      ↓"
    echo "  N01 ←→ N02         (L3 cluster)"
    echo "           ↓"
    echo "          N07         (L4 bridge)"
    echo "           ↓"
    echo "  N08 ←→ N09         (L7 cluster)"
    echo ""
    echo "📊 Network Metrics:"
    echo "  • Total connections: 14"
    echo "  • Avg connections/neuron: 2.8"
    echo "  • Network density: 31%"
    echo "  • Clustering coefficient: 0.67"
fi

# Cleanup
rm -f /tmp/a2a_demo.rs /tmp/a2a_demo

echo ""
echo "💡 A2A enables:"
echo "  • Decentralized self-organization"
echo "  • Emergent hierarchy without central control"
echo "  • Resilient network formation"
echo "  • Natural clustering by compatibility"
echo ""
echo "See also:"
echo "  • layers/L2_implementation/neurons/core/hierarchical/cognitive/a2a/"
echo "  • Run: ./demo/integrated-consciousness.sh for full system demo"