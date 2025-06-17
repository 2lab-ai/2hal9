# 🚀 HAL9 CI/CD Guide

## Overview

HAL9 uses GitHub Actions for continuous integration and deployment with the following workflows:

1. **CI** - Runs on every push and PR
2. **Deploy** - Deploys to staging/production
3. **PR Check** - Additional checks for pull requests

## Workflows

### 1. Continuous Integration (CI)

**File**: `.github/workflows/ci.yml`

**Triggers**:
- Push to `main` or `develop` branches
- Pull requests to `main`

**Jobs**:
- **Test**: Rust formatting, clippy, tests, and build
- **Docker**: Build multi-platform Docker images
- **Integration Test**: Run API and database tests

### 2. Deployment

**File**: `.github/workflows/deploy.yml`

**Triggers**:
- Push to `main` (staging)
- Git tags `v*` (production)
- Manual workflow dispatch

**Jobs**:
- **Build and Push**: Create and push Docker images
- **Deploy Staging**: Deploy to staging environment
- **Deploy Production**: Blue-green deployment to production
- **Rollback**: Automatic rollback on failure

### 3. Pull Request Checks

**File**: `.github/workflows/pr-check.yml`

**Triggers**:
- Pull request events

**Jobs**:
- **Lint**: Rust formatting and clippy
- **Security**: Dependency vulnerability scan
- **Test Coverage**: Code coverage report
- **Size Check**: Binary size monitoring
- **Docker Check**: Container security scan

## Setup Requirements

### Secrets Configuration

Add these secrets to your GitHub repository:

```yaml
# Docker Hub
DOCKER_USERNAME
DOCKER_PASSWORD

# Kubernetes (base64 encoded kubeconfig)
STAGING_KUBECONFIG
PRODUCTION_KUBECONFIG

# Notifications
SLACK_WEBHOOK
```

### Branch Protection

Configure branch protection for `main`:
- Require PR reviews
- Require status checks to pass:
  - `Test`
  - `Lint and Format`
  - `Security Audit`
- Require branches to be up to date

## Deployment Process

### Staging Deployment

1. Push to `main` branch
2. CI workflow runs tests
3. Docker image is built and pushed
4. Deployment to staging is automatic
5. Smoke tests verify deployment

### Production Deployment

1. Create and push a git tag:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```
2. Staging deployment runs first
3. Manual approval required in GitHub
4. Blue-green deployment to production
5. Automatic rollback if tests fail

### Manual Deployment

```bash
# Deploy to staging
gh workflow run deploy.yml -f environment=staging

# Deploy to production
gh workflow run deploy.yml -f environment=production
```

## Kubernetes Configuration

### Staging Environment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hal9-server
  namespace: hal9-staging
spec:
  replicas: 2
  selector:
    matchLabels:
      app: hal9-server
  template:
    metadata:
      labels:
        app: hal9-server
    spec:
      containers:
      - name: hal9-server
        image: hal9/hal9-server:main
        ports:
        - containerPort: 8080
```

### Production Environment (Blue-Green)

```yaml
# Blue deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hal9-server-blue
  namespace: hal9-production
spec:
  replicas: 5
  selector:
    matchLabels:
      app: hal9-server
      version: blue
  # ... rest of config

# Green deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hal9-server-green
  namespace: hal9-production
spec:
  replicas: 5
  selector:
    matchLabels:
      app: hal9-server
      version: green
  # ... rest of config

# Service (switches between blue/green)
apiVersion: v1
kind: Service
metadata:
  name: hal9-server
  namespace: hal9-production
spec:
  selector:
    app: hal9-server
    version: blue  # or green
  ports:
  - port: 80
    targetPort: 8080
```

## Monitoring

### Build Status

- Check Actions tab in GitHub
- Status badges in README
- Slack notifications on failure

### Deployment Metrics

Monitor these after deployment:
- Response time
- Error rate
- CPU/Memory usage
- Active connections

## Troubleshooting

### Common Issues

1. **Docker build fails**
   - Check Rust version compatibility
   - Verify all dependencies in Cargo.lock

2. **Kubernetes deployment stuck**
   - Check pod logs: `kubectl logs -n hal9-staging -l app=hal9-server`
   - Verify secrets are configured

3. **Tests fail in CI**
   - Run locally: `cargo test --workspace`
   - Check for flaky tests

### Rollback Procedures

1. **Automatic rollback** (production only)
   - Triggered on deployment test failure
   - Switches traffic back to previous version

2. **Manual rollback**
   ```bash
   # Switch traffic to green (previous version)
   kubectl patch service hal9-server \
     -p '{"spec":{"selector":{"version":"green"}}}' \
     -n hal9-production
   ```

## Best Practices

1. **Version Tags**
   - Use semantic versioning: `v1.2.3`
   - Tag only tested commits

2. **Docker Images**
   - Multi-stage builds for size optimization
   - Security scanning with Trivy

3. **Testing**
   - Write tests for new features
   - Maintain >80% code coverage

4. **Monitoring**
   - Check metrics after deployment
   - Set up alerts for anomalies

## Future Improvements

1. **GitOps Integration**
   - ArgoCD for declarative deployments
   - Automated rollbacks based on metrics

2. **Advanced Testing**
   - Load testing in staging
   - Chaos engineering tests

3. **Multi-Region Deployment**
   - Geographic distribution
   - Automated failover