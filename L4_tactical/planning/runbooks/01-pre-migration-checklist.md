# Pre-Migration Checklist Runbook

**Version**: 1.0  
**Last Updated**: January 2025  
**Criticality**: HIGH  
**Estimated Time**: 2-4 hours

## Purpose

This runbook ensures all prerequisites are met before initiating the HAL9 hierarchical migration. Completing this checklist reduces migration risks and ensures readiness.

## Prerequisites

- Access to production systems
- Migration operator permissions
- `hal9-migrate` CLI tool installed (v1.0+)
- Grafana monitoring access
- Communication channels ready

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Incomplete backup | Data loss | Low | Verify backup completion |
| High system load | Performance degradation | Medium | Schedule during low traffic |
| Missing dependencies | Migration failure | Low | Validate all components |
| Communication failure | Confusion/delays | Medium | Test all channels |

## Step-by-Step Procedures

### 1. System Health Verification (30 min)

#### 1.1 Check Overall System Health
```bash
# Run comprehensive health check
hal9-migrate pre-check --deep

# Expected output:
# ✅ Substrate Layer: Healthy
# ✅ Protocol Layer: Healthy
# ✅ Cognitive Layer: Healthy
# ✅ Orchestration Layer: Healthy
# ✅ Intelligence Layer: Healthy
```

**Decision Point**: If any layer is unhealthy, DO NOT PROCEED. Fix issues first.

#### 1.2 Verify Resource Availability
```bash
# Check resource usage
hal9-cli status --resources

# Ensure:
# - CPU usage < 70%
# - Memory usage < 80%
# - Disk space > 100GB free
# - Network latency < 10ms between nodes
```

#### 1.3 Check Database Health
```bash
# PostgreSQL health
psql -h $DB_HOST -U $DB_USER -d hal9 -c "SELECT version();"
psql -h $DB_HOST -U $DB_USER -d hal9 -c "SELECT pg_database_size('hal9');"

# Verify replication lag (if applicable)
psql -h $DB_HOST -U $DB_USER -d hal9 -c "SELECT * FROM pg_stat_replication;"
```

### 2. Backup Verification (45 min)

#### 2.1 Create Fresh Backup
```bash
# Trigger full system backup
./scripts/backup-production.sh

# Verify backup completed
aws s3 ls s3://hal9-backups/production/$(date +%Y-%m-%d)/
```

#### 2.2 Validate Backup Integrity
```bash
# Test restore to validation environment
./scripts/validate-backup.sh --date $(date +%Y-%m-%d)

# Expected: "Backup validation successful"
```

#### 2.3 Document Backup Location
```
Backup Date: ________________
Backup Location: s3://hal9-backups/production/____________
Validation Status: [ ] Passed
Operator Signature: ________________
```

### 3. Dependency Verification (20 min)

#### 3.1 Check Migration Infrastructure
```bash
# Verify migration API is accessible
curl -s https://api.hal9.production.example.com/migration/health | jq .

# Check feature flag system
hal9-migrate feature list

# Verify traffic router
curl -s https://api.hal9.production.example.com/router/status | jq .
```

#### 3.2 Validate Monitoring Systems
- [ ] Grafana accessible: https://grafana.production.example.com
- [ ] Prometheus healthy: https://prometheus.production.example.com
- [ ] Alertmanager configured: https://alertmanager.production.example.com
- [ ] Log aggregation working: https://logs.production.example.com

#### 3.3 Check External Dependencies
```bash
# Claude API connectivity
hal9-cli test claude-api

# MCP tools availability (if enabled)
hal9-cli test mcp-tools

# Network connectivity to all nodes
for node in node1 node2 node3; do
  ping -c 3 $node.hal9.internal
done
```

### 4. Team Readiness (15 min)

#### 4.1 Communication Channels
- [ ] Slack #hal9-migration channel active
- [ ] PagerDuty escalation tested
- [ ] War room scheduled (if needed)
- [ ] Stakeholders notified

