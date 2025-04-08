# Notes: Reproducing BN254 Contract Logic in Rust

## Goal

The purpose of this project is to **replicate and verify Solidity-based cryptographic logic** (specifically BN254 scalar multiplication) within a Rust environment, using the [`arkworks`](https://github.com/arkworks-rs) ecosystem.

By doing this, we can validate smart contract results (e.g. point operations in `BN254.G1Point`) against a known-good Rust implementation, allowing for independent correctness checks and deeper debugging workflows.

---

## Solidity ↔ Rust Mapping Strategy

### Solidity Side

Solidity uses a custom `BN254` library with types and methods like:

```solidity
struct G1Point {
    uint256 X;
    uint256 Y;
}

function scalar_mul(G1Point memory p, uint256 s) internal view returns (G1Point memory r)
```

These are **affine points** over the BN254 G1 curve:
- `X`, `Y`: elements of Fq (field modulus `FP_MODULUS`)
- `s`: scalar in Fr (field modulus `FR_MODULUS`)

Returned value (e.g. `sig_out`) is encoded as `abi.encodePacked(p.X, p.Y)` — a 64-byte array: X || Y.

---

### Rust Side

We use `ark_bn254` primitives:

```rust
use ark_bn254::{Fq, Fr, G1Affine, G1Projective};
```

Mapping of Solidity constructs:
| Solidity                | Rust Equivalent                     | Notes                                                  |
|-------------------------|-------------------------------------|--------------------------------------------------------|
| `uint256` (Fr)          | `Fr::from_str(...)?`                | Parse from decimal string                              |
| `uint256` (Fq)          | `Fq::from_str(...)?`                | Parse from decimal string                              |
| `G1Point`               | `G1Affine::new_unchecked(Fq, Fq)`   | Rust affine curve point                                |
| `scalar_mul(p, s)`      | `G1Projective::from(p).mul(s)`      | Scalar multiplication, converted back to affine        |
| ABI-encoded result      | `hex::decode(sig_out)` → [u8; 64]   | Decode X and Y from ABI output (big-endian byte order) |

---

## Data Assumptions

1. All numeric values (scalars and coordinates) are given as **decimal strings**.
2. `sig_out.value` is a **64-byte ABI-encoded result**: `X || Y` where each is 32 bytes, big-endian.
3. `pubkey_registration_message_hash.value` is a **nested JSON** array encoded as hex (e.g., `Vec<Vec<u8>>`).

---

## Validation Strategy

### Property-Based Testing

We run a `proptest` loop in Rust to validate scalar multiplication across 1000+ randomly generated scalars. Each test:
- Uses the **G1 generator** as the input point
- Applies scalar multiplication in Rust
- Calls the Solidity contract's `scalar_mul` with the same inputs
- Asserts equality of the resulting affine coordinates (x, y)

We also test edge cases:
- Scalar = 0
- Scalar = 1
- Scalar = -1
- Scalar = 2

All edge cases and fuzzed scalars passed, confirming behavioral equivalence of the Solidity and Rust implementations.

---

## Known Issue: `txtx` Output Deserialization

While the scalar multiplication logic works perfectly when directly driven through the Solidity interface, **attempts to simulate the transaction using the `txtx` tool output have failed**.

Symptoms:
- No points are validated or passed through correctly
- It's unclear whether the format mismatch lies in how the call is encoded or how `sig_out` is extracted

Next Steps:
- Compare direct call encoding against `txtx`-generated calldata
- Verify the `result` vs `abi_encoded_result` on `sig_out`
- Confirm ordering and layout of calldata and return data

---

## Dependencies

- `ark_bn254`
- `ark_ec`
- `ark_ff`
- `hex`
- `serde_json`
- `proptest`

---

## Debugging Tip from Micaiah

To see the actual Solidity return bytes, inspect `.result`, not `.abi_encoded_result`. The latter may contain meta-formatting or embedded type descriptors.
