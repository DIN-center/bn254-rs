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
# Run the service
cargo run --bin bn254-rs

# Run tests
cargo test
```

## BN254 Library

### Features
- G1 and G2 group operations
- Bilinear pairing checks
- Point hashing
- Field element conversions

### Usage Examples

## Key Management Service

### Features
- Separation of concerns for security foundation
- Signing operations with keys from storage
- No key generation, only read from existing key store

See [Key Management Design](KeyManagement.md) for detailed architecture.

## Development

### Repository Structure
```
bn254-rs/
├── src/
│   ├── lib.rs          # BN254 library implementation
│   ├── web/            # Key Management Service
│   │   ├── README.md   # API documentation
│   │   ├── handlers.rs # API endpoints
│   │   ├── models.rs   # Data models
│   │   └── server.rs   # Server implementation
├── KeyManagement.md    # Service architecture
└── FutureConsiderations.md  # Security roadmap
```

### Setup and Testing

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
