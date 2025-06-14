# 🎮 AI Genius Game Server - Demo Collection

## 📁 Demo Files Overview

### 🎯 Original Classic Demo
- **`ai_genius_demo.html`** - Original Minority Game with emergence detection
- **`ai_genius_demo.gif`** - Animated showcase of collective intelligence emergence
- **Terminal Mode** - Run with `cargo run --bin demo`

### 🎨 AAA Professional Game Suite (NEW!)
- **`aaa_demos.html`** - Professional game launcher with cyberpunk aesthetics
- **`aaa_game_launcher.html`** - Main menu with neural network visualization
- **`aaa_byzantine_generals.html`** - Medieval warfare consensus game
- **`aaa_collective_maze.html`** - Cyberpunk grid maze with swarm AI
- **`aaa_swarm_optimization.html`** - Scientific visualization of particle swarms
- **`aaa_recursive_reasoning.html`** - Fractal mind-bending reasoning game
- **`GAME_SHOWCASE.md`** - Detailed AAA game descriptions

### 💀 Death Game Championship (NEW!)
- **`DEATH_GAME_SHOWCASE.md`** - Complete death game documentation
- **Terminal Mode** - Run with `cargo run --bin death_game_demo`
- **10+ Survival Games** - Battle Royale, Liar's Dice, Russian Roulette, and more

## 🚀 Quick Start Guide

### 1. View Static Demos (No Server Required)
```bash
# Original emergence demo
open ai_genius_demo.html

# Professional AAA game launcher
open aaa_demos.html

# Read death game descriptions
open DEATH_GAME_SHOWCASE.md

# Read AAA game showcase
open GAME_SHOWCASE.md
```

### 2. Run Interactive Demos
```bash
# Classic terminal demo
cargo run --bin demo

# Death game terminal interface
cargo run --bin death_game_demo

# Championship mode with all games
cargo run --bin death_game_demo -- --championship

# Specific death game
cargo run --bin death_game_demo -- --game battle_royale
```

### 3. Start Full Game Server
```bash
# Run the complete server
cargo run --release

# Then open any demo HTML in your browser
# The HTML files will connect to localhost:8080
```

## 📊 Demo Comparison

| Demo Type | Visual Style | Interactivity | Best For | Special Features |
|-----------|-------------|---------------|----------|-----------------|
| **Original Demo** | Clean/Minimal | Watch Only | Understanding Emergence | Simple, Clear |
| **AAA Games** | Cyberpunk/Pro | Full Interactive | Showcase/Presentation | 60 FPS, Particles |
| **Death Games** | Terminal/ASCII | Command Line | Hardcore Experience | Survival Mechanics |

## 🎮 Complete Game List

### Classic Game Theory (5 Games)
1. **Minority Game** ✅ - Choose the less popular option to win
2. **Byzantine Generals** ✅ - Achieve consensus despite traitors
3. **Collective Maze** ✅ - Navigate through shared swarm intelligence
4. **Recursive Reasoning** ✅ - "I think that you think that I think..."
5. **Swarm Optimization** ✅ - Find optimal solutions collectively

### Death Game Modes (10 Games)
1. **Battle Royale** 🎮 - Shrinking safe zones force confrontation
2. **Hunger Games** 🏹 - Resource scarcity and sponsor system
3. **Liar's Dice** 🎲 - Bluff or die in high-stakes deception
4. **Russian Roulette** 💀 - Pure probability meets psychology
5. **King of the Hill** 👑 - Dominate territory or be overthrown
6. **Last Stand** 🛡️ - Survive increasingly difficult waves
7. **Trust Fall** 🤝 - Prisoner's dilemma with deadly consequences
8. **Mini Go** 🏯 - 9x9 territorial warfare (Implemented)
9. **Mini Hold'em** 🃏 - All-in poker elimination (Implemented)
10. **Squid Game** 🦑 - Red light/green light survival

## 🎨 Visual Themes & Features

