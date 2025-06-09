# HAL9 Code Migration Guide

**Level**: L2 Implementation  
**Audience**: Developers migrating existing code  
**Purpose**: Practical guide for migrating from flat to hierarchical architecture

## Migration Overview

This guide helps you migrate existing HAL9 code to the new hierarchical architecture with minimal disruption.

## Pre-Migration Checklist

- [ ] Backup current codebase
- [ ] Document current neuron configurations
- [ ] Inventory external dependencies
- [ ] Identify critical paths
- [ ] Plan rollback strategy

## Migration Patterns

### Pattern 1: Neuron Migration

#### Old Code (Flat Architecture)
```rust
pub struct Neuron {
    id: String,
    layer: String,
    claude_process: Option<Child>,
    rx: mpsc::Receiver<Signal>,
    tx: mpsc::Sender<Signal>,
}

impl Neuron {
    pub async fn process(&mut self, signal: Signal) -> Result<Signal> {
        // Direct processing
        let response = self.claude_process.send(signal)?;
        Ok(response)
    }
}
```

#### New Code (Hierarchical Architecture)
```rust
pub struct HierarchicalNeuron<L: Layer> {
    id: NeuronId,
    layer: L,
    substrate: Arc<dyn Substrate>,
    protocol: Box<dyn Protocol>,
    state: NeuronState,
}

#[async_trait]
impl<L: Layer> CognitiveUnit for HierarchicalNeuron<L> {
    type Input = Message;
    type Output = Message;
    type State = NeuronState;
    
    async fn process(&mut self, input: Self::Input) -> Result<Self::Output> {
        // Layered processing
        let decoded = self.protocol.decode(input).await?;
        let processed = self.layer.process(decoded).await?;
        self.protocol.encode(processed).await
    }
}
```

#### Migration Steps
1. Create wrapper for old neuron
2. Implement CognitiveUnit trait
3. Add substrate and protocol
4. Test compatibility
5. Replace old with new

### Pattern 2: Signal Flow Migration

#### Old Code
```rust
// Direct channel communication
let (tx, rx) = mpsc::channel(100);
neurons.insert("L4", tx.clone());
neurons.insert("L3", tx.clone());

// Send signal
tx.send(signal).await?;
```

#### New Code
```rust
// Router-based communication
let router = Router::new(substrate.transport());
router.register(l4_neuron.id(), Layer::L4).await?;
router.register(l3_neuron.id(), Layer::L3).await?;

// Route signal
router.route(signal).await?;
```

#### Migration Adapter
```rust
pub struct ChannelToRouterAdapter {
    old_channels: HashMap<String, mpsc::Sender<Signal>>,
    new_router: Arc<Router>,
}

impl ChannelToRouterAdapter {
    pub async fn send(&self, layer: &str, signal: Signal) -> Result<()> {
        // Try new router first
        if let Ok(()) = self.new_router.route(signal.clone()).await {
            return Ok(());
        }
        
        // Fallback to old channels
        self.old_channels.get(layer)
            .ok_or(Error::LayerNotFound)?
            .send(signal).await
            .map_err(Into::into)
    }
}
```

### Pattern 3: Configuration Migration

#### Old Configuration
```yaml
neurons:
  - id: "L4-strategic"
    layer: "L4"
    claude_model: "claude-3-opus"
    prompt_template: "strategic_template.txt"
  
  - id: "L3-design"
    layer: "L3"
    claude_model: "claude-3-sonnet"
    prompt_template: "design_template.txt"
```

#### New Configuration
```yaml
substrate:
  type: "local"
  runtime: "tokio"
  transport: "channels"
  storage: "memory"

neurons:
  - type: "strategic"
    id: "L5-strategic-001"
    layer: "L5"
    cognitive:
      model: "claude-3-opus"
      temperature: 0.7
      system_prompt: "strategic_visionary"
    
  - type: "tactical"
    id: "L4-tactical-001"
    layer: "L4"
    cognitive:
      model: "claude-3-sonnet"
      temperature: 0.5
      system_prompt: "tactical_planner"
```

#### Configuration Converter
```rust
pub fn convert_config(old: OldConfig) -> Result<NewConfig> {
    let mut new = NewConfig::default();
    
    // Set default substrate
    new.substrate = SubstrateConfig {
        type_: "local".into(),
        runtime: "tokio".into(),
        transport: "channels".into(),
        storage: "memory".into(),
    };
    
    // Convert neurons
    for old_neuron in old.neurons {
        let layer = parse_layer(&old_neuron.layer)?;
        let neuron_type = infer_type(&old_neuron)?;
        
        new.neurons.push(NeuronConfig {
            type_: neuron_type,
            id: generate_id(&old_neuron.id),
            layer,
            cognitive: CognitiveConfig {
                model: old_neuron.claude_model,
                temperature: 0.5, // Default
                system_prompt: load_prompt(&old_neuron.prompt_template)?,
            },
        });
    }
    
    Ok(new)
}
```

