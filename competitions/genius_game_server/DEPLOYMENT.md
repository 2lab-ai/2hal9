# Genius Game Server Deployment Guide

## Overview

This guide covers deploying the Genius Game Server using Docker and Kubernetes.

## Prerequisites

- Docker 20.10+
- Kubernetes 1.26+
- kubectl
- Helm 3.0+ (optional)
- Domain name for ingress

## Quick Start with Docker Compose

```bash
# Clone the repository
git clone https://github.com/2lab-ai/2hal9.git
cd 2hal9/competitions/genius_game_server

# Start all services
docker-compose up -d

# Check logs
docker-compose logs -f genius-game-server

# Access services
# - Game API: http://localhost:8080
# - WebSocket: ws://localhost:8081
# - PostgreSQL: localhost:5432
# - Redis: localhost:6379
# - Grafana: http://localhost:3000 (admin/genius_admin)
# - Prometheus: http://localhost:9090
```

## Kubernetes Deployment

### 1. Create Namespace and Secrets

```bash
# Create namespace
kubectl apply -f k8s/namespace.yaml

# Create secrets (copy template first)
cp k8s/secrets-template.yaml k8s/secrets.yaml
# Edit k8s/secrets.yaml with your values
kubectl apply -f k8s/secrets.yaml
```

### 2. Deploy with Kustomize

```bash
# Deploy all resources
kubectl apply -k k8s/

# Check deployment status
kubectl get all -n genius-games

# Watch pods
kubectl get pods -n genius-games -w

# Check logs
kubectl logs -f deployment/genius-game-server -n genius-games
```

### 3. Configure Ingress

Update `k8s/ingress.yaml` with your domain:

```yaml
spec:
  rules:
  - host: your-domain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: genius-game-server
            port:
              number: 80
```

### 4. Enable TLS with cert-manager

```bash
# Install cert-manager
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# Create ClusterIssuer
cat <<EOF | kubectl apply -f -
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: your-email@example.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF
```

## Scaling

### Horizontal Pod Autoscaling

The HPA is configured to scale between 3-20 replicas based on:
- CPU utilization (70%)
- Memory utilization (80%)
- Active WebSocket connections (100 per pod)

```bash
# Check HPA status
kubectl get hpa -n genius-games

# Manually scale
kubectl scale deployment genius-game-server --replicas=5 -n genius-games
```

### Database Scaling

For production workloads, consider:
- PostgreSQL cluster with read replicas
- Redis Cluster or Redis Sentinel
- Managed database services (RDS, Cloud SQL)

## Monitoring

### Prometheus Metrics

Available at `/metrics` endpoint:
- `genius_active_games` - Current active games
- `genius_connected_players` - Connected players
- `genius_game_duration_seconds` - Game duration histogram
- `genius_ai_decision_duration_seconds` - AI decision time
- `genius_websocket_connections` - WebSocket connections

### Grafana Dashboards

Import dashboards from `monitoring/grafana/dashboards/`:
- Game Server Overview
- AI Performance Metrics
- Resource Utilization
- Error Rates and Alerts

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level | `info` |
| `SERVER_HOST` | HTTP server host | `0.0.0.0` |
| `SERVER_PORT` | HTTP server port | `8080` |
| `WS_PORT` | WebSocket port | `8081` |
| `DATABASE_URL` | PostgreSQL connection | Required |
| `REDIS_URL` | Redis connection | Required |
| `OPENAI_API_KEY` | OpenAI API key | Optional |
| `ANTHROPIC_API_KEY` | Anthropic API key | Optional |
| `OLLAMA_ENDPOINT` | Ollama server URL | Optional |

### Game Configuration

Edit `k8s/configmap.yaml` to configure:
- Game rules and limits
- AI provider settings
- Rate limiting
- Feature flags

## Troubleshooting

### Common Issues

1. **Pods not starting**
   ```bash
   kubectl describe pod <pod-name> -n genius-games
   kubectl logs <pod-name> -n genius-games --previous
   ```

2. **Database connection errors**
   - Check secrets are correctly configured
   - Verify network policies allow connections
   - Test connection from pod:
     ```bash
     kubectl exec -it deployment/genius-game-server -n genius-games -- psql $DATABASE_URL
     ```

3. **WebSocket connection issues**
   - Ensure ingress supports WebSocket
   - Check nginx configuration:
     ```yaml
     nginx.ingress.kubernetes.io/websocket-services: "genius-game-server"
     ```

4. **High memory usage**
   - Adjust resource limits in deployment
   - Enable connection pooling
   - Review AI model loading strategy

### Health Checks

```bash
# HTTP health check
curl http://your-domain.com/health

# WebSocket test
wscat -c ws://your-domain.com:8081/ws

# Metrics endpoint
curl http://your-domain.com/metrics
```

## Production Checklist

- [ ] Configure resource limits appropriately
- [ ] Enable persistent storage for game data
- [ ] Set up database backups
- [ ] Configure monitoring and alerting
- [ ] Enable TLS/SSL
- [ ] Set up log aggregation
- [ ] Configure rate limiting
- [ ] Enable distributed tracing
- [ ] Set up CI/CD pipeline
- [ ] Document runbooks

## Security Considerations

1. **API Keys**: Store in Kubernetes secrets, never in code
2. **Network Policies**: Restrict pod-to-pod communication
3. **RBAC**: Limit service account permissions
4. **Image Scanning**: Scan Docker images for vulnerabilities
5. **Updates**: Keep dependencies and base images updated

## Support

- GitHub Issues: https://github.com/2lab-ai/2hal9/issues
- Documentation: https://docs.genius-games.ai
- Discord: https://discord.gg/genius-games