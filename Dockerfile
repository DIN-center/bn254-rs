# Multi-stage build for txtx-bn254-signer
# Stage 1: Build dependencies
FROM rust:1.83-alpine AS dependencies

# Install build dependencies
RUN apk add --no-cache musl-dev

# Create a new empty project
WORKDIR /usr/src
RUN USER=root cargo new --bin app
WORKDIR /usr/src/app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies - this is the caching layer
RUN cargo build --release --bin txtx-bn254-signer
RUN rm -f src/*.rs target/release/deps/bn254_rs* || true

# Stage 2: Build application
FROM rust:1.83-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

WORKDIR /usr/src/app

# Copy dependencies from previous stage
COPY --from=dependencies /usr/src/app/target target
COPY --from=dependencies /usr/local/cargo /usr/local/cargo

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the application
RUN cargo build --release --bin txtx-bn254-signer

# Stage 3: Runtime
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    && addgroup -g 1000 app \
    && adduser -u 1000 -G app -D app

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/txtx-bn254-signer /usr/local/bin/txtx-bn254-signer

# Create data directory
RUN mkdir -p /app/data && chown -R app:app /app

WORKDIR /app

# Copy BLS key pool and EOA mapping
COPY --chown=app:app data/keys.json /app/data/keys.json
COPY --chown=app:app data/eoa-keymap.json /app/data/eoa-keymap.json

# Switch to non-root user
USER app

# Expose the default port
EXPOSE 8080

# Health check - using /health endpoint
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://127.0.0.1:8080/health || exit 1

# Default command - no mapping file needed by default
ENTRYPOINT ["txtx-bn254-signer"]
CMD ["--port", "8080", "--data-dir", "./data", "--log-level", "info"]
