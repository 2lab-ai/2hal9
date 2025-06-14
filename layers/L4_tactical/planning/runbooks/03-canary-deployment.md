# Canary Deployment Runbook

**Version**: 1.0  
**Last Updated**: January 2025  
**Criticality**: HIGH  
**Estimated Time**: 2-4 hours per stage  
**Risk Level**: MEDIUM (production traffic affected)

## Purpose

Progressively route production traffic to the hierarchical architecture, starting with 5% and gradually increasing. This runbook covers safe canary deployment with automatic rollback capabilities.

## Prerequisites

- Shadow mode running successfully for 24+ hours
- Response match rate > 99.9% in shadow mode
- All team members available
- Rollback procedures reviewed
- Customer support notified

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Response errors | User impact | Medium | Automatic rollback at 0.1% error rate |
| Performance degradation | User experience | Low | Rollback at p99 > 50ms |
| Data inconsistency | Data integrity | Low | Shadow mode validation |
| Cascade failure | System outage | Very Low | Circuit breakers active |

## Canary Stages

| Stage | Traffic % | Duration | Success Criteria |
|-------|-----------|----------|------------------|
| 1 | 5% | 2 hours | Error rate < 0.1%, p99 < 15ms |
| 2 | 10% | 4 hours | Error rate < 0.1%, p99 < 12ms |
| 3 | 25% | 8 hours | Error rate < 0.1%, p99 < 10ms |
| 4 | 50% | 24 hours | Error rate < 0.1%, p99 < 10ms |

## Step-by-Step Procedures

### 1. Pre-Canary Setup (30 min)

#### 1.1 Create Canary Checkpoint
```bash
# Create checkpoint before canary
hal9-migrate state checkpoint \
  --name "pre-canary-$(date +%Y%m%d-%H%M%S)" \
  --description "Checkpoint before canary deployment"

# Export current state
hal9-migrate state export --output pre-canary-state.json
```

#### 1.2 Configure Automatic Rollback
```bash
# Set rollback thresholds
hal9-migrate configure rollback \
  --error-threshold 0.001 \
  --latency-threshold-p99 50 \
  --latency-threshold-p50 15 \
  --window 5m

# Verify configuration
hal9-migrate config show --rollback
```

#### 1.3 Set Up Monitoring Alerts
```bash
# Configure canary-specific alerts
cat <<EOF | kubectl apply -f -
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: canary-alerts
  namespace: hal9
spec:
  groups:
  - name: canary
    interval: 30s
    rules:
    - alert: CanaryHighErrorRate
      expr: |
        rate(hal9_errors_total{system="hierarchical"}[5m]) > 0.001
      for: 2m
      labels:
        severity: critical
        phase: canary
      annotations:
        summary: "High error rate in canary deployment"
        description: "Error rate {{ \$value }} exceeds threshold"
EOF
```

#### 1.4 Notify Stakeholders
```bash
# Send notification
echo "Starting HAL9 canary deployment at $(date)" | \
  mail -s "[HAL9] Canary Deployment Starting" engineering@company.com

# Update status page
curl -X POST https://status.company.com/api/v1/incidents \
  -H "Authorization: Bearer $STATUS_PAGE_TOKEN" \
  -d '{"status": "investigating", "message": "HAL9 canary deployment in progress"}'
```

### 2. Stage 1: 5% Traffic (2 hours)

#### 2.1 Enable Canary with 5% Traffic
```bash
# Start canary deployment
hal9-migrate migrate --phase canary --percentage 5 --yes

# Monitor deployment
watch -n 5 'hal9-migrate status --detailed'
```

**Wait for**: Status shows "Phase: canary (5%)"

#### 2.2 Initial Validation (First 15 minutes)
```bash
# Watch real-time metrics
hal9-migrate monitor --dashboard combined

# In another terminal, watch for errors
tail -f /var/log/hal9/error.log | grep -E "(hierarchical|canary)"

# Check immediate impact
hal9-migrate metrics --compare --realtime
```

**STOP CRITERIA**: If error rate > 0.1% or p99 > 50ms, execute rollback immediately

#### 2.3 Monitor Key Metrics
Open multiple terminals:

**Terminal 1 - Error Rate**:
```bash
watch -n 2 'hal9-migrate metrics --metric error-rate --window 5m'
```

**Terminal 2 - Latency**:
```bash
watch -n 2 'hal9-migrate metrics --metric latency --percentiles p50,p99 --window 5m'
```

**Terminal 3 - Traffic Split**:
```bash
watch -n 5 'hal9-migrate traffic --show-split'
```

#### 2.4 User Experience Validation
```bash
# Check user-facing metrics
hal9-cli ux-metrics --compare-systems

# Sample user sessions
hal9-migrate trace --sample-sessions 10 --system hierarchical
```

#### 2.5 Stage 1 Validation (After 2 hours)
```bash
# Generate stage report
hal9-migrate report --stage "canary-5" --duration 2h

# Checklist:
# [ ] Error rate < 0.1%
# [ ] p99 latency < 15ms
# [ ] No user complaints
# [ ] No system alerts
# [ ] Team consensus to proceed
```

### 3. Stage 2: 10% Traffic (4 hours)

