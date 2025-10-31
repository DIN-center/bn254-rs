#!/bin/bash
set -e

echo "🔨 Building txtx-bn254-signer binary on host..."
cd /home/amal/work/din-workspace/bn254-rs

# Build the binary on host (much faster with cargo cache)
cargo build --release --bin txtx-bn254-signer

# Copy binary to build context
cp target/release/txtx-bn254-signer .

echo "🐳 Building Docker image with pre-built binary..."
# docker build -t bn254-rs:latest -f Dockerfile.fast .
docker build -t din-bn254:latest -f Dockerfile.fast .
# Clean up
rm -f txtx-bn254-signer

echo "✅ Done! Image built as bn254-rs:latest"
