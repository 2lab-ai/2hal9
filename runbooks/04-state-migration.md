# State Migration Runbook

**Version**: 1.0  
**Last Updated**: January 2025  
**Criticality**: CRITICAL  
**Estimated Time**: 4-8 hours  
**Risk Level**: HIGH (data consistency critical)

## Purpose

Migrate persistent state from flat architecture to hierarchical architecture. This is the most critical phase as it involves data transformation and ensures consistency between both systems.

## Prerequisites

- Canary deployment stable at 50% for 24+ hours
- State migration tools tested in staging
- Database backup completed within 1 hour
- All team members present
- Maintenance window scheduled (if required)

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Data corruption | Critical | Low | Incremental migration with validation |
| State inconsistency | High | Medium | Continuous reconciliation |
| Performance impact | Medium | Medium | Throttled migration |
| Rollback complexity | High | Low | Checkpoint-based recovery |

## State Categories

| Category | Volume | Complexity | Migration Order |
|----------|--------|------------|-----------------|
| User Sessions | ~100K | Low | 1 |
| Neuron States | ~1M | High | 2 |
| Learning Data | ~10GB | Medium | 3 |
| Configuration | ~1K | Low | 4 |
| Operational Metadata | ~100MB | Low | 5 |

## Step-by-Step Procedures

### 1. Pre-Migration Setup (1 hour)

#### 1.1 Create Migration Checkpoint
```bash
# Create full state backup
hal9-migrate state checkpoint \
  --name "pre-state-migration-$(date +%Y%m%d-%H%M%S)" \
  --full-backup \
  --verify

# Export checkpoint manifest
hal9-migrate state export-manifest \
  --checkpoint latest \
  --output state-manifest.json
```

#### 1.2 Verify State Consistency
```bash
# Run consistency check between systems
hal9-migrate state verify --deep \
  --compare flat,hierarchical \
  --report consistency-report.json

# Expected output:
# ✅ User Sessions: 100% consistent
# ✅ Neuron States: 100% consistent
# ✅ Learning Data: 100% consistent
# ✅ Configuration: 100% consistent
# ✅ Metadata: 100% consistent
```

#### 1.3 Configure Migration Parameters
```bash
# Set migration rate limits
hal9-migrate configure state-migration \
  --batch-size 1000 \
  --rate-limit 100/s \
  --parallel-workers 4 \
  --validation-mode strict

# Configure reconciliation
hal9-migrate configure reconciliation \
  --interval 30s \
  --auto-fix true \
  --alert-threshold 0.01
```

#### 1.4 Enable Migration Mode
```bash
# Put system in migration mode
hal9-migrate mode migration --enable

# This will:
# - Enable dual-write mode
# - Start state reconciliation
# - Begin audit logging
# - Pause non-critical operations
```

### 2. User Session Migration (1 hour)

#### 2.1 Start Session Migration
```bash
# Migrate active sessions first
hal9-migrate state migrate \
  --category user-sessions \
  --filter "active=true" \
  --priority high

# Monitor progress
watch -n 5 'hal9-migrate state progress --category user-sessions'
```

#### 2.2 Validate Session Integrity
```bash
# Verify session continuity
hal9-migrate state validate \
  --category user-sessions \
  --tests continuity,integrity,performance

# Check user experience
hal9-cli ux-test --sample-users 100 --verify-sessions
```

#### 2.3 Migrate Historical Sessions
```bash
# Migrate inactive sessions
hal9-migrate state migrate \
  --category user-sessions \
  --filter "active=false" \
  --batch-mode \
  --low-priority
```

### 3. Neuron State Migration (2 hours)

#### 3.1 Prepare Neuron Migration
```bash
# Snapshot current neuron states
hal9-migrate neurons snapshot --pre-migration

# Group neurons by layer
hal9-migrate neurons group \
  --by layer \
  --output neuron-groups.json
```

#### 3.2 Migrate Layer by Layer
```bash
# Migrate from bottom up
for layer in substrate protocol cognitive orchestration intelligence; do
  echo "=== Migrating $layer neurons ==="
  
  hal9-migrate state migrate \
    --category neuron-states \
    --filter "layer=$layer" \
    --validate-each \
    --checkpoint-interval 1000
    
  # Verify layer migration
  hal9-migrate neurons verify --layer $layer
done
```

#### 3.3 Validate Neuron Connectivity
```bash
# Verify neuron connections preserved
hal9-migrate neurons validate-connections \
  --deep \
  --fix-orphaned

# Check signal propagation
hal9-migrate neurons test-signals \
  --samples 1000 \
  --verify-paths
```

### 4. Learning Data Migration (2 hours)

#### 4.1 Export Learning Patterns
```bash
# Export from flat architecture
hal9-migrate learning export \
  --system flat \
  --format hierarchical \
  --output learning-export/

# Verify export completeness
du -sh learning-export/
ls -la learning-export/ | wc -l
```

#### 4.2 Transform and Import
```bash
# Transform learning data to hierarchical format
hal9-migrate learning transform \
  --input learning-export/ \
  --output learning-hierarchical/ \
  --validate-schemas

# Import to hierarchical system
hal9-migrate learning import \
  --input learning-hierarchical/ \
  --system hierarchical \
  --preserve-timestamps \
  --batch-size 10000
```

#### 4.3 Verify Learning Continuity
```bash
# Test pattern recognition
hal9-migrate learning test \
  --patterns 100 \
  --compare-systems

# Verify gradient calculations
hal9-migrate learning verify-gradients \
  --sample-size 1000
```

### 5. Configuration Migration (30 min)

