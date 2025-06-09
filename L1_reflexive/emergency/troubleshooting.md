# HAL9 Troubleshooting Guide

**Level**: L1 Operational  
**Audience**: Operators, Support Staff  
**Purpose**: Fix common HAL9 issues quickly

## Quick Diagnosis

### Is HAL9 Running?
```bash
# Check process
ps aux | grep hal9

# Check port
netstat -an | grep 8080

# Check Docker
docker ps | grep hal9
```

### Is HAL9 Responding?
```bash
# Health check
curl http://localhost:8080/health

# If no response, check logs
docker logs hal9-container
# OR
tail -f /var/log/hal9/hal9.log
```

## Common Issues & Solutions

### 🔴 HAL9 Won't Start

#### Port Already in Use
```bash
# Error: "address already in use"

# Find what's using port 8080
lsof -i :8080

# Kill it (carefully!)
kill -9 <PID>

# Or use different port
./hal9-server --port 8081
```

#### Missing Config File
```bash
# Error: "config file not found"

# Create default config
cat > config.yaml << EOF
neurons:
  - id: "basic"
    layer: "L2"
    type: "implementation"
EOF

# Run with config
./hal9-server --config config.yaml
```

#### Permission Denied
```bash
# Error: "permission denied"

# Make executable
chmod +x hal9-server

# Check file ownership
ls -la hal9-server

# Run with sudo if needed (not recommended)
sudo ./hal9-server
```

### 🟡 HAL9 is Slow

#### Check Resource Usage
```bash
# CPU and Memory
top -p $(pgrep hal9)

# Docker stats
docker stats hal9-container

# If high CPU/Memory:
# 1. Restart HAL9
# 2. Reduce neuron count
# 3. Add resource limits
```

#### Network Issues
```bash
# Test network latency
ping localhost

# Check DNS
nslookup localhost

# Check firewall
sudo iptables -L

# If network is slow:
# 1. Check network configuration
# 2. Restart network service
# 3. Use local connections
```

#### Database Connection
```bash
# Test database connection
psql -h localhost -U hal9 -d hal9_db -c "SELECT 1"

# If connection fails:
# 1. Check database is running
# 2. Verify credentials
# 3. Check connection pool settings
```

### 🔵 HAL9 Returns Errors

#### 500 Internal Server Error
```bash
# Check logs immediately
tail -n 100 /var/log/hal9/error.log

# Common causes:
# - Out of memory
# - Neuron crash
# - Database down

# Quick fix:
./hal9-server restart
```

#### 502 Bad Gateway
```bash
# Check upstream services
curl http://localhost:8080/health

# Check reverse proxy (if using)
nginx -t
systemctl status nginx

# Quick fix:
systemctl restart nginx
systemctl restart hal9
```

#### 503 Service Unavailable
```bash
# HAL9 is overloaded

# Check request queue
curl http://localhost:8080/metrics | grep queue

# Solutions:
# 1. Increase rate limits
# 2. Add more neurons
# 3. Scale horizontally
```

### ⚡ Performance Issues

#### High Memory Usage
```bash
# Check memory
free -h

# Find memory leaks
./hal9-server --debug-memory

# Solutions:
# 1. Restart regularly
# 2. Limit neuron memory
# 3. Enable garbage collection

# Add to config:
echo "memory_limit: 2G" >> config.yaml
```

#### High CPU Usage
```bash
# Check CPU
mpstat 1 5

# Find hot spots
./hal9-server --profile-cpu

# Solutions:
# 1. Reduce concurrent requests
# 2. Enable request batching
# 3. Optimize neuron algorithms
```

### 🌐 Connection Issues

#### Cannot Connect to HAL9
```bash
# Test local connection
telnet localhost 8080

# Check firewall
sudo ufw status

# Allow port
sudo ufw allow 8080

# Check binding address
# Should be 0.0.0.0 not 127.0.0.1 for external access
./hal9-server --bind 0.0.0.0:8080
```

#### WebSocket Disconnects
```bash
# Check timeout settings
grep -i timeout config.yaml

# Increase timeouts
cat >> config.yaml << EOF
websocket:
  ping_interval: 30s
  timeout: 300s
EOF
```

