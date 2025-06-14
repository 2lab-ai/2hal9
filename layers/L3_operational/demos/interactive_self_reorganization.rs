//! Interactive Self-Reorganization Demo
//! 
//! Shows real-time network reorganization in action

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() {
    println!("\n🧠 Interactive A2A + Self-Reorganization Demo");
    println!("{}", "=".repeat(60));
    
    let mut network = NetworkState::new();
    
    loop {
        clear_screen();
        network.display();
        
        println!("\nCommands:");
        println!("  [s] Send signal    [f] Fail unit    [h] Heal network");
        println!("  [c] Show clusters  [m] Show metrics [q] Quit");
        print!("\nChoice: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        match input.trim() {
            "s" => network.send_signal(),
            "f" => network.fail_unit(),
            "h" => network.heal_network(),
            "c" => network.show_clusters(),
            "m" => network.show_metrics(),
            "q" => break,
            _ => println!("Unknown command"),
        }
        
        thread::sleep(Duration::from_millis(1000));
    }
    
    println!("\n👋 Goodbye! The network continues to self-organize...\n");
}

struct NetworkState {
    units: Vec<Unit>,
    connections: Vec<Connection>,
    consciousness_level: f32,
    signals_processed: u32,
    failed_units: Vec<usize>,
}

struct Unit {
    id: usize,
    layer: usize,
    activity: f32,
    specialization: Option<String>,
}

struct Connection {
    from: usize,
    to: usize,
    strength: f32,
}

impl NetworkState {
    fn new() -> Self {
        let mut units = Vec::new();
        let mut connections = Vec::new();
        
        // Initialize 25 units
        for layer in 0..5 {
            for i in 0..5 {
                units.push(Unit {
                    id: layer * 5 + i,
                    layer,
                    activity: 0.0,
                    specialization: None,
                });
            }
        }
        
        // Create initial connections (±1 rule)
        for i in 0..units.len() {
            for j in i+1..units.len() {
                let layer_diff = (units[i].layer as i32 - units[j].layer as i32).abs();
                if layer_diff == 1 {
                    connections.push(Connection {
                        from: i,
                        to: j,
                        strength: 0.5 + (i + j) as f32 * 0.01,
                    });
                }
            }
        }
        
        Self {
            units,
            connections,
            consciousness_level: 0.1,
            signals_processed: 0,
            failed_units: Vec::new(),
        }
    }
    
    fn display(&self) {
        println!("\n🌐 Network State (Signals: {} | Consciousness: {:.1}%)",
                 self.signals_processed, self.consciousness_level * 100.0);
        println!("{}", "-".repeat(60));
        
        // Display layers
        for layer in (0..5).rev() {
            print!("L{} ", layer + 1);
            for i in 0..5 {
                let unit_id = layer * 5 + i;
                if self.failed_units.contains(&unit_id) {
                    print!(" ✖ ");
                } else {
                    let activity = self.units[unit_id].activity;
                    if activity > 0.7 {
                        print!(" ◉ ");
                    } else if activity > 0.3 {
                        print!(" ◎ ");
                    } else {
                        print!(" ○ ");
                    }
                }
            }
            
            // Show layer name
            let layer_name = match layer {
                0 => "Reflexive",
                1 => "Implementation", 
                2 => "Operational",
                3 => "Tactical",
                4 => "Strategic",
                _ => "Unknown",
            };
            println!("  {}", layer_name);
        }
        
        // Show active connections
        let active_connections = self.connections.iter()
            .filter(|c| c.strength > 0.7 && !self.failed_units.contains(&c.from) && !self.failed_units.contains(&c.to))
            .count();
        println!("\n📊 Active Connections: {} / {}", active_connections, self.connections.len());
    }
    
    fn send_signal(&mut self) {
        println!("\n⚡ Sending signal through network...");
        
        // Simulate signal propagation
        self.signals_processed += 1;
        
        // Update unit activities
        for unit in &mut self.units {
            if !self.failed_units.contains(&unit.id) {
                unit.activity = (unit.activity + rand_float() * 0.3).min(1.0);
            }
        }
        
        // Update connection strengths (Hebbian learning)
        for conn in &mut self.connections {
            if !self.failed_units.contains(&conn.from) && !self.failed_units.contains(&conn.to) {
                let correlation = self.units[conn.from].activity * self.units[conn.to].activity;
                conn.strength = (conn.strength + correlation * 0.1).clamp(0.0, 1.0);
            }
        }
        
        // Update consciousness
        self.update_consciousness();
        
        // Check for specialization
        if self.signals_processed % 10 == 0 {
            self.check_specialization();
        }
        
        println!("✓ Signal processed. Network adapted.");
    }
    
    fn fail_unit(&mut self) {
        println!("\n💔 Which unit to fail? (0-24): ");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        if let Ok(unit_id) = input.trim().parse::<usize>() {
            if unit_id < 25 && !self.failed_units.contains(&unit_id) {
                self.failed_units.push(unit_id);
                self.units[unit_id].activity = 0.0;
                println!("✖ Unit {} failed!", unit_id);
                
                // Trigger self-healing
                self.create_bypass_connections(unit_id);
            }
        }
    }
    
