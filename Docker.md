# Docker Build and Versioning Guide

This document provides instructions for building and versioning the bn254-rs Docker images.

## Overview

The project uses a multi-stage Docker build process to create an optimized runtime image for the `txtx-bn254-signer` service. The docker-compose.yml follows modern Docker Compose standards (no version field required).

The build process:

1. Caches dependencies in a separate stage for faster rebuilds
2. Builds the Rust application in Alpine Linux for smaller image size
3. Creates a minimal runtime image with only necessary components
4. Runs as a non-root user for security

## Building the Docker Image

### Using Docker Compose (Recommended)

```bash
# Build the image
docker-compose build

# Build with no cache (for clean builds)
docker-compose build --no-cache

# Build with specific build args
docker-compose build --build-arg RUST_VERSION=1.83
```

### Using Docker CLI

```bash
# Build with default tag
docker build -t txtx-bn254-signer:latest .

# Build with specific version tag
docker build -t txtx-bn254-signer:v1.0.0 .

# Build with multiple tags
docker build -t txtx-bn254-signer:latest -t txtx-bn254-signer:v1.0.0 .

# Build with build arguments
docker build --build-arg RUST_VERSION=1.83 -t txtx-bn254-signer:latest .
```

## Versioning Strategy

### Semantic Versioning

Use semantic versioning (MAJOR.MINOR.PATCH) for releases:

```bash
# Major version (breaking changes)
docker build -t txtx-bn254-signer:v2.0.0 -t txtx-bn254-signer:v2 -t txtx-bn254-signer:latest .

# Minor version (new features, backwards compatible)
docker build -t txtx-bn254-signer:v1.1.0 -t txtx-bn254-signer:v1.1 -t txtx-bn254-signer:v1 -t txtx-bn254-signer:latest .

# Patch version (bug fixes)
docker build -t txtx-bn254-signer:v1.0.1 -t txtx-bn254-signer:v1.0 -t txtx-bn254-signer:v1 -t txtx-bn254-signer:latest .
```

### Development Builds

For development and testing:

```bash
# Development build with commit hash
export COMMIT=$(git rev-parse --short HEAD)
docker build -t txtx-bn254-signer:dev-$COMMIT .

# Feature branch build
export BRANCH=$(git branch --show-current | sed 's/\//-/g')
docker build -t txtx-bn254-signer:$BRANCH .

# Nightly/unstable builds
docker build -t txtx-bn254-signer:nightly-$(date +%Y%m%d) .
```

## Build Optimization

### Cache Management

The Dockerfile uses multi-stage builds to cache dependencies:

```bash
# Leverage build cache
docker build --cache-from txtx-bn254-signer:latest -t txtx-bn254-signer:new .

# Export/import build cache
docker build --cache-to type=local,dest=/tmp/docker-cache .
docker build --cache-from type=local,src=/tmp/docker-cache .
```

### Build Arguments

Customize the build process:

```bash
# Use different Rust version
docker build --build-arg RUST_VERSION=1.84 -t txtx-bn254-signer:rust1.84 .

# Use different Alpine version
docker build --build-arg ALPINE_VERSION=3.20 -t txtx-bn254-signer:alpine3.20 .
```

## Running the Container

### Using Docker Compose

```bash
# Start the service
docker-compose up -d

# View logs
docker-compose logs -f txtx-bn254-signer

# Stop the service
docker-compose down
```

### Using Docker CLI

```bash
# Run with default settings
docker run -d --name txtx-bn254-signer -p 3000:3000 txtx-bn254-signer:latest

# Run with custom configuration
docker run -d \
  --name txtx-bn254-signer \
  -p 3000:3000 \
  -v $(pwd)/data:/app/data:ro \
  -e RUST_LOG=debug \
  txtx-bn254-signer:latest \
  --db /app/data/players.json \
  --port 3000 \
  --log-level debug

# Run with resource limits
docker run -d \
  --name txtx-bn254-signer \
  -p 3000:3000 \
  --memory="1g" \
  --cpus="2.0" \
  txtx-bn254-signer:latest
```

## Health Checks

The container includes health checks:

```bash
# Check container health
docker ps --format "table {{.Names}}\t{{.Status}}"

# Manually test health endpoint
curl http://localhost:3000/keys
```

## Security Considerations

The Docker setup includes several security features:

1. **Non-root user**: Runs as user `app` (UID 1000)
2. **Read-only filesystem**: Container filesystem is read-only
3. **No new privileges**: Prevents privilege escalation
4. **Minimal base image**: Uses Alpine Linux for smaller attack surface
5. **Resource limits**: CPU and memory limits in docker-compose.yml

## CI/CD Integration

### GitHub Actions Example

```yaml
- name: Build and push Docker image
  run: |
    VERSION=$(git describe --tags --always)
    docker build -t txtx-bn254-signer:$VERSION .
    docker tag txtx-bn254-signer:$VERSION txtx-bn254-signer:latest
    # Push to registry (configure registry credentials first)
    # docker push txtx-bn254-signer:$VERSION
    # docker push txtx-bn254-signer:latest
```

### Version Automation Script

Create a `scripts/docker-build.sh`:

```bash
#!/bin/bash
set -e

# Get version from Cargo.toml or git
VERSION=$(grep '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/')
COMMIT=$(git rev-parse --short HEAD)
BRANCH=$(git branch --show-current)

# Build with multiple tags
docker build \
  -t txtx-bn254-signer:latest \
  -t txtx-bn254-signer:v$VERSION \
  -t txtx-bn254-signer:$BRANCH-$COMMIT \
  .

echo "Built images:"
echo "  - txtx-bn254-signer:latest"
echo "  - txtx-bn254-signer:v$VERSION"
echo "  - txtx-bn254-signer:$BRANCH-$COMMIT"
```

## Troubleshooting

### Build Issues

```bash
# Clean Docker cache
docker system prune -a

# Build with verbose output
docker build --progress=plain .

# Debug build context
docker build --no-cache --progress=plain --target dependencies .
```

### Runtime Issues

```bash
# Check container logs
docker logs txtx-bn254-signer

# Execute commands in running container
docker exec -it txtx-bn254-signer sh

# Inspect container configuration
docker inspect txtx-bn254-signer
```

## Docker Image Structure

### Build Stages

1. **dependencies**: Builds and caches Rust dependencies
2. **builder**: Compiles the application
3. **runtime**: Minimal Alpine image with the binary

### Image Layers

- Base: Alpine Linux 3.19 (~5MB)
- Runtime dependencies: ca-certificates, libgcc (~10MB)
- Application binary: txtx-bn254-signer (~15-20MB)
- Total image size: ~30-35MB

### File Layout

```
/app/
├── data/
│   └── players.json    # Key storage (mounted volume)
└── /usr/local/bin/
    └── txtx-bn254-signer  # Application binary
```