# Full Cutover Runbook

**Version**: 1.0  
**Last Updated**: January 2025  
**Criticality**: CRITICAL  
**Estimated Time**: 2-4 hours  
**Risk Level**: LOW (extensive validation completed)

## Purpose

Complete the migration by routing 100% of traffic to the hierarchical architecture and safely decommissioning the flat architecture. This is the final step in the HAL9 hierarchical migration.

## Prerequisites

- Traffic ramp-up completed (99% for 12+ hours)
- All validations passed
- Team sign-offs obtained
- Communication plan activated
- Rollback plan reviewed and tested

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Last 1% reveals issues | Service degradation | Low | Extended 99% validation |
| Flat system dependencies | Feature loss | Very Low | Dependency scan completed |
| Rollback complexity | Extended downtime | Very Low | State preserved, procedures tested |
| Monitoring gaps | Delayed issue detection | Low | Comparison traffic maintained |

## Cutover Phases

| Phase | Duration | Actions | Checkpoint |
|-------|----------|---------|------------|
| Final Validation | 30 min | Last checks, team ready | All green |
| Traffic Cutover | 15 min | Route 100% to hierarchical | Traffic confirmed |
| Verification | 30 min | Validate full traffic | Metrics stable |
| Flat Decommission | 60 min | Scale down flat architecture | Resources freed |
| Final Confirmation | 30 min | Complete validation | Migration complete |

## Step-by-Step Procedures

### 1. Final Pre-Cutover Validation (30 min)

#### 1.1 System State Verification
```bash
# Final health check
hal9-migrate health --pre-cutover-final

# Expected output:
# ✅ Current Traffic: 99% hierarchical, 1% flat
# ✅ Error Rate (1h): 0.003%
# ✅ P99 Latency (1h): 8.2ms
# ✅ System Health: Optimal
# ✅ Resource Usage: 67% (33% headroom)
# ✅ Ready for cutover: YES
```

#### 1.2 Create Final Checkpoint
```bash
# Create point-of-no-return checkpoint
hal9-migrate state checkpoint \
  --name "pre-cutover-final-$(date +%Y%m%d-%H%M%S)" \
  --full \
  --include-flat-state \
  --verify

# Export configuration
hal9-migrate config export --all \
  --output final-config-backup.json
```

#### 1.3 Team Readiness Check
```bash
# Send readiness poll
hal9-migrate team notify \
  --message "Full cutover starting in 15 minutes" \
  --require-acknowledgment \
  --timeout 10m

# Verify all acknowledged
hal9-migrate team status
# Must show all team members ready
```

### 2. Communication (10 min)

#### 2.1 Stakeholder Notification
```bash
# Send cutover notification
echo "HAL9 hierarchical migration entering final cutover phase" | \
  mail -s "[HAL9] Final Cutover Starting" \
  -c operations@company.com \
  -c product@company.com \
  engineering@company.com

# Update status page
curl -X PUT https://status.company.com/api/v1/incidents/$INCIDENT_ID \
  -H "Authorization: Bearer $STATUS_PAGE_TOKEN" \
  -d '{
    "status": "monitoring",
    "message": "HAL9 migration entering final phase. No user impact expected."
  }'
```

#### 2.2 Start Recording
```bash
# Begin cutover recording for audit
hal9-migrate record start \
  --phase cutover \
  --include-metrics \
  --include-logs \
  --include-traces
```

### 3. Traffic Cutover (15 min)

#### 3.1 Route Final 1% Traffic
```bash
# Final traffic switch
hal9-migrate migrate --phase cutover \
  --percentage 100 \
  --gradual \
  --duration 5m \
  --monitor

# Watch traffic flow
watch -n 2 'hal9-migrate traffic --show-split --realtime'
```

**Wait for confirmation**: "Traffic: 100% hierarchical, 0% flat"

#### 3.2 Verify No Flat Traffic
```bash
# Confirm flat system receives no traffic
hal9-migrate traffic verify \
  --system flat \
  --expect zero

# Check flat system metrics
curl -s https://api.hal9.production.example.com/flat/metrics | \
  grep http_requests_total
# Should show no increase
```

#### 3.3 Disable Flat System Routes
```bash
# Remove flat system from load balancer
kubectl patch service hal9-gateway -n hal9 --type='json' \
  -p='[{"op": "remove", "path": "/spec/selector/version"}]'

# Verify routing table
hal9-migrate routes list --active
# Should only show hierarchical routes
```

### 4. Immediate Verification (30 min)

#### 4.1 Traffic Flow Validation
```bash
# Verify all traffic on hierarchical
hal9-migrate monitor --traffic-only --duration 5m

# Check request distribution
hal9-migrate traffic analyze \
  --last 5m \
  --by endpoint \
  --by layer
```

#### 4.2 Error Rate Monitoring
```bash
# Watch for any error spikes
hal9-migrate monitor --errors \
  --window 1m \
  --alert-threshold 0.001 &

# Sample real requests
hal9-migrate trace --sample 100 \
  --verify-hierarchical-only
```

#### 4.3 Performance Validation
```bash
# Compare with 99% baseline
hal9-migrate metrics compare \
  --baseline "99-percent" \
  --current "100-percent" \
  --duration 10m

# Key metrics:
# Error rate delta: < 0.01%
# Latency p99 delta: < 1ms
# Throughput delta: < 1%
```

### 5. Flat System Decommission (60 min)

#### 5.1 Preserve Flat System State
```bash
# Final flat system backup
hal9-migrate flat backup \
  --final \
  --include-logs \
  --compress \
  --output flat-final-backup.tar.gz

# Archive to cold storage
aws s3 cp flat-final-backup.tar.gz \
  s3://hal9-archives/migrations/flat-final/
```

