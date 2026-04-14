# bn254-rs

A Rust implementation of BN254 curve operations with a separate key management service for EigenLayer AVS operators.

## Overview

This project provides:
1. A Rust library for BN254 curve operations compatible with EigenLayer's [BN254.sol](https://github.com/Layr-Labs/eigenlayer-middleware/blob/dev/src/libraries/BN254.sol)
2. A proof-of-concept key management service demonstrating separation of concerns for AVS operators

## Project Status

⚠️ **Current Status**: **Experimental/Proof of Concept**

- ✅ Basic BN254 operations implemented and tested
- ✅ Property-based tests passing for scalar multiplication
- ✅ Solidity and Rust agree on scalar multiplication outputs
- ✅ **BLS handshake verified**: msg_hash * priv_key = sig_out
- ✅ **Key Management Service**: Proof of concept demonstrating separation of concerns

## Documentation Structure

### Core Documentation
- [README.md](README.md) - Project overview and BN254 library usage
- [KeyManagement.md](KeyManagement.md) - Key Management Service architecture and design
- [KEY_DERIVATION.md](KEY_DERIVATION.md) - **NEW**: On-demand key derivation mode with salt
- [FutureConsiderations.md](FutureConsiderations.md) - Security roadmap and future enhancements

### API Documentation
- [src/web/README.md](src/web/README.md) - Key Management Service API reference
- [src/lib.rs](src/lib.rs) - BN254 library API documentation

## Quick Start

### BN254 Library Usage
```bash
# Add to Cargo.toml
[dependencies]
bn254-rs = "0.1.0"  # Replace with actual version
```

### Key Management Service
```bash
# Recommended: derivation mode with salt (generates deterministic keys)
export BN254_SALT=$(openssl rand -hex 32)
cargo run --bin txtx-bn254-signer -- --salt "$BN254_SALT"

# Or quick start with random salt
make serve

# Run tests
cargo test        # Run all Rust tests
```

See [KEY_DERIVATION.md](KEY_DERIVATION.md) for detailed key derivation documentation.

## BN254 Library

### Features
- G1 and G2 group operations
- Bilinear pairing checks
- Point hashing
- Field element conversions

### Usage Examples

## Key Management Service

The Key Management Service is a proof-of-concept web service that demonstrates separation of concerns for managing BLS key pairs used by EigenLayer AVS operators.

### Features
- **Derivation Mode** (recommended): On-demand key derivation from EOA + salt
- **Static Pool Mode**: Pre-generated keys from key pool (legacy)
- RESTful API for key management operations
- BLS signing operations with cryptographically validated signatures
- Scalar multiplication support

### Running the Service

```bash
# Recommended: derivation mode with salt
export BN254_SALT=$(openssl rand -hex 32)
cargo run --bin txtx-bn254-signer -- --salt "$BN254_SALT"

# Or quick start with random salt
make serve

# Legacy: static pool mode (requires data directory)
cargo run --bin txtx-bn254-signer -- --data-dir ./data

# Start with custom log level (default: info)
cargo run --bin txtx-bn254-signer -- --salt "$BN254_SALT" --log-level debug
RUST_LOG=trace cargo run --bin txtx-bn254-signer -- --salt "$BN254_SALT"
```

**See [KEY_DERIVATION.md](KEY_DERIVATION.md) for detailed mode comparison and usage.**

### Logging and Tracing

The web service includes request tracing for easier integration and debugging:

- **Log Levels**: `error`, `warn`, `info`, `debug`, `trace`
- **Request Tracing**: All HTTP requests are automatically traced with method, path, status, and latency
- **Structured Logging**: Uses the `tracing` crate for structured, contextual logging
- **Environment Variable**: You can also set log level via `RUST_LOG` environment variable

```bash
# Using environment variable
RUST_LOG=debug cargo run --bin txtx-bn254-signer -- --salt "$BN254_SALT"

# View trace-level logs for detailed debugging
cargo run --bin txtx-bn254-signer -- --salt "$BN254_SALT" --log-level trace
```

### API Endpoints

#### Key Management
- `GET /key/:eoa_address` - Get public key components for a specific operator
- `GET /keys` - List all public keys

#### Cryptographic Operations
- `POST /scalar_mul` - Perform scalar multiplication
- `POST /sign` - Sign a message hash with BLS

### Example Usage

