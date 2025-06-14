# L1: Reflexive Layer - Operational Intelligence

## Overview

The L1 Reflexive Layer represents the operational foundation of the HAL9 system, providing immediate responses, monitoring, and day-to-day operational procedures. This layer handles:

- **Immediate Response**: Quick reactions to system events and user requests
- **Operational Monitoring**: Real-time health checks and performance tracking
- **Emergency Procedures**: Rapid response protocols for system failures
- **Testing & Validation**: Automated test suites for system components

## Directory Structure

```
L1_reflexive/
├── README.md                    # This file
├── cache/                       # Temporary cache for reflexive responses
├── check-dependencies.sh        # Dependency verification script
├── common-env.sh               # Shared environment configuration
├── fix-remaining-scripts.sh    # Script maintenance utility
├── test-l1-fixes.sh           # L1 layer test suite
├── emergency/                   # Emergency response procedures
│   ├── scripts/                # Emergency test and recovery scripts
│   │   ├── test-*.sh          # Various component tests
│   │   └── test-*.sh.bak      # Backup files
│   └── troubleshooting.md      # Comprehensive troubleshooting guide
├── responses/                   # Standard operational responses
│   ├── daily-tasks.md          # Daily operational runbook
│   └── scripts/                # Operational automation scripts
│       ├── run-*.sh           # Demo and execution scripts
│       └── run-*.sh.bak       # Backup files
└── status/                      # System status and monitoring
    ├── monitoring.md           # Monitoring setup and metrics guide
    ├── quick-start.md          # Quick start guide for operators
    └── scripts/                # Health check and monitoring scripts
        ├── health-check.sh    # System health verification
        └── monitor.sh         # Real-time monitoring
```

## Quick Navigation

### For Operators
- **Getting Started**: [status/quick-start.md](status/quick-start.md)
- **Daily Operations**: [responses/daily-tasks.md](responses/daily-tasks.md)
- **Monitoring Setup**: [status/monitoring.md](status/monitoring.md)

### For Troubleshooting
- **Troubleshooting Guide**: [emergency/troubleshooting.md](emergency/troubleshooting.md)
- **Emergency Scripts**: [emergency/scripts/](emergency/scripts/)

### For Testing
- **Test Scripts**: Located in `emergency/scripts/test-*.sh`
- **Demo Scripts**: Located in `responses/scripts/run-*.sh`

## Environment Setup

Before running any L1 scripts, ensure your environment is properly configured:

```bash
# Set HAL9 base directory
export HAL9_HOME=$(pwd)

# Set configuration paths
export HAL9_CONFIG_DIR="$HAL9_HOME/L5_strategic/research/examples"
export HAL9_DATA_DIR="$HAL9_HOME/substrate/storage/databases"
export HAL9_LOG_DIR="$HAL9_HOME/logs"

# Create necessary directories
mkdir -p "$HAL9_LOG_DIR"

# Source common environment (optional, for shared settings)
source ./L1_reflexive/common-env.sh

# Check dependencies
./L1_reflexive/check-dependencies.sh
```

## Available Scripts

### Utility Scripts
- `check-dependencies.sh` - Verify system dependencies
- `common-env.sh` - Shared environment configuration
- `fix-remaining-scripts.sh` - Script maintenance utility
- `test-l1-fixes.sh` - Test L1 layer fixes

### Emergency Test Scripts
- `test-3neuron-demo.sh` - Test 3-neuron configuration
- `test-auth.sh` - Test authentication system
- `test-browser-automation.sh` - Test browser automation
- `test-claude-api.sh` - Test Claude API integration
- `test-codegen.sh` - Test code generation
- `test-discovery.sh` - Test service discovery
- `test-hybrid-mode.sh` - Test hybrid operation mode
- `test-metrics.sh` - Test metrics collection
- `test-performance.sh` - Test system performance
- `test-prometheus.sh` - Test Prometheus integration

### Response Scripts
- `run-3neuron-demo.sh` - Run 3-neuron demonstration
- `run-benchmarks.sh` - Run performance benchmarks
- `run-demo.sh` - Run basic demonstration

### Status Scripts
- `health-check.sh` - Check system health
- `monitor.sh` - Real-time system monitoring

