# BN254 Key Management Service API

## Overview

This document describes the API for the BN254 Key Management Service, a proof-of-concept implementation demonstrating separation of concerns for key management operations. This service is designed to show how key management can be isolated from other system components, establishing a foundation for future security enhancements.

> **Important Note**: This is a proof-of-concept implementation that demonstrates architectural patterns for separation of concerns. It does NOT provide production-level security guarantees. The current implementation is for demonstration purposes only.

## API Endpoints

### Key Management

#### Get Public Key for EOA
```
GET /api/keys/{eoa_address}
```

Returns the public key components for a given EOA address.

**Parameters:**
- `eoa_address` (path): The Ethereum address of the operator

**Response:**
```json
{
  "g1": {
    "x": "0x1234...",
    "y": "0x5678..."
  },
  "g2": {
    "x": ["0xabcd...", "0xefgh..."],
    "y": ["0xijkl...", "0xmnop..."]
  }
}
```

#### List All Public Keys
```
GET /api/keys
```

Returns public key components for all registered EOAs.

**Response:**
```json
{
  "keys": [
    {
      "eoa": "0x1234...",
      "g1": {
        "x": "0x1234...",
        "y": "0x5678..."
      },
      "g2": {
        "x": ["0xabcd...", "0xefgh..."],
        "y": ["0xijkl...", "0xmnop..."]
      }
    }
  ]
}
```

### Signing Operations

#### Sign Data
```
POST /api/sign
```

Signs data using the private key associated with the provided EOA.

**Request Body:**
```json
{
  "eoa": "0x1234...",
  "hash": {
    "x": "0x1234...",
    "y": "0x5678..."
  }
}
```

**Response:**
```json
{
  "signature": {
    "x": "0x1234...",
    "y": "0x5678..."
  }
}
```

#### Scalar Multiplication
```
POST /api/scalar_mul
```

Performs scalar multiplication on a G1 point using the private key associated with the provided EOA.

**Request Body:**
```json
{
  "eoa": "0x1234...",
  "point": {
    "x": "0x1234...",
    "y": "0x5678..."
  }
}
```

**Response:**
```json
{
  "result": {
    "x": "0x1234...",
    "y": "0x5678..."
  }
}
```

## Architecture

The service is designed with separation of concerns in mind:

1. **Key Storage**: Keys are stored separately from the application logic
2. **Signing Operations**: Cryptographic operations are isolated in a dedicated service
3. **API Interface**: Clean API boundaries for future security enhancements

This architecture demonstrates the pattern needed for a secure key management system, though the current implementation does not provide production-level security guarantees.

## Development Setup

### Building
```bash
cargo build
```

### Running
```bash
cargo run --bin bn254-key-service
```

### Testing
```bash
cargo test
```

## Security Notes

This proof-of-concept implementation:
- Demonstrates separation of concerns for key management
- Shows how cryptographic operations can be isolated
- Establishes patterns for future security enhancements
- Does NOT provide production-level security guarantees

For a production implementation, additional security measures would be required as outlined in [FutureConsiderations.md](../../FutureConsiderations.md).