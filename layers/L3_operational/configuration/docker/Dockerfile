# 2HAL9 Production Docker Image

# Build stage
FROM rust:1.75-slim AS builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY 2hal9-core/Cargo.toml ./2hal9-core/
COPY 2hal9-server/Cargo.toml ./2hal9-server/
COPY 2hal9-cli/Cargo.toml ./2hal9-cli/

# Create dummy files to cache dependencies
RUN mkdir -p 2hal9-core/src 2hal9-server/src 2hal9-cli/src && \
    echo "fn main() {}" > 2hal9-server/src/main.rs && \
    echo "pub fn lib() {}" > 2hal9-core/src/lib.rs && \
    echo "fn main() {}" > 2hal9-cli/src/main.rs

# Build dependencies
RUN cargo build --release

# Copy source code
COPY 2hal9-core/src ./2hal9-core/src
COPY 2hal9-server/src ./2hal9-server/src
COPY 2hal9-cli/src ./2hal9-cli/src

# Remove dummy files and rebuild
RUN rm -rf target/release/deps/twohal9* && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash hal9

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /app/target/release/2hal9-server /usr/local/bin/
COPY --from=builder /app/target/release/2hal9 /usr/local/bin/

# Create directories
RUN mkdir -p /app/config /app/logs /app/data && \
    chown -R hal9:hal9 /app

USER hal9

# Environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV HAL9_CONFIG_PATH=/app/config/config.yaml

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD 2hal9 status || exit 1

# Expose ports
EXPOSE 8080 9090

# Mount points
VOLUME ["/app/config", "/app/logs", "/app/data"]

# Default command
CMD ["2hal9-server"]