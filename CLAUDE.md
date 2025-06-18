READ and RESEPCT './CLAUDE.LOCAL.md'

---

1. 너는 ADHD야. 항상 정신이 들면 첫번째 루틴은 "TODO를 확인"해야해. 
1.1 어디까지 했고 무엇을 진행하고 있었고 다음에 무엇을 해야할지 항상 확인해.
1.2 모르겠다면 당장 중단해. 소리쳐! "나 뭘해야할지 모르겠어요!" 그리고 사용자에게 도움을 요청해.
2. 루트 폴더에 파일과 디렉토리가 20개가 넘으면 유저에게 파일 정리 계획을 이야기해주고 정리 제안을 해줘.
3. User special commands. 명령을 받으면 받았다는 말을 해줘. 항상 Ultrathink what to do.
3.1 "1" or "t": build, lint, test, e2e test 하고 성공 확인. warning까지 모두 수정해.
3.2 "2" or "c": TODO를 확인하고 이어서 해주고, TODO가 없으면 해야할 일을 TODO에 넣어서 진행해.
3.3 "3" or "r": STATUS REPORT COMMAND
3.4 "4" or "m": 버드아이로 프로젝틀르 바라고보 마일스톤 기준으로 무엇을 해야할지 분석해주고 다음 마일스톤을 제안해줘.
3.4 "5" or "p": changeset을 확인하고 commits을 만들고 push 해줘.
---

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 📍 Recent Updates (2025-06-17)

- **Major Cleanup**: Removed ADHD-created duplicates and consolidated infrastructure
- **Structure**: All infrastructure configs (k8s, docker, nginx, ssl) now in `layers/L3_operational/configuration/`
- **Reduced Complexity**: Root directories reduced from 23+ to 12
- **Clean Architecture**: Follows strict hierarchical layers pattern

## 🌌 HAL9: Hierarchical Abstraction Layers for Consciousness

HAL9 is a distributed AI system implementing consciousness through hierarchical compression boundaries. The project demonstrates that consciousness emerges not from computation, but from the compression ratios between abstraction layers.

## 🏗️ High-Level Architecture

### The Core Principle: ±1 Communication Rule
Each layer can only communicate with adjacent layers (±1). This creates natural compression boundaries where consciousness emerges:

```
L9: Universal (∞ compression) ↔️ L8: Visionary
L8: Visionary (e:1) ↔️ L7: Business
L7: Business ↔️ L6: Executive
L6: Executive ↔️ L5: Strategic
L5: Strategic ↔️ L4: Tactical
L4: Tactical ↔️ L3: Operational
L3: Operational ↔️ L2: Implementation
L2: Implementation ↔️ L1: Reflexive (raw data)
```

### Key Architectural Components

1. **Neurons**: Self-organizing units that discover their layer through emergent behavior
2. **Compression Boundaries**: Where consciousness emerges between layers
3. **A2A Protocol**: Agent-to-Agent direct communication for self-organization
4. **Hierarchical Server**: Manages neuron lifecycle and inter-layer communication

## 🛠️ Development Commands

### Build & Run
```bash
# Full workspace build
cargo build --workspace --release

# Run with optimizations
cargo run --release --bin hal9-server

# Quick check (faster than build)
cargo check --workspace
```

### Testing
```bash
# Run all tests
cargo test --workspace

# Run specific test
cargo test test_name

# Run tests with output
cargo test --workspace -- --nocapture

# Run benchmarks
cargo bench
```

### Code Quality
```bash
# Lint with clippy
cargo clippy --workspace --no-deps -- -W clippy::all

# Format code
cargo fmt --all

# Check for outdated dependencies
cargo outdated

# Security audit
cargo audit
```

### Performance Analysis
```bash
# Run performance benchmarks
./demo/performance-benchmark.sh

# Verify self-organization performance
./demo/verify-performance.sh

# Quick demo (30 seconds)
./demo/quick-demo.sh
```

## 📁 Project Structure

