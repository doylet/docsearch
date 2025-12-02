# Multi-stage Dockerfile for docsearch production deployment
FROM rust:1.90-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    build-essential \
    g++ \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

# Set working directory
WORKDIR /app

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Copy all crate manifests (preserving directory structure)
COPY crates/zero-latency-api/Cargo.toml ./crates/zero-latency-api/
COPY crates/zero-latency-observability/Cargo.toml ./crates/zero-latency-observability/
COPY crates/zero-latency-contracts/Cargo.toml ./crates/zero-latency-contracts/
COPY crates/zero-latency-search/Cargo.toml ./crates/zero-latency-search/
COPY crates/zero-latency-vector/Cargo.toml ./crates/zero-latency-vector/
COPY crates/cli/Cargo.toml ./crates/cli/
COPY crates/zero-latency-core/Cargo.toml ./crates/zero-latency-core/
COPY crates/zero-latency-config/Cargo.toml ./crates/zero-latency-config/

# Copy service manifests
COPY services/doc-indexer/Cargo.toml ./services/doc-indexer/

# Create dummy source files to build dependencies
RUN mkdir -p crates/zero-latency-api/src && echo "fn main() {}" > crates/zero-latency-api/src/lib.rs
RUN mkdir -p crates/zero-latency-observability/src && echo "fn main() {}" > crates/zero-latency-observability/src/lib.rs
RUN mkdir -p crates/zero-latency-contracts/src && echo "fn main() {}" > crates/zero-latency-contracts/src/lib.rs
RUN mkdir -p crates/zero-latency-search/src && echo "fn main() {}" > crates/zero-latency-search/src/lib.rs
RUN mkdir -p crates/zero-latency-vector/src && echo "fn main() {}" > crates/zero-latency-vector/src/lib.rs
RUN mkdir -p crates/cli/src && echo "fn main() {}" > crates/cli/src/main.rs
RUN mkdir -p crates/zero-latency-core/src && echo "fn main() {}" > crates/zero-latency-core/src/lib.rs
RUN mkdir -p crates/zero-latency-config/src && echo "fn main() {}" > crates/zero-latency-config/src/lib.rs
RUN mkdir -p services/doc-indexer/src && echo "fn main() {}" > services/doc-indexer/src/lib.rs && echo "fn main() {}" > services/doc-indexer/src/main.rs

# Build dependencies only
RUN cargo build --release --workspace
RUN rm -rf crates/*/src services/*/src

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
