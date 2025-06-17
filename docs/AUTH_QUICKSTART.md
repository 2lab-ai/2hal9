# 🔐 HAL9 Authentication Quick Start

## Enable Authentication in 30 Seconds

```bash
# Run the enable script
./scripts/enable_auth.sh
```

That's it! Authentication is now enabled with:
- JWT token authentication
- API key support  
- Default admin user
- Secure password hashing

## Test Authentication

```bash
# Run comprehensive auth tests
./scripts/test_auth.sh
```

## Default Admin Credentials

⚠️ **CHANGE THESE IMMEDIATELY IN PRODUCTION!**

- Username: `admin`
- Password: `AdminPass123!`

## Quick API Examples

### Register a User
```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "johndoe",
    "email": "john@example.com", 
    "password": "SecurePass123!",
    "role": "user"
  }'
```

### Login
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "johndoe",
    "password": "SecurePass123!"
  }'
```

### Use the Token
```bash
# Get your token from login response
TOKEN="your-jwt-token-here"

# Use it in requests
curl -X GET http://localhost:8080/api/v1/auth/profile \
  -H "Authorization: Bearer $TOKEN"
```

## Security Checklist

- [ ] Change default admin password
- [ ] Use strong JWT_SECRET (auto-generated)
- [ ] Enable HTTPS in production
- [ ] Configure CORS properly
- [ ] Set up rate limiting
- [ ] Enable audit logging

## Troubleshooting

### Auth endpoints return 404
- Authentication might not be enabled
- Run `./scripts/enable_auth.sh`

### Can't login
- Check username/password
- Verify user exists in database
- Check server logs: `docker logs hal9-server`

### Token errors
- Token might be expired (default: 1 hour)
- Use refresh token to get new access token
- Check JWT_SECRET is consistent

## Next Steps

1. Read full guide: [AUTHENTICATION_SETUP.md](./AUTHENTICATION_SETUP.md)
2. Configure production settings
3. Set up user management UI
4. Implement OAuth2/OIDC (optional)