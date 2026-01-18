# syntax=docker/dockerfile:1

# Ralph MCP Server - Multi-stage build for minimal image size
# This Dockerfile builds the Ralph MCP server for use with Docker MCP toolkit
#
# Build Arguments:
#   VERSION     - Semantic version for the build (e.g., "1.0.0")
#   COMMIT_SHA  - Git commit SHA for traceability
#
# Usage:
#   docker build --build-arg VERSION=1.0.0 --build-arg COMMIT_SHA=$(git rev-parse HEAD) -t ralph .
#
# These build args are embedded as labels for version tracking and debugging.

# Build arguments for version tracking
ARG VERSION=dev
ARG COMMIT_SHA=unknown

# Stage 1: Build the Rust binary
FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace files for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY cli/Cargo.toml ./cli/

# Create dummy source files to cache dependencies
RUN mkdir -p src cli/src && \
    echo 'fn main() {}' > src/main.rs && \
    echo 'fn main() {}' > cli/src/main.rs

# Build dependencies only (this layer gets cached)
RUN cargo build --release && rm -rf src cli/src

# Copy actual source code
COPY src ./src
COPY cli/src ./cli/src
COPY quality ./quality

# Touch main.rs to force rebuild with actual code
RUN touch src/main.rs cli/src/main.rs

# Build the release binary
RUN cargo build --release --bin ralph

# Stage 2: Create minimal runtime image
FROM debian:bookworm-slim AS runtime

# Re-declare build args in runtime stage (required for multi-stage builds)
ARG VERSION
ARG COMMIT_SHA

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false ralph

WORKDIR /app

# Copy the built binary
COPY --from=builder /app/target/release/ralph /usr/local/bin/ralph

# Copy quality configuration (needed for quality checks)
COPY --from=builder /app/quality ./quality

# Set ownership
RUN chown -R ralph:ralph /app

# Switch to non-root user
USER ralph

# MCP server uses stdio for communication
# The entrypoint runs the MCP server mode
ENTRYPOINT ["ralph", "mcp-server"]

# Default: no PRD preloaded (can be overridden with --prd flag)
CMD []

# Labels for Docker MCP toolkit compatibility and version tracking
LABEL org.opencontainers.image.title="Ralph MCP Server"
LABEL org.opencontainers.image.description="Enterprise-ready autonomous AI agent framework with MCP server"
LABEL org.opencontainers.image.vendor="kcirtapfromspace"
LABEL org.opencontainers.image.source="https://github.com/kcirtapfromspace/ralph"
LABEL org.opencontainers.image.version="${VERSION}"
LABEL org.opencontainers.image.revision="${COMMIT_SHA}"
LABEL mcp.server="true"
LABEL mcp.transport="stdio"
