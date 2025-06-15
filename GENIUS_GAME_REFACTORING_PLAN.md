# Genius Game Server Refactoring Plan

## Overview

The `genius_game_server` will be extracted from the HAL9 monorepo and refactored into a clean, modular demo project in `../2hal9-demo`. This separation follows the gradient-core philosophy of separating concerns.

## Current Issues

1. **Monolithic Structure**: Everything bundled in one crate
2. **Tight Coupling**: Direct dependencies between games and AI providers
3. **Configuration Chaos**: Hardcoded values scattered throughout
4. **Limited Extensibility**: Hard to add new games or AI providers
5. **Testing Difficulties**: Mocked providers mixed with real implementations

## New Architecture

```
2hal9-demo/
├── README.md                           # Project overview
├── Cargo.toml                          # Workspace definition
├── crates/
│   ├── genius-core/                    # Core traits and types
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── game.rs                # Game trait
│   │   │   ├── player.rs              # Player abstractions
│   │   │   ├── state.rs               # Game state types
│   │   │   └── error.rs               # Common error types
│   │   └── Cargo.toml
│   │
│   ├── genius-engine/                  # Game execution engine
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── engine.rs              # Main game engine
│   │   │   ├── scheduler.rs           # Turn scheduling
│   │   │   ├── analytics.rs           # Analytics engine
│   │   │   └── emergence.rs           # Emergence detection
│   │   └── Cargo.toml
│   │
│   ├── genius-ai/                      # AI provider abstractions
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── provider.rs            # Provider trait
│   │   │   ├── collective.rs          # Collective intelligence
│   │   │   ├── sota.rs                # SOTA management
│   │   │   └── providers/
│   │   │       ├── mod.rs
│   │   │       ├── ollama.rs
│   │   │       ├── bedrock.rs
│   │   │       └── mock.rs
│   │   └── Cargo.toml
│   │
│   ├── genius-games/                   # Individual game implementations
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── strategic/             # Strategic games
│   │   │   │   ├── mod.rs
│   │   │   │   ├── mini_go.rs
│   │   │   │   ├── mini_holdem.rs
│   │   │   │   └── byzantine.rs
│   │   │   ├── collective/            # Collective games
│   │   │   │   ├── mod.rs
│   │   │   │   ├── swarm.rs
│   │   │   │   └── maze.rs
│   │   │   ├── survival/              # Survival games
│   │   │   │   ├── mod.rs
│   │   │   │   ├── battle_royale.rs
│   │   │   │   ├── hunger_games.rs
│   │   │   │   └── squid_game.rs
│   │   │   └── trust/                 # Trust-based games
│   │   │       ├── mod.rs
│   │   │       ├── prisoners_dilemma.rs
│   │   │       └── trust_fall.rs
│   │   └── Cargo.toml
│   │
│   ├── genius-server/                  # HTTP/WebSocket server
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── routes/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── games.rs
│   │   │   │   ├── players.rs
│   │   │   │   └── analytics.rs
│   │   │   ├── websocket.rs
│   │   │   └── state.rs
│   │   └── Cargo.toml
│   │
│   └── genius-client/                  # Client SDK
│       ├── src/
│       │   ├── lib.rs
│       │   ├── client.rs
│       │   └── types.rs
│       └── Cargo.toml
│
├── demos/                              # Demo applications
│   ├── battle-royale-demo/
│   ├── collective-intelligence/
│   ├── death-games-showcase/
│   └── ollama-swarm/
│
├── examples/                           # Simple examples
│   ├── basic_game.rs
│   ├── custom_ai_provider.rs
│   └── analytics_viewer.rs
│
├── tests/                              # Integration tests
│   ├── engine_tests.rs
│   ├── game_tests.rs
│   └── e2e_tests.rs
│
└── docs/                               # Documentation
    ├── architecture.md
    ├── game_development.md
    ├── ai_integration.md
    └── api_reference.md
```

## Refactoring Steps

### Phase 1: Core Extraction (Week 1)
1. Create workspace structure
2. Extract core traits and types to `genius-core`
3. Define clean interfaces for games and AI providers
4. Implement error handling with custom types

### Phase 2: Engine Separation (Week 1)
1. Extract game engine to `genius-engine`
2. Implement proper scheduling and state management
3. Move analytics to dedicated module
4. Add emergence detection as first-class feature

### Phase 3: AI Provider Modularization (Week 2)
1. Create `genius-ai` crate with provider traits
2. Implement providers as plugins
3. Add collective intelligence as a provider type
4. Create mock providers for testing

### Phase 4: Game Organization (Week 2)
1. Move games to categorized modules
2. Ensure each game is self-contained
3. Add game-specific configuration
4. Implement game registry pattern

### Phase 5: Server Modernization (Week 3)
1. Clean up server implementation
2. Add proper routing with API versioning
3. Implement authentication/authorization
4. Add metrics and monitoring

### Phase 6: Client SDK (Week 3)
1. Create Rust client SDK
2. Generate TypeScript types
3. Add Python bindings
4. Create example applications

### Phase 7: Testing & Documentation (Week 4)
1. Add comprehensive unit tests
2. Create integration test suite
3. Write architecture documentation
4. Add game development guide

## Benefits of Refactoring

1. **Modularity**: Each component can be developed and tested independently
2. **Extensibility**: Easy to add new games or AI providers
3. **Testability**: Clear boundaries make testing easier
4. **Performance**: Better resource management and optimization opportunities
5. **Developer Experience**: Clear structure and documentation
6. **Deployment Flexibility**: Can deploy components separately

## Migration Strategy

1. **Copy Don't Move**: Initially copy to new structure, keeping original intact
2. **Incremental Testing**: Test each component as it's extracted
3. **Backward Compatibility**: Maintain API compatibility during transition
4. **Feature Flags**: Use flags to switch between old and new implementations
5. **Gradual Rollout**: Deploy new version alongside old for testing

## Configuration Management

```toml
# genius.toml
[server]
host = "0.0.0.0"
port = 8080

[ai.ollama]
endpoint = "http://localhost:11434"
models = ["llama2", "mistral"]

[ai.bedrock]
region = "us-east-1"
models = ["claude-3", "llama-2-70b"]

[games]
enabled = ["mini_go", "prisoners_dilemma", "battle_royale"]
max_concurrent = 100

[analytics]
enabled = true
emergence_threshold = 0.7
```

## Success Metrics

1. **Build Time**: < 30s for full build
2. **Test Coverage**: > 80% for core components
3. **API Response Time**: < 100ms for game operations
4. **Memory Usage**: < 100MB per game instance
5. **Developer Onboarding**: < 1 hour to add new game

## Timeline

- **Week 1**: Core extraction and engine separation
- **Week 2**: AI providers and game organization
- **Week 3**: Server modernization and client SDK
- **Week 4**: Testing, documentation, and deployment

Total estimated time: 4 weeks for complete refactoring

## Next Steps

1. Create initial workspace structure
2. Set up CI/CD pipeline
3. Begin extracting core components
4. Create migration checklist
5. Document API changes