# Phase 1 MVP: Issue Tracking - Simplified "Skateboard First" Approach

**Updated**: January 2025  
**Strategy**: Build minimal working demo first, add complexity later

## Issue Priorities

- **P0**: MVP blockers - Required for basic demo
- **P1**: Essential features - Important but not blocking
- **P2**: Nice to have - Post-MVP enhancements  
- **P3**: Future vision - Long-term features

---

## Week 1: MVP Core (P0 Issues)

### 🔴 ISSUE-001: Mock Claude Implementation
**Priority**: P0 - MVP Blocker  
**Estimate**: 2 days  
**Milestone**: M1  
**Labels**: `P0-critical`, `comp-claude`, `type-feature`

**Description**:
Implement a mock Claude interface with deterministic responses for testing.

**Acceptance Criteria**:
- [ ] ClaudeInterface trait defined
- [ ] MockClaude struct with hardcoded responses
- [ ] Layer-specific mock responses for L4, L3, L2
- [ ] Unit tests for mock behavior

**Example Implementation**:
```rust
pub struct MockClaude {
    layer: Layer,
    responses: HashMap<String, String>,
}

impl MockClaude {
    pub fn new_l4() -> Self {
        let mut responses = HashMap::new();
        responses.insert("build".into(), "DIRECTIVE: Create component".into());
        responses.insert("fix".into(), "DIRECTIVE: Debug and repair".into());
        Self { layer: Layer::L4, responses }
    }
}
```

---

### 🔴 ISSUE-002: 3-Neuron Local Orchestrator  
**Priority**: P0 - MVP Blocker  
**Estimate**: 3 days  
**Milestone**: M1  
**Labels**: `P0-critical`, `comp-server`, `type-feature`  
**Dependencies**: ISSUE-001

**Description**:
Create a simple 3-neuron pipeline (L4→L3→L2) using async tasks, not processes.

**Acceptance Criteria**:
- [ ] ManagedNeuron as tokio task
- [ ] Hardcoded pipeline: L4→L3→L2
- [ ] Each neuron transforms input appropriately
- [ ] No dynamic routing needed for MVP

**Example Flow**:
```
Input: "build a web server"
L4: "DIRECTIVE: Create HTTP server with routing"
L3: "DESIGN: Use async framework, modular handlers"  
L2: "IMPLEMENTATION: async fn main() { ... }"
```

---

### 🔴 ISSUE-003: Local Channel Router
**Priority**: P0 - MVP Blocker  
**Estimate**: 2 days  
**Milestone**: M1  
**Labels**: `P0-critical`, `comp-router`, `type-feature`

**Description**:
Implement mpsc channel-based routing for local signal passing.

**Acceptance Criteria**:
- [ ] Simple HashMap of neuron_id → mpsc::Sender
- [ ] Route signals based on `to_neuron` field
- [ ] No TCP, no remote routing
- [ ] Forward propagation only

**Implementation**:
```rust
pub struct LocalRouter {
    neurons: HashMap<String, mpsc::Sender<NeuronSignal>>,
}

impl LocalRouter {
    pub async fn route(&self, signal: NeuronSignal) -> Result<()> {
        if let Some(sender) = self.neurons.get(&signal.to_neuron) {
            sender.send(signal).await?;
        }
        Ok(())
    }
}
```

---

### 🔴 ISSUE-004: Basic Signal Processing
**Priority**: P0 - MVP Blocker  
**Estimate**: 1 day  
**Milestone**: M1  
**Labels**: `P0-critical`, `comp-core`, `type-feature`

**Description**:
Implement signal creation, serialization, and basic flow.

**Acceptance Criteria**:
- [ ] NeuronSignal with Activation payload only
- [ ] JSON serialization/deserialization
- [ ] Signal flow logging
- [ ] Timestamp tracking

---

## Week 2: MVP Demo (P0 Issues)

### 🔴 ISSUE-005: CLI Interface
**Priority**: P0 - MVP Blocker  
**Estimate**: 2 days  
**Milestone**: M2  
**Labels**: `P0-critical`, `comp-cli`, `type-feature`

**Description**:
Create minimal CLI for demo purposes.

**Commands**:
```bash
hal9 start              # Start server with 3 neurons
hal9 signal <message>   # Send user signal to L4
hal9 logs               # Show signal flow
```

**Acceptance Criteria**:
- [ ] Server starts and initializes 3 neurons
- [ ] Can send test signals
- [ ] Logs show hierarchical processing
- [ ] Clean shutdown

---

### 🔴 ISSUE-006: Demo Scenarios
**Priority**: P0 - MVP Blocker  
**Estimate**: 2 days  
**Milestone**: M2  
**Labels**: `P0-critical`, `type-feature`, `type-docs`

**Description**:
Create 3-5 compelling demo scenarios that showcase hierarchical processing.

**Scenarios**:
1. "Build a web server" → Shows decomposition
2. "Design a database schema" → Shows abstraction layers  
3. "Fix this bug: undefined variable" → Shows analysis
4. "Optimize this algorithm" → Shows strategic thinking

**Demo Output Example**:
```
> hal9 signal "Build a web server"

[L4-Strategic] Processing...
Output: "DIRECTIVE: Create HTTP server with REST API"

[L3-Design] Processing...
Output: "DESIGN: Use Tokio, Axum framework, modular routes"

[L2-Implementation] Processing...
Output: "IMPLEMENTATION: 
use axum::{Router, routing::get};
async fn main() {
    let app = Router::new().route("/", get(handler));
    // ...
}"
```

