-- Initialize Log Server Database
-- This script sets up the initial database schema

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create schemas table for storing user-defined log schemas
CREATE TABLE IF NOT EXISTS schemas (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    schema_definition JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(name, version)
);

-- Create logs table for storing log entries
CREATE TABLE IF NOT EXISTS logs (
    id SERIAL PRIMARY KEY,
    schema_id UUID NOT NULL REFERENCES schemas(id),
    log_data JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_logs_schema_id ON logs(schema_id);
CREATE INDEX IF NOT EXISTS idx_logs_created_at ON logs(created_at);
CREATE INDEX IF NOT EXISTS idx_schemas_name ON schemas(name);
CREATE INDEX IF NOT EXISTS idx_schemas_name_version ON schemas(name, version);

-- GIN index for JSON queries on log data
CREATE INDEX IF NOT EXISTS idx_logs_data_gin ON logs USING GIN (log_data);

-- Insert sample schema for testing
INSERT INTO schemas (id, name, version, description, schema_definition) 
VALUES (
    '550e8400-e29b-41d4-a716-446655440000',
    'web-server-logs',
    '1.0.0',
    'Schema for web server access logs',
    '{
        "type": "object",
        "required": ["level", "message", "request_id"],
        "properties": {
            "timestamp": {
                "type": "string",
                "format": "date-time"
            },
            "level": {
                "type": "string",
                "enum": ["DEBUG", "INFO", "WARN", "ERROR"]
            },
            "message": {
                "type": "string",
                "minLength": 1
            },
            "request_id": {
                "type": "string",
                "pattern": "^[a-zA-Z0-9-]+$"
            },
            "user_id": {
                "type": "string"
            },
            "response_time_ms": {
                "type": "number",
                "minimum": 0
            }
        }
    }'::jsonb
) ON CONFLICT (id) DO NOTHING;

-- Insert sample log entry for testing
INSERT INTO logs (schema_id, log_data)
VALUES (
    '550e8400-e29b-41d4-a716-446655440000',
    '{
        "timestamp": "2025-10-26T10:00:00Z",
        "level": "INFO",
        "message": "Sample log entry created during database initialization",
        "request_id": "init-001"
    }'::jsonb
);

-- Create a function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically update updated_at
DROP TRIGGER IF EXISTS update_schemas_updated_at ON schemas;
CREATE TRIGGER update_schemas_updated_at
    BEFORE UPDATE ON schemas
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- @{todo}(milo): finialize the permissions for the production workflow
-- Grant permissions (for production use)
-- CREATE USER loguser WITH PASSWORD 'secure_password';
-- GRANT CONNECT ON DATABASE logs TO loguser;
-- GRANT USAGE ON SCHEMA public TO loguser;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO loguser;
-- GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO loguser;
