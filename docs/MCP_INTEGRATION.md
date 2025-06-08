# Model Context Protocol (MCP) Integration for 2HAL9

## Overview

The 2HAL9 system uses the Model Context Protocol (MCP) to provide standardized communication between the wrapper server (orchestrator) and individual neurons. This design enables:

- **Standardized Communication**: All neurons communicate via the same protocol
- **Dynamic Discovery**: Wrapper can discover neuron capabilities at runtime
- **Tool-based Processing**: Neurons expose their functions as MCP tools
- **Resource Sharing**: Neurons can share state and context via resources
- **Extensibility**: Easy to add new neuron types without changing the protocol

## Architecture

```
┌─────────────────────┐
│   Wrapper/Orch      │
│  (MCP Client)       │
└──────┬──────────────┘
       │ MCP Protocol
       │ (JSON-RPC)
┌──────┴──────────────┐
│   Neuron Server     │
│  (MCP Server)       │
│                     │
│ - Tools             │
│ - Resources         │
│ - Processing Logic  │
└─────────────────────┘
```

## Protocol Flow

### 1. Connection Initialization

```json
// Client → Server
{
  "jsonrpc": "2.0",
  "id": "1",
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "clientInfo": {
      "name": "2HAL9 Wrapper",
      "version": "1.0.0"
    }
  }
}

// Server → Client
{
  "jsonrpc": "2.0",
  "id": "1",
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "id": "neuron-1",
      "name": "L4 Strategic Neuron",
      "layer": "L4",
      "tools": ["process_task", "get_status"],
      "resources": ["neuron://l4/capabilities"],
      "supports_batch": true,
      "max_batch_size": 10
    }
  }
}
```

### 2. Task Processing

```json
// Client → Server
{
  "jsonrpc": "2.0",
  "id": "2",
  "method": "neuron/processTask",
  "params": {
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "content": "Build a simple TODO API service",
    "context": {
      "layer_from": "Input",
      "layer_to": "L4",
      "batch_id": "660e8400-e29b-41d4-a716-446655440001"
    }
  }
}

// Server → Client
{
  "jsonrpc": "2.0",
  "id": "2",
  "result": {
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "subtasks": [
      {
        "id": "770e8400-e29b-41d4-a716-446655440002",
        "content": "Design backend architecture and data model for TODO service",
        "target_neuron": "neuron-2",
        "target_layer": "L3"
      },
      {
        "id": "880e8400-e29b-41d4-a716-446655440003",
        "content": "Design API endpoints and request/response schemas",
        "target_neuron": "neuron-3",
        "target_layer": "L3"
      }
    ],
    "status": "success",
    "processing_time_ms": 150
  }
}
```

## Task Decomposition Pattern

The MCP integration supports the hierarchical task decomposition:

```
User Input (1 task)
    ↓
L4 Strategic (1 task → 2 subtasks)
    ↓
L3 Design (2 tasks → 4 subtasks total)
    ↓
L2 Implementation (4 tasks)
```

### Example Flow

1. **User Input**: "Build a simple TODO API service"

2. **L4 Processing** (1 → 2):
   - Subtask 1: "Design backend architecture and data model for TODO service"
   - Subtask 2: "Design API endpoints and request/response schemas"

3. **L3 Processing** (2 → 4):
   - From Subtask 1:
     - "Implement database schema with id, title, description, completed, created_at fields"
     - "Implement repository pattern with create, read, update, delete methods"
   - From Subtask 2:
     - "Implement REST endpoints: POST /todos, GET /todos, PUT /todos/:id, DELETE /todos/:id"
     - "Implement validation middleware and error handling for API requests"

4. **L2 Processing**: Generate actual implementation code

## Available Tools

### 1. process_task
Process a task at the neuron's layer level.

```json
{
  "name": "process_task",
  "description": "Process a task at L4 strategic level",
  "inputSchema": {
    "type": "object",
    "properties": {
      "task": {
        "type": "string",
        "description": "The task content to process"
      },
      "context": {
        "type": "object",
        "description": "Additional context for processing"
      }
    },
    "required": ["task"]
  }
}
```

### 2. get_status
Get current neuron status and metrics.

```json
{
  "name": "get_status",
  "description": "Get current neuron status and metrics",
  "inputSchema": {
    "type": "object",
    "properties": {}
  }
}
```

## Resources

Neurons can expose resources for state and capability information:

- `neuron://l4/capabilities` - L4 neuron capabilities
- `neuron://l3/capabilities` - L3 neuron capabilities
- `neuron://l2/capabilities` - L2 neuron capabilities
- `neuron://status` - Current neuron status
- `neuron://metrics` - Performance metrics

## Implementation Guide

### Creating a New Neuron with MCP

```rust
use twohal9_core::mcp::{NeuronMCPServer, ProcessTaskTool, StatusTool};

// Create MCP server for neuron
let mut mcp_server = NeuronMCPServer::new("neuron-1".to_string(), "L4".to_string());

// Register tools
mcp_server.register_tool(Box::new(ProcessTaskTool::new("L4".to_string())));
mcp_server.register_tool(Box::new(StatusTool::new("neuron-1".to_string())));

// Register resources
mcp_server.register_resource(Resource {
    uri: "neuron://l4/capabilities".to_string(),
    name: "L4 Capabilities".to_string(),
    mime_type: "application/json".to_string(),
    description: Some("Strategic layer capabilities".to_string()),
});
```

### Connecting from Wrapper

```rust
use twohal9_core::mcp::{WrapperMCPClient, TaskContext};

// Create client
let mut client = WrapperMCPClient::new();

// Connect to neuron
client.connect("neuron-1").await?;

// Process a task
let response = client.process_task(
    "Build a TODO API".to_string(),
    TaskContext {
        layer_from: "Input".to_string(),
        layer_to: "L4".to_string(),
        batch_id: Uuid::new_v4(),
        metadata: HashMap::new(),
    }
).await?;

// Handle subtasks
for subtask in response.subtasks {
    println!("Subtask: {} -> {}", subtask.content, subtask.target_neuron);
}
```

## Benefits

1. **Standardization**: All neurons use the same protocol, making integration consistent
2. **Discoverability**: Clients can discover neuron capabilities dynamically
3. **Flexibility**: Easy to add new tools and resources without changing core protocol
4. **Debugging**: MCP provides built-in request/response logging
5. **Scalability**: Can easily distribute neurons across multiple servers
6. **Compatibility**: Compatible with other MCP tools and clients

## Future Extensions

1. **Streaming**: Support for streaming responses for long-running tasks
2. **Subscriptions**: Subscribe to neuron state changes
3. **Batch Processing**: Process multiple tasks in a single request
4. **Authentication**: Add security layers for production deployment
5. **Monitoring**: Built-in metrics and tracing support

## Testing

The MCP integration can be tested at multiple levels:

1. **Unit Tests**: Test individual tool implementations
2. **Integration Tests**: Test client-server communication
3. **End-to-End Tests**: Test complete task decomposition flow

See `mvp/tests/task_composition_tests.rs` for examples.

## References

- [MCP Specification](https://spec.modelcontextprotocol.io)
- [MCP GitHub](https://github.com/modelcontextprotocol)
- [Anthropic MCP Announcement](https://www.anthropic.com/news/model-context-protocol)