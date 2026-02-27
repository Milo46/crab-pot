# Crab Pot

## Description

**Crab Pot** is a centralized log ingestion service with schema validation at write time. It uses JSON Schema to define log structures and validates every log entry against its schema before storage, ensuring data quality and consistency.

**Key features:**
- **Schema-first approach**: Define your log structure with JSON Schema
- **Automatic validation**: Reject invalid logs at ingestion time
- **RESTful API**: Standard HTTP for all CRUD operations
- **Real-time streaming**: WebSocket support for live log events
- **Cursor-based pagination**: Efficient querying of large log collections

## Architecture

The system consists of two components running in Docker:

**Main API** (`:8080`) - Production-facing service:
- Log ingestion and retrieval (schema-validated)
- Schema management (create, read, update, delete)
- WebSocket streaming for real-time log events
- Protected by API key authentication

**Admin API** (`:8081`) - Internal management:
- API key lifecycle (create, rotate, revoke)
- Key management and monitoring
- Should be network-isolated in production

**Database**: PostgreSQL stores schemas, logs, and API keys with full transactional support.

## Installation

### Prerequisites
- Docker and Docker Compose
- Ports 8080 and 8081 available

### Quick Start

Start all services with a single command:

```sh
docker compose up
```

The system will:
1. Pull required images (Postgres, Crab Pot)
2. Initialize the database with schema and seed data
3. Start the Main API on `http://localhost:8080`
4. Start the Admin API on `http://localhost:8081`

> **🔒 Security Note:** The Admin API is bound to all interfaces by default for development. In production, bind it to `127.0.0.1` and access via SSH tunnel or VPN.

### Development Mode

For hot-reload during development:
```sh
docker compose -f docker-compose.dev.yml up
```

## First-time Use

This guide walks you through the basic workflow: creating an API key, defining a schema, and ingesting logs.

### Generate an API Key

Create an API key via the admin API to authenticate your requests:

```sh
curl -X POST http://localhost:8081/api-keys \
  -H "Content-Type: application/json" \
  -d '{"name": "my-api-key"}'
```

**Response:**
```json
{
  "id": 1,
  "key": "sk_KPceDoZWl1--DlXGNjqJS3IZQKbUubcKhVhcUHDEcyo",
  "key_prefix": "sk_KPceDoZ...",
  "name": "my-api-key",
  "created_at": "2026-02-27T11:54:58.147643Z"
}
```

> **⚠️ Important:** Save the `key` value - it won't be shown again! Export it for convenience:
> ```sh
> export API_KEY="sk_KPceDoZWl1--DlXGNjqJS3IZQKbUubcKhVhcUHDEcyo"
> ```

### Define a Log Schema

Create a JSON Schema to validate incoming logs. Example: temperature measurements.

```sh
curl -X POST http://localhost:8080/schemas \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "temperature-measurements",
    "version": "1.0.0",
    "description": "Temperature sensor readings",
    "schema_definition": {
      "type": "object",
      "required": ["timestamp", "location", "temperature"],
      "properties": {
        "timestamp": {"type": "string", "format": "date-time"},
        "location": {"type": "string"},
        "temperature": {"type": "number"}
      }
    }
  }'
```

**Response:**
```json
{
  "id": "d5141a46-f180-4073-943f-16d6a73a9fb2",
  "name": "temperature-measurements",
  "version": "1.0.0",
  "created_at": "2026-02-27T12:14:11.305577+00:00"
}
```

> **📝 Tip:** Save the schema `id` for future use:
> ```sh
> export SCHEMA_ID="d5141a46-f180-4073-943f-16d6a73a9fb2"
> ```

### Ingest Logs

Submit log entries that conform to your schema:

```sh
curl -X POST http://localhost:8080/logs \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "schema_id": "'"$SCHEMA_ID"'",
    "log_data": {
      "timestamp": "2026-02-27T13:20:00+00:00",
      "location": "outside-window",
      "temperature": 20.0
    }
  }'
```

**Response:**
```json
{
  "id": 1,
  "schema_id": "d5141a46-f180-4073-943f-16d6a73a9fb2",
  "log_data": {
    "timestamp": "2026-02-27T13:20:00+00:00",
    "location": "outside-window",
    "temperature": 20.0
  },
  "created_at": "2026-02-27T12:27:48.435435+00:00"
}
```

### Query Logs

Retrieve logs by schema ID (with cursor-based pagination):

```sh
curl http://localhost:8080/logs/schemas/$SCHEMA_ID \
  -H "Authorization: Bearer $API_KEY"
```

**Response:**
```json
{
  "schema_id": "d5141a46-f180-4073-943f-16d6a73a9fb2",
  "logs": [
    {
      "id": 3,
      "log_data": {
        "timestamp": "2026-02-27T14:00:00+00:00",
        "location": "outside-window",
        "temperature": 23.5
      },
      "created_at": "2026-02-27T12:36:13.261374+00:00"
    },
    {
      "id": 2,
      "log_data": {
        "timestamp": "2026-02-27T13:40:00+00:00",
        "location": "outside-window",
        "temperature": 22.0
      },
      "created_at": "2026-02-27T12:36:02.585400+00:00"
    }
  ],
  "cursor": {
    "limit": 10,
    "next_cursor": null,
    "prev_cursor": 3,
    "has_more": false
  }
}
```

> **🚀 Quick Start:** Use the helper script to avoid typing API keys:
> ```sh
> ./scripts/api-curl.sh http://localhost:8080/schemas
> ```

## Features

_(Coming soon)_

## License

MIT License - See [LICENSE](LICENSE) for details.
