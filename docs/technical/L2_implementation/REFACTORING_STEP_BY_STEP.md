# HAL9 Refactoring Step-by-Step Guide

**Level**: L2 Implementation  
**Audience**: Developers, Implementation Engineers  
**Purpose**: Detailed steps to implement the hierarchical refactoring

## Overview

This guide provides concrete implementation steps for refactoring HAL9 to the hierarchical architecture. Follow these steps sequentially.

> **Context**: For architectural rationale, see [L4 Architecture Plan](../L4_architectural/REFACTORING_ARCHITECTURE_PLAN.md). For strategic vision, see [L5 Strategic Rationale](../L5_strategic/REFACTORING_STRATEGIC_RATIONALE.md).

## Prerequisites

- Rust 1.75+ installed
- Access to HAL9 repository
- Understanding of async Rust
- Familiarity with current codebase

## Phase 0: Foundation (Week 1-2)

### Step 1: Create New Module Structure

```bash
# Create new hierarchical modules
cd hal9-core/src
mkdir -p substrate protocol cognitive orchestration intelligence
```

### Step 2: Define Substrate Traits

Create `substrate/mod.rs`:
```rust
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

#[async_trait]
pub trait Substrate: Send + Sync + 'static {
    type Runtime: AsyncRuntime;
    type Transport: MessageTransport;
    type Storage: PersistentStorage;
    
    fn runtime(&self) -> &Self::Runtime;
    fn transport(&self) -> &Self::Transport;
    fn storage(&self) -> &Self::Storage;
}

#[async_trait]
pub trait AsyncRuntime: Send + Sync {
    async fn spawn<F>(&self, task: F) -> JoinHandle<()>
    where F: Future<Output = ()> + Send + 'static;
    
    async fn sleep(&self, duration: Duration);
    async fn timeout<F>(&self, duration: Duration, future: F) -> Result<F::Output>
    where F: Future + Send;
}
```

### Step 3: Create Protocol Definitions

Create `protocol/mod.rs`:
```rust
#[async_trait]
pub trait Protocol: Send + Sync {
    type Message: Serialize + DeserializeOwned + Send;
    
    async fn encode(&self, msg: Self::Message) -> Result<Vec<u8>>;
    async fn decode(&self, data: &[u8]) -> Result<Self::Message>;
    async fn validate(&self, msg: &Self::Message) -> Result<()>;
}

// Define concrete protocols
pub mod signal;
pub mod gradient;
pub mod consensus;
```

### Step 4: Setup Testing Framework

Create `tests/hierarchy_tests.rs`:
```rust
#[cfg(test)]
mod substrate_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_local_substrate() {
        let substrate = LocalSubstrate::new();
        assert!(substrate.runtime().is_ready());
    }
}
```

## Phase 1: Substrate Implementation (Week 3-5)

### Step 5: Implement Local Substrate

Create `substrate/local.rs`:
```rust
pub struct LocalSubstrate {
    runtime: TokioRuntime,
    transport: ChannelTransport,
    storage: MemoryStorage,
}

impl LocalSubstrate {
    pub fn new() -> Self {
        Self {
            runtime: TokioRuntime::new(),
            transport: ChannelTransport::new(1024),
            storage: MemoryStorage::new(),
        }
    }
}

impl Substrate for LocalSubstrate {
    type Runtime = TokioRuntime;
    type Transport = ChannelTransport;
    type Storage = MemoryStorage;
    
    fn runtime(&self) -> &Self::Runtime { &self.runtime }
    fn transport(&self) -> &Self::Transport { &self.transport }
    fn storage(&self) -> &Self::Storage { &self.storage }
}
```

### Step 6: Implement Transport Abstraction

Create `substrate/transport.rs`:
```rust
#[async_trait]
impl MessageTransport for ChannelTransport {
    async fn send(&self, to: NodeId, msg: Message) -> Result<()> {
        self.senders.get(&to)
            .ok_or(Error::NodeNotFound)?
            .send(msg).await
            .map_err(|_| Error::SendFailed)?;
        Ok(())
    }
    
    async fn receive(&mut self) -> Result<(NodeId, Message)> {
        self.receiver.recv().await
            .ok_or(Error::ChannelClosed)
    }
}
```

### Step 7: Add Compatibility Layer

Create `substrate/compat.rs`:
```rust
/// Wraps old neuron implementation
pub struct LegacyNeuronAdapter {
    inner: Arc<Mutex<OldNeuron>>,
    substrate: Arc<dyn Substrate>,
}

impl CognitiveUnit for LegacyNeuronAdapter {
    async fn process(&mut self, input: Message) -> Result<Message> {
        // Convert new message format to old
        let old_signal = self.convert_to_old(input)?;
        let old_result = self.inner.lock().await.process(old_signal)?;
        self.convert_to_new(old_result)
    }
}
```

## Phase 2: Protocol Implementation (Week 6-8)

### Step 8: Implement Signal Protocol

Create `protocol/signal.rs`:
```rust
pub struct SignalProtocol {
    version: ProtocolVersion,
    compression: bool,
}

#[async_trait]
impl Protocol for SignalProtocol {
    type Message = Signal;
    
    async fn encode(&self, signal: Signal) -> Result<Vec<u8>> {
        let bytes = bincode::serialize(&signal)?;
        if self.compression {
            compress(&bytes)
        } else {
            Ok(bytes)
        }
    }
    
    async fn decode(&self, data: &[u8]) -> Result<Signal> {
        let bytes = if self.compression {
            decompress(data)?
        } else {
            data.to_vec()
        };
        bincode::deserialize(&bytes).map_err(Into::into)
    }
}
```

