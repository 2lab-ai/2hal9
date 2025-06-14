# Emergency Rollback Procedures

**Version**: 1.0  
**Last Updated**: January 2025  
**Criticality**: CRITICAL  
**Estimated Time**: 5-30 minutes depending on phase  
**Risk Level**: LOW (rollback is safe)

## Purpose

This runbook provides procedures for rolling back the HAL9 hierarchical migration at any phase. Rollback procedures are designed to be safe, fast, and preserve data integrity.

## When to Rollback

### Automatic Triggers
- Error rate > 0.1% for 2 minutes
- P99 latency > 50ms for 5 minutes  
- P50 latency > 15ms for 5 minutes
- Health check failures > 3 consecutive
- Memory usage > 95%
- CPU usage > 95% for 5 minutes

### Manual Triggers
- User complaints spike
- Data inconsistency detected
- Unexpected behavior observed
- Business metrics degraded
- Team consensus to rollback

## Quick Rollback Commands

### EMERGENCY - Immediate Full Rollback
```bash
# STOPS ALL HIERARCHICAL TRAFFIC IMMEDIATELY
hal9-migrate rollback --emergency --force
```

### Standard Rollback by Phase

**Shadow Mode**:
```bash
hal9-migrate rollback --from-phase shadow
```

**Canary**:
```bash
hal9-migrate rollback --from-phase canary --percentage 0
```

**State Migration**:
```bash
hal9-migrate rollback --from-phase state-migration --restore-checkpoint
```

**Traffic Ramp**:
```bash
hal9-migrate rollback --from-phase ramp-up --target-percentage 0
```

**Full Migration**:
```bash
hal9-migrate rollback --from-phase full --complete
```

## Detailed Rollback Procedures

### 1. Assess Situation (2 min)

#### 1.1 Identify Trigger
```bash
# Check what triggered rollback need
hal9-migrate status --alerts
hal9-migrate metrics --anomalies

# Check monitoring
open https://grafana.production.example.com/d/hal9-migration
```

#### 1.2 Determine Rollback Type
- **Partial**: Reduce traffic percentage
- **Full**: Complete rollback to flat architecture
- **Emergency**: Immediate cessation of hierarchical processing

#### 1.3 Notify Team
```bash
# Page on-call if not already paged
pagerduty-cli incident create --service hal9 --title "Migration Rollback Required"

# Post to Slack
echo "Initiating HAL9 migration rollback. Trigger: $TRIGGER" | \
  slack-cli post --channel "#hal9-migration" --alert
```

### 2. Execute Rollback

#### 2.1 Shadow Mode Rollback (Safest)
```bash
# Disable shadow mode
hal9-migrate feature disable shadow_mode

# Verify disabled
hal9-migrate status
# Should show: Phase: none

# Clean up shadow resources
kubectl scale deployment hal9-shadow --replicas=0 -n hal9
```

#### 2.2 Canary Rollback (Quick)
```bash
# Option 1: Gradual rollback (recommended)
hal9-migrate rollback --from-phase canary --gradual --duration 10m

# Option 2: Immediate rollback
hal9-migrate rollback --from-phase canary --immediate

# Monitor rollback progress
watch -n 2 'hal9-migrate traffic --show-split'
```

#### 2.3 State Migration Rollback (Complex)
```bash
# STOP state migration immediately
hal9-migrate state-migration --stop

# List available checkpoints
hal9-migrate state list-checkpoints

# Restore from checkpoint
hal9-migrate state restore --checkpoint "pre-state-migration-TIMESTAMP"

# Verify restoration
hal9-migrate state verify --deep
```

#### 2.4 Traffic Ramp Rollback
```bash
# Reduce traffic to safe level or zero
hal9-migrate migrate --phase ramp-up --percentage 0 --immediate

# Or rollback to canary level
hal9-migrate migrate --phase canary --percentage 5
```

#### 2.5 Full Migration Rollback
```bash
# Complete rollback to flat architecture
hal9-migrate rollback --complete --preserve-data

# This will:
# 1. Route 100% traffic to flat
# 2. Preserve hierarchical state
# 3. Maintain feature flags
# 4. Keep monitoring active
```

### 3. Verify Rollback Success (5 min)

#### 3.1 Traffic Verification
```bash
# Confirm all traffic on flat architecture
hal9-migrate traffic --verify
# Should show: Flat: 100%, Hierarchical: 0%

# Check no hierarchical processing
hal9-migrate monitor --system hierarchical
# Should show: No active requests
```

#### 3.2 System Health Check
```bash
# Verify system health restored
hal9-migrate health --all-systems

# Check error rates returning to normal
watch -n 5 'hal9-migrate metrics --error-rate --window 5m'
```

#### 3.3 User Impact Assessment
```bash
# Check user-facing metrics
hal9-cli ux-metrics --post-rollback

# Monitor for user complaints
tail -f /var/log/hal9/user-feedback.log
```

### 4. Post-Rollback Actions (10 min)

