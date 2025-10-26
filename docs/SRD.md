# Log Server SRD (Software Requirements Document) - v1.0.0

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
* Required fields: `name`, `version`, `schema`
* Optional fields: `description`
* Validates that the provided schema is a valid JSON Schema
* Stores the schema definition in the database with a unique ID
* Returns HTTP 201 on successful creation with the assigned schema ID
* Supports JSON Schema Draft 7 specification
* Example payload:

    ```json
    {
        "name": "web-server-logs",
        "version": "1.0.0",
        "description": "Schema for web server access logs",
        "schema": {
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

* Retrieves all registered schemas or a specific schema by ID
* Query parameters:
  * `id`: Retrieve specific schema by ID
  * `name`: Filter schemas by name
  * `version`: Filter schemas by version
* Returns JSON array of schema definitions or single schema object
* Includes metadata like creation date and usage statistics

### 4.3 POST /logs/{schema_id}

* Accepts a JSON object representing a single log entry
* Validates the log entry against the specified schema
* Path parameter `schema_id`: The unique name identifier of the schema to validate against
* Stores validated log entries in PostgreSQL database with schema reference
* Returns HTTP 201 on successful creation
* Returns HTTP 400 if schema_id doesn't exist
* Returns HTTP 422 if log entry doesn't conform to schema
* Example request to `/logs/web-server-logs`:

    ```json
    {
        "timestamp": "2025-10-23T10:00:00Z",
        "level": "INFO",
        "message": "User login successful",
        "request_id": "req-12345",
        "user_id": "user-67890",
        "response_time_ms": 150
    }
    ```

### 4.4 GET /logs

* Retrieves stored log entries with optional filtering
* Query parameters:
  * `limit`: Maximum number of entries (default: 100, max: 1000)
  * `offset`: Number of entries to skip (default: 0)
  * `schema_id`: Filter by schema ID
  * `from`: Start timestamp (ISO 8601 format)
  * `to`: End timestamp (ISO 8601 format)
* Returns JSON array of log entries with schema information
* Supports pagination via `limit` and `offset`
* Each log entry includes the schema ID it was validated against

### 4.5 GET /health

* Health check endpoint for monitoring and load balancers
* Returns HTTP 200 with service status information
* Includes database connectivity status

### 4.6 Error Handling

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
    id VARCHAR(255) PRIMARY KEY,
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
    schema_id VARCHAR(255) NOT NULL REFERENCES schemas(id),
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
    "id": "web-server-logs",
    "name": "web-server-logs",
    "version": "1.0.0",
    "description": "Schema for web server access logs",
    "schema": {
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
    "schema_id": "web-server-logs",
    "schema_name": "web-server-logs",
    "schema_version": "1.0.0",
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
    "error": "Schema validation failed",
    "details": "Missing required field: request_id",
    "schema_id": "web-server-logs",
    "violations": [
        {
            "field": "request_id",
            "message": "Field is required but missing"
        },
        {
            "field": "response_time_ms",
            "message": "Value must be a non-negative number"
        }
    ]
}
```

**General Error Response:**

```json
{
    "error": "Schema not found",
    "details": "No schema found with ID: invalid-schema-name"
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

* Phase 1: No authentication (suitable for internal networks)
* Future: Bearer token or API key authentication

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
  * Rust 1.70+
  * PostgreSQL 15+
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
