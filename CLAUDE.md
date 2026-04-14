# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

### Rust/Cargo Commands
```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Run all tests
cargo test

# Run specific test suites
cargo test --test integration    # Integration tests
cargo test --test operators      # Operator tests  
cargo test --test signature      # Signature tests
cargo test --test solidity       # Solidity compatibility tests
cargo test --test prop_scalar_mul # Property-based tests

# Run the web service (Key Management Service)
# Recommended: use salt-based derivation mode
export BN254_SALT=$(openssl rand -hex 32)
cargo run --bin txtx-bn254-signer -- --salt "$BN254_SALT"

# Legacy: static pool mode with data directory
cargo run --bin txtx-bn254-signer -- --data-dir /path/to/data

# Run on a different port (default: 3000)
cargo run --bin txtx-bn254-signer -- --port 8080

# Run on all interfaces (for Docker)
cargo run --bin txtx-bn254-signer -- --host 0.0.0.0

# Run with custom log level
cargo run --bin txtx-bn254-signer -- --log-level debug
RUST_LOG=trace cargo run --bin txtx-bn254-signer

# Format code
cargo fmt

# Check linting
cargo clippy
```

### Solidity/Foundry Commands
```bash
# Build Solidity contracts
forge build

# Run Solidity tests
forge test
```

## Architecture Overview

This codebase implements two main components:

### 1. BN254 Cryptographic Library
A Rust implementation of BN254 curve operations compatible with EigenLayer's BN254.sol:
- **Core operations**: G1/G2 point arithmetic, scalar multiplication, pairing checks
- **Entry point**: `src/lib.rs`
- **Key modules**:
  - `g1.rs`: G1 group operations
  - `g2.rs`: G2 group operations  
  - `pairing.rs`: Bilinear pairing operations
  - `hash.rs`: Point hashing utilities
  - `utils.rs`: Field element conversions

### 2. Key Management Service (Proof of Concept)
A web service demonstrating separation of concerns for BLS key management:
- **Purpose**: Shows architectural pattern for isolating key operations (NOT production-ready)
- **Entry point**: `src/main.rs`
- **Web service**: `src/web/` directory
  - `server.rs`: Axum HTTP server setup (port 3000)
  - `handlers.rs`: API endpoint implementations
  - `models.rs`: Request/response structures
  - `store.rs`: Key storage with EOA mapping support

### Key Architectural Decisions
1. **Separation of Concerns**: Key management isolated from other operations to enable future security hardening
2. **API Design**: RESTful endpoints that accept only public data (hashes) for signing
3. **Compatibility**: BN254 operations match EigenLayer's Solidity implementation exactly
4. **Testing**: Comprehensive test suite including property-based tests and Solidity cross-validation

## API Endpoints

The Key Management Service provides:
- `GET /key/:eoa_address` - Get public key components
- `GET /keys` - List all public keys
- `POST /scalar_mul` - Perform scalar multiplication
- `POST /sign` - Sign a message hash with BLS

See `queries.http` for example requests and `src/web/README.md` for full API documentation.

## Important Notes

1. **Security**: This is a proof-of-concept demonstrating architectural patterns. It does NOT provide production security guarantees.
2. **Key Derivation** (recommended): Use `--salt` to derive keys deterministically from EOA + salt
3. **Key Store** (legacy): Uses `--data-dir` to load pre-generated keys from `keys.json` and `eoa-keymap.json`
4. **Port**: Service runs on port 3000 by default (configurable via `--port`)
5. **Host**: Service binds to `0.0.0.0` by default (all interfaces). Use `--host 127.0.0.1` for localhost-only
6. **Logging**: Use `--log-level` or `RUST_LOG` for debugging
7. **Binary Name**: The executable is named `txtx-bn254-signer`
8. **Docker Support**: Full Docker and docker-compose support with health checks and multi-stage builds

## Key Derivation (Recommended)

When using `--salt`:
1. Keys are derived on-demand: `BLS_Key = DeriveKey(Keccak256(EOA || ":" || Salt))`
2. Same EOA + salt always produces the same key
3. Keys are cached in memory after first derivation
4. No key files needed - just store the salt securely