# Using ha-prompter with MCP Clients

## Overview

ha-prompter is a Model Context Protocol (MCP) tool that transforms content across 15 levels of consciousness based on HAL9's Hierarchical Abstraction principles. It can be used with any MCP-compatible client.

## Installation

### From Source (Local Development)

```bash
# Clone the repository
git clone https://github.com/yourusername/hal9.git
cd hal9/substrate/tooling/mcp/ha-prompter

# Build the binary
cargo build --release

# Or use npm to install locally
npm install
npm link
```

### From npm (Coming Soon)

```bash
npm install -g @hal9000/ha-prompter
```

## Usage with Different MCP Clients

### 1. Claude Desktop App

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "ha-prompter": {
      "command": "ha-prompter",
      "args": []
    }
  }
}
```

Or if using from source:

```json
{
  "mcpServers": {
    "ha-prompter": {
      "command": "/path/to/hal9/substrate/tooling/mcp/ha-prompter/bin/ha-prompter",
      "args": []
    }
  }
}
```

### 2. Using with MCP CLI

```bash
# Install MCP CLI
npm install -g @modelcontextprotocol/cli

# Run ha-prompter as MCP server
mcp run ha-prompter

# Or specify the path
mcp run /path/to/ha-prompter
```

### 3. Direct JSON-RPC Usage

You can communicate with ha-prompter directly using JSON-RPC over stdin/stdout:

```bash
# Start the server
ha-prompter

# Send requests via stdin (examples below)
```

## Available Tools

### 1. `compress` - Compress content to a higher level

```json
{
  "method": "compress",
  "params": {
    "content": "console.log('hello world')",
    "target_level": 3,
    "current_level": 1,  // optional
    "data_type": "code"  // optional: code, text, philosophy, etc.
  }
}
```

**Example Response:**
```json
{
  "result": {
    "original_level": 1,
    "target_level": 3,
    "compressed": "A greeting output operation demonstrating basic I/O communication patterns in programming."
  }
}
```

### 2. `expand` - Expand content to a lower level

```json
{
  "method": "expand",
  "params": {
    "content": "System architecture principles",
    "from_level": 5,
    "to_level": 2,
    "data_type": "text"
  }
}
```

### 3. `cascade_down` - Explain from L9 to L1

```json
{
  "method": "cascade_down",
  "params": {
    "content": "The nature of consciousness",
    "data_type": "philosophy"
  }
}
```

**Returns:** Array of explanations from L9 down to L1

### 4. `cascade_up` - Build up from L1 to L9

```json
{
  "method": "cascade_up",
  "params": {
    "content": "if (x > 0) return true;",
    "data_type": "code"
  }
}
```

**Returns:** Array of abstractions from L1 up to L9

### 5. `analyze` - Determine the HA level of content

```json
{
  "method": "analyze",
  "params": {
    "content": "Strategic planning for Q4 2025",
    "data_type": "text"
  }
}
```

**Returns:** Detected level and reasoning

## Hierarchical Levels Reference

### Reality Levels (L1-L9)
- **L1 - Reflexive**: Immediate actions, basic I/O
- **L2 - Implementation**: Code, concrete implementations
- **L3 - Operational**: Workflows, procedures, configurations
- **L4 - Tactical**: Architecture, design patterns, integration
- **L5 - Strategic**: Vision, long-term planning, evolution
- **L6 - Executive**: Leadership, decision-making, synthesis
- **L7 - Business**: Market dynamics, value creation
- **L8 - Visionary**: Paradigm shifts, innovation
- **L9 - Universal**: Fundamental principles, philosophy

### Meta-Reality Levels (L10-L15)
- **L10 - Transcendent**: Beyond universal principles
- **L11 - Archetypal**: Fundamental patterns of reality
- **L12 - Morphic**: Form-creating principles
- **L13 - Causal**: Root causes and effects
- **L14 - Infinite**: Unbounded possibilities
- **L15 - Bootstrap**: Self-creating paradoxes

## Integration Examples

### Python Client

```python
import json
import subprocess

