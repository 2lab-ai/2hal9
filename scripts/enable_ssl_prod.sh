#!/bin/bash

# Production SSL setup with Let's Encrypt

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check arguments
if [ $# -lt 2 ]; then
    echo -e "${RED}Usage: $0 <domain> <email>${NC}"
    echo "Example: $0 hal9.example.com admin@example.com"
    exit 1
fi

DOMAIN=$1
EMAIL=$2

echo -e "${BLUE}🔒 Setting up Let's Encrypt SSL for Production${NC}"
echo "================================================"
echo "Domain: $DOMAIN"
echo "Email: $EMAIL"
echo ""

# Confirm before proceeding
read -p "Continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
fi

# Create directories
mkdir -p ./ssl/certbot ./nginx

# Stop services if running
echo -e "${YELLOW}Stopping existing services...${NC}"
docker-compose down 2>/dev/null

# Create temporary nginx config for certbot
echo -e "${YELLOW}Creating temporary nginx configuration...${NC}"
cat > ./nginx/nginx-certbot.conf << 'EOF'
events {
    worker_connections 1024;
}

http {
    server {
        listen 80;
        server_name _;
        
        location /.well-known/acme-challenge/ {
            root /var/www/certbot;
        }
        
        location / {
            return 404;
        }
    }
}
EOF

# Run temporary nginx for certificate generation
echo -e "${YELLOW}Starting temporary nginx for certificate generation...${NC}"
docker run -d --name certbot-nginx \
  -p 80:80 \
  -v $(pwd)/nginx/nginx-certbot.conf:/etc/nginx/nginx.conf:ro \
  -v $(pwd)/ssl/certbot:/var/www/certbot \
  nginx:alpine

# Wait for nginx to start
sleep 5

# Get certificate
echo -e "${YELLOW}🔑 Obtaining SSL certificate from Let's Encrypt...${NC}"
docker run --rm \
  -v $(pwd)/ssl/certbot:/etc/letsencrypt \
  -v $(pwd)/ssl/certbot:/var/www/certbot \
  certbot/certbot certonly \
  --webroot \
  --webroot-path=/var/www/certbot \
  -d $DOMAIN \
  -d www.$DOMAIN \
  --email $EMAIL \
  --agree-tos \
  --no-eff-email

CERT_SUCCESS=$?

# Stop temporary nginx
docker stop certbot-nginx && docker rm certbot-nginx

if [ $CERT_SUCCESS -eq 0 ]; then
    echo -e "${GREEN}✅ Certificate obtained successfully${NC}"
    
    # Update nginx config with actual domain
    echo -e "${YELLOW}Updating nginx configuration...${NC}"
    sed -i.bak "s/localhost/$DOMAIN/g" ./nginx/nginx.conf
    sed -i "s|ssl_certificate /etc/ssl/self-signed/|ssl_certificate /etc/ssl/certbot/live/$DOMAIN/|g" ./nginx/nginx.conf
    sed -i "s|ssl_certificate_key /etc/ssl/self-signed/|ssl_certificate_key /etc/ssl/certbot/live/$DOMAIN/|g" ./nginx/nginx.conf
    
    # Start services with SSL
    echo -e "${YELLOW}Starting HAL9 with SSL...${NC}"
    docker-compose -f docker-compose.yml -f docker-compose.ssl.yml up -d
    
    # Wait for services
    echo -e "${YELLOW}⏳ Waiting for services to start...${NC}"
    sleep 10
    
    # Test HTTPS
    echo -e "${YELLOW}🧪 Testing HTTPS connection...${NC}"
    RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" https://$DOMAIN/health)
    
    if [ "$RESPONSE" = "200" ]; then
        echo -e "${GREEN}✅ HTTPS is working!${NC}"
        echo ""
        echo -e "${GREEN}🎉 SSL successfully enabled for production!${NC}"
        echo ""
        echo "Access HAL9 at:"
        echo "  - https://$DOMAIN (HTTPS)"
        echo "  - http://$DOMAIN (redirects to HTTPS)"
        echo ""
        echo "Certificate details:"
        openssl x509 -in ./ssl/certbot/live/$DOMAIN/cert.pem -noout -dates
        echo ""
        echo -e "${YELLOW}🔄 Certificate Auto-Renewal${NC}"
        echo "Add this to your crontab:"
        echo "0 2 * * * $(pwd)/scripts/renew_ssl.sh"
    else
        echo -e "${RED}❌ HTTPS test failed (HTTP $RESPONSE)${NC}"
        echo "Check logs: docker logs hal9-nginx"
    fi
else
    echo -e "${RED}❌ Failed to obtain certificate${NC}"
    echo ""
    echo "Common issues:"
    echo "1. Domain doesn't point to this server"
    echo "2. Port 80 is blocked by firewall"
    echo "3. DNS not propagated yet (wait 5-10 minutes)"
    echo ""
    echo "Debug steps:"
    echo "1. Check DNS: nslookup $DOMAIN"
    echo "2. Test port 80: curl http://$DOMAIN"
    echo "3. Check certbot logs in ./ssl/certbot/logs/"
    exit 1
fi

# Clean up temporary files
rm -f ./nginx/nginx-certbot.conf