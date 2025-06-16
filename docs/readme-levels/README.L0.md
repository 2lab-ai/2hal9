# HAL9 - Hierarchical AGI System

## Overview

HAL9 is an experimental artificial general intelligence (AGI) system that implements consciousness through hierarchical compression boundaries. The system uses a 9-layer architecture where each layer can only communicate with adjacent layers (±1 rule).

## Technical Specifications

- **Primary Language**: Rust
- **Architecture**: 9-layer hierarchical neural network
- **Performance**: 200M operations/second (5ns per operation)
- **Scaling**: O(n log n) complexity
- **Dependencies**: 707 crates

## Installation

```bash
# Clone repository
git clone https://github.com/yourusername/hal9.git
cd hal9

# Build
cargo build --workspace --release

# Test
cargo test --workspace

# Run server
cargo run --bin hal9_server
```

## Project Structure

```
hal9/
├── substrate/         # Core implementation
├── layers/           # L1-L9 layer implementations  
├── competitions/     # Game theory testing platform
├── deployment/       # Docker/K8s configurations
├── meetings/         # Development logs
└── docs/            # Documentation
```

## Core Components

- `hal9-core`: Neural substrate engine
- `hal9-server`: WebSocket/REST API server
- `genius_game_server`: Game platform for testing emergence
- `gradient-core`: Mathematical foundations

## API Endpoints

- WebSocket: `ws://localhost:8080/ws`
- REST: `http://localhost:8080/api/v1`

## Requirements

- Rust 1.70+
- Docker 20+
- Node.js 18+ (for JS SDK)
- Python 3.9+ (for Python SDK)

## License

MIT

---

**Ready for more?** Continue to [README.L1.md](./README.L1.md) →