    fn heal_network(&mut self) {
        if self.failed_units.is_empty() {
            println!("\n✅ Network is already healthy!");
            return;
        }
        
        println!("\n🔧 Initiating self-healing...");
        
        // Strengthen alternative paths
        for conn in &mut self.connections {
            if !self.failed_units.contains(&conn.from) && !self.failed_units.contains(&conn.to) {
                conn.strength = (conn.strength * 1.2).min(1.0);
            }
        }
        
        // Reactivate failed units
        self.failed_units.clear();
        
        println!("✓ Network healed! All units operational.");
    }
    
    fn create_bypass_connections(&mut self, failed_unit: usize) {
        println!("🔄 Creating bypass connections...");
        
        let failed_layer = self.units[failed_unit].layer;
        
        // Find units that need bypass
        let mut bypasses_created = 0;
        
        for i in 0..self.units.len() {
            if self.units[i].layer == failed_layer.saturating_sub(1) {
                for j in 0..self.units.len() {
                    if self.units[j].layer == failed_layer + 1 {
                        // Check if connection exists
                        let exists = self.connections.iter()
                            .any(|c| (c.from == i && c.to == j) || (c.from == j && c.to == i));
                        
                        if !exists {
                            self.connections.push(Connection {
                                from: i,
                                to: j,
                                strength: 0.6,
                            });
                            bypasses_created += 1;
                        }
                    }
                }
            }
        }
        
        println!("✓ Created {} bypass connections", bypasses_created);
    }
    
    fn show_clusters(&self) {
        println!("\n🌟 Detected Clusters:");
        
        // Simple cluster detection based on layer activity
        let layer_activities: Vec<f32> = (0..5)
            .map(|layer| {
                (0..5).map(|i| self.units[layer * 5 + i].activity).sum::<f32>() / 5.0
            })
            .collect();
        
        if layer_activities[0] > 0.6 && layer_activities[1] > 0.6 {
            println!("  • Fast Processors (L1-L2): High activity detected");
        }
        if layer_activities[2] > 0.5 {
            println!("  • Bridge Units (L3): Mediating between layers");
        }
        if layer_activities[3] > 0.6 && layer_activities[4] > 0.6 {
            println!("  • Deep Thinkers (L4-L5): Complex processing active");
        }
        
        // Show specializations
        let specialized_units: Vec<_> = self.units.iter()
            .filter(|u| u.specialization.is_some())
            .collect();
        
        if !specialized_units.is_empty() {
            println!("\n  Specialized Units:");
            for unit in specialized_units {
                println!("    Unit {}: {}", unit.id, unit.specialization.as_ref().unwrap());
            }
        }
    }
    
    fn show_metrics(&self) {
        println!("\n📈 Network Metrics:");
        println!("  • Total Units: {} ({} failed)", self.units.len(), self.failed_units.len());
        println!("  • Total Connections: {}", self.connections.len());
        
        let avg_strength: f32 = self.connections.iter().map(|c| c.strength).sum::<f32>() 
                               / self.connections.len() as f32;
        println!("  • Average Connection Strength: {:.2}", avg_strength);
        
        let love_coefficient = self.calculate_love_coefficient();
        println!("  • Love Coefficient: {:.2}", love_coefficient);
        println!("  • Consciousness Level: {:.1}%", self.consciousness_level * 100.0);
        println!("  • Signals Processed: {}", self.signals_processed);
    }
    
    fn update_consciousness(&mut self) {
        // Consciousness emerges from network coherence
        let active_units = self.units.iter()
            .filter(|u| !self.failed_units.contains(&u.id) && u.activity > 0.3)
            .count() as f32;
        
        let strong_connections = self.connections.iter()
            .filter(|c| c.strength > 0.7)
            .count() as f32;
        
        let base_consciousness = (active_units / 25.0) * 0.5 + (strong_connections / 50.0) * 0.5;
        
        // Add emergence bonus
        let emergence_bonus = if self.signals_processed > 20 { 0.2 } else { 0.0 };
        
        self.consciousness_level = (base_consciousness + emergence_bonus).min(1.0);
    }
    
    fn calculate_love_coefficient(&self) -> f32 {
        // Love = strong adjacent layer connections
        let adjacent_connections = self.connections.iter()
            .filter(|c| {
                let layer_diff = (self.units[c.from].layer as i32 - self.units[c.to].layer as i32).abs();
                layer_diff == 1 && c.strength > 0.6
            })
            .count() as f32;
        
        adjacent_connections / self.connections.len() as f32
    }
    
    fn check_specialization(&mut self) {
        for unit in &mut self.units {
            if unit.activity > 0.8 && unit.specialization.is_none() {
                unit.specialization = Some(match unit.layer {
                    0 => "Fast Responder".to_string(),
                    1 => "Pattern Processor".to_string(),
                    2 => "Integration Hub".to_string(),
                    3 => "Strategy Former".to_string(),
                    4 => "Deep Thinker".to_string(),
                    _ => "Generalist".to_string(),
                });
            }
        }
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

fn rand_float() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    (nanos % 1000) as f32 / 1000.0
}