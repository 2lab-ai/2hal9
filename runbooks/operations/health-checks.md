# Health Check Procedures

**Version**: 1.0  
**Last Updated**: January 2025  
**Criticality**: HIGH  
**Estimated Time**: 15-30 minutes  
**Frequency**: Every 4 hours during migration, daily otherwise

## Purpose

Comprehensive health check procedures for HAL9's hierarchical architecture. These checks ensure system stability and early detection of issues during and after migration.

## Quick Health Check

### One-Line Health Status
```bash
hal9-migrate health --quick
```

Expected output:
```
✅ HAL9 Health: GOOD | Layers: 5/5 | Neurons: 847 active | Errors: 0.01% | Latency: 8.5ms
```

## Comprehensive Health Checks

### 1. Layer Health Verification

#### 1.1 Check All Layers
```bash
# Check each hierarchical layer
for layer in substrate protocol cognitive orchestration intelligence; do
  echo "=== Checking $layer layer ==="
  hal9-cli health --layer $layer --verbose
done
```

Expected for each layer:
```
Layer: substrate
Status: ✅ Healthy
Components: 12/12 active
Resource Usage: CPU 45%, Memory 2.1GB
Errors: 0
Latency: 0.5ms (internal)
```

#### 1.2 Layer Communication Health
```bash
# Verify inter-layer communication
hal9-migrate health --inter-layer

# Should show matrix like:
#              TO
#         Sub Pro Cog Orc Int
# F  Sub  [✓] [✓] [✓] [-] [-]
# R  Pro  [✓] [✓] [✓] [✓] [-]
# O  Cog  [-] [✓] [✓] [✓] [✓]
# M  Orc  [-] [-] [✓] [✓] [✓]
#    Int  [-] [-] [✓] [✓] [✓]
```

### 2. Neuron Health Checks

#### 2.1 Neuron Population Health
```bash
# Check neuron distribution and health
hal9-cli neurons health --summary

# Expected output:
# Strategic Neurons (L5):     45 active, 0 unhealthy
# Tactical Neurons (L4):      89 active, 0 unhealthy  
# Operational Neurons (L3):   156 active, 1 degraded
# Implementation Neurons (L2): 234 active, 0 unhealthy
# Reflexive Neurons (L1):     323 active, 0 unhealthy
# Total: 847 active, 1 degraded, 0 failed
```

#### 2.2 Individual Neuron Health
```bash
# Check specific neuron types having issues
hal9-cli neurons health --status degraded,unhealthy --details

# For each problematic neuron:
# Neuron: op-neuron-42 (L3)
# Status: Degraded
# Issue: High memory usage (1.2GB)
# Queue: 145 messages (above threshold)
# Last Error: None
# Recommendation: Monitor, consider scaling
```

#### 2.3 Neuron Connection Health  
```bash
# Verify neuron connectivity
hal9-cli neurons connections --verify

# Check for:
# - Isolated neurons (no connections)
# - Overconnected neurons (>100 connections)
# - Connection latency issues
# - Broken connections
```

### 3. Performance Health Checks

#### 3.1 Latency Health
```bash
# Check latency across percentiles
hal9-migrate health --latency

# Healthy ranges:
# P50: < 5ms   ✅ Current: 3.2ms
# P90: < 8ms   ✅ Current: 6.7ms
# P95: < 10ms  ✅ Current: 8.1ms
# P99: < 15ms  ✅ Current: 11.3ms
# P99.9: < 50ms ⚠️ Current: 47ms
```

#### 3.2 Throughput Health
```bash
# Verify processing capacity
hal9-migrate health --throughput

# Should show:
# Current: 1,247 req/s
# Capacity: 5,000 req/s  
# Utilization: 25% ✅
# Peak (24h): 2,156 req/s
# Headroom: 75% ✅
```

#### 3.3 Error Rate Health
```bash
# Check error rates by category
hal9-migrate health --errors --breakdown

# Acceptable levels:
# Total Error Rate: 0.01% ✅ (threshold: 0.1%)
# Timeout Errors: 0.001% ✅
# Processing Errors: 0.005% ✅
# Network Errors: 0.004% ✅
# Unknown Errors: 0% ✅
```

### 4. Resource Health Checks

#### 4.1 Memory Health
```bash
# Check memory usage and patterns
hal9-migrate health --memory

# Per-layer memory usage:
# Substrate: 1.2GB / 4GB (30%) ✅
# Protocol: 0.8GB / 2GB (40%) ✅
# Cognitive: 3.5GB / 8GB (44%) ✅
# Orchestration: 2.1GB / 4GB (52%) ✅
# Intelligence: 4.2GB / 8GB (52%) ✅
# 
# Memory growth rate: +12MB/hour ✅
# Garbage collection: Active ✅
# Memory leaks detected: None ✅
```

#### 4.2 CPU Health
```bash
# Check CPU utilization
hal9-migrate health --cpu

# CPU usage by layer:
# Substrate: 25% ✅
# Protocol: 15% ✅
# Cognitive: 45% ✅
# Orchestration: 35% ✅
# Intelligence: 55% ✅
#
# Total system CPU: 42% ✅
# CPU throttling: None ✅
# Context switches: Normal ✅
```

#### 4.3 Disk I/O Health
```bash
# Check disk performance
hal9-migrate health --disk

# Disk metrics:
# Write IOPS: 1,234 (limit: 10,000) ✅
# Read IOPS: 3,456 (limit: 10,000) ✅
# Write throughput: 45 MB/s ✅
# Read throughput: 123 MB/s ✅
# Disk queue depth: 12 ✅
# Free space: 567GB (73%) ✅
```

