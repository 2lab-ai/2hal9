# Multi-stage build for Rust application
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev

# Create app directory
WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/lib.rs

# Build dependencies
RUN cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the application
RUN cargo build --release --bin genius_game_server

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    openssl

# Create non-root user
RUN addgroup -g 1000 genius && \
    adduser -D -u 1000 -G genius genius

# Copy binary from builder
COPY --from=builder /app/target/release/genius_game_server /usr/local/bin/genius_game_server

# Create data directory
RUN mkdir -p /app/data && \
    chown -R genius:genius /app

WORKDIR /app

# Switch to non-root user
USER genius

# Expose ports
EXPOSE 8080 8081

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV WS_PORT=8081

# Run the server
CMD ["genius_game_server"]