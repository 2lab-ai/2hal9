# HAL9 Prometheus Metrics Architecture

## Overview
This document defines the Prometheus metrics architecture for the HAL9 distributed AI consciousness system. We'll expose comprehensive metrics for monitoring system health, performance, and learning effectiveness.

## Metric Categories

### 1. System Metrics

#### Server Health
```
# Server uptime
hal9_server_uptime_seconds{server_id="string"} gauge

# Active connections
hal9_server_connections_active{server_id="string"} gauge

# Memory usage
hal9_server_memory_bytes{server_id="string", type="heap|stack"} gauge
```

#### API Metrics
```
# HTTP request total
hal9_http_requests_total{method="GET|POST", endpoint="/path", status="200|400|500"} counter

# Request duration
hal9_http_request_duration_seconds{method="GET|POST", endpoint="/path"} histogram

# Active websocket connections
hal9_websocket_connections_active gauge
```

### 2. Neuron Metrics

#### Neuron State
```
# Neuron health status (1=healthy, 0=unhealthy)
hal9_neuron_health{neuron_id="string", layer="L4|L3|L2"} gauge

# Neuron state
hal9_neuron_state{neuron_id="string", state="idle|processing|error"} gauge

# Processing queue size
hal9_neuron_queue_size{neuron_id="string"} gauge
```

#### Signal Processing
```
# Signals processed total
hal9_signals_processed_total{neuron_id="string", layer="L4|L3|L2", status="success|error"} counter

# Signal processing duration
hal9_signal_processing_duration_seconds{neuron_id="string", layer="L4|L3|L2"} histogram

# Signals forwarded
hal9_signals_forwarded_total{from_neuron="string", to_neuron="string"} counter
```

### 3. Claude Integration Metrics

#### API Usage
```
# Claude API calls total
hal9_claude_api_calls_total{mode="real|mock", model="string", status="success|error"} counter

# API call duration
hal9_claude_api_duration_seconds{mode="real|mock", model="string"} histogram

# Token usage
hal9_claude_tokens_used_total{type="prompt|completion", model="string"} counter
```

#### Cost Tracking
```
# API cost in dollars
hal9_claude_cost_dollars_total{model="string"} counter

# Cost rate (dollars per hour)
hal9_claude_cost_rate_dollars_per_hour gauge

# Budget remaining
hal9_claude_budget_remaining_dollars{period="hour|day|month"} gauge
```

### 4. Memory System Metrics

#### Storage
```
# Memory entries total
hal9_memory_entries_total{neuron_id="string"} gauge

# Memory database size
hal9_memory_database_bytes gauge

# Memory operations
hal9_memory_operations_total{operation="store|retrieve|search", status="success|error"} counter
```

#### Performance
```
# Memory operation duration
hal9_memory_operation_duration_seconds{operation="store|retrieve|search"} histogram

# Cache hit rate
hal9_memory_cache_hits_total counter
hal9_memory_cache_misses_total counter
```

### 5. Learning Metrics

#### Backward Propagation
```
# Learning cycles completed
hal9_learning_cycles_total counter

# Error gradient magnitude
hal9_learning_error_gradient{neuron_id="string"} gauge

# Prompt adjustments made
hal9_learning_adjustments_total{neuron_id="string", type="success|failure"} counter
```

#### Pattern Recognition
```
# Patterns identified
hal9_learning_patterns_identified_total{pattern_type="string"} counter

# Learning effectiveness (success rate)
hal9_learning_effectiveness_ratio{neuron_id="string"} gauge
```

### 6. MCP Tools Metrics

#### Tool Usage
```
# Tool invocations total
hal9_mcp_tool_invocations_total{tool="file_read|file_write|shell|web_fetch", status="success|error"} counter

# Tool execution duration
hal9_mcp_tool_duration_seconds{tool="string"} histogram

# Security violations attempted
hal9_mcp_security_violations_total{tool="string", violation_type="path_traversal|forbidden_command"} counter
```