```bash
# Get public key for an operator (key derived automatically in derivation mode)
curl http://localhost:3000/key/0x70997970C51812dc3A010C7d01b50e0d17dc79C8

# Sign a message (message must be a valid G1 curve point)
curl -X POST http://localhost:3000/sign \
  -H "Content-Type: application/json" \
  -d '{
    "eoa_address": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
    "message": "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002"
  }'
```

**Important**: The `/sign` endpoint requires the `message` to be a **valid BN254 G1 curve point** (128 hex characters representing x,y coordinates).

### Testing with HTTP Client

For API testing, use the included HTTP request collection:

**File**: `queries.http`

This file contains ready-to-use HTTP requests for all endpoints, including:
- Health checks and key retrieval
- Scalar multiplication with various input formats
- Message signing with proper 128-character hex strings
- Registration parameter generation
- Error case testing (invalid inputs, missing fields)
- Performance testing scenarios

To use with VS Code REST Client extension or similar HTTP clients:
1. Open `queries.http` in your editor
2. Ensure the service is running on port 3000
3. Click "Send Request" on any example
4. Replace EOA addresses with actual values from your `data/eoa-keymap.json`

See [Key Management Design](KeyManagement.md) for detailed architecture and [API Documentation](src/web/README.md) for complete API reference.

### Testing

#### Rust Tests

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --test integration      # Integration tests
cargo test --test operators        # Operator tests
cargo test --test signature        # Signature tests
cargo test --test solidity         # Solidity compatibility tests
cargo test --test key_validation   # Pairing-based key validation
```

#### Key Validation Tests

The project includes pairing-based validation to verify that generated keys are mathematically correct:

```bash
# Run key validation tests
cargo test --test key_validation -- --nocapture
```

**What gets validated:**

The validation uses the Ethereum pairing precompile to verify keypairs through the BN254 bilinear pairing property. For a valid BLS keypair where both public keys are derived from the same private key:

```
e(pubkey_g1, G2) == e(G1, pubkey_g2)
```

This works because of bilinearity:
- If `pubkey_g1 = sk * G1` and `pubkey_g2 = sk * G2`
- Then `e(sk * G1, G2) = e(G1, G2)^sk = e(G1, sk * G2)`

**Test coverage:**
- Validates keys derived from EOA addresses using `derive_key_from_eoa()`
- Tests multiple keys in batch
- Detects invalid/mismatched keypairs
- Validates generator points

**Implementation:**
- Solidity contract: `contracts/BN254KeyValidator.sol`
- Rust tests: `tests/key_validation.rs`
- Uses local Anvil instance for testing against the pairing precompile

## Development

### Repository Structure
```
bn254-rs/
├── src/
│   ├── lib.rs          # BN254 library entry point
│   ├── g1.rs           # G1 group operations
│   ├── g2.rs           # G2 group operations
│   ├── pairing.rs      # Bilinear pairing operations
│   ├── hash.rs         # Point hashing utilities
│   ├── utils.rs        # Field element conversions
│   ├── keygen.rs       # Key derivation from EOA + salt
│   ├── main.rs         # Web service entry point
│   └── web/            # Key Management Service
│       ├── README.md   # API documentation
│       ├── mod.rs      # Module definition
│       ├── server.rs   # Axum server setup
│       ├── handlers.rs # API endpoint handlers
│       ├── models.rs   # Request/response models
│       └── store.rs    # Key storage and derivation logic
├── data/               # Sample data for static pool mode (legacy)
│   ├── eoa-keymap.json # EOA to key mapping
│   └── keys.json       # BLS key pool
├── tests/              # Integration tests
├── contracts/          # Solidity contracts
├── Makefile            # Build and test targets
├── KeyManagement.md    # Service architecture
├── KEY_DERIVATION.md   # Key derivation mode guide
└── FutureConsiderations.md  # Security roadmap
```

### Setup and Testing

```bash
# Install Rust dependencies
cargo build

# Run all Rust tests
cargo test

# Run specific test suites
cargo test --test integration    # Integration tests
cargo test --test operators      # Operator tests
cargo test --test signature      # Signature tests
cargo test --test solidity       # Solidity compatibility tests

# Run the web service (recommended: salt mode)
export BN254_SALT=$(openssl rand -hex 32)
cargo run --bin txtx-bn254-signer -- --salt "$BN254_SALT"

