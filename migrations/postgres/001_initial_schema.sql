-- Initial PostgreSQL schema for HAL9

-- Enable extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Neurons table
CREATE TABLE IF NOT EXISTS neurons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    layer VARCHAR(10) NOT NULL,
    system_prompt TEXT NOT NULL,
    settings JSONB DEFAULT '{}',
    state VARCHAR(50) DEFAULT 'idle',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_neurons_layer ON neurons(layer);
CREATE INDEX idx_neurons_state ON neurons(state);
CREATE INDEX idx_neurons_created_at ON neurons(created_at DESC);

-- Signals table with partitioning
CREATE TABLE IF NOT EXISTS signals (
    id UUID DEFAULT gen_random_uuid(),
    from_neuron VARCHAR(255) NOT NULL,
    to_neuron VARCHAR(255) NOT NULL,
    layer_from VARCHAR(10) NOT NULL,
    layer_to VARCHAR(10) NOT NULL,
    propagation_type VARCHAR(20) NOT NULL DEFAULT 'forward',
    content TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, timestamp)
) PARTITION BY RANGE (timestamp);

-- Create initial partitions
CREATE TABLE signals_2025_01 PARTITION OF signals
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');

CREATE TABLE signals_2025_02 PARTITION OF signals
    FOR VALUES FROM ('2025-02-01') TO ('2025-03-01');

CREATE TABLE signals_2025_03 PARTITION OF signals
    FOR VALUES FROM ('2025-03-01') TO ('2025-04-01');

-- Indexes on signals
CREATE INDEX idx_signals_timestamp ON signals(timestamp DESC);
CREATE INDEX idx_signals_from_neuron ON signals(from_neuron);
CREATE INDEX idx_signals_to_neuron ON signals(to_neuron);
CREATE INDEX idx_signals_layers ON signals(layer_from, layer_to);

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login TIMESTAMPTZ
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_role ON users(role);

-- API keys table
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) UNIQUE NOT NULL,
    permissions JSONB DEFAULT '[]',
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX idx_api_keys_key_hash ON api_keys(key_hash);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) UNIQUE NOT NULL,
    ip_address INET,
    user_agent TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_token_hash ON sessions(token_hash);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

-- Memories table
CREATE TABLE IF NOT EXISTS memories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    neuron_id VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    importance FLOAT DEFAULT 0.5,
    context JSONB DEFAULT '{}',
    embedding VECTOR(1536), -- For future use with pgvector
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_memories_neuron_id ON memories(neuron_id);
CREATE INDEX idx_memories_importance ON memories(importance DESC);
CREATE INDEX idx_memories_created_at ON memories(created_at DESC);

-- Full text search on memories
ALTER TABLE memories ADD COLUMN search_vector tsvector;
CREATE INDEX idx_memories_search ON memories USING GIN(search_vector);

-- Update search vector trigger
CREATE OR REPLACE FUNCTION update_memories_search_vector()
RETURNS trigger AS $$
BEGIN
    NEW.search_vector := to_tsvector('english', NEW.content);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER memories_search_vector_update
    BEFORE INSERT OR UPDATE ON memories
    FOR EACH ROW
    EXECUTE FUNCTION update_memories_search_vector();

-- Metrics table (for aggregated data)
CREATE TABLE IF NOT EXISTS metrics_aggregate (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_name VARCHAR(255) NOT NULL,
    neuron_id VARCHAR(255),
    value NUMERIC NOT NULL,
    labels JSONB DEFAULT '{}',
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_metrics_name_time ON metrics_aggregate(metric_name, timestamp DESC);
CREATE INDEX idx_metrics_neuron_id ON metrics_aggregate(neuron_id);

-- Audit log table
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(255) NOT NULL,
    resource_type VARCHAR(255),
    resource_id VARCHAR(255),
    details JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_user_id ON audit_log(user_id);
CREATE INDEX idx_audit_action ON audit_log(action);
CREATE INDEX idx_audit_created_at ON audit_log(created_at DESC);

-- Update timestamp function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Add update triggers
CREATE TRIGGER update_neurons_updated_at BEFORE UPDATE ON neurons
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to create monthly partitions automatically
CREATE OR REPLACE FUNCTION create_monthly_partition(table_name text, start_date date)
RETURNS void AS $$
DECLARE
    partition_name text;
    start_timestamp timestamp;
    end_timestamp timestamp;
BEGIN
    partition_name := table_name || '_' || to_char(start_date, 'YYYY_MM');
    start_timestamp := start_date;
    end_timestamp := start_date + interval '1 month';
    
    EXECUTE format('CREATE TABLE IF NOT EXISTS %I PARTITION OF %I FOR VALUES FROM (%L) TO (%L)',
        partition_name, table_name, start_timestamp, end_timestamp);
END;
$$ LANGUAGE plpgsql;

-- Create a maintenance job to ensure partitions exist
CREATE OR REPLACE FUNCTION maintain_partitions()
RETURNS void AS $$
DECLARE
    current_date date;
    future_date date;
BEGIN
    current_date := date_trunc('month', CURRENT_DATE);
    
    -- Create partitions for next 3 months
    FOR i IN 0..2 LOOP
        future_date := current_date + (i || ' months')::interval;
        PERFORM create_monthly_partition('signals', future_date);
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Initial data
INSERT INTO users (username, password_hash, role) VALUES
    ('admin', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewKyNiGH1mQrFZiO', 'admin')
ON CONFLICT (username) DO NOTHING;