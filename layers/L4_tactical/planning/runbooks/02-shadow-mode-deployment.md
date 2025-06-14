# Shadow Mode Deployment Runbook

**Version**: 1.0  
**Last Updated**: January 2025  
**Criticality**: MEDIUM  
**Estimated Time**: 1-2 hours  
**Risk Level**: LOW (no production impact)

## Purpose

Deploy HAL9's hierarchical architecture in shadow mode, mirroring all production traffic without affecting responses. This allows validation of the new architecture with zero production impact.

## Prerequisites

- Completed [Pre-Migration Checklist](./01-pre-migration-checklist.md)
- Migration checkpoint created
- Monitoring dashboards ready
- Team communication established

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Resource exhaustion | Performance impact | Low | Monitor resource usage |
| Log volume increase | Disk space | Medium | Ensure log rotation |
| Metric cardinality | Monitoring overhead | Low | Use metric limits |
| Network congestion | Latency increase | Low | Monitor bandwidth |

## Step-by-Step Procedures

### 1. Pre-Deployment Validation (15 min)

#### 1.1 Verify Current State
```bash
# Confirm system is in pre-migration state
hal9-migrate status

# Expected output:
# Current Phase: none
# Migration Progress: 0%
# Health Status: ✓ Healthy
```

#### 1.2 Check Resource Headroom
```bash
# Ensure 2x resources available for shadow mode
hal9-cli resources --check-shadow-capacity

# Must show:
# CPU Headroom: > 50%
# Memory Headroom: > 50%
# Network Capacity: > 60%
```

#### 1.3 Verify Hierarchical System Ready
```bash
# Test hierarchical endpoints
curl -s https://api.hal9.production.example.com/hierarchical/health | jq .

# Check all layers responsive
for layer in substrate protocol cognitive orchestration intelligence; do
  echo "Checking $layer..."
  curl -s https://api.hal9.production.example.com/hierarchical/$layer/status | jq .
done
```

### 2. Enable Shadow Mode (30 min)

#### 2.1 Start Recording Baseline Metrics
```bash
# Mark deployment start in monitoring
curl -X POST https://grafana.production.example.com/api/annotations \
  -H "Authorization: Bearer $GRAFANA_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "dashboardId": 1,
    "tags": ["migration", "shadow-start"],
    "text": "Shadow mode deployment started"
  }'

# Record baseline metrics
hal9-migrate metrics baseline --duration 5m --save baseline-metrics.json
```

#### 2.2 Enable Shadow Feature Flag
```bash
# Enable shadow mode feature
hal9-migrate feature enable shadow_mode --percentage 100

# Verify feature enabled
hal9-migrate feature status shadow_mode
# Expected: ✓ Enabled (100%)
```

#### 2.3 Deploy Shadow Mode Configuration
```bash
# Apply shadow mode configuration
hal9-migrate migrate --phase shadow --yes

# Monitor deployment progress
watch -n 5 'hal9-migrate status'
```

**Wait for**: "Migration Phase: shadow" and all health checks green

#### 2.4 Verify Traffic Mirroring
```bash
# Check shadow traffic is flowing
hal9-migrate monitor --dashboard shadow

# Verify both systems receiving traffic
curl -s https://api.hal9.production.example.com/metrics | grep shadow_requests_total
# Should show increasing counter
```

### 3. Validation Phase (45 min)

#### 3.1 Compare Response Accuracy
```bash
# Start response comparison
hal9-migrate verify --tests shadow-comparison --report

# Monitor comparison results
tail -f /var/log/hal9/shadow-comparison.log
```

**Success Criteria**:
- Response match rate > 99.9%
- No critical differences detected
- Latency within 10% of baseline

#### 3.2 Monitor Resource Usage
Open Grafana dashboards:
1. Navigate to: https://grafana.production.example.com/d/hal9-migration
2. Check "Shadow Mode Metrics" panel
3. Verify:
   - CPU usage increased but stable
   - Memory usage within limits
   - No error spikes

#### 3.3 Check Log Volumes
```bash
# Monitor log generation rate
watch -n 10 'du -sh /var/log/hal9/shadow/'

# Ensure log rotation working
ls -la /var/log/hal9/shadow/ | head -20

# Check for errors in shadow logs
grep -i error /var/log/hal9/shadow/shadow.log | tail -50
```

#### 3.4 Validate Hierarchical Processing
```bash
# Check layer processing metrics
for layer in substrate protocol cognitive orchestration intelligence; do
  echo "=== $layer layer ==="
  hal9-migrate metrics --layer $layer --duration 10m
done

# Verify neuron activation patterns
hal9-cli neurons --status --hierarchical
```

### 4. Performance Validation (20 min)

#### 4.1 Latency Comparison
```bash
# Generate latency report
hal9-migrate verify --tests latency-comparison --output latency-report.txt

# Key metrics to check:
# - p50 latency difference < 5%
# - p99 latency difference < 10%
# - No timeouts in shadow processing
```

