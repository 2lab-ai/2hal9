# L1 - Reflexive Layer

**Cognitive Level**: L1_reflexive  
**Temporal Scope**: Microseconds to seconds  
**Purpose**: Immediate operational responses and system health

## Overview

This level contains everything needed for immediate operational responses and system health. All content here operates at the same level of abstraction, eliminating cognitive switching.

## Structure

- `status/` - System status, health checks, monitoring dashboards
- `responses/` - Pre-computed responses, quick actions, operational scripts  
- `cache/` - Response caches, quick lookup tables
- `emergency/` - Emergency procedures, circuit breakers, incident response

## For Operators

This is your home. Everything here is designed for immediate action without deep thinking:

### Quick Health Check
```bash
./status/scripts/health-check.sh
```

### Monitor System
```bash
./status/scripts/monitor.sh
```

### Emergency Response
Check `emergency/` for:
- Circuit breaker procedures
- Incident response playbooks
- Rollback commands
- Emergency contacts

## Navigation

- **Down** → [L2 Implementation](../L2_implementation/) for code details (rarely needed)
- **Up** → [L3 Operational](../L3_operational/) for system design context
- **Lateral** → Everything here is at the same "immediate action" level

## Principles

1. All content here shares the same temporal scope (microseconds to seconds)
2. No implementation details from lower levels
3. No strategic concerns from higher levels
4. Self-contained within this cognitive space

## What Belongs Here

✅ DO include:
- Health check scripts
- Status dashboards
- Emergency procedures
- Quick reference cards
- Operational commands
- Alert responses

❌ DON'T include:
- Source code (→ L2)
- Architecture docs (→ L3)
- Planning documents (→ L4)
- Strategic vision (→ L5+)