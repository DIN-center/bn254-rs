# BN254 Key Management Service API

## Overview

This document describes the API for the BN254 Key Management Service, a proof-of-concept implementation demonstrating separation of concerns for key management operations.

> **Important Note**: This is a proof-of-concept implementation. It does NOT provide production-level security guarantees.

## Quick Start

```bash
# Recommended: use salt-based key derivation
export BN254_SALT=$(openssl rand -hex 32)
cargo run --bin txtx-bn254-signer -- --salt "$BN254_SALT"

# Or use make
make serve
```

> **Important**: The salt must be stored securely as a secret to maintain determinism. The same salt + EOA address will always produce the same BLS key. If you lose the salt, you lose the ability to regenerate the keys.

## API Endpoints

### Key Management

#### Get Public Key for EOA
```
GET /key/{eoa_address}
```

Returns the public key components for a given EOA address. In derivation mode, keys are generated on-demand.

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
GET /keys
```

Returns public key components for all cached/loaded EOAs.

### Signing Operations

#### Sign Message
```
POST /sign
```

Signs a message (G1 curve point) using the BLS private key for the provided EOA.

**Request Body:**
```json
{
  "eoa_address": "0x1234...",
  "message": "00...00"  // 128 hex chars (64 bytes = x,y coordinates)
}
```

**Response:**
```json
{
  "signature": { "x": "0x...", "y": "0x..." },
  "g1": { "x": "0x...", "y": "0x..." },
  "g2": { "x": ["0x...", "0x..."], "y": ["0x...", "0x..."] }
}
```

#### Scalar Multiplication
```
POST /scalar_mul
```

Performs scalar multiplication on a G1 point using the private key.

**Request Body:**
```json
{
  "eoa_address": "0x1234...",
  "hash_x": "0x1234...",
  "hash_y": "0x5678..."
}
```

## Architecture

The service supports two operation modes:

1. **Derivation Mode** (recommended): Keys derived on-demand from EOA + salt
2. **Static Pool Mode** (legacy): Pre-generated keys from JSON files

## Security Notes

- This is a proof-of-concept - NOT production-ready
- In derivation mode, the salt is the master secret - store it securely (e.g., secrets manager, vault)
- Never commit the salt to version control
- See [FutureConsiderations.md](../../FutureConsiderations.md) for security roadmap