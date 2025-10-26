# Log Server

A lightweight HTTP-based **schema-driven log sink** service that allows users to define custom log schemas and then receive JSON log entries validated against those schemas, storing them in a persistent database, deployable with Docker Compose.

## 🚀 Features

- **Schema-driven logging**: Define custom JSON schemas for different log types
- **JSON Schema validation**: All log entries validated against JSON Schema Draft 7
- **PostgreSQL storage**: Efficient JSONB storage with indexing
- **RESTful API**: Clean HTTP API with comprehensive OpenAPI documentation
- **Docker deployment**: Ready-to-deploy with Docker Compose
- **Schema versioning**: Support for schema evolution with semantic versioning

## 📋 Quick Start

### 1. Register a Log Schema

```bash
curl -X POST http://localhost:8080/schemas \
  -H "Content-Type: application/json" \
  -d '{
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
    }
  }'
```

### 2. Send Log Entries

```bash
curl -X POST http://localhost:8080/logs/web-server-logs \
  -H "Content-Type: application/json" \
  -d '{
    "timestamp": "2025-10-26T10:00:00Z",
    "level": "INFO",
    "message": "User login successful",
    "request_id": "req-12345"
  }'
```

### 3. Retrieve Logs

```bash
# Get all logs
curl "http://localhost:8080/logs"

# Get logs for a specific schema
curl "http://localhost:8080/logs?schema_id=web-server-logs"
```

## 🏗️ Architecture

```
┌─────────────────┐    HTTP/JSON    ┌──────────────────┐
│   Client Apps   │ ──────────────► │   Log Server     │
│                 │                 │   (Rust/Axum)    │
└─────────────────┘                 └──────────────────┘
                                             │
                                             │ SQL
                                             ▼
                                    ┌──────────────────┐
                                    │   PostgreSQL     │
                                    │   Database       │
                                    └──────────────────┘
```

### Components

- **Log Server**: Rust-based HTTP server using Axum framework
- **Database**: PostgreSQL with JSONB support for flexible log storage
- **Validation**: JSON Schema Draft 7 validation for all log entries
- **API**: RESTful endpoints with comprehensive error handling

## 📚 API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST   | `/schemas` | Register a new log schema |
| GET    | `/schemas` | Retrieve registered schemas |
| GET    | `/schemas/{id}` | Retrieve a specific schema |
| POST   | `/logs/{schema_id}` | Create a new log entry (validated against schema) |
| GET    | `/logs` | Retrieve log entries with filtering |
| GET    | `/health` | Health check endpoint |

## 🚀 Deployment

### Using Docker Compose

1. Clone the repository
2. Run with Docker Compose:

```bash
docker-compose up -d
```

This will start:
- Log server on port 8080
- PostgreSQL database with persistent storage

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://user:pass@localhost/logs` |
| `SERVER_PORT` | HTTP server port | `8080` |
| `LOG_LEVEL` | Application log level | `info` |

## 📖 Documentation

- **[Software Requirements Document](docs/SRD.md)** - Complete technical specification
- **[API Documentation](docs/openapi.yaml)** - OpenAPI 3.0 specification
- **[Documentation Guide](docs/README.md)** - How to use and maintain the docs

### Interactive API Documentation

View the API documentation:
1. Upload `docs/openapi.yaml` to [Swagger Editor](https://editor.swagger.io/)
2. Or run locally: `npx swagger-ui-serve docs/openapi.yaml`

## 🛠️ Development

### Prerequisites

- Rust 1.70+
- PostgreSQL 15+
- Docker & Docker Compose (for deployment)

### Local Development

1. Set up the database:
```bash
# Start PostgreSQL with Docker
docker run -d --name logs-db -p 5432:5432 -e POSTGRES_PASSWORD=password postgres:15
```

2. Run the server:
```bash
cargo run
```

3. Run tests:
```bash
cargo test
```

## 🎯 Use Cases

### Development Environment Logging
- Multiple services logging to a centralized endpoint
- Different log schemas for different service types
- Easy log aggregation and filtering

### Microservices Architecture
- Each service defines its own log schema
- Centralized log storage with schema validation
- Consistent log structure across services

### Application Monitoring
- Structured error tracking with custom schemas
- Performance metrics logging
- User activity tracking

## 📋 Schema Examples

### Web Server Logs
```json
{
  "name": "web-server-logs",
  "version": "1.0.0",
  "schema": {
    "type": "object",
    "required": ["timestamp", "method", "path", "status", "response_time"],
    "properties": {
      "timestamp": {"type": "string", "format": "date-time"},
      "method": {"type": "string", "enum": ["GET", "POST", "PUT", "DELETE"]},
      "path": {"type": "string"},
      "status": {"type": "integer", "minimum": 100, "maximum": 599},
      "response_time": {"type": "number", "minimum": 0},
      "user_id": {"type": "string"}
    }
  }
}
```

### Error Tracking
```json
{
  "name": "error-tracking",
  "version": "1.0.0",
  "schema": {
    "type": "object",
    "required": ["timestamp", "error_type", "message", "stack_trace"],
    "properties": {
      "timestamp": {"type": "string", "format": "date-time"},
      "error_type": {"type": "string"},
      "message": {"type": "string"},
      "stack_trace": {"type": "string"},
      "context": {"type": "object"}
    }
  }
}
```

## 🔧 Technology Stack

- **Backend**: Rust with Axum web framework
- **Database**: PostgreSQL 15+ with JSONB support
- **Validation**: JSON Schema Draft 7
- **Deployment**: Docker & Docker Compose
- **API Documentation**: OpenAPI 3.0

## 📈 Roadmap

- [ ] Authentication & authorization (JWT, API keys)
- [ ] Advanced querying (full-text search, aggregations)
- [ ] Real-time log streaming (WebSocket support)
- [ ] Log retention policies and archiving
- [ ] Prometheus metrics endpoint
- [ ] Kubernetes deployment manifests

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙋‍♂️ Support

- Check the [documentation](docs/) for detailed information
- Review the [OpenAPI specification](docs/openapi.yaml) for API details
- Open an issue for bug reports or feature requests