### Pattern 4: API Migration

#### Old API Endpoint
```rust
#[post("/signal")]
async fn send_signal(signal: web::Json<Signal>) -> Result<HttpResponse> {
    let neuron = neurons.get(&signal.target)?;
    neuron.send(signal.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}
```

#### New API Endpoint
```rust
#[post("/signal")]
async fn send_signal(
    signal: web::Json<SignalRequest>,
    gateway: web::Data<ApiGateway>,
) -> Result<HttpResponse> {
    let message = gateway.transform(signal.into_inner()).await?;
    gateway.route(message).await?;
    Ok(HttpResponse::Ok().json(&SignalResponse::success()))
}
```

#### Backward Compatibility Layer
```rust
pub struct ApiCompatibility {
    gateway: Arc<ApiGateway>,
}

impl ApiCompatibility {
    pub async fn handle_legacy(&self, req: LegacyRequest) -> Result<Response> {
        // Convert legacy format
        let modern = self.convert_request(req)?;
        
        // Process with new system
        let result = self.gateway.process(modern).await?;
        
        // Convert back to legacy format
        self.convert_response(result)
    }
}
```

## Step-by-Step Migration Process

### Phase 1: Parallel Run (Week 1-2)
1. Deploy new system alongside old
2. Duplicate traffic to both systems
3. Compare outputs
4. Log discrepancies
5. Fix issues

### Phase 2: Gradual Cutover (Week 3-4)
1. Route 10% traffic to new system
2. Monitor metrics
3. Increase to 50%
4. Full cutover when stable
5. Keep old system as fallback

### Phase 3: Cleanup (Week 5)
1. Remove old code paths
2. Delete compatibility layers
3. Update documentation
4. Archive old configs
5. Celebrate! 🎉

## Common Migration Issues

### Issue 1: Message Format Mismatch
```rust
// Problem
Old: Signal { data: String }
New: Message { payload: Value }

// Solution
impl From<Signal> for Message {
    fn from(signal: Signal) -> Self {
        Message {
            payload: serde_json::from_str(&signal.data)
                .unwrap_or_else(|_| Value::String(signal.data)),
        }
    }
}
```

### Issue 2: Async Compatibility
```rust
// Problem
Old: fn process(&mut self, signal: Signal) -> Signal
New: async fn process(&mut self, input: Message) -> Result<Message>

// Solution
pub fn sync_to_async<F>(f: F) -> impl Future<Output = Result<Message>>
where F: FnOnce() -> Signal
{
    async move {
        let signal = tokio::task::spawn_blocking(f).await?;
        Ok(Message::from(signal))
    }
}
```

### Issue 3: State Management
```rust
// Problem
Old: Mutable state in neuron
New: Immutable message passing

// Solution
pub struct StateBridge {
    old_state: Arc<Mutex<OldState>>,
    new_state: StateManager,
}

impl StateBridge {
    pub async fn sync(&self) -> Result<()> {
        let old = self.old_state.lock().await;
        self.new_state.update(StateUpdate::from(&*old)).await
    }
}
```

## Testing Migration

### Compatibility Tests
```rust
#[tokio::test]
async fn test_old_new_compatibility() {
    let old_neuron = create_old_neuron();
    let new_neuron = create_new_neuron();
    
    let test_signal = test_signal();
    
    let old_result = old_neuron.process(test_signal.clone());
    let new_result = new_neuron.process(Message::from(test_signal)).await?;
    
    assert_eq!(old_result, Signal::from(new_result));
}
```

### Performance Comparison
```rust
#[bench]
fn bench_old_vs_new(b: &mut Bencher) {
    let runtime = Runtime::new().unwrap();
    
    b.iter(|| {
        runtime.block_on(async {
            let old_time = time_old_process().await;
            let new_time = time_new_process().await;
            
            assert!(new_time <= old_time * 1.1); // Allow 10% overhead
        });
    });
}
```

## Rollback Plan

If migration fails:

1. **Immediate**: Switch traffic back to old system
2. **Investigate**: Analyze logs and metrics
3. **Fix**: Address issues in new system
4. **Retry**: Attempt migration again
5. **Document**: Update runbook with learnings

## Post-Migration

### Cleanup Checklist
- [ ] Remove feature flags
- [ ] Delete old code
- [ ] Update CI/CD pipelines
- [ ] Archive old documentation
- [ ] Update monitoring dashboards

### Success Metrics
- Zero downtime during migration
- Performance within 10% of old system
- All tests passing
- No customer complaints
- Team trained on new architecture

---

*"The best migration is the one users don't notice."*

**For developers navigating change.**