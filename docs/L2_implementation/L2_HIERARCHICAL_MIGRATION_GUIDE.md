# HAL9 Hierarchical Migration Guide

**Level**: L2 - Implementation  
**Audience**: Development Teams, DevOps Engineers, Migration Operators  
**Purpose**: Step-by-step guide for migrating to hierarchical architecture

## Overview

This guide provides detailed instructions for migrating from HAL9's flat architecture to the new hierarchical system. Follow these steps carefully to ensure a smooth, zero-downtime migration.

## Prerequisites

### System Requirements
- Kubernetes 1.26+ or Docker 24.0+
- PostgreSQL 14+ or SQLite 3.40+
- 16GB RAM minimum (32GB recommended)
- 100GB available storage
- Network connectivity between all nodes

### Tools Required
- `hal9-migrate` CLI tool
- `kubectl` (for Kubernetes deployments)
- `docker` and `docker-compose`
- Monitoring access (Prometheus/Grafana)

### Pre-Migration Checklist
- [ ] Backup current system state
- [ ] Document current configuration
- [ ] Verify monitoring is operational
- [ ] Confirm rollback procedures
- [ ] Schedule maintenance window (if needed)
- [ ] Notify stakeholders

## Migration Phases

### Phase 1: Shadow Mode

**Duration**: 24-48 hours  
**Risk**: None (read-only mirroring)

#### Step 1: Install Migration Tools
```bash
# Install the migration CLI
cargo install --path hal9-migrate

# Verify installation
hal9-migrate --version
```

#### Step 2: Pre-Flight Checks
```bash
# Run comprehensive health checks
hal9-migrate pre-check --deep

# Check specific components if needed
hal9-migrate pre-check --components substrate protocol cognitive
```

#### Step 3: Enable Shadow Mode
```bash
# Start shadow mode
hal9-migrate migrate --phase shadow --yes

# Verify shadow mode is active
hal9-migrate status --detailed
```

#### Step 4: Monitor Shadow Metrics
```bash
# Watch real-time metrics
hal9-migrate monitor --dashboard metrics

# Check for discrepancies
hal9-migrate verify --tests shadow-comparison
```

### Phase 2: Canary Deployment

**Duration**: 3-7 days  
**Risk**: Low (automatic rollback available)

#### Step 1: Start Small (5%)
```bash
# Create checkpoint before canary
hal9-migrate state checkpoint -n "pre-canary" -d "Before canary deployment"

# Enable canary with 5% traffic
hal9-migrate migrate --phase canary --percentage 5
```

#### Step 2: Monitor and Validate
```bash
# Monitor error rates and latency
hal9-migrate status --watch --interval 10

# Check feature flags
hal9-migrate feature list

# View detailed metrics
hal9-migrate monitor --dashboard combined
```

#### Step 3: Gradual Increase
```bash
# If stable after 24 hours, increase to 10%
hal9-migrate migrate --phase canary --percentage 10

# After another 24 hours, go to 25%
hal9-migrate migrate --phase canary --percentage 25

# Finally increase to 35%
hal9-migrate migrate --phase canary --percentage 35
```

#### Step 4: Rollback (if needed)
```bash
# Automatic rollback triggers on:
# - Error rate > 5%
# - Latency p99 > 50ms
# - Health check failures

# Manual rollback
hal9-migrate rollback --to-phase shadow
```

### Phase 3: State Migration

**Duration**: 2-4 hours  
**Risk**: Medium (requires careful coordination)

#### Step 1: Prepare State Migration
```bash
# Export current state
hal9-migrate state export -o pre-migration-state.json

# Create migration checkpoint
hal9-migrate state checkpoint -n "pre-state-migration" \
  -d "Before state migration"
```

#### Step 2: Execute State Migration
```bash
# Start state migration
hal9-migrate migrate --phase state-migration

# Monitor progress
hal9-migrate status --watch
```

#### Step 3: Verify Data Integrity
```bash
# Run integrity checks
hal9-migrate verify --tests data-integrity state-consistency

# Compare state checksums
hal9-migrate verify --full --report
```

### Phase 4: Traffic Ramp-up

**Duration**: 3-5 days  
**Risk**: Low (gradual increase)

#### Step 1: Increase to 50%
```bash
# Ramp up to 50%
hal9-migrate migrate --phase ramp-up --percentage 50

# Run load tests
hal9-migrate verify --tests load-test-50
```

#### Step 2: Monitor at Scale
```bash
# Watch performance metrics
hal9-migrate monitor --dashboard metrics

# Check distributed traces
hal9-migrate monitor --dashboard traces
```

#### Step 3: Increase to 75%
```bash
# After 24 hours of stability
hal9-migrate migrate --phase ramp-up --percentage 75

# Verify all systems
hal9-migrate verify --full
```

#### Step 4: Prepare for Full Migration
```bash
# Final checkpoint
hal9-migrate state checkpoint -n "pre-full-migration" \
  -d "Before 100% migration"

# Run final checks
hal9-migrate pre-check --deep
```

### Phase 5: Full Migration

**Duration**: 1-2 hours  
**Risk**: Low (with proper preparation)

#### Step 1: Complete Migration
```bash
# Migrate to 100%
hal9-migrate migrate --phase full

# Verify completion
hal9-migrate status --detailed
```

#### Step 2: Validate Full System
```bash
# Comprehensive validation
hal9-migrate verify --full --report

# Check all endpoints
hal9-migrate verify --tests endpoint-validation
```

