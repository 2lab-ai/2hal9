# Multi-stage build for HAL9 project
FROM rust:slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY substrate/ substrate/
COPY layers/ layers/
COPY demo/ demo/

# Build in release mode
RUN cargo build --release --workspace

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 hal9

# Copy binaries from builder
COPY --from=builder /app/target/release/hal9-server /usr/local/bin/
COPY --from=builder /app/target/release/hal9 /usr/local/bin/

# Copy demo files
COPY --from=builder /app/demo /home/hal9/demo

# Set ownership
RUN chown -R hal9:hal9 /home/hal9

# Switch to non-root user
USER hal9
WORKDIR /home/hal9

# Expose ports
EXPOSE 8080 9090

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=40s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Default command
CMD ["hal9-server"]