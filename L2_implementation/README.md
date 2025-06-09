# L2 - Implementation Layer

**Cognitive Level**: L2_implementation  
**Temporal Scope**: Milliseconds to seconds  
**Purpose**: Code execution and neuron implementations

## Overview

This level contains all executable code, implementations, and validation. Developers work entirely within this cognitive space without needing to understand higher-level strategy or lower-level infrastructure.

## Structure

- `neurons/` - Core neuron implementations, signal processing
- `execution/` - Execution engines, runtime implementations
- `codegen/` - Code generation, compilation, optimization
- `validation/` - Tests, benchmarks, quality assurance

## For Developers

This is your workspace. Everything needed for implementation is here:

### Core Components
```
neurons/
├── core/           # From hal9-core/src
├── reflexive/      # L1 neuron implementations  
├── operational/    # L3 neuron implementations
├── tactical/       # L4 neuron implementations
└── strategic/      # L5 neuron implementations
```

### Running Tests
```bash
cd validation/tests
cargo test
```

### Benchmarking
```bash
cd validation/benchmarks
cargo bench
```

## Navigation

- **Down** → [Substrate](../substrate/) for infrastructure details
- **Up** → [L3 Operational](../L3_operational/) for architecture decisions
- **Lateral** → All implementation concerns at the same level

## Principles

1. Focus on HOW, not WHY
2. Implementation details stay here
3. No architectural decisions (those go to L3)
4. No strategic planning (that goes to L4+)

## What Belongs Here

✅ DO include:
- Source code files
- Unit tests
- Implementation documentation
- Code generation tools
- Validation suites
- Performance benchmarks

❌ DON'T include:
- Deployment configs (→ L3)
- Architecture diagrams (→ L3)
- Strategic plans (→ L4+)
- Business requirements (→ L7)

## Development Workflow

1. Write code in `neurons/` or `execution/`
2. Generate code using tools in `codegen/`
3. Validate with tests in `validation/`
4. Never leave this level during implementation