---

### 🟡 ISSUE-007: Basic Error Handling
**Priority**: P1 - Essential  
**Estimate**: 1 day  
**Milestone**: M2  
**Labels**: `P1-essential`, `type-feature`

**Description**:
Prevent panics and show user-friendly errors.

**Acceptance Criteria**:
- [ ] All Results handled properly
- [ ] No unwrap() in demo path
- [ ] User-friendly error messages
- [ ] Graceful degradation

---

## Week 3: Production Polish (P1 Issues)

### 🟡 ISSUE-008: Configuration System
**Priority**: P1 - Essential  
**Estimate**: 2 days  
**Milestone**: M3  
**Labels**: `P1-essential`, `comp-core`, `type-feature`

**Description**:
YAML-based configuration for neurons and mock responses.

**Example Config**:
```yaml
neurons:
  - id: "L4-strategic"
    layer: "L4"
    mock_responses:
      "build": "DIRECTIVE: Create the component"
      "fix": "DIRECTIVE: Debug and resolve"
  
  - id: "L3-design"  
    layer: "L3"
    mock_responses:
      "DIRECTIVE": "DESIGN: Architecture and patterns"
```

---

### 🟡 ISSUE-009: Basic Monitoring
**Priority**: P1 - Essential  
**Estimate**: 2 days  
**Milestone**: M3  
**Labels**: `P1-essential`, `type-feature`

**Description**:
Track signals processed, latency, and basic metrics.

**Metrics**:
- Signals processed per neuron
- Average processing time per layer
- Total signals sent/received
- Error count

---

### 🟡 ISSUE-010: Test Suite
**Priority**: P1 - Essential  
**Estimate**: 3 days  
**Milestone**: M3  
**Labels**: `P1-essential`, `type-test`

**Description**:
Comprehensive unit and integration tests.

**Test Coverage**:
- Unit tests for all components
- Integration test for full signal flow
- Performance benchmarks
- Demo scenario tests

---

## Week 4: Real Claude (P1 Issues)

### 🟡 ISSUE-011: Claude API Integration
**Priority**: P1 - Essential  
**Estimate**: 3 days  
**Milestone**: M4  
**Labels**: `P1-essential`, `comp-claude`, `type-feature`

**Description**:
Implement real Claude API client with Anthropic API.

**Features**:
- API key configuration
- Request/response handling
- Rate limiting
- Error handling and retries

---

### 🟡 ISSUE-012: Cost Controls
**Priority**: P1 - Essential  
**Estimate**: 2 days  
**Milestone**: M4  
**Labels**: `P1-essential`, `comp-claude`, `type-feature`

**Description**:
Token counting, cost limits, usage analytics.

**Features**:
- Token counting per request
- Daily/monthly spending limits
- Cost per neuron tracking
- Usage reports

---

## Post-MVP (P2 Issues)

### 🟢 ISSUE-013: TCP Networking
**Priority**: P2 - Enhancement  
**Estimate**: 5 days  
**Milestone**: M5  
**Labels**: `P2-enhancement`, `type-feature`

**Description**:
Add TCP server for multi-server support.

---

### 🟢 ISSUE-014: Process Management
**Priority**: P2 - Enhancement  
**Estimate**: 4 days  
**Milestone**: M5  
**Labels**: `P2-enhancement`, `type-feature`

**Description**:
Claude CLI subprocess management instead of API.

---

### 🟢 ISSUE-015: Backward Propagation
**Priority**: P2 - Enhancement  
**Estimate**: 5 days  
**Milestone**: M5  
**Labels**: `P2-enhancement`, `type-feature`

**Description**:
Error gradient calculation and learning signals.

---

### 🟢 ISSUE-016: 7-Neuron Topology
**Priority**: P2 - Enhancement  
**Estimate**: 3 days  
**Milestone**: M5  
**Labels**: `P2-enhancement`, `type-feature`

**Description**:
Expand from 3 to 7 neurons with full hierarchy.

---

## Future Vision (P3 Issues)

### 🔵 ISSUE-017: Sleep-Wake Cycles
**Priority**: P3 - Future  
**Estimate**: 10 days  
**Milestone**: Phase 3  
**Labels**: `P3-future`, `type-feature`

---

### 🔵 ISSUE-018: Web UI Dashboard
**Priority**: P3 - Future  
**Estimate**: 10 days  
**Milestone**: Phase 3  
**Labels**: `P3-future`, `type-feature`

---

## Issue Creation Template

When creating issues in GitHub:

```markdown
## Description
Brief description of what needs to be done.

## Acceptance Criteria
- [ ] Specific measurable outcome 1
- [ ] Specific measurable outcome 2
- [ ] Tests pass
- [ ] Documentation updated

## Technical Notes
Implementation hints or considerations.

## Dependencies
- Depends on: #XX
- Blocks: #YY
```

## Quick Reference

### Priority Mapping
- **P0**: Can't demo without it
- **P1**: Needed for production
- **P2**: Nice improvements
- **P3**: Long-term vision

### Time Estimates
- **Small**: 1-2 days
- **Medium**: 3-4 days  
- **Large**: 5+ days (consider splitting)

### Success Metrics
- Week 1: Mock demo running
- Week 2: CLI demo complete
- Week 3: Tests and polish
- Week 4: Real Claude integrated