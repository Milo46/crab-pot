# Log Server

**Log Server** is a centralized schema-on-write log sink. Right now, the application
covers very simple functionalities, e.g. creating schemas, log entries and then
retrieving them back to the user. It validates the data structure and makes sure
that data is consistent (every log has it's schema). It supports data transmission
via HTTP and in the future via WebSocket also.

## Quickstart

🎯 The current user workflow:

1. 📋 Create/Update log schemas
2. 📤 Push/Update logs continuously to the sink
3. 📥 Retrieve and analyze logs anytime
4. 🔄 Repeat the push/retrieve cycle as needed
5. 🗑️ Delete schemas or individual logs anytime

## Why Log Server?

- ✅ Schema Validation — Ensures data consistency across all logs
- ✅ Centralized — All your logs in one secure place
- ✅ Simple HTTP API — Easy to integrate with any system
- ✅ Data Integrity — Every log is validated against its schema

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

Only available interface is via the HTTP requests, e.g. `curl`.

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
  ]
}
```

## Features
## Configuration
## License