#### 5.2 Gradual Shutdown
```bash
# Scale down flat system gradually
for replicas in 10 5 2 1 0; do
  echo "Scaling flat system to $replicas replicas"
  kubectl scale deployment hal9-flat --replicas=$replicas -n hal9
  
  # Wait for scale down
  sleep 30
  
  # Verify no errors
  hal9-migrate health --quick
done
```

#### 5.3 Resource Cleanup
```bash
# Stop flat system services
kubectl delete deployment hal9-flat -n hal9
kubectl delete service hal9-flat -n hal9
kubectl delete configmap hal9-flat-config -n hal9

# Verify resources freed
kubectl get all -n hal9 | grep flat
# Should return nothing
```

#### 5.4 Database Cleanup
```bash
# Archive flat system tables
hal9-migrate db archive-flat-tables \
  --target hal9_flat_archive \
  --maintain-read-access

# Optimize hierarchical tables
hal9-migrate db optimize \
  --system hierarchical \
  --vacuum \
  --analyze
```

### 6. Final System Configuration (30 min)

#### 6.1 Remove Migration Features
```bash
# Disable migration-specific features
hal9-migrate feature disable shadow_mode
hal9-migrate feature disable comparison_traffic
hal9-migrate feature disable dual_write

# Clean up feature flags
hal9-migrate feature cleanup --migration-related
```

#### 6.2 Update System Configuration
```bash
# Switch to production mode
hal9-migrate config set \
  --mode production \
  --architecture hierarchical \
  --remove-migration-endpoints

# Restart with clean config
hal9-migrate restart --clean-config
```

#### 6.3 Monitoring Adjustments
```bash
# Update monitoring for hierarchical-only
kubectl apply -f monitoring/hierarchical-only/

# Remove flat system dashboards
grafana-cli dashboard remove flat-system
grafana-cli dashboard remove migration-comparison

# Update alerts
kubectl apply -f monitoring/alerts/hierarchical-production.yaml
```

### 7. Final Validation (30 min)

#### 7.1 Complete System Test
```bash
# Run full production test suite
hal9-migrate test production-suite \
  --comprehensive \
  --hierarchical-only

# All tests must pass:
# ✅ API Tests: 1,234 passed
# ✅ Integration Tests: 567 passed
# ✅ Performance Tests: 89 passed
# ✅ Security Tests: 45 passed
```

#### 7.2 Business Validation
```bash
# Verify business metrics
hal9-cli business-metrics validate \
  --compare "pre-migration" \
  --significance 0.95

# Check critical KPIs
hal9-cli kpi check \
  --critical-only \
  --alert-on-degradation
```

#### 7.3 Sign-off Checklist
```
Final Cutover Completion
========================
Date: ________________
Time: ________________

Cutover Steps Completed:
[ ] Final validation passed
[ ] 100% traffic confirmed
[ ] Flat system decommissioned
[ ] Resources cleaned up
[ ] Monitoring updated
[ ] All tests passed

Final Metrics:
- Error Rate: _____% 
- P99 Latency: _____ ms
- Throughput: _____ req/s
- Active Neurons: _____
- Resource Usage: _____%

No issues detected: [ ] Yes [ ] No

Final Sign-offs:
Engineering Lead: ________________
Operations Lead: ________________
Product Owner: ________________
Executive Sponsor: ________________
```

### 8. Migration Completion

#### 8.1 Mark Migration Complete
```bash
# Official completion
hal9-migrate complete \
  --confirmed \
  --generate-certificate

# Stop recording
hal9-migrate record stop \
  --save-to migration-complete-record.tar.gz
```

#### 8.2 Update Documentation
```bash
# Generate final report
hal9-migrate report final \
  --comprehensive \
  --include-all-phases \
  --output hal9-migration-final-report.pdf

# Update system documentation
echo "Architecture: Hierarchical" > docs/ARCHITECTURE.md
echo "Migration Date: $(date)" >> docs/ARCHITECTURE.md
```

#### 8.3 Celebrate! 🎉
```bash
# Send success notification
echo "HAL9 hierarchical migration completed successfully!" | \
  mail -s "[HAL9] Migration Complete! 🎉" \
  -c all-hands@company.com \
  engineering@company.com

# Update status page
curl -X PUT https://status.company.com/api/v1/incidents/$INCIDENT_ID \
  -H "Authorization: Bearer $STATUS_PAGE_TOKEN" \
  -d '{
    "status": "resolved",
    "message": "HAL9 migration completed successfully. System operating normally on hierarchical architecture."
  }'
```

## Validation Summary

Migration is complete when:
1. ✅ 100% traffic on hierarchical
2. ✅ Flat system decommissioned
3. ✅ All resources cleaned up
4. ✅ Monitoring updated
5. ✅ Documentation updated
6. ✅ Team sign-offs complete

## Post-Cutover Monitoring

Continue enhanced monitoring for 48 hours:
```bash
# Set up post-migration monitoring
hal9-migrate monitor post-migration \
  --duration 48h \
  --enhanced \
  --comparison-baseline pre-migration
```

## Emergency Procedures

Even after cutover, maintain rollback capability for 7 days:
```bash
# If critical issues arise
hal9-migrate emergency restore \
  --from-archive \
  --checkpoint "pre-cutover-final-TIMESTAMP"
```

## Lessons Learned Session

Schedule within 1 week:
- What went well
- What could improve  
- Process refinements
- Tool enhancements
- Documentation updates

## Archive Migration Data

After 30 days of stable operation:
```bash
# Archive migration data
hal9-migrate archive \
  --all-migration-data \
  --compress \
  --move-to-cold-storage
```

---

**Next Step**: Proceed to [Post-Migration Validation](./07-post-migration-validation.md) for ongoing monitoring and optimization.