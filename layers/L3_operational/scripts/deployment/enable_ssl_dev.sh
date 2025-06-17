#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}🔒 Setting up SSL for development${NC}"
echo "======================================"
echo ""

# Check if certificates already exist
if [ -f "./ssl/self-signed/fullchain.pem" ] && [ -f "./ssl/self-signed/privkey.pem" ]; then
    echo -e "${YELLOW}ℹ️  Certificates already exist. Regenerating...${NC}"
fi

# Create directories
mkdir -p ./ssl/self-signed

# Generate self-signed certificate
echo -e "${YELLOW}🔑 Generating self-signed certificate...${NC}"
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout ./ssl/self-signed/privkey.pem \
  -out ./ssl/self-signed/fullchain.pem \
  -subj "/C=US/ST=Development/L=Local/O=HAL9/CN=localhost" \
  2>/dev/null

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Self-signed certificate created${NC}"
    echo "   Valid for: 365 days"
    echo "   CN: localhost"
else
    echo -e "${RED}❌ Failed to generate certificate${NC}"
    exit 1
fi

echo ""

# Stop existing services
echo -e "${YELLOW}🔄 Restarting services with SSL...${NC}"
docker-compose down

# Start with SSL
docker-compose -f docker-compose.yml -f docker-compose.ssl.yml up -d

echo ""
echo -e "${YELLOW}⏳ Waiting for services to start...${NC}"
sleep 10

# Test HTTPS connection
echo ""
echo -e "${YELLOW}🧪 Testing HTTPS connection...${NC}"
RESPONSE=$(curl -k -s -o /dev/null -w "%{http_code}" https://localhost/health)

if [ "$RESPONSE" = "200" ]; then
    echo -e "${GREEN}✅ HTTPS is working!${NC}"
    echo ""
    echo -e "${GREEN}🎉 SSL successfully enabled for development!${NC}"
    echo ""
    echo "Access HAL9 at:"
    echo "  - https://localhost (HTTPS)"
    echo "  - http://localhost (redirects to HTTPS)"
    echo ""
    echo -e "${YELLOW}⚠️  Note: Browser will show security warning${NC}"
    echo "This is normal for self-signed certificates."
    echo "Click 'Advanced' and 'Proceed to localhost' to continue."
else
    echo -e "${RED}❌ HTTPS test failed (HTTP $RESPONSE)${NC}"
    echo ""
    echo "Troubleshooting:"
    echo "1. Check nginx logs: docker logs hal9-nginx"
    echo "2. Check if services are running: docker ps"
    echo "3. Try accessing directly: curl -k https://localhost/health"
fi

echo ""
echo "SSL Configuration:"
echo "  - Certificate: ./ssl/self-signed/fullchain.pem"
echo "  - Private Key: ./ssl/self-signed/privkey.pem"
echo "  - Nginx Config: ./nginx/nginx.conf"
echo ""
echo "To stop: docker-compose -f docker-compose.yml -f docker-compose.ssl.yml down"