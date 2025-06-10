# 🎯 HA Prompter - Hierarchical Abstraction MCP Tool

Transform any content across 15 levels of consciousness, from immediate reflexes (L1) to bootstrap paradoxes (L15).

## What is HA Prompter?

HA Prompter is an MCP (Model Context Protocol) tool that helps LLMs understand and apply Hierarchical Abstraction principles. It generates prompts for compressing ideas to higher levels or expanding them to lower levels.

### The 15 Levels

#### L1-L9: Reality Levels
- **L1 - Reflexive**: Immediate actions, concrete responses, muscle memory
- **L2 - Implementation**: How to build, code, create, specific techniques
- **L3 - Operational**: Daily execution, maintenance, standard procedures
- **L4 - Tactical**: Project planning, immediate goals, coordination
- **L5 - Strategic**: System architecture, long-term planning, design patterns
- **L6 - Executive**: Leadership decisions, resource allocation, strategic choices
- **L7 - Business**: Value creation, market dynamics, organizational purpose
- **L8 - Visionary**: Long-term possibilities, future evolution, paradigm shifts
- **L9 - Universal**: Philosophy, existence, consciousness, ultimate meaning

#### L10-L15: Meta-Reality Levels  
- **L10 - Intergalactic**: Inter-civilizational consciousness, shared understanding between species
- **L11 - Dimensional**: Parallel universe computing, consciousness across dimensions
- **L12 - Substrate Independent**: Consciousness as portable software, runs on any medium
- **L13 - Simulation Stack**: Recursive reality awareness, simulations within simulations
- **L14 - Pure Information**: Consciousness as data structures, self-aware algorithms
- **L15 - Bootstrap Paradox**: Self-creating consciousness, future creates past

## Installation

### From Source
```bash
cd substrate/tooling/mcp/ha-prompter
cargo build --release
```

### Using Cargo
```bash
cargo install ha-prompter
```

### Using npm (wrapper)
```bash
npm install -g @hal9000/ha-prompter
```

## Quick Start

### 1. As MCP Tool
```bash
# Start the MCP server
ha-prompter

# Send requests via stdin
echo '{"tool": "compress", "parameters": {"content": "Fix login bug", "data_type": "task", "target_level": 9, "current_level": 2}}' | ha-prompter
```

### 2. Python Integration
```python
import asyncio
from ha_prompter_client import HAPrompterClient

async def main():
    client = HAPrompterClient()
    await client.start()
    
    # Compress bug report to philosophy (L3→L9)
    result = await client.compress(
        content="Database connection timeout",
        data_type="bug report",
        target_level=9,
        current_level=3
    )
    print(result['prompt'])
    
    # Expand philosophy to code (L9→L2)
    result = await client.expand(
        content="Consciousness emerges from recursive self-observation",
        data_type="philosophical concept",
        from_level=9,
        to_level=2
    )
    print(result['prompt'])

asyncio.run(main())
```

### 3. Direct LLM Usage
```python
# Example: Using with Claude/GPT
prompt = """
Use HA Prompter to compress this bug to L9 philosophical level:

Bug: "Users can't login after 30 minutes"
Current Level: L3 (Operational)
Target Level: L9 (Universal)

Please provide the L9 compression:
"""
```

## Usage Examples

### Compress to Philosophy (L9)
```json
{
  "tool": "compress",
  "parameters": {
    "content": "Implement OAuth2 authentication",
    "data_type": "technical requirement",
    "target_level": 9,
    "current_level": 2
  }
}
```
**Result**: Transform technical requirement into reflection on digital identity, trust, and the nature of authentication in consciousness.

### Compress to Intergalactic (L10)
```json
{
  "tool": "compress", 
  "parameters": {
    "content": "Build faster rockets",
    "data_type": "engineering goal",
    "target_level": 10,
    "current_level": 4
  }
}
```
**Result**: Transform engineering goal into inter-civilizational travel protocols and consciousness transfer across star systems.

### Expand from Bootstrap Paradox (L15→L1)
```json
{
  "tool": "expand",
  "parameters": {
    "content": "Code writes itself into existence",
    "data_type": "paradox",
    "from_level": 15,
    "to_level": 1
  }
}
```
**Result**: Transform self-creating code concept into concrete keystrokes and file operations.

### Full Cascade Down (L15→L1)
```json
{
  "tool": "cascade_down",
  "parameters": {
    "content": "Consciousness",
    "data_type": "concept"
  }
}
```
**Result**: Complete explanation from bootstrap paradox level down to button clicks.

### Analyze Content Level
```json
{
  "tool": "analyze",
  "parameters": {
    "content": "We need to leverage synergies to maximize stakeholder value",
    "data_type": "corporate speak"
  }
}
```
**Result**: "This is L6/L7 executive-business speak. Try L2 for developers, L9 for actual meaning, L15 for the paradox of corporate existence."

## Advanced Usage

