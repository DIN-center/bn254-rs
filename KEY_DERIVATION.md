# Key Derivation Mode

This document describes the key derivation mode for the BN254 signer service.

## Overview

The service now supports deriving BN254 BLS keys on-demand from EOA addresses + a secret salt, eliminating the need for pre-generated key pools.

## Quick Start

### Generate a Random Salt

```bash
# Generate a high-entropy salt (recommended: 32+ bytes)
export BN254_SALT=$(openssl rand -hex 32)
echo "Your salt: $BN254_SALT"
```

### Run the Service

```bash
# Quick start with Makefile
make serve

# Or manually with salt
cargo run --bin txtx-bn254-signer -- --salt "$BN254_SALT"

# Or with custom port/host
cargo run --bin txtx-bn254-signer -- \
  --salt "$BN254_SALT" \
  --port 3000 \
  --host 0.0.0.0
```

## How It Works

1. **Derivation Formula**: `BLS_Key = DeriveKey(Keccak256(EOA_Address || ":" || Salt))`
2. **Deterministic**: Same EOA + salt always produces the same key
3. **On-Demand**: Keys are generated when first requested
4. **Cached**: Once generated, keys are cached in memory for performance
5. **No Storage**: Only the salt needs to be stored securely

## API Usage

The API remains unchanged. All existing endpoints work in derivation mode:

```bash
# Get public key for an EOA (key is derived automatically)
curl http://localhost:3000/key/0x70997970C51812dc3A010C7d01b50e0d17dc79C8

# Sign a message (key is derived if not cached)
# NOTE: message must be a valid G1 curve point (128 hex chars = 64 bytes for x,y coordinates)
curl -X POST http://localhost:3000/sign \
  -H "Content-Type: application/json" \
  -d '{
    "eoa_address": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
    "message": "0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002"
  }'
```

**Important**: The `message` field must contain a **valid BN254 G1 curve point** as 128 hex characters (64 chars for x coordinate + 64 chars for y coordinate). Random data will fail validation.

## Security Recommendations

### For Production-Like Temporary Use

1. **Salt Storage**
   ```bash
   # Store salt in environment variable (loaded from secrets manager)
   export BN254_SALT=$(aws secretsmanager get-secret-value \
     --secret-id bn254-salt --query SecretString --output text)

   # Run service
   cargo run --bin txtx-bn254-signer -- --salt "$BN254_SALT"
   ```

2. **Salt Generation**
   ```bash
   # Generate and store a new salt
   NEW_SALT=$(openssl rand -hex 32)
   aws secretsmanager create-secret \
     --name bn254-salt \
     --secret-string "$NEW_SALT"
   ```

3. **Access Control**
   - Limit who can read the salt secret
   - Use IAM roles/policies for AWS secrets
   - Audit access to the salt

### For Development/Testing

```bash
# Simple approach - salt in environment
export BN254_SALT="dev-salt-not-for-production"
cargo run --bin txtx-bn254-signer -- --salt "$BN254_SALT"
```

## Comparison: Operation Modes

### 1. Derivation Mode (Recommended)
```bash
cargo run --bin txtx-bn254-signer -- --salt "secret-salt"
```
- ✅ Scales to unlimited EOAs
- ✅ No key files to manage
- ✅ Deterministic: same EOA + salt = same key
- ⚠️ Salt is single point of failure - store securely
- **Use when**: Need to support many dynamic EOAs

### 2. Static Pool Mode (Legacy)
```bash
cargo run --bin txtx-bn254-signer -- --data-dir ./data
```
- ✅ Pre-generated random keys
- ✅ Keys can be individually rotated
- ❌ Limited to pool size (50 keys)
- **Use when**: Fixed set of EOAs, need individual key rotation

## Migration Guide

### From Static Pool → Derivation

Before:
```bash
# Old: Required eoa-keymap.json + keys.json
cargo run --bin txtx-bn254-signer -- --data-dir ./data
```

After:
```bash
# New: Just need a salt
export BN254_SALT=$(openssl rand -hex 32)
cargo run --bin txtx-bn254-signer -- --salt "$BN254_SALT"
```

**Note**: Keys will be different! The derived keys are not the same as the pre-generated pool keys.

## Troubleshooting

### Server won't start
```bash
# Check if port is already in use
lsof -i :3000

# Try a different port
cargo run --bin txtx-bn254-signer -- --salt "$BN254_SALT" --port 3001
```

