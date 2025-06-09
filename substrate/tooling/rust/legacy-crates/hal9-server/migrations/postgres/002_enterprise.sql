-- Enterprise features for PostgreSQL

-- Organizations table
CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    domain VARCHAR(255) UNIQUE,
    subscription_tier VARCHAR(50) NOT NULL,
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Teams table
CREATE TABLE IF NOT EXISTS teams (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    permissions JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, name)
);

-- User organizations table
CREATE TABLE IF NOT EXISTS user_organizations (
    user_id UUID NOT NULL,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    role VARCHAR(50) NOT NULL,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, organization_id)
);

-- Team members table
CREATE TABLE IF NOT EXISTS team_members (
    user_id UUID NOT NULL,
    team_id UUID NOT NULL REFERENCES teams(id),
    role VARCHAR(50) NOT NULL,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, team_id)
);

-- SSO configurations table
CREATE TABLE IF NOT EXISTS sso_configurations (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    provider_type VARCHAR(50) NOT NULL,
    configuration JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Roles table
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    permissions JSONB NOT NULL,
    is_system BOOLEAN NOT NULL DEFAULT false,
    organization_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Role assignments table
CREATE TABLE IF NOT EXISTS role_assignments (
    user_id UUID NOT NULL,
    role_id UUID NOT NULL REFERENCES roles(id),
    scope JSONB NOT NULL,
    granted_by UUID NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    PRIMARY KEY (user_id, role_id)
);

-- Audit log table
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    organization_id UUID NOT NULL,
    user_id UUID NOT NULL,
    session_id UUID,
    action VARCHAR(255) NOT NULL,
    resource_type VARCHAR(255) NOT NULL,
    resource_id VARCHAR(255) NOT NULL,
    ip_address INET NOT NULL,
    user_agent TEXT NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    risk_score REAL NOT NULL DEFAULT 0.0,
    status VARCHAR(50) NOT NULL,
    error_message TEXT
);

-- Add partitioning for audit log by month
CREATE TABLE IF NOT EXISTS audit_log_template (LIKE audit_log INCLUDING ALL);
-- Partitions will be created dynamically

-- Data subject requests table
CREATE TABLE IF NOT EXISTS data_subject_requests (
    id UUID PRIMARY KEY,
    request_type VARCHAR(50) NOT NULL,
    subject_id UUID NOT NULL,
    subject_email VARCHAR(255) NOT NULL,
    organization_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    response JSONB
);

-- Consent records table
CREATE TABLE IF NOT EXISTS consent_records (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    organization_id UUID NOT NULL,
    purpose VARCHAR(255) NOT NULL,
    legal_basis VARCHAR(50) NOT NULL,
    granted BOOLEAN NOT NULL,
    version VARCHAR(50) NOT NULL,
    ip_address INET NOT NULL,
    user_agent TEXT NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    withdrawn_at TIMESTAMPTZ
);

-- Users table (adding enterprise fields)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255),
    name VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    processing_restricted BOOLEAN NOT NULL DEFAULT false
);

-- Neurons table (adding organization support)
CREATE TABLE IF NOT EXISTS neurons (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    layer VARCHAR(50) NOT NULL,
    organization_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Signals table (adding user tracking)
CREATE TABLE IF NOT EXISTS signals (
    id UUID PRIMARY KEY,
    neuron_id UUID REFERENCES neurons(id),
    user_id UUID REFERENCES users(id),
    organization_id UUID,
    data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
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
CREATE INDEX idx_audit_log_action ON audit_log(action);
CREATE INDEX idx_consent_records_user ON consent_records(user_id);
CREATE INDEX idx_consent_records_organization ON consent_records(organization_id);
CREATE INDEX idx_neurons_organization ON neurons(organization_id);
CREATE INDEX idx_signals_user ON signals(user_id);
CREATE INDEX idx_signals_organization ON signals(organization_id);