#!/bin/bash

# Test SSL/TLS setup

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🔒 SSL/TLS Configuration Test${NC}"
echo "=================================="
echo

# Function to test SSL configuration
test_ssl() {
    local host=$1
    local port=$2
    echo -e "${YELLOW}Testing SSL on $host:$port...${NC}"
    
    # Check if we can connect
    timeout 5 openssl s_client -connect $host:$port -servername $host < /dev/null 2>/dev/null > /tmp/ssl_test.txt
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ SSL connection successful${NC}"
        
        # Check certificate details
        echo -e "\n${BLUE}Certificate Information:${NC}"
        openssl s_client -connect $host:$port -servername $host < /dev/null 2>/dev/null | \
            openssl x509 -noout -subject -issuer -dates
        
        # Check protocol versions
        echo -e "\n${BLUE}Supported Protocols:${NC}"
        for proto in tls1_2 tls1_3; do
            if openssl s_client -connect $host:$port -servername $host -$proto < /dev/null 2>/dev/null | grep -q "Protocol"; then
                echo -e "${GREEN}✅ $proto supported${NC}"
            else
                echo -e "${RED}❌ $proto not supported${NC}"
            fi
        done
        
        # Check cipher suites
        echo -e "\n${BLUE}Cipher Suite:${NC}"
        openssl s_client -connect $host:$port -servername $host < /dev/null 2>/dev/null | \
            grep "Cipher" | head -1
        
        # Security headers test
        echo -e "\n${BLUE}Security Headers Test:${NC}"
        headers=$(curl -sI https://$host:$port 2>/dev/null)
        
        # Check for important security headers
        for header in "Strict-Transport-Security" "X-Frame-Options" "X-Content-Type-Options" "X-XSS-Protection"; do
            if echo "$headers" | grep -qi "$header"; then
                echo -e "${GREEN}✅ $header present${NC}"
                echo "$headers" | grep -i "$header" | head -1
            else
                echo -e "${RED}❌ $header missing${NC}"
            fi
        done
        
    else
        echo -e "${RED}❌ SSL connection failed${NC}"
        return 1
    fi
}

# Function to generate self-signed certificate for testing
generate_test_cert() {
    echo -e "${YELLOW}Generating self-signed certificate for testing...${NC}"
    
    mkdir -p ./ssl/test
    openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
        -keyout ./ssl/test/privkey.pem \
        -out ./ssl/test/fullchain.pem \
        -subj "/C=US/ST=Test/L=Test/O=HAL9/CN=localhost" 2>/dev/null
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Test certificate generated${NC}"
        echo "  Certificate: ./ssl/test/fullchain.pem"
        echo "  Private key: ./ssl/test/privkey.pem"
        return 0
    else
        echo -e "${RED}❌ Failed to generate certificate${NC}"
        return 1
    fi
}

# Check for existing SSL setup
echo -e "${BLUE}Checking existing SSL configuration...${NC}"

# Check for nginx config
if [ -f "./layers/L3_operational/configuration/nginx/nginx.conf" ]; then
    echo -e "${GREEN}✅ Nginx configuration found${NC}"
    
    # Check SSL directives
    if grep -q "ssl_certificate" ./layers/L3_operational/configuration/nginx/nginx.conf; then
        echo -e "${GREEN}✅ SSL directives configured${NC}"
    else
        echo -e "${RED}❌ SSL directives missing${NC}"
    fi
else
    echo -e "${RED}❌ Nginx configuration not found${NC}"
fi

# Check for SSL scripts
echo -e "\n${BLUE}SSL Setup Scripts:${NC}"
scripts=(
    "./layers/L3_operational/scripts/deployment/enable_ssl_dev.sh"
    "./layers/L3_operational/scripts/deployment/enable_ssl_prod.sh"
    "./layers/L3_operational/scripts/deployment/renew_ssl.sh"
)

for script in "${scripts[@]}"; do
    if [ -f "$script" ]; then
        echo -e "${GREEN}✅ $(basename $script) found${NC}"
    else
        echo -e "${RED}❌ $(basename $script) missing${NC}"
    fi
done

# Test local SSL if running
echo -e "\n${BLUE}Testing local SSL setup...${NC}"
if curl -k -s https://localhost:443/health 2>/dev/null | grep -q "ok"; then
    echo -e "${GREEN}✅ Local HTTPS server is running${NC}"
    test_ssl localhost 443
else
    echo -e "${YELLOW}⚠️  No local HTTPS server detected${NC}"
    echo "To enable SSL for development:"
    echo "  ./layers/L3_operational/scripts/deployment/enable_ssl_dev.sh"
fi

# SSL/TLS best practices checklist
echo -e "\n${BLUE}📋 SSL/TLS Security Checklist:${NC}"
echo "================================"

checklist=(
    "Use TLS 1.2 or higher (disable older versions)"
    "Use strong cipher suites (ECDHE, AES-GCM)"
    "Enable HSTS header with long max-age"
    "Implement certificate pinning for mobile apps"
    "Use CAA DNS records to restrict certificate issuers"
    "Enable OCSP stapling for performance"
    "Redirect all HTTP traffic to HTTPS"
    "Use secure cookies (Secure, HttpOnly, SameSite)"
    "Implement proper certificate renewal automation"
    "Monitor certificate expiration dates"
)

for i in "${!checklist[@]}"; do
    echo "$((i+1)). ${checklist[$i]}"
done

echo -e "\n${BLUE}🔧 Quick SSL Commands:${NC}"
echo "======================="
echo "# Development (self-signed):"
echo "  ./layers/L3_operational/scripts/deployment/enable_ssl_dev.sh"
echo ""
echo "# Production (Let's Encrypt):"
echo "  ./layers/L3_operational/scripts/deployment/enable_ssl_prod.sh yourdomain.com your@email.com"
echo ""
echo "# Test SSL configuration:"
echo "  openssl s_client -connect yourdomain.com:443 -servername yourdomain.com"
echo ""
echo "# Check certificate expiration:"
echo "  openssl x509 -in /path/to/cert.pem -noout -dates"
echo ""
echo "# Test SSL security:"
echo "  curl -I https://yourdomain.com"

# Clean up
rm -f /tmp/ssl_test.txt

echo -e "\n${GREEN}✅ SSL/TLS configuration test complete!${NC}"