### 🧠 Neuron Issues

#### Neuron Not Responding
```bash
# Check neuron status
curl http://localhost:8080/neurons

# Restart specific neuron
curl -X POST http://localhost:8080/neurons/L3-001/restart

# If still broken:
# 1. Check neuron logs
# 2. Recreate neuron
# 3. Check configuration
```

#### Learning Not Working
```bash
# Check learning status
curl http://localhost:8080/learning/status

# Reset learning state
curl -X POST http://localhost:8080/learning/reset

# Verify gradients flowing
curl http://localhost:8080/metrics | grep gradient
```

## Log Analysis

### Where to Find Logs
```bash
# Docker logs
docker logs -f hal9-container

# System logs
/var/log/hal9/hal9.log     # Main log
/var/log/hal9/error.log    # Errors only
/var/log/hal9/neuron-*.log # Per-neuron logs

# Journalctl (systemd)
journalctl -u hal9 -f
```

### Understanding Log Levels
```
ERROR - Something is broken
WARN  - Something might break
INFO  - Normal operation
DEBUG - Detailed information
TRACE - Everything (verbose!)
```

### Common Log Patterns
```bash
# Find errors
grep ERROR /var/log/hal9/hal9.log

# Find slow queries
grep "duration>" /var/log/hal9/hal9.log | grep -E "[0-9]{4,}ms"

# Find crashes
grep -E "panic|fatal|crash" /var/log/hal9/hal9.log
```

## Emergency Procedures

### HAL9 Complete Failure
```bash
# 1. Save logs
cp -r /var/log/hal9 /tmp/hal9-logs-$(date +%s)

# 2. Kill everything
pkill -9 hal9

# 3. Clear state
rm -rf /var/lib/hal9/state/*

# 4. Restart fresh
./hal9-server --clean-start
```

### Data Corruption
```bash
# 1. Stop HAL9
systemctl stop hal9

# 2. Backup current data
tar -czf hal9-backup-$(date +%s).tar.gz /var/lib/hal9

# 3. Run integrity check
./hal9-server --check-integrity

# 4. Restore from backup if needed
./hal9-server --restore /path/to/backup
```

### Memory Leak
```bash
# 1. Confirm memory leak
./scripts/detect-memory-leak.sh

# 2. Emergency restart
systemctl restart hal9

# 3. Enable memory profiling
./hal9-server --memory-profile

# 4. Set up automatic restarts
echo "0 */4 * * * systemctl restart hal9" | crontab -
```

## Monitoring Commands

### Quick Status Check
```bash
#!/bin/bash
# save as check-hal9.sh

echo "=== HAL9 Status ==="
echo "Process: $(pgrep hal9 > /dev/null && echo "Running" || echo "Stopped")"
echo "Port: $(netstat -an | grep 8080 > /dev/null && echo "Listening" || echo "Not listening")"
echo "Health: $(curl -s http://localhost:8080/health | jq -r .status)"
echo "Neurons: $(curl -s http://localhost:8080/metrics | grep neurons_active | awk '{print $2}')"
echo "Memory: $(ps aux | grep hal9 | awk '{print $6/1024 " MB"}')"
```

### Real-time Monitoring
```bash
# Watch metrics
watch -n 1 'curl -s http://localhost:8080/metrics | grep -E "requests|errors|latency"'

# Monitor logs
tail -f /var/log/hal9/hal9.log | grep -E "ERROR|WARN"

# System resources
htop -p $(pgrep hal9)
```

## Getting More Help

### Collect Debug Info
```bash
# Run debug script
./scripts/collect-debug-info.sh

# This collects:
# - System info
# - HAL9 version
# - Configuration
# - Recent logs
# - Metrics snapshot
```

### Contact Support
1. Run debug script (above)
2. Create issue: https://github.com/2lab/2hal9/issues
3. Include:
   - Error messages
   - Steps to reproduce
   - Debug info output
   - Expected behavior

### Community Help
- Discord: https://discord.gg/hal9
- Stack Overflow: [hal9] tag
- Forums: https://forums.hal9.ai

---

*"Even HAL9000 had issues. The key is knowing how to fix them."*

**Stay calm and troubleshoot on! 🔧**