-- Enterprise features tables for SQLite

-- Organizations table
CREATE TABLE IF NOT EXISTS organizations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    domain TEXT UNIQUE,
    subscription_tier TEXT NOT NULL DEFAULT 'free',
    settings TEXT DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    CHECK (subscription_tier IN ('free', 'starter', 'professional', 'enterprise'))
);

CREATE INDEX idx_organizations_domain ON organizations(domain);
CREATE INDEX idx_organizations_created_at ON organizations(created_at);

-- User-Organization relationship
CREATE TABLE IF NOT EXISTS user_organizations (
    user_id TEXT NOT NULL,
    organization_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'member',
    joined_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (user_id, organization_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    CHECK (role IN ('owner', 'admin', 'member', 'viewer'))
);

CREATE INDEX idx_user_organizations_user_id ON user_organizations(user_id);
CREATE INDEX idx_user_organizations_organization_id ON user_organizations(organization_id);

-- Teams table
CREATE TABLE IF NOT EXISTS teams (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    permissions TEXT DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    UNIQUE(organization_id, name)
);

CREATE INDEX idx_teams_organization_id ON teams(organization_id);
CREATE INDEX idx_teams_name ON teams(name);

-- Team members table
CREATE TABLE IF NOT EXISTS team_members (
    user_id TEXT NOT NULL,
    team_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'member',
    joined_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (user_id, team_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    CHECK (role IN ('lead', 'member'))
);

CREATE INDEX idx_team_members_user_id ON team_members(user_id);
CREATE INDEX idx_team_members_team_id ON team_members(team_id);

-- Update existing audit_log table to add enterprise features
ALTER TABLE audit_log ADD COLUMN organization_id TEXT;
ALTER TABLE audit_log ADD COLUMN risk_score INTEGER DEFAULT 0;
ALTER TABLE audit_log ADD COLUMN session_id TEXT;
ALTER TABLE audit_log ADD COLUMN success INTEGER DEFAULT 1;
ALTER TABLE audit_log ADD COLUMN error_message TEXT;
ALTER TABLE audit_log ADD COLUMN metadata TEXT DEFAULT '{}';
ALTER TABLE audit_log ADD COLUMN timestamp TEXT DEFAULT CURRENT_TIMESTAMP;

CREATE INDEX idx_audit_log_organization_id ON audit_log(organization_id);
CREATE INDEX idx_audit_log_risk_score ON audit_log(risk_score);
CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp);

-- SSO configurations table
CREATE TABLE IF NOT EXISTS sso_configurations (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 0,
    config TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    UNIQUE(organization_id, provider),
    CHECK (provider IN ('saml', 'oidc', 'google', 'microsoft', 'okta'))
);

CREATE INDEX idx_sso_configurations_organization_id ON sso_configurations(organization_id);

-- API usage tracking
CREATE TABLE IF NOT EXISTS api_usage (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    user_id TEXT,
    endpoint TEXT NOT NULL,
    method TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    response_time_ms INTEGER NOT NULL,
    timestamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE
);

CREATE INDEX idx_api_usage_organization_id ON api_usage(organization_id);
CREATE INDEX idx_api_usage_timestamp ON api_usage(timestamp);
CREATE INDEX idx_api_usage_endpoint ON api_usage(endpoint);

-- Add organization_id to neurons table
ALTER TABLE neurons ADD COLUMN organization_id TEXT;
CREATE INDEX idx_neurons_organization_id ON neurons(organization_id);