## Common Operations

### 1. Health Check and Monitoring
```bash
# Local development
./L1_reflexive/status/scripts/health-check.sh --local

# Production (Kubernetes)
./L1_reflexive/status/scripts/health-check.sh --k8s

# Real-time monitoring
./L1_reflexive/status/scripts/monitor.sh
```

### 2. Run Demos and Benchmarks
```bash
# Run basic demo
./L1_reflexive/responses/scripts/run-demo.sh

# Run 3-neuron demo
./L1_reflexive/responses/scripts/run-3neuron-demo.sh

# Run performance benchmarks
./L1_reflexive/responses/scripts/run-benchmarks.sh
```

### 3. Test System Components
```bash
# Core tests
./L1_reflexive/emergency/scripts/test-auth.sh
./L1_reflexive/emergency/scripts/test-claude-api.sh
./L1_reflexive/emergency/scripts/test-metrics.sh

# Additional component tests
./L1_reflexive/emergency/scripts/test-3neuron-demo.sh
./L1_reflexive/emergency/scripts/test-browser-automation.sh
./L1_reflexive/emergency/scripts/test-codegen.sh
./L1_reflexive/emergency/scripts/test-discovery.sh
./L1_reflexive/emergency/scripts/test-hybrid-mode.sh
./L1_reflexive/emergency/scripts/test-performance.sh
./L1_reflexive/emergency/scripts/test-prometheus.sh
```

## Port Allocation

Standard port assignments for HAL9 services:

| Service | Port | Purpose |
|---------|------|---------|
| Main Server | 8080 | Primary HAL9 API |
| Auth Service | 8081 | Authentication endpoint |
| Metrics | 9090 | Prometheus metrics |
| GraphQL | 8082 | GraphQL API (if enabled) |
| Admin | 8083 | Administrative interface |

## Environment Variables

Required environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `ANTHROPIC_API_KEY` | Claude API key | None (uses mock) |
| `DATABASE_URL` | Primary database connection | `sqlite://hal9.db` |
| `REDIS_URL` | Redis cache connection | `redis://localhost:6379` |
| `LOG_LEVEL` | Logging verbosity | `info` |

Optional variables for production:

| Variable | Description |
|----------|-------------|
| `SLACK_WEBHOOK_URL` | Slack notifications |
| `PAGERDUTY_TOKEN` | PagerDuty integration |
| `SENTRY_DSN` | Error tracking |

## Development vs Production

### Development Setup
- Uses local file paths
- SQLite databases in `substrate/storage/databases/`
- Direct binary execution
- Mock Claude responses by default

### Production Setup
- Uses system paths (`/etc/hal9/`, `/var/log/hal9/`)
- PostgreSQL database
- Systemd service management
- Real Claude API integration
- Kubernetes deployment

## Troubleshooting

### Common Issues

1. **Scripts fail with "file not found"**
   - Run from HAL9 root directory
   - Check environment setup above

2. **Port already in use**
   - Check running processes: `lsof -i :8080`
   - Kill existing process or use different port

3. **Database connection errors**
   - Ensure databases exist: `ls substrate/storage/databases/`
   - Check DATABASE_URL environment variable

4. **Missing dependencies**
   - Run dependency check script
   - Install missing tools (jq, curl, etc.)

For detailed troubleshooting, see [emergency/troubleshooting.md](emergency/troubleshooting.md)

## Layer Integration

The L1 Reflexive layer interfaces with:

- **L2 Implementation**: Receives implementation requests
- **L3 Operational**: Reports operational status
- **L4 Tactical**: Executes tactical decisions
- **L5 Strategic**: Implements strategic directives

## Contributing

When adding new L1 scripts or procedures:

1. Place emergency procedures in `emergency/`
2. Place routine operations in `responses/`
3. Place monitoring tools in `status/`
4. Update this README with new operations
5. Ensure scripts use environment variables
6. Add proper error handling and logging

## Maintenance

Regular maintenance tasks:

- Review and update troubleshooting guides monthly
- Test all emergency procedures quarterly
- Update monitoring dashboards as needed
- Archive old logs and cache files
- Review and optimize frequently used scripts

---

For architectural details, see [L9 Universal Principles](../L9_universal/README.md)