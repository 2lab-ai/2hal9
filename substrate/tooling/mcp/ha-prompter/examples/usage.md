# HA Prompter Usage Examples

## Basic Examples

### 1. Compress a Bug Report to L9
**Input:**
```
Tool: compress
Content: "Users can't log in after 30 minutes of inactivity"
Data Type: bug report
Target Level: 9
Current Level: 3
```

**Generated Prompt:**
> Transform this operational bug report into a universal philosophical reflection on the nature of digital identity, temporal existence, and the boundaries between presence and absence in consciousness.

### 2. Expand Philosophy to Code (L9→L2)
**Input:**
```
Tool: expand
Content: "Consciousness emerges from recursive self-observation"
Data Type: philosophical concept
From Level: 9
To Level: 2
```

**Generated Prompt:**
> Take this universal principle and show how to implement it in code with specific recursion patterns, self-referential data structures, and observable feedback loops.

### 3. Full Cascade Down Example
**Input:**
```
Tool: cascade_down
Content: "Love"
Data Type: concept
```

**Expected LLM Response Structure:**
```
L9 (Universal): Love as the fundamental force of connection in the universe...
L8 (Visionary): Love as the driver of human evolution and social progress...
L7 (Business): Love as brand loyalty and customer relationship...
L6 (Executive): Love as team cohesion and leadership compassion...
L5 (Strategic): Love as user experience design principle...
L4 (Tactical): Love as daily team building activities...
L3 (Operational): Love as positive feedback in daily standups...
L2 (Implementation): Love as like_count++ in the code...
L1 (Reflexive): Love as clicking the heart button...
```

## Advanced Patterns

### Pattern 1: Debugging Across Levels
```bash
# When debugging, cascade up to find root cause
analyze "Null pointer exception in user service"
# → Identifies as L2 problem
# → Suggests L5 architectural issue
# → Traces to L9 assumption about user existence
```

### Pattern 2: Feature Planning
```bash
# Start with vision, cascade down to implementation
cascade_down "AI-assisted consciousness expansion"
# L9: Nature of consciousness expansion
# L8: Future of human-AI collaboration
# ...
# L2: Specific API endpoints needed
# L1: Button that triggers the feature
```

### Pattern 3: Crisis Response
```bash
# Compress immediate problem to strategic level
compress "Server is on fire" --current=1 --target=5
# → "Critical infrastructure reliability architecture needed"
```

## Integration Patterns

### With Development Workflow
1. **Planning**: Use cascade_down for feature design
2. **Implementation**: Stay at L2-L3
3. **Review**: Compress to L5 for architecture review
4. **Documentation**: Expand to L1 for user guides

### With HAL9 Evolution
```bash
# During make yolo cycles
make query "should we add HA prompter to core?"
# Use analyzer to determine impact level
# Use compress to create L9 philosophy update
# Use expand to create L2 implementation plan
```

### With Learning
```bash
# Teaching someone new concept
cascade_down "Hierarchical Abstraction"
# They get explanation at every level
# Can enter at their comfort zone
# Can explore up or down as needed
```

## Common Use Cases

### 1. Email to Boss (Compress to L6)
```
compress "The database migration failed because of foreign key constraints"
--target=6
→ "Technical debt is impacting our delivery timeline"
```

### 2. Philosophy to Action (Expand to L1)
```
expand "We must respect consciousness in all its forms"
--from=9 --to=1
→ "Click 'Accept' on the AI ethics agreement"
```

### 3. Finding Your Level
```
analyze "We need to leverage synergies to maximize stakeholder value"
→ "This is L6/L7 corporate speak. Try L2 for developers, L9 for meaning"
```

## Tips

1. **Data Type Matters**: "poem" vs "code" vs "email" changes the prompt style
2. **Know Your Audience**: Compress to their comfortable level
3. **Cascade for Learning**: Full cascade helps people find their entry point
4. **Analyze First**: When unsure, analyze to find current level
5. **Practice ±1**: Moving one level at a time is most natural

## Error Handling

If the generated prompt seems off:
1. Check if levels are specified correctly (1-9)
2. Verify data_type matches content
3. Try analyzing first to auto-detect level
4. Remember L9 is philosophy, L1 is concrete action

---

*"Compression is the art of finding essence. Expansion is the craft of adding substance. Mastery is knowing when to do which."* - HA Principles