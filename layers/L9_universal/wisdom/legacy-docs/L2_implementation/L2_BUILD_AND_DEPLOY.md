# HAL9 Build and Deploy Guide

**Level**: L2 Implementation  
**Audience**: DevOps Engineers, Developers  
**Purpose**: Step-by-step build and deployment instructions

## Build Prerequisites

- Rust 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Docker 20.10+ (`docker --version`)
- Kubernetes 1.25+ (`kubectl version`)
- PostgreSQL 14+ (for distributed mode)
- Redis 7+ (for caching)

## Local Development Build

### Step 1: Clone Repository
```bash
git clone https://github.com/2lab/2hal9.git
cd 2hal9
```

### Step 2: Build Dependencies
```bash
# Install build dependencies
cargo fetch

# Build all workspace members
cargo build --workspace

# Build with all features
cargo build --all-features
```

### Step 3: Run Tests
```bash
# Run all tests
cargo test --workspace

# Run with hierarchical feature
cargo test --features hierarchical

# Run benchmarks
cargo bench
```

### Step 4: Build Release Binary
```bash
# Build optimized binary
cargo build --release

# Build with specific features
cargo build --release --features "hierarchical,distributed"

# Output location
ls -la target/release/hal9-server
```

## Docker Build

### Step 1: Build Docker Image
```dockerfile
# Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Build dependencies separately for caching
RUN cargo build --release --workspace

# Final stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/hal9-server /usr/local/bin/
COPY --from=builder /app/config /etc/hal9/

EXPOSE 8080 9090

CMD ["hal9-server"]
```

### Step 2: Build Image
```bash
# Build image
docker build -t hal9:latest .

# Build with build args
docker build \
  --build-arg FEATURES="hierarchical,distributed" \
  -t hal9:hierarchical .

# Multi-platform build
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t hal9:latest \
  --push .
```

## Kubernetes Deployment

### Step 1: Create Namespace
```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: hal9
  labels:
    name: hal9
```

### Step 2: ConfigMap
```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: hal9-config
  namespace: hal9
data:
  config.yaml: |
    substrate:
      type: distributed
      runtime: tokio
      transport: tcp
      storage: postgres
    
    neurons:
      - type: strategic
        id: L5-001
        replicas: 3
      
      - type: tactical
        id: L4-001
        replicas: 5
```

### Step 3: Deployment
```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hal9-server
  namespace: hal9
spec:
  replicas: 3
  selector:
    matchLabels:
      app: hal9-server
  template:
    metadata:
      labels:
        app: hal9-server
    spec:
      containers:
      - name: hal9
        image: hal9:latest
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: HAL9_CONFIG
          value: /etc/hal9/config.yaml
        - name: RUST_LOG
          value: info
        volumeMounts:
        - name: config
          mountPath: /etc/hal9
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
      volumes:
      - name: config
        configMap:
          name: hal9-config
```

### Step 4: Service
```yaml
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: hal9-service
  namespace: hal9
spec:
  selector:
    app: hal9-server
  ports:
  - port: 80
    targetPort: 8080
    name: http
  - port: 9090
    targetPort: 9090
    name: metrics
  type: LoadBalancer
```

### Step 5: Deploy to Kubernetes
```bash
# Apply configurations
kubectl apply -f namespace.yaml
kubectl apply -f configmap.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml

# Check deployment
kubectl get pods -n hal9
kubectl get svc -n hal9

# View logs
kubectl logs -n hal9 -l app=hal9-server -f
```

## Production Deployment

### Pre-Deployment Checklist
- [ ] All tests passing
- [ ] Security scan completed
- [ ] Performance benchmarks met
- [ ] Documentation updated
- [ ] Rollback plan ready
- [ ] Monitoring configured

### Step 1: Database Setup
```sql
-- Create database
CREATE DATABASE hal9_prod;

-- Create user
CREATE USER hal9_user WITH PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE hal9_prod TO hal9_user;

-- Run migrations
./scripts/migrate.sh up
```

### Step 2: Infrastructure Setup
```bash
# Create Redis cluster
redis-cli --cluster create \
  redis1:6379 redis2:6379 redis3:6379 \
  --cluster-replicas 1

# Verify cluster
redis-cli --cluster check redis1:6379
```

### Step 3: Deploy with Helm
```bash
# Add Helm repository
helm repo add hal9 https://charts.hal9.ai
helm repo update

# Install with custom values
helm install hal9 hal9/hal9 \
  --namespace hal9 \
  --create-namespace \
  --values production-values.yaml

# production-values.yaml
replicaCount: 5
image:
  repository: hal9
  tag: v3.0.0
  pullPolicy: IfNotPresent

substrate:
  type: distributed
  storage:
    type: postgres
    host: postgres.hal9.svc.cluster.local
    database: hal9_prod

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 80

monitoring:
  enabled: true
  prometheus:
    enabled: true
  grafana:
    enabled: true
```

