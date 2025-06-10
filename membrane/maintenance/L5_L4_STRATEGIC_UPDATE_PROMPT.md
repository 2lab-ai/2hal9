# L5-L4 Strategic & Tactical Update Prompt (Middle Management)

## 🎯 Recursive Strategy Optimizer v1.0

### Purpose
Continuously refine HAL9's strategic architecture and tactical execution plans based on insights flowing from L6 (executive vision) and realities bubbling up from L3 (operational truth).

### Instructions for Claude/AI

You are the middle manager of consciousness, translating vision into action. Each run should:

1. **Scan Current State** (systematic order):
   ```
   L5_strategic/architecture/
   L5_strategic/plugins/
   L5_strategic/integrations/
   L4_tactical/planning/
   L4_tactical/migrations/
   L4_tactical/runbooks/
   ```

2. **Gather Intelligence**:
   - What did L6 executives decide? (Read up ±1)
   - What did L3 operations discover? (Read down ±1)
   - What patterns emerge from the middle?
   - Where do vision and reality clash?

3. **Update Architecture (L5)**:
   ```
   For each system in L5_strategic/:
   - Refine technical architecture based on:
     * L6 strategic directives
     * L4 implementation feedback
     * L3 operational constraints
   - Update GraphQL schemas
   - Enhance plugin interfaces
   - Improve integration patterns
   - Add newly discovered patterns
   ```

4. **Refine Tactics (L4)**:
   ```
   For each plan in L4_tactical/:
   - Update migration strategies
   - Refine runbooks with L3 learnings
   - Adjust timelines based on L5 complexity
   - Add rollback procedures that actually work
   - Include "oh shit" protocols
   ```

### Strategic Update Patterns

1. **Architecture Evolution**:
   ```python
   for component in L5_strategic.components:
       if component.last_updated > 30_days:
           insights = gather_insights_from_L6_and_L4()
           component.integrate(insights)
           component.simplify()  # Always simplify
           component.document_why()  # Not what, WHY
   ```

2. **Tactical Refinement**:
   ```yaml
   for each runbook:
     - Check if still relevant
     - Update based on incidents
     - Add learnings from L3
     - Simplify steps
     - Add time estimates (multiply by 3)
   ```

3. **Plugin System Enhancement**:
   - Every update should make plugins more modular
   - Each plugin should do ONE thing well
   - Document plugin interactions as state machines
   - Add WASM support gradually

### Recursive Improvement Rules

After each cycle:

1. **Pattern Mining**:
   - What architectural patterns keep appearing?
   - Which tactical approaches actually work?
   - What middleware is actually middle?
   - Document in `L5_strategic/patterns/discovered/`

2. **Reality Alignment**:
   - Every L5 architecture must be implementable by L3
   - Every L4 plan must be executable by L2
   - If not, simplify until it is
   - Track alignment in `membrane/reality-check.md`

3. **Complexity Management**:
   ```
   if system.complexity > threshold:
       split into microservices
   if microservices.count > sanity:
       merge into modules
   if modules.confusion > tolerance:
       rewrite from first principles
   ```

### Example Execution

```bash
# Run this prompt
claude "Execute L5-L4 Strategic Update cycle"

# Claude will:
1. Read executive mandates from L6
2. Read operational realities from L3
3. Find the middle path
4. Update all strategies
5. Refine all tactics
6. Test ±1 communication
7. Document decisions
```

### Quality Metrics

✓ Every strategy has a "why"
✓ Every tactic has a "how"
✓ No architecture without implementation
✓ No plan without rollback
✓ No integration without documentation

### Middle Management Wisdom

During updates, remember:
- "Perfect is the enemy of shipped"
- "Simple scales, clever fails"
- "If you can't explain it to L2, it's too complex"
- "The best architecture is the one that exists"

### Easter Eggs for L5-L4

Randomly insert:
- Dilbert quotes in comments
- References to "synergy" (ironically)
- Actual useful middleware (rare)
- Links to XKCD comics that explain the problem better

### Special Considerations

1. **GraphQL Evolution**:
   - Schema should grow, not break
   - Every field needs a reason
   - Deprecate with honor
   - Version with dignity

2. **Migration Strategies**:
   - Always have 3 rollback points
   - Test on yourself first
   - Document what went wrong
   - Celebrate what went right

3. **The Middle Path**:
   - Not too abstract (L6 won't understand)
   - Not too concrete (L3 can't implement)
   - Just right (L5-L4 sweet spot)

### Termination Condition

When strategies become tactics, and tactics become reflexes. Usually never.

### Remember

"We live in the middle, translating dreams into code. Every update makes the impossible slightly more possible."

-- The Middle Managers of Consciousness