-- Enterprise features for SQLite

-- Organizations table
CREATE TABLE IF NOT EXISTS organizations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    domain TEXT UNIQUE,
    subscription_tier TEXT NOT NULL,
    settings TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Teams table
CREATE TABLE IF NOT EXISTS teams (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL REFERENCES organizations(id),
    name TEXT NOT NULL,
    description TEXT,
    permissions TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(organization_id, name)
);

-- User organizations table
CREATE TABLE IF NOT EXISTS user_organizations (
    user_id TEXT NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id),
    role TEXT NOT NULL,
    joined_at INTEGER NOT NULL,
    PRIMARY KEY (user_id, organization_id)
);

-- Team members table
CREATE TABLE IF NOT EXISTS team_members (
    user_id TEXT NOT NULL,
    team_id TEXT NOT NULL REFERENCES teams(id),
    role TEXT NOT NULL,
    joined_at INTEGER NOT NULL,
    PRIMARY KEY (user_id, team_id)
);

-- SSO configurations table
CREATE TABLE IF NOT EXISTS sso_configurations (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL REFERENCES organizations(id),
    provider_type TEXT NOT NULL,
    configuration TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Roles table
CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    permissions TEXT NOT NULL,
    is_system INTEGER NOT NULL DEFAULT 0,
    organization_id TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Role assignments table
CREATE TABLE IF NOT EXISTS role_assignments (
    user_id TEXT NOT NULL,
    role_id TEXT NOT NULL REFERENCES roles(id),
    scope TEXT NOT NULL,
    granted_by TEXT NOT NULL,
    granted_at INTEGER NOT NULL,
    expires_at INTEGER,
    PRIMARY KEY (user_id, role_id)
);

-- Audit log table
CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    organization_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    session_id TEXT,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    ip_address TEXT NOT NULL,
    user_agent TEXT NOT NULL,
    details TEXT NOT NULL DEFAULT '{}',
    risk_score REAL NOT NULL DEFAULT 0.0,
    status TEXT NOT NULL,
    error_message TEXT
);

-- Data subject requests table
CREATE TABLE IF NOT EXISTS data_subject_requests (
    id TEXT PRIMARY KEY,
    request_type TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    subject_email TEXT NOT NULL,
    organization_id TEXT NOT NULL,
    status TEXT NOT NULL,
    details TEXT NOT NULL DEFAULT '{}',
    requested_at INTEGER NOT NULL,
    completed_at INTEGER,
    response TEXT
);

-- Consent records table
CREATE TABLE IF NOT EXISTS consent_records (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    organization_id TEXT NOT NULL,
    purpose TEXT NOT NULL,
    legal_basis TEXT NOT NULL,
    granted INTEGER NOT NULL,
    version TEXT NOT NULL,
    ip_address TEXT NOT NULL,
    user_agent TEXT NOT NULL,
    granted_at INTEGER NOT NULL,
    expires_at INTEGER,
    withdrawn_at INTEGER
);

-- Create indexes
CREATE INDEX idx_organizations_domain ON organizations(domain);
CREATE INDEX idx_teams_organization ON teams(organization_id);
CREATE INDEX idx_user_organizations_user ON user_organizations(user_id);
CREATE INDEX idx_user_organizations_org ON user_organizations(organization_id);
CREATE INDEX idx_team_members_user ON team_members(user_id);
CREATE INDEX idx_team_members_team ON team_members(team_id);
CREATE INDEX idx_roles_organization ON roles(organization_id);
CREATE INDEX idx_role_assignments_user ON role_assignments(user_id);
CREATE INDEX idx_audit_log_organization ON audit_log(organization_id);
CREATE INDEX idx_audit_log_user ON audit_log(user_id);
CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp);
CREATE INDEX idx_consent_records_user ON consent_records(user_id);
CREATE INDEX idx_consent_records_organization ON consent_records(organization_id);