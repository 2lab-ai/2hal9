#!/bin/bash

# Domain setup helper script

# Colors
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🌐 HAL9 Domain Setup Helper${NC}"
echo "============================="
echo ""

# Get domain from user
read -p "Enter your domain (e.g., hal9.example.com): " DOMAIN
read -p "Enter your email for SSL: " EMAIL
read -p "Enter your server's public IP: " SERVER_IP

echo ""
echo -e "${YELLOW}Configuration Summary:${NC}"
echo "Domain: $DOMAIN"
echo "Email: $EMAIL"
echo "Server IP: $SERVER_IP"
echo ""

read -p "Is this correct? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Setup cancelled."
    exit 0
fi

echo ""
echo -e "${BLUE}Step 1: DNS Configuration${NC}"
echo "========================="
echo ""
echo "Add these DNS records at your domain registrar:"
echo ""
echo "Type    Name    Value           TTL"
echo "----    ----    -----           ---"
echo "A       @       $SERVER_IP      300"
echo "A       www     $SERVER_IP      300"
echo ""
echo "For subdomain (e.g., api.example.com):"
echo "A       api     $SERVER_IP      300"
echo ""
echo -e "${YELLOW}Press Enter after adding DNS records...${NC}"
read

echo -e "${BLUE}Step 2: Testing DNS Propagation${NC}"
echo "==============================="
echo ""

# Function to check DNS
check_dns() {
    echo -n "Checking DNS for $DOMAIN... "
    RESOLVED_IP=$(dig +short $DOMAIN | tail -n1)
    
    if [ "$RESOLVED_IP" = "$SERVER_IP" ]; then
        echo -e "${GREEN}✅ Success! Resolves to $RESOLVED_IP${NC}"
        return 0
    elif [ -z "$RESOLVED_IP" ]; then
        echo -e "${RED}❌ Not resolving yet${NC}"
        return 1
    else
        echo -e "${YELLOW}⚠️  Resolves to $RESOLVED_IP (expected $SERVER_IP)${NC}"
        return 1
    fi
}

# Wait for DNS propagation
MAX_ATTEMPTS=20
ATTEMPT=1

while [ $ATTEMPT -le $MAX_ATTEMPTS ]; do
    if check_dns; then
        break
    fi
    
    if [ $ATTEMPT -lt $MAX_ATTEMPTS ]; then
        echo "Waiting 30 seconds before retry ($ATTEMPT/$MAX_ATTEMPTS)..."
        sleep 30
    fi
    
    ATTEMPT=$((ATTEMPT + 1))
done

if [ $ATTEMPT -gt $MAX_ATTEMPTS ]; then
    echo -e "${RED}DNS propagation is taking longer than expected.${NC}"
    echo "You can continue anyway or wait longer."
    read -p "Continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo ""
echo -e "${BLUE}Step 3: Updating Configuration${NC}"
echo "=============================="

# Update .env file
if [ -f .env ]; then
    echo "Updating .env file..."
    
    # Add or update DOMAIN_NAME
    if grep -q "DOMAIN_NAME=" .env; then
        sed -i.bak "s/DOMAIN_NAME=.*/DOMAIN_NAME=$DOMAIN/" .env
    else
        echo "DOMAIN_NAME=$DOMAIN" >> .env
    fi
    
    # Add or update ALLOWED_ORIGINS
    if grep -q "ALLOWED_ORIGINS=" .env; then
        sed -i.bak "s|ALLOWED_ORIGINS=.*|ALLOWED_ORIGINS=https://$DOMAIN,https://www.$DOMAIN|" .env
    else
        echo "ALLOWED_ORIGINS=https://$DOMAIN,https://www.$DOMAIN" >> .env
    fi
    
    echo -e "${GREEN}✅ .env updated${NC}"
else
    echo -e "${YELLOW}No .env file found. Creating one...${NC}"
    cat > .env << EOF
# Domain Configuration
DOMAIN_NAME=$DOMAIN
ALLOWED_ORIGINS=https://$DOMAIN,https://www.$DOMAIN
PUBLIC_URL=https://$DOMAIN

# Add other configuration as needed
EOF
    echo -e "${GREEN}✅ .env created${NC}"
fi

# Update nginx config
if [ -f ./nginx/nginx.conf ]; then
    echo "Updating nginx configuration..."
    sed -i.bak "s/server_name localhost;/server_name $DOMAIN www.$DOMAIN;/g" ./nginx/nginx.conf
    echo -e "${GREEN}✅ nginx.conf updated${NC}"
fi

echo ""
echo -e "${BLUE}Step 4: SSL Certificate Setup${NC}"
echo "============================="
echo ""
echo "Running SSL setup for production..."
echo ""

# Stop any running services
docker-compose down 2>/dev/null

# Run SSL setup
if [ -f ./scripts/enable_ssl_prod.sh ]; then
    ./scripts/enable_ssl_prod.sh $DOMAIN $EMAIL
else
    echo -e "${RED}SSL setup script not found!${NC}"
    echo "Please run manually: ./scripts/enable_ssl_prod.sh $DOMAIN $EMAIL"
fi

echo ""
echo -e "${GREEN}🎉 Domain setup complete!${NC}"
echo ""
echo "Your HAL9 instance should now be accessible at:"
echo "  - https://$DOMAIN"
echo "  - https://www.$DOMAIN"
echo ""
echo "Next steps:"
echo "1. Test the domain: curl https://$DOMAIN/health"
echo "2. Set up monitoring for the domain"
echo "3. Configure CDN if needed"
echo "4. Enable DNSSEC at your registrar"