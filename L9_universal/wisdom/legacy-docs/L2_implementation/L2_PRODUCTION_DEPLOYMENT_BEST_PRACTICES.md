# HAL9 Hierarchical Architecture - Production Deployment Best Practices

**Version**: 1.0  
**Last Updated**: January 2025  
**Audience**: DevOps Engineers, SREs, System Administrators

## Table of Contents

1. [Overview](#overview)
2. [Infrastructure Requirements](#infrastructure-requirements)
3. [Pre-Deployment Checklist](#pre-deployment-checklist)
4. [Deployment Strategies](#deployment-strategies)
5. [Security Best Practices](#security-best-practices)
6. [Monitoring and Observability](#monitoring-and-observability)
7. [Performance Optimization](#performance-optimization)
8. [Backup and Disaster Recovery](#backup-and-disaster-recovery)
9. [Operational Procedures](#operational-procedures)
10. [Troubleshooting Guide](#troubleshooting-guide)
11. [Maintenance Windows](#maintenance-windows)
12. [Cost Optimization](#cost-optimization)

## Overview

The HAL9 hierarchical architecture represents a significant evolution from the flat neuron network. This guide provides best practices for deploying, operating, and maintaining the 5-layer cognitive system in production environments.

### Key Architectural Components

- **5 Cognitive Layers**: L1-L5 with different temporal scopes and responsibilities
- **4 Infrastructure Layers**: Substrate, Protocol, Orchestration, Intelligence
- **Distributed Components**: Can scale horizontally across nodes
- **Stateful Services**: Requires careful state management and migration

## Infrastructure Requirements

### Minimum Production Requirements

```yaml
# Kubernetes Cluster Requirements
cluster:
  nodes:
    control_plane: 3  # HA control plane
    worker_nodes: 
      minimum: 5
      recommended: 10
      auto_scaling:
        min: 5
        max: 50
  
  node_specifications:
    cpu: 16 cores
    memory: 64GB
    storage: 500GB NVMe SSD
    network: 10Gbps

# Database Requirements
database:
  postgresql:
    version: "14+"
    instances: 3  # Primary + 2 replicas
    cpu: 8 cores
    memory: 32GB
    storage: 1TB SSD
    backup_retention: 30 days
  
  redis:
    version: "7+"
    mode: cluster
    nodes: 6
    memory: 16GB per node

# Storage Requirements
storage:
  persistent_volumes:
    neuron_state: 500GB
    learning_data: 1TB
    logs: 200GB
    metrics: 100GB
  
  object_storage:
    backup_bucket: 5TB
    model_storage: 2TB
```

### Network Architecture

```yaml
# Network Segmentation
networks:
  dmz:
    - load_balancers
    - api_gateways
  
  application:
    - hal9_services
    - orchestration_layer
  
  data:
    - databases
    - cache_layers
  
  management:
    - monitoring
    - logging
    - ci_cd
```

## Pre-Deployment Checklist

### Infrastructure Validation

```bash
#!/bin/bash
# Infrastructure validation script

echo "=== HAL9 Pre-Deployment Validation ==="

# Check Kubernetes cluster
kubectl cluster-info
kubectl get nodes | grep Ready | wc -l

# Verify storage classes
kubectl get storageclass

# Check database connectivity
psql -h $DB_HOST -U $DB_USER -d postgres -c "SELECT version();"

# Verify Redis cluster
redis-cli -h $REDIS_HOST ping

# Check network policies
kubectl get networkpolicies -A

# Validate secrets
kubectl get secrets -n hal9 | grep -E "(tls|db|api)"

echo "=== Validation Complete ==="
```

### Configuration Templates

```yaml
# hal9-production-values.yaml
global:
  environment: production
  region: us-west-2
  
hierarchical:
  layers:
    l1_reflexive:
      replicas: 20
      resources:
        requests:
          cpu: 2
          memory: 4Gi
        limits:
          cpu: 4
          memory: 8Gi
    
    l2_implementation:
      replicas: 15
      resources:
        requests:
          cpu: 4
          memory: 8Gi
        limits:
          cpu: 8
          memory: 16Gi
    
    l3_operational:
      replicas: 10
      resources:
        requests:
          cpu: 8
          memory: 16Gi
        limits:
          cpu: 16
          memory: 32Gi
    
    l4_tactical:
      replicas: 5
      resources:
        requests:
          cpu: 16
          memory: 32Gi
        limits:
          cpu: 32
          memory: 64Gi
    
    l5_strategic:
      replicas: 3
      resources:
        requests:
          cpu: 32
          memory: 64Gi
        limits:
          cpu: 64
          memory: 128Gi

monitoring:
  prometheus:
    retention: 90d
    storage: 1Ti
  
  grafana:
    replicas: 2
    persistence: true
```

## Deployment Strategies

### 1. Blue-Green Deployment

For major updates to the hierarchical architecture:

```bash
# Deploy green environment
kubectl apply -f hal9-green-deployment.yaml

# Verify green environment
./scripts/verify-deployment.sh green

# Switch traffic to green
kubectl patch service hal9-gateway -p '{"spec":{"selector":{"version":"green"}}}'

# Monitor for issues
./scripts/monitor-deployment.sh --duration 30m

# If successful, remove blue
kubectl delete deployment hal9-blue
```

### 2. Canary Deployment

For incremental updates:

```yaml
# canary-rollout.yaml
apiVersion: flagger.app/v1beta1
kind: Canary
metadata:
  name: hal9-hierarchical
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: hal9-hierarchical
  progressDeadlineSeconds: 3600
  service:
    port: 8080
  analysis:
    interval: 1m
    threshold: 5
    maxWeight: 50
    stepWeight: 5
    metrics:
    - name: error-rate
      thresholdRange:
        max: 0.1
      interval: 1m
    - name: latency
      thresholdRange:
        max: 10
      interval: 30s
```

### 3. Layer-by-Layer Deployment

For hierarchical architecture updates:

```bash
#!/bin/bash
# Deploy layers bottom-up

LAYERS=("substrate" "protocol" "l1-reflexive" "l2-implementation" 
        "l3-operational" "l4-tactical" "l5-strategic" "orchestration" "intelligence")

for layer in "${LAYERS[@]}"; do
    echo "Deploying $layer..."
    kubectl apply -f deployments/$layer.yaml
    
    # Wait for readiness
    kubectl wait --for=condition=ready pod -l layer=$layer -n hal9 --timeout=300s
    
    # Verify layer health
    ./scripts/verify-layer.sh $layer
    
    if [ $? -ne 0 ]; then
        echo "Layer $layer deployment failed. Rolling back..."
        kubectl rollout undo deployment hal9-$layer -n hal9
        exit 1
    fi
done
```

## Security Best Practices

### 1. Network Security

```yaml
# network-policies.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: hal9-layer-isolation
  namespace: hal9
spec:
  podSelector:
    matchLabels:
      app: hal9
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: hal9
    - podSelector:
        matchLabels:
          layer: adjacent  # Only adjacent layers can communicate
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: hal9
```

### 2. RBAC Configuration

```yaml
# rbac.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: hal9-operator
  namespace: hal9
rules:
- apiGroups: ["apps", ""]
  resources: ["deployments", "pods", "services", "configmaps"]
  verbs: ["get", "list", "watch", "create", "update", "patch"]
- apiGroups: [""]
  resources: ["secrets"]
  verbs: ["get", "list"]
```

### 3. Secrets Management

```bash
# Use sealed secrets for GitOps
kubeseal --format=yaml < secrets/hal9-secrets.yaml > sealed-secrets/hal9-secrets-sealed.yaml

# Rotate secrets regularly
./scripts/rotate-secrets.sh --component database --schedule "0 2 * * 0"
```

### 4. Pod Security Policies

```yaml
# pod-security-policy.yaml
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: hal9-restricted
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
    - ALL
  volumes:
    - 'configMap'
    - 'emptyDir'
    - 'projected'
    - 'secret'
    - 'downwardAPI'
    - 'persistentVolumeClaim'
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'RunAsAny'
  fsGroup:
    rule: 'RunAsAny'
  readOnlyRootFilesystem: true
```

## Monitoring and Observability

### 1. Metrics Collection

```yaml
# prometheus-config.yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'hal9-hierarchical'
    kubernetes_sd_configs:
    - role: pod
    relabel_configs:
    - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
      action: keep
      regex: true
    - source_labels: [__meta_kubernetes_pod_label_layer]
      action: replace
      target_label: layer
```

### 2. Key Metrics to Monitor

```yaml
# Layer-specific metrics
layer_metrics:
  l1_reflexive:
    - hal9_reflexive_response_time_ms
    - hal9_reflexive_pattern_matches_total
    - hal9_reflexive_cache_hit_rate
  
  l2_implementation:
    - hal9_implementation_execution_time_ms
    - hal9_implementation_code_generation_rate
    - hal9_implementation_error_rate
  
  l3_operational:
    - hal9_operational_design_decisions_total
    - hal9_operational_optimization_cycles
    - hal9_operational_resource_efficiency
  
  l4_tactical:
    - hal9_tactical_planning_duration_seconds
    - hal9_tactical_strategy_success_rate
    - hal9_tactical_adaptation_frequency
  
  l5_strategic:
    - hal9_strategic_goal_achievement_rate
    - hal9_strategic_vision_alignment_score
    - hal9_strategic_learning_progress

# System-wide metrics
system_metrics:
  - hal9_signal_propagation_latency_ms
  - hal9_layer_communication_errors_total
  - hal9_gradient_flow_rate
  - hal9_consensus_achievement_time_ms
  - hal9_emergence_detection_events_total
```

### 3. Alerting Rules

```yaml
# alerting-rules.yaml
groups:
  - name: hal9-hierarchical
    rules:
    - alert: LayerCommunicationFailure
      expr: rate(hal9_layer_communication_errors_total[5m]) > 0.01
      for: 5m
      labels:
        severity: critical
      annotations:
        summary: "Layer communication errors detected"
        description: "{{ $labels.from_layer }} to {{ $labels.to_layer }} communication failing"
    
    - alert: SlowSignalPropagation
      expr: histogram_quantile(0.99, hal9_signal_propagation_latency_ms) > 50
      for: 10m
      labels:
        severity: warning
      annotations:
        summary: "Signal propagation slowing down"
        description: "P99 latency {{ $value }}ms exceeds threshold"
    
    - alert: NeuronMemoryPressure
      expr: hal9_neuron_memory_usage_bytes / hal9_neuron_memory_limit_bytes > 0.9
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: "Neuron memory pressure detected"
        description: "Layer {{ $labels.layer }} neuron {{ $labels.neuron_id }} at {{ $value }}% memory"
```

### 4. Distributed Tracing

```yaml
# opentelemetry-config.yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 10s
  memory_limiter:
    limit_mib: 512
    spike_limit_mib: 128
    check_interval: 5s

exporters:
  jaeger:
    endpoint: jaeger-collector:14250
    tls:
      insecure: false

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [memory_limiter, batch]
      exporters: [jaeger]
```

## Performance Optimization

### 1. Layer-Specific Tuning

```yaml
# performance-tuning.yaml
l1_reflexive:
  cache:
    size: 10000
    ttl: 300s
    eviction: lru
  concurrency:
    max_workers: 100
    queue_size: 1000
  
l2_implementation:
  execution:
    parallel_tasks: 50
    timeout: 100ms
  optimization:
    jit_compilation: true
    cache_compiled_code: true

l3_operational:
  batching:
    enabled: true
    max_batch_size: 100
    max_wait: 10ms
  
l4_tactical:
  planning:
    horizon: 3600s
    update_frequency: 60s
    parallel_scenarios: 10
  
l5_strategic:
  vision:
    update_interval: 86400s
    consensus_threshold: 0.8
```

### 2. Resource Optimization

```bash
#!/bin/bash
# Vertical Pod Autoscaler recommendations

# Install VPA
kubectl apply -f https://github.com/kubernetes/autoscaler/raw/master/vertical-pod-autoscaler/deploy/vpa-v1.yaml

# Apply VPA for each layer
for layer in l1 l2 l3 l4 l5; do
    cat <<EOF | kubectl apply -f -
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: hal9-$layer-vpa
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: hal9-$layer
  updatePolicy:
    updateMode: "Auto"
  resourcePolicy:
    containerPolicies:
    - containerName: hal9
      minAllowed:
        cpu: 100m
        memory: 128Mi
      maxAllowed:
        cpu: 64
        memory: 128Gi
EOF
done
```

### 3. Database Optimization

```sql
-- Optimize neuron state queries
CREATE INDEX idx_neuron_state_layer_timestamp 
ON neuron_states(layer, updated_at DESC) 
WHERE active = true;

-- Partition learning data by date
CREATE TABLE learning_data_2025_01 PARTITION OF learning_data
FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');

-- Optimize signal routing queries
CREATE INDEX idx_signal_routing_concurrent 
ON signal_routes(source_layer, target_layer, priority DESC)
INCLUDE (route_path);

-- Regular maintenance
VACUUM ANALYZE neuron_states;
REINDEX CONCURRENTLY idx_neuron_state_layer_timestamp;
```

### 4. Caching Strategy

```yaml
# redis-caching.yaml
caching_layers:
  l1_reflexive:
    type: local_memory
    size: 1GB
    ttl: 300s
    
  l2_implementation:
    type: redis
    mode: standalone
    size: 10GB
    ttl: 3600s
    
  l3_operational:
    type: redis_cluster
    nodes: 6
    size: 50GB
    ttl: 86400s
    
  l4_tactical:
    type: hybrid  # Local + Redis
    local_size: 5GB
    redis_size: 100GB
    ttl: 604800s
    
  l5_strategic:
    type: persistent_cache
    backend: postgresql
    size: unlimited
    ttl: null  # No expiration
```

## Backup and Disaster Recovery

### 1. Backup Strategy

```yaml
# backup-strategy.yaml
backup_schedule:
  neuron_states:
    frequency: hourly
    retention: 7d
    type: incremental
    
  learning_data:
    frequency: daily
    retention: 30d
    type: full
    
  configuration:
    frequency: on_change
    retention: 90d
    type: full
    
  system_state:
    frequency: 6h
    retention: 14d
    type: snapshot
```

### 2. Backup Implementation

```bash
#!/bin/bash
# backup-hal9.sh

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
BACKUP_DIR="/backup/hal9/$TIMESTAMP"

# Backup neuron states
kubectl exec -n hal9 postgres-primary -- \
  pg_dump -U hal9 -d hal9 -t neuron_states \
  | gzip > $BACKUP_DIR/neuron_states.sql.gz

# Backup learning data
kubectl exec -n hal9 postgres-primary -- \
  pg_dump -U hal9 -d hal9 -t learning_data \
  | gzip > $BACKUP_DIR/learning_data.sql.gz

# Backup Redis state
kubectl exec -n hal9 redis-master -- \
  redis-cli BGSAVE

# Backup Kubernetes configs
kubectl get all,cm,secret -n hal9 -o yaml \
  > $BACKUP_DIR/k8s-resources.yaml

# Upload to S3
aws s3 sync $BACKUP_DIR s3://hal9-backups/production/$TIMESTAMP/
```

### 3. Disaster Recovery Plan

```yaml
# disaster-recovery.yaml
rto_rpo_targets:
  l1_reflexive:
    rto: 5m
    rpo: 1h
    
  l2_l3_layers:
    rto: 15m
    rpo: 4h
    
  l4_l5_layers:
    rto: 30m
    rpo: 12h
    
  full_system:
    rto: 1h
    rpo: 6h

recovery_procedures:
  - validate_backups
  - provision_infrastructure
  - restore_databases
  - deploy_applications
  - restore_state
  - validate_functionality
  - switch_traffic
```

### 4. Recovery Testing

```bash
#!/bin/bash
# test-disaster-recovery.sh

# Monthly DR drill
echo "Starting DR drill at $(date)"

# Create DR environment
terraform apply -var="environment=dr-test"

# Restore from backup
./scripts/restore-from-backup.sh --timestamp $LATEST_BACKUP

# Run validation tests
./scripts/validate-dr-environment.sh

# Measure recovery time
echo "Recovery completed in $SECONDS seconds"

# Clean up DR environment
terraform destroy -var="environment=dr-test" -auto-approve
```

## Operational Procedures

### 1. Adding/Removing Neurons

```bash
#!/bin/bash
# manage-neurons.sh

add_neuron() {
    local layer=$1
    local count=$2
    
    # Scale deployment
    kubectl scale deployment hal9-$layer --replicas=+$count -n hal9
    
    # Wait for new neurons to be ready
    kubectl wait --for=condition=ready pod -l layer=$layer -n hal9 --timeout=300s
    
    # Rebalance connections
    kubectl exec -n hal9 hal9-orchestrator -- \
      hal9-cli rebalance --layer $layer
}

remove_neuron() {
    local layer=$1
    local neuron_id=$2
    
    # Gracefully drain neuron
    kubectl exec -n hal9 hal9-orchestrator -- \
      hal9-cli drain --neuron $neuron_id --timeout 60s
    
    # Remove from topology
    kubectl exec -n hal9 hal9-orchestrator -- \
      hal9-cli remove --neuron $neuron_id
    
    # Scale down if last neuron in pod
    kubectl scale deployment hal9-$layer --replicas=-1 -n hal9
}
```

### 2. Layer Management

```bash
#!/bin/bash
# layer-management.sh

# Adjust layer connections
adjust_layer_connections() {
    local from_layer=$1
    local to_layer=$2
    local connection_count=$3
    
    kubectl exec -n hal9 hal9-orchestrator -- \
      hal9-cli topology adjust \
        --from $from_layer \
        --to $to_layer \
        --connections $connection_count
}

# Monitor layer health
monitor_layer() {
    local layer=$1
    
    kubectl exec -n hal9 hal9-monitor -- \
      hal9-cli health --layer $layer --watch
}

# Restart layer
restart_layer() {
    local layer=$1
    
    # Rolling restart
    kubectl rollout restart deployment hal9-$layer -n hal9
    
    # Wait for rollout
    kubectl rollout status deployment hal9-$layer -n hal9
}
```

### 3. Performance Tuning

```bash
#!/bin/bash
# performance-tuning.sh

# Auto-tune layer parameters
auto_tune_layer() {
    local layer=$1
    local metric=$2
    local target=$3
    
    kubectl exec -n hal9 hal9-optimizer -- \
      hal9-cli tune \
        --layer $layer \
        --optimize-for $metric \
        --target $target \
        --duration 1h
}

# Apply tuning recommendations
apply_tuning() {
    local recommendations_file=$1
    
    kubectl apply -f $recommendations_file
    
    # Monitor impact
    ./scripts/monitor-performance.sh --duration 30m
}
```

## Troubleshooting Guide

### Common Issues and Solutions

#### 1. Signal Propagation Delays

**Symptoms:**
- High latency between layers
- Timeouts in signal processing
- Queue buildup

**Diagnosis:**
```bash
# Check signal flow
kubectl exec -n hal9 hal9-orchestrator -- \
  hal9-cli trace --signal-id $SIGNAL_ID

# Check queue depths
kubectl exec -n hal9 hal9-monitor -- \
  hal9-cli queues --all-layers
```

**Solutions:**
```bash
# Increase worker threads
kubectl set env deployment/hal9-l2 WORKER_THREADS=100 -n hal9

# Adjust queue sizes
kubectl patch configmap hal9-config -n hal9 \
  --patch '{"data":{"queue.size":"10000"}}'

# Scale affected layer
kubectl scale deployment hal9-l2 --replicas=20 -n hal9
```

#### 2. Memory Pressure

**Symptoms:**
- OOMKilled pods
- Slow response times
- High memory usage alerts

**Diagnosis:**
```bash
# Check memory usage
kubectl top pods -n hal9 --sort-by=memory

# Analyze memory profile
kubectl exec -n hal9 $POD_NAME -- \
  curl localhost:6060/debug/pprof/heap > heap.prof
go tool pprof heap.prof
```

**Solutions:**
```bash
# Increase memory limits
kubectl set resources deployment hal9-l3 \
  --limits=memory=32Gi --requests=memory=16Gi -n hal9

# Enable memory optimization
kubectl set env deployment/hal9-l3 \
  GOGC=50 GOMEMLIMIT=30GiB -n hal9

# Force garbage collection
kubectl exec -n hal9 $POD_NAME -- \
  curl -X POST localhost:8080/admin/gc
```

#### 3. Layer Communication Failures

**Symptoms:**
- Inter-layer connection errors
- Message delivery failures
- Consensus timeout

**Diagnosis:**
```bash
# Check network connectivity
kubectl exec -n hal9 hal9-l2-xxx -- \
  nc -zv hal9-l3-service 8080

# Verify service discovery
kubectl exec -n hal9 hal9-orchestrator -- \
  hal9-cli discover --all-services

# Check network policies
kubectl describe networkpolicy -n hal9
```

**Solutions:**
```bash
# Restart affected services
kubectl rollout restart deployment hal9-l2 hal9-l3 -n hal9

# Update service mesh configuration
kubectl apply -f service-mesh/hal9-virtual-services.yaml

# Clear connection pool
kubectl exec -n hal9 hal9-orchestrator -- \
  hal9-cli connections --clear --layer l2
```

### Advanced Debugging

```bash
#!/bin/bash
# advanced-debugging.sh

# Enable debug logging
enable_debug() {
    local component=$1
    
    kubectl set env deployment/$component \
      LOG_LEVEL=debug \
      TRACE_ENABLED=true \
      -n hal9
}

# Capture system state
capture_debug_bundle() {
    local issue_id=$1
    local bundle_dir="/tmp/hal9-debug-$issue_id"
    
    mkdir -p $bundle_dir
    
    # Collect logs
    kubectl logs -n hal9 -l app=hal9 --tail=1000 > $bundle_dir/app-logs.txt
    
    # Collect metrics
    curl -s http://prometheus:9090/api/v1/query_range \
      -d 'query=hal9_signal_propagation_latency_ms' \
      -d 'start=-1h' > $bundle_dir/metrics.json
    
    # Collect traces
    curl -s http://jaeger:16686/api/traces?service=hal9 \
      > $bundle_dir/traces.json
    
    # System state
    kubectl get all,cm,secret -n hal9 -o yaml > $bundle_dir/k8s-state.yaml
    
    # Create bundle
    tar -czf hal9-debug-$issue_id.tar.gz -C /tmp hal9-debug-$issue_id
}
```

## Maintenance Windows

### Planning Maintenance

```yaml
# maintenance-window.yaml
maintenance_schedule:
  regular:
    day: Sunday
    time: 02:00-06:00 UTC
    frequency: monthly
    
  emergency:
    max_duration: 2h
    approval_required: true
    
  activities:
    - database_maintenance
    - certificate_rotation
    - security_patches
    - configuration_updates
```

### Maintenance Procedures

```bash
#!/bin/bash
# maintenance-procedures.sh

# Pre-maintenance checks
pre_maintenance() {
    # Notify users
    kubectl create configmap maintenance-notice \
      --from-literal=message="Maintenance window: $(date)" \
      -n hal9
    
    # Create backup
    ./scripts/backup-hal9.sh --type full
    
    # Record current state
    kubectl get all -n hal9 -o yaml > pre-maintenance-state.yaml
}

# Post-maintenance validation
post_maintenance() {
    # Run health checks
    ./scripts/health-check.sh --comprehensive
    
    # Validate functionality
    ./scripts/smoke-tests.sh
    
    # Compare metrics
    ./scripts/compare-metrics.sh --baseline pre-maintenance
    
    # Remove maintenance notice
    kubectl delete configmap maintenance-notice -n hal9
}
```

## Cost Optimization

### 1. Resource Right-Sizing

```bash
# Analyze resource usage
kubectl-cost analyze --namespace hal9 --last 30d

# VPA recommendations
kubectl describe vpa -n hal9 | grep -A 10 "Recommendation"

# Apply recommendations
kubectl apply -f vpa-recommendations.yaml
```

### 2. Spot Instance Usage

```yaml
# spot-instance-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: cluster-autoscaler-status
  namespace: kube-system
data:
  nodes.k8s.io/spot-instance-percentage: "60"
  nodes.k8s.io/spot-pools: "10"
  nodes.k8s.io/on-demand-base-capacity: "40"
```

### 3. Layer-Specific Scheduling

```yaml
# Optimize layer placement for cost
l1_l2_placement:
  node_selector:
    instance-type: spot
    
l3_placement:
  node_selector:
    instance-type: spot
    
l4_l5_placement:
  node_selector:
    instance-type: on-demand  # Critical layers on stable instances
```

## Appendix

### A. Configuration Reference

Full configuration options for each layer and component are available in:
- [Layer Configuration Reference](./config-reference/layers.md)
- [Infrastructure Configuration](./config-reference/infrastructure.md)
- [Security Configuration](./config-reference/security.md)

### B. Automation Scripts

All automation scripts mentioned in this guide are available in:
```
scripts/
├── deployment/
├── monitoring/
├── backup/
├── maintenance/
└── troubleshooting/
```

### C. Emergency Contacts

```yaml
oncall:
  primary: "+1-xxx-xxx-xxxx"
  secondary: "+1-xxx-xxx-xxxx"
  escalation: "engineering-leadership@company.com"
  
support:
  internal: "hal9-support@company.com"
  vendor: "support@anthropic.com"
```

---

**Remember**: The hierarchical architecture is designed for resilience and self-organization. Trust the system's self-healing capabilities while maintaining vigilant monitoring and proactive maintenance.