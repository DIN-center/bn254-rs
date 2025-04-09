# BN254 Key Management Service

A secure web service for managing BN254 keys for AVS operators. This service provides secure key generation, storage, and signing operations within a secure enclave.

## Security Features

- Keys are generated and stored in a secure enclave
- Private keys never leave the secure environment
- All cryptographic operations performed within the enclave
- EOA-based authentication for operators
- Support for key rotation (planned)

## API Endpoints

### Key Management

- `GET /api/keys/{eoa_address}` - Get public key information for an operator
  - Returns only public components (G1, G2 points)
  - Private key remains in secure storage

- `GET /api/keys` - List all registered operator public keys
  - Returns only public components for all operators

### Signing Operations

- `POST /api/sign` - Sign a message using the operator's stored key
  - Requires EOA authentication
  - Returns signature and public components only

- `POST /api/scalar_mul` - Perform scalar multiplication (for BLS signatures)
  - Requires EOA authentication
  - Returns result and public components

## Example Usage

### Get Public Key Information

```bash
curl http://localhost:8080/api/keys/0x1234567890123456789012345678901234567890
```

Response:
```json
{
  "eoa_address": "0x1234567890123456789012345678901234567890",
  "public_key_g1": {
    "x": "...",
    "y": "..."
  },
  "public_key_g2": {
    "x_a": "...",
    "x_b": "...",
    "y_a": "...",
    "y_b": "..."
  }
}
```

### Sign Data

```bash
curl -X POST http://localhost:8080/api/sign \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <eoa_signature>" \
  -d '{
    "eoa_address": "0x1234567890123456789012345678901234567890",
    "point": "..."
  }'
```

Response:
```json
{
  "product": {
    "x": "...",
    "y": "..."
  },
  "signer_g1": {
    "x": "...",
    "y": "..."
  }
}
```

## Architecture

The service follows a secure architecture where:
1. Operators authenticate using their EOA
2. All key operations happen within a secure enclave
3. Only public data and signatures are returned
4. Private keys are never exposed

For detailed architecture and security considerations, see:
- [Key Management Design](../../KeyManagement.md)
- [Future Security Enhancements](../../FutureConsiderations.md)

## Development Setup

1. Install dependencies:
```bash
cargo build
```

2. Run the service:
```bash
cargo run --bin bn254-key-service
```

3. Run tests:
```bash
cargo test
```

## Security Notes

- This service is designed to be run in a secure environment
- Private keys are never exposed via the API. Current APIS that 
  expose keys are for debugging purposes and should be removed 
  in the future.
- All requests must be authenticated using EOA signatures
- Monitor logs for any unauthorized access attempts 