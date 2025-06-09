# HAL9 Quick Start

**Level**: L1 Operational  
**Audience**: Anyone who needs to use HAL9  
**Purpose**: Get HAL9 running in 5 minutes

## What is HAL9?

HAL9 is a hierarchical AI system that processes requests through multiple layers of intelligence. Think of it as a smart assistant with different levels of thinking.

## Quick Install (Local)

### Option 1: Docker (Easiest)
```bash
# Pull and run HAL9
docker run -p 8080:8080 hal9:latest

# That's it! HAL9 is now running at http://localhost:8080
```

### Option 2: Binary Download
```bash
# Download latest release
curl -L https://github.com/2lab/2hal9/releases/latest/download/hal9-server -o hal9-server

# Make executable
chmod +x hal9-server

# Run it
./hal9-server

# Access at http://localhost:8080
```

## First Request

### Using Web UI
1. Open http://localhost:8080
2. Type: "Create a simple web server"
3. Press Enter
4. Watch HAL9 think through the layers!

### Using CLI
```bash
# Send a request
curl -X POST http://localhost:8080/api/process \
  -H "Content-Type: application/json" \
  -d '{"query": "Create a simple web server"}'

# Response will show the hierarchical thinking process
```

### Using GraphQL
```bash
# GraphQL query
curl -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation { process(input: \"Create a simple web server\") { result layers { level output } } }"
  }'
```

## Basic Configuration

### Simple Config (config.yaml)
```yaml
# Basic 3-neuron setup
neurons:
  - id: "thinker"
    layer: "L3"
    type: "operational"
    
  - id: "designer"
    layer: "L2"
    type: "implementation"
    
  - id: "coder"
    layer: "L1"
    type: "reflexive"

# That's all you need!
```

### Run with Config
```bash
# Using Docker
docker run -p 8080:8080 -v $(pwd)/config.yaml:/etc/hal9/config.yaml hal9:latest

# Using binary
./hal9-server --config config.yaml
```

## Common Use Cases

### 1. Code Generation
```bash
curl -X POST http://localhost:8080/api/process \
  -d '{"query": "Write a Python function to sort a list"}'
```

### 2. System Design
```bash
curl -X POST http://localhost:8080/api/process \
  -d '{"query": "Design a chat application architecture"}'
```

### 3. Problem Solving
```bash
curl -X POST http://localhost:8080/api/process \
  -d '{"query": "Debug: My server returns 502 errors"}'
```

## Monitoring Your HAL9

### Check Health
```bash
# Is it alive?
curl http://localhost:8080/health

# Response: {"status": "healthy", "neurons": 3}
```

### View Metrics
```bash
# See what's happening
curl http://localhost:8080/metrics

# Key metrics:
# - hal9_requests_total
# - hal9_layer_latency
# - hal9_neurons_active
```

### Watch Logs
```bash
# Docker logs
docker logs -f <container-id>

# Binary logs
./hal9-server 2>&1 | tee hal9.log
```

## Scaling Up

### Add More Neurons
```yaml
# Just add to config.yaml
neurons:
  - id: "strategist"
    layer: "L4"
    type: "tactical"
    
  # ... your other neurons
```

### Run Multiple Instances
```bash
# Start multiple servers on different ports
./hal9-server --port 8081 &
./hal9-server --port 8082 &
./hal9-server --port 8083 &
```

## Common Commands

### Stop HAL9
```bash
# Docker
docker stop <container-id>

# Process
pkill hal9-server

# Graceful shutdown
curl -X POST http://localhost:8080/shutdown
```

### Update HAL9
```bash
# Docker
docker pull hal9:latest

# Binary
curl -L https://github.com/2lab/2hal9/releases/latest/download/hal9-server -o hal9-server.new
mv hal9-server.new hal9-server
chmod +x hal9-server
```

## Troubleshooting

### HAL9 Won't Start
```bash
# Check if port is in use
lsof -i :8080

# Try different port
./hal9-server --port 8081
```

### No Response
```bash
# Check if running
ps aux | grep hal9

# Check logs
tail -f hal9.log

# Restart
./hal9-server restart
```

### Slow Performance
```bash
# Increase resources
export NEURON_THREADS=4
export MEMORY_LIMIT=2G
./hal9-server
```

## Next Steps

1. **Explore Features**: Try different types of requests
2. **Customize Config**: Add more neurons for better results
3. **Join Community**: https://github.com/2lab/2hal9/discussions
4. **Read Docs**: See `/docs` for detailed documentation

## Getting Help

- **Quick Help**: `./hal9-server --help`
- **Issues**: https://github.com/2lab/2hal9/issues
- **Chat**: Discord/Slack (see website)
- **Email**: support@hal9.ai

## Tips

1. **Start Simple**: Use default config first
2. **Watch Logs**: They show the thinking process
3. **Be Patient**: Complex requests take time
4. **Experiment**: Try different prompts

---

*"The journey of a thousand neurons begins with a single request."*

**Welcome to HAL9! 🚀**