# Crab Pot SRD (Software Requirements Document) - v1.0.0

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
  - [4.6 API Key Management](#46-api-key-management)
- [5. API Endpoints](#5-api-endpoints)
  - [5.1 Main API (Port 8080)](#51-main-api-port-8080)
    - [5.1.1 POST /schemas](#511-post-schemas)
    - [5.1.2 GET /schemas](#512-get-schemas)
    - [5.1.3 GET /schemas/{id}](#513-get-schemasid)
    - [5.1.4 GET /schemas/{schema_name}/versions/{schema_version}](#514-get-schemasschema_nameversionsschema_version)
    - [5.1.5 PUT /schemas/{id}](#515-put-schemasid)
    - [5.1.6 DELETE /schemas/{id}](#516-delete-schemasid)
    - [5.1.7 POST /logs](#517-post-logs)
    - [5.1.8 GET /logs/schema/{schema_name}](#518-get-logsschemaschema_name)
    - [5.1.9 GET /logs/schema/{schema_name}/versions/{schema_version}](#519-get-logsschemaschema_nameversionsschema_version)
    - [5.1.10 POST /logs/schema/{schema_name}/query](#5110-post-logsschemaschema_namequery)
    - [5.1.11 POST /logs/schema/{schema_name}/versions/{schema_version}/query](#5111-post-logsschemaschema_nameversionsschema_versionquery)
    - [5.1.12 GET /logs/{id}](#5112-get-logsid)
    - [5.1.13 DELETE /logs/{id}](#5113-delete-logsid)
    - [5.1.14 GET /ws/logs](#5114-get-wslogs)
    - [5.1.15 GET /health](#5115-get-health)
  - [5.2 Admin API (Port 8081)](#52-admin-api-port-8081)
    - [5.2.1 POST /api-keys](#521-post-api-keys)
    - [5.2.2 GET /api-keys](#522-get-api-keys)
    - [5.2.3 GET /api-keys/{key_id}](#523-get-api-keyskey_id)
    - [5.2.4 DELETE /api-keys/{key_id}](#524-delete-api-keyskey_id)
    - [5.2.5 POST /api-keys/{key_id}/rotate](#525-post-api-keyskey_idrotate)
    - [5.2.6 GET /health](#526-get-health)
  - [5.3 Request Tracking](#53-request-tracking)
  - [5.4 Error Handling](#54-error-handling)
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
2. They register this schema with the Crab Pot
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

### 4.6 API Key Management

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-601 | The system SHALL provide a separate Admin API on a different port for security | Must |
| FR-602 | The system SHALL allow creation of API keys with name, description, expiration, and IP restrictions | Must |
| FR-603 | The system SHALL generate secure random API keys with SHA-256 hashing | Must |
| FR-604 | The system SHALL store only hashed API keys in the database | Must |
| FR-605 | The system SHALL return the plain-text API key only once upon creation | Must |
| FR-606 | The system SHALL support listing all API keys with metadata (excluding plain key) | Must |
| FR-607 | The system SHALL support retrieving individual API key details by ID | Must |
| FR-608 | The system SHALL support API key rotation (generate new key, invalidate old) | Must |
| FR-609 | The system SHALL support API key deletion | Must |
| FR-610 | The system SHALL validate API keys using Bearer token authentication | Must |
| FR-611 | The system SHALL check API key expiration before allowing requests | Must |
| FR-612 | The system SHALL enforce IP address restrictions when configured | Should |
| FR-613 | The system SHALL track API key usage (last_used_at, usage_count) | Should |
| FR-614 | The system SHALL enforce per-API-key rate limits | Must |
| FR-615 | The system SHALL return rate limit headers on all responses | Should |
| FR-616 | The system SHALL support configurable rate limits per API key | Should |
| FR-617 | The Admin API SHALL be bound to localhost by default for security | Must |
| FR-507 | The system SHALL return appropriate HTTP status codes for all error conditions | Must |
| FR-508 | The system SHALL return descriptive error messages in a consistent JSON format | Must |

---

## 5. API Endpoints

Crab Pot operates with **two separate HTTP servers** for enhanced security:

1. **Main API (Port 8080)** - Public-facing API for schemas, logs, and real-time events
   - Requires API key authentication (Bearer token)
   - Accessible from external networks
   - Handles all log and schema operations

2. **Admin API (Port 8081)** - Administrative interface for API key management
   - No authentication required (network-level security)
   - Bound to localhost (127.0.0.1) by default
   - Access via SSH tunnel or local machine only
   - Manages API key lifecycle

This architectural separation ensures that administrative operations are isolated from public-facing services and can be secured at the network level.

### 5.1 Main API (Port 8080)

All Main API endpoints require authentication via Bearer token in the `Authorization` header, except for `/health`.

#### 5.1.1 POST /schemas

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
        "service": "crab-pot",
        "timestamp": "2025-11-13T10:00:00Z"
    }
    ```

---

### 5.2 Admin API (Port 8081)

The Admin API is a **separate HTTP server** dedicated to API key management operations. For security:
- Bound to `127.0.0.1:8081` by default (localhost only)
- No authentication required (relies on network-level security)
- Access via SSH tunnel for remote administration
- Not exposed to public networks in production

**Environment Variables:**
- `ADMIN_API_ADDR` - Admin API bind address (default: `127.0.0.1:8081`)
- `MAIN_API_ADDR` - Main API bind address (default: `0.0.0.0:8080`)

**Security Best Practices:**
1. Keep Admin API bound to localhost
2. Use SSH tunneling for remote access: `ssh -L 8081:localhost:8081 user@server`
3. Use firewall rules to restrict access
4. Deploy behind VPN for team access
5. Never expose Admin API directly to the internet

#### 5.2.1 POST /api-keys

Creates a new API key for accessing the Main API.

**Request:**
```json
{
  "name": "Production API Key",
  "description": "Key for production services",
  "expires_at": "2026-12-31T23:59:59Z",
  "allowed_ips": "192.168.1.0/24,10.0.0.5"
}
```

**Request Fields:**
* `name` (required, string): Descriptive name for the API key
* `description` (optional, string): Additional details about key usage
* `expires_at` (optional, string): ISO 8601 timestamp for key expiration
* `allowed_ips` (optional, string): Comma-separated list of CIDR blocks or IP addresses

**Response (201 Created):**
```json
{
  "id": 1,
  "key": "sk_abc123def456...",
  "key_prefix": "sk_abc123...",
  "name": "Production API Key",
  "created_at": "2026-01-02T10:00:00Z",
  "expires_at": "2026-12-31T23:59:59Z"
}
```

**⚠️ Important:** The `key` field is shown **only once**. Save it immediately - it cannot be retrieved again.

**Example:**
```bash
curl -X POST http://127.0.0.1:8081/api-keys \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Development Key",
    "description": "For local testing"
  }'
```

#### 5.2.2 GET /api-keys

Lists all API keys with their metadata (plain keys are never returned).

**Response (200 OK):**
```json
{
  "api_keys": [
    {
      "id": 1,
      "key_prefix": "sk_abc123...",
      "name": "Production API Key",
      "description": "Key for production services",
      "created_at": "2026-01-02T10:00:00Z",
      "last_used_at": "2026-01-02T15:30:00Z",
      "expires_at": "2026-12-31T23:59:59Z",
      "is_active": true,
      "allowed_ips": ["192.168.1.0/24", "10.0.0.5"],
      "usage_count": 1250
    }
  ]
}
```

**Response Fields:**
* `key_prefix`: First 13 characters of the key (for identification)
* `last_used_at`: Timestamp of last successful authentication
* `usage_count`: Number of times the key has been used
* `is_active`: Whether the key is currently active

**Example:**
```bash
curl http://127.0.0.1:8081/api-keys
```

#### 5.2.3 GET /api-keys/{key_id}

Retrieves details of a specific API key by its ID.

**Path Parameters:**
* `key_id` (integer): The API key ID

**Response (200 OK):**
```json
{
  "id": 1,
  "key_prefix": "sk_abc123...",
  "name": "Production API Key",
  "description": "Key for production services",
  "created_at": "2026-01-02T10:00:00Z",
  "last_used_at": "2026-01-02T15:30:00Z",
  "expires_at": "2026-12-31T23:59:59Z",
  "is_active": true,
  "allowed_ips": ["192.168.1.0/24"],
  "usage_count": 1250
}
```

**Error Responses:**
* `404 Not Found`: API key with the specified ID does not exist

**Example:**
```bash
curl http://127.0.0.1:8081/api-keys/1
```

#### 5.2.4 DELETE /api-keys/{key_id}

Permanently deletes an API key. The key will immediately become invalid for authentication.

**Path Parameters:**
* `key_id` (integer): The API key ID to delete

**Response (204 No Content):**
No response body.

**Error Responses:**
* `404 Not Found`: API key with the specified ID does not exist

**Example:**
```bash
curl -X DELETE http://127.0.0.1:8081/api-keys/1
```

**⚠️ Warning:** This operation is irreversible. All services using this key will immediately lose access.

#### 5.2.5 POST /api-keys/{key_id}/rotate

Rotates an API key by generating a new key and invalidating the old one. All metadata (name, description, expiration, IP restrictions) is preserved.

**Path Parameters:**
* `key_id` (integer): The API key ID to rotate

**Response (200 OK):**
```json
{
  "id": 1,
  "key": "sk_new_xyz789...",
  "key_prefix": "sk_new_xyz...",
  "name": "Production API Key",
  "created_at": "2026-01-02T10:00:00Z",
  "expires_at": "2026-12-31T23:59:59Z"
}
```

**⚠️ Important:** 
- The new `key` is shown **only once**
- The old key becomes **immediately invalid**
- Update all services to use the new key before the rotation completes

**Use Cases:**
* Regular security key rotation (e.g., every 90 days)
* After suspected key compromise
* When migrating services to new credentials

**Example:**
```bash
curl -X POST http://127.0.0.1:8081/api-keys/1/rotate
```

**Error Responses:**
* `404 Not Found`: API key with the specified ID does not exist

#### 5.2.6 GET /health

Admin API health check endpoint.

**Response (200 OK):**
```json
{
  "status": "healthy",
  "service": "crab-pot-admin",
  "timestamp": "2026-01-02T10:00:00Z"
}
```

Also available at `GET /` (root path).

**Example:**
```bash
curl http://127.0.0.1:8081/health
```

---

### 5.3 Request Tracking

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

### 5.4 Error Handling

* HTTP 400: Invalid JSON, missing required fields, or invalid schema_id
* HTTP 401: Missing or invalid API key (Main API only)
* HTTP 403: Valid API key but access forbidden (expired, IP restriction, inactive)
* HTTP 404: Resource not found (schema, log, or API key)
* HTTP 422: Valid JSON but fails schema validation (for logs) or invalid JSON Schema (for schemas)
* HTTP 500: Internal server errors (database connectivity, etc.)
* All error responses include descriptive error messages and validation details
* All error responses include the `X-Request-ID` header for debugging

**Error Response Format:**
```json
{
  "error": "ERROR_CODE",
  "message": "Human-readable error description",
  "field_errors": {
    "field_name": ["validation error details"]
  },
  "request_id": "uuid-v4-or-client-provided"
}
```

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

### 7.1 Dual-Server Architecture

Crab Pot implements a **security-first architecture** with two separate HTTP servers:

```
┌─────────────────────────────────────────────┐
│          Main API (Port 8080)               │
│  ┌──────────────────────────────────────┐   │
│  │  Public Routes (No Auth Required)    │   │
│  │  • GET  /health                      │   │
│  │  • GET  /                            │   │
│  └──────────────────────────────────────┘   │
│  ┌──────────────────────────────────────┐   │
│  │  Protected Routes (API Key Auth)     │   │
│  │  • All /schemas/* endpoints          │   │
│  │  • All /logs/* endpoints             │   │
│  │  • GET /ws/logs (WebSocket)          │   │
│  └──────────────────────────────────────┘   │
│                                             │
│  Binding: 0.0.0.0:8080 (public-facing)      │
│  Auth: Bearer token (API key)               │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│         Admin API (Port 8081)               │
│  ┌──────────────────────────────────────┐   │
│  │  API Key Management (No App Auth)    │   │
│  │  • POST   /api-keys                  │   │
│  │  • GET    /api-keys                  │   │
│  │  • GET    /api-keys/{id}             │   │
│  │  • DELETE /api-keys/{id}             │   │
│  │  • POST   /api-keys/{id}/rotate      │   │
│  │  • GET    /health                    │   │
│  └──────────────────────────────────────┘   │
│                                             │
│  Binding: 127.0.0.1:8081 (localhost only)   │
│  Auth: Network-level security (SSH, VPN)    │
└─────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────┐
│      PostgreSQL Database (Port 5432)        │
│  • schemas table (UUID, JSONB)              │
│  • logs table (schema_id FK, JSONB)         │
│  • api_keys table (hash, metadata)          │
└─────────────────────────────────────────────┘
```

**Key Architectural Decisions:**

1. **Separation of Concerns**: Administrative functions isolated from public API
2. **Defense in Depth**: Admin API secured at network level, Main API at application level
3. **Least Privilege**: Admin API not accessible from external networks by default
4. **Stateless Design**: Both APIs are stateless for horizontal scaling

### 7.2 Technology Stack

* **Backend**: Rust 1.82+ with Axum 0.8 web framework
* **Database**: PostgreSQL 16+ with JSONB support
* **Containerization**: Docker and Docker Compose
* **Serialization**: JSON with serde
* **Database Access**: SQLx for async PostgreSQL operations with compile-time verification
* **Cryptography**: SHA-256 for API key hashing
* **WebSocket**: Native Axum WebSocket support for real-time events

### 7.3 Network Architecture

**Production Deployment:**
* Main API: Exposed on public network (e.g., `0.0.0.0:8080`)
* Admin API: Bound to localhost (`127.0.0.1:8081`)
* Database: Internal Docker network only, not exposed externally
* Access Admin API via:
  - SSH tunnel: `ssh -L 8081:localhost:8081 user@server`
  - VPN connection
  - Jump host/bastion server

**Development Setup:**
* Main API: `localhost:8080`
* Admin API: `localhost:8081` (exposed for convenience)
* Database: `localhost:5432` (exposed for direct access)

### 7.4 Security Model

**Main API Security:**
* API key authentication via `Authorization: Bearer` header
* Key validation against database (hashed comparison)
* Automatic expiration checking
* Optional IP address restriction enforcement
* Usage tracking for audit purposes

**Admin API Security:**
* Network isolation (localhost binding)
* No application-level authentication (admins manage keys)
* Access control via:
  - SSH key authentication
  - VPN membership
  - Firewall rules
  - Network policies (Kubernetes)

**Database Security:**
* Only hashed API keys stored (SHA-256)
* Parameterized queries prevent SQL injection
* Connection pooling with secure credentials
* Internal network communication only

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

-- Table for storing API keys
CREATE TABLE api_keys (
    id SERIAL PRIMARY KEY,
    key_hash VARCHAR(64) NOT NULL UNIQUE,
    key_prefix VARCHAR(20),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    allowed_ips INET[],
    usage_count BIGINT DEFAULT 0
);

-- Indexes for performance
CREATE INDEX idx_logs_schema_id ON logs(schema_id);
CREATE INDEX idx_logs_created_at ON logs(created_at);
CREATE INDEX idx_schemas_name ON schemas(name);
CREATE INDEX idx_schemas_name_version ON schemas(name, version);

-- GIN index for JSON queries on log data
CREATE INDEX idx_logs_data_gin ON logs USING GIN (log_data);

-- Indexes for API key operations
CREATE UNIQUE INDEX idx_api_keys_key_hash ON api_keys(key_hash);
CREATE INDEX idx_api_keys_is_active ON api_keys(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_api_keys_expires_at ON api_keys(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_api_keys_last_used_at ON api_keys(last_used_at);
```

**Table Relationships:**
* `logs.schema_id` → `schemas.id` (foreign key, enforces referential integrity)
* `api_keys` is independent (no foreign keys to other tables)

**Key Design Decisions:**
* **schemas**: UUID primary key for globally unique identifiers
* **logs**: SERIAL for auto-increment, efficient integer-based lookups
* **api_keys**: SERIAL for simple integer IDs, hash for security
* **JSONB**: Flexible schema definition and log data storage with indexing support
* **INET[]**: Array type for storing multiple IP addresses/CIDR blocks

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

**API Key Response (List/Get):**

```json
{
    "id": 1,
    "key_prefix": "lgs_prod_abc",
    "name": "Production App Key",
    "description": "API key for production application server",
    "created_at": "2025-10-23T09:00:00Z",
    "last_used_at": "2025-10-23T10:30:00Z",
    "expires_at": "2026-10-23T09:00:00Z",
    "is_active": true,
    "allowed_ips": ["192.168.1.100", "10.0.0.0/24"],
    "usage_count": 1542
}
```

**API Key Creation Response:**

```json
{
    "id": 1,
    "key_prefix": "lgs_prod_abc",
    "name": "Production App Key",
    "description": "API key for production application server",
    "created_at": "2025-10-23T09:00:00Z",
    "expires_at": "2026-10-23T09:00:00Z",
    "is_active": true,
    "allowed_ips": ["192.168.1.100", "10.0.0.0/24"],
    "api_key": "lgs_prod_abc_def123456789..."
}
```

**Note:** The `api_key` field (full plaintext key) is ONLY returned on creation or rotation. Store it securely—it cannot be retrieved again.

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

**Main API (Port 8080) Headers:**
* `Content-Type: application/json` (required for POST/PUT requests)
* `Authorization: Bearer <api-key>` (required for all endpoints except `/health`)
* `X-Request-ID: <request-id>` (optional, for request tracking)
  * If not provided, server generates a UUID v4
  * Returned in response headers for correlation

**Admin API (Port 8081) Headers:**
* `Content-Type: application/json` (required for POST requests)
* `X-Request-ID: <request-id>` (optional, for request tracking)
* **No authentication headers required** (secured at network level)

**Response Headers:**
* `Content-Type: application/json`
* `X-Request-ID: <request-id>` (always present)
  * Echoes client-provided value or contains server-generated UUID

### 9.4 Authentication

**Current Implementation (v1.2.0+)**: API Key Authentication with Dual-Server Architecture

#### Main API (Port 8080) - API Key Required
* **Authentication Method**: Bearer token authentication
* **Header Format**: `Authorization: Bearer sk_<your_api_key>`
* **Protected Endpoints**: All schema and log operations, WebSocket
* **Public Endpoints**: `/health`, `/` (root)
* **Key Features**:
  - SHA-256 hashed keys stored in database
  - Optional expiration date enforcement
  - IP address restriction support (CIDR blocks)
  - Usage tracking (last_used_at, usage_count)
  - Automatic expiration checking

**Example Authenticated Request:**
```bash
curl -X POST http://localhost:8080/schemas \
  -H "Authorization: Bearer sk_abc123def456..." \
  -H "Content-Type: application/json" \
  -d '{"name": "test-schema", "version": "1.0.0", ...}'
```

#### Admin API (Port 8081) - Network-Level Security
* **Authentication Method**: Network isolation (no application-level auth)
* **Default Binding**: `127.0.0.1:8081` (localhost only)
* **Access Methods**:
  - Local machine access
  - SSH tunnel: `ssh -L 8081:localhost:8081 user@server`
  - VPN for team access
  - Reverse proxy with additional authentication
* **Security Model**: 
  - Administrative operations isolated on separate port
  - Not exposed to public internet
  - Relies on network-level access control
  - No API keys needed (admin manages the keys themselves)

**Example SSH Tunnel Access:**
```bash
# Create SSH tunnel
ssh -L 8081:localhost:8081 user@production-server

# In another terminal, access admin API
curl http://localhost:8081/api-keys
```

#### API Key Lifecycle
1. **Creation**: Admin creates key via Admin API (`POST /api-keys`)
2. **Distribution**: Plain key shown once, must be saved by recipient
3. **Usage**: Key used for Main API authentication
4. **Monitoring**: Track usage via Admin API (`GET /api-keys`)
5. **Rotation**: Generate new key, invalidate old (`POST /api-keys/{id}/rotate`)
6. **Deletion**: Permanently remove key (`DELETE /api-keys/{id}`)

#### Security Features
* **Hashing**: Only SHA-256 hashes stored in database
* **One-time Display**: Plain keys shown only on creation/rotation
* **Expiration**: Optional automatic expiration enforcement
* **IP Restrictions**: Limit key usage to specific IP addresses/CIDR blocks
* **Revocation**: Keys can be marked inactive without deletion
* **Audit Trail**: Track creation time, last usage, usage count

**Environment Variables:**
* `MAIN_API_ADDR`: Main API bind address (default: `0.0.0.0:8080`)
* `ADMIN_API_ADDR`: Admin API bind address (default: `127.0.0.1:8081`)
* `DATABASE_URL`: PostgreSQL connection string

**Previous Versions:**
* **v1.0.0**: No authentication - all endpoints publicly accessible
  * Suitable only for development and trusted internal networks
* **v1.1.0**: Single-key environment variable authentication (deprecated)

**Future Enhancements (v2.0.0+)**:
* Role-based access control (RBAC) for API keys
* JWT-based authentication for user sessions
* Multi-tenant support with organization isolation
* OAuth2/OIDC integration

### 9.5 Rate Limiting

* Default: 1000 requests per minute per IP
* Configurable via environment variables
* Returns HTTP 429 when exceeded
* Applied independently to Main API and Admin API

### 9.6 Schema Validation

* All log entries must conform to a pre-registered schema
* JSON Schema Draft 7 specification is used for validation
* Schemas are versioned to support evolution over time
* Invalid log entries are rejected with detailed error messages

## 10. Version Information

* **Current Version**: 0.9.0 (Pre-release)
* **API Version**: v1
* **Database Schema Version**: 1.1 (includes api_keys table)
* **Release Date**: January 2026
* **Status**: Development/Testing phase (not yet production-ready)
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

* **Advanced Authentication & Authorization**
  * ~~JWT-based authentication~~ *Deferred - API key authentication currently sufficient*
  * Role-based access control (RBAC)
  * ~~API key management~~ ✅ **Implemented in v1.2.0**
  * Multi-tenant API key scoping
  * Granular permission system

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
