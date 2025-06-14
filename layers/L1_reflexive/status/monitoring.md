# HAL9 Monitoring Guide

## Overview
HAL9 provides comprehensive monitoring through Prometheus metrics and Grafana dashboards. This guide covers setup, usage, and customization of the monitoring stack.

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│ HAL9 Server │────▶│  Prometheus  │────▶│   Grafana   │
│  /metrics   │     │   Scraper    │     │ Dashboards  │
└─────────────┘     └──────────────┘     └─────────────┘
                            │
                            ▼
                    ┌──────────────┐
                    │ AlertManager │
                    └──────────────┘
```

## Quick Start

### 1. Start HAL9 with Metrics
```bash
# Run HAL9 server (metrics enabled by default)
cargo run --bin hal9-server config/production.yaml
```

### 2. Start Monitoring Stack
```bash
# Create docker network
docker network create hal9-network

# Start Prometheus, Grafana, and AlertManager
docker-compose -f docker-compose.monitoring.yml up -d
```

### 3. Access Dashboards
- **Grafana**: http://localhost:3000 (admin/hal9admin)
- **Prometheus**: http://localhost:9090
- **AlertManager**: http://localhost:9093
- **HAL9 Metrics**: http://localhost:8080/metrics

## Metrics Reference

### System Metrics
- `hal9_server_uptime_seconds` - Server uptime
- `hal9_server_memory_bytes` - Memory usage
- `hal9_signals_rate_per_second` - Signal processing rate

### Neuron Metrics
- `hal9_neurons_active` - Active neuron count
- `hal9_neuron_health` - Neuron health status (0/1)
- `hal9_neuron_queue_size` - Processing queue depth
- `hal9_signal_processing_duration_seconds` - Processing latency

### Claude Integration
- `hal9_claude_tokens_used_total` - Token usage by type
- `hal9_claude_cost_dollars_total` - Total API cost
- `hal9_claude_cost_rate_dollars_per_hour` - Current cost rate
- `hal9_claude_api_calls_total` - API call counts

### Learning Metrics
- `hal9_learning_cycles_total` - Completed learning cycles
- `hal9_learning_effectiveness_ratio` - Learning success rate
- `hal9_learning_error_gradient` - Error gradient values
- `hal9_learning_adjustments_total` - Prompt adjustments

### Security Metrics
- `hal9_auth_users_active` - Active user count
- `hal9_auth_attempts_total` - Authentication attempts
- `hal9_mcp_security_violations_total` - Security violations
- `hal9_mcp_tool_invocations_total` - Tool usage

## Grafana Dashboards

### 1. System Overview (`hal9-overview`)
Main dashboard showing:
- Server health and uptime
- Signal processing rates
- Cost tracking
- Error rates
- Token usage

### 2. Neuron Performance (`hal9-neurons`)
Detailed neuron metrics:
- Health matrix
- Queue depths
- Processing latency heatmap
- Signal forwarding flow
- Layer-specific metrics

### 3. Learning & Security (`hal9-learning-security`)
Specialized views for:
- Learning effectiveness
- Error gradients
- Authentication metrics
- Security violations
- MCP tool usage

## Alert Configuration

### Critical Alerts
- `HAL9ServerDown` - Server unavailable
- `HAL9DailyBudgetExceeded` - Cost limit reached
- `HAL9SecurityViolation` - Security breach detected

### Warning Alerts
- `HAL9HighErrorRate` - Error rate > 0.1/sec
- `HAL9HighLatency` - p95 latency > 5s
- `HAL9NeuronUnhealthy` - Neuron health check failed
- `HAL9LearningDegraded` - Learning effectiveness < 50%

### Customizing Alerts
Edit `monitoring/alerts.yml`:
```yaml
- alert: CustomAlert
  expr: your_metric > threshold
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "Alert summary"
```

## Prometheus Queries

### Useful Queries
```promql
# Signal processing rate by layer
sum by (layer) (rate(hal9_signals_processed_total[5m]))

