# Log Server SRD (Software Requirements Document) - v1.0.0

# Table of Contents

- [1. Short description](#1-short-description)
- [2. Purpose of the project](#2-purpose-of-the-project)
- [3. Use Case](#3-use-case)
- [4. Functional Requirements](#4-functional-requirements)
  - [4.1 Schema Management](#41-schema-management)
  - [4.2 Log Management](#42-log-management)
  - [4.3 Query & Filtering](#43-query--filtering)
  - [4.4 Real-time Events](#44-real-time-events)
  - [4.5 System Operations](#45-system-operations)
- [5. API Endpoints](#5-api-endpoints)
  - [5.1 POST /schemas](#51-post-schemas)
  - [5.2 GET /schemas](#52-get-schemas)
  - [5.3 GET /schemas/{id}](#53-get-schemasid)
  - [5.4 GET /schemas/{schema_name}/versions/{schema_version}](#54-get-schemasschema_nameversionsschema_version)
  - [5.5 PUT /schemas/{id}](#55-put-schemasid)
  - [5.6 DELETE /schemas/{id}](#56-delete-schemasid)
  - [5.7 POST /logs](#57-post-logs)
  - [5.8 GET /logs/schema/{schema_name}](#58-get-logsschemaschema_name)
  - [5.9 GET /logs/schema/{schema_name}/versions/{schema_version}](#59-get-logsschemaschema_nameversionsschema_version)
  - [5.10 POST /logs/schema/{schema_name}/query](#510-post-logsschemaschema_namequery)
  - [5.11 POST /logs/schema/{schema_name}/versions/{schema_version}/query](#511-post-logsschemaschema_nameversionsschema_versionquery)
  - [5.12 GET /logs/{id}](#512-get-logsid)
  - [5.13 DELETE /logs/{id}](#513-delete-logsid)
  - [5.14 GET /ws/logs](#514-get-wslogs)
  - [5.15 GET /health](#515-get-health)
  - [5.16 Request Tracking](#516-request-tracking)
  - [5.17 Error Handling](#517-error-handling)
- [6. Non-Functional Requirements](#6-non-functional-requirements)
- [7. System Architecture](#7-system-architecture)
- [8. Data Models](#8-data-models)
- [9. API Specification](#9-api-specification)
- [10. Version Information](#10-version-information)
- [11. Future Improvements](#11-future-improvements)

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

This section defines the functional requirements using testable, verifiable statements. Each requirement is identified with a unique FR-XXX code for traceability.

### 4.1 Schema Management

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-101 | The system SHALL allow users to create log schemas with a name, version, description, and JSON Schema definition | Must |
| FR-102 | The system SHALL enforce unique combinations of schema name and version | Must |
| FR-103 | The system SHALL validate that schema definitions conform to JSON Schema Draft 7 specification | Must |
| FR-104 | The system SHALL generate a UUID for each newly created schema | Must |
| FR-105 | The system SHALL allow users to retrieve all registered schemas | Must |
| FR-106 | The system SHALL allow filtering schemas by name and/or version | Should |
| FR-107 | The system SHALL allow users to retrieve a specific schema by UUID | Must |
| FR-108 | The system SHALL allow users to retrieve a specific schema by name and version | Must |
| FR-109 | The system SHALL allow users to update an existing schema by UUID | Must |
| FR-110 | The system SHALL allow users to delete a schema by UUID | Must |
| FR-111 | The system SHALL prevent deletion of schemas that have associated logs unless force flag is provided | Must |
| FR-112 | The system SHALL cascade delete all associated logs when force deletion is requested | Should |
| FR-113 | The system SHALL automatically resolve "latest" version when only schema name is provided | Must |

### 4.2 Log Management

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-201 | The system SHALL allow users to create log entries referencing a schema by UUID | Must |
| FR-202 | The system SHALL validate log data against the referenced schema before storage | Must |
| FR-203 | The system SHALL reject log entries that do not conform to their schema with descriptive errors | Must |
| FR-204 | The system SHALL reject log entries referencing non-existent schema UUIDs | Must |
| FR-205 | The system SHALL store log entries with automatic timestamps | Must |
| FR-206 | The system SHALL allow users to retrieve log entries by schema name (using latest version) | Must |
| FR-207 | The system SHALL allow users to retrieve log entries by schema name and specific version | Must |
| FR-208 | The system SHALL allow users to retrieve a specific log entry by numeric ID | Must |
| FR-209 | The system SHALL allow users to delete a specific log entry by ID | Must |

### 4.3 Query & Filtering

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-301 | The system SHALL support pagination with configurable page number and limit | Must |
| FR-302 | The system SHALL return pagination metadata (page, limit, total, total_pages) in responses | Must |
| FR-303 | The system SHALL support filtering logs by date range (date_begin, date_end) | Must |
| FR-304 | The system SHALL return timewindow metadata when date filters are applied | Should |
| FR-305 | The system SHALL support filtering logs by exact field matching using JSONB containment | Must |
| FR-306 | The system SHALL apply multiple filters using AND logic | Must |
| FR-307 | The system SHALL perform filtering at the database level using appropriate indexes | Should |
| FR-308 | The system SHALL support complex queries via POST endpoints with JSON body | Should |

### 4.4 Real-time Events

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-401 | The system SHALL provide a WebSocket endpoint for real-time log events | Should |
| FR-402 | The system SHALL broadcast log creation events to connected WebSocket clients | Should |
| FR-403 | The system SHALL broadcast log deletion events to connected WebSocket clients | Should |

### 4.5 System Operations

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-501 | The system SHALL provide a health check endpoint returning service status | Must |
| FR-502 | The system SHALL include a timestamp in health check responses | Should |
| FR-503 | The system SHALL support request tracking via X-Request-ID header | Must |
| FR-504 | The system SHALL generate a UUID for requests without X-Request-ID header | Must |
| FR-505 | The system SHALL echo the X-Request-ID in all responses | Must |
| FR-506 | The system SHALL include request IDs in all server logs for correlation | Should |
| FR-507 | The system SHALL return appropriate HTTP status codes for all error conditions | Must |
| FR-508 | The system SHALL return descriptive error messages in a consistent JSON format | Must |

---

## 5. API Endpoints

This section provides detailed API endpoint documentation including request/response formats and examples.

### 5.1 POST /schemas

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

### 5.2 GET /schemas

* Retrieves all registered schemas with optional filtering
* Query parameters (all optional):
  * `name`: Filter schemas by exact name match
  * `version`: Filter schemas by exact version match
* Returns JSON object with `schemas` array
* Filtering is performed at the database level for optimal performance
* Example: `GET /schemas?name=web-server-logs&version=1.0.0`

### 5.3 GET /schemas/{id}

* Retrieves a specific schema by its UUID
* Path parameter `id`: The UUID of the schema
* Returns HTTP 200 with schema object
* Returns HTTP 404 if schema not found

### 5.4 GET /schemas/{schema_name}/versions/{schema_version}

* Retrieves a specific schema by its combined and name and version
* Path parameters:
  * `schema_name`: The name of the schema
  * `schema_version`: The specific version of the schema
* Returns HTTP 200 with schema object
* Returns HTTP 404 if schema not found

### 5.5 POST /logs

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

### 5.6 GET /logs

* Retrieves stored log entries with filtering capabilities

#### 5.6.1 GET /logs/schema/{schema_name}

* Get all logs for a specific schema by name (resolves to latest version)
* Path parameter `schema_name`: The name of the schema
* Query parameters (all optional):
  * `filters`: Full JSON object for exact-match filtering (encoding the special symbols is necessary)
  * `page`: Number of the page to retrieve (default: 1)
  * `limit`: Number of entries per page (default: 10)
  * `date_begin`: Lower bound for `created_at` filter (ISO 8601 format)
  * `date_end`: Upper bound for `created_at` filter (ISO 8601 format)
* Example: `GET /logs/schema/temperature-readings?page=2&limit=10`
* Example: `GET /logs/schema/temperature-readings?date_begin=2025-01-01T10:00:00Z&date_end=2025-01-01T11:00:00Z`

#### 5.6.2 GET /logs/schema/{schema_name}/versions/{schema_version}

* Get all logs for a specific schema name and version
* Path parameters:
  * `schema_name`: The name of the schema
  * `schema_version`: The specific version (e.g., "1.0.0")
* Query parameters (all optional):
  * `filters`: Full JSON object for exact-match filtering (encoding the special symbols is necessary)
  * `page`: Number of the page to retrieve (default: 1)
  * `limit`: Number of entries per page (default: 10)
  * `date_begin`: Lower bound for `created_at` filter (ISO 8601 format)
  * `date_end`: Upper bound for `created_at` filter (ISO 8601 format)
* Example: `GET /logs/schema/temperature-readings/versions/1.0.0?page=2&limit=10`

#### 5.6.3 POST /logs/schema/{schema_name}/query

* Get all logs for a specific schema name (latest version) with complex query
* Accepts a JSON object representing query parameters
* Path parameter `schema_name`: The name of the schema
* Example request:

    ```json
    {
        "page": 1,
        "limit": 10,
        "date_begin": "2025-12-01T00:00:00Z",
        "date_end": "2025-12-01T23:59:59Z",
        "filters": {
            "level": "INFO"
        }
    }
    ```

#### 5.6.4 POST /logs/schema/{schema_name}/versions/{schema_version}/query

* Get all logs for a specific schema name and version with complex query
* Accepts a JSON object representing query parameters
* Path parameters:
  * `schema_name`: The name of the schema
  * `schema_version`: The specific version of the schema
* Example request:

    ```json
    {
        "page": 1,
        "limit": 10,
        "date_begin": "2025-12-01T00:00:00Z",
        "date_end": "2025-12-01T23:59:59Z",
        "filters": {
            "level": "INFO"
        }
    }
    ```

#### 5.6.5 Paginated Response Format

All log query endpoints return a paginated response:

```json
{
    "logs": [
        {
            "id": 123,
            "schema_id": "550e8400-e29b-41d4-a716-446655440000",
            "log_data": { ... },
            "created_at": "2025-10-23T10:00:01Z"
        }
    ],
    "timewindow": {
        "date_begin": "2025-12-01T00:00:00Z",
        "date_end": "2025-12-01T23:59:59Z"
    },
    "pagination": {
        "page": 1,
        "limit": 10,
        "total": 42,
        "total_pages": 5
    }
}
```

**Note:** The `timewindow` field is only included when date filters are applied.

#### 5.6.6 GET /logs/{id}

* Retrieve a specific log entry by its numeric ID
* Path parameter `id`: The log entry ID
* Returns HTTP 200 with log entry details
* Returns HTTP 404 if log not found

**Filtering:**
* JSONB field filtering uses PostgreSQL's `@>` containment operator
* Supports exact matching on top-level fields
* Multiple query parameters use AND logic
* All filtering performed at database level using GIN index

### 5.7 PUT /schemas/{id}

* Update an existing schema by UUID
* Path parameter `id`: The UUID of the schema to update
* Request body same as POST /schemas (name, version, description, schema_definition)
* Returns HTTP 200 with updated schema
* Returns HTTP 404 if schema not found

### 5.8 DELETE /schemas/{id}

* Delete a schema by UUID
* Path parameter `id`: The UUID of the schema to delete
* Query parameter `force`: Deletes the schema together with it's logs.
* Returns HTTP 409 when trying to delete a schema that haslogs without the `force` parameter
* Returns HTTP 204 (No Content) on success
* Returns HTTP 404 if schema not found
* Note: Consider cascade deletion or orphan log handling

### 5.9 DELETE /logs/{id}

* Delete a specific log entry by ID
* Path parameter `id`: The numeric ID of the log entry
* Returns HTTP 204 (No Content) on success
* Returns HTTP 404 if log not found
* Broadcasts deletion event to WebSocket clients

### 5.10 GET /ws/logs

* WebSocket endpoint for real-time log event streaming
* Clients receive notifications when logs are created or deleted
* Connection URL: `ws://localhost:8080/ws/logs`

**Event Types:**

Log Created Event:
```json
{
    "event": "log_created",
    "data": {
        "id": 123,
        "schema_id": "550e8400-e29b-41d4-a716-446655440000",
        "log_data": { ... },
        "created_at": "2025-10-23T10:00:01Z"
    }
}
```

Log Deleted Event:
```json
{
    "event": "log_deleted",
    "data": {
        "id": 123,
        "schema_id": "550e8400-e29b-41d4-a716-446655440000"
    }
}
```

### 5.11 GET /health

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

### 5.12 Request Tracking

All API endpoints support request tracking through the `X-Request-ID` header for distributed tracing and debugging.

**Request Header:**
* `X-Request-ID` (optional): Client-provided request identifier
  * Format: Any string value (UUID recommended)
  * If not provided, the server automatically generates a UUID v4

**Response Header:**
* `X-Request-ID`: Echoed or generated request identifier
  * Always present in all responses (success or error)
  * Same value as provided in request, or server-generated if not provided

**Benefits:**
* **Distributed Tracing**: Track requests across multiple services
* **Debugging**: Correlate client requests with server logs
* **Idempotency**: Identify duplicate or retry requests
* **Audit Trail**: Link requests to specific operations

**Example Usage:**

Request with client-provided ID:
```bash
curl -X POST http://localhost:8080/logs \
  -H "Content-Type: application/json" \
  -H "X-Request-ID: req-custom-12345" \
  -d '{"schema_id": "...", "log_data": {...}}'
```

Response includes the same ID:
```
HTTP/1.1 201 Created
X-Request-ID: req-custom-12345
Content-Type: application/json
...
```

Request without client-provided ID:
```bash
curl -X GET http://localhost:8080/schemas
```

Response includes server-generated UUID:
```
HTTP/1.1 200 OK
X-Request-ID: 550e8400-e29b-41d4-a716-446655440000
Content-Type: application/json
...
```

**Server Logging:**
* All server logs include the request_id for correlation
* Log format: `[request_id=<id>] <log message>`
* Enables quick filtering and debugging of specific requests

### 5.13 Error Handling

* HTTP 400: Invalid JSON, missing required fields, or invalid schema_id
* HTTP 422: Valid JSON but fails schema validation (for logs) or invalid JSON Schema (for schemas)
* HTTP 404: Schema not found for the provided schema_id
* HTTP 500: Internal server errors (database connectivity, etc.)
* All error responses include descriptive error messages and validation details
* All error responses include the `X-Request-ID` header for debugging

## 6. Non-Functional Requirements

### 6.1 Performance

* Handle at least 1000 requests per second under normal load
* Database queries should complete within 100ms for typical operations
* Memory usage should remain stable under continuous operation

### 6.2 Reliability

* Service should have 99.9% uptime during normal operations
* Graceful handling of database connection failures
* Proper error logging and recovery mechanisms

### 6.3 Security

* Input validation for all endpoints
* SQL injection prevention through parameterized queries
* Rate limiting to prevent abuse (configurable)

### 6.4 Scalability

* Stateless service design for horizontal scaling
* Database connection pooling for efficient resource usage
* Container-ready for orchestration platforms

### 6.5 Maintainability

* Clean, documented code following Rust best practices
* Comprehensive error handling and logging
* Configuration via environment variables

## 7. System Architecture

Components:

* `app`: Rust-based HTTP server handling requests and DB communication.
* `db`: PostgreSQL database container storing logs.
* `docker-compose.yml`: Defines both services, volumes and internal network.

### 7.1 Technology Stack

* **Backend**: Rust with Axum web framework
* **Database**: PostgreSQL 15+
* **Containerization**: Docker and Docker Compose
* **Serialization**: JSON with serde
* **Database Access**: SQLx for async PostgreSQL operations

### 7.2 Network Architecture

* Internal Docker network for app-database communication
* Exposed HTTP port (default: 8080) for external API access
* Database port not exposed externally for security

## 8. Data Models

### 8.1 Database Schema

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

### 8.2 API Response Models

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

## 9. API Specification

### 9.1 Base URL

* Development: `http://localhost:8080`
* Production: Configurable via environment variables

### 9.2 Content Types

* Request: `application/json`
* Response: `application/json`

### 9.3 Request Headers

**Standard Headers:**
* `Content-Type: application/json` (required for POST/PUT requests)
* `X-Request-ID: <request-id>` (optional, for request tracking)
  * If not provided, server generates a UUID v4
  * Returned in response headers for correlation

**Response Headers:**
* `Content-Type: application/json`
* `X-Request-ID: <request-id>` (always present)
  * Echoes client-provided value or contains server-generated UUID

### 9.4 Authentication

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

### 9.5 Rate Limiting

* Default: 1000 requests per minute per IP
* Configurable via environment variables
* Returns HTTP 429 when exceeded

### 9.6 Schema Validation

* All log entries must conform to a pre-registered schema
* JSON Schema Draft 7 specification is used for validation
* Schemas are versioned to support evolution over time
* Invalid log entries are rejected with detailed error messages

## 10. Version Information

* **Current Version**: 1.0.0
* **API Version**: v1
* **Database Schema Version**: 1.0
* **Compatibility**:
  * Rust 1.82+ (2021 edition)
  * PostgreSQL 16+
  * Docker 20.10+
  * Docker Compose 2.0+

### 10.1 Versioning Strategy

* Semantic versioning (MAJOR.MINOR.PATCH)
* API versioning through URL path (`/api/v1/`)
* Database migrations for schema changes
* Backward compatibility maintained within major versions

## 11. Future Improvements

### 11.1 Phase 2 Features

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
  * Structured logging for the service itself (currently includes request_id correlation)
  * Health check with detailed component status
  * Distributed tracing integration (OpenTelemetry)

### 11.2 Phase 3 Features

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

### 11.3 Operational Improvements

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
