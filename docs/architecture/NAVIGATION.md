# HAL9 Hierarchical Navigation

Welcome to the cognitively-organized HAL9 codebase. Choose your cognitive level:

## 🚀 Quick Access by Role

### Operators & SREs → [L1 Reflexive](L1_reflexive/)
- System health, monitoring, emergency procedures
- Immediate responses, no deep thinking required
- **Temporal scope**: Microseconds to seconds

### Developers → [L2 Implementation](L2_implementation/)
- Code, neurons, execution engines
- Implementation details and validation
- **Temporal scope**: Milliseconds to seconds

### Architects → [L3 Operational](L3_operational/)
- System design, configuration, workflows
- Architecture and optimization
- **Temporal scope**: Seconds to minutes

### Tech Leads → [L4 Tactical](L4_tactical/)
- Planning, analysis, strategies
- Performance and adaptation
- **Temporal scope**: Minutes to hours

### CTOs & Principal Engineers → [L5 Strategic](L5_strategic/)
- Technical vision, innovation, research
- Long-term technical evolution
- **Temporal scope**: Hours to days

### Executives → [L6 Executive](L6_executive/)
- Overviews, decisions, metrics
- Leadership communication
- **Temporal scope**: Days to weeks

### Product & Business → [L7 Business](L7_business/)
- Product strategy, market analysis
- Business value and growth
- **Temporal scope**: Weeks to months

### Visionaries → [L8 Visionary](L8_visionary/)
- Future scenarios, paradigm shifts
- Moonshot projects
- **Temporal scope**: Months to years

### Philosophers → [L9 Universal](L9_universal/)
- Core principles, universal patterns
- Timeless wisdom
- **Temporal scope**: Eternal

## 🌐 Infrastructure Layers

### [Substrate](substrate/)
Supporting infrastructure for all cognitive levels
- Compute, storage, network, security
- Development tooling and utilities

### [Membrane](membrane/)
Inter-level communication and protocols
- How levels talk to each other
- Information flow management

### [.substrate](.substrate/) (Hidden)
Technical necessities that don't fit the hierarchy
- Build artifacts, logs, caches
- Things that must exist but aren't conceptually important

## 📖 Navigation Principles

### 1. **Stay at Your Level**
Everything you need for your current cognitive task is in one directory. No need to jump between abstraction levels.

### 2. **Natural Movement**
- **Up** ↑ For more context and purpose
- **Down** ↓ For more detail and implementation
- **Lateral** ↔ For related concepts at same abstraction

### 3. **No Forced Switching**
You should never need to think at multiple abstraction levels simultaneously. The structure prevents this cognitive overhead.

### 4. **Self-Contained Levels**
Each level is complete for its purpose. You can work entirely within one level without missing critical information.

## 🎯 Finding What You Need

### "Where is the code?"
→ [L2 Implementation](L2_implementation/) for all executable code

### "How do I deploy?"
→ [L3 Operational](L3_operational/workflows/) for deployment procedures

### "What's the system status?"
→ [L1 Reflexive](L1_reflexive/status/) for immediate status

### "How do we scale?"
→ [L4 Tactical](L4_tactical/strategies/) for scaling strategies

### "What's our vision?"
→ [L5 Strategic](L5_strategic/vision/) for technical vision

### "Brief me on progress"
→ [L6 Executive](L6_executive/overview/) for executive summaries

### "What's the product roadmap?"
→ [L7 Business](L7_business/product/) for product strategy

### "What could this become?"
→ [L8 Visionary](L8_visionary/future/) for long-term possibilities

### "Why do we exist?"
→ [L9 Universal](L9_universal/principles/) for core philosophy

## 🔄 Migration from Traditional Structure

If you're looking for traditional directories:
- `src/` → [L2 Implementation](L2_implementation/)
- `docs/` → Distributed across all levels at appropriate abstractions
- `tests/` → [L2 Implementation/validation](L2_implementation/validation/)
- `scripts/` → Distributed by purpose (L1 for ops, L4 for planning)
- `config/` → [L3 Operational/configuration](L3_operational/configuration/)

## 🧭 Start Here

1. **Identify your role** and cognitive level
2. **Navigate to your level** using links above
3. **Stay within that level** for all related work
4. **Move up or down** only when changing cognitive context

Remember: The goal is to eliminate cognitive switching. Let the structure guide you to work at the appropriate level of abstraction.