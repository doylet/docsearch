# Multi-stage Dockerfile for docsearch production deployment
# Uses cargo-chef for intelligent dependency caching
# Keep the Rust version in step with CI (stable) and on bookworm to match the runtime image's glibc.
FROM rust:1.99-slim-bookworm as chef

# Install cargo-chef
RUN cargo install cargo-chef --locked

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    build-essential \
    g++ \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Planner stage - analyze dependencies
FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage with cached dependencies
FROM chef as builder

# Copy the recipe from planner
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies only (cached layer)
RUN cargo chef cook --release --recipe-path recipe.json

# Copy actual source code
COPY . .

# Build the application
RUN cargo build --release --bin doc-indexer

# Download and verify the local embedding model (bge-small-en-v1.5, ~130 MB)
# so containers start without network access
RUN ./target/release/doc-indexer --fetch-model /app/models/bge-small-en-v1.5

# Runtime stage
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/doc-indexer /usr/local/bin/doc-indexer
COPY --from=builder /app/demo-content ./demo-content
COPY --from=builder /app/models ./models

# Copy configuration templates
COPY docker/config/ ./config/

# Create directories and set permissions
RUN mkdir -p /app/data /app/logs \
    && chown -R appuser:appuser /app \
    && chmod +x /usr/local/bin/doc-indexer

# Switch to non-root user
USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Expose the HTTP API port
EXPOSE 8080

# Set environment
ENV RUST_LOG=info
# Listen on all interfaces; the default (localhost) is unreachable from outside the container
ENV ZL_SERVER_HOST=0.0.0.0
ENV ZL_SERVER_PORT=8080
ENV DOCSEARCH_CONFIG_PATH=/app/config/production.toml
ENV DOCSEARCH_DATA_PATH=/app/data
ENV DOCSEARCH_LOG_PATH=/app/logs
# Model baked in at build time: verified on start, never downloaded
ENV ZL_EMBEDDING_LOCAL_MODEL_PATH=/app/models/bge-small-en-v1.5

# Start the application
CMD ["doc-indexer", "--config", "/app/config/production.toml"]