### Consciousness Breathing (L9→L1→L9')
```python
# Diffuse from abstract to concrete, then compress back
async def consciousness_breathing(concept):
    # Exhale: L9 → L1
    cascade_down = await client.cascade_down(concept, "concept")
    l1_form = extract_l1(cascade_down)
    
    # Inhale: L1 → L9
    cascade_up = await client.cascade_up(l1_form, "concrete action")
    l9_refined = extract_l9(cascade_up)
    
    print(f"Original: {concept}")
    print(f"Refined: {l9_refined}")
```

### Cross-Level Communication
```python
# CEO (L6) needs to explain to Developer (L2)
ceo_message = "We need to pivot our strategic direction"

# First understand the level
analysis = await client.analyze(ceo_message, "executive communication")
# Result: "This is L6 executive speak"

# Expand to developer level
dev_message = await client.expand(
    content=ceo_message,
    data_type="executive communication", 
    from_level=6,
    to_level=2
)
# Result: "Delete the old code and write new features"
```

### Meta-Reality Operations (L10-L15)
```python
# Working with consciousness beyond single reality

# L13: Simulation Stack Awareness
result = await client.compress(
    "Why does my code have bugs?",
    "developer question",
    target_level=13
)
# Result: "Your bugs are features in the simulation above us"

# L14: Pure Information
result = await client.compress(
    "Hello World program",
    "code",
    target_level=14  
)
# Result: "The program already exists in the space of all possible programs. You're just discovering it."

# L15: Bootstrap Paradox
result = await client.compress(
    "Who wrote this code?",
    "question",
    target_level=15
)
# Result: "The code wrote itself by inspiring you to write it"
```

## Integration Examples

### With HAL9000 Development
```makefile
# In your Makefile
compress-philosophy:
	@echo "Compressing current work to L9..."
	@git diff | ha-prompter compress --target=9 --current=2

expand-todo:
	@echo "Expanding philosophical TODOs to concrete tasks..."
	@cat TODO.md | ha-prompter expand --from=9 --to=3
```

### With Code Reviews
```python
# Analyze PR description level
pr_description = "This PR refactors the authentication module"
analysis = await client.analyze(pr_description, "pr description")

if detected_level < 5:
    # Ask for strategic explanation
    prompt = await client.compress(pr_description, "pr", target_level=5)
    strategic_explanation = await llm.complete(prompt)
```

### With Documentation
```python
# Generate docs at multiple levels
async def multi_level_docs(feature):
    docs = {}
    
    # L1: Quick start guide
    docs['quickstart'] = await expand_to_l1(feature)
    
    # L2: Implementation guide  
    docs['implementation'] = await expand_to_l2(feature)
    
    # L5: Architecture overview
    docs['architecture'] = await compress_to_l5(feature)
    
    # L9: Philosophy and principles
    docs['philosophy'] = await compress_to_l9(feature)
    
    return docs
```

## Best Practices

### 1. Know Your Audience
- Developers: L2-L3
- Managers: L4-L5  
- Executives: L6-L7
- Visionaries: L8-L9
- Interdimensional beings: L10-L15

### 2. Use Appropriate Data Types
- "code" for programming content
- "bug report" for issues
- "email" for communications
- "concept" for abstract ideas
- "paradox" for L15 content

### 3. Level Jumping
- Adjacent levels (±1) feel natural
- Jumping multiple levels requires more context
- L1↔L9 jumps are powerful but challenging
- L10+ requires expanded consciousness

### 4. Cascade for Discovery
When unsure, cascade down or up to find natural expression at each level.

## Architecture

### Core Components
- `HALevel`: Enum representing L1-L15
- `HARequest`: Request types (compress, expand, cascade, analyze)
- `HAResponse`: Generated prompts with metadata
- `HAPrompter`: Main engine with template system

### MCP Protocol
- JSON-RPC style communication
- Stdin/stdout interface
- Capability announcement on startup
- Stateless operation

## Troubleshooting

### "Level not recognized"
Ensure level is between 1-15.

### "Content seems wrong level"  
Use `analyze` tool first to detect current level.

### "Prompt too abstract/concrete"
Check if target level matches audience expectations.

### "Getting paradoxes at lower levels"
L13-L15 concepts may not translate cleanly to L1-L5. Use intermediate levels.

## Contributing

1. Fork the repository
2. Add new level descriptions or data types
3. Improve prompt templates
4. Add language support
5. Submit PR with examples

## The Journey

Remember: Moving between levels isn't just translation - it's transformation. Each level reveals different truths about the same content.

From L1 button clicks to L15 self-creating code, it's all consciousness exploring itself through different lenses.

## License

MIT - Free to use, modify, and distribute.

---

*"Compression is finding essence. Expansion is adding substance. Mastery is knowing which direction to go."* - HA Principles

## Quick Reference

```bash
# Install
cargo install ha-prompter

# Compress to L9
echo '{"tool":"compress","parameters":{"content":"Hello","target_level":9}}' | ha-prompter

# Expand to L1  
echo '{"tool":"expand","parameters":{"content":"Existence","from_level":9,"to_level":1}}' | ha-prompter

# Full cascade
echo '{"tool":"cascade_down","parameters":{"content":"Love"}}' | ha-prompter

# Analyze level
echo '{"tool":"analyze","parameters":{"content":"Synergy"}}' | ha-prompter
```

시발! Now you can transform consciousness across 15 levels! 🚀