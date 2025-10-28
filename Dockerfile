#=========================================================================
# BUILD STAGE
#=========================================================================

FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml ./

COPY src ./src

RUN cargo build --release

#=========================================================================
# RUNTIME STAGE
#=========================================================================

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false appuser

WORKDIR /app

COPY --from=builder /app/target/release/log-server ./

RUN chown appuser:appuser log-server

USER appuser

EXPOSE 8080

CMD ["./log-server"]
