#!/bin/bash
# 2HAL9 Production Deployment Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DEPLOY_ENV="${DEPLOY_ENV:-production}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is not installed"
        exit 1
    fi
    
    # Check environment file
    if [ ! -f "$PROJECT_ROOT/.env" ]; then
        log_error ".env file not found. Please create it from .env.example"
        exit 1
    fi
    
    # Check API key
    if ! grep -q "ANTHROPIC_API_KEY=" "$PROJECT_ROOT/.env" || grep -q "ANTHROPIC_API_KEY=$" "$PROJECT_ROOT/.env"; then
        log_error "ANTHROPIC_API_KEY not set in .env file"
        exit 1
    fi
    
    log_info "Prerequisites check passed"
}

# Build Docker images
build_images() {
    log_info "Building Docker images..."
    
    cd "$PROJECT_ROOT"
    
    # Build with cache
    docker-compose build --parallel
    
    # Tag for production
    docker tag hal9:latest hal9:production
    
    log_info "Docker images built successfully"
}

# Run tests
run_tests() {
    log_info "Running tests..."
    
    # Run unit tests
    docker run --rm -v "$PROJECT_ROOT:/app" -w /app rust:1.75-slim \
        cargo test --all-features
    
    # Run integration tests
    docker-compose -f docker-compose.test.yml up --abort-on-container-exit
    docker-compose -f docker-compose.test.yml down
    
    log_info "All tests passed"
}

# Backup existing data
backup_data() {
    log_info "Backing up existing data..."
    
    BACKUP_DIR="$PROJECT_ROOT/backups/$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$BACKUP_DIR"
    
    # Backup config
    if [ -d "$PROJECT_ROOT/config" ]; then
        cp -r "$PROJECT_ROOT/config" "$BACKUP_DIR/"
    fi
    
    # Backup volumes if they exist
    if docker volume ls | grep -q hal9-data; then
        docker run --rm -v hal9-data:/data -v "$BACKUP_DIR:/backup" \
            alpine tar czf /backup/hal9-data.tar.gz -C /data .
    fi
    
    log_info "Backup completed: $BACKUP_DIR"
}

# Deploy application
deploy() {
    log_info "Deploying HAL9 to $DEPLOY_ENV..."
    
    cd "$PROJECT_ROOT"
    
    # Stop existing services
    log_info "Stopping existing services..."
    docker-compose down || true
    
    # Start services
    log_info "Starting services..."
    docker-compose up -d
    
    # Wait for health check
    log_info "Waiting for services to be healthy..."
    local retries=30
    while [ $retries -gt 0 ]; do
        if docker-compose exec -T hal9-server 2hal9 status &> /dev/null; then
            log_info "HAL9 server is healthy"
            break
        fi
        retries=$((retries - 1))
        sleep 2
    done
    
    if [ $retries -eq 0 ]; then
        log_error "HAL9 server failed to start"
        docker-compose logs hal9-server
        exit 1
    fi
}

# Post-deployment checks
post_deployment_checks() {
    log_info "Running post-deployment checks..."
    
    # Check API endpoint
    if curl -f -s http://localhost:8080/health > /dev/null; then
        log_info "API health check passed"
    else
        log_error "API health check failed"
        exit 1
    fi
    
    # Check metrics endpoint
    if curl -f -s http://localhost:9090/metrics > /dev/null; then
        log_info "Metrics endpoint check passed"
    else
        log_warn "Metrics endpoint check failed (non-critical)"
    fi
    
    # Show service status
    docker-compose ps
    
    log_info "Post-deployment checks completed"
}

# Setup monitoring
setup_monitoring() {
    log_info "Setting up monitoring..."
    
    # Wait for Prometheus
    local retries=30
    while [ $retries -gt 0 ]; do
        if curl -f -s http://localhost:9091/api/v1/targets > /dev/null; then
            log_info "Prometheus is ready"
            break
        fi
        retries=$((retries - 1))
        sleep 2
    done
    
    # Import Grafana dashboard
    if [ $retries -gt 0 ]; then
        log_info "Grafana is available at http://localhost:3000 (admin/admin)"
    else
        log_warn "Monitoring setup incomplete - check manually"
    fi
}

# Main deployment flow
main() {
    log_info "Starting HAL9 production deployment..."
    
    check_prerequisites
    
    # Ask for confirmation
    read -p "Deploy to $DEPLOY_ENV? This will restart all services. (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Deployment cancelled"
        exit 0
    fi
    
    # Run deployment steps
    build_images
    run_tests
    backup_data
    deploy
    post_deployment_checks
    setup_monitoring
    
    log_info "Deployment completed successfully!"
    log_info "HAL9 API: http://localhost:8080"
    log_info "Metrics: http://localhost:9090/metrics"
    log_info "Grafana: http://localhost:3000"
}

# Run main function
main "$@"