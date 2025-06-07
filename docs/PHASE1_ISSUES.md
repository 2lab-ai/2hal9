# Phase 1 MVP: Issue Tracking

## Issue Template

Each issue follows this structure:
```markdown
**ID**: [Milestone.Issue]
**Title**: [Short descriptive title]
**Priority**: P0 (Critical) | P1 (High) | P2 (Medium) | P3 (Low)
**Estimate**: [Days]
**Dependencies**: [List of prerequisite issues]
**Assignee**: [TBD]
```

---

## Critical Path Issues (P0)

These must be completed in order for the MVP to function.

### 🔴 M1.1: Project Setup
**Priority**: P0  
**Estimate**: 2 days  
**Dependencies**: None  

```toml
# Cargo.toml structure
[workspace]
members = ["2hal9-server", "2hal9-cli", "2hal9-core"]

[workspace.dependencies]
tokio = { version = "1.36", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
```

**Definition of Done**:
- [ ] Workspace created with 3 crates
- [ ] All dependencies added
- [ ] `cargo check` passes
- [ ] GitHub Actions CI/CD configured

---

### 🔴 M1.2: Core Type Definitions
**Priority**: P0  
**Estimate**: 3 days  
**Dependencies**: M1.1  

```rust
// 2hal9-core/src/types.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronSignal {
    pub signal_id: Uuid,
    pub from_neuron: String,
    pub to_neuron: String,
    // ... rest of fields
}
```

**Definition of Done**:
- [ ] All types from draft implemented
- [ ] Serialization tests pass
- [ ] Documentation complete
- [ ] Example JSON files created

---

### 🔴 M2.1: Claude Process Wrapper
**Priority**: P0  
**Estimate**: 4 days  
**Dependencies**: M1.2  

**Implementation Tasks**:
```rust
pub struct ClaudeNeuron {
    pub id: String,
    pub layer: String,
    pub process: Child,
    pub stdin_tx: mpsc::Sender<String>,
    pub stdout_rx: mpsc::Receiver<String>,
}
```

**Test Scenarios**:
1. Spawn process successfully
2. Handle process crash
3. Graceful shutdown
4. Timeout handling

---

### 🔴 M2.2: Neuron-CLI Testing Module
**Priority**: P0  
**Estimate**: 3 days  
**Dependencies**: M2.1  

**Description**: Create a neuron-cli using Claude Process Wrapper for testing and as a fun side project module

**Implementation Tasks**:
- Create `neuron-cli` crate in workspace
- Implement Claude Process Wrapper integration
- Add testing commands and utilities
- Create interactive REPL for neuron testing
- Support direct signal injection and monitoring

**Features**:
```rust
// neuron-cli commands
neuron-cli spawn --layer L4 --id test-neuron
neuron-cli signal --to test-neuron --content "test signal"
neuron-cli monitor --neuron test-neuron
neuron-cli test --scenario forward-propagation
```

**Test Scenarios**:
1. Spawn test neurons with Claude wrapper
2. Send test signals and verify responses
3. Monitor neuron health and performance
4. Run automated test suites

**Definition of Done**:
- [ ] neuron-cli crate created and integrated
- [ ] Claude Process Wrapper fully functional
- [ ] Test commands implemented
- [ ] Documentation and examples provided
- [ ] Integration tests pass

---

### 🔴 M3.1: Signal Queue
**Priority**: P0  
**Estimate**: 3 days  
**Dependencies**: M2.1  

**Key Components**:
- Bounded mpsc channel (1000 capacity)
- Signal processor task
- Dead letter queue for failed signals
- Metrics collection

---

## High Priority Issues (P1)

Important for full functionality but not blocking.

### 🟡 M1.3: Configuration System
**Priority**: P1  
**Estimate**: 3 days  
**Dependencies**: M1.2  

**Config Example**:
```yaml
server_id: "hal9-0"
listen_addr: "127.0.0.1"
listen_port: 8080
neurons:
  - id: "neuron-1"
    layer: "L4"
    claude_command: "claude"
    forward_connections: ["neuron-2", "neuron-3"]
```