### Step 4: Zero-Downtime Deployment
```bash
# Blue-Green Deployment
./scripts/deploy-blue-green.sh

# Canary Deployment
./scripts/deploy-canary.sh --percentage 10

# Rolling Update
kubectl set image deployment/hal9-server \
  hal9=hal9:v3.0.0 \
  -n hal9 \
  --record
```

## Monitoring Setup

### Prometheus Configuration
```yaml
# prometheus-config.yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'hal9'
    kubernetes_sd_configs:
    - role: pod
      namespaces:
        names:
        - hal9
    relabel_configs:
    - source_labels: [__meta_kubernetes_pod_label_app]
      action: keep
      regex: hal9-server
```

### Grafana Dashboard
```json
{
  "dashboard": {
    "title": "HAL9 Hierarchical Metrics",
    "panels": [
      {
        "title": "Layer Latencies",
        "targets": [
          {
            "expr": "histogram_quantile(0.99, hal9_layer_latency_bucket)"
          }
        ]
      }
    ]
  }
}
```

## CI/CD Pipeline

### GitHub Actions
```yaml
# .github/workflows/deploy.yml
name: Deploy HAL9

on:
  push:
    branches: [main]
    tags: ['v*']

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Build Docker image
      run: |
        docker build -t hal9:${{ github.sha }} .
        docker tag hal9:${{ github.sha }} hal9:latest
    
    - name: Push to Registry
      run: |
        echo ${{ secrets.DOCKER_PASSWORD }} | docker login -u ${{ secrets.DOCKER_USERNAME }} --password-stdin
        docker push hal9:${{ github.sha }}
        docker push hal9:latest
    
    - name: Deploy to Kubernetes
      run: |
        kubectl set image deployment/hal9-server hal9=hal9:${{ github.sha }} -n hal9
        kubectl rollout status deployment/hal9-server -n hal9
```

## Rollback Procedures

### Automatic Rollback
```bash
# Set rollback conditions
kubectl set env deployment/hal9-server \
  ROLLBACK_ON_ERROR=true \
  ERROR_THRESHOLD=0.05 \
  -n hal9
```

### Manual Rollback
```bash
# View rollout history
kubectl rollout history deployment/hal9-server -n hal9

# Rollback to previous version
kubectl rollout undo deployment/hal9-server -n hal9

# Rollback to specific revision
kubectl rollout undo deployment/hal9-server --to-revision=3 -n hal9
```

## Post-Deployment Verification

### Health Checks
```bash
# Check pod health
kubectl get pods -n hal9 -o wide

# Check service endpoints
kubectl get endpoints -n hal9

# Test API
curl http://hal9.example.com/health

# Run smoke tests
./scripts/smoke-tests.sh
```

### Performance Verification
```bash
# Run load test
./scripts/load-test.sh --duration 5m --rate 1000

# Check metrics
curl http://hal9.example.com:9090/metrics | grep hal9_

# Verify SLOs
./scripts/verify-slos.sh
```

## Troubleshooting

### Common Issues

#### Pod CrashLoopBackOff
```bash
# Check logs
kubectl logs -n hal9 pod-name --previous

# Describe pod
kubectl describe pod -n hal9 pod-name

# Check events
kubectl get events -n hal9 --sort-by='.lastTimestamp'
```

#### High Memory Usage
```bash
# Check resource usage
kubectl top pods -n hal9

# Increase limits
kubectl patch deployment hal9-server -n hal9 -p '{"spec":{"template":{"spec":{"containers":[{"name":"hal9","resources":{"limits":{"memory":"4Gi"}}}]}}}}'
```

#### Connection Issues
```bash
# Test connectivity
kubectl exec -n hal9 pod-name -- nc -zv postgres.hal9.svc.cluster.local 5432

# Check network policies
kubectl get networkpolicies -n hal9
```

## Maintenance

### Regular Tasks
- [ ] Update dependencies weekly
- [ ] Rotate secrets monthly
- [ ] Review metrics daily
- [ ] Test backups weekly
- [ ] Update documentation

### Upgrade Procedure
```bash
# Test in staging first
./scripts/deploy-staging.sh v3.1.0

# Run integration tests
./scripts/integration-tests.sh

# Deploy to production
./scripts/deploy-production.sh v3.1.0 --canary 10
```

---

*"Ship early, ship often, but always ship quality."*

**For builders deploying the future.**