### 7. Authentication Metrics

#### User Activity
```
# Active users
hal9_auth_users_active gauge

# Authentication attempts
hal9_auth_attempts_total{type="login|api_key", status="success|failure"} counter

# JWT tokens issued
hal9_auth_tokens_issued_total{type="access|refresh"} counter
```

#### API Keys
```
# Active API keys
hal9_auth_api_keys_active gauge

# API key usage
hal9_auth_api_key_usage_total{key_id="string"} counter
```

### 8. Network Metrics (Distributed Mode)

#### Connectivity
```
# Connected servers
hal9_network_servers_connected gauge

# Network latency
hal9_network_latency_seconds{from_server="string", to_server="string"} gauge

# Messages exchanged
hal9_network_messages_total{direction="sent|received", type="signal|heartbeat"} counter
```

## Implementation Details

### Metric Export Format
All metrics will be exposed in Prometheus text format at `/metrics` endpoint:
```
# HELP hal9_server_uptime_seconds Server uptime in seconds
# TYPE hal9_server_uptime_seconds gauge
hal9_server_uptime_seconds{server_id="hal9-1"} 3600

# HELP hal9_signals_processed_total Total signals processed
# TYPE hal9_signals_processed_total counter
hal9_signals_processed_total{neuron_id="neuron-l4",layer="L4",status="success"} 1234
```

### Scrape Configuration
Example Prometheus configuration:
```yaml
scrape_configs:
  - job_name: 'hal9'
    static_configs:
      - targets: ['localhost:8080']
    scrape_interval: 15s
    metrics_path: '/metrics'
```

### Histogram Buckets
Default buckets for duration metrics:
- API calls: [0.01, 0.05, 0.1, 0.5, 1, 2, 5, 10, 30]
- Signal processing: [0.001, 0.01, 0.1, 0.5, 1, 5, 10, 60]
- Memory operations: [0.0001, 0.001, 0.01, 0.1, 0.5, 1]

## Grafana Dashboard Structure

### 1. System Overview Dashboard
- Server health status
- API request rate and latency
- Error rate trends
- Active connections

### 2. Neuron Performance Dashboard
- Neuron health matrix
- Signal processing rates by layer
- Processing latency heatmap
- Queue depth visualization

### 3. Claude Usage Dashboard
- API call volume
- Cost tracking and projections
- Token usage by model
- Mock vs real API usage

### 4. Learning Analytics Dashboard
- Learning cycle effectiveness
- Error gradient trends
- Pattern recognition stats
- Prompt adjustment success rate

### 5. Security Dashboard
- Authentication attempts
- API key usage patterns
- MCP tool security violations
- User activity timeline

## Alert Rules

### Critical Alerts
- Server down: `up{job="hal9"} == 0`
- High error rate: `rate(hal9_http_requests_total{status=~"5.."}[5m]) > 0.1`
- Budget exceeded: `hal9_claude_budget_remaining_dollars{period="day"} < 0`

### Warning Alerts
- High latency: `histogram_quantile(0.95, hal9_signal_processing_duration_seconds) > 5`
- Memory pressure: `hal9_memory_database_bytes > 1e9`
- Learning degradation: `hal9_learning_effectiveness_ratio < 0.5`

## Implementation Plan

1. **Phase 1**: Core metrics (system, API, neurons)
2. **Phase 2**: Integration metrics (Claude, memory, MCP)
3. **Phase 3**: Advanced metrics (learning, network)
4. **Phase 4**: Dashboards and alerts

## Best Practices

1. **Cardinality Control**: Limit label values to prevent metric explosion
2. **Naming Convention**: Use `hal9_` prefix for all metrics
3. **Units**: Include units in metric names (e.g., `_seconds`, `_bytes`)
4. **Documentation**: Include HELP text for all metrics
5. **Performance**: Use counters over gauges where possible