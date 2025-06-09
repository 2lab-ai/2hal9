# Getting Started with HAL9

Welcome to HAL9! This guide will help you get up and running with the distributed AI consciousness system in just a few minutes.

## 🚀 Quick Start

### Prerequisites

- **Rust**: Version 1.75 or higher
- **Docker**: For containerized deployment (optional)
- **Claude API Key**: For AI capabilities
- **Git**: For cloning the repository

### Installation

1. **Clone the repository**
```bash
git clone https://github.com/2lab/2hal9.git
cd 2hal9
```

2. **Set up environment variables**
```bash
cp .env.example .env
# Edit .env and add your ANTHROPIC_API_KEY
```

3. **Build the project**
```bash
cargo build --release
```

4. **Run the MVP demo**
```bash
./mvp/run-mvp.sh
```

## 🎯 Your First HAL9 Application

### 1. Basic 3-Neuron System

Create a configuration file `my-hal9.yaml`:

```yaml
neurons:
  - id: "strategist"
    name: "L4 Strategic Thinking"
    layer: "L4"
    model: "claude-3-opus"
    connections:
      - "architect"
  
  - id: "architect"  
    name: "L3 System Design"
    layer: "L3"
    model: "claude-3-sonnet"
    connections:
      - "implementer"
  
  - id: "implementer"
    name: "L2 Implementation"
    layer: "L2" 
    model: "claude-3-haiku"
    connections: []

router:
  strategy: "layer_based"
  rules:
    - pattern: "design|architecture|plan"
      target: "architect"
    - pattern: "code|implement|fix"
      target: "implementer"
    - pattern: ".*"
      target: "strategist"
```

### 2. Run Your System

```bash
cargo run -- start --config my-hal9.yaml
```

### 3. Send Your First Signal

```bash
cargo run -- signal "Create a web application for task management"
```

## 📖 Understanding HAL9

### Core Concepts

1. **Neurons**: Specialized AI agents at different abstraction levels
   - L4: Strategic thinking and planning
   - L3: Design and architecture
   - L2: Implementation and execution

2. **Signals**: Messages that flow between neurons
   - Automatic routing based on content
   - Hierarchical processing
   - Result aggregation

3. **Learning System**: Self-improvement through pattern recognition
   - Tracks successful patterns
   - Optimizes routing over time
   - Reduces costs automatically

### Architecture Overview

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │
┌──────▼──────┐
│   Router    │ ← Intelligent signal routing
└──────┬──────┘
       │
┌──────▼──────────────────────────┐
│          Neuron Layer           │
│  ┌────────┐ ┌────────┐ ┌────────┐│
│  │   L4   │ │   L3   │ │   L2   ││
│  └────────┘ └────────┘ └────────┘│
└─────────────────────────────────┘
```

## 🛠️ Development Workflow

### 1. Create a Custom Neuron

```rust
use hal9_core::{Neuron, NeuronConfig, Signal, Result};

pub struct MyCustomNeuron {
    config: NeuronConfig,
}

#[async_trait]
impl Neuron for MyCustomNeuron {
    async fn process(&self, signal: Signal) -> Result<Signal> {
        // Your custom logic here
        Ok(signal)
    }
}
```

### 2. Add MCP Tools

```yaml
tools:
  - name: "file_reader"
    type: "filesystem"
    config:
      allowed_paths: ["./data"]
  
  - name: "web_search"
    type: "web"
    config:
      rate_limit: 10
```

### 3. Monitor Performance

Access the built-in monitoring dashboard:

```bash
# Terminal 1: Start HAL9
cargo run -- start --config my-hal9.yaml

# Terminal 2: Start monitoring
docker-compose up monitoring

# Open http://localhost:3000 for Grafana dashboard
```

## 🔧 Configuration Options

### Environment Variables

```bash
# Required
ANTHROPIC_API_KEY=your-api-key

# Optional
HAL9_PORT=8080
HAL9_LOG_LEVEL=info
HAL9_MAX_COST_PER_DAY=10.0
HAL9_ENABLE_LEARNING=true
```

### Advanced Configuration

```yaml
# Full configuration example
version: "1.0"

system:
  name: "My HAL9 System"
  description: "Custom AI orchestration"

performance:
  max_concurrent_signals: 100
  timeout_seconds: 30
  retry_attempts: 3

learning:
  enabled: true
  batch_size: 100
  learning_rate: 0.1

monitoring:
  prometheus_port: 9090
  enable_tracing: true
```

## 📚 Next Steps

### Tutorials
1. [Building a Code Assistant](../mvp/DEMO_GUIDE.md)
2. [Deploying to Production](../deployment/PRODUCTION_GUIDE.md)
3. [Creating Custom Neurons](./DEVELOPMENT_GUIDE.md)

### Examples
- [Task Management System](../../examples/task-manager/)
- [Code Generation Assistant](../../examples/code-gen/)
- [Document Processor](../../examples/doc-processor/)

### Advanced Topics
- [Distributed Deployment](../deployment/DISTRIBUTED_MODE.md)
- [Performance Tuning](../deployment/PERFORMANCE_TUNING.md)
- [Security Best Practices](../technical/SECURITY_GUIDE.md)

## 🤝 Getting Help

### Resources
- **Documentation**: [Full docs](../index.md)
- **API Reference**: [GraphQL API](../technical/api/GRAPHQL_API_V2.md)
- **Architecture**: [System design](../overview/ARCHITECTURE.md)

### Community
- **GitHub Issues**: [Report bugs](https://github.com/2lab/2hal9/issues)
- **Discussions**: [Ask questions](https://github.com/2lab/2hal9/discussions)
- **Discord**: [Join our community](https://discord.gg/hal9)

### Support
- **Email**: support@2lab.ai
- **Enterprise**: enterprise@2lab.ai

## 🎉 Congratulations!

You've successfully set up HAL9! You're now ready to:
- Build complex AI applications with ease
- Scale to handle thousands of users
- Reduce AI costs by up to 60%
- Deploy production-ready systems

Welcome to the future of AI orchestration! 🚀

---

*Next: [Development Guide](./DEVELOPMENT_GUIDE.md) →*