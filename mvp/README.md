# HAL9 MVP - Simplified Hierarchical AI Demo

## What is this?

This is a **simplified MVP** of the HAL9 hierarchical AI system that demonstrates the core concept with enhanced visual presentation. It shows how a user request flows through 3 layers of AI neurons, each handling a different level of abstraction.

## 🎯 Recent Enhancements

- **Visual Hierarchy Diagram**: Shows parent-child signal relationships
- **Enhanced Output Formatting**: Clear visual separation between layers
- **Timing Information**: Processing time for each neuron
- **Improved Code Examples**: More realistic backend/frontend implementations
- **Progress Animation**: Visual feedback during processing
- **Better UI**: Enhanced menu and status messages

## Key Simplifications

✅ **What's Included:**
- 3 neurons only (L4→L3→L2)
- Deterministic mock responses
- Clear hierarchical decomposition
- Interactive demo interface
- No external dependencies

❌ **What's Removed (for now):**
- Real Claude API calls
- Process spawning complexity
- TCP networking
- Configuration files
- Complex routing
- Error propagation

## Quick Start

```bash
# Run the demo
./run-mvp.sh

# Or manually:
cargo run --release -p hal9_mvp
```

## How It Works

```
User Input: "Create a task management web application"
    ↓
L4 Strategic: "Break this down into major components"
    ↓
L3 Design: "Design the architecture and user interface"
    ↓
L2 Implementation: "Here's the actual code..."
```

Each layer transforms the request at its level of abstraction:
- **L4** thinks strategically
- **L3** creates designs
- **L2** implements code

## Demo Scenarios

The MVP includes 3 pre-built scenarios:
1. Task management web app
2. E-commerce platform  
3. Real-time chat system

Each demonstrates clear hierarchical decomposition.

## Code Structure

```
mvp/
├── src/
│   └── main.rs     # Everything in one file for clarity
├── Cargo.toml      # Minimal dependencies
├── run-mvp.sh      # Demo runner
└── README.md       # This file
```

## Why This Approach?

The revised PRD calls for a "Skateboard First" MVP - something that works and demonstrates value quickly. This MVP:

- **Works in minutes** not weeks
- **Shows the concept** clearly
- **Has zero friction** to run
- **Proves the architecture** before adding complexity

## Next Steps

Once this MVP demonstrates value, we can add:
1. Real Claude API integration
2. Configuration system
3. More neurons (7 total)
4. Network distribution
5. Learning mechanisms

But first, we prove it works!

## Technical Notes

- Uses Tokio async tasks (not OS processes)
- Simple mpsc channels for communication
- Hardcoded responses for determinism
- ~300 lines of focused code

This is the "skateboard" - it moves and demonstrates the concept. The "car" comes later.