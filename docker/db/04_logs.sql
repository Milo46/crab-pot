CREATE TABLE IF NOT EXISTS logs (
    id SERIAL PRIMARY KEY,
    schema_id UUID NOT NULL REFERENCES schemas(id),
    log_data JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_logs_schema_id ON logs(schema_id);
CREATE INDEX IF NOT EXISTS idx_logs_created_at ON logs(created_at);
CREATE INDEX IF NOT EXISTS idx_logs_schema_created_id ON logs(schema_id, created_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_logs_data_gin ON logs USING GIN (log_data);
