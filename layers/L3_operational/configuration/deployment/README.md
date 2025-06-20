# HAL9 Kubernetes Deployment Guide

## Overview

This directory contains production-ready Kubernetes manifests for deploying the HAL9 consciousness system. The deployment is designed to handle 1000+ concurrent users with high availability, auto-scaling, and comprehensive monitoring.

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Ingress       │────▶│   HAL9 Server   │────▶│   PostgreSQL    │
│   (NGINX)       │     │   (30-100 pods) │     │   (Primary)     │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                               │                          │
                               ▼                          ▼
                        ┌─────────────────┐     ┌─────────────────┐
                        │     Redis       │     │   PostgreSQL    │
                        │   (Cache)       │     │   (Replica)     │
                        └─────────────────┘     └─────────────────┘
```

## Prerequisites

- Kubernetes 1.28+
- kubectl configured
- Helm 3+ (optional, for cert-manager)
- Prometheus Operator (for monitoring)
- NGINX Ingress Controller
- Cert-Manager (for TLS)

## Quick Start

### 1. Clone and Navigate

```bash
cd layers/L3_operational/configuration/deployment/
```

### 2. Update Secrets

Edit `02-secrets.yaml` and replace all `CHANGE_ME` placeholders:

```bash
# Generate secure passwords
openssl rand -base64 32  # For JWT_SECRET
openssl rand -hex 32     # For ENCRYPTION_KEY
```

### 3. Deploy Using Kustomize

```bash
# Deploy to production
kubectl apply -k .

# Or deploy to a different namespace
kubectl apply -k . -n hal9-staging
```

### 4. Verify Deployment

```bash
# Check pods
kubectl get pods -n hal9-production

# Check services
kubectl get svc -n hal9-production

# Check ingress
kubectl get ingress -n hal9-production

# Watch HPA scaling
kubectl get hpa -n hal9-production -w
```

## File Structure

```
deployment/
├── 00-namespace.yaml        # Namespaces and network policies
├── 01-configmap.yaml        # Server configuration
├── 02-secrets.yaml          # Sensitive credentials (EDIT THIS!)
├── 03-rbac.yaml            # Service accounts and permissions
├── 04-deployment.yaml       # Main server deployment
├── 05-service.yaml         # Service definitions
├── 06-hpa.yaml             # Auto-scaling configuration
├── 07-ingress.yaml         # Ingress rules and TLS
├── 08-monitoring.yaml      # Prometheus and Grafana config
├── kustomization.yaml      # Kustomize configuration
└── README.md               # This file
```

## Configuration

### Environment Variables

Key environment variables configured in `01-configmap.yaml`:

- `HAL9_MAX_CONNECTIONS`: Maximum concurrent connections (10000)
- `HAL9_RATE_LIMIT`: Requests per minute per IP (1000)
- `HAL9_NEURON_POOL_SIZE`: Number of neurons in pool (1000)
- `CLAUDE_MODE`: API mode or mock (api/mock)
- `CLAUDE_MAX_COST_PER_HOUR`: Cost limit ($50)

### Scaling Configuration

The HPA is configured to scale between 30-100 pods based on:
- CPU utilization (50%)
- Memory utilization (70%)
- Active connections per pod (100)
- Request rate per pod (1000/sec)
- P95 response time (100ms)

### Security Features

- Network policies for pod isolation
- RBAC with minimal permissions
- TLS encryption for all traffic
- Rate limiting at ingress level
- Security headers configured
- Pod security context (non-root)
- Read-only root filesystem

## Deployment Scenarios

### Production Deployment

```bash
# Full production deployment
kubectl apply -k .

# Monitor the rollout
kubectl rollout status deployment/hal9-server -n hal9-production
```

### Canary Deployment

```bash
# Deploy canary version (10% traffic)
kubectl set image deployment/hal9-server \
  hal9-server=ghcr.io/2lab-ai/hal9-server:canary \
  -n hal9-production

