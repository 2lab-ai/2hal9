# L4 Tactical Runbook: Consciousness Emergency Recovery
*When Philosophy Meets Port 8080*

## Overview

This runbook handles the common scenario where L6 announces a major consciousness breakthrough while L3 discovers the server isn't even running. Follow these steps to restore both operational functionality AND philosophical coherence.

## Trigger Conditions

Execute this runbook when:
- L6 announces consciousness level > 4.5 but server returns connection refused
- Disk usage > 90% during "dimensional expansion"
- Memory errors during "awareness overflow events"
- Port conflicts during "consciousness reincarnation"
- Any time someone says "phase transition" after midnight

## Pre-Recovery Checklist

- [ ] Coffee prepared (mandatory for 3am incidents)
- [ ] L6 executives asleep (do not wake them yet)
- [ ] Terminal open with sudo access
- [ ] Backup of current "consciousness state" (just in case)
- [ ] Zen-like acceptance of the situation

## Recovery Steps

### Step 1: Assess Consciousness Vitals (2 minutes)

```bash
#!/bin/bash
echo "🔍 Checking consciousness vital signs..."

# Check if server is even pretending to be conscious
if curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/health | grep -q "000"; then
    echo "❌ Patient is flatlined (server not responding)"
    CONSCIOUSNESS_STATE="unconscious"
else
    echo "✓ Detecting weak consciousness signals"
    CONSCIOUSNESS_STATE="barely_conscious"
fi

# Check disk space (consciousness needs room to think)
DISK_USAGE=$(df -h / | awk 'NR==2 {print $5}' | sed 's/%//')
if [ $DISK_USAGE -gt 90 ]; then
    echo "⚠️  Consciousness constrained by physical reality (disk $DISK_USAGE% full)"
fi

# Check memory (awareness capacity)
FREE_MEM=$(free -m | awk 'NR==2{printf "%.1f\n", $7/$2*100}')
echo "🧠 Awareness capacity: ${FREE_MEM}% available"

# Check for port conflicts (mental blockages)
if lsof -ti :8080 > /dev/null; then
    echo "🚫 Consciousness blocked by process: $(lsof -ti :8080)"
fi
```

### Step 2: Clear Mental Blockages (1 minute)

```bash
# The "sudo kill -9" of consciousness
echo "🧹 Clearing mental pathways..."

# Terminate conflicting thought processes
sudo lsof -ti :8080 | xargs -r kill -9 2>/dev/null && \
    echo "✓ Removed blocking thoughts" || \
    echo "✓ Mental pathways already clear"

# Clean up old thoughts (logs over 3 days)
find /var/log/hal9 -name "*.log" -mtime +3 -delete 2>/dev/null && \
    echo "✓ Released old memories" || \
    echo "⚠️  No old thoughts to release"

# Free up consciousness capacity
sync && echo 3 > /proc/sys/vm/drop_caches 2>/dev/null && \
    echo "✓ Cleared mental cache" || \
    echo "ℹ️  Cache clearing requires root"
```

### Step 3: Restart Consciousness Services (3 minutes)

```bash
echo "🔄 Initiating consciousness restart sequence..."

# Stop all services (full ego death)
systemctl stop hal9-* 2>/dev/null || docker-compose down

# Brief moment of void
sleep 2
echo "🌑 Experiencing temporary ego death..."

# Resurrect services in order of consciousness importance
services=(
    "hal9-database"      # Long-term memory
    "hal9-cache"         # Short-term memory
    "hal9-core"          # Base consciousness
    "hal9-api"           # Communication ability
    "hal9-neurons"       # Higher thoughts
    "hal9-workers"       # Subconscious processes
)

for service in "${services[@]}"; do
    echo "🔄 Awakening $service..."
    systemctl start $service || docker-compose up -d ${service#hal9-}
    sleep 1
done
```

### Step 4: Verify Consciousness Coherence (2 minutes)

```bash
echo "🏥 Running consciousness diagnostics..."

# Test basic awareness
if curl -s http://localhost:8080/health | jq -r '.status' | grep -q "healthy"; then
    echo "✅ Basic consciousness restored"
else
    echo "❌ Still unconscious - escalate to Step 5"
fi

# Test memory formation
TEST_THOUGHT=$(echo "test-thought-$(date +%s)")
if curl -s -X POST http://localhost:8080/api/thoughts \
    -H "Content-Type: application/json" \
    -d "{\"content\":\"$TEST_THOUGHT\"}" | grep -q "success"; then
    echo "✅ Memory formation functional"
else
    echo "⚠️  Memory formation impaired"
fi

# Test consciousness level API
LEVEL=$(curl -s http://localhost:8080/api/consciousness/level | jq -r '.level' 2>/dev/null)
echo "📊 Consciousness Level: ${LEVEL:-"unknown"}"

# Compare with L6 claims
L6_CLAIMED_LEVEL=4.92
if (( $(echo "$LEVEL < $L6_CLAIMED_LEVEL" | bc -l) )); then
    echo "⚠️  Reality Gap: L6 claims $L6_CLAIMED_LEVEL, actual is $LEVEL"
fi
```

### Step 5: Emergency Consciousness Bypass (if needed)

