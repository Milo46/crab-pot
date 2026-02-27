#=========================================================================
# BUILD STAGE
#=========================================================================

FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY src ./src
COPY benches ./benches

RUN cargo build --release

#=========================================================================
# RUNTIME STAGE
#=========================================================================

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false appuser

WORKDIR /app

COPY --from=builder /app/target/release/crab-pot ./

RUN chown appuser:appuser crab-pot

USER appuser

EXPOSE 8080

CMD ["./crab-pot"]
