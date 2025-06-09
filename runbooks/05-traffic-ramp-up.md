# Traffic Ramp-Up Runbook

**Version**: 1.0  
**Last Updated**: January 2025  
**Criticality**: HIGH  
**Estimated Time**: 24-48 hours  
**Risk Level**: MEDIUM (increasing production exposure)

## Purpose

Gradually increase traffic from 50% to 100% on the hierarchical architecture. This phase validates system stability under full production load before complete cutover.

## Prerequisites

- State migration completed successfully
- System stable at 50% traffic for 24+ hours
- All data reconciliation complete
- Performance metrics acceptable
- Rollback procedures reviewed

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Performance degradation | User experience | Medium | Gradual ramp with monitoring |
| Resource exhaustion | System stability | Low | Auto-scaling configured |
| Hidden bottlenecks | Latency spikes | Medium | Load testing validation |
| Cascade failures | Partial outage | Low | Circuit breakers active |

## Ramp-Up Stages

| Stage | Traffic % | Duration | Success Criteria |
|-------|-----------|----------|------------------|
| 1 | 60% | 4 hours | Error rate < 0.1%, p99 < 10ms |
| 2 | 70% | 4 hours | Error rate < 0.1%, p99 < 10ms |
| 3 | 80% | 8 hours | Error rate < 0.1%, p99 < 10ms |
| 4 | 90% | 8 hours | Error rate < 0.1%, p99 < 10ms |
| 5 | 95% | 12 hours | Error rate < 0.1%, p99 < 10ms |
| 6 | 99% | 12 hours | Error rate < 0.1%, p99 < 10ms |

## Step-by-Step Procedures

### 1. Pre-Ramp Validation (1 hour)

#### 1.1 System Health Check
```bash
# Comprehensive health check
hal9-migrate health --all-systems --deep

# Verify state migration stability
hal9-migrate state verify --post-migration
# Should show: "State drift: 0.00%"

# Check resource headroom
hal9-migrate resources --forecast 100
# Must have 30% headroom for all resources
```

#### 1.2 Performance Baseline
```bash
# Capture current performance
hal9-migrate metrics baseline \
  --traffic-level 50 \
  --duration 1h \
  --save ramp-baseline.json

# Key metrics to record:
# - P50 latency: _____ ms
# - P99 latency: _____ ms
# - Error rate: _____ %
# - CPU usage: _____ %
# - Memory usage: _____ %
```

#### 1.3 Configure Auto-Scaling
```bash
# Update auto-scaling for higher traffic
kubectl apply -f - <<EOF
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: hal9-hierarchical
  namespace: hal9
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: hal9-hierarchical
  minReplicas: 20
  maxReplicas: 100
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 60
EOF
```

### 2. Stage 1: 60% Traffic (4 hours)

#### 2.1 Increase Traffic to 60%
```bash
# Gradual increase over 10 minutes
hal9-migrate migrate --phase ramp-up \
  --percentage 60 \
  --gradual \
  --duration 10m

# Monitor the increase
watch -n 5 'hal9-migrate traffic --show-split --realtime'
```

#### 2.2 Initial Monitoring (First Hour)
```bash
# Real-time dashboard
hal9-migrate monitor --dashboard ramp-up

# Watch key metrics in separate terminals:
# Terminal 1: Error rate
watch -n 2 'hal9-migrate metrics --metric error-rate --window 5m'

# Terminal 2: Latency
watch -n 2 'hal9-migrate metrics --metric latency --percentiles p50,p90,p99'

# Terminal 3: Resource usage
watch -n 5 'hal9-migrate resources --usage --pods'
```

#### 2.3 Stability Verification
```bash
# After 4 hours, verify stability
hal9-migrate verify --stage "ramp-60" --duration 4h

# Generate stage report
hal9-migrate report --stage "ramp-60" \
  --compare-baseline ramp-baseline.json
```

### 3. Stage 2: 70% Traffic (4 hours)

#### 3.1 Increase to 70%
```bash
# Continue ramp-up
hal9-migrate migrate --phase ramp-up \
  --percentage 70 \
  --gradual \
  --duration 10m

# Verify scaling triggered if needed
kubectl get hpa hal9-hierarchical -n hal9 --watch
```

#### 3.2 Load Distribution Check
```bash
# Verify even distribution across layers
hal9-migrate traffic analyze \
  --distribution \
  --by-layer \
  --by-region

# Check for hot spots
hal9-migrate neurons hot-spots --threshold 80
```

### 4. Stage 3: 80% Traffic (8 hours)

#### 4.1 Increase to 80%
```bash
# Major traffic milestone
hal9-migrate migrate --phase ramp-up \
  --percentage 80 \
  --gradual \
  --duration 15m

# Enable enhanced monitoring
hal9-migrate monitor --enhanced \
  --alert-sensitivity high
```

#### 4.2 Peak Load Testing
```bash
# Simulate peak hour patterns
hal9-migrate test peak-load \
  --multiplier 1.5 \
  --duration 1h \
  --traffic-percentage 80

# Monitor during test
hal9-migrate monitor --peak-test
```

#### 4.3 Extended Validation
```bash
# Run comprehensive tests at 80%
hal9-migrate validate --comprehensive \
  --at-percentage 80 \
  --include-stress-tests

# Business metrics validation
hal9-cli business-metrics \
  --compare-baseline \
  --alert-on-degradation
```

### 5. Stage 4: 90% Traffic (8 hours)

