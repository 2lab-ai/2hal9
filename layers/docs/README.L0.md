# HAL9 - Hierarchical AGI System

## Overview

HAL9 is an experimental artificial general intelligence (AGI) system based on hierarchical abstraction and emergence principles. The system implements a 9-layer architecture where consciousness emerges from compression boundaries between layers.

## Technical Requirements

- Rust 1.70+ (primary implementation language)
- Node.js 18+ (for JavaScript SDK)
- Python 3.9+ (for Python SDK)
- Docker 20+ (for containerization)
- Kubernetes 1.25+ (for orchestration)

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/hal9.git
cd hal9

# Build the project
cargo build --workspace --release

# Run tests
cargo test --workspace

# Run the server
cargo run --bin hal9_server
```

## Project Structure

```
2hal9/
├── substrate/           # Core implementation
│   └── tooling/
│       └── rust/       # Rust codebase
├── layers/             # Hierarchical layer implementations (L1-L9)
├── competitions/       # Game theory demonstrations
├── deployment/         # Infrastructure configuration
└── docs/              # Documentation
```

## Key Components

- **hal9-core**: Neural substrate and self-organization engine
- **hal9-server**: WebSocket and REST API server
- **genius_game_server**: Game theory platform for emergence testing
- **gradient-core**: Mathematical foundations (being separated)

## API Documentation

- WebSocket API: Connect to `ws://localhost:8080/ws`
- REST API: Available at `http://localhost:8080/api/v1`

See [API_DESIGN.md](./API_DESIGN.md) for detailed documentation.

## Contributing

Please read our contributing guidelines before submitting pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

---

**Next Level**: Ready to understand the deeper architecture? Continue to [README.L1.md](./README.L1.md) →