#### 5.1 Migrate System Configuration
```bash
# Export all configuration
hal9-migrate config export --all \
  --output config-backup.json

# Migrate to hierarchical
hal9-migrate state migrate \
  --category configuration \
  --atomic \
  --verify-each
```

#### 5.2 Validate Configuration
```bash
# Compare configurations
hal9-migrate config compare \
  --system1 flat \
  --system2 hierarchical \
  --report config-diff.txt

# Test configuration hot-reload
hal9-migrate config test-reload \
  --changes 10 \
  --verify-application
```

### 6. Metadata Migration (30 min)

#### 6.1 Migrate Operational Metadata
```bash
# Migrate metrics metadata
hal9-migrate state migrate \
  --category metadata \
  --type metrics,logs,traces

# Migrate audit logs
hal9-migrate state migrate \
  --category metadata \
  --type audit \
  --preserve-order
```

### 7. State Reconciliation (1 hour)

#### 7.1 Run Full Reconciliation
```bash
# Start reconciliation process
hal9-migrate reconcile --full \
  --fix-mode auto \
  --report reconcile-report.json

# Monitor reconciliation
tail -f /var/log/hal9/reconciliation.log
```

#### 7.2 Verify Data Integrity
```bash
# Deep integrity check
hal9-migrate state verify \
  --all-categories \
  --checksums \
  --cross-reference

# Should show:
# Total Records: 1,234,567
# Verified: 1,234,567 (100%)
# Mismatches: 0
# Missing: 0
# Extra: 0
```

#### 7.3 Performance Validation
```bash
# Test state access performance
hal9-migrate benchmark \
  --operations read,write,update \
  --categories all \
  --compare-systems

# Ensure hierarchical performance >= flat
```

### 8. Finalize Migration (30 min)

#### 8.1 Disable Dual-Write
```bash
# Switch to hierarchical-only writes
hal9-migrate mode migration --disable-dual-write

# Verify single write mode
hal9-migrate state status
# Should show: "Write Mode: hierarchical-only"
```

#### 8.2 Create Post-Migration Checkpoint
```bash
# Checkpoint completed state
hal9-migrate state checkpoint \
  --name "post-state-migration-$(date +%Y%m%d-%H%M%S)" \
  --description "State migration completed"
```

#### 8.3 Update Migration Status
```bash
# Mark state migration complete
hal9-migrate phase complete state-migration

# Generate migration report
hal9-migrate report \
  --phase state-migration \
  --detailed \
  --output state-migration-report.pdf
```

## Validation Steps

### Comprehensive Validation Suite
```bash
# Run full validation
hal9-migrate validate --phase state-migration --comprehensive

# Must pass all checks:
# ✅ Data Integrity: 100%
# ✅ State Consistency: No issues
# ✅ Performance: Meets SLA
# ✅ User Sessions: Active
# ✅ Learning Continuity: Verified
# ✅ Configuration: Applied
# ✅ Metadata: Complete
```

### Manual Verification Checklist
- [ ] Spot check 10 random user sessions
- [ ] Verify neuron activation patterns
- [ ] Test learning on known patterns
- [ ] Confirm configuration changes apply
- [ ] Check audit log continuity
- [ ] Validate metrics collection

## Rollback Procedures

### Quick State Rollback
```bash
# EMERGENCY: Revert to checkpoint
hal9-migrate state rollback \
  --to-checkpoint "pre-state-migration-TIMESTAMP" \
  --force

# Re-enable flat architecture writes
hal9-migrate mode flat-only --force
```

### Selective Rollback
```bash
# Rollback specific category
hal9-migrate state rollback \
  --category neuron-states \
  --to-checkpoint latest \
  --preserve-others
```

### Data Recovery
```bash
# If data corruption detected
hal9-migrate state recover \
  --from-backup \
  --validate-each \
  --quarantine-corrupted
```

## Troubleshooting

### State Inconsistencies
```bash
# Find inconsistencies
hal9-migrate state diff \
  --systems flat,hierarchical \
  --verbose

# Auto-fix safe inconsistencies
hal9-migrate state fix \
  --auto \
  --safe-only

# Manual fix for complex issues
hal9-migrate state fix \
  --interactive \
  --guidance
```

### Migration Performance Issues
```bash
# Reduce migration rate
hal9-migrate configure state-migration \
  --rate-limit 50/s \
  --parallel-workers 2

# Monitor resource usage
watch -n 5 'hal9-migrate state stats --resources'
```

### Validation Failures
```bash
# Get detailed validation errors
hal9-migrate state validate \
  --failed-only \
  --debug \
  --suggestions

# Re-run specific validations
hal9-migrate state validate \
  --category user-sessions \
  --fix-and-retry
```

## Success Criteria

State migration is successful when:
1. ✅ All state categories migrated
2. ✅ Zero data loss confirmed
3. ✅ Reconciliation shows 100% consistency
4. ✅ Performance meets or exceeds baseline
5. ✅ All validations pass
6. ✅ No user impact detected
7. ✅ Team confidence high

## Post-Migration Monitoring

Continue monitoring for 24 hours:
```bash
# Set up continuous monitoring
hal9-migrate monitor \
  --phase post-state-migration \
  --duration 24h \
  --alert-on-drift
```

## Migration Log

```
Migration Start: ________________
Migration End: ________________
Total Duration: ________________

Records Migrated:
- User Sessions: ________________
- Neuron States: ________________  
- Learning Data: ________________
- Configuration: ________________
- Metadata: ________________

Issues Encountered:
_________________________________________________________________
_________________________________________________________________

Resolution:
_________________________________________________________________
_________________________________________________________________

Operator Signature: ________________
```

---

**Next Step**: After successful state migration, proceed to [Traffic Ramp-up](./05-traffic-ramp-up.md)