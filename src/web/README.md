# BN254 Web Service

This web service provides a REST API for BN254 curve operations, specifically for scalar multiplication and key management.

## API Endpoints

### Key Management

- `GET /api/keys` - List all key pairs
- `GET /api/keys/{eoa_address}` - Get a key pair by EOA address
- `POST /api/keys` - Create a new key pair

### BN254 Operations

- `POST /api/scalar-mul` - Perform scalar multiplication on a G1 point

## Example Usage

### Scalar Multiplication

```bash
curl -X POST http://localhost:8080/api/scalar-mul \
  -H "Content-Type: application/json" \
  -d '{
    "g1_x": "1",
    "g1_y": "2",
    "scalar": "3"
  }'
```

Response:
```json
{
  "result_x": "3",
  "result_y": "6"
}
```

### Create a Key Pair

```bash
curl -X POST http://localhost:8080/api/keys \
  -H "Content-Type: application/json" \
  -d '{
    "eoa_address": "0x1234567890123456789012345678901234567890",
    "g1_x": "1",
    "g1_y": "2",
    "g2_x_a": "1",
    "g2_x_b": "2",
    "g2_y_a": "3",
    "g2_y_b": "4",
    "private_key": "5"
  }'
```

Response:
```json
1
```

### Get a Key Pair

```bash
curl http://localhost:8080/api/keys/0x1234567890123456789012345678901234567890
```

Response:
```json
{
  "id": 1,
  "eoa_address": "0x1234567890123456789012345678901234567890",
  "g1_x": "1",
  "g1_y": "2",
  "g2_x_a": "1",
  "g2_x_b": "2",
  "g2_y_a": "3",
  "g2_y_b": "4",
  "private_key": "5",
  "created_at": "2023-04-09T12:00:00Z"
}
``` 