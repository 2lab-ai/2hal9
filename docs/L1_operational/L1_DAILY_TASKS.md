# HAL9 Daily Operations Tasks

**Level**: L1 Operational  
**Audience**: System Operators, DevOps  
**Purpose**: Daily tasks to keep HAL9 healthy

## Morning Checklist (Start of Day)

### 1. System Health Check (5 min)
```bash
# Run morning health check
./scripts/morning-check.sh

# Or manually:
echo "=== HAL9 Morning Check ==="
echo "1. Service Status:"
systemctl status hal9

echo "2. Port Check:"
netstat -tlpn | grep 8080

echo "3. Health Endpoint:"
curl -s http://localhost:8080/health | jq .

echo "4. Active Neurons:"
curl -s http://localhost:8080/neurons | jq '.neurons[].status'

echo "5. Last Night's Errors:"
grep ERROR /var/log/hal9/hal9.log | tail -10
```

### 2. Review Overnight Metrics (10 min)
```bash
# Check key metrics
curl -s http://localhost:8080/metrics | grep -E "
hal9_errors_total|
hal9_requests_total|
hal9_uptime_seconds|
hal9_memory_usage_bytes"

# Check for anomalies
./scripts/check-anomalies.sh --since yesterday
```

### 3. Clear Old Logs (5 min)
```bash
# Rotate logs if needed
if [ $(du -m /var/log/hal9 | cut -f1) -gt 1000 ]; then
    logrotate -f /etc/logrotate.d/hal9
fi

# Archive yesterday's logs
tar -czf logs-$(date -d yesterday +%Y%m%d).tar.gz /var/log/hal9/*.log.1
```

## Midday Tasks

### 1. Performance Check (5 min)
```bash
# Check response times
./scripts/perf-check.sh

# Sample output:
# Layer L5: avg 2ms (✓)
# Layer L4: avg 3ms (✓)
# Layer L3: avg 4ms (✓)
# Layer L2: avg 3ms (✓)
# Total: avg 12ms (✓)
```

### 2. Resource Usage (5 min)
```bash
# Memory check
free -h
ps aux | grep hal9 | awk '{sum+=$6} END {print "HAL9 Memory: " sum/1024 " MB"}'

# CPU check
top -bn1 | grep hal9

# Disk usage
df -h /var/lib/hal9
du -sh /var/lib/hal9/*
```

### 3. Queue Status (2 min)
```bash
# Check request queues
curl -s http://localhost:8080/queues | jq .

# If queues are backing up:
# 1. Check for slow neurons
# 2. Consider scaling
# 3. Check rate limits
```

## End of Day Tasks

### 1. Daily Backup (15 min)
```bash
# Backup script
./scripts/daily-backup.sh

# Or manually:
# Stop writes
curl -X POST http://localhost:8080/maintenance/enable

# Backup data
tar -czf hal9-backup-$(date +%Y%m%d).tar.gz \
    /var/lib/hal9 \
    /etc/hal9/config.yaml

# Resume writes
curl -X POST http://localhost:8080/maintenance/disable

# Copy backup offsite
scp hal9-backup-*.tar.gz backup-server:/backups/hal9/
```

### 2. Generate Daily Report (10 min)
```bash
# Generate report
./scripts/daily-report.sh > report-$(date +%Y%m%d).txt

# Email report
mail -s "HAL9 Daily Report $(date +%Y-%m-%d)" team@company.com < report-*.txt
```

### 3. Cleanup Tasks (5 min)
```bash
# Clean temporary files
find /tmp -name "hal9-*" -mtime +1 -delete

# Clean old session data
curl -X POST http://localhost:8080/sessions/cleanup

# Vacuum database (if using PostgreSQL)
psql -U hal9 -c "VACUUM ANALYZE;"
```

## Weekly Tasks (Do on Friday)

### 1. Deep Health Check (30 min)
```bash
# Run comprehensive tests
./scripts/weekly-health-check.sh

# This includes:
# - Memory leak detection
# - Performance regression tests
# - Security scan
# - Dependency updates check
```

### 2. Update Check (15 min)
```bash
# Check for updates
./hal9-server --version
curl -s https://api.github.com/repos/2lab/2hal9/releases/latest | jq -r .tag_name

# If update available:
# 1. Read changelog
# 2. Test in staging
# 3. Plan maintenance window
```

