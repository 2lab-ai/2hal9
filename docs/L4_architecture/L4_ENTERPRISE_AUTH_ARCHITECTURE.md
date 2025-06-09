# HAL9 Enterprise Authentication Architecture

## Overview

Enterprise authentication extends HAL9's existing JWT-based auth with SSO capabilities, organization management, and advanced RBAC for B2B deployments.

## Architecture Components

```
┌────────────────────────────────────────────────────────┐
│                   Enterprise Client                      │
│                                                         │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ Web Portal  │  │ Mobile App   │  │ API Client   │ │
│  └──────┬──────┘  └──────┬───────┘  └──────┬───────┘ │
└─────────┼────────────────┼──────────────────┼──────────┘
          │                │                  │
          ▼                ▼                  ▼
┌────────────────────────────────────────────────────────┐
│                  HAL9 Auth Gateway                      │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ │
│  │ SAML 2.0     │  │ OAuth2/OIDC  │  │ API Keys    │ │
│  │ Handler      │  │ Handler      │  │ Handler     │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬──────┘ │
└─────────┼──────────────────┼─────────────────┼─────────┘
          │                  │                 │
          ▼                  ▼                 ▼
┌────────────────────────────────────────────────────────┐
│              Identity Provider Integration              │
│                                                         │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   Okta      │  │ Auth0/Azure  │  │   Google     │ │
│  │   SAML      │  │     AD       │  │   Workspace  │ │
│  └─────────────┘  └──────────────┘  └──────────────┘ │
└────────────────────────────────────────────────────────┘
          │
          ▼
┌────────────────────────────────────────────────────────┐
│                HAL9 Identity Management                 │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ │
│  │Organization  │  │    Team      │  │    User     │ │
│  │Management    │  │  Management  │  │  Management │ │
│  └──────────────┘  └──────────────┘  └─────────────┘ │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ │
│  │Role-Based    │  │ Attribute    │  │  Policy     │ │
│  │Access Control│  │Based Control │  │  Engine     │ │
│  └──────────────┘  └──────────────┘  └─────────────┘ │
└────────────────────────────────────────────────────────┘
```

## Core Components

### 1. SAML 2.0 Integration

```rust
pub struct SamlConfig {
    /// Identity Provider metadata URL
    pub idp_metadata_url: String,
    
    /// Service Provider entity ID
    pub sp_entity_id: String,
    
    /// Assertion Consumer Service URL
    pub acs_url: String,
    
    /// Single Logout Service URL
    pub sls_url: String,
    
    /// X.509 certificate for signing
    pub sp_certificate: String,
    
    /// Private key for decryption
    pub sp_private_key: String,
}

pub trait SamlHandler {
    /// Generate SAML authentication request
    async fn create_auth_request(&self) -> Result<SamlAuthRequest>;
    
    /// Process SAML response
    async fn process_response(&self, response: &str) -> Result<SamlAssertion>;
    
    /// Map SAML attributes to user
    async fn map_attributes(&self, assertion: SamlAssertion) -> Result<User>;
}
```

### 2. OAuth2/OIDC Integration

```rust
pub struct OidcConfig {
    /// Provider configuration URL
    pub discovery_url: String,
    
    /// Client ID
    pub client_id: String,
    
    /// Client secret
    pub client_secret: String,
    
    /// Redirect URI
    pub redirect_uri: String,
    
    /// Requested scopes
    pub scopes: Vec<String>,
}

pub trait OidcHandler {
    /// Get authorization URL
    fn get_auth_url(&self, state: &str) -> String;
    
    /// Exchange code for tokens
    async fn exchange_code(&self, code: &str) -> Result<TokenResponse>;
    
    /// Get user info from token
    async fn get_user_info(&self, token: &str) -> Result<UserInfo>;
}
```

