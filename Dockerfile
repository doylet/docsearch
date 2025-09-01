# Multi-stage Dockerfile for docsearch production deployment
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

# Set working directory
WORKDIR /app

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates/*/Cargo.toml ./crates/*/
COPY services/*/Cargo.toml ./services/*/
COPY test/*/Cargo.toml ./test/*/

# Create dummy source files to build dependencies
RUN find crates services test -name "Cargo.toml" -exec dirname {} \; | xargs -I {} mkdir -p {}/src
RUN find crates services test -name "Cargo.toml" -exec dirname {} \; | xargs -I {} touch {}/src/lib.rs
RUN find services -name "Cargo.toml" -exec dirname {} \; | xargs -I {} touch {}/src/main.rs

# Build dependencies only
RUN cargo build --release --workspace
RUN rm -rf crates/*/src services/*/src test/*/src

# Copy actual source code
COPY . .

# Build the application
RUN cargo build --release --bin doc-indexer

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

# Copy configuration templates
COPY docker/config/ ./config/

# Create directories and set permissions
RUN mkdir -p /app/data /app/logs \
    && chown -R appuser:appuser /app \
    && chmod +x /usr/local/bin/doc-indexer

# Switch to non-root user
USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Expose ports
EXPOSE 8080 8081

# Set environment
ENV RUST_LOG=info
ENV DOCSEARCH_CONFIG_PATH=/app/config/production.toml
ENV DOCSEARCH_DATA_PATH=/app/data
ENV DOCSEARCH_LOG_PATH=/app/logs

# Start the application
CMD ["doc-indexer", "--config", "/app/config/production.toml"]
