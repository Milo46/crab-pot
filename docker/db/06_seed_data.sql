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
