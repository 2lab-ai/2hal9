# HAL9 Migration CLI

Production-ready command-line tool for managing HAL9's hierarchical architecture migration.

## Overview

`hal9-migrate` provides operators with comprehensive tools to manage the zero-downtime migration from HAL9's flat architecture to the new hierarchical system. It supports all five migration phases: Shadow, Canary, State Migration, Ramp-up, and Full.

## Installation

```bash
cargo install --path hal9-migrate
```

## Quick Start

```bash
# Check system readiness
hal9-migrate pre-check

# View current migration status
hal9-migrate status

# Start shadow mode
hal9-migrate migrate --phase shadow

# Progress to canary with 5% traffic
hal9-migrate migrate --phase canary --percentage 5

# Monitor live metrics
hal9-migrate monitor
```

## Commands

### Pre-flight Checks
```bash
# Basic health check
hal9-migrate pre-check

# Deep validation including resources
hal9-migrate pre-check --deep

# Check specific components
hal9-migrate pre-check --components substrate protocol
```

### Migration Control
```bash
# Start migration phase
hal9-migrate migrate --phase <shadow|canary|state-migration|ramp-up|full>

# Canary deployment with traffic percentage
hal9-migrate migrate --phase canary --percentage 10

# Dry run to preview changes
hal9-migrate migrate --phase ramp-up --percentage 50 --dry-run

# Auto-approve without prompts
hal9-migrate migrate --phase full --yes
```

### Status Monitoring
```bash
# Current status
hal9-migrate status

# Detailed status with metrics
hal9-migrate status --detailed

# Watch mode (updates every 5 seconds)
hal9-migrate status --watch
```

### Feature Flags
```bash
# List all features
hal9-migrate feature list

# Enable a feature
hal9-migrate feature enable hierarchical_neurons

# Enable with traffic percentage
hal9-migrate feature enable meta_learning --percentage 20

# Disable a feature
hal9-migrate feature disable self_organization
```

### State Management
```bash
# Export current state
hal9-migrate state export -o migration-backup.json

# Create checkpoint
hal9-migrate state checkpoint -n "pre-canary" -d "Before canary deployment"

# List checkpoints
hal9-migrate state list-checkpoints

# Restore from checkpoint
hal9-migrate state restore -c "pre-canary"
```

### Rollback
```bash
# Rollback to previous phase
hal9-migrate rollback

# Rollback to specific phase
hal9-migrate rollback --to-phase shadow

# Force rollback (bypass checks)
hal9-migrate rollback --force --yes
```

### Live Monitoring
```bash
# Terminal-based monitoring
hal9-migrate monitor

# Logs view
hal9-migrate monitor --dashboard logs

# Distributed traces
hal9-migrate monitor --dashboard traces

# Combined view
hal9-migrate monitor --dashboard combined
```

### Web Dashboard
```bash
# Start web dashboard on default port (8080)
hal9-migrate dashboard

# Custom port
hal9-migrate dashboard --port 3000

# Access at http://localhost:8080
```

The web dashboard provides:
- Real-time migration progress visualization
- Performance metrics charts
- Feature flag status
- Recent events log
- Resource usage monitoring
- Health status indicators

## Migration Phases

### 1. Shadow Mode
- Mirrors all traffic to new system
- No production impact
- Validates functionality

### 2. Canary Deployment
- Routes small percentage to new system
- Monitors error rates
- Automatic rollback on failures

### 3. State Migration
- Migrates persistent data
- Maintains consistency
- Creates rollback snapshots

### 4. Ramp-up
- Gradually increases traffic
- Load tests at scale
- Performance optimization

### 5. Full Migration
- 100% traffic to new system
- Legacy decommission
- Final validation

## Safety Features

- **Automatic Rollback**: Triggers on error rate/latency thresholds
- **Health Checks**: Continuous validation of all components
- **State Snapshots**: Point-in-time recovery capability
- **Traffic Control**: Precise percentage-based routing
- **Dry Run Mode**: Preview changes before execution

## Configuration

Set server URL via environment:
```bash
export HAL9_SERVER=https://hal9.example.com:3030
hal9-migrate status
```

Or pass directly:
```bash
hal9-migrate --server https://hal9.example.com:3030 status
```

## Output Formats

```bash
# Pretty terminal output (default)
hal9-migrate status

# JSON for automation
hal9-migrate status --format json

# Table format
hal9-migrate feature list --format table
```

## Best Practices

1. **Always run pre-checks** before starting migration
2. **Start with shadow mode** to validate without impact
3. **Use small percentages** for initial canary deployments
4. **Monitor continuously** during migration phases
5. **Create checkpoints** before major transitions
6. **Test rollback procedures** in non-production first

## Troubleshooting

### Common Issues

**Pre-check failures**
- Ensure all components are healthy
- Check resource availability
- Verify network connectivity

**Migration stuck**
- Check logs with `hal9-migrate monitor --dashboard logs`
- Verify feature flags are enabled
- Look for resource constraints

**High error rates**
- Automatic rollback should trigger
- If not, use `hal9-migrate rollback --force`
- Check distributed traces for issues

## Architecture

The CLI communicates with HAL9's migration API endpoints:
- `/api/migration/*` - Migration control
- `/api/health` - Component health checks
- `/api/metrics` - Performance monitoring

It's designed for:
- Zero-downtime migrations
- Progressive rollouts
- Safe rollbacks
- Comprehensive monitoring

## Contributing

See main HAL9 repository for contribution guidelines.

## License

MIT License - see LICENSE file in root repository.