# Build for release
cargo build --release --bin txtx-bn254-signer
```

### Prerequisites

- **Rust**: 1.70 or later
- **Foundry**: For Solidity tests
- **mold linker**: Recommended for faster Linux builds (auto-configured)

## Maintenance Guide

This section provides guidance for maintaining and extending the BN254 Key Management Service.

### Code Organization

The web service is organized into distinct modules:

- **`src/main.rs`** - Entry point with CLI argument parsing
- **`src/web/mod.rs`** - Web service initialization
- **`src/web/server.rs`** - HTTP server configuration
- **`src/web/handlers.rs`** - API endpoint implementations
- **`src/web/models.rs`** - Request/response structures
- **`src/web/store.rs`** - Key storage logic

### Critical Maintenance Points

#### 1. Port Configuration
- **Current**: Port 3000 (hardcoded in `server.rs`)
- **To Change**: Update `server.rs`, README, and notify all clients

#### 2. Key Store Location
- **Current**: `./data/eoa-keymap.json` and `./data/keys.json`
- **Format**: JSON with specific structure (see `store.rs` docs)
- **To Change**: Update path in `store.rs` and documentation

#### 3. API Format

The service uses the following formats:

| Endpoint | Request Format | Response Format |
|----------|---------------|-----------------|
| `/sign` | `message`: 128-char hex string | G2 as arrays: `x:[a,b], y:[a,b]` |
| `/scalar_mul` | `hash_x`, `hash_y` coordinates | Same G2 array format |
| `/registration_params` | `message_hash` field | Includes `formatted_params` |

#### 4. Adding New Endpoints

1. Define models in `models.rs`
2. Implement handler in `handlers.rs`
3. Add route in `server.rs`
4. Update API documentation
5. Consider backward compatibility

#### 5. Logging and Debugging

- **Log Levels**: error, warn, info, debug, trace
- **Default**: info
- **Override**: `--log-level` or `RUST_LOG` env var
- **Trace Level**: Shows all request/response details

#### 6. Error Handling

| Status Code | Meaning | Common Causes |
|-------------|---------|---------------|
| 400 | Bad Request | Invalid input format |
| 404 | Not Found | EOA address not in store |
| 500 | Server Error | Private key parsing failed |

### Common Maintenance Tasks

#### Updating Dependencies
```bash
cargo update
cargo test
```

#### Adding a New Operator Key
Add to `data/eoa-keymap.json`:
```json
{
  "new_operator": {
    "pub": "0x...",
    "bls": {
      "priv_key": "...",
      "g1_x": "...",
      "g1_y": "...",
      "g2_x_0": "...",
      "g2_x_1": "...",
      "g2_y_0": "...",
      "g2_y_1": "..."
    }
  }
}
```

#### Debugging Request Issues
```bash
# Run with trace logging
cargo run --bin txtx-bn254-signer -- --salt "$BN254_SALT" --log-level trace

# Check specific request flow
curl -v http://localhost:3000/sign \
  -H "Content-Type: application/json" \
  -d '{"eoa_address":"0x70997970C51812dc3A010C7d01b50e0d17dc79C8", "message":"..."}'
```

### Security Considerations

1. **Derivation Mode** (recommended):
   - Salt is single point of failure - store securely (secrets manager, vault)
   - Never commit salt to version control
   - Use high-entropy salts (32+ bytes)
2. **Static Pool Mode** (legacy):
   - Private keys stored in plaintext JSON - ensure strict file permissions
   - Keys can be individually rotated
3. **Input Validation**: All hex strings and curve points validated before processing
4. **Memory Safety**: Derived keys cached in memory with thread-safe RwLock

See [KEY_DERIVATION.md](KEY_DERIVATION.md) for detailed security recommendations.

### Performance Notes

- Key lookups: O(1) via HashMap
- No database queries during requests
- Cryptographic operations are CPU-bound
- Consider load balancing for high traffic

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. When contributing:

1. Ensure all tests pass
1. Add tests for new functionality
1. Update documentation as needed
1. Follow the existing code style

## License

This project is licensed under the [MIT license](./LICENSE.md).

## Related Documentation

- [EigenLayer Documentation](https://github.com/Layr-Labs/eigenlayer-middleware)
- [BN254 Curve Specifications](https://eips.ethereum.org/EIPS/eip-197)
- [BLS Signature Scheme](https://eips.ethereum.org/EIPS/eip-2539)
