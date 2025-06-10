# HAL9 Production Migration Runbooks

This directory contains detailed operational runbooks for migrating HAL9 from flat to hierarchical architecture in production.

## Runbook Overview

### Core Migration Runbooks
1. **[Pre-Migration Checklist](./01-pre-migration-checklist.md)** - Preparation and validation steps
2. **[Shadow Mode Deployment](./02-shadow-mode-deployment.md)** - Deploy and validate shadow mode
3. **[Canary Deployment](./03-canary-deployment.md)** - Progressive traffic migration
4. **[State Migration](./04-state-migration.md)** - Migrate neuron states and data
5. **[Traffic Ramp-Up](./05-traffic-ramp-up.md)** - Increase traffic percentage
6. **[Full Cutover](./06-full-cutover.md)** - Complete migration to hierarchical
7. **[Post-Migration Validation](./07-post-migration-validation.md)** - Verify successful migration

### Emergency Procedures
- **[Rollback Procedures](./emergency/rollback-procedures.md)** - Emergency rollback steps

### Operational Procedures
- **[Health Check Procedures](./operations/health-checks.md)** - System health validation

## Critical Information

### On-Call Contacts
- **Primary**: migration-oncall@hal9.ai
- **Escalation**: platform-team@hal9.ai
- **Emergency**: cto@hal9.ai

### Key Systems
- **Production HAL9**: https://hal9.production.example.com
- **Monitoring**: https://grafana.production.example.com
- **Logs**: https://logs.production.example.com
- **Metrics**: https://prometheus.production.example.com

### Migration Tools
- **CLI**: `hal9-migrate` (must be installed on operator workstations)
- **Dashboard**: https://migration.hal9.production.example.com
- **API**: https://api.hal9.production.example.com/migration

## Runbook Standards

Each runbook follows this structure:
1. **Purpose** - What the runbook accomplishes
2. **Prerequisites** - Required conditions before starting
3. **Risk Assessment** - Potential risks and mitigations
4. **Step-by-Step Procedures** - Detailed instructions
5. **Validation Steps** - How to verify success
6. **Rollback Procedures** - How to revert if needed
7. **Troubleshooting** - Common issues and solutions

## Quick Decision Tree

```
Start Migration?
├─ Pre-checks Pass? → No → Fix Issues
│                     ↓
│                    Yes
│                     ↓
├─ Shadow Mode → Issues? → Yes → Investigate
│                ↓         
│               No
│                ↓
├─ Canary 5% → Error Rate OK? → No → Rollback
│               ↓
│              Yes
│               ↓
├─ State Migration → Validated? → No → Retry/Rollback
│                    ↓
│                   Yes
│                    ↓
├─ Ramp to 50% → Stable? → No → Hold/Rollback
│                 ↓
│                Yes
│                 ↓
└─ Full Cutover → Success → Monitor 24h → Decommission Legacy
```

## Safety Principles

1. **Never skip validation steps**
2. **Always have rollback ready**
3. **Monitor continuously during migration**
4. **Communicate all changes**
5. **Document any deviations**

## Usage Instructions

1. **Read the entire runbook before starting**
2. **Have a second operator for verification**
3. **Log all actions in the migration ticket**
4. **Use the buddy system for critical steps**
5. **Take breaks during long procedures**

## Training Requirements

Before executing migration:
- Complete HAL9 architecture training
- Practice in staging environment
- Shadow experienced operator
- Pass migration certification test

---

For architectural details, see [L4 Architecture Documentation](../../strategies/architecture/)