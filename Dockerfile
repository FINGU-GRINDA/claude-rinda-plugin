# Stage 1: Build
FROM rust:1.88-slim AS builder

# Install system dependencies (pkg-config needed for some crates)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./

# Copy only the workspace members needed by rinda-mcp
COPY crates/sdk/ ./crates/sdk/
COPY crates/common/ ./crates/common/
COPY crates/mcp-server/ ./crates/mcp-server/

# Copy doc/openapi.json required by sdk/build.rs
COPY doc/openapi.json ./doc/openapi.json

# Remove crates/cli workspace member (not in Docker build context)
RUN sed -i 's/"crates\/cli", //' Cargo.toml

# Build release binary for rinda-mcp
RUN cargo build --release -p rinda-mcp

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install ca-certificates for HTTPS connections to rinda.ai API
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rinda-mcp /usr/local/bin/rinda-mcp

EXPOSE 3000

ENV PORT=3000 \
    RUST_LOG=info

CMD ["rinda-mcp"]
