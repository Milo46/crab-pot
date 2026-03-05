CREATE TYPE api_key_role AS ENUM (
    'reader', 'writer', 'schema_manager', 'log_manager', 'admin'
);

ALTER TABLE api_keys
    ADD COLUMN role api_key_role NOT NULL DEFAULT 'writer';