-- Enterprise features tables for PostgreSQL

-- Organizations table
CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    domain VARCHAR(255) UNIQUE,
    subscription_tier VARCHAR(50) NOT NULL DEFAULT 'free',
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT valid_subscription_tier CHECK (subscription_tier IN ('free', 'starter', 'professional', 'enterprise'))
);

CREATE INDEX idx_organizations_domain ON organizations(domain);
CREATE INDEX idx_organizations_created_at ON organizations(created_at);

-- User-Organization relationship
CREATE TABLE IF NOT EXISTS user_organizations (
    user_id UUID NOT NULL,
    organization_id UUID NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (user_id, organization_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CONSTRAINT valid_role CHECK (role IN ('owner', 'admin', 'member', 'viewer'))
);

CREATE INDEX idx_user_organizations_user_id ON user_organizations(user_id);
CREATE INDEX idx_user_organizations_organization_id ON user_organizations(organization_id);

-- Teams table
CREATE TABLE IF NOT EXISTS teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    permissions JSONB DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    UNIQUE(organization_id, name)
);

CREATE INDEX idx_teams_organization_id ON teams(organization_id);
CREATE INDEX idx_teams_name ON teams(name);

-- Team members table
CREATE TABLE IF NOT EXISTS team_members (
    user_id UUID NOT NULL,
    team_id UUID NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (user_id, team_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    CONSTRAINT valid_team_role CHECK (role IN ('lead', 'member'))
);

CREATE INDEX idx_team_members_user_id ON team_members(user_id);
CREATE INDEX idx_team_members_team_id ON team_members(team_id);

-- Update existing audit_log table to add enterprise features
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS organization_id UUID;
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS resource_type VARCHAR(100);
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS resource_id VARCHAR(255);
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS risk_score INTEGER DEFAULT 0;
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS session_id UUID;
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS success BOOLEAN DEFAULT true;
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS error_message TEXT;

-- Add foreign key constraints
ALTER TABLE audit_log ADD CONSTRAINT fk_audit_log_organization 
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_audit_log_organization_id ON audit_log(organization_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_risk_score ON audit_log(risk_score);

-- SSO configurations table
CREATE TABLE IF NOT EXISTS sso_configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    provider VARCHAR(50) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT false,
    config JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    UNIQUE(organization_id, provider),
    CONSTRAINT valid_provider CHECK (provider IN ('saml', 'oidc', 'google', 'microsoft', 'okta'))
);

CREATE INDEX idx_sso_configurations_organization_id ON sso_configurations(organization_id);

-- API usage tracking
CREATE TABLE IF NOT EXISTS api_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    user_id UUID,
    endpoint VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    status_code INTEGER NOT NULL,
    response_time_ms INTEGER NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_api_usage_organization_id ON api_usage(organization_id);
CREATE INDEX idx_api_usage_timestamp ON api_usage(timestamp);
CREATE INDEX idx_api_usage_endpoint ON api_usage(endpoint);

-- Add organization_id to neurons table
ALTER TABLE neurons ADD COLUMN IF NOT EXISTS organization_id UUID;
CREATE INDEX IF NOT EXISTS idx_neurons_organization_id ON neurons(organization_id);