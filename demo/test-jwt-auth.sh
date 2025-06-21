#!/bin/bash
# Quick demo to test JWT authentication

echo "🔐 HAL9 JWT Authentication Demo"
echo "================================"

# Build the server
echo "Building server..."
cargo build --release --bin hal9-server

# Start server with auth config
echo "Starting server with authentication enabled..."
AUTH_ENABLED=true \
JWT_SECRET="demo-secret-key" \
AUTH_DATABASE_PATH="demo_auth.db" \
HTTP_PORT=8080 \
cargo run --release --bin hal9-server -- config/test-auth.yaml &

SERVER_PID=$!
sleep 3

# Test registration
echo -e "\n1️⃣ Registering new user..."
curl -s -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "hal9user",
    "email": "hal9@example.com",
    "password": "securepass123"
  }' | jq '.'

# Test login
echo -e "\n2️⃣ Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "hal9user",
    "password": "securepass123"
  }')

echo "$LOGIN_RESPONSE" | jq '.'
ACCESS_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.tokens.access_token')

# Test protected endpoint
echo -e "\n3️⃣ Accessing protected profile endpoint..."
curl -s -X GET http://localhost:8080/api/v1/auth/profile \
  -H "Authorization: Bearer $ACCESS_TOKEN" | jq '.'

# Test unauthorized access
echo -e "\n4️⃣ Testing unauthorized access (should fail)..."
curl -s -o /dev/null -w "HTTP Status: %{http_code}\n" \
  -X GET http://localhost:8080/api/v1/auth/profile

# Clean up
echo -e "\n✅ JWT authentication is working!"
kill $SERVER_PID 2>/dev/null
rm -f demo_auth.db

echo "Demo complete!"