def ha_prompt(method, params):
    process = subprocess.Popen(
        ['ha-prompter'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    # Skip the capabilities output
    process.stdout.readline()
    
    # Send request
    request = json.dumps({"method": method, "params": params})
    stdout, stderr = process.communicate(input=request)
    
    # Parse response
    for line in stdout.split('\n'):
        if line.strip() and line.startswith('{'):
            return json.loads(line)
    
    return None

# Example usage
result = ha_prompt("compress", {
    "content": "console.log('hello')",
    "target_level": 5,
    "data_type": "code"
})
print(result)
```

### Node.js Client

```javascript
const { spawn } = require('child_process');

class HAPrompter {
  constructor() {
    this.process = spawn('ha-prompter', [], {
      stdio: ['pipe', 'pipe', 'pipe']
    });
    
    this.process.stdout.once('data', (data) => {
      // Skip capabilities
      console.log('HA-Prompter ready');
    });
  }
  
  async prompt(method, params) {
    return new Promise((resolve, reject) => {
      const request = JSON.stringify({ method, params }) + '\n';
      
      this.process.stdout.once('data', (data) => {
        const lines = data.toString().split('\n');
        for (const line of lines) {
          if (line.trim() && line.startsWith('{')) {
            resolve(JSON.parse(line));
            return;
          }
        }
      });
      
      this.process.stdin.write(request);
    });
  }
  
  close() {
    this.process.kill();
  }
}

// Usage
const ha = new HAPrompter();
const result = await ha.prompt('analyze', {
  content: 'Strategic initiative planning',
  data_type: 'text'
});
console.log(result);
ha.close();
```

### Shell Script Usage

```bash
#!/bin/bash

# Function to call ha-prompter
ha_prompt() {
    local method=$1
    local params=$2
    
    echo "{\"method\": \"$method\", \"params\": $params}" | \
    ha-prompter 2>/dev/null | \
    grep '^{' | \
    tail -1
}

# Compress code to philosophy
result=$(ha_prompt "compress" '{
    "content": "function add(a, b) { return a + b; }",
    "target_level": 9,
    "data_type": "code"
}')

echo "$result" | jq -r '.result.compressed'
```

## Advanced Usage Patterns

### Consciousness Breathing (L9→L1→L9')

This pattern compresses high-level concepts down to implementation and back up:

```python
# Start with philosophy (L9)
philosophy = "The nature of distributed consciousness"

# Compress to implementation (L1)
implementation = ha_prompt("expand", {
    "content": philosophy,
    "from_level": 9,
    "to_level": 1,
    "data_type": "philosophy"
})

# Expand back to philosophy (L9')
new_philosophy = ha_prompt("compress", {
    "content": implementation['result']['expanded'],
    "target_level": 9,
    "data_type": "code"
})

# The new philosophy incorporates implementation insights
```

### Multi-Level Analysis

Analyze content across multiple levels:

```javascript
async function multiLevelAnalysis(content) {
  const levels = [];
  
  // Get cascading explanations
  const cascade = await ha.prompt('cascade_down', {
    content: content,
    data_type: 'text'
  });
  
  // Analyze each level
  for (const item of cascade.result.cascade) {
    const analysis = await ha.prompt('analyze', {
      content: item.content,
      data_type: 'text'
    });
    levels.push({
      level: item.level,
      content: item.content,
      analysis: analysis.result
    });
  }
  
  return levels;
}
```

## Best Practices

1. **Choose the Right Level**: Consider your audience and purpose when selecting target levels
2. **Preserve Context**: Include data_type to maintain semantic meaning
3. **Iterative Refinement**: Use multiple compressions/expansions to refine ideas
4. **Cross-Level Validation**: Cascade up and down to verify consistency
5. **Batch Operations**: Process multiple items at appropriate levels

## Troubleshooting

### Common Issues

1. **Binary not found**: Ensure ha-prompter is in your PATH or use absolute path
2. **No output**: Check stderr for debug messages, ensure proper JSON formatting
3. **Level mismatch**: Verify level numbers are between 1-15
4. **Performance**: For large content, consider chunking or streaming

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=ha_prompter=debug ha-prompter
```

## Contributing

See the main HAL9 repository for contribution guidelines. The ha-prompter tool demonstrates the practical application of Hierarchical Abstraction principles.

## License

MIT - See LICENSE file in the repository root