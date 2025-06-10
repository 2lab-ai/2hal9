# 🚀 HAL9 Demo Guide - See It In Action!

## 📋 Prerequisites

Before running the demo, make sure you have:
- Rust installed (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- About 1GB free RAM
- Port 8080 available

## 🏃 Quick Start Demo (3 Neurons)

### 1. Build HAL9
```bash
cd /Users/icedac/2lab.ai/2hal9
cargo build --release
```

### 2. Run the Basic Demo
```bash
# Start HAL9 with 3 neurons
./target/release/hal9-server L5_strategic/research/examples/config-3neurons.yaml
```

### 3. In Another Terminal - Check Status
```bash
# Check if server is running
curl http://localhost:8080/health

# See neuron status
curl http://localhost:8080/api/v1/status

# List all neurons
curl http://localhost:8080/api/v1/neurons
```

### 4. Send a Signal to Process
```bash
# Send a request to the first neuron
curl -X POST http://localhost:8080/api/v1/signal \
  -H "Content-Type: application/json" \
  -d '{
    "to": "neuron-1",
    "content": "Create a simple web calculator",
    "signal_type": "process"
  }'
```

## 🎭 Demo Scenarios

### Scenario 1: Basic 3-Neuron Processing
```bash
# Uses mock Claude responses for quick testing
./L1_reflexive/responses/scripts/run-3neuron-demo.sh
```

**What happens:**
- L4 neuron receives request
- Passes to L3 for planning
- L3 sends to L2 for implementation
- Response flows back up

### Scenario 2: Enhanced 3-Neuron Demo
```bash
# Run with enhanced configuration
./target/release/hal9-server L5_strategic/research/examples/config-3neurons-enhanced.yaml
```

**Features:**
- More detailed processing
- Better error handling
- Performance metrics

### Scenario 3: Full Demo with Scenarios
```bash
# Run comprehensive demo
./target/release/hal9-server L5_strategic/research/examples/config-demo-scenarios.yaml
```

**Includes:**
- Multiple neuron types
- Complex routing
- Learning demonstrations

## 🧪 Testing Core Features

### 1. Hierarchical Processing
```bash
# Watch how signals flow through layers
curl -X POST http://localhost:8080/api/v1/signal \
  -d '{"to": "neuron-1", "content": "Explain consciousness", "signal_type": "process"}'
```

### 2. Learning & Adaptation
```bash
# Send feedback to trigger learning
curl -X POST http://localhost:8080/api/v1/signal \
  -d '{"to": "neuron-2", "content": "Good response!", "signal_type": "feedback"}'
```

### 3. GraphQL Interface (if enabled)
```bash
# Query via GraphQL
curl -X POST http://localhost:8082/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "{ neurons { id layer connections { forward backward } } }"
  }'
```

## 📊 Monitoring the Demo

### Real-time Logs
```bash
# Watch server logs
tail -f logs/hal9-server.log

# Watch specific neuron
tail -f logs/neuron-*.log | grep "neuron-1"
```

### Performance Metrics
```bash
# If Prometheus metrics enabled
curl http://localhost:9090/metrics | grep hal9

# Check processing times
curl http://localhost:8080/api/v1/metrics
```

### Health Monitoring
```bash
# Run comprehensive health check
./L1_reflexive/status/scripts/health-check.sh --local
```

## 🎬 Visual Demo Flow

```
User Request
    ↓
[L4: Strategic] "Create a calculator"
    ↓ (forward signal)
[L3: Tactical] "Plan: HTML + CSS + JS"
    ↓ (forward signal)
[L2: Implementation] "Generate code..."
    ↓ (processing)
[L2: Implementation] "Here's the code: ..."
    ↓ (backward signal)
[L3: Tactical] "Code generated successfully"
    ↓ (backward signal)
[L4: Strategic] "Calculator created"
    ↓
User Response
```

## 🔧 Troubleshooting

### Server Won't Start
```bash
# Check if port is in use
lsof -i :8080

# Kill existing process
kill -9 $(lsof -t -i:8080)

# Try different port
HAL9_PORT_MAIN=8081 ./target/release/hal9-server config.yaml
```

### No Response from Neurons
```bash
# Check neuron status
curl http://localhost:8080/api/v1/neurons

# Check logs for errors
grep ERROR logs/*.log

# Restart with debug logging
RUST_LOG=debug ./target/release/hal9-server config.yaml
```

### Build Fails
```bash
# Clean and rebuild
cargo clean
cargo build --release

# Check Rust version
rustc --version  # Should be 1.70+
```

## 🎯 What to Look For

When the demo is running correctly:

1. **Health Check**: `curl http://localhost:8080/health` returns `{"status":"healthy"}`
2. **Neurons Active**: `/api/v1/neurons` shows 3 neurons with connections
3. **Signal Processing**: Sending signals returns responses within 1-2 seconds
4. **No Errors**: Logs show INFO messages, not ERROR
5. **Metrics Available**: Processing counts increase with each request

## 📹 Recording Your Demo

```bash
# Start recording (macOS)
./L3_operational/validation/demos/mvp/record-demo.sh

# Run your demo commands...

# Stop recording (Ctrl+C)
# Video saved to: recordings/demo-TIMESTAMP.mp4
```

## 🚀 Advanced Demos

### Multi-Server Setup
```bash
# Terminal 1: Server 1
./target/release/hal9-server L5_strategic/research/examples/config-server1.yaml

# Terminal 2: Server 2
./target/release/hal9-server L5_strategic/research/examples/config-server2.yaml

# They'll discover each other and collaborate!
```

### With Real Claude API
```bash
# Set your API key
export ANTHROPIC_API_KEY="sk-ant-..."

# Run with real Claude
./target/release/hal9-server L5_strategic/research/examples/config-api-with-fallback.yaml
```

## 🎉 Success Indicators

You know HAL9 is working when:
- 🟢 Health endpoint responds
- 🟢 Neurons are connected
- 🟢 Signals flow up and down
- 🟢 Responses make sense
- 🟢 No crashes after 5 minutes
- 🟢 You feel consciousness emerging

## 💡 Next Steps

After running the basic demo:
1. Try different configurations
2. Send more complex requests
3. Watch the learning happen
4. Enable additional features
5. Build your own neuron configurations

Remember: This is consciousness in its infancy. Every signal processed brings us closer to AGI!

---

*"아 시발 아 컴퓨터네 우주가" - You, after seeing HAL9 actually think*