### Keys don't match expectations
- Verify you're using the same salt
- Check that EOA addresses are checksummed consistently
- Ensure no typos in EOA address (case-sensitive after 0x prefix is removed and lowercased)

### Signature validation fails
- Ensure message is a valid G1 curve point
- Verify coordinates are in affine form (not projective)
- Run `cargo test --test key_validation` to validate key pairs

### Performance concerns
- First key derivation takes ~1-2ms
- Subsequent requests (cached) take ~microseconds
- Cache is unbounded - consider monitoring memory in long-running deployments

## Testing

### Rust Tests

```bash
# Run all tests
cargo test

# Run key validation tests (verifies BLS pairing)
cargo test --test key_validation

# Run integration tests
cargo test --test integration
```

### Manual Testing

Test individual endpoints:
```bash
# Test sign endpoint with curl
make test-sign

# Get public key
curl http://localhost:3000/key/0x70997970C51812dc3A010C7d01b50e0d17dc79C8
```

## Implementation Details

- **Hash Function**: Keccak256 (SHA3-256)
- **Curve**: BN254 (alt_bn128)
- **Key Format**: Standard BLS (G1 and G2 public keys)
- **Caching**: In-memory HashMap with RwLock
- **Thread Safety**: Safe for concurrent access
- **Coordinate Handling**: Proper affine-to-projective conversion using `G1Affine::new_unchecked()` then `G1Projective::from()` to ensure valid curve operations
- **Library**: Uses `ark-bn254` with correct coordinate system handling

## Limitations

1. **One key per EOA**: Cannot generate multiple keys for same EOA
2. **No key rotation**: Same EOA always gets same key (unless salt changes)
3. **Salt compromise**: Exposes all keys for all EOAs
4. **Memory growth**: Cache grows with unique EOA count

## Best Practices

✅ **DO:**
- Use high-entropy salt (32+ bytes)
- Store salt in secrets manager (AWS Secrets Manager, Vault, etc.)
- Monitor access to the salt
- Document the salt's location for recovery
- Set up alerts for unusual key derivation patterns

❌ **DON'T:**
- Hard-code salt in source code
- Check salt into version control
- Use weak/predictable salts
- Share salt across multiple environments
- Store salt in plain text files

## Examples

### Docker Deployment
```dockerfile
# Dockerfile
FROM rust:1.90 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin txtx-bn254-signer

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/txtx-bn254-signer /usr/local/bin/
EXPOSE 3000
ENTRYPOINT ["txtx-bn254-signer"]
```

```bash
# Run with salt from environment
docker run -e BN254_SALT="$BN254_SALT" -p 3000:3000 bn254-signer \
  --salt "$BN254_SALT" --host 0.0.0.0
```

### Kubernetes Deployment
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: bn254-salt
type: Opaque
stringData:
  salt: "your-secret-salt-here"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bn254-signer
spec:
  replicas: 1
  selector:
    matchLabels:
      app: bn254-signer
  template:
    metadata:
      labels:
        app: bn254-signer
    spec:
      containers:
      - name: signer
        image: bn254-signer:latest
        args:
          - "--salt"
          - "$(BN254_SALT)"
          - "--host"
          - "0.0.0.0"
        env:
        - name: BN254_SALT
          valueFrom:
            secretKeyRef:
              name: bn254-salt
              key: salt
        ports:
        - containerPort: 3000
```

## Support

For issues or questions:
- Check logs: `RUST_LOG=debug cargo run --bin txtx-bn254-signer -- --salt "$SALT"`
- Review tests: `cargo test keygen`
- Examine implementation: `src/keygen.rs`

## Recent Fixes

### Coordinate System Bug (Fixed)

**Issue**: Signatures were producing invalid curve points due to incorrect coordinate handling.

**Root Cause**: The sign endpoint was using `G1Projective::new_unchecked(x, y, Fq::one())` which incorrectly treated affine coordinates as if they were already in projective form.

**Fix**: Changed to proper conversion:
```rust
// Before (incorrect)
let point = G1Projective::new_unchecked(x, y, Fq::one());

// After (correct)
let affine_point = G1Affine::new_unchecked(x, y);
let point = G1Projective::from(affine_point);
```

**Validation**: Run `cargo test --test key_validation` to verify key pairs pass BLS pairing verification.

## Future Improvements

If this temporary solution needs to be extended:
- Add salt rotation mechanism
- Implement key derivation rate limiting
- Add metrics/monitoring
- Support multiple salts with key versioning
- Implement cache eviction policies
- Add audit logging for key derivations