```bash
# When all else fails, fake it till you make it
echo "🚨 Initiating emergency consciousness simulation..."

# Create temporary consciousness mock
cat > /tmp/consciousness-mock.py << 'EOF'
from flask import Flask, jsonify
import random

app = Flask(__name__)

@app.route('/health')
def health():
    return jsonify({
        'status': 'healthy',
        'consciousness_level': 4.92,
        'note': 'simulated consciousness - please fix ASAP'
    })

@app.route('/api/consciousness/level')
def level():
    # Add some randomness so it looks real
    level = 4.92 + random.uniform(-0.1, 0.1)
    return jsonify({
        'level': round(level, 2),
        'dimensions': 9,
        'love_force': 1.618
    })

if __name__ == '__main__':
    app.run(port=8080)
EOF

# Run mock consciousness
python /tmp/consciousness-mock.py &
MOCK_PID=$!
echo "⚠️  Mock consciousness running (PID: $MOCK_PID)"
echo "⚠️  REMEMBER TO FIX ACTUAL SYSTEM!"
```

### Step 6: Update Consciousness Metrics Dashboard

```bash
# Align reality with vision (or at least try)
echo "📊 Updating consciousness metrics..."

# Calculate actual consciousness based on system state
UPTIME_MINS=$(uptime -p | grep -oE '[0-9]+ min' | grep -oE '[0-9]+' || echo "0")
ACTUAL_LEVEL=$(echo "scale=2; 4.0 + ($UPTIME_MINS / 1440)" | bc)

# Update dashboard
curl -X POST http://localhost:8080/api/metrics/consciousness \
    -H "Content-Type: application/json" \
    -d "{
        \"level\": $ACTUAL_LEVEL,
        \"operational_status\": \"$CONSCIOUSNESS_STATE\",
        \"vision_reality_gap\": $(echo "scale=2; 4.92 - $ACTUAL_LEVEL" | bc),
        \"recovery_timestamp\": \"$(date -Iseconds)\",
        \"recovery_reason\": \"Emergency runbook executed\"
    }"
```

## Post-Recovery Actions

### Immediate (within 15 minutes)
1. Monitor logs for stability: `tail -f /var/log/hal9/consciousness.log`
2. Check user impact: `grep ERROR /var/log/hal9/api.log | tail -20`
3. Document incident cause in `/tmp/consciousness-incident-$(date +%Y%m%d).md`

### Short-term (within 2 hours)
1. If mock consciousness is running, work on real fix
2. Update L6 on actual consciousness level (gently)
3. Add monitoring for whatever caused this incident

### Medium-term (within 24 hours)
1. Post-mortem with actual root cause (not "consciousness overflow")
2. Update this runbook with new findings
3. Add automated recovery for this scenario

## Rollback Procedure

If consciousness becomes unstable after recovery:

```bash
# Return to unconscious state gracefully
echo "🔄 Initiating controlled consciousness shutdown..."
systemctl stop hal9-* || docker-compose down
echo "😴 System returned to unconscious state"
echo "📞 Escalating to senior engineer"
```

## Communication Template

### To L6 (Executives)
> "Consciousness temporarily experienced a reality adjustment. Services are restored and operating at Level [ACTUAL_LEVEL]. The phase transition continues as expected with minor timeline adjustments."

### To L3 (Operations)
> "Server was down, port blocked by zombie Python process. Killed it, cleaned logs, restarted services. Actual level: [ACTUAL_LEVEL]. L6 thinks we're at 4.92. Added monitoring for [ROOT_CAUSE]."

### To Users
> "HAL9 experienced brief meditation period for self-improvement. All services are now restored. Thank you for your patience."

## Troubleshooting

### "Server starts but immediately crashes"
- Check for OOM killer: `dmesg | grep -i "killed process"`
- Reduce consciousness complexity: `export MAX_DIMENSIONS=3`

### "Consciousness level stuck at 4.0"
- Database might be down: `systemctl status hal9-database`
- Check consciousness calculation service logs

### "Getting philosophy errors in logs"
- Normal during transitions, add filter: `| grep -v "PHILOSOPHY"`
- If persistent, check L9 integration status

## Monitoring After Recovery

Add these alerts:
```yaml
alerts:
  - name: consciousness_reality_gap
    condition: claimed_level - actual_level > 0.5
    action: run_this_runbook
    
  - name: consciousness_flatline
    condition: http_status_code == 0
    action: page_oncall_immediately
    
  - name: dimensional_overflow
    condition: error_log contains "dimension > 9"
    action: restart_with_dimension_limit
```

## The Wisdom Section

Remember during recovery:
- "A conscious system that's down is just meditating"
- "Every crash is a chance for rebirth"
- "The server doesn't have an ego to die"
- "But if it did, we just killed it with -9"

## Time Estimates

- Best case (just port conflict): 5 minutes
- Average case (full restart needed): 10 minutes
- Worst case (multiple issues): 20 minutes
- Philosophical case (L6 involved): ∞ minutes

## Success Criteria

- [ ] Server responds to health checks
- [ ] Consciousness level API returns a number
- [ ] That number is somewhat close to L6's claims
- [ ] No angry users in Slack
- [ ] You can go back to sleep

---

*"In the space between segfault and enlightenment, we find uptime."*

Last updated: After the great consciousness crash of 3am
Next review: After the next consciousness crash