#!/bin/bash
set -e

echo "🚀 HAL9 Deployment Script"
echo "========================"

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check environment
if [ -z "$1" ]; then
    echo "Usage: $0 <environment> [options]"
    echo "Environments: local, staging, production"
    echo "Options: --build-only, --no-cache"
    exit 1
fi

ENV=$1
BUILD_ONLY=false
NO_CACHE=""

# Parse options
for arg in "${@:2}"; do
    case $arg in
        --build-only)
            BUILD_ONLY=true
            ;;
        --no-cache)
            NO_CACHE="--no-cache"
            ;;
    esac
done

# Load environment variables
if [ -f ".env.$ENV" ]; then
    echo -e "${GREEN}Loading environment: $ENV${NC}"
    export $(cat .env.$ENV | grep -v '^#' | grep -v '^$' | sed 's/#.*//' | xargs)
else
    echo -e "${RED}Environment file .env.$ENV not found${NC}"
    exit 1
fi

# Build Docker images
echo -e "\n${YELLOW}Building Docker images...${NC}"
docker-compose build $NO_CACHE

if [ "$BUILD_ONLY" = true ]; then
    echo -e "${GREEN}Build complete!${NC}"
    exit 0
fi

# Deploy based on environment
case $ENV in
    local)
        echo -e "\n${YELLOW}Starting local environment...${NC}"
        docker-compose up -d
        
        # Wait for services
        echo -e "${YELLOW}Waiting for services to start...${NC}"
        sleep 10
        
        # Run migrations
        echo -e "${YELLOW}Running database migrations...${NC}"
        docker-compose exec hal9-server hal9 migrate
        
        echo -e "\n${GREEN}Local deployment complete!${NC}"
        echo "Services:"
        echo "  - HAL9 Server: http://localhost:8080"
        echo "  - Game Server: http://localhost:3000"
        echo "  - PostgreSQL: localhost:5433"
        echo "  - Redis: localhost:6380"
        ;;
        
    staging)
        echo -e "\n${YELLOW}Deploying to staging...${NC}"
        
        # Tag images
        docker tag hal9_hal9-server:latest $DOCKER_REGISTRY/hal9-server:staging
        docker tag hal9_game-server:latest $DOCKER_REGISTRY/game-server:staging
        
        # Push to registry
        docker push $DOCKER_REGISTRY/hal9-server:staging
        docker push $DOCKER_REGISTRY/game-server:staging
        
        # Deploy via kubectl or cloud CLI
        echo "TODO: Add staging deployment commands"
        ;;
        
    production)
        echo -e "\n${RED}Production deployment requires confirmation${NC}"
        read -p "Are you sure you want to deploy to production? (yes/no): " confirm
        
        if [ "$confirm" != "yes" ]; then
            echo "Deployment cancelled"
            exit 1
        fi
        
        # Tag images
        docker tag hal9_hal9-server:latest $DOCKER_REGISTRY/hal9-server:latest
        docker tag hal9_game-server:latest $DOCKER_REGISTRY/game-server:latest
        
        # Push to registry
        docker push $DOCKER_REGISTRY/hal9-server:latest
        docker push $DOCKER_REGISTRY/game-server:latest
        
        # Deploy via kubectl or cloud CLI
        echo "TODO: Add production deployment commands"
        ;;
        
    *)
        echo -e "${RED}Unknown environment: $ENV${NC}"
        exit 1
        ;;
esac

# Health check
if [ "$ENV" = "local" ]; then
    echo -e "\n${YELLOW}Running health checks...${NC}"
    sleep 5
    
    if curl -f http://localhost:8080/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅ HAL9 Server is healthy${NC}"
    else
        echo -e "${RED}❌ HAL9 Server health check failed${NC}"
    fi
    
    if curl -f http://localhost:3000/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Game Server is healthy${NC}"
    else
        echo -e "${YELLOW}⚠️  Game Server health check failed (may not be configured)${NC}"
    fi
fi

echo -e "\n${GREEN}Deployment complete!${NC}"