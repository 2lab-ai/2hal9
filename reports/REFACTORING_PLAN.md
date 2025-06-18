# HAL9 Project Refactoring Plan

## Current Issues
1. **Duplication**: Same functionality exists in both `2hal9` and `2hal9-demo`
2. **Confusion**: Unclear which project to use for what purpose
3. **Scattered Code**: Game implementations spread across multiple locations

## Proposed Structure

```
2hal9/                              # Main HAL9 Project
├── README.md                       # Project overview
├── CLAUDE.md                       # AI assistant instructions
├── demos/                          # Performance & neuron demos only
│   ├── hal9_demo.sh               # Basic HAL9 performance tests
│   └── self_organization.sh       # Neuron self-organization demos
│
├── games/                         # All game-related code (NEW)
│   ├── server/                    # Game server implementation
│   │   ├── src/
│   │   │   ├── main.rs           # Server entry point
│   │   │   ├── games/            # Game implementations
│   │   │   │   ├── mini_go.rs
│   │   │   │   ├── holdem.rs
│   │   │   │   └── mod.rs
│   │   │   └── ai/               # AI providers
│   │   │       ├── ollama.rs
│   │   │       └── bedrock.rs
│   │   └── Cargo.toml
│   │
│   ├── visualizations/           # Game visualizations (NEW)
│   │   ├── mini_go_viz.html
│   │   ├── holdem_viz.html
│   │   └── assets/
│   │       ├── go_board.css
│   │       └── poker_table.css
│   │
│   └── examples/                 # Game examples
│       ├── mini_go_demo.rs
│       └── holdem_demo.rs
│
└── layers/                       # HAL9 core architecture
    └── (existing neuron code)
```

## Migration Steps

1. **Phase 1: Consolidate Games**
   - Move all game code from `competitions/genius_game_server` to `games/server`
   - Merge best implementations from `2hal9-demo`

2. **Phase 2: Create Visualizations**
   - Build professional HTML/CSS visualizations
   - Connect to WebSocket for real-time updates

3. **Phase 3: Clean Up**
   - Archive or remove `2hal9-demo` 
   - Update all import paths
   - Update documentation

## Immediate Actions
1. Create visualization for Mini Go and Holdem
2. Place them in appropriate location
3. Keep `./demos` for HAL9 neuron tests only