### 3. Organization Management

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub domain: String,
    pub subscription_tier: SubscriptionTier,
    pub settings: OrganizationSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettings {
    /// SSO configuration
    pub sso_enabled: bool,
    pub sso_provider: Option<SsoProvider>,
    pub sso_config: Option<serde_json::Value>,
    
    /// Security settings
    pub enforce_2fa: bool,
    pub password_policy: PasswordPolicy,
    pub session_timeout_minutes: u32,
    
    /// Usage limits
    pub max_users: Option<u32>,
    pub max_api_calls_per_month: Option<u64>,
    pub max_neurons: Option<u32>,
    
    /// Features
    pub enabled_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionTier {
    Free,
    Starter,
    Professional,
    Enterprise,
    Custom(String),
}
```

### 4. Team Management

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: TeamPermissions,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPermissions {
    /// Neuron access
    pub allowed_neurons: Vec<String>,
    pub neuron_creation: bool,
    pub neuron_deletion: bool,
    
    /// API access
    pub api_rate_limit: Option<u32>,
    pub allowed_endpoints: Vec<String>,
    
    /// Resource limits
    pub max_concurrent_signals: Option<u32>,
    pub max_memory_gb: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub user_id: Uuid,
    pub team_id: Uuid,
    pub role: TeamRole,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TeamRole {
    Owner,
    Admin,
    Member,
    ReadOnly,
}
```

### 5. Advanced RBAC

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub permissions: Vec<Permission>,
    pub is_system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub actions: Vec<Action>,
    pub conditions: Option<Vec<Condition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Create,
    Read,
    Update,
    Delete,
    Execute,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    In,
    NotIn,
    GreaterThan,
    LessThan,
    Contains,
}
```

### 6. Audit & Compliance

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: String,
    pub ip_address: IpAddr,
    pub user_agent: String,
    pub details: serde_json::Value,
    pub risk_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    // Authentication
    Login,
    Logout,
    FailedLogin,
    PasswordChange,
    TwoFactorEnabled,
    
    // Data access
    DataAccess,
    DataExport,
    DataModification,
    DataDeletion,
    
    // Administration
    UserCreation,
    UserDeletion,
    RoleAssignment,
    PermissionChange,
    
    // Security
    SecurityAlert,
    PolicyViolation,
    SuspiciousActivity,
}

pub struct ComplianceManager {
    /// GDPR compliance
    pub enable_right_to_deletion: bool,
    pub enable_data_portability: bool,
    pub data_retention_days: u32,
    
    /// SOC2 compliance
    pub enable_encryption_at_rest: bool,
    pub enable_audit_logging: bool,
    pub password_complexity_rules: PasswordPolicy,
    
    /// Custom policies
    pub custom_policies: Vec<CompliancePolicy>,
}
```

## Authentication Flows

### 1. SAML SSO Flow

```
User → HAL9 → Redirect to IdP
         ↓
IdP authenticates user
         ↓
IdP → SAML Response → HAL9
         ↓
HAL9 validates assertion
         ↓
HAL9 creates/updates user
         ↓
HAL9 issues JWT token → User
```

### 2. OAuth2/OIDC Flow

```
User → HAL9 → Authorization URL
         ↓
Provider authenticates user
         ↓
Provider → Code → HAL9
         ↓
HAL9 → Exchange code → Provider
         ↓
Provider → Tokens → HAL9
         ↓
HAL9 → Get user info → Provider
         ↓
HAL9 issues JWT token → User
```

## Security Considerations

### 1. Token Security
- Short-lived access tokens (15 minutes)
- Refresh tokens with rotation
- Token binding to IP/device
- Revocation support

### 2. Session Management
- Concurrent session limits
- Session fixation protection
- Idle timeout enforcement
- Geographic anomaly detection

### 3. Multi-Factor Authentication
- TOTP/HOTP support
- WebAuthn/FIDO2
- SMS/Email backup codes
- Biometric authentication

### 4. API Security
- Rate limiting per organization
- API key rotation policies
- Scope-based permissions
- Request signing

## Database Schema

### Organizations Table
```sql
CREATE TABLE organizations (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    domain VARCHAR(255) UNIQUE,
    subscription_tier VARCHAR(50) NOT NULL,
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Teams Table
```sql
CREATE TABLE teams (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    permissions JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, name)
);
```

### User Organizations Table
```sql
CREATE TABLE user_organizations (
    user_id UUID NOT NULL REFERENCES users(id),
    organization_id UUID NOT NULL REFERENCES organizations(id),
    role VARCHAR(50) NOT NULL,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, organization_id)
);
```

### SSO Configurations Table
```sql
CREATE TABLE sso_configurations (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    provider_type VARCHAR(50) NOT NULL,
    configuration JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## Implementation Priorities

1. **Phase 1: Core SSO**
   - SAML 2.0 support
   - OAuth2/OIDC support
   - Basic organization management

2. **Phase 2: Advanced RBAC**
   - Custom roles and permissions
   - Team management
   - Attribute-based access control

3. **Phase 3: Compliance**
   - Enhanced audit logging
   - GDPR compliance tools
   - Data retention policies

4. **Phase 4: Advanced Features**
   - Risk-based authentication
   - Behavioral analytics
   - Zero-trust architecture

## Integration Examples

### Okta SAML Integration
```yaml
sso:
  provider: okta
  type: saml
  config:
    idp_metadata_url: "https://company.okta.com/app/metadata"
    sp_entity_id: "https://hal9.company.com"
    acs_url: "https://hal9.company.com/auth/saml/callback"
```

### Azure AD OAuth Integration
```yaml
sso:
  provider: azure_ad
  type: oidc
  config:
    tenant_id: "your-tenant-id"
    client_id: "your-client-id"
    discovery_url: "https://login.microsoftonline.com/{tenant}/.well-known/openid-configuration"
```

## Monitoring & Metrics

Key metrics to track:
- SSO success/failure rates
- Authentication latency
- Session duration
- Failed login attempts
- Permission check performance
- Audit log volume

## Conclusion

This enterprise authentication architecture provides HAL9 with:
- Industry-standard SSO support
- Flexible organization management
- Granular access control
- Comprehensive audit trails
- Compliance readiness

The system is designed to scale to thousands of organizations while maintaining security and performance.