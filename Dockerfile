# Stage 1: Build
FROM rust:slim-bookworm AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev curl \
    && rm -rf /var/lib/apt/lists/*

# Install nightly toolchain + wasm target
RUN rustup toolchain install nightly \
    && rustup default nightly \
    && rustup target add wasm32-unknown-unknown

# Install cargo-leptos
RUN cargo install cargo-leptos

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/

# Build the release binary + WASM bundle
RUN cargo leptos build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary
COPY --from=builder /app/target/release/splitvibe-server ./splitvibe-server

# Copy the site assets (WASM, CSS, JS)
COPY --from=builder /app/target/site ./target/site

# Copy migrations for runtime migration support
COPY --from=builder /app/crates/splitvibe-db/migrations ./crates/splitvibe-db/migrations

# Default environment
ENV LEPTOS_SITE_ADDR=0.0.0.0:8080
ENV LEPTOS_SITE_ROOT=target/site

EXPOSE 8080

HEALTHCHECK --interval=10s --timeout=3s --start-period=30s --retries=3 \
    CMD curl -sf http://localhost:8080/api/health || exit 1

ENTRYPOINT ["./splitvibe-server"]