#### 5.1 Increase to 90%
```bash
# Near-full traffic
hal9-migrate migrate --phase ramp-up \
  --percentage 90 \
  --gradual \
  --duration 15m \
  --checkpoint-before
```

#### 5.2 Flat Architecture Wind-Down
```bash
# Prepare flat architecture for minimal traffic
hal9-migrate flat-system scale-down \
  --target 10 \
  --keep-minimum

# Monitor flat system health
hal9-migrate health --system flat
```

#### 5.3 Full System Validation
```bash
# Validate entire system at 90%
hal9-migrate validate --full-system \
  --traffic-level 90 \
  --duration 2h

# Check all integrations
hal9-migrate test integrations --all
```

### 6. Stage 5: 95% Traffic (12 hours)

#### 6.1 Increase to 95%
```bash
# Near-complete migration
hal9-migrate migrate --phase ramp-up \
  --percentage 95 \
  --gradual \
  --duration 20m

# Set strict monitoring
hal9-migrate monitor --strict \
  --page-on-anomaly
```

#### 6.2 Edge Case Testing
```bash
# Test rare scenarios
hal9-migrate test edge-cases \
  --scenarios all \
  --at-percentage 95

# Test failure recovery
hal9-migrate test failure-recovery \
  --inject-failures 5 \
  --verify-recovery
```

#### 6.3 Extended Stability
```bash
# 12-hour stability test
hal9-migrate monitor --continuous \
  --duration 12h \
  --record-for-analysis

# Generate stability report
hal9-migrate report --stability \
  --traffic-level 95 \
  --duration 12h
```

### 7. Stage 6: 99% Traffic (12 hours)

#### 7.1 Near-Complete Cutover
```bash
# Final ramp stage
hal9-migrate migrate --phase ramp-up \
  --percentage 99 \
  --gradual \
  --duration 20m

# Keep 1% on flat for comparison
hal9-migrate configure comparison-traffic \
  --percentage 1 \
  --purpose monitoring
```

#### 7.2 Production Readiness
```bash
# Full production validation
hal9-migrate validate --production-ready \
  --checklist all \
  --sign-off-required

# Performance certification
hal9-migrate certify performance \
  --sla-compliance \
  --generate-certificate
```

#### 7.3 Final Preparations
```bash
# Prepare for full cutover
hal9-migrate prepare cutover \
  --checklist \
  --notifications \
  --rollback-ready

# Document current state
hal9-migrate state export \
  --full \
  --output pre-cutover-state.json
```

### 8. Ramp-Up Completion

#### 8.1 Final Validation
```bash
# Comprehensive final check
hal9-migrate validate --phase ramp-complete

# Success criteria:
# ✅ All stages completed
# ✅ 99% traffic stable for 12 hours
# ✅ Error rate < 0.1%
# ✅ Performance SLA met
# ✅ No manual interventions required
# ✅ Auto-scaling working correctly
```

#### 8.2 Team Sign-Off
```
Ramp-Up Completion Checklist
============================
Date: ________________
Time: ________________

Traffic Stages Completed:
[ ] 60% - Duration: _____ Status: _____
[ ] 70% - Duration: _____ Status: _____
[ ] 80% - Duration: _____ Status: _____
[ ] 90% - Duration: _____ Status: _____
[ ] 95% - Duration: _____ Status: _____
[ ] 99% - Duration: _____ Status: _____

Performance Metrics:
- Peak Error Rate: _____% 
- Peak P99 Latency: _____ ms
- Resource Usage Peak: _____%
- Auto-scaling Events: _____

Sign-offs:
Engineering Lead: ________________
Operations Lead: ________________
Product Owner: ________________
Executive Sponsor: ________________
```

## Validation Steps

At each stage:
1. Error rate remains < 0.1%
2. P99 latency < 10ms
3. No sustained alerts
4. Resource usage < 85%
5. Auto-scaling responsive
6. User metrics stable

## Rollback Procedures

### Partial Traffic Rollback
```bash
# Reduce traffic to last stable level
hal9-migrate rollback --to-percentage 50 \
  --reason "Performance degradation at X%"
```

### Emergency Stop
```bash
# Immediate return to 50%
hal9-migrate rollback --emergency \
  --target-percentage 50 \
  --preserve-state
```

## Troubleshooting

### Performance Degradation
```bash
# Identify bottlenecks
hal9-migrate analyze --bottlenecks \
  --at-percentage current

# Common solutions:
# - Scale specific layers
# - Optimize slow queries
# - Adjust caching
# - Review resource limits
```

### Uneven Traffic Distribution
```bash
# Rebalance traffic
hal9-migrate traffic rebalance \
  --algorithm consistent-hash \
  --verify
```

### Resource Constraints
```bash
# Emergency scaling
kubectl scale deployment hal9-hierarchical \
  --replicas=50 \
  -n hal9

# Add node capacity if needed
```

## Success Criteria

Ramp-up is successful when:
1. ✅ 99% traffic stable for 12+ hours
2. ✅ Performance meets or exceeds flat architecture
3. ✅ No manual interventions in final 24 hours
4. ✅ Auto-scaling handling load changes
5. ✅ All validations pass
6. ✅ Team confidence for full cutover

## Notes Section

```
_________________________________________________________________
_________________________________________________________________
_________________________________________________________________
_________________________________________________________________
```

---

**Next Step**: After successful ramp-up to 99%, proceed to [Full Cutover](./06-full-cutover.md)