---

### 🟡 M2.3: Neuron Registry
**Priority**: P1  
**Estimate**: 3 days  
**Dependencies**: M2.1  

**Features**:
- Thread-safe neuron tracking
- Health check every 30s
- Automatic restart (max 3 attempts)
- Graceful shutdown coordination

---

### 🟡 M3.3: Forward Propagation
**Priority**: P1  
**Estimate**: 4 days  
**Dependencies**: M3.1, M2.3  

**Signal Flow**:
```
User Input → L4 (Strategy) → L3 (Design) → L2 (Implementation) → L1 (Execution)
```

---

## Medium Priority Issues (P2)

Enhance usability and observability.

### 🟢 M4.1: CLI Interface
**Priority**: P2  
**Estimate**: 3 days  
**Dependencies**: M3.3  

**Commands**:
```bash
2hal9 start --config config.yaml
2hal9 status
2hal9 signal --from user --to neuron-1 --content "Create a web server"
2hal9 logs --follow
```

---

### 🟢 M4.3: Basic Monitoring
**Priority**: P2  
**Estimate**: 3 days  
**Dependencies**: M3.1  

**Metrics to Track**:
- Signal processing rate
- Average latency per layer
- Neuron uptime
- Error rates

---

## Implementation Order

### Week 1-2: Foundation
1. M1.1: Project Setup ✓
2. M1.2: Core Types ✓
3. M1.4: Error Handling ✓

### Week 3-4: Process Management
4. M2.1: Claude Wrapper ✓
5. M2.2: Communication Channels ✓

### Week 5-6: Core Functionality
6. M3.1: Signal Queue ✓
7. M2.3: Neuron Registry ✓

### Week 7-8: Signal Flow
8. M1.3: Configuration ✓
9. M3.2: Local Routing ✓
10. M3.3: Forward Propagation ✓

### Week 9-10: Backward Flow
11. M2.4: Prompt Formatting ✓
12. M3.4: Backward Propagation ✓

### Week 11-12: Polish
13. M4.1: CLI Interface ✓
14. M4.3: Monitoring ✓
15. M4.2: REPL ✓
16. M4.4: Demos ✓

---

## Quick Start Issues

For rapid prototyping, these 5 issues give a working demo:

### 🚀 Quick Demo Path
1. **QD1**: Minimal types (1 day)
   - Just NeuronSignal and basic types
   
2. **QD2**: Mock Claude process (1 day)
   - Echo server that responds to signals
   
3. **QD3**: Simple router (2 days)
   - Hardcoded 3-neuron setup
   
4. **QD4**: Basic CLI (1 day)
   - Start server and send test signal
   
5. **QD5**: Demo script (1 day)
   - Show signal flowing through layers

**Total**: 6 days to working demo

---

## Testing Strategy

### Unit Test Coverage Goals
- Core types: 100%
- Process management: 80%
- Signal routing: 90%
- CLI: 70%

### Integration Tests
```rust
#[tokio::test]
async fn test_full_signal_flow() {
    // Spawn 3 neurons
    // Send signal to L4
    // Verify it reaches L2
    // Verify response returns
}
```

### Mock Claude Implementation
```rust
// For testing without real Claude
pub struct MockClaude {
    responses: HashMap<String, String>,
}
```

---

## Acceptance Criteria Checklist

### Minimum Viable Product
- [ ] Can spawn at least 3 neurons in hierarchy
- [ ] Signals flow from L4 → L3 → L2
- [ ] Failed signals trigger backward propagation
- [ ] Crashed neurons automatically restart
- [ ] CLI can start/stop/monitor system
- [ ] Logs clearly show signal flow
- [ ] Configuration is validated on load
- [ ] Graceful shutdown works

### Stretch Goals
- [ ] 7-neuron full hierarchy
- [ ] Dynamic neuron addition
- [ ] Signal replay for debugging
- [ ] Performance dashboard
- [ ] Docker container support