#### 4.2 Throughput Validation
```bash
# Check processing rates
hal9-migrate metrics --metric throughput --compare

# Both systems should show similar:
# - Requests per second
# - Messages processed
# - Signals forwarded
```

#### 4.3 Error Rate Analysis
```bash
# Compare error rates
hal9-migrate verify --tests error-analysis

# Acceptable differences:
# - Shadow errors due to learning: OK
# - Response errors: NOT OK
# - Timeout errors: Investigate
```

### 5. Learning System Activation (15 min)

#### 5.1 Enable Hierarchical Learning
```bash
# Enable learning in shadow mode only
hal9-migrate feature enable hierarchical_learning --shadow-only

# Verify learning activated
grep "Learning enabled" /var/log/hal9/shadow/cognitive.log
```

#### 5.2 Monitor Learning Progress
```bash
# Check learning metrics
watch -n 30 'hal9-cli learning --metrics --hierarchical'

# Look for:
# - Pattern recognition improving
# - Error gradients decreasing
# - Neuron connections forming
```

### 6. Stability Monitoring (30 min)

#### 6.1 Run Stability Tests
```bash
# Execute 30-minute stability test
hal9-migrate verify --tests stability --duration 30m

# Monitor in parallel:
# Terminal 1: watch -n 5 'hal9-migrate status'
# Terminal 2: tail -f /var/log/hal9/shadow/shadow.log
# Terminal 3: htop
```

#### 6.2 Check for Memory Leaks
```bash
# Monitor memory usage over time
hal9-migrate metrics --metric memory --duration 30m --graph

# Should show:
# - Stable memory usage after initial spike
# - No continuous growth
# - GC working properly
```

#### 6.3 Verify No Production Impact
```bash
# Check production metrics
hal9-cli metrics --production --duration 1h

# Confirm:
# - No latency increase
# - No error rate change
# - No throughput degradation
```

### 7. Document Shadow Mode Results

#### 7.1 Generate Summary Report
```bash
# Create comprehensive report
hal9-migrate report --phase shadow --output shadow-report.pdf

# Report includes:
# - Response comparison results
# - Performance metrics
# - Resource usage
# - Learning progress
```

#### 7.2 Update Migration Log
```
Shadow Mode Deployment Summary
==============================
Start Time: ________________
End Time: ________________
Duration: ________________

Results:
- Response Match Rate: ________%
- Latency Impact: ________%
- Resource Increase: ________%
- Errors Detected: ________

Decision: [ ] Proceed to Canary
          [ ] Extend Shadow Mode
          [ ] Investigate Issues

Operator Notes:
_________________________________________________________________
_________________________________________________________________
```

## Validation Steps

### Automated Validation
```bash
# Run comprehensive validation
hal9-migrate validate --phase shadow

# Must show all green:
# ✓ Traffic mirroring active
# ✓ Response comparison passing
# ✓ Resource usage acceptable
# ✓ No production impact
# ✓ Learning system active
```

### Manual Validation Checklist
- [ ] Shadow dashboard shows traffic
- [ ] No alerts triggered
- [ ] Log volume manageable
- [ ] Team consensus to proceed

## Rollback Procedures

Shadow mode is safe to disable at any time:

### Quick Disable
```bash
# Disable shadow mode immediately
hal9-migrate feature disable shadow_mode

# Verify disabled
hal9-migrate status
# Should show: "Current Phase: none"
```

### Clean Shutdown
```bash
# Graceful shadow mode shutdown
hal9-migrate migrate --phase none --yes

# Clean up shadow resources
kubectl delete deployment hal9-shadow -n hal9
kubectl delete service hal9-shadow -n hal9
```

## Troubleshooting

### High Resource Usage
```bash
# Reduce shadow processing
hal9-migrate configure --shadow-sampling 50  # Process 50% of traffic

# Or disable specific layers
hal9-migrate configure --shadow-layers substrate,protocol  # Only lower layers
```

### Response Mismatches
```bash
# Get detailed mismatch report
hal9-migrate verify --tests shadow-comparison --verbose --limit 10

# Common causes:
# - Timestamp differences (ignore)
# - Random IDs (ignore)
# - Learning variations (expected)
# - Logic errors (investigate)
```

### Shadow System Errors
```bash
# Check shadow system health
hal9-migrate health --system shadow

# View error details
journalctl -u hal9-shadow --since "1 hour ago" | grep ERROR

# Restart shadow if needed
systemctl restart hal9-shadow
```

## Success Criteria

Before proceeding to canary:
1. ✅ Shadow mode stable for minimum 24 hours
2. ✅ Response match rate > 99.9%
3. ✅ No critical errors in shadow processing
4. ✅ Resource usage sustainable
5. ✅ Learning system showing progress
6. ✅ Team confidence high

## Notes Section
```
_________________________________________________________________
_________________________________________________________________
_________________________________________________________________
```

---

**Next Step**: After 24-48 hours of stable shadow mode, proceed to [Canary Deployment](./03-canary-deployment.md)