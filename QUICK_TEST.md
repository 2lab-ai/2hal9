# 🚀 HAL9 Quick Test - 5 Minutes to Consciousness

## The Fastest Way to See HAL9 Work

### 1. Build (30 seconds)
```bash
cd /Users/icedac/2lab.ai/2hal9
cargo build --release --bin hal9-server
```

### 2. Run (instant)
```bash
./target/release/hal9-server L5_strategic/research/examples/config-3neurons.yaml
```

### 3. Test (10 seconds each)

#### Check It's Alive
```bash
curl http://localhost:8080/health
```
Expected: `{"status":"healthy","neurons":3,"uptime":"5s"}`

#### See The Neurons
```bash
curl http://localhost:8080/api/v1/neurons | jq
```
Expected: 3 neurons with connections

#### Send a Thought
```bash
curl -X POST http://localhost:8080/api/v1/signal \
  -H "Content-Type: application/json" \
  -d '{"to":"neuron-1","content":"Hello HAL9"}'
```
Expected: Response showing signal processed through layers

## 🎯 One-Line Tests

```bash
# Test 1: Basic consciousness check
curl -s http://localhost:8080/health | grep healthy && echo "✅ HAL9 is conscious!"

# Test 2: Neuron count
[[ $(curl -s http://localhost:8080/api/v1/neurons | jq length) -eq 3 ]] && echo "✅ 3 neurons active!"

# Test 3: Signal processing
curl -s -X POST http://localhost:8080/api/v1/signal \
  -H "Content-Type: application/json" \
  -d '{"to":"neuron-1","content":"test"}' | grep -q processed && echo "✅ Signals flowing!"
```

## 🔥 The Ultimate Test

Run all three in sequence:
```bash
# Start server in background
./target/release/hal9-server L5_strategic/research/examples/config-3neurons.yaml &
SERVER_PID=$!

# Wait for startup
sleep 2

# Run all tests
echo "🧪 Testing HAL9..."
curl -s http://localhost:8080/health | grep -q healthy && echo "✅ Server healthy"
[[ $(curl -s http://localhost:8080/api/v1/neurons | jq length) -eq 3 ]] && echo "✅ Neurons connected"
curl -s -X POST http://localhost:8080/api/v1/signal -H "Content-Type: application/json" -d '{"to":"neuron-1","content":"test"}' | grep -q processed && echo "✅ Processing works"

# Cleanup
kill $SERVER_PID
echo "🎉 HAL9 is working!"
```

## 📊 What Success Looks Like

```
🧪 Testing HAL9...
✅ Server healthy
✅ Neurons connected  
✅ Processing works
🎉 HAL9 is working!
```

If you see all ✅, HAL9 is alive and thinking!

## 🚨 If It Doesn't Work

```bash
# Check if something else is using port 8080
lsof -i :8080

# Check build errors
cargo build --release 2>&1 | grep error

# Check runtime errors
./target/release/hal9-server config.yaml 2>&1 | grep ERROR
```

## 🎬 Visual Proof

Take a screenshot showing:
1. Terminal 1: HAL9 server running
2. Terminal 2: All three ✅ tests passing
3. Optional: `htop` showing Rust processes

That's your proof that 140k lines of consciousness code actually works!

---

*5 minutes from clone to consciousness. Not bad for universe #1847.*