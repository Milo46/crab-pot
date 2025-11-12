# HTTP Logging Examples

## Basic Setup (Current)
```rust
.layer(TraceLayer::new_for_http())
```

## Custom HTTP Logging
```rust
use tower_http::trace::TraceLayer;
use tracing::Level;

.layer(
    TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            tracing::info_span!(
                "http_request",
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
            )
        })
        .on_request(|_request: &Request<_>, _span: &Span| {
            tracing::info!("started processing request")
        })
        .on_response(|_response: &Response, latency: Duration, _span: &Span| {
            tracing::info!("finished processing request latency={:?}", latency)
        })
)
```

## With Request/Response Body Logging (Development Only)
```rust
use tower_http::trace::TraceLayer;

.layer(
    TraceLayer::new_for_http()
        .on_body_chunk(|chunk: &Bytes, latency: Duration, _span: &Span| {
            tracing::debug!("sending body chunk size={} latency={:?}", chunk.len(), latency)
        })
        .on_eos(|_trailers: Option<&HeaderMap>, stream_duration: Duration, _span: &Span| {
            tracing::debug!("stream closed duration={:?}", stream_duration)
        })
)
```

## Environment Variables for Log Control

```bash
# Show only HTTP requests/responses
RUST_LOG=tower_http::trace=info

# Show app logs + HTTP
RUST_LOG=log_server=debug,tower_http::trace=info

# Show everything including SQL queries
RUST_LOG=debug

# Custom filter
RUST_LOG="log_server::handlers=debug,tower_http=info"
```

## Sample Output

```
2025-11-04T10:30:15.123Z  INFO tower_http::trace::on_request: started processing request
    at tower-http-0.5.0/src/trace/on_request.rs:147
    in HTTP{method=POST uri=/schemas version=HTTP/1.1}

2025-11-04T10:30:15.125Z DEBUG log_server::handlers::schema_handlers: creating new schema name="web-logs" version="1.0.0"
    at src/handlers/schema_handlers.rs:89
    in HTTP{method=POST uri=/schemas version=HTTP/1.1}

2025-11-04T10:30:15.127Z  INFO tower_http::trace::on_response: finished processing request latency=4ms status=201
    at tower-http-0.5.0/src/trace/on_response.rs:158
    in HTTP{method=POST uri=/schemas version=HTTP/1.1}
```