### 5. Data Health Checks

#### 5.1 Database Health
```bash
# Check database connections and performance
hal9-migrate health --database

# PostgreSQL health:
# Active connections: 45/200 ✅
# Idle connections: 23 ✅
# Query latency (avg): 2.3ms ✅
# Slow queries (>100ms): 3 ⚠️
# Replication lag: 0.1s ✅
# Deadlocks (24h): 0 ✅
```

#### 5.2 Cache Health
```bash
# Verify caching layer health
hal9-migrate health --cache

# Redis health:
# Memory usage: 4.5GB/16GB ✅
# Hit rate: 94.3% ✅
# Eviction rate: 0.01% ✅
# Connection pool: 50/100 ✅
# Response time: 0.3ms ✅
```

#### 5.3 Message Queue Health
```bash
# Check message queue status
hal9-migrate health --queues

# Queue depths:
# signal-queue: 234 messages ✅
# gradient-queue: 89 messages ✅
# consensus-queue: 12 messages ✅
# emergency-queue: 0 messages ✅
#
# Oldest message: 2.3 seconds ✅
# Processing rate: 1,450 msg/s ✅
```

### 6. Integration Health Checks

#### 6.1 External API Health
```bash
# Check external dependencies
hal9-migrate health --external

# Claude API:
# Status: ✅ Healthy
# Latency: 125ms
# Rate limit: 450/1000 requests
# Errors (1h): 0
#
# MCP Tools:
# Status: ✅ Healthy
# Available tools: 15/15
# Avg execution: 45ms
```

#### 6.2 Network Health
```bash
# Verify network connectivity
hal9-migrate health --network

# Network health:
# Internal latency: 0.5ms ✅
# Cross-region latency: 45ms ✅
# Packet loss: 0.0001% ✅
# Bandwidth usage: 234 Mbps / 10 Gbps ✅
# Active connections: 1,234 ✅
```

### 7. Security Health Checks

#### 7.1 Authentication Health
```bash
# Check auth system health
hal9-migrate health --auth

# Authentication health:
# Active sessions: 234 ✅
# Failed logins (1h): 12 ✅
# Token validation latency: 2ms ✅
# Certificate expiry: 89 days ✅
# Suspicious activity: None ✅
```

#### 7.2 Encryption Health
```bash
# Verify encryption status
hal9-migrate health --encryption

# Encryption status:
# TLS connections: 100% ✅
# Data at rest: Encrypted ✅
# Key rotation: 23 days ago ✅
# Cipher strength: AES-256 ✅
```

## Health Check Automation

### Continuous Health Monitoring
```bash
# Set up automated health checks
hal9-migrate health --monitor --interval 5m --alert-on-failure

# This will:
# - Run health checks every 5 minutes
# - Log results to monitoring system
# - Alert on any failures
# - Generate daily health reports
```

### Health Check Dashboard
```bash
# Open real-time health dashboard
hal9-migrate health --dashboard

# Or access at:
# https://grafana.production.example.com/d/hal9-health
```

## Health Score Calculation

Overall health score (0-100):
```
Score = (
  Layer Health × 0.3 +
  Performance Health × 0.3 +
  Resource Health × 0.2 +
  Error Rate Health × 0.2
)

Current Score: 94/100 ✅

Breakdown:
- Layers: 100/100 (all healthy)
- Performance: 92/100 (slight latency increase)
- Resources: 88/100 (moderate usage)
- Errors: 96/100 (minimal errors)
```

## Health Check Response Procedures

### If Health Check Fails

1. **Identify failing component**:
   ```bash
   hal9-migrate health --failed-only --details
   ```

2. **Check recent changes**:
   ```bash
   hal9-migrate changes --last 1h
   ```

3. **View component logs**:
   ```bash
   hal9-migrate logs --component $FAILED_COMPONENT --error
   ```

4. **Attempt auto-recovery**:
   ```bash
   hal9-migrate recover --component $FAILED_COMPONENT
   ```

5. **Escalate if needed**:
   ```bash
   hal9-migrate escalate --component $FAILED_COMPONENT --severity high
   ```

## Health Trends Analysis

### Daily Health Report
```bash
# Generate daily health trend report
hal9-migrate health --report daily --date yesterday

# Includes:
# - Health score trends
# - Component availability
# - Performance trends
# - Resource usage patterns
# - Error rate analysis
```

### Weekly Health Summary
```bash
# Weekly executive summary
hal9-migrate health --report weekly --format pdf

# Sent to: engineering-leaders@company.com
# Every: Monday 9:00 AM
```

## Quick Reference Card

```
Component         | Command                           | Healthy Range
------------------|-----------------------------------|---------------
Overall           | hal9-migrate health --quick       | Score > 90
Layers            | hal9-migrate health --layers      | All green
Neurons           | hal9-cli neurons health           | <5 degraded
Latency P99       | hal9-migrate health --latency     | <15ms
Error Rate        | hal9-migrate health --errors      | <0.1%
CPU Usage         | hal9-migrate health --cpu         | <80%
Memory Usage      | hal9-migrate health --memory      | <85%
Database          | hal9-migrate health --database    | Lag <1s
Queue Depth       | hal9-migrate health --queues      | <1000
External APIs     | hal9-migrate health --external    | All green
```

---

**Remember**: Regular health checks prevent major incidents. When in doubt, run a health check!