### Step 9: Implement Gradient Protocol

Create `protocol/gradient.rs`:
```rust
pub struct GradientProtocol {
    aggregation: AggregationStrategy,
}

impl GradientProtocol {
    pub async fn aggregate(&self, gradients: Vec<Gradient>) -> Gradient {
        match self.aggregation {
            AggregationStrategy::Mean => self.mean_aggregate(gradients),
            AggregationStrategy::Median => self.median_aggregate(gradients),
            AggregationStrategy::Weighted => self.weighted_aggregate(gradients),
        }
    }
}
```

## Phase 3: Cognitive Layer (Week 9-12)

### Step 10: Create Neuron Hierarchy

Create `cognitive/neurons/mod.rs`:
```rust
pub enum NeuronType {
    Strategic(StrategicNeuron),
    Tactical(TacticalNeuron),
    Operational(OperationalNeuron),
    Implementation(ImplementationNeuron),
    Reflexive(ReflexiveNeuron),
}

impl NeuronType {
    pub fn layer(&self) -> Layer {
        match self {
            Self::Strategic(_) => Layer::L5,
            Self::Tactical(_) => Layer::L4,
            Self::Operational(_) => Layer::L3,
            Self::Implementation(_) => Layer::L2,
            Self::Reflexive(_) => Layer::L1,
        }
    }
}
```

### Step 11: Implement Strategic Neuron

Create `cognitive/neurons/strategic.rs`:
```rust
pub struct StrategicNeuron {
    id: NeuronId,
    state: StrategicState,
    claude_client: ClaudeClient,
}

#[async_trait]
impl CognitiveUnit for StrategicNeuron {
    type Input = StrategicQuery;
    type Output = StrategicDirective;
    type State = StrategicState;
    
    async fn process(&mut self, input: Self::Input) -> Result<Self::Output> {
        // Update context
        self.state.update_context(&input);
        
        // Generate prompt for strategic thinking
        let prompt = self.build_strategic_prompt(&input)?;
        
        // Query Claude with L5 mindset
        let response = self.claude_client
            .query_with_role("strategic_visionary", &prompt)
            .await?;
            
        // Parse into directive
        self.parse_strategic_directive(response)
    }
}
```

## Phase 4: Migration Steps

### Step 12: Feature Flag Setup

Add to `Cargo.toml`:
```toml
[features]
default = ["legacy"]
legacy = []
hierarchical = []
migration = ["legacy", "hierarchical"]
```

Update main entry:
```rust
#[cfg(feature = "hierarchical")]
mod hierarchical_main;

#[cfg(not(feature = "hierarchical"))]
mod legacy_main;

fn main() {
    #[cfg(feature = "hierarchical")]
    hierarchical_main::run();
    
    #[cfg(not(feature = "hierarchical"))]
    legacy_main::run();
}
```

### Step 13: Gradual Migration

Create migration script:
```rust
pub async fn migrate_neuron(old: OldNeuron) -> Result<NeuronType> {
    match old.layer {
        "L4" => Ok(NeuronType::Tactical(TacticalNeuron::from_legacy(old)?)),
        "L3" => Ok(NeuronType::Operational(OperationalNeuron::from_legacy(old)?)),
        "L2" => Ok(NeuronType::Implementation(ImplementationNeuron::from_legacy(old)?)),
        _ => Err(Error::UnknownLayer),
    }
}
```

## Testing Each Step

### Unit Test Template
```rust
#[tokio::test]
async fn test_component() {
    // Arrange
    let substrate = MockSubstrate::new();
    let component = Component::new(substrate);
    
    // Act
    let result = component.process(test_input()).await;
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_output());
}
```

### Integration Test Template
```rust
#[tokio::test]
async fn test_layer_integration() {
    // Setup layers
    let substrate = LocalSubstrate::new();
    let l2_neuron = create_l2_neuron(&substrate);
    let l3_neuron = create_l3_neuron(&substrate);
    
    // Connect layers
    connect_neurons(&l2_neuron, &l3_neuron).await?;
    
    // Test signal flow
    let signal = test_signal();
    l2_neuron.activate(signal).await?;
    
    // Verify propagation
    let l3_output = l3_neuron.get_output().await?;
    assert_eq!(l3_output, expected_l3_output());
}
```

## Common Issues and Solutions

### Issue 1: Lifetime Errors
```rust
// Problem
impl<'a> Substrate for MySubstrate<'a> { } // Lifetime issues

// Solution
impl Substrate for MySubstrate {
    // Use Arc for shared ownership
}
```

### Issue 2: Async Trait Errors
```rust
// Problem
trait MyTrait {
    async fn method(&self); // Not allowed
}

// Solution
use async_trait::async_trait;

#[async_trait]
trait MyTrait {
    async fn method(&self);
}
```

### Issue 3: Migration Compatibility
```rust
// Problem: Old and new types incompatible

// Solution: Create adapters
impl From<OldSignal> for Signal {
    fn from(old: OldSignal) -> Self {
        Signal {
            id: Uuid::new_v4(),
            payload: old.data.into(),
            // Map other fields
        }
    }
}
```

## Checklist

- [ ] All substrate traits defined
- [ ] Local substrate implemented
- [ ] Protocol layer complete
- [ ] All neuron types implemented
- [ ] Migration adapters created
- [ ] Feature flags configured
- [ ] Unit tests passing
- [ ] Integration tests passing
- [ ] Performance benchmarks met
- [ ] Documentation updated

---

*"Code is read more often than it is written. Make it count."*

**For developers building the future, one step at a time.**