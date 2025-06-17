# 📊 HAL9 Monitoring Guide

## Overview

HAL9 uses Prometheus and Grafana for monitoring system performance, health, and metrics.

## Components

1. **Prometheus** - Metrics collection and storage
2. **Grafana** - Visualization and dashboards
3. **Node Exporter** - System metrics
4. **cAdvisor** - Container metrics

## Quick Start

```bash
# Start monitoring stack
./scripts/start_monitoring.sh

# Access dashboards
open http://localhost:3001  # Grafana (admin/hal9admin)
open http://localhost:9091  # Prometheus
```

## Metrics Collected

### HAL9 Application Metrics

| Metric | Description | Type |
|--------|-------------|------|
| `hal9_http_requests_total` | Total HTTP requests | Counter |
| `hal9_http_request_duration_seconds` | Request latency | Histogram |
| `hal9_neurons_total` | Total neurons by state | Gauge |
| `hal9_signals_processed_total` | Signals processed | Counter |
| `hal9_websocket_connections` | Active WebSocket connections | Gauge |

### System Metrics

- CPU usage per container
- Memory usage and limits
- Network I/O
- Disk I/O
- Container restart count

### Database Metrics

- PostgreSQL connections
- Query performance
- Redis operations/sec
- Cache hit rate

## Grafana Dashboards

### 1. HAL9 Overview

Main dashboard showing:
- Request rate and latency
- Neuron states
- Resource usage
- Error rates

### 2. System Performance

- CPU and memory by container
- Network traffic
- Disk usage
- System load

### 3. Database Performance

- Query latency
- Connection pool status
- Cache performance
- Slow queries

## Prometheus Queries

### Useful Queries

```promql
# Request rate by endpoint
rate(hal9_http_requests_total[5m])

# P99 latency
histogram_quantile(0.99, hal9_http_request_duration_seconds_bucket)

# Error rate
rate(hal9_http_requests_total{status=~"5.."}[5m])

# Memory usage percentage
(container_memory_usage_bytes / container_spec_memory_limit_bytes) * 100

# Active neurons
hal9_neurons_total{state="active"}
```

## Alerting Rules

Create alerts in `monitoring/prometheus/alerts/`:

```yaml
groups:
- name: hal9_alerts
  rules:
  - alert: HighErrorRate
    expr: rate(hal9_http_requests_total{status=~"5.."}[5m]) > 0.05
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: High error rate detected
      description: "Error rate is {{ $value }} errors/sec"

  - alert: HighMemoryUsage
    expr: (container_memory_usage_bytes / container_spec_memory_limit_bytes) > 0.9
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: Container memory usage is too high
      description: "Memory usage is {{ $value }}%"
```

## Custom Metrics

### Adding Application Metrics

```rust
use prometheus::{register_counter, register_histogram, Counter, Histogram};

lazy_static! {
    static ref REQUEST_COUNTER: Counter = register_counter!(
        "hal9_custom_metric_total",
        "Description of custom metric"
    ).unwrap();
    
    static ref LATENCY_HISTOGRAM: Histogram = register_histogram!(
        "hal9_operation_duration_seconds",
        "Operation latency in seconds"
    ).unwrap();
}

// Usage
REQUEST_COUNTER.inc();
let timer = LATENCY_HISTOGRAM.start_timer();
// ... operation ...
timer.observe_duration();
```

### Exposing Metrics

Metrics are exposed at `/metrics` endpoint:

```rust
use prometheus::{Encoder, TextEncoder};

async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
```

## Grafana Configuration

### Adding Data Source

1. Navigate to Configuration > Data Sources
2. Add Prometheus data source
3. URL: `http://prometheus:9090`
4. Save & Test

### Importing Dashboards

1. Navigate to Create > Import
2. Upload JSON file from `monitoring/grafana/dashboards/`
3. Select Prometheus data source
4. Import

### Creating Custom Dashboards

1. Create new dashboard
2. Add panels with Prometheus queries
3. Save to `monitoring/grafana/dashboards/`
4. Export as JSON for version control

## Production Considerations

### 1. Data Retention

Configure Prometheus retention:
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
storage:
  tsdb:
    retention.time: 30d
    retention.size: 10GB
```

### 2. High Availability

- Run multiple Prometheus instances
- Use remote storage (Thanos, Cortex)
- Grafana clustering

### 3. Security

- Enable authentication
- Use TLS for metrics endpoints
- Restrict network access

### 4. Resource Planning

| Component | CPU | Memory | Storage |
|-----------|-----|--------|---------|
| Prometheus | 2 cores | 4GB | 100GB |
| Grafana | 1 core | 2GB | 10GB |
| Node Exporter | 0.5 core | 128MB | - |
| cAdvisor | 0.5 core | 256MB | - |

## Troubleshooting

### Prometheus Not Scraping

1. Check targets: http://localhost:9091/targets
2. Verify network connectivity
3. Check metric endpoint: `curl http://hal9-server:9091/metrics`

### Grafana Not Showing Data

1. Test data source connection
2. Check time range
3. Verify Prometheus queries

### High Memory Usage

1. Reduce retention period
2. Increase scrape interval
3. Use recording rules for complex queries

## Useful Links

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)
- [PromQL Tutorial](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [Grafana Dashboard Gallery](https://grafana.com/grafana/dashboards/)