#!/bin/bash

# Script to enable authentication for HAL9 server

echo "🔐 Enabling HAL9 Authentication System"
echo "====================================="
echo ""

# Check if .env file exists
if [ ! -f .env ]; then
    echo "📄 Creating .env file from example..."
    cp .env.example .env
    echo "✅ .env file created"
fi

# Generate a secure JWT secret if not already set
if ! grep -q "JWT_SECRET=" .env || grep -q "JWT_SECRET=your-secret-key-here" .env; then
    echo "🔑 Generating secure JWT secret..."
    JWT_SECRET=$(openssl rand -base64 32)
    if grep -q "JWT_SECRET=" .env; then
        # Update existing JWT_SECRET
        sed -i.bak "s/JWT_SECRET=.*/JWT_SECRET=${JWT_SECRET}/" .env
    else
        # Add JWT_SECRET
        echo "JWT_SECRET=${JWT_SECRET}" >> .env
    fi
    echo "✅ JWT secret generated and saved"
else
    echo "ℹ️ JWT secret already configured"
fi

# Add authentication enabled flag
if ! grep -q "AUTH_ENABLED=" .env; then
    echo "AUTH_ENABLED=true" >> .env
    echo "✅ Authentication enabled in .env"
fi

# Set default admin credentials if not present
if ! grep -q "ADMIN_USERNAME=" .env; then
    echo "" >> .env
    echo "# Admin credentials (change these!)" >> .env
    echo "ADMIN_USERNAME=admin" >> .env
    echo "ADMIN_EMAIL=admin@hal9.local" >> .env
    echo "ADMIN_PASSWORD=AdminPass123!" >> .env
    echo "✅ Default admin credentials added (PLEASE CHANGE THESE!)"
fi

echo ""
echo "🔄 Restarting services with authentication enabled..."
echo ""

# Use both compose files
docker-compose -f docker-compose.yml -f docker-compose.auth.yml down
docker-compose -f docker-compose.yml -f docker-compose.auth.yml up -d

echo ""
echo "⏳ Waiting for services to start..."
sleep 10

# Test if authentication is working
echo ""
echo "🧪 Testing authentication endpoints..."
echo ""

# Test registration endpoint
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"test","email":"test@test.com","password":"Test123!","role":"user"}')

if [ "$RESPONSE" = "201" ] || [ "$RESPONSE" = "400" ] || [ "$RESPONSE" = "409" ]; then
    echo "✅ Authentication endpoints are active!"
    echo ""
    echo "🎉 Authentication successfully enabled!"
    echo ""
    echo "Default admin credentials:"
    echo "  Username: admin"
    echo "  Password: AdminPass123!"
    echo ""
    echo "⚠️  IMPORTANT: Change the admin password immediately!"
    echo ""
    echo "Next steps:"
    echo "1. Run ./scripts/test_auth.sh to test all auth endpoints"
    echo "2. Change admin password using the API"
    echo "3. Review docs/AUTHENTICATION_SETUP.md for full configuration"
else
    echo "❌ Authentication endpoints not responding (HTTP $RESPONSE)"
    echo ""
    echo "Troubleshooting:"
    echo "1. Check logs: docker logs hal9-server"
    echo "2. Verify database is running: docker ps"
    echo "3. Check .env file for correct configuration"
fi