# Cost per hour trend
hal9_claude_cost_rate_dollars_per_hour

# p95 latency by neuron
histogram_quantile(0.95, 
  sum by (neuron_id, le) (
    rate(hal9_neuron_processing_duration_seconds_bucket[5m])
  )
)

# Authentication failure rate
rate(hal9_auth_attempts_total{status="failure"}[5m])
```

## Production Deployment

### 1. Configure Prometheus Targets
Edit `monitoring/prometheus.yml`:
```yaml
scrape_configs:
  - job_name: 'hal9-server'
    static_configs:
      - targets: 
        - 'hal9-prod-1.example.com:8080'
        - 'hal9-prod-2.example.com:8080'
```

### 2. Set Up Remote Storage
For long-term storage, configure Prometheus remote write:
```yaml
remote_write:
  - url: "https://prometheus-storage.example.com/api/v1/write"
```

### 3. Configure Alerting
Update AlertManager receivers:
```yaml
receivers:
  - name: 'critical'
    pagerduty_configs:
      - service_key: 'your-pagerduty-key'
    slack_configs:
      - api_url: 'your-slack-webhook'
```

### 4. Secure Grafana
- Change default password
- Enable HTTPS
- Configure authentication
- Set up user roles

## Troubleshooting

### No Metrics Appearing
1. Check HAL9 server is running: `curl http://localhost:8080/metrics`
2. Verify Prometheus can reach HAL9: Check Targets page
3. Check firewall rules

### High Cardinality Issues
Monitor metric cardinality:
```promql
prometheus_tsdb_symbol_table_size_bytes
```

Reduce cardinality by:
- Limiting label values
- Using recording rules
- Aggregating metrics

### Missing Dashboards
1. Verify dashboard files exist in `monitoring/grafana/dashboards/`
2. Check Grafana logs: `docker logs hal9-grafana`
3. Manually import dashboards if needed

## Best Practices

1. **Retention Policy**: Configure appropriate retention
   ```yaml
   storage.tsdb.retention.time: 30d
   ```

2. **Recording Rules**: Pre-compute expensive queries
   ```yaml
   - record: hal9:signal_rate:5m
     expr: rate(hal9_signals_processed_total[5m])
   ```

3. **Dashboard Variables**: Use Grafana variables for flexibility

4. **SLO Monitoring**: Define and track Service Level Objectives

5. **Capacity Planning**: Monitor growth trends

## Integration Examples

### Slack Notifications
```yaml
receivers:
  - name: 'slack'
    slack_configs:
      - api_url: 'YOUR_WEBHOOK_URL'
        channel: '#hal9-alerts'
        title: 'HAL9 Alert'
        text: '{{ .GroupLabels.alertname }}'
```

### PagerDuty Integration
```yaml
receivers:
  - name: 'pagerduty'
    pagerduty_configs:
      - service_key: 'YOUR_SERVICE_KEY'
        description: '{{ .GroupLabels.alertname }}'
```

### Custom Webhook
```python
from flask import Flask, request
app = Flask(__name__)

@app.route('/webhook', methods=['POST'])
def webhook():
    alert = request.json
    # Process alert
    return "OK"
```

## Monitoring Costs

Track monitoring overhead:
- Prometheus memory: ~2GB per million series
- Grafana memory: ~512MB base
- Network bandwidth: ~1KB per metric scrape

## Advanced Topics

### Federation
For multi-cluster setups:
```yaml
- job_name: 'federate'
  honor_labels: true
  metrics_path: '/federate'
  params:
    'match[]':
      - '{job="hal9-server"}'
```

### Custom Metrics
Add custom metrics in your neurons:
```rust
CUSTOM_METRIC.with_label_values(&["label"]).inc();
```

### PromQL Optimization
- Use recording rules for complex queries
- Aggregate before rate()
- Limit time ranges in queries