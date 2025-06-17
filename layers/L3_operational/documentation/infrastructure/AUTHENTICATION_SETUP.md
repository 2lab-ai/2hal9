# HAL9 Authentication System Setup Guide

## Overview

HAL9 includes a built-in authentication system with JWT tokens and API keys. This guide explains how to enable and configure authentication.

## Features

- **JWT-based authentication** for web clients
- **API key authentication** for programmatic access
- **User management** with roles and permissions
- **Token refresh** mechanism
- **Secure password hashing** with bcrypt

## Prerequisites

- PostgreSQL database (already running)
- JWT secret key
- Environment configuration

## Setup Steps

### 1. Configure Environment Variables

Add these to your `.env` file or docker-compose.yml:

```bash
# Authentication Configuration
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
JWT_EXPIRY=3600  # 1 hour in seconds
JWT_REFRESH_EXPIRY=604800  # 7 days in seconds
API_KEY_PREFIX=hal9_
BCRYPT_COST=12  # Password hashing cost (10-15 recommended)
```

### 2. Update Docker Compose

Add authentication environment variables to `docker-compose.yml`:

```yaml
services:
  hal9-server:
    environment:
      # ... existing config ...
      
      # Authentication
      JWT_SECRET: ${JWT_SECRET:-development-secret-change-in-production}
      JWT_EXPIRY: ${JWT_EXPIRY:-3600}
      JWT_REFRESH_EXPIRY: ${JWT_REFRESH_EXPIRY:-604800}
      API_KEY_PREFIX: ${API_KEY_PREFIX:-hal9_}
      BCRYPT_COST: ${BCRYPT_COST:-12}
```

### 3. Initialize Database Tables

The authentication system will automatically create required tables on startup:
- `users` - User accounts
- `api_keys` - API key storage
- `refresh_tokens` - Refresh token tracking

### 4. Restart Services

```bash
docker-compose down
docker-compose up -d
```

## API Endpoints

### Public Endpoints (No Authentication Required)

#### Register New User
```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "SecurePass123!",
    "role": "user"
  }'
```

#### Login
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "SecurePass123!"
  }'
```

Response:
```json
{
  "user": {
    "id": "user-uuid",
    "username": "testuser",
    "email": "test@example.com",
    "role": "user",
    "created_at": 1234567890,
    "is_active": true
  },
  "tokens": {
    "access_token": "eyJ...",
    "refresh_token": "eyJ...",
    "expires_in": 3600
  }
}
```

#### Refresh Token
```bash
curl -X POST http://localhost:8080/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "your-refresh-token"
  }'
```

### Protected Endpoints (Authentication Required)

#### Get User Profile
```bash
curl -X GET http://localhost:8080/api/v1/auth/profile \
  -H "Authorization: Bearer your-access-token"
```

#### Update Profile
```bash
curl -X PUT http://localhost:8080/api/v1/auth/profile \
  -H "Authorization: Bearer your-access-token" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newemail@example.com"
  }'
```

#### Create API Key
```bash
curl -X POST http://localhost:8080/api/v1/auth/api-keys \
  -H "Authorization: Bearer your-access-token" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My API Key",
    "expires_in_days": 365
  }'
```

#### List API Keys
```bash
curl -X GET http://localhost:8080/api/v1/auth/api-keys \
  -H "Authorization: Bearer your-access-token"
```

## Using Authentication in Your Application

### JWT Token Usage

Include the access token in the Authorization header:
```
Authorization: Bearer eyJ...
```

### API Key Usage

Include the API key in the X-API-Key header:
```
X-API-Key: hal9_abcd1234...
```

## Security Best Practices

1. **JWT Secret**: Use a strong, random secret (at least 32 characters)
2. **HTTPS**: Always use HTTPS in production
3. **Token Expiry**: Keep access tokens short-lived (1 hour recommended)
4. **Refresh Tokens**: Implement proper refresh token rotation
5. **Password Policy**: Enforce strong passwords
6. **Rate Limiting**: Enable rate limiting on auth endpoints
7. **Audit Logging**: Log all authentication events

## Roles and Permissions

Default roles:
- `admin` - Full system access
- `user` - Standard user access
- `service` - Service account for integrations

## Troubleshooting

### Common Issues

1. **"JWT_SECRET not set" error**
   - Ensure JWT_SECRET environment variable is set
   - Check docker-compose.yml configuration

2. **"Invalid credentials" on login**
   - Verify username and password are correct
   - Check if user account is active
   - Ensure password meets requirements

3. **"Token expired" errors**
   - Implement token refresh logic in your client
   - Use refresh token to get new access token

4. **Database connection errors**
   - Verify PostgreSQL is running
   - Check DATABASE_URL configuration
   - Ensure auth tables are created

### Testing Authentication

Use the provided test script:
```bash
./scripts/test_auth.sh
```

This will:
1. Register a test user
2. Login and get tokens
3. Test protected endpoints
4. Create and use API keys
5. Test token refresh

## Monitoring

Authentication metrics are exposed at `/metrics`:
- `auth_login_attempts_total` - Total login attempts
- `auth_login_success_total` - Successful logins
- `auth_token_refresh_total` - Token refresh count
- `auth_api_key_usage_total` - API key usage

## Next Steps

1. Enable authentication in production
2. Configure SSL/TLS for secure communication
3. Set up user management UI
4. Implement OAuth2/OIDC integration (optional)
5. Configure audit logging