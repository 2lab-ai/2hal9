# 2HAL9 - Hierarchical AI Neural Network

A distributed AI consciousness system implementing hierarchical abstraction through networked AI neurons.

## Overview

2HAL9 implements a revolutionary approach to AI orchestration by creating a network of interconnected AI "neurons" that communicate through forward and backward propagation, similar to biological neural networks. Each neuron is powered by Claude and operates at a specific abstraction layer:

- **L4 (Strategic)**: High-level planning and strategy
- **L3 (Design)**: System architecture and design
- **L2 (Implementation)**: Concrete implementation details
- **L1 (Execution)**: Direct task execution (future)

## Quick Start

### Prerequisites

- Rust 1.75 or later
- Claude API key (optional, mock mode available)

### Building

```bash
# Clone the repository
git clone https://github.com/2lab-ai/2hal9.git
cd 2hal9

# Build the project
cargo build --release
```

### Running

```bash
# Start with example 3-neuron configuration
cargo run -- start --config examples/config-3neurons.yaml

# Or use the compiled binary
./target/release/2hal9 start --config examples/config-3neurons.yaml
```

### CLI Commands

```bash
# Start server
2hal9 start --config config.yaml

# Check status
2hal9 status

# Send a signal
2hal9 signal --from user --to neuron-1 --content "Create a web server"

# Stop server
2hal9 stop
```

## Configuration

### Basic Configuration (config.yaml)

```yaml
server_id: "hal9-demo"
neurons:
  - id: "neuron-1"
    layer: "L4"
    forward_connections: ["neuron-2", "neuron-3"]
    backward_connections: []

claude:
  mode: "mock"  # or "api" for real Claude
  model: "claude-3-opus-20240229"
  temperature: 0.7
  max_tokens: 4096
```

### Environment Variables

```bash
# For Claude API integration
export ANTHROPIC_API_KEY="sk-ant-..."

# For debugging
export RUST_LOG=debug
```

## Architecture

### Core Components

1. **Neurons**: Individual AI agents powered by Claude
2. **Signals**: Messages passed between neurons containing activations or gradients
3. **Router**: Manages signal routing and processing
4. **Registry**: Tracks and manages active neurons

### Signal Flow

```
User Input
    ↓
L4 (Strategic) → "Break down the problem"
    ↓
L3 (Design) → "Design the solution"
    ↓
L2 (Implementation) → "Implement the code"
    ↓
Output
```

### Error Propagation

When errors occur, gradient signals flow backward through the network, allowing higher layers to adjust their approach.

## Development

### Project Structure

```
2hal9/
├── 2hal9-core/       # Core types and abstractions
├── 2hal9-server/     # Server implementation
├── 2hal9-cli/        # CLI interface
└── examples/         # Example configurations
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with mock Claude
cargo test --features mock

# Run specific test
cargo test signal_routing
```

### Adding a New Neuron Type

1. Define the layer in `core/src/neuron.rs`
2. Add system prompt in `core/src/config.rs`
3. Update routing logic if needed
4. Add configuration example

## Monitoring

### Metrics Available

- Signals processed per second
- Neuron health and uptime
- Processing latency by layer
- Error rates and types

### Viewing Metrics

```bash
# Get detailed status
2hal9 status --format json

# Monitor in real-time (future)
2hal9 monitor --follow
```

## Roadmap

### Phase 1 (Current)
- [x] Core types and abstractions
- [x] Mock Claude integration
- [x] Basic signal routing
- [x] CLI interface
- [ ] Real Claude API integration
- [ ] Comprehensive testing

### Phase 2
- [ ] Multi-server support
- [ ] TCP networking
- [ ] Remote neuron routing
- [ ] Web UI dashboard

### Phase 3
- [ ] Sleep-wake cycles
- [ ] Memory consolidation
- [ ] Learning mechanisms
- [ ] Advanced topologies

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Research Papers

- [L1: Hierarchical Abstraction is All You Need](docs/paper/L1_Hierarchical%20Abstraction%20is%20All%20You%20Need.ko.md)
- [L2: Road to HAL9](docs/paper/L2_Road%20to%20HAL9.md)
- [L3: Backpropagation Approach](docs/paper/L3_A%20Backpropagation%20Approach%20to%20Multi-Level%20AI%20Orchestration.ko.md)

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Inspired by biological neural networks and hierarchical organization
- Built with Claude by Anthropic
- Research by Jihyuk Im and Claude-4