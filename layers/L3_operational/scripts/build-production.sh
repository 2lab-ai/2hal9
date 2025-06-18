#!/bin/bash
# Build optimized production Docker images for HAL9

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "🚀 Building HAL9 Production Docker Image"
echo "======================================"
echo

# Check if we're in the right directory
if [ ! -f "substrate/tooling/rust/workspace.toml" ]; then
    echo -e "${RED}Error: Must run from project root directory${NC}"
    exit 1
fi

# Parse command line arguments
VERSION=${1:-latest}
PUSH=${2:-false}

echo -e "${YELLOW}Building version: $VERSION${NC}"
echo

# Build the production image
echo "📦 Building production Docker image..."
docker build \
    -f layers/L3_operational/configuration/docker/Dockerfile.production \
    -t 2lab/hal9:$VERSION \
    -t 2lab/hal9:latest \
    --build-arg BUILDKIT_INLINE_CACHE=1 \
    --cache-from 2lab/hal9:latest \
    .

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Docker image built successfully${NC}"
else
    echo -e "${RED}✗ Docker build failed${NC}"
    exit 1
fi

# Show image size
echo
echo "📊 Image size:"
docker images 2lab/hal9:$VERSION --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"

# Security scan
echo
echo "🔒 Running security scan..."
if command -v trivy &> /dev/null; then
    trivy image --severity HIGH,CRITICAL 2lab/hal9:$VERSION
else
    echo -e "${YELLOW}Trivy not installed, skipping security scan${NC}"
    echo "Install with: brew install aquasecurity/trivy/trivy"
fi

# Test the image
echo
echo "🧪 Testing the image..."
docker run --rm 2lab/hal9:$VERSION hal9 --version
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Image test passed${NC}"
else
    echo -e "${RED}✗ Image test failed${NC}"
    exit 1
fi

# Export image for air-gapped environments
echo
echo "💾 Exporting image for offline deployment..."
mkdir -p artifacts/docker
docker save 2lab/hal9:$VERSION | gzip > artifacts/docker/hal9-$VERSION.tar.gz
echo -e "${GREEN}✓ Image exported to artifacts/docker/hal9-$VERSION.tar.gz${NC}"

# Push to registry if requested
if [ "$PUSH" = "push" ]; then
    echo
    echo "📤 Pushing to Docker registry..."
    docker push 2lab/hal9:$VERSION
    docker push 2lab/hal9:latest
    echo -e "${GREEN}✓ Images pushed to registry${NC}"
fi

# Generate deployment files
echo
echo "📝 Generating deployment files..."

# Create .env template
cat > artifacts/docker/.env.production << EOF
# HAL9 Production Configuration
# Generated on $(date)

# Database
DATABASE_URL=postgresql://hal9:CHANGE_ME@postgres:5432/hal9
POSTGRES_PASSWORD=CHANGE_ME

# Claude API
CLAUDE_API_KEY=sk-ant-api03-CHANGE_ME
CLAUDE_MODE=api

# Redis
REDIS_URL=redis://redis:6379

# JWT Secret (generate with: openssl rand -base64 32)
JWT_SECRET=CHANGE_ME_USE_STRONG_SECRET_MIN_32_CHARS

# Grafana
GRAFANA_PASSWORD=CHANGE_ME

# Rate Limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_MAX_REQUESTS=60
RATE_LIMIT_WINDOW_SECONDS=60

# Logging
LOG_LEVEL=info
LOG_FORMAT=json
EOF

# Copy docker-compose file
cp layers/L3_operational/configuration/docker/docker-compose.production.yml artifacts/docker/

# Create deployment script
cat > artifacts/docker/deploy.sh << 'EOF'
#!/bin/bash
# Deploy HAL9 in production

set -e

echo "🚀 Deploying HAL9..."

# Check for .env file
if [ ! -f .env.production ]; then
    echo "Error: .env.production file not found!"
    echo "Please copy .env.production.template and configure it"
    exit 1
fi

# Load the saved image if present
if [ -f hal9-latest.tar.gz ]; then
    echo "Loading Docker image..."
    docker load < hal9-latest.tar.gz
fi

# Start services
docker-compose -f docker-compose.production.yml --env-file .env.production up -d

# Wait for services to be healthy
echo "Waiting for services to be healthy..."
sleep 30

# Check status
docker-compose -f docker-compose.production.yml ps

echo "✓ Deployment complete!"
echo
echo "Access points:"
echo "- API: http://localhost:8080"
echo "- Metrics: http://localhost:9090"
echo "- Grafana: http://localhost:3000"
EOF

chmod +x artifacts/docker/deploy.sh

echo -e "${GREEN}✓ Deployment files generated in artifacts/docker/${NC}"
echo
echo "📋 Summary:"
echo "- Docker image: 2lab/hal9:$VERSION"
echo "- Export file: artifacts/docker/hal9-$VERSION.tar.gz"
echo "- Deployment files: artifacts/docker/"
echo
echo "Next steps:"
echo "1. Copy artifacts/docker/ to your production server"
echo "2. Configure .env.production with your settings"
echo "3. Run ./deploy.sh to start services"
echo
echo -e "${GREEN}Build complete!${NC}"