#### 4.1 Create Incident Report
```bash
# Generate automated report
hal9-migrate report --rollback \
  --reason "$ROLLBACK_REASON" \
  --output "rollback-$(date +%Y%m%d-%H%M%S).pdf"

# Include:
# - Trigger event
# - Timeline
# - Impact assessment
# - Root cause hypothesis
```

#### 4.2 Preserve Evidence
```bash
# Capture system state
hal9-migrate debug --capture-all \
  --output /data/rollback-evidence/$(date +%Y%m%d-%H%M%S)/

# Export metrics
hal9-migrate metrics --export \
  --time-range "last 2h" \
  --output rollback-metrics.json

# Save logs
hal9-migrate logs --export \
  --systems all \
  --time-range "last 2h" \
  --output rollback-logs.tar.gz
```

#### 4.3 Update Status
```bash
# Update status page
curl -X PUT https://status.company.com/api/v1/incidents/$INCIDENT_ID \
  -H "Authorization: Bearer $STATUS_PAGE_TOKEN" \
  -d '{"status": "monitoring", "message": "HAL9 migration rolled back, monitoring stability"}'

# Update ticket
jira-cli issue update HAL9-MIGRATION \
  --status "Rolled Back" \
  --comment "Rollback executed at $(date). Reason: $ROLLBACK_REASON"
```

### 5. Stabilization Period (30 min)

#### 5.1 Extended Monitoring
```bash
# Monitor for 30 minutes post-rollback
hal9-migrate monitor --post-rollback --duration 30m

# Watch for:
# - Error rate stabilization
# - Latency normalization  
# - Resource usage settling
# - No new alerts
```

#### 5.2 Gradual Stand-Down
After 30 minutes of stability:
1. Reduce monitoring intensity
2. Cancel war room if active
3. Update stakeholders
4. Schedule retrospective

## Rollback Decision Matrix

| Scenario | Rollback Type | Urgency | Authority Needed |
|----------|---------------|---------|------------------|
| Error rate > 1% | Emergency | Immediate | Any operator |
| Error rate > 0.1% | Full | 2 min | Any operator |
| Latency p99 > 100ms | Emergency | Immediate | Any operator |
| Latency p99 > 50ms | Full | 5 min | Any operator |
| Data inconsistency | Emergency + Restore | Immediate | Lead + Manager |
| User complaints | Gradual | 10 min | Lead approval |
| Performance degradation | Partial | 15 min | Team consensus |

## Special Scenarios

### Partial System Failure
```bash
# Rollback only affected components
hal9-migrate rollback --components cognitive,orchestration

# Keep working components
hal9-migrate rollback --exclude substrate,protocol
```

### Geographic Rollback
```bash
# Rollback specific regions
hal9-migrate rollback --regions us-west-2,eu-west-1

# Keep successful regions
hal9-migrate rollback --exclude-regions ap-southeast-1
```

### Feature-Specific Rollback
```bash
# Disable specific features
hal9-migrate feature disable hierarchical_learning
hal9-migrate feature disable meta_cognition

# Keep basic hierarchical routing
```

## Recovery Procedures

After successful rollback:

### 1. Root Cause Analysis
```bash
# Analyze failure
hal9-migrate analyze --failure \
  --time-range "rollback -1h to rollback +30m" \
  --deep

# Common issues:
# - Resource constraints
# - Timeout configurations
# - Dependency failures
# - Code bugs
```

### 2. Fix Issues
1. Identify root cause
2. Implement fix in staging
3. Test thoroughly
4. Get additional review

### 3. Prepare for Retry
```bash
# Reset migration state
hal9-migrate reset --keep-learnings

# Create new migration plan
hal9-migrate plan --conservative \
  --based-on-failure \
  --output retry-plan.yaml
```

## Communication Templates

### Rollback Initiated
```
Subject: [HAL9] Migration Rollback Initiated

Team,

We are initiating a rollback of the HAL9 hierarchical migration due to:
[REASON]

Current Status: Rollback in progress
Expected Duration: [DURATION]
User Impact: [IMPACT]

Updates will follow every 15 minutes.

- [OPERATOR NAME]
```

### Rollback Complete
```
Subject: [HAL9] Migration Rollback Complete

Team,

The HAL9 migration rollback is complete.

Summary:
- Rollback Started: [TIME]
- Rollback Completed: [TIME]
- Total Duration: [DURATION]
- User Impact: [IMPACT]
- Root Cause: Under investigation

Next Steps:
1. Retrospective scheduled for [DATE/TIME]
2. Root cause analysis in progress
3. System monitoring continues

- [OPERATOR NAME]
```

## Important Notes

1. **Rollback is always safe** - Designed for zero data loss
2. **Don't hesitate** - Better to rollback early than risk users
3. **Preserve evidence** - Critical for learning
4. **Communicate often** - Keep everyone informed
5. **Learn and improve** - Each rollback makes next attempt better

---

**Remember**: A successful rollback is a win - it means our safety systems work!