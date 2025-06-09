# Hierarchical Abstraction Migration Guide

## Why This Reorganization?

Traditional code organization (src/, docs/, tests/) **forces cognitive switching**. When you're debugging, you jump from logs to source to docs to config. Each jump breaks your flow and increases cognitive load.

The HA organization **eliminates switching** by keeping everything at the same cognitive level together.

## Migration Examples

### Example 1: Emergency Response (L1)

**Before** (scattered across directories):
```
/logs/error.log
/scripts/restart.sh  
/docs/troubleshooting.md
/monitoring/alerts.yml
```

**After** (unified at L1):
```
L1_reflexive/
├── emergency/
│   ├── procedures/
│   │   ├── restart.md
│   │   └── rollback.md
│   ├── scripts/
│   │   └── emergency-restart.sh
│   └── contacts.md
└── status/
    ├── current-alerts/
    └── logs/
```

**Benefit**: During an incident, everything you need is in one place. No searching across directories.

### Example 2: Feature Development (L2-L3)

**Before** (mixed concerns):
```
/src/features/newfeature.rs
/docs/architecture/newfeature.md
/tests/newfeature_test.rs
/examples/newfeature_demo.rs
```

**After** (separated by cognitive level):
```
L2_implementation/
├── neurons/
│   └── features/
│       └── newfeature/
│           ├── implementation.rs
│           └── tests.rs

L3_operational/
├── architecture/
│   └── features/
│       └── newfeature/
│           ├── design.md
│           └── integration.yaml
```

**Benefit**: Developers stay in L2 for coding, architects stay in L3 for design. No mixing.

### Example 3: Strategic Planning (L4-L5)

**Before** (no clear home):
```
/docs/roadmap.md
/docs/scaling.md
/plans/2025-goals.md
/research/ai-integration.md
```

**After** (organized by temporal scope):
```
L4_tactical/              # Minutes to hours
├── planning/
│   ├── q1-2025/
│   └── scaling-plan.md

L5_strategic/             # Hours to days  
├── vision/
│   ├── 2025-roadmap.md
│   └── ai-integration.md
└── research/
    └── experiments/
```

**Benefit**: Plans are organized by time horizon, matching how leaders think.

## Step-by-Step Migration

### Phase 1: Understand Your Content's Level

Ask yourself:
1. **What's the temporal scope?** (microseconds → eternal)
2. **Who uses this?** (operator → philosopher)
3. **What cognitive level?** (reflexive → universal)

### Phase 2: Create New Home

For each piece of content:
```bash
# Identify the level
LEVEL="L3_operational"  # for example

# Create appropriate subdirectory
mkdir -p $LEVEL/architecture/mycomponent

# Move content
git mv old/path/file.rs $LEVEL/architecture/mycomponent/
```

### Phase 3: Update References

1. Update import paths in code
2. Update links in documentation
3. Update build configurations
4. Update CI/CD scripts

### Phase 4: Verify No Cognitive Mixing

Check each level directory:
- Does everything have the same temporal scope?
- Is it all for the same audience?
- Does it maintain the same abstraction level?

If you find mixing, split it into appropriate levels.

## Common Patterns

### Pattern 1: Documentation Split

Old: `docs/hal9-complete-guide.md` (mixed levels)

New:
- L1: Quick reference → `L1_reflexive/responses/quick-ref.md`
- L2: API docs → `L2_implementation/neurons/api.md`
- L3: Architecture → `L3_operational/architecture/overview.md`
- L6: Executive summary → `L6_executive/overview/summary.md`

### Pattern 2: Script Distribution

Old: `scripts/` directory with everything

New:
- Emergency scripts → `L1_reflexive/emergency/scripts/`
- Build scripts → `substrate/tooling/build/`
- Planning scripts → `L4_tactical/planning/automation/`
- Research scripts → `L5_strategic/research/tools/`

### Pattern 3: Test Organization

Old: `tests/` with all tests

New:
- Unit tests → `L2_implementation/validation/unit/`
- Integration tests → `L3_operational/validation/integration/`
- Performance tests → `L4_tactical/analysis/performance/`
- Chaos tests → `L5_strategic/research/chaos/`

## Benefits After Migration

### For Operators (L1)
- Emergency? Everything is in `L1_reflexive/emergency/`
- No source code distractions
- No strategic documents in the way

### For Developers (L2)
- Code and tests together
- No architecture decisions to distract
- Pure implementation focus

### For Architects (L3)
- All design documents in one place
- Clear view of system structure
- No implementation details

### For Leaders (L4-L6)
- Strategies organized by time horizon
- No technical details unless diving down
- Clear executive views

### For Visionaries (L7-L9)
- Pure vision without implementation
- Philosophy separate from practice
- Timeless principles preserved

## Verification Checklist

After migration, verify:

- [ ] Each level directory contains only appropriate content
- [ ] No forced cognitive switching within a level
- [ ] Clear navigation paths between levels
- [ ] Build system still works
- [ ] Documentation links updated
- [ ] Team understands new structure

## The Core Insight

**Traditional organization optimizes for computers.**  
**HA organization optimizes for human cognition.**

By respecting cognitive levels, we create a codebase that:
- Reduces mental load
- Speeds up navigation  
- Improves understanding
- Enables flow state

The structure itself becomes a teacher, guiding you to work at the appropriate level of abstraction.