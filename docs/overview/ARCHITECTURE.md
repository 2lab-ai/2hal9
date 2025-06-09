# HAL9 Architecture Document

**Version**: 2.0  
**Date**: January 2025  
**Status**: Simplified MVP Architecture

## Overview

This document describes the simplified architecture for the HAL9 MVP, focusing on proving the hierarchical AI orchestration concept with minimal complexity.

## MVP Architecture (Phase 1)

### System Design - "Skateboard First"

```
┌─────────────────────────────────────────┐
│          HAL9 Server (Single)          │
├─────────────────────────────────────────┤
│                                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐│
│  │   L4    │  │   L3    │  │   L2    ││
│  │Strategic│→ │ Design  │→ │ Impl.   ││
│  │ Neuron  │  │ Neuron  │  │ Neuron  ││
│  └─────────┘  └─────────┘  └─────────┘│
│       ↓            ↓            ↓       │
│  ┌─────────────────────────────────┐   │
│  │     Mock Claude Interface       │   │
│  │   (Deterministic Responses)     │   │
│  └─────────────────────────────────┘   │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │    Local Channel Router         │   │
│  │     (mpsc channels only)        │   │
│  └─────────────────────────────────┘   │
│                                         │
└─────────────────────────────────────────┘
           ↑
           │ CLI Commands
┌──────────┴──────────┐
│   hal9 CLI Tool    │
│  - start server     │
│  - send signal      │
│  - view logs        │
└─────────────────────┘
```

### Core Components (Simplified)

#### 1. Mock Claude Interface
```rust
pub trait ClaudeInterface {
    async fn send_message(&self, message: &str) -> Result<String>;
}

pub struct MockClaude {
    layer: Layer,
    responses: HashMap<String, String>, // Deterministic
}
```

#### 2. Managed Neuron (Async Task)
```rust
pub struct ManagedNeuron {
    id: String,
    layer: Layer,
    claude: Box<dyn ClaudeInterface>,
    rx: mpsc::Receiver<NeuronSignal>,
    tx: mpsc::Sender<NeuronSignal>,
}
```

#### 3. Simple Router
```rust
pub struct LocalRouter {
    neurons: HashMap<String, mpsc::Sender<NeuronSignal>>,
}
```

### Signal Flow (MVP)

1. **User Input** → CLI → Server
2. **L4 Processing**:
   - Receives: "Build a web server"
   - Outputs: "DIRECTIVE: Create HTTP server with routing"
3. **L3 Processing**:
   - Receives: L4 output
   - Outputs: "DESIGN: Use async server, route handlers, middleware"
4. **L2 Processing**:
   - Receives: L3 output
   - Outputs: "IMPLEMENTATION: `async fn main() { ... }`"
5. **Result** → CLI → User

### What's NOT in MVP

- ❌ Real Claude API calls
- ❌ Process spawning
- ❌ TCP networking
- ❌ Multi-server support
- ❌ Backward propagation
- ❌ Complex routing tables
- ❌ Dynamic topology
- ❌ Persistent storage
- ❌ Advanced monitoring

## Phase 2 Architecture (Future)

### Distributed System Design

```
┌─────────────────┐         ┌─────────────────┐
│  HAL9 Server A │  TCP    │  HAL9 Server B │
│   Neurons 1-3   │←─────→  │   Neurons 4-7   │
└─────────────────┘         └─────────────────┘
```

### Added Components

1. **TCP Network Layer**
   - Server-to-server communication
   - Remote signal routing
   - Connection management

2. **Process Management**
   - Claude CLI subprocess spawning
   - Health monitoring
   - Restart on failure

3. **Backward Propagation**
   - Error gradient calculation
   - Learning signal distribution
   - Weight updates

## Data Structures

### Core Signal Type (Simplified for MVP)
```rust
pub struct NeuronSignal {
    pub id: Uuid,
    pub from_neuron: String,
    pub to_neuron: String,
    pub layer_from: Layer,
    pub layer_to: Layer,
    pub payload: SignalPayload,
    pub timestamp: DateTime<Utc>,
}

pub enum SignalPayload {
    Activation(Activation),  // Forward only in MVP
    // Gradient(Gradient),   // Phase 2
}
```

### Configuration (MVP)
```yaml
# Simple 3-neuron config
neurons:
  - id: "L4-strategic"
    layer: "L4"
    mock_responses:
      "create": "DIRECTIVE: Build the requested component"
      
  - id: "L3-design"
    layer: "L3"
    mock_responses:
      "DIRECTIVE": "DESIGN: Use modular architecture"
      
  - id: "L2-implementation"
    layer: "L2"
    mock_responses:
      "DESIGN": "IMPLEMENTATION: fn main() { ... }"
```

## Development Approach

### Week 1: Core Implementation
1. MockClaude trait implementation
2. Basic neuron as async task
3. Local channel routing
4. Signal processing loop

### Week 2: CLI and Demo
1. CLI command structure
2. Server lifecycle management
3. Demo scenarios
4. Basic logging

### Week 3: Polish
1. Error handling
2. Configuration loading
3. Tests
4. Documentation

### Week 4: Real Integration
1. Claude API client
2. Cost monitoring
3. Production config
4. Deployment guide

## Testing Strategy

### Unit Tests
- Mock Claude responses
- Signal routing logic
- Neuron state management

### Integration Tests
- Full signal flow (L4→L3→L2)
- CLI commands
- Configuration loading

### Demo Scenarios
1. "Create a web server"
2. "Design a database schema"
3. "Implement a sorting algorithm"
4. "Optimize this code"
5. "Debug this error"

## Performance Targets (MVP)

- **Latency**: < 10ms per layer (mocked)
- **Memory**: < 100MB total
- **CPU**: < 5% idle
- **Demo Time**: < 5 seconds total

## Security Considerations (MVP)

- No external network access
- No subprocess execution
- No file system writes
- Sandboxed operation

## Monitoring (Basic)

- Signal count per neuron
- Processing time per layer
- Error count
- Memory usage

## Future Evolution Path

1. **MVP** → Prove hierarchical orchestration works
2. **Real Claude** → Demonstrate with actual AI
3. **Distribution** → Scale across servers
4. **Intelligence** → Add learning mechanisms
5. **Evolution** → Scale to HAL1 and beyond

The key principle: Start simple, prove the concept, then add complexity incrementally.