#### 4.2 Personnel Availability
```
Primary Operator: ________________ (confirmed available)
Secondary Operator: ________________ (confirmed available)
On-call Engineer: ________________ (confirmed available)
Escalation Contact: ________________ (confirmed available)
```

#### 4.3 Access Verification
Each operator must verify:
```bash
# Test production access
ssh production-bastion
hal9-migrate status

# Verify sudo access (if needed)
sudo -l
```

### 5. Migration Plan Review (30 min)

#### 5.1 Review Migration Phases
- [ ] Shadow mode plan reviewed
- [ ] Canary percentages agreed (5% → 10% → 25% → 50%)
- [ ] State migration timing confirmed
- [ ] Full cutover schedule approved

#### 5.2 Review Rollback Procedures
- [ ] Rollback runbook location known
- [ ] Rollback tested in staging
- [ ] Rollback decision criteria clear
- [ ] Emergency contacts verified

#### 5.3 Review Success Criteria
- [ ] Error rate threshold: < 0.1%
- [ ] Latency p99 threshold: < 10ms
- [ ] Zero data loss tolerance confirmed
- [ ] Monitoring dashboard bookmarked

### 6. Final Pre-Flight Checks (15 min)

#### 6.1 Create Migration Checkpoint
```bash
# Create pre-migration checkpoint
hal9-migrate state checkpoint \
  --name "pre-migration-$(date +%Y%m%d-%H%M%S)" \
  --description "Checkpoint before hierarchical migration"

# Document checkpoint ID
echo "Checkpoint ID: ________________"
```

#### 6.2 Disable Non-Critical Jobs
```bash
# Pause batch jobs
kubectl scale deployment batch-processor --replicas=0 -n hal9

# Disable non-critical cron jobs
crontab -l > crontab.backup
crontab -r
```

#### 6.3 Clear Alert Silence
```bash
# Ensure no alerts are silenced
curl -X GET https://alertmanager.production.example.com/api/v1/silences | jq .

# If any found, review and remove if appropriate
```

### 7. Final Approval

**ALL ITEMS MUST BE CHECKED BEFORE PROCEEDING**

- [ ] All system health checks passed
- [ ] Backup verified and location documented  
- [ ] All dependencies validated
- [ ] Team ready and available
- [ ] Migration plan reviewed and understood
- [ ] Rollback procedures reviewed
- [ ] No critical alerts active
- [ ] Change ticket approved: #________________

**Migration Authorization**
```
Primary Operator: ________________ Date: ________ Time: ________
Secondary Operator: ________________ Date: ________ Time: ________
Management Approval: ________________ Date: ________ Time: ________
```

## Validation Steps

After completing checklist:
1. Export checklist state:
   ```bash
   hal9-migrate state export --output pre-migration-state.json
   ```

2. Verify readiness score:
   ```bash
   hal9-migrate pre-check --score
   # Must show: "Migration Readiness: 100%"
   ```

## Rollback Procedures

If any check fails:
1. Document failure reason in migration ticket
2. Address the issue
3. Re-run the entire checklist
4. Do not proceed until all checks pass

## Troubleshooting

### Common Issues

#### Health Check Failures
```bash
# Get detailed health status
hal9-migrate pre-check --deep --verbose

# Check specific component
hal9-migrate pre-check --components substrate
```

#### Backup Failures
```bash
# Check backup logs
tail -f /var/log/hal9/backup.log

# Verify S3 permissions
aws s3 ls s3://hal9-backups/ --profile production
```

#### Resource Constraints
```bash
# Find resource hogs
htop
kubectl top pods -n hal9 --sort-by=cpu
kubectl top pods -n hal9 --sort-by=memory
```

## Notes Section

Use this space to document any special considerations:
```
_________________________________________________________________
_________________________________________________________________
_________________________________________________________________
_________________________________________________________________
```

---

**Next Step**: Proceed to [Shadow Mode Deployment](./02-shadow-mode-deployment.md)