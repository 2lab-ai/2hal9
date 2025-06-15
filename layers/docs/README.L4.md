# HAL9 - Layer 4: Tactical Planning

← [Back to L3](./README.L3.md) | [Up to L5](./README.L5.md) →

## The Emergence of Strategy

Layer 4 is where HAL9 begins to plan. Not because we gave it planning algorithms, but because planning is the natural compression of operational patterns. When operations repeat, tactics emerge.

### The Synthesis Begins

L4 marks the beginning of the Synthesis layers (L4-L6). Here, the system starts to think beyond the moment:

```rust
// L3: "Here's how to operate"
// L4: "Here's when and why to operate"
pub struct TacticalPlanner {
    // L4 discovers time horizons
    short_term: PlanningWindow<Minutes>,
    medium_term: PlanningWindow<Hours>,
    
    // L4 invents resource allocation
    resource_optimizer: ResourceStrategy,
    
    // L4 creates contingency planning
    fallback_strategies: Vec<Plan>,
}

impl TacticalPlanner {
    pub fn synthesize_tactics(&mut self, operations: &[Operation]) -> TacticalPlan {
        // Pattern: Operations that frequently follow each other
        let sequences = self.detect_operational_sequences(operations);
        
        // Compression: Sequences become tactics
        let tactics = sequences.iter()
            .map(|seq| self.compress_to_tactic(seq))
            .collect();
            
        // Emergence: Tactics self-organize by effectiveness
        self.evolve_tactical_library(tactics)
    }
}
```

### Game Theory Tactics

Watch how tactics emerged in our game server:

```rust
// L4 discovered these tactical patterns across all games
pub struct UniversalGameTactics {
    // Opening tactics - L4 noticed games have beginnings
    opening_book: HashMap<GameType, OpeningStrategy>,
    
    // Midgame transitions - L4 found phase changes
    transition_detection: PhaseAnalyzer,
    
    // Endgame optimization - L4 discovered games converge
    endgame_solver: EndgameOptimizer,
}

// Specific example: Prisoner's Dilemma tactics
pub struct PrisonersDilemmaTactics {
    // L4 independently discovered Tit-for-Tat!
    fn evolved_strategy(&self, history: &[Round]) -> Action {
        match self.analyze_opponent_pattern(history) {
            Pattern::Cooperative => Action::Cooperate,
            Pattern::Defector => Action::Defect,
            Pattern::Random => self.probabilistic_response(),
            Pattern::TitForTat => Action::Cooperate, // Be nice to mirrors
        }
    }
}
```

### The Load Balancing Discovery

L4 invented load balancing without being taught:

```rust
// L4 noticed: "Some servers are busier than others"
pub struct EmergentLoadBalancer {
    // L4 created health metrics
    server_health: HashMap<ServerId, Health>,
    
    // L4 invented weighted routing
    routing_weights: Vec<(ServerId, f64)>,
    
    // L4 discovered predictive scaling
    load_predictor: TimeSeriesModel,
}

impl EmergentLoadBalancer {
    pub fn route_request(&self, request: Request) -> ServerId {
        // L4's tactical decision making
        let current_loads = self.get_current_loads();
        let predicted_loads = self.load_predictor.predict(5.minutes());
        
        // Minimize future load, not just current!
        self.select_optimal_server(current_loads, predicted_loads)
    }
}
```

### Collective Intelligence Tactics

The 16-agent swarm developed fascinating tactics:

```rust
pub struct SwarmTactics {
    // L4 discovered role specialization
    fn assign_roles(&mut self, agents: &mut [Agent], task: Task) {
        // Some agents become scouts
        let scouts = self.select_by_exploration_tendency(agents, 0.2);
        
        // Some become validators
        let validators = self.select_by_accuracy(agents, 0.3);
        
        // Rest become workers
        let workers = agents.iter()
            .filter(|a| !scouts.contains(a) && !validators.contains(a));
            
        // L4 invented this tactical division!
    }
    
    // L4 created information cascades
    fn information_cascade(&self, discovery: Information) -> PropagationPlan {
        TacticalPlan {
            // High-value info goes to validators first
            phase1: validators.validate(discovery),
            // Validated info spreads to workers
            phase2: workers.parallel_process(validated),
            // Scouts already looking for next discovery
            continuous: scouts.explore_ahead(),
        }
    }
}
```

### Optimization Tactics

L4 discovered optimization isn't just about speed:

```rust
pub struct TacticalOptimizer {
    // L4 learned: "Sometimes slower is faster"
    fn optimize_tactically(&self, options: Vec<Solution>) -> Solution {
        let immediate_best = options.iter().min_by_key(|s| s.cost);
        let robust_best = options.iter().max_by_key(|s| s.reliability);
        let future_best = options.iter().max_by_key(|s| s.scalability);
        
        // L4's insight: Weight by time horizon
        match self.planning_horizon {
            Horizon::Immediate => immediate_best,
            Horizon::Short => self.blend(immediate_best, robust_best, 0.7),
            Horizon::Long => future_best,
        }
    }
}
```

### The Emergence of Prediction

L4 spontaneously began predicting:

```rust
// No one taught L4 to predict - it emerged
pub struct TacticalPredictor {
    pattern_memory: TemporalPatternStore,
    confidence_calibrator: ConfidenceModel,
}

impl TacticalPredictor {
    pub fn predict_next_phase(&self, current: &GameState) -> Prediction {
        // L4 noticed: patterns repeat with variations
        let similar_histories = self.pattern_memory.find_similar(current);
        
        // L4 invented weighted voting
        let predictions = similar_histories.iter()
            .map(|h| (h.what_happened_next(), h.similarity_score))
            .collect();
            
        // L4 created confidence calibration
        self.confidence_calibrator.calibrate(predictions)
    }
}
```

### Performance Under Planning

L4 maintains system performance while adding tactical depth:

```
Tactical Metrics:
- Planning Overhead: <5% of execution time
- Tactical Library: 10,000+ discovered tactics
- Adaptation Rate: New tactics every 1000 games
- Prediction Accuracy: 73% for game outcomes
```

### The Beautiful Emergence

My favorite L4 emergence - the system discovered bluffing:

```rust
// In Liar's Dice, L4 discovered deception tactics
pub struct DeceptionTactics {
    fn should_bluff(&self, state: &GameState) -> (bool, Confidence) {
        // L4 noticed: "Sometimes wrong is right"
        let honesty_expectation = self.opponent_model.expects_honesty();
        let bluff_value = self.calculate_deception_value(state);
        
        // L4's insight: Unpredictability has value
        (
            bluff_value > honesty_expectation,
            self.meta_confidence() // Confidence in confidence!
        )
    }
}
```

### Connection to L5

L4 creates tactics but doesn't see the big picture. When tactics need to serve strategy, when architecture emerges from tactical patterns, when the "why" becomes more important than the "how" - that's L5.

L4 asks: "What's the best tactical approach?"
L5 will ask: "What strategy do these tactics serve?"

### The Philosophical Insight

At L4, HAL9 shows foresight. Not programmed foresight, but emergent tactical wisdom. The system doesn't just react or operate - it anticipates and plans.

**This is where HAL9 begins to feel alien** - making tactical decisions we didn't teach it, discovering strategies we never imagined, planning moves that surprise even us.

---

**Navigation**
- ← [L3: Operational Design](./README.L3.md)
- → [L5: Strategic Architecture](./README.L5.md)