#### 3.1 Increase to 10%
```bash
# Increase canary traffic
hal9-migrate migrate --phase canary --percentage 10 --yes

# Confirm change
hal9-migrate status
# Should show: "Phase: canary (10%)"
```

#### 3.2 Enhanced Monitoring
```bash
# Start detailed monitoring
hal9-migrate monitor --enhanced --duration 4h &

# Enable trace sampling for analysis
hal9-migrate trace --enable --sample-rate 0.1
```

#### 3.3 Load Pattern Analysis
```bash
# Analyze traffic patterns
hal9-migrate analyze --traffic-patterns --duration 1h

# Check for:
# - Peak load handling
# - Geographic distribution
# - API endpoint coverage
```

### 4. Stage 3: 25% Traffic (8 hours)

#### 4.1 Increase to 25%
```bash
# Quarter of traffic to hierarchical
hal9-migrate migrate --phase canary --percentage 25 --yes

# Set up extended monitoring
hal9-migrate monitor --extended --alert-threshold strict
```

#### 4.2 Comprehensive Validation
```bash
# Run validation suite
hal9-migrate validate --comprehensive --phase canary-25

# Tests include:
# - Response accuracy
# - Data consistency
# - Performance benchmarks
# - Resource utilization
```

#### 4.3 Business Metrics Check
```bash
# Verify business KPIs
hal9-cli business-metrics --compare-period "8h"

# Check:
# - User engagement unchanged
# - Transaction success rates
# - API consumer satisfaction
```

### 5. Stage 4: 50% Traffic (24 hours)

#### 4.1 Increase to 50%
```bash
# Half traffic to each system
hal9-migrate migrate --phase canary --percentage 50 --yes

# Enable 24-hour monitoring
hal9-migrate monitor --duration 24h --record
```

#### 4.2 A/B Analysis
```bash
# Compare both systems equally
hal9-migrate analyze --ab-test --duration 24h

# Generate comparison report
hal9-migrate report --ab-comparison --output ab-report.pdf
```

#### 4.3 Stability Verification
```bash
# Long-running stability test
hal9-migrate verify --tests stability,performance,accuracy --duration 24h
```

### 6. Canary Success Validation

#### 6.1 Final Metrics Review
```bash
# Generate comprehensive canary report
hal9-migrate report --phase canary-complete --output canary-final.pdf

# Review all metrics
hal9-migrate metrics --summary --phase canary
```

#### 6.2 Team Review Meeting
Document in meeting notes:
```
Canary Deployment Review
========================
Date: ________________
Attendees: ________________

Metrics Review:
- Total Duration: ______ hours
- Peak Error Rate: _____% 
- Peak p99 Latency: _____ ms
- User Complaints: ______
- Rollbacks Triggered: ______

Decision:
[ ] Proceed to State Migration
[ ] Extend Canary Period
[ ] Rollback and Investigate

Sign-offs:
Engineering Lead: ________________
Operations Lead: ________________
Product Owner: ________________
```

## Validation Steps

After each stage:
1. Error rate consistently < 0.1%
2. Latency p99 < target for stage
3. No data inconsistencies detected
4. Resource usage sustainable
5. No user complaints
6. All health checks passing

## Rollback Procedures

### Automatic Rollback
Triggered automatically when thresholds exceeded:
```bash
# Monitor automatic rollback
tail -f /var/log/hal9/rollback.log

# If triggered, system will:
# 1. Route 100% traffic to flat architecture
# 2. Create incident report
# 3. Page on-call engineer
```

### Manual Rollback
```bash
# Immediate rollback to 0%
hal9-migrate rollback --immediate --yes

# Or gradual rollback
hal9-migrate rollback --gradual --target 0 --duration 30m
```

### Post-Rollback Actions
1. Analyze failure:
   ```bash
   hal9-migrate analyze --failure --time-range "last 1h"
   ```

2. Create incident report:
   ```bash
   hal9-migrate report --incident --output incident-$(date +%Y%m%d).pdf
   ```

3. Schedule retrospective

## Troubleshooting

### High Error Rate
```bash
# Identify error patterns
hal9-migrate errors --analyze --group-by type,endpoint

# Check specific errors
hal9-migrate logs --errors --hierarchical --tail 100

# Common fixes:
# - Timeout adjustments
# - Retry configuration
# - Circuit breaker tuning
```

### Latency Spikes
```bash
# Find slow operations
hal9-migrate trace --slow-queries --threshold 100ms

# Analyze layer latencies
hal9-migrate latency --breakdown-by-layer

# Common causes:
# - Cold starts
# - Database queries
# - Network issues
```

### Traffic Imbalance
```bash
# Check actual vs expected traffic
hal9-migrate traffic --verify-split

# Rebalance if needed
hal9-migrate traffic --rebalance
```

## Success Criteria

Canary deployment is successful when:
1. ✅ All stages completed without rollback
2. ✅ 50% traffic stable for 24 hours
3. ✅ Error rate consistently < 0.1%
4. ✅ Performance meets or exceeds flat architecture
5. ✅ No data inconsistencies detected
6. ✅ Team confidence to proceed

## Notes Section
```
_________________________________________________________________
_________________________________________________________________
_________________________________________________________________
```

---

**Next Step**: After successful canary deployment, proceed to [State Migration](./04-state-migration.md)