#### Step 3: Decommission Legacy
```bash
# After 24 hours of stability
# Stop legacy containers
docker stop hal9-legacy

# Archive legacy data
tar -czf hal9-legacy-backup.tar.gz /var/lib/hal9-legacy/
```

## Feature Flag Management

### Available Feature Flags

```bash
# List all features
hal9-migrate feature list
```

Key features:
- `hierarchical_neurons`: Core hierarchical processing
- `substrate_abstraction`: Infrastructure abstraction layer
- `protocol_negotiation`: Dynamic protocol selection
- `meta_learning`: Learning-to-learn capabilities
- `self_organization`: Topology evolution

### Progressive Feature Enablement

```bash
# Enable features gradually
hal9-migrate feature enable hierarchical_neurons --percentage 10
hal9-migrate feature enable substrate_abstraction --percentage 25
hal9-migrate feature enable protocol_negotiation --percentage 50
```

## Monitoring and Observability

### Key Metrics to Watch

1. **Error Rate**: Should stay below 1%
2. **Latency p99**: Should stay below 10ms
3. **CPU Usage**: Should not exceed 80%
4. **Memory Usage**: Should not exceed 90%
5. **Throughput**: Should meet or exceed baseline

### Grafana Dashboards

Access dashboards at: `http://monitoring.hal9.local:3000`

- **HAL9 Overview**: System-wide metrics
- **Migration Progress**: Phase-specific metrics
- **Layer Performance**: Per-layer breakdown
- **Neuron Activity**: Individual neuron metrics

### Alerts

Critical alerts configured:
```yaml
- alert: HighErrorRate
  expr: hal9_error_rate > 0.05
  for: 5m
  
- alert: HighLatency
  expr: hal9_latency_p99 > 50
  for: 5m
  
- alert: MigrationStalled
  expr: hal9_migration_progress == 0
  for: 30m
```

## Troubleshooting

### Common Issues

#### Issue: Pre-check failures
```bash
# Solution: Fix component health
hal9-cli health fix --component <name>

# Retry pre-check
hal9-migrate pre-check
```

#### Issue: High error rate during canary
```bash
# Solution: Check logs
hal9-migrate monitor --dashboard logs

# Review distributed traces
hal9-migrate monitor --dashboard traces

# Rollback if needed
hal9-migrate rollback
```

#### Issue: State migration stuck
```bash
# Solution: Check migration status
hal9-migrate status --detailed

# Resume from checkpoint
hal9-migrate state restore -c <checkpoint-name>

# Retry migration
hal9-migrate migrate --phase state-migration
```

#### Issue: Performance degradation
```bash
# Solution: Check resource usage
kubectl top pods -n hal9

# Scale up if needed
kubectl scale deployment hal9-hierarchical --replicas=20

# Optimize configuration
hal9-cli config tune --auto
```

## Rollback Procedures

### Automatic Rollback

Triggers on:
- Error rate > 5% for 5 minutes
- Latency p99 > 50ms for 5 minutes
- Failed health checks for 3 consecutive times
- Memory/CPU thresholds exceeded

### Manual Rollback

```bash
# Rollback to previous phase
hal9-migrate rollback

# Rollback to specific phase
hal9-migrate rollback --to-phase canary

# Force rollback (bypass checks)
hal9-migrate rollback --force --yes
```

### Post-Rollback

1. Investigate root cause
2. Fix identified issues
3. Create new checkpoint
4. Retry migration phase

## Configuration Examples

### Kubernetes Configuration

```yaml
# hal9-hierarchical-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: hal9-config
data:
  config.yaml: |
    system:
      mode: hierarchical
      layers:
        substrate:
          type: distributed
          nodes: 10
        cognitive:
          neuron_types:
            - strategic
            - tactical
            - operational
            - implementation
            - reflexive
    migration:
      phase: canary
      percentage: 35
      auto_rollback: true
```

### Docker Compose Configuration

```yaml
# docker-compose.hierarchical.yml
version: '3.8'
services:
  hal9-hierarchical:
    image: hal9:hierarchical-latest
    ports:
      - "3030:3030"
    environment:
      - HAL9_MODE=hierarchical
      - HAL9_SUBSTRATE=distributed
      - HAL9_MIGRATION_PHASE=canary
    volumes:
      - ./config:/app/config
      - hal9-data:/app/data
    healthcheck:
      test: ["CMD", "hal9-cli", "health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

## Post-Migration Tasks

### Verification
- [ ] All traffic routed to new system
- [ ] Legacy system decommissioned
- [ ] Monitoring shows stable metrics
- [ ] All feature flags at 100%
- [ ] Documentation updated

### Optimization
- [ ] Tune resource allocations
- [ ] Optimize topology
- [ ] Enable advanced features
- [ ] Configure auto-scaling

### Cleanup
- [ ] Remove migration tools
- [ ] Archive migration logs
- [ ] Clean up checkpoints
- [ ] Update runbooks

## Support

### Resources
- Documentation: `/docs/L4_architecture/`
- Migration CLI: `hal9-migrate --help`
- Slack: #hal9-migration
- On-call: migration-oncall@hal9.ai

### Emergency Contacts
- Migration Lead: migration-lead@hal9.ai
- Platform Team: platform@hal9.ai
- CTO: cto@hal9.ai

---

*"A journey of a thousand neurons begins with a single migration."*

**Good luck with your migration!**