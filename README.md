# Crab Pot

**Crab Pot** is a centralized schema-on-write log sink. Right now, the application
covers very simple functionalities, e.g. creating schemas, log entries and then
retrieving them back to the user. It validates the data structure and makes sure
that data is consistent (every log has it's schema). It supports data transmission
via HTTP and data events via WebSocket also.

## Quickstart

🎯 The current user workflow:

1. 📋 Create/Update log schemas
2. 📤 Push/Update logs continuously to the sink
3. 📥 Retrieve and analyze logs anytime
4. 🔄 Repeat the push/retrieve cycle as needed
5. 🗑️ Delete schemas or individual logs anytime

## Why Crab Pot?

- ✅ Schema Validation — Ensures data consistency across all logs
- ✅ Centralized — All your logs in one secure place
- ✅ Simple HTTP API — Easy to integrate with any system
- ✅ Data Integrity — Every log is validated against its schema
- ✅ Live data updates - Every log write/deletion is now being pushed via WebSocket
- ✅ Rate Limiting - Per API key custom rate limiting feature

## API Key Rate Limiting

Each API key has configurable rate limits:
- `rate_limit_per_second` - Base rate (default: 10 req/s)
- `rate_limit_burst` - Burst capacity (default: 2x base rate)

Rate limit headers are returned on all requests:
- `X-RateLimit-Limit` - Your burst capacity
- `X-RateLimit-Remaining` - Requests remaining
- `X-RateLimit-Reset` - Seconds until reset
- `Retry-After` - When rate limited (429 response)

## Prerequisites
- Docker and Docker Compose installed
- Basic understanding of JSON and HTTP requests

## Installation Guide

The project runs on top of `docker compose` and is necessary in order to run the software
in its production and development workflow.

To run the production workflow in the background, run this command:
```bash
docker compose -f docker-compose.yml up -d
```

## Usage Examples

There are two available interfaces:
- pure HTTP requests for writing and reading data
- WebSocket for getting live write updates on the data

### 1. Create your schema.
```bash
curl \
    --request POST \
    --location http://localhost:8080/schemas \
    --header "Content-Type: application/json" \
    --data '{
        "name": "temperature-readings",
        "version": "1.0.0",
        "description": "Logs for the temperature sensors inside my room",
        "schema_definition": {
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "reading": { "type": "number" }
            },
            "required": [ "name", "reading" ]
        }
    }'
```
Response:
```json
{
  "id": "891db49b-4d64-4ba0-b075-156c8c17ce1d",
  "name": "temperature-readings",
  "version": "1.0.0",
  "description": "Logs for the temperature sensors inside my room",
  "schema_definition": {
    "properties": {
      "name": {
        "type": "string"
      },
      "reading": {
        "type": "number"
      }
    },
    "required": [
      "name",
      "reading"
    ],
    "type": "object"
  },
  "created_at": "2025-11-20T20:52:14.548098+00:00",
  "updated_at": "2025-11-20T20:52:14.548098+00:00"
}
```

### 2. Save schema's UUID from the application response. It will be needed to POST logs.

### 3. Create your first log.
```bash
curl \
    --request POST \
    --location http://localhost:8080/logs \
    --header "Content-Type: application/json" \
    --data '{
        "schema_id": "891db49b-4d64-4ba0-b075-156c8c17ce1d",
        "log_data": {
            "name": "desk",
            "reading": 34
        }
    }'
```
Response:
```json
{
  "id": 10,
  "schema_id": "891db49b-4d64-4ba0-b075-156c8c17ce1d",
  "log_data": {
    "name": "desk",
    "reading": 34
  },
  "created_at": "2025-11-20T20:54:59.555233+00:00"
}
```

### 4. Retrieve all your logs.
```bash
curl \
    --request GET \
    --location http://localhost:8080/logs/schema/temperature-readings/1.0.0
```
Response:
```json
{
  "logs": [
    {
      "created_at": "2025-11-20T20:54:59.555233+00:00",
      "id": 10,
      "log_data": {
        "name": "desk",
        "reading": 34
      },
      "schema_id": "891db49b-4d64-4ba0-b075-156c8c17ce1d"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 50,
    "total_pages": 1,
    "total_logs": 1
  }
}
```

**Note**: All filtering is performed at the database level for optimal performance.
You can filter logs by providing query parameters such as pagination, sorting, or custom filters
to minimize data transfer and improve response times.

### 5. Filter logs with query parameters.

You can use query parameters to paginate, sort, and filter logs by date range:

```bash
# Pagination: Get page 2 with 10 logs per page
curl --request GET \
    --location "http://localhost:8080/logs/schema/temperature-readings/1.0.0?page=2&limit=10"

# Date filtering: Get logs from a specific time range
curl --request GET \
    --location "http://localhost:8080/logs/schema/temperature-readings/1.0.0?date_begin=2025-11-20T00:00:00Z&date_end=2025-11-21T00:00:00Z"

# Combined: Pagination + date filtering
curl --request GET \
    --location "http://localhost:8080/logs/schema/temperature-readings/1.0.0?page=1&limit=20&date_begin=2025-11-20T00:00:00Z&date_end=2025-11-21T00:00:00Z"
```

### 6. Query logs with custom filters using POST.

For more complex queries, use the POST endpoint to filter logs by their content:

```bash
curl \
    --request POST \
    --location http://localhost:8080/logs/schema/temperature-readings/1.0.0/query \
    --header "Content-Type: application/json" \
    --data '{
        "page": 1,
        "limit": 10,
        "date_begin": "2025-11-20T00:00:00Z",
        "date_end": "2025-11-21T00:00:00Z",
        "filters": {
            "name": "desk",
            "reading": 34
        }
    }'
```

The `filters` object allows you to search for logs where specific fields match the provided values. This example returns logs where `name` equals "desk" AND `reading` equals 34.

**Available query parameters:**
- `page` — Page number (default: 1)
- `limit` — Logs per page (default: 50)
- `date_begin` — Start date in ISO 8601 format
- `date_end` — End date in ISO 8601 format
- `filters` — Object with field-value pairs to match (POST only)

## Listening to events via WebSocket

In order to get live updates on the logs, you have to somehow get
notified by the server and you can achieve it by connecting to the
WebSocket endpoint of the application.

```bash
# If you want to listen to all logs
websocat "ws://localhost:8081/ws/logs"

# And if you want to listen to only a specific schema
websocat "ws://localhost:8081/ws/logs?schema_id=0a9dadf1-fd1b-4727-88d5-98aad5ce70a3"
```

**Note**: If you provide an invalid or non-existent `schema_id`,
the WebSocket connection will fail with a `404 Not Found` error:
```bash
websocat: WebSocketError: Received unexpected status code (404 Not Found)
```

Make sure the schema exists before attempting to connect.

The following events are currently supported:

### 1. Log creation message
```json
{
    "event_type": "created",
    "id": 5826,
    "schema_id": "0a9dadf1-fd1b-4727-88d5-98aad5ce70a3",
    "log_data": {
        "message":"Hello World from the working WebSocket connection!"
    },
    "created_at": "2025-12-05T11:13:36.361797+00:00"
}
```

### 2. Log deletion message
```json
{
    "event_type": "deleted",
    "id": 5826,
    "schema_id": "0a9dadf1-fd1b-4727-88d5-98aad5ce70a3"
}
```

## Features
## Configuration
## License