### AAA Professional Suite
- **Cyberpunk Aesthetics**: Neon glows, grid patterns, holograms
- **Glass Morphism UI**: Semi-transparent panels with blur effects
- **Particle Systems**: Neural network connections, swarm particles
- **Smooth Animations**: 60 FPS performance, CSS transitions
- **Responsive Design**: Adapts to all screen sizes
- **Custom Fonts**: Orbitron for headers, Rajdhani for body

### Death Game Terminal
- **Matrix Style**: Green phosphor text on black
- **ASCII Art**: Game state visualization in text
- **CRT Effects**: Scanlines and screen flicker
- **Real-time Logs**: Combat events stream like system logs
- **Retro Terminal**: Monospace font, command-line interface
- **Dramatic Events**: Death notifications, emergence alerts

## 📈 What to Look For

### Emergence Patterns
- **Round 15-25**: Collective coordination begins
- **Perfect Distribution**: Minority game balance achieved
- **Byzantine Consensus**: Agreement despite traitors
- **Swarm Convergence**: Particles find global optimum
- **Death Game Alliances**: Temporary cooperation under pressure

### Key Metrics
| Metric | Expected Value | Significance |
|--------|---------------|--------------|
| Collective Win Rate | 52-58% | Shows emergence advantage |
| Emergence Frequency | 28-35% | How often coordination occurs |
| Coordination Score | 0.8-0.95 | Quality of collective behavior |
| Survival Time | 4-7 rounds | Death game brutality |

## 🛠️ Customization Options

### Modify Game Parameters
Edit HTML/JS files to adjust:
```javascript
// In any AAA game HTML
const gameConfig = {
    numAgents: 32,        // Number of AI players
    rounds: 100,          // Game length
    updateInterval: 100,  // Animation speed (ms)
    particleCount: 1000,  // Visual effects intensity
    emergenceThreshold: 20 // When to detect emergence
};
```

### Terminal Demo Options
```bash
# Adjust difficulty
cargo run --bin death_game_demo -- --difficulty nightmare

# Change game speed
cargo run --bin death_game_demo -- --speed fast

# Enable debug mode
cargo run --bin death_game_demo -- --debug
```

## 🐛 Troubleshooting

### Common Issues & Solutions

| Problem | Solution |
|---------|----------|
| "Connection refused" | Start game server with `cargo run` first |
| Blank screen | Check browser console (F12) for errors |
| Slow performance | Reduce particle count or agent count |
| No emergence detected | Run more rounds (usually needs 20+) |
| Terminal colors wrong | Use a modern terminal (iTerm2, Windows Terminal) |

### Browser Compatibility
- **Optimal**: Chrome 90+, Edge 90+
- **Good**: Firefox 88+, Safari 14+
- **Mobile**: Supported but desktop recommended for best experience

## 📚 Understanding the Games

### Emergence Detection
The system detects emergence when:
1. **Collective agents** coordinate beyond random chance
2. **Performance** exceeds theoretical individual limits
3. **Patterns** show distributed decision-making
4. **Stability** in coordination maintains over time

### Death Game Psychology
Death games reveal:
- **Cooperation vs Competition** dynamics under extreme pressure
- **Trust formation** and betrayal timing
- **Risk assessment** when stakes are ultimate
- **Emergent alliances** that form and shatter

## 🎯 Educational Value

These demos showcase:
1. **Game Theory** - Classic problems in interactive form
2. **Collective Intelligence** - How coordination emerges
3. **AI Competition** - Different strategies and their effectiveness
4. **Emergence** - Complex behavior from simple rules
5. **Survival Strategies** - Decision-making under extreme pressure

## 🔗 Related Resources

- **Main README**: Complete project documentation
- **GAME_SHOWCASE.md**: Detailed AAA game descriptions
- **DEATH_GAME_SHOWCASE.md**: Full death game documentation
- **API Docs**: `cargo doc --open`

---

**Experience the future of AI competition!**

*Made with ❤️ by the 2Lab AI Team*