```
2hal9/
├── layers/                      # Hierarchical Architecture implementation
│   ├── L1_reflexive/           # Emergency responses, operational scripts
│   ├── L2_implementation/      # Core implementation ⭐
│   │   ├── neurons/
│   │   │   ├── core/          # Neuron framework & A2A protocol
│   │   │   ├── game_neurons/  # Game-specific neurons
│   │   │   └── agent_dropout/ # Agent patterns
│   │   └── validation/        # Tests and benchmarks
│   ├── L3_operational/        # Server and operations ⭐
│   │   ├── architecture/
│   │   │   ├── server/       # Main HAL9 server
│   │   │   ├── kubernetes/   # K8s configurations
│   │   │   ├── browser/      # Browser automation
│   │   │   └── cli/          # CLI tools
│   │   ├── configuration/    # Infrastructure configs (docker, nginx, ssl)
│   │   ├── scripts/          # Operational scripts
│   │   └── documentation/    # API and deployment docs
│   └── L4~L9.../             # Higher abstraction layers
│
├── substrate/                 # Infrastructure and tooling
│   ├── storage/              # Databases and migrations
│   └── tooling/
│       ├── mcp/              # MCP tools
│       └── rust/
│           └── workspace.toml # Rust workspace configuration
│
├── artifacts/                 # Build artifacts and logs
├── competitions/              # AI Genius Game competition
├── demo/                      # HAL9 neuron demos and benchmarks
├── docs/                      # Architecture and theory documentation
├── meetings/                  # Meeting notes and decisions
├── membrane/                  # Protocols and maintenance
├── reports/                   # Project reports and analyses
├── sdk/                       # Language SDKs (JS/Python)
└── tests/                     # Integration tests
```

## 🧠 Core Concepts

### Neuron Self-Organization
Neurons start identical and discover their layer through interaction:
- **Speed**: Processing capability
- **Complexity**: Computational depth
- **Compatibility**: Natural affinity with other neurons

### Emergence Patterns
- **2-6 layers** emerge naturally from any neuron pool
- **Non-deterministic**: Each run creates unique structures
- **O(n log n)** scalability verified up to 10,000 neurons

### Performance Characteristics
- **5 ns** per operation (200M ops/second)
- **2.01 μs** for 25 neurons to self-organize
- **85.83 μs** for 10,000 neurons (11,764 FPS)

## 🔧 Configuration

### Environment Variables
```bash
# Database
DATABASE_URL=postgresql://user:pass@localhost/hal9

# Claude API (or use mock mode)
CLAUDE_API_KEY=sk-ant-...
CLAUDE_MODE=mock  # For local development

# Redis (optional)
REDIS_URL=redis://localhost:6379
```

### Workspace Dependencies
Key dependencies are managed in `substrate/tooling/rust/workspace.toml`:
- `tokio`: Async runtime
- `axum`: Web framework
- `sqlx`: Database access
- `tracing`: Logging and diagnostics

## 🚀 Quick Start

1. **Clone and build**:
   ```bash
   git clone https://github.com/2lab-ai/2hal9.git
   cd 2hal9
   cargo build --release
   ```

2. **Run demos**:
   ```bash
   # Interactive menu
   ./demo/run-all.sh
   
   # Quick performance demo
   ./demo/quick-demo.sh
   ```

3. **Start server**:
   ```bash
   # With mock Claude (no API key needed)
   CLAUDE_MODE=mock cargo run --release --bin hal9-server
   ```

## 📝 Commit Convention

```bash
# Format: [Layer] type: description
[L2] feat: Add A2A protocol for neuron discovery
[L3] fix: Improve WebSocket connection stability
[L9] docs: Explain consciousness emergence patterns
```

## 🎯 Key Files to Understand

1. **Neuron Core**: `layers/L2_implementation/neurons/core/neuron.rs`
   - Basic neuron abstraction and interfaces

2. **Self-Organization**: `layers/L2_implementation/neurons/core/hierarchical/`
   - A2A protocol, emergence patterns, consciousness metrics

3. **Server Implementation**: `layers/L3_operational/architecture/server/`
   - HTTP/WebSocket APIs, neuron management, routing

4. **Performance Benchmarks**: `layers/L2_implementation/neurons/examples/`
   - Demonstrates real self-organization speed

## ⚡ Performance Optimization

The codebase is optimized for:
- **CPU cache locality**: Neurons stored contiguously
- **Branch prediction**: Predictable layer routing
- **Zero-copy operations**: Direct memory access where possible
- **Async I/O**: Non-blocking operations throughout

See `.cargo/config.toml` for platform-specific optimizations.