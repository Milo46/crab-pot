# Log Server SRD (Software Requirements Document) - v1.0.0

# Table of Contents

- [1. Short description](#1-short-description)
- [2. Purpose of the project](#2-purpose-of-the-project)
- [3. Use Case](#3-use-case)
- [4. Functional Requirements](#4-functional-requirements)
  - [4.1 POST /schemas](#41-post-schemas)
  - [4.2 GET /schemas](#42-get-schemas)
  - [4.2.1 GET /schemas/{id}](#421-get-schemasid)
  - [4.3 POST /logs](#43-post-logs)
  - [4.4 GET /logs](#44-get-logs)
  - [4.4.1 GET /logs/schema/{schema_name}](#441-get-logsschemaschema_name)
  - [4.4.2 GET /logs/schema/{schema_name}/{version}](#442-get-logsschemaschema_nameversion)
  - [4.4.3 GET /logs/{id}](#443-get-logsid)
  - [4.5 PUT /schemas/{id}](#45-put-schemasid)
  - [4.6 DELETE /schemas/{id}](#46-delete-schemasid)
  - [4.7 DELETE /logs/{id}](#47-delete-logsid)
  - [4.8 GET /health](#48-get-health)
  - [4.9 Error Handling](#49-error-handling)
- [5. Non-Functional Requirements](#5-non-functional-requirements)
- [6. System Architecture](#6-system-architecture)
- [7. Data Models](#7-data-models)
- [8. API Specification](#8-api-specification)
- [9. Version Information](#9-version-information)
- [10. Future Improvements](#10-future-improvements)

## 1. Short description

A lightweight HTTP-based **schema-driven log sink** service that allows users to define custom log schemas
and then receive JSON log entries validated against those schemas, storing them in a persistent database,
deployable with Docker Compose.

## 2. Purpose of the project

Polish out software engineering, networking and programming skills through
a small but well-structured system that demonstrates professional development practices.
The project also server as a showcase of:

* Backend service design (HTTP server, routing, validation)
* Database schema design and persistence
* Containerization using Docker and Docker Compose
* Clean repository organization and versioning discipline

## 3. Use Case

Applications, microservices, or scripts can define custom logging schemas and then send structured logs
that conform to those schemas for centralized storage and later retrieval or inspection.

Example workflow:

1. A development team defines a schema for their application logs (e.g., web server access logs)
2. They register this schema with the log server
3. Multiple components send logs conforming to this schema
4. The service validates each log entry against the schema before storage

Benefits:

* **Data consistency**: All logs conform to predefined structures
* **Flexibility**: Different applications can use different log formats
* **Validation**: Invalid log entries are rejected with clear error messages
* **Schema evolution**: Schemas can be versioned and updated over time

## 4. Functional Requirements

### 4.1 POST /schemas

* Accepts a JSON Schema definition that will be used to validate log entries
* Required fields: `name`, `version`, `schema_definition`
* Optional fields: `description`
* Validates that the provided schema is a valid JSON Schema
* Stores the schema definition in the database with an auto-generated UUID
* Returns HTTP 201 on successful creation with the assigned schema UUID
* Supports **only** JSON Schema Draft 7 specification
* Example payload:

    ```json
    {
        "name": "web-server-logs",
        "version": "1.0.0",
        "description": "Schema for web server access logs",
        "schema_definition": {
            "type": "object",
            "required": ["timestamp", "level", "message", "request_id"],
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
        }
    }
    ```

### 4.2 GET /schemas

* Retrieves all registered schemas with optional filtering
* Query parameters (all optional):
  * `name`: Filter schemas by exact name match
  * `version`: Filter schemas by exact version match
* Returns JSON object with `schemas` array
* Filtering is performed at the database level for optimal performance
* Example: `GET /schemas?name=web-server-logs&version=1.0.0`

### 4.2.1 GET /schemas/{id}

* Retrieves a specific schema by its UUID
* Path parameter `id`: The UUID of the schema
* Returns HTTP 200 with schema object
* Returns HTTP 404 if schema not found

### 4.2.2 GET /schemas/{schema_name}/{schema_version}

* Retrieves a specific schema by its combined and name and version
* Path parameters:
  * `schema_name`: The name of the schema
  * `schema_version`: The specific version of the schema
* Returns HTTP 200 with schema object
* Returns HTTP 404 if schema not found

### 4.3 POST /logs

* Accepts a JSON object representing a single log entry with schema reference
* Required fields in request body:
  * `schema_id`: UUID of the schema to validate against
  * `log_data`: JSON object containing the log entry
* Validates the log entry against the specified schema
* Stores validated log entries in PostgreSQL database with schema reference
* Returns HTTP 201 on successful creation with the log entry details
* Returns HTTP 404 if schema_id doesn't exist
* Returns HTTP 422 if log entry doesn't conform to schema
* Example request:

    ```json
    {
        "schema_id": "550e8400-e29b-41d4-a716-446655440000",
        "log_data": {
            "timestamp": "2025-10-23T10:00:00Z",
            "level": "INFO",
            "message": "User login successful",
            "request_id": "req-12345",
            "user_id": "user-67890",
            "response_time_ms": 150
        }
    }
    ```

### 4.4 GET /logs

* Retrieves stored log entries with filtering capabilities

#### 4.4.1 GET /logs/schema/{schema_name}

* Get all logs for a specific schema by name (uses latest version, defaults to 1.0.0)
* Path parameter `schema_name`: The name of the schema
* Query parameters: Any top-level JSONB field for exact-match filtering
* Example: `GET /logs/schema/temperature-readings?location=desk-thermometer&temperature=22.5`

#### 4.4.2 GET /logs/schema/{schema_name}/{schema_version}

* Get all logs for a specific schema name and version
* Path parameters:
  * `schema_name`: The name of the schema
  * `schema_version`: The specific version
* Query parameters: Any top-level JSONB field for exact-match filtering
* Example: `GET /logs/schema/web-server-logs/1.0.0?level=ERROR&user_id=user-123`

#### 4.4.3 GET /logs/{id}

* Retrieve a specific log entry by its numeric ID
* Path parameter `id`: The log entry ID
* Returns HTTP 200 with log entry details
* Returns HTTP 404 if log not found

**Filtering:**
* JSONB field filtering uses PostgreSQL's `@>` containment operator
* Supports exact matching on top-level fields
* Multiple query parameters use AND logic
* All filtering performed at database level using GIN index

### 4.5 PUT /schemas/{id}

* Update an existing schema by UUID
* Path parameter `id`: The UUID of the schema to update
* Request body same as POST /schemas (name, version, description, schema_definition)
* Returns HTTP 200 with updated schema
* Returns HTTP 404 if schema not found

### 4.6 DELETE /schemas/{id}

* Delete a schema by UUID
* Path parameter `id`: The UUID of the schema to delete
* Query parameter `force`: Deletes the schema together with it's logs.
* Returns HTTP 409 when trying to delete a schema that haslogs without the `force` parameter
* Returns HTTP 204 (No Content) on success
* Returns HTTP 404 if schema not found
* Note: Consider cascade deletion or orphan log handling

### 4.7 DELETE /logs/{id}

* Delete a specific log entry by ID
* Path parameter `id`: The numeric ID of the log entry
* Returns HTTP 204 (No Content) on success
* Returns HTTP 404 if log not found

### 4.8 GET /health

* Health check endpoint for monitoring and load balancers
* Also available at `GET /` (root path)
* Returns HTTP 200 with service status information
* Includes database connectivity status (when implemented)
* Response format:
    ```json
    {
        "status": "healthy",
        "service": "log-server",
        "timestamp": "2025-11-13T10:00:00Z"
    }
    ```

### 4.9 Error Handling

* HTTP 400: Invalid JSON, missing required fields, or invalid schema_id
* HTTP 422: Valid JSON but fails schema validation (for logs) or invalid JSON Schema (for schemas)
* HTTP 404: Schema not found for the provided schema_id
* HTTP 500: Internal server errors (database connectivity, etc.)
* All error responses include descriptive error messages and validation details

## 5. Non-Functional Requirements

### 5.1 Performance

* Handle at least 1000 requests per second under normal load
* Database queries should complete within 100ms for typical operations
* Memory usage should remain stable under continuous operation

### 5.2 Reliability

* Service should have 99.9% uptime during normal operations
* Graceful handling of database connection failures
* Proper error logging and recovery mechanisms

### 5.3 Security

* Input validation for all endpoints
* SQL injection prevention through parameterized queries
* Rate limiting to prevent abuse (configurable)

### 5.4 Scalability

* Stateless service design for horizontal scaling
* Database connection pooling for efficient resource usage
* Container-ready for orchestration platforms

### 5.5 Maintainability

* Clean, documented code following Rust best practices
* Comprehensive error handling and logging
* Configuration via environment variables

## 6. System Architecture

Components:

* `app`: Rust-based HTTP server handling requests and DB communication.
* `db`: PostgreSQL database container storing logs.
* `docker-compose.yml`: Defines both services, volumes and internal network.

### 6.1 Technology Stack

* **Backend**: Rust with Axum web framework
* **Database**: PostgreSQL 15+
* **Containerization**: Docker and Docker Compose
* **Serialization**: JSON with serde
* **Database Access**: SQLx for async PostgreSQL operations

### 6.2 Network Architecture

* Internal Docker network for app-database communication
* Exposed HTTP port (default: 8080) for external API access
* Database port not exposed externally for security

## 7. Data Models

### 7.1 Database Schema

```sql
-- Table for storing user-defined schemas
CREATE TABLE schemas (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    schema_definition JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(name, version)
);

-- Table for storing log entries
CREATE TABLE logs (
    id SERIAL PRIMARY KEY,
    schema_id UUID NOT NULL REFERENCES schemas(id),
    log_data JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_logs_schema_id ON logs(schema_id);
CREATE INDEX idx_logs_created_at ON logs(created_at);
CREATE INDEX idx_schemas_name ON schemas(name);
CREATE INDEX idx_schemas_name_version ON schemas(name, version);

-- GIN index for JSON queries on log data
CREATE INDEX idx_logs_data_gin ON logs USING GIN (log_data);
```

### 7.2 API Response Models

**Schema Response:**

```json
{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "web-server-logs",
    "version": "1.0.0",
    "description": "Schema for web server access logs",
    "schema_definition": {
        "type": "object",
        "required": ["timestamp", "level", "message", "request_id"],
        "properties": {
            "timestamp": {"type": "string", "format": "date-time"},
            "level": {"type": "string", "enum": ["DEBUG", "INFO", "WARN", "ERROR"]},
            "message": {"type": "string", "minLength": 1},
            "request_id": {"type": "string", "pattern": "^[a-zA-Z0-9-]+$"}
        }
    },
    "created_at": "2025-10-23T09:00:00Z",
    "updated_at": "2025-10-23T09:00:00Z"
}
```

**Log Entry Response:**

```json
{
    "id": 123,
    "schema_id": "550e8400-e29b-41d4-a716-446655440000",
    "log_data": {
        "timestamp": "2025-10-23T10:00:00Z",
        "level": "INFO",
        "message": "User login successful",
        "request_id": "req-12345",
        "user_id": "user-67890",
        "response_time_ms": 150
    },
    "created_at": "2025-10-23T10:00:01Z"
}
```

**Schema Validation Error Response:**

```json
{
    "error": "CREATION_FAILED",
    "message": "Schema validation failed: Validation error at '/request_id': 'request_id' is a required property"
}
```

**General Error Response:**

```json
{
    "error": "NOT_FOUND",
    "message": "Schema with id '550e8400-e29b-41d4-a716-446655440000' not found"
}
```

## 8. API Specification

### 8.1 Base URL

* Development: `http://localhost:8080`
* Production: Configurable via environment variables

### 8.2 Content Types

* Request: `application/json`
* Response: `application/json`

### 8.3 Authentication

* **Current (v1.0.0)**: No authentication implemented
  * All endpoints are publicly accessible
  * Suitable for development environments and trusted internal networks only
  * **Not recommended for production without network-level security**
* **Planned (v1.1.0+)**: Optional API key authentication
  * Environment variable-based API key configuration
  * Header-based authentication (`X-API-Key`)
  * Public endpoints: `/`, `/health`
  * Protected endpoints: All schema and log management operations
* **Future (v2.0.0)**: Advanced authentication & authorization
  * JWT-based authentication
  * Role-based access control (RBAC)
  * Multi-tenant support

### 8.4 Rate Limiting

* Default: 1000 requests per minute per IP
* Configurable via environment variables
* Returns HTTP 429 when exceeded

### 8.5 Schema Validation

* All log entries must conform to a pre-registered schema
* JSON Schema Draft 7 specification is used for validation
* Schemas are versioned to support evolution over time
* Invalid log entries are rejected with detailed error messages

## 9. Version Information

* **Current Version**: 1.0.0
* **API Version**: v1
* **Database Schema Version**: 1.0
* **Compatibility**:
  * Rust 1.82+ (2021 edition)
  * PostgreSQL 16+
  * Docker 20.10+
  * Docker Compose 2.0+

### 9.1 Versioning Strategy

* Semantic versioning (MAJOR.MINOR.PATCH)
* API versioning through URL path (`/api/v1/`)
* Database migrations for schema changes
* Backward compatibility maintained within major versions

## 10. Future Improvements

### 10.1 Phase 2 Features

* **Authentication & Authorization**
  * JWT-based authentication
  * Role-based access control
  * API key management

* **Advanced Querying**
  * Full-text search in log messages
  * Advanced filtering with logical operators
  * Aggregation endpoints (counts, statistics)

* **Monitoring & Observability**
  * Prometheus metrics endpoint
  * Structured logging for the service itself
  * Health check with detailed component status

### 10.2 Phase 3 Features

* **Data Management**
  * Log retention policies
  * Automatic archiving of old logs
  * Data compression for storage optimization

* **Performance Enhancements**
  * Redis caching layer
  * Connection pooling optimization
  * Batch insertion capabilities

* **Integration Features**
  * Webhook notifications for critical logs
  * Integration with popular log aggregation tools
  * Export capabilities (CSV, JSON, etc.)

### 10.3 Operational Improvements

* **Deployment**
  * Kubernetes manifests
  * Helm charts
  * CI/CD pipeline with automated testing

* **Security**
  * TLS/SSL support
  * Input sanitization
  * Audit logging

* **Documentation**
  * Interactive API documentation (Swagger UI)
  * Client SDKs for popular languages
  * Deployment guides for various platforms
