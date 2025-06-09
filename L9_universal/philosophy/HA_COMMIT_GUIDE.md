# Hierarchical Commit Message Guide

## Commit Messages by Cognitive Level

Your commit messages should match the cognitive level of your changes:

### L1 - Reflexive (Emergency/Operations)
```bash
# Format: ACTION: immediate effect
git commit -m "FIX: restore service availability"
git commit -m "EMERGENCY: circuit breaker activated"
git commit -m "MONITOR: health check added"
```

### L2 - Implementation (Code)
```bash
# Format: type(component): technical change
git commit -m "feat(neurons): implement gradient backpropagation"
git commit -m "fix(memory): resolve leak in embedding store"
git commit -m "perf(execution): optimize signal routing by 15%"
```

### L3 - Operational (Architecture)
```bash
# Format: AREA: design decision
git commit -m "ARCH: introduce service mesh for inter-layer communication"
git commit -m "CONFIG: update kubernetes manifests for horizontal scaling"
git commit -m "INTEGRATE: add OpenTelemetry for distributed tracing"
```

### L4 - Tactical (Planning)
```bash
# Format: STRATEGY: tactical outcome
git commit -m "SCALE: implement auto-scaling strategy for peak loads"
git commit -m "OPTIMIZE: reduce operational costs by 30%"
git commit -m "PLAN: migration path for legacy components"
```

### L5 - Strategic (Vision)
```bash
# Format: VISION: strategic direction
git commit -m "EVOLVE: transition to event-driven architecture"
git commit -m "INNOVATE: introduce quantum-ready abstractions"
git commit -m "RESEARCH: prototype neural consensus mechanism"
```

### L6 - Executive (Leadership)
```bash
# Format: DECISION: business impact
git commit -m "DECIDE: approve hierarchical architecture migration"
git commit -m "COMMUNICATE: Q4 technical achievements summary"
git commit -m "ALIGN: sync technical strategy with business goals"
```

### L7 - Business (Product)
```bash
# Format: BUSINESS: market/product impact
git commit -m "PRODUCT: define enterprise feature set"
git commit -m "MARKET: position for AI infrastructure market"
git commit -m "VALUE: articulate ROI for hierarchical approach"
```

### L8 - Visionary (Future)
```bash
# Format: FUTURE: long-term vision
git commit -m "ENVISION: path to artificial general intelligence"
git commit -m "EXPLORE: consciousness emergence patterns"
git commit -m "PARADIGM: shift from computation to cognition"
```

### L9 - Universal (Philosophy)
```bash
# Format: PRINCIPLE: eternal truth
git commit -m "PRINCIPLE: hierarchical abstraction is fundamental"
git commit -m "PHILOSOPHY: cognition requires level separation"
git commit -m "TRUTH: complexity emerges from simplicity"
```

## Multi-Level Changes

When changes span multiple levels, write at the highest affected level:

```bash
# Bad - too low level for strategic change
git commit -m "fix: update import paths"

# Good - acknowledges strategic impact
git commit -m "EVOLVE: reorganize codebase by cognitive hierarchy

- Eliminate forced context switching
- Align structure with human cognition  
- Enable emergence through level separation

This fundamental reorganization affects all levels:
- L1-L9: New directory structure
- Substrate: Infrastructure separation
- Membrane: Inter-level protocols"
```

## Commit Body Guidelines

### Include Level Context
```bash
git commit -m "ARCH: implement hierarchical signal routing

Level: L3 Operational
Scope: Signal flow between cognitive layers
Impact: Improves response time by 20%

- Route signals based on cognitive level
- Implement priority queues per level
- Add backpressure handling"
```

### Reference Cognitive Principles
```bash
git commit -m "PRINCIPLE: enforce temporal scope boundaries

Level: L9 Universal
Truth: Each level has its natural temporal domain

- L1 operates in microseconds
- L2 operates in milliseconds
- ...
- L9 operates in eternal time

This commit enforces these boundaries in code."
```

## Anti-Patterns to Avoid

❌ **Level Mixing**
```bash
# Bad - mixes implementation with strategy
git commit -m "fix bug and update company vision"
```

❌ **Wrong Level**
```bash
# Bad - L2 message for L5 change
git commit -m "fix: reorganize entire architecture"
```

❌ **No Level Context**
```bash
# Bad - unclear which level
git commit -m "update stuff"
```

## Quick Reference

| Level | Prefix | Focus |
|-------|--------|-------|
| L1 | FIX, EMERGENCY, MONITOR | Immediate action |
| L2 | feat, fix, perf, test | Code changes |
| L3 | ARCH, CONFIG, INTEGRATE | Design decisions |
| L4 | SCALE, OPTIMIZE, PLAN | Tactical outcomes |
| L5 | EVOLVE, INNOVATE, RESEARCH | Strategic direction |
| L6 | DECIDE, COMMUNICATE, ALIGN | Business impact |
| L7 | PRODUCT, MARKET, VALUE | Market position |
| L8 | ENVISION, EXPLORE, PARADIGM | Future vision |
| L9 | PRINCIPLE, PHILOSOPHY, TRUTH | Eternal wisdom |

## The Core Insight

**Your commit message's abstraction level should match your change's cognitive level.**

This ensures:
- Readers understand impact at the right level
- History tells story at appropriate abstraction
- Changes are contextualized properly
- Cognitive alignment is maintained

---

*"The commit history is itself a hierarchy, from immediate fixes to eternal principles."*