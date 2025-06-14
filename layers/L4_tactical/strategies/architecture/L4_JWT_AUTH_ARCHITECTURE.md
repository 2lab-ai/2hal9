# JWT Authentication Architecture for HAL9

## Overview
This document outlines the JWT-based authentication and multi-user support system for HAL9.

## Architecture Components

### 1. User Management
- User registration and login
- User profiles and preferences
- Role-based access control (RBAC)
- API key management

### 2. Authentication Flow
```
Client → Login → JWT Token → API Request with Bearer Token → Validation → Access
```

### 3. Database Schema
```sql
-- Users table
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    is_active BOOLEAN DEFAULT TRUE
);

-- API Keys table
CREATE TABLE api_keys (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    key_hash TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    permissions TEXT NOT NULL, -- JSON array
    created_at INTEGER NOT NULL,
    last_used INTEGER,
    expires_at INTEGER,
    is_active BOOLEAN DEFAULT TRUE,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- User Sessions table
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    token_hash TEXT UNIQUE NOT NULL,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Audit Log table
CREATE TABLE audit_log (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    details TEXT, -- JSON
    ip_address TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

### 4. Security Features
- Bcrypt password hashing
- JWT token expiration (15 minutes access, 7 days refresh)
- Rate limiting per user
- IP-based session tracking
- Audit logging for all actions

### 5. API Endpoints
- `POST /api/v1/auth/register` - User registration
- `POST /api/v1/auth/login` - User login
- `POST /api/v1/auth/refresh` - Refresh JWT token
- `POST /api/v1/auth/logout` - Logout user
- `GET /api/v1/auth/profile` - Get user profile
- `PUT /api/v1/auth/profile` - Update user profile
- `POST /api/v1/auth/api-keys` - Create API key
- `GET /api/v1/auth/api-keys` - List API keys
- `DELETE /api/v1/auth/api-keys/:id` - Revoke API key

### 6. Middleware Integration
```rust
// Authentication middleware
async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Response> {
    let token = extract_bearer_token(&req)?;
    let claims = validate_jwt(token)?;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}
```

### 7. User Isolation
- Each user has isolated:
  - Neuron configurations
  - Memory storage
  - Cost tracking
  - API usage limits
- Shared resources:
  - Model weights (read-only)
  - System tools (with permissions)

## Implementation Plan
1. Add dependencies (jsonwebtoken, bcrypt, etc.)
2. Create user database and models
3. Implement authentication service
4. Add JWT token generation/validation
5. Create authentication middleware
6. Update API endpoints with auth
7. Add user isolation to neurons
8. Implement API key management
9. Add audit logging