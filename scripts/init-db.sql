-- zkIPFS Proof Database Initialization Script
-- This script runs automatically when the PostgreSQL container starts

-- Create extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Proof Jobs Table
CREATE TABLE IF NOT EXISTS proof_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    status VARCHAR(20) NOT NULL DEFAULT 'Pending',
    file_hash VARCHAR(64),
    ipfs_cid VARCHAR(100),
    content_selection TEXT,
    security_level INTEGER DEFAULT 128,
    result JSONB,
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE
);

-- Proof Cache Table (for verified proofs)
CREATE TABLE IF NOT EXISTS proof_cache (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    content_hash VARCHAR(64) UNIQUE NOT NULL,
    proof_data BYTEA NOT NULL,
    public_inputs JSONB,
    verification_key BYTEA,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    access_count INTEGER DEFAULT 0
);

-- Users Table (for future authentication)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE,
    api_key VARCHAR(64) UNIQUE,
    rate_limit INTEGER DEFAULT 100,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_active TIMESTAMP WITH TIME ZONE
);

-- API Usage Logs (for monitoring)
CREATE TABLE IF NOT EXISTS api_logs (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    endpoint VARCHAR(100) NOT NULL,
    method VARCHAR(10) NOT NULL,
    status_code INTEGER,
    response_time_ms INTEGER,
    ip_address INET,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_proof_jobs_status ON proof_jobs(status);
CREATE INDEX IF NOT EXISTS idx_proof_jobs_created_at ON proof_jobs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_proof_cache_content_hash ON proof_cache(content_hash);
CREATE INDEX IF NOT EXISTS idx_proof_cache_expires_at ON proof_cache(expires_at);
CREATE INDEX IF NOT EXISTS idx_api_logs_created_at ON api_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_api_logs_user_id ON api_logs(user_id);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger for proof_jobs updated_at
DROP TRIGGER IF EXISTS update_proof_jobs_updated_at ON proof_jobs;
CREATE TRIGGER update_proof_jobs_updated_at
    BEFORE UPDATE ON proof_jobs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Insert a default system user for anonymous API access
INSERT INTO users (email, api_key, rate_limit)
VALUES ('system@zkipfs.local', 'default-api-key-change-in-production', 1000)
ON CONFLICT (email) DO NOTHING;

COMMENT ON TABLE proof_jobs IS 'Tracks ZK proof generation jobs and their status';
COMMENT ON TABLE proof_cache IS 'Caches verified proofs for faster retrieval';
COMMENT ON TABLE users IS 'User accounts and API keys for rate limiting';
COMMENT ON TABLE api_logs IS 'API usage logs for monitoring and debugging';
