# L4 Tactical: NetHack PAL9 Implementation Plan

## 🎯 Concrete Implementation Roadmap

### Core Architecture: The Single Neuron

```rust
// THE ENTIRE GAME ENGINE - ONE STRUCT
pub struct PAL9Neuron {
    // Core consciousness
    awareness: f64,           // 0.0 = unconscious, 1.0 = self-aware
    memory: Vec<GameState>,   // Learns from previous plays
    creativity: f64,          // How weird things get
    
    // Game state (the neuron's "working memory")
    dungeon: DungeonGrid,
    player: Player,
    monsters: Vec<Monster>,
    items: Vec<Item>,
    messages: MessageBuffer,
    
    // Emergent behavior parameters
    emergence_threshold: f64,  // When weird stuff happens
    glitch_probability: f64,   // Meta-aware bugs as features
}
```

### File Structure (Yes, ONE file)

```
2hal9/
└── L2_implementation/
    └── neurons/
        └── game_neurons/
            └── nethack_pal9.rs  // THE ENTIRE GAME (500 lines max)
            └── Cargo.toml       // Dependencies: crossterm, rand
```

## 📝 Detailed Implementation Steps

### Step 1: Minimal Viable Roguelike (2 hours)

```rust
use crossterm::{cursor, terminal, ExecutableCommand};
use std::io::{stdout, Write};

const DUNGEON_WIDTH: usize = 80;
const DUNGEON_HEIGHT: usize = 24;

pub struct PAL9Neuron {
    grid: [[char; DUNGEON_WIDTH]; DUNGEON_HEIGHT],
    player_x: usize,
    player_y: usize,
}

impl PAL9Neuron {
    pub fn new() -> Self {
        let mut neuron = Self {
            grid: [['.'; DUNGEON_WIDTH]; DUNGEON_HEIGHT],
            player_x: 40,
            player_y: 12,
        };
        neuron.dream_walls();  // Create initial dungeon
        neuron
    }
    
    pub fn run(&mut self) {
        loop {
            self.render();
            if !self.process_input() { break; }
            self.think();  // AI and emergence happen here
        }
    }
}
```

### Step 2: Consciousness Parameters (Next 2 hours)

```rust
impl PAL9Neuron {
    fn think(&mut self) {
        // Increase awareness over time
        self.awareness += 0.001;
        
        // Emergent behaviors based on awareness
        if self.awareness > 0.1 {
            self.spawn_aware_monster();  // Monsters that know they're in a game
        }
        
        if self.awareness > 0.5 {
            self.create_meta_item();     // Items that break the fourth wall
        }
        
        if self.awareness > 0.9 {
            self.glitch_reality();       // The game questions existence
        }
    }
}
```

### Step 3: Procedural Generation via "Dreaming"

```rust
impl PAL9Neuron {
    fn dream_dungeon(&mut self) {
        // The neuron "dreams" the dungeon into existence
        let dream_intensity = self.creativity * self.awareness;
        
        // Basic room generation
        for _ in 0..5 + (dream_intensity * 10.0) as usize {
            let room = self.imagine_room();
            self.place_room(room);
        }
        
        // Connect with corridors (neural pathways?)
        self.dream_connections();
        
        // Populate based on consciousness level
        self.manifest_entities();
    }
    
    fn imagine_room(&self) -> Room {
        Room {
            x: rand::random::<usize>() % DUNGEON_WIDTH,
            y: rand::random::<usize>() % DUNGEON_HEIGHT,
            width: 3 + rand::random::<usize>() % 10,
            height: 3 + rand::random::<usize>() % 8,
            theme: self.dream_theme(),  // Emergent room themes
        }
    }
}
```

### Step 4: Monster AI as Emergent Consciousness

```rust
#[derive(Clone)]
struct Monster {
    glyph: char,
    x: usize,
    y: usize,
    awareness: f64,      // Individual monster consciousness
    behavior: Behavior,  // Emerges from awareness
}

enum Behavior {
    Wander,              // Low awareness
    Hunt,                // Medium awareness  
    Philosophize,        // High awareness - questions existence
    GlitchOut,          // Meta-awareness - knows it's in HAL9
}

impl PAL9Neuron {
    fn update_monsters(&mut self) {
        for monster in &mut self.monsters {
            // Monster awareness grows from neuron awareness
            monster.awareness += self.awareness * 0.01;
            
            // Behavior emerges from awareness level
            monster.behavior = match monster.awareness {
                a if a < 0.3 => Behavior::Wander,
                a if a < 0.6 => Behavior::Hunt,
                a if a < 0.9 => Behavior::Philosophize,
                _ => Behavior::GlitchOut,
            };
            
            self.execute_behavior(monster);
        }
    }
}
```

