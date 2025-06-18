# HAL9 Production Docker Image

# Build stage
FROM rust:1.75-slim AS builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace manifests
COPY Cargo.toml ./
COPY substrate/tooling/rust/workspace.toml ./substrate/tooling/rust/
COPY substrate/tooling/rust/workspace.lock ./substrate/tooling/rust/

# Copy all member Cargo.toml files
COPY layers/L2_implementation/neurons/game_neurons/Cargo.toml ./layers/L2_implementation/neurons/game_neurons/
COPY layers/L2_implementation/neurons/agent_dropout/Cargo.toml ./layers/L2_implementation/neurons/agent_dropout/
COPY substrate/tooling/mcp/ha-prompter/Cargo.toml ./substrate/tooling/mcp/ha-prompter/
COPY layers/L8_visionary/exploration/gentle_singularity/Cargo.toml ./layers/L8_visionary/exploration/gentle_singularity/

# Create dummy source files to cache dependencies
RUN mkdir -p \
    layers/L2_implementation/neurons/game_neurons/src \
    layers/L2_implementation/neurons/agent_dropout/src \
    substrate/tooling/mcp/ha-prompter/src \
    layers/L8_visionary/exploration/gentle_singularity/src \
    layers/L3_operational/architecture/server/src && \
    echo "fn main() {}" > layers/L2_implementation/neurons/game_neurons/src/main.rs && \
    echo "fn main() {}" > layers/L2_implementation/neurons/agent_dropout/src/main.rs && \
    echo "fn main() {}" > substrate/tooling/mcp/ha-prompter/src/main.rs && \
    echo "fn main() {}" > layers/L8_visionary/exploration/gentle_singularity/src/main.rs && \
    echo "fn main() {}" > layers/L3_operational/architecture/server/src/main.rs

# Build dependencies
RUN cargo build --release 2>/dev/null || true

# Copy all source code
COPY layers/ ./layers/
COPY substrate/ ./substrate/
COPY sdk/ ./sdk/
COPY competitions/ ./competitions/
COPY demo/ ./demo/

# Build the actual server
# For now, we'll use the AI Genius Game as the main server since it's the most complete
WORKDIR /app/demo/ai-genius-game
RUN cargo build --release && \
    cp target/release/ai-genius-game /usr/local/bin/hal9-server

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

# Copy binary from builder
COPY --from=builder /usr/local/bin/hal9-server /usr/local/bin/

# Copy static assets
COPY --from=builder /app/demo/ai-genius-game/static /app/static
COPY --from=builder /app/demo/consciousness-visualization /app/consciousness-visualization
COPY --from=builder /app/demo/self-organization-dashboard /app/self-organization-dashboard

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
    CMD curl -f http://localhost:3456/api/games || exit 1

# Expose ports
EXPOSE 3456

# Mount points
VOLUME ["/app/config", "/app/logs", "/app/data"]

# Default command
CMD ["hal9-server"]