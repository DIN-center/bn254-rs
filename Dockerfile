# Multi-stage build for bn254-rs
FROM rust:1.81-slim as builder

# Install dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY contracts ./contracts
COPY testdata ./testdata
COPY tests ./tests
COPY foundry.toml ./

# Build the application in release mode
RUN cargo build --release --bin bn254-rs

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 appuser

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/bn254-rs /usr/local/bin/bn254-rs

# Create directory for data files
RUN mkdir -p /home/appuser/src/web

# Copy the players.json file
COPY --chown=appuser:appuser src/web/players.json /home/appuser/src/web/players.json

# Switch to non-root user
USER appuser

# Set working directory
WORKDIR /home/appuser

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV PORT=8080

# Run the binary
CMD ["bn254-rs"]