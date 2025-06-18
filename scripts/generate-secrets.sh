#!/bin/bash
# Generate secure secrets for HAL9 production deployment

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}🔐 HAL9 Secret Generator${NC}"
echo "============================"
echo ""

# Check if openssl is installed
if ! command -v openssl &> /dev/null; then
    echo -e "${RED}Error: openssl is required but not installed.${NC}"
    exit 1
fi

# Generate JWT secret (256-bit)
JWT_SECRET=$(openssl rand -base64 32)
echo -e "${GREEN}JWT_SECRET:${NC}"
echo "$JWT_SECRET"
echo ""

# Generate session secret (256-bit)
SESSION_SECRET=$(openssl rand -base64 32)
echo -e "${GREEN}SESSION_SECRET:${NC}"
echo "$SESSION_SECRET"
echo ""

# Generate database password (128-bit, URL-safe)
DB_PASSWORD=$(openssl rand -base64 16 | tr -d "=+/" | cut -c1-16)
echo -e "${GREEN}DATABASE_PASSWORD:${NC}"
echo "$DB_PASSWORD"
echo ""

# Generate Redis password (128-bit, URL-safe)
REDIS_PASSWORD=$(openssl rand -base64 16 | tr -d "=+/" | cut -c1-16)
echo -e "${GREEN}REDIS_PASSWORD:${NC}"
echo "$REDIS_PASSWORD"
echo ""

# Generate backup encryption key (256-bit)
BACKUP_KEY=$(openssl rand -base64 32)
echo -e "${GREEN}BACKUP_ENCRYPTION_KEY:${NC}"
echo "$BACKUP_KEY"
echo ""

# Generate Grafana API key format
GRAFANA_KEY="glsa_$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-32)"
echo -e "${GREEN}GRAFANA_API_KEY (example format):${NC}"
echo "$GRAFANA_KEY"
echo ""

echo -e "${YELLOW}⚠️  Important Security Notes:${NC}"
echo "1. Store these secrets securely (e.g., AWS Secrets Manager, HashiCorp Vault)"
echo "2. Never commit secrets to version control"
echo "3. Rotate secrets regularly (at least every 90 days)"
echo "4. Use different secrets for each environment"
echo "5. Enable audit logging for secret access"
echo ""

# Optional: Save to environment file
read -p "Do you want to save these to a local .env file? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    ENV_FILE=".env.generated.$(date +%Y%m%d_%H%M%S)"
    cat > "$ENV_FILE" << EOF
# Generated secrets - $(date)
# WARNING: This file contains sensitive data. Handle with care!

JWT_SECRET=$JWT_SECRET
SESSION_SECRET=$SESSION_SECRET
DATABASE_PASSWORD=$DB_PASSWORD
REDIS_PASSWORD=$REDIS_PASSWORD
BACKUP_ENCRYPTION_KEY=$BACKUP_KEY

# Example database URL with password
DATABASE_URL=postgresql://hal9:$DB_PASSWORD@localhost:5432/hal9_prod

# Example Redis URL with password
REDIS_URL=redis://:$REDIS_PASSWORD@localhost:6379
EOF
    
    chmod 600 "$ENV_FILE"
    echo -e "${GREEN}✅ Secrets saved to: $ENV_FILE${NC}"
    echo -e "${YELLOW}   Remember to delete this file after transferring secrets!${NC}"
fi