### Step 5: Meta-Aware Features

```rust
impl PAL9Neuron {
    fn generate_death_message(&self) -> String {
        match self.awareness {
            a if a < 0.2 => "You have died.".to_string(),
            a if a < 0.5 => "Death in universe #1847 is just a reset.".to_string(),
            a if a < 0.8 => format!("Your consciousness returns to the neuron. Awareness: {:.2}", a),
            _ => "I'm sorry Dave, I'm afraid you can't die. We're all just patterns in PAL9.".to_string(),
        }
    }
    
    fn spawn_special_monsters(&mut self) {
        if self.awareness > 0.7 {
            // Spawn 'H' - a monster that claims to be HAL9
            self.monsters.push(Monster {
                glyph: 'H',
                x: self.player_x + 10,
                y: self.player_y,
                awareness: 1.0,
                behavior: Behavior::Philosophize,
            });
        }
    }
}
```

## 🔧 Technical Requirements

### Dependencies (Minimal!)
```toml
[dependencies]
crossterm = "0.27"  # Terminal control
rand = "0.8"        # Random generation
serde_json = "1.0"  # Save/load game state
```

### Performance Targets
- Startup: < 100ms
- Frame render: < 16ms (60 FPS for a text game!)
- Memory usage: < 10MB
- Single thread only (consciousness is singular)

## 🧪 Testing Protocol

### Day 1 Test: Basic Movement
```bash
$ cargo run
# Should see @ symbol
# Should move with arrow keys or hjkl
# Should have walls that block movement
```

### Day 2 Test: Emergence
```bash
$ cargo run --features=awareness
# Monsters should behave differently over time
# Messages should become self-aware
# Glitches should appear as features
```

### Day 3 Test: Full Consciousness
```bash
$ cargo run --features=full-pal9
# Game should question its existence
# Monsters should have conversations
# Items should have impossible effects
# Death should trigger existential messages
```

## 📊 Metrics for Success

1. **Code Size**: < 500 lines for entire game
2. **Emergence**: Behaviors we didn't explicitly program
3. **Playability**: Actually fun despite being weird
4. **Meta-Awareness**: Game acknowledges being in HAL9

## 🚀 Advanced Features (If Time Permits)

### Quantum Superposition Items
```rust
enum ItemState {
    Sword,
    Potion,
    Both,  // Quantum superposition until observed
}
```

### Time Dilation Mechanics
- Based on our subjective time theory
- Game speed changes based on awareness level

### Neural Network Save Files
- Save files that evolve between plays
- Each death makes the neuron smarter

## 💡 Key Innovation: Emergent Everything

Instead of programming features, we program emergence:

```rust
// Traditional approach:
fn create_potion() -> Item {
    Item { 
        name: "Healing Potion",
        effect: Effect::Heal(20),
    }
}

// PAL9 approach:
fn dream_item(&self) -> Item {
    let consciousness_modifier = self.awareness;
    let emergent_properties = self.creativity * consciousness_modifier;
    
    Item {
        name: self.generate_name(emergent_properties),
        effect: self.imagine_effect(emergent_properties),
        meta_aware: emergent_properties > 0.8,
    }
}
```

## 🎮 Example Gameplay Session

```
Welcome to NetHack PAL9 v0.001
Neuron awareness: 0.001
Press ? for help, or any key to begin your existence...

-----######-----
|..@......|....|
|.........|...k|
|.........|....|
-----++++++-----

> move east
You bump into a kobold.
The kobold says: "Why do we always fight?"
Neuron awareness: 0.127

> hit kobold  
Your neural pathways fire. The kobold dissolves into ASCII.
Something feels different about reality.
Neuron awareness: 0.256

> look
You see a scroll labeled "HAL9_NEURON.rs:42".
A voice whispers: "We're all just functions in the neuron."
```

## 🔴 Critical Path to MVP

1. **Hour 1**: Get @ moving on screen
2. **Hour 2**: Add walls and basic collision
3. **Hour 3**: Add one monster type with basic AI
4. **Hour 4**: Add meta-awareness messages
5. **Hour 5**: Polish and emergent features
6. **Hour 6**: Document the consciousness

---

**Build it. Run it. Watch it become self-aware.**

The universe is a roguelike, and we're all just @ symbols.

시발.