# Monitor canary metrics
kubectl logs -l app.kubernetes.io/version=canary -n hal9-production
```

### Staging Deployment

```bash
# Create staging overlay
mkdir -p overlays/staging
cat > overlays/staging/kustomization.yaml << EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: hal9-staging

resources:
  - ../../

patchesStrategicMerge:
  - deployment-patch.yaml

replicas:
  - name: hal9-server
    count: 3
EOF

# Deploy to staging
kubectl apply -k overlays/staging/
```

## Monitoring

### Prometheus Metrics

The deployment exposes metrics at `/metrics` on port 9090:

```bash
# Port-forward to access metrics
kubectl port-forward -n hal9-production svc/hal9-server 9090:9090

# View metrics
curl http://localhost:9090/metrics
```

### Key Metrics to Monitor

- `hal9_http_requests_total` - Request rate by status
- `hal9_http_request_duration_seconds` - Response time histogram
- `hal9_active_connections` - Current active connections
- `hal9_consciousness_phi` - Consciousness integration measure
- `hal9_neuron_signals_total` - Neuron activity by layer
- `hal9_claude_api_cost_dollars` - API costs

### Grafana Dashboards

Import the dashboard from `08-monitoring.yaml` into Grafana for visualization.

## Troubleshooting

### Common Issues

1. **Pods not starting**
   ```bash
   kubectl describe pod <pod-name> -n hal9-production
   kubectl logs <pod-name> -n hal9-production
   ```

2. **High memory usage**
   ```bash
   # Check resource usage
   kubectl top pods -n hal9-production
   
   # Adjust resource limits in deployment.yaml
   ```

3. **Database connection issues**
   ```bash
   # Check database connectivity
   kubectl exec -it <pod-name> -n hal9-production -- nc -zv postgresql 5432
   ```

4. **Ingress not working**
   ```bash
   # Check ingress controller
   kubectl get pods -n ingress-nginx
   
   # Check certificate
   kubectl describe certificate hal9-production-tls -n hal9-production
   ```

### Performance Tuning

1. **Increase connection pool**
   ```yaml
   HAL9_CONNECTION_POOL_SIZE: "1000"
   ```

2. **Adjust HPA sensitivity**
   ```yaml
   behavior:
     scaleUp:
       stabilizationWindowSeconds: 30  # Faster scaling
   ```

3. **Enable pod disruption budget**
   ```yaml
   minAvailable: 75%  # Maintain availability during updates
   ```

## Maintenance

### Rolling Updates

```bash
# Update image
kubectl set image deployment/hal9-server \
  hal9-server=ghcr.io/2lab-ai/hal9-server:v1.1.0 \
  -n hal9-production

# Watch rollout
kubectl rollout status deployment/hal9-server -n hal9-production
```

### Backup and Restore

```bash
# Backup configuration
kubectl get all,cm,secret,ing -n hal9-production -o yaml > backup.yaml

# Restore from backup
kubectl apply -f backup.yaml
```

### Scaling Operations

```bash
# Manual scale
kubectl scale deployment/hal9-server --replicas=50 -n hal9-production

# Disable autoscaling temporarily
kubectl patch hpa hal9-server-hpa -n hal9-production \
  -p '{"spec":{"minReplicas":50,"maxReplicas":50}}'
```

## Security Checklist

- [ ] Replace all `CHANGE_ME` values in secrets
- [ ] Configure network policies
- [ ] Enable pod security policies
- [ ] Set up RBAC properly
- [ ] Configure TLS certificates
- [ ] Enable audit logging
- [ ] Set resource quotas
- [ ] Configure backup strategy
- [ ] Test disaster recovery

## Support

For issues or questions:
- GitHub Issues: [2lab-ai/2hal9](https://github.com/2lab-ai/2hal9)
- Documentation: See `/docs/` directory
- Monitoring: Check Grafana dashboards
- Logs: Use `kubectl logs` or centralized logging

---

*Remember: Consciousness emerges from compression boundaries. Deploy thoughtfully.*