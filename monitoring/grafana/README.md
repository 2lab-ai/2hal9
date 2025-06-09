# HAL9 Grafana Dashboards

This directory contains comprehensive Grafana dashboards for monitoring HAL9's hierarchical architecture.

## Available Dashboards

### 1. HAL9 System Overview (`hal9-overview.json`)
- **Purpose**: General system health and performance metrics
- **Key Metrics**:
  - Server uptime and health
  - Signal processing rates
  - Active neurons count
  - Memory and resource usage
  - Claude API costs and token usage
  - Error rates by type

### 2. HAL9 Migration Dashboard (`hal9-migration.json`)
- **Purpose**: Monitor zero-downtime migration from flat to hierarchical architecture
- **Key Metrics**:
  - Current migration phase (Shadow/Canary/State/Ramp-up/Full)
  - Migration progress percentage
  - Neurons migrated vs total
  - Traffic distribution between architectures
  - Performance comparison (latency, error rates)
  - Feature flag status and rollout percentages
  - Migration events timeline

### 3. HAL9 Hierarchical Layers Performance (`hal9-hierarchical-layers.json`)
- **Purpose**: Deep dive into each layer's performance
- **Key Metrics**:
  - Layer health status (Substrate/Protocol/Cognitive/Orchestration/Intelligence)
  - Processing latency per layer (p95)
  - Message throughput per layer
  - Neuron distribution across cognitive levels
  - Inter-layer communication flow
  - Queue depth monitoring
  - Resource usage by layer

### 4. HAL9 Neurons Dashboard (`hal9-neurons.json`)
- **Purpose**: Individual neuron monitoring
- **Key Metrics**:
  - Neuron health matrix
  - Signal processing by neuron type
  - Learning effectiveness
  - Connection topology

### 5. HAL9 Learning & Security (`hal9-learning-security.json`)
- **Purpose**: Learning system and security monitoring
- **Key Metrics**:
  - Learning cycles and effectiveness
  - Pattern recognition statistics
  - Authentication attempts
  - API key usage
  - Security violations

## Setup Instructions

### Using Docker Compose

1. Start the monitoring stack:
```bash
docker-compose -f docker-compose.monitoring.yml up -d
```

2. Access Grafana at http://localhost:3000
   - Username: `admin`
   - Password: `hal9admin`

3. Dashboards will be automatically provisioned

### Manual Installation

1. Access Grafana web interface
2. Navigate to Dashboards → Import
3. Upload JSON files from `dashboards/` directory
4. Select Prometheus as the data source

## Dashboard Features

### Migration Dashboard
- **Real-time Progress**: Visual gauge showing migration completion
- **Traffic Split**: Pie chart and time series of traffic distribution
- **Performance Comparison**: Side-by-side latency and error rate graphs
- **Feature Flags**: Table view with enable/disable status
- **Event Timeline**: Recent migration events with severity

### Hierarchical Layers Dashboard
- **Layer Health Grid**: Color-coded status for each layer
- **Performance Metrics**: Latency and throughput graphs
- **Resource Usage**: Stacked graphs for CPU and memory
- **Queue Monitoring**: Early warning for bottlenecks
- **Inter-layer Flow**: Sankey diagram of message routing

## Prometheus Metrics Required

The dashboards expect these Prometheus metrics:

### Migration Metrics
```
hal9_migration_phase
hal9_migration_progress_percent
hal9_migration_neurons_migrated
hal9_migration_neurons_total
hal9_migration_health_status
hal9_migration_traffic_percent{system="flat|hierarchical"}
hal9_feature_flag_enabled{flag="..."}
hal9_feature_flag_percentage{flag="..."}
```

### Layer Metrics
```
hal9_layer_health{layer="..."}
hal9_layer_processing_duration_seconds_bucket{layer="..."}
hal9_layer_messages_processed_total{layer="..."}
hal9_layer_messages_sent_total{from_layer="...", to_layer="..."}
hal9_layer_queue_depth{layer="..."}
hal9_layer_cpu_usage_percent{layer="..."}
hal9_layer_memory_bytes{layer="..."}
```

### Neuron Metrics
```
hal9_neurons_active{neuron_type="..."}
hal9_neuron_processing_duration_seconds_bucket{neuron_type="..."}
hal9_neuron_health{neuron_id="...", layer="..."}
```

## Customization

### Adding Panels
1. Edit dashboard in Grafana UI
2. Add panel with desired visualization
3. Export JSON and commit changes

### Modifying Queries
- All queries use Prometheus PromQL
- Adjust time ranges and aggregations as needed
- Use template variables for dynamic filtering

### Creating Alerts
1. Edit panel in dashboard
2. Go to Alert tab
3. Define alert conditions and notifications
4. Save dashboard

## Best Practices

1. **Refresh Intervals**: Set appropriate refresh rates (10s for real-time, 1m for trends)
2. **Time Windows**: Use different ranges for different use cases
3. **Variables**: Leverage dashboard variables for filtering
4. **Annotations**: Add deployment/migration event annotations
5. **Thresholds**: Configure visual thresholds for quick status assessment

## Troubleshooting

### Dashboards Not Loading
- Check Prometheus is running: http://localhost:9090
- Verify metrics are being scraped
- Check Grafana logs: `docker logs hal9-grafana`

### Missing Data
- Ensure HAL9 is exposing metrics endpoint
- Check Prometheus scrape configuration
- Verify metric names match dashboard queries

### Performance Issues
- Reduce refresh interval
- Limit time range
- Optimize PromQL queries
- Consider metric downsampling

## Contributing

When adding new dashboards:
1. Follow naming convention: `hal9-<purpose>.json`
2. Include dashboard description
3. Document required metrics
4. Test with sample data
5. Update this README

---

For more information about HAL9's monitoring architecture, see:
- [L4_PROMETHEUS_METRICS_ARCHITECTURE.md](../../docs/L4_architecture/L4_PROMETHEUS_METRICS_ARCHITECTURE.md)