# 🚀 HAL9 Deployment Checklist

## Pre-Deployment Verification

### 1. Docker Build Success ⏳
- [ ] Docker build completes without errors
- [ ] All workspace members compile successfully
- [ ] Binary artifacts created: `hal9-server`, `hal9`

### 2. Local Testing
```bash
# Run local deployment
./scripts/deploy.sh local

# Check service health
curl http://localhost:8080/health
curl http://localhost:3000/health  # Game server (if enabled)

# Verify database connection
docker-compose exec postgres psql -U hal9 -d hal9db -c "SELECT 1;"

# Check Redis
docker-compose exec redis redis-cli ping
```

### 3. Service Verification
- [ ] HAL9 Server responds on port 8080
- [ ] PostgreSQL accessible on port 5432
- [ ] Redis accessible on port 6379
- [ ] Health checks passing

### 4. Database Migration
- [ ] Run migrations: `docker-compose exec hal9-server hal9 migrate`
- [ ] Verify schema created
- [ ] Test basic CRUD operations

## Production Deployment Steps

### 1. Environment Configuration
- [ ] Create production `.env.production` file
- [ ] Set secure JWT_SECRET
- [ ] Configure external database URLs
- [ ] Set proper CORS origins

### 2. Container Registry
```bash
# Tag images
docker tag hal9_hal9-server:latest <registry>/hal9-server:v1.0.0
docker tag hal9_hal9-server:latest <registry>/hal9-server:latest

# Push to registry
docker push <registry>/hal9-server:v1.0.0
docker push <registry>/hal9-server:latest
```

### 3. Kubernetes Deployment
```bash
# Create namespace and secrets
kubectl create namespace hal9
kubectl create secret generic hal9-secrets \
  --from-env-file=.env.production \
  -n hal9

# Deploy
kubectl apply -f k8s/
```

### 4. SSL/TLS Setup
- [ ] Obtain SSL certificates (Let's Encrypt)
- [ ] Configure ingress with cert-manager
- [ ] Test HTTPS endpoints

### 5. Monitoring Setup
- [ ] Deploy Prometheus
- [ ] Configure Grafana dashboards
- [ ] Set up alerts for:
  - High CPU/Memory usage
  - Error rates > 1%
  - Response time > 500ms

## Post-Deployment Verification

### 1. Smoke Tests
- [ ] API health check
- [ ] Authentication flow
- [ ] Create/read game data
- [ ] WebSocket connectivity

### 2. Load Testing
```bash
# Run k6 load test
k6 run scripts/load-test.js
```

### 3. Security Checks
- [ ] No exposed secrets in logs
- [ ] Proper CORS configuration
- [ ] Rate limiting active
- [ ] Authentication required for protected endpoints

## Rollback Plan

1. **Quick Rollback**
   ```bash
   kubectl rollout undo deployment/hal9-server -n hal9
   ```

2. **Database Rollback**
   - Keep migration rollback scripts
   - Test rollback procedure in staging

3. **DNS/Traffic Switch**
   - Keep previous version running
   - Use blue-green deployment
   - Switch traffic gradually

## Communication Plan

1. **Pre-Deployment**
   - Notify team of deployment window
   - Document expected downtime (if any)

2. **During Deployment**
   - Update status in team channel
   - Monitor error rates

3. **Post-Deployment**
   - Announce completion
   - Share metrics dashboard
   - Document any issues

## Success Criteria

- ✅ All health checks passing
- ✅ Error rate < 0.1%
- ✅ P95 latency < 100ms
- ✅ No memory leaks after 24 hours
- ✅ Auto-scaling working correctly

---

Remember: **Test in staging first!** 🧪