### 3. Capacity Planning (20 min)
```bash
# Growth trends
./scripts/capacity-report.sh

# Review:
# - Request growth rate
# - Resource utilization trends
# - Scaling needs
# - Budget implications
```

## Monthly Tasks

### 1. Security Audit (1 hour)
```bash
# Run security scan
./scripts/security-audit.sh

# Check:
# - Open ports
# - User permissions
# - SSL certificates
# - API keys rotation
```

### 2. Performance Tuning (2 hours)
```bash
# Analyze slow queries
./scripts/analyze-slow-queries.sh

# Optimize configuration
./scripts/tune-performance.sh

# Test improvements
./scripts/benchmark.sh
```

## Automation Scripts

### Create Morning Check Script
```bash
#!/bin/bash
# save as /usr/local/bin/hal9-morning-check

set -e

echo "HAL9 Morning Check - $(date)"
echo "========================="

# Function to check and report
check() {
    local name=$1
    local cmd=$2
    echo -n "$name: "
    if eval $cmd > /dev/null 2>&1; then
        echo "✓ OK"
    else
        echo "✗ FAILED"
        ERRORS=$((ERRORS + 1))
    fi
}

ERRORS=0

check "Service Running" "systemctl is-active hal9"
check "Port Listening" "netstat -an | grep -q :8080"
check "Health Check" "curl -sf http://localhost:8080/health"
check "Database Connection" "psql -U hal9 -c 'SELECT 1' > /dev/null"
check "Disk Space" "[ $(df /var/lib/hal9 | tail -1 | awk '{print $5}' | tr -d '%') -lt 80 ]"

echo "========================="
echo "Total Errors: $ERRORS"

exit $ERRORS
```

### Create Daily Report Script
```bash
#!/bin/bash
# save as /usr/local/bin/hal9-daily-report

cat << EOF
HAL9 Daily Report
Date: $(date)
================

SYSTEM STATUS
- Uptime: $(curl -s http://localhost:8080/metrics | grep uptime | awk '{print $2/3600 " hours"}')
- Health: $(curl -s http://localhost:8080/health | jq -r .status)
- Version: $(./hal9-server --version)

REQUEST STATISTICS
- Total Requests: $(curl -s http://localhost:8080/metrics | grep requests_total | awk '{print $2}')
- Error Rate: $(curl -s http://localhost:8080/metrics | grep errors_total | awk '{print $2}')
- Avg Response Time: $(curl -s http://localhost:8080/metrics | grep latency_seconds | awk '{print $2*1000 "ms"}')

RESOURCE USAGE
- Memory: $(ps aux | grep hal9 | awk '{sum+=$6} END {print sum/1024 " MB"}')
- CPU: $(ps aux | grep hal9 | awk '{print $3 "%"}')
- Disk: $(df -h /var/lib/hal9 | tail -1 | awk '{print $3 " / " $2 " (" $5 ")"}')

TOP ERRORS (Last 24h)
$(grep ERROR /var/log/hal9/hal9.log | tail -5)

RECOMMENDATIONS
$(./scripts/analyze-and-recommend.sh)

EOF
```

## Quick Reference Card

### Essential Commands
```bash
# Start/Stop
systemctl start hal9
systemctl stop hal9
systemctl restart hal9

# Status
systemctl status hal9
curl http://localhost:8080/health

# Logs
tail -f /var/log/hal9/hal9.log
journalctl -u hal9 -f

# Metrics
curl http://localhost:8080/metrics

# Backup
./scripts/daily-backup.sh

# Emergency
./scripts/emergency-restart.sh
```

### Important Paths
```
/etc/hal9/config.yaml     # Configuration
/var/lib/hal9/            # Data directory
/var/log/hal9/            # Logs
/usr/local/bin/hal9       # Binary
/opt/hal9/scripts/        # Utility scripts
```

### Key Metrics to Watch
- `hal9_errors_total` - Should be low
- `hal9_latency_seconds` - Should be <0.01
- `hal9_memory_usage_bytes` - Should be stable
- `hal9_neurons_active` - Should match config

## Escalation

### When to Page On-Call
1. Service down > 5 minutes
2. Error rate > 5%
3. Response time > 1 second
4. Memory usage > 90%
5. Disk full warnings

### Escalation Contacts
1. Tier 1: Operations team
2. Tier 2: DevOps lead
3. Tier 3: Engineering
4. Tier 4: CTO

---

*"A smooth operation is the result of daily diligence."*

**Keep HAL9 humming! 🎯**