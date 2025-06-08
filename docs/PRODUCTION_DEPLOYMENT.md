# HAL9 Production Deployment Guide

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Environment Setup](#environment-setup)
3. [Configuration](#configuration)
4. [Deployment Methods](#deployment-methods)
5. [Monitoring Setup](#monitoring-setup)
6. [Security Best Practices](#security-best-practices)
7. [Backup and Recovery](#backup-and-recovery)
8. [Troubleshooting](#troubleshooting)

## Prerequisites

- Docker 20.10+ and Docker Compose 2.0+
- 4GB RAM minimum (8GB recommended)
- 20GB disk space
- Claude API key (for production mode)
- SSL certificates for HTTPS

## Environment Setup

### 1. Clone Repository
```bash
git clone https://github.com/yourusername/hal9.git
cd hal9
```

### 2. Environment Variables
Create `.env` file:
```bash
# Claude API Configuration
ANTHROPIC_API_KEY=your-api-key-here

# Server Configuration
RUST_LOG=info
HAL9_ENV=production

# Cost Controls
MAX_COST_PER_HOUR=10.0
MAX_COST_PER_DAY=100.0

# Security
JWT_SECRET=generate-strong-secret
ADMIN_PASSWORD=secure-password
```

### 3. Production Configuration
Create `config/production.yaml`:
```yaml
server_id: "hal9-prod-01"
neurons:
  # L4 Strategic Layer
  - id: "prod-l4-strategic"
    layer: "L4"
    claude_command: "claude"
    forward_connections: ["prod-l3-arch", "prod-l3-design"]
    settings:
      temperature: 0.7
      max_tokens: 4000
      
  # L3 Architectural Layer
  - id: "prod-l3-arch"
    layer: "L3"
    claude_command: "claude"
    forward_connections: ["prod-l2-impl-1", "prod-l2-impl-2"]
    backward_connections: ["prod-l4-strategic"]
    settings:
      temperature: 0.5
      max_tokens: 3000
      
  # L2 Implementation Layer
  - id: "prod-l2-impl-1"
    layer: "L2"
    claude_command: "claude"
    backward_connections: ["prod-l3-arch"]
    settings:
      temperature: 0.3
      max_tokens: 2000

claude:
  mode: "api"
  api_key: "${ANTHROPIC_API_KEY}"
  model: "claude-3-sonnet-20240229"
  temperature: 0.5
  max_tokens: 4000
  rate_limit: 60
  fallback_to_mock: true
  cost_controls:
    max_cost_per_hour: ${MAX_COST_PER_HOUR}
    max_cost_per_day: ${MAX_COST_PER_DAY}
    max_tokens_per_request: 4000
    alert_threshold: 0.8

monitoring:
  enabled: true
  metrics_interval: 60
  log_level: "info"
```

## Deployment Methods

### Method 1: Docker Compose (Recommended)

1. **Build and Start Services**
```bash
docker-compose up -d --build
```

2. **Verify Deployment**
```bash
docker-compose ps
docker-compose logs -f hal9-server
```

3. **Check Health**
```bash
curl http://localhost:8080/health
docker exec hal9-server hal9 status
```

### Method 2: Kubernetes

1. **Create ConfigMap**
```bash
kubectl create configmap hal9-config --from-file=config/production.yaml
```

2. **Create Secret**
```bash
kubectl create secret generic hal9-secrets \
  --from-literal=ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY
```

3. **Apply Manifests**
Create `k8s/deployment.yaml`:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hal9-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: hal9
  template:
    metadata:
      labels:
        app: hal9
    spec:
      containers:
      - name: hal9-server
        image: hal9:latest
        ports:
        - containerPort: 8080
        - containerPort: 9090
        env:
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: hal9-secrets
              key: ANTHROPIC_API_KEY
        volumeMounts:
        - name: config
          mountPath: /app/config
        resources:
          requests:
            memory: "1Gi"
            cpu: "1000m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: hal9-config
```

Apply:
```bash
kubectl apply -f k8s/
```

### Method 3: Systemd (Bare Metal)

1. **Install Binary**
```bash
cargo build --release
sudo cp target/release/hal9-server /usr/local/bin/
sudo cp target/release/hal9 /usr/local/bin/
```

2. **Create Service File**
```bash
sudo tee /etc/systemd/system/hal9.service << EOF
[Unit]
Description=HAL9 AI Server
After=network.target

[Service]
Type=simple
User=hal9
Group=hal9
WorkingDirectory=/opt/hal9
Environment="RUST_LOG=info"
Environment="HAL9_CONFIG_PATH=/opt/hal9/config/production.yaml"
EnvironmentFile=/opt/hal9/.env
ExecStart=/usr/local/bin/hal9-server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF
```

3. **Start Service**
```bash
sudo systemctl daemon-reload
sudo systemctl enable hal9
sudo systemctl start hal9
```

## Monitoring Setup

### Prometheus Configuration
Create `monitoring/prometheus.yml`:
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'hal9'
    static_configs:
      - targets: ['hal9-server:9090']
    metrics_path: '/metrics'
```

### Grafana Dashboard
Import the provided dashboard from `monitoring/grafana/dashboards/hal9-dashboard.json`.

Key metrics to monitor:
- Signal processing latency (p50, p95, p99)
- Token usage and costs
- Error rates by neuron layer
- Circuit breaker status
- Memory and CPU usage

### Alerts
Configure alerts for:
- High error rate (>5%)
- Cost threshold exceeded (>80%)
- Circuit breaker open
- High latency (>100ms p95)
- Low disk space (<10%)

## Security Best Practices

### 1. API Key Management
- Store API keys in environment variables or secrets management
- Rotate keys regularly
- Use separate keys for dev/staging/prod

### 2. Network Security
- Use HTTPS with valid SSL certificates
- Implement rate limiting
- Whitelist IP addresses if possible
- Use VPN for admin access

### 3. Access Control
```nginx
# nginx/nginx.conf
server {
    listen 443 ssl;
    server_name hal9.example.com;

    ssl_certificate /etc/nginx/ssl/cert.pem;
    ssl_certificate_key /etc/nginx/ssl/key.pem;

    location /api/ {
        proxy_pass http://hal9-server:8080/;
        proxy_set_header X-Real-IP $remote_addr;
        
        # Rate limiting
        limit_req zone=api burst=10 nodelay;
    }

    location /metrics {
        allow 10.0.0.0/8;  # Internal only
        deny all;
        proxy_pass http://hal9-server:9090/metrics;
    }
}
```

### 4. Container Security
- Run as non-root user
- Use read-only root filesystem
- Scan images for vulnerabilities
- Keep base images updated

## Backup and Recovery

### 1. Backup Strategy
```bash
#!/bin/bash
# backup.sh
BACKUP_DIR="/backups/hal9/$(date +%Y%m%d_%H%M%S)"
mkdir -p $BACKUP_DIR

# Backup configuration
cp -r /opt/hal9/config $BACKUP_DIR/

# Backup logs
tar -czf $BACKUP_DIR/logs.tar.gz /opt/hal9/logs/

# Backup metrics data
docker exec hal9-prometheus tar -czf - /prometheus | cat > $BACKUP_DIR/prometheus.tar.gz

# Upload to S3
aws s3 sync $BACKUP_DIR s3://your-backup-bucket/hal9/
```

### 2. Recovery Procedure
1. Stop services
2. Restore configuration files
3. Restore data volumes
4. Start services
5. Verify functionality

## Troubleshooting

### Common Issues

1. **High Memory Usage**
```bash
# Check memory usage
docker stats hal9-server

# Increase memory limit
docker-compose down
# Edit docker-compose.yml memory limits
docker-compose up -d
```

2. **Claude API Errors**
```bash
# Check logs
docker logs hal9-server | grep ERROR

# Test API key
curl -X POST https://api.anthropic.com/v1/messages \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{"model":"claude-3-sonnet-20240229","messages":[{"role":"user","content":"test"}],"max_tokens":10}'
```

3. **Circuit Breaker Open**
```bash
# Check circuit breaker status
curl http://localhost:8080/api/health | jq '.circuit_breakers'

# Reset circuit breaker
curl -X POST http://localhost:8080/api/admin/reset-circuit-breaker
```

### Debug Mode
```bash
# Enable debug logging
export RUST_LOG=debug
docker-compose restart hal9-server

# View detailed logs
docker logs -f hal9-server --since 10m
```

### Performance Tuning
1. Adjust worker threads based on CPU cores
2. Tune connection pool sizes
3. Enable response caching for repeated queries
4. Use CDN for static assets

## Maintenance

### Weekly Tasks
- Review error logs
- Check cost reports
- Update dependencies
- Verify backups

### Monthly Tasks
- Rotate API keys
- Review and optimize neuron configuration
- Performance analysis
- Security audit

### Upgrade Procedure
1. Test new version in staging
2. Backup production data
3. Deploy with rolling update
4. Monitor for issues
5. Rollback if necessary

## Support

For issues or questions:
- GitHub Issues: https://github.com/yourusername/hal9/issues
- Documentation: https://hal9.docs.example.com
- Emergency: contact@example.com