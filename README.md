# bn254-rs

A Rust implementation of BN254 curve operations that mirrors EigenLayer's BN254.sol library.

## Project Status

⚠️ **Current Status**: **Experimental/In Development**

The library is currently in development with the following status:
- ✅ Basic BN254 operations implemented and tested
- ✅ Property-based tests passing for scalar multiplication
- ✅ Solidity and Rust agree on scalar multiplication outputs
- ❌ **sig_out mismatch**:
  - We generated a custom `for_testing` output via `txtx` in `operator/step-5`
    ```hcl2
    output "for_testing" {
      value = {
        g1 = variable.g1
        g2 = variable.g2
        priv_key = evm::uint256(input.priv_key)
        call_pubkey_registration_message_hash_result = action.call_pubkey_registration_message_hash.result
        sig_out = action.scalar_mul.result
      }
    }
    ```
  - Rust scalar multiplication matches `cast call` results directly from Solidity
  - But `sig_out` returned by `txtx` **does not match** the actual contract
  call output suggesting we should investigate `sig_out`s provinance


### Quick Start

```bash
# Add to Cargo.toml
[dependencies]
bn254-rs = "0.1.0"  # Replace with actual version

# Run tests
cargo test-fuzz      # Run only the randomized fuzz tests
```

## Overview

This library provides a Rust implementation of the BN254 elliptic curve operations, designed to be compatible with EigenLayer's [BN254.sol](https://github.com/Layr-Labs/eigenlayer-middleware/blob/dev/src/libraries/BN254.sol) Solidity library. It offers the following operations for working with points on the BN254 curve:

- G1 and G2 group operations
- Bilinear pairing checks
- Point hashing
- Field element conversions

## Project Purpose

The primary purpose of this project is to provide a pure Rust implementation of BN254 curve operations for TXTX. Currently, TXTX relies on a deployed BN254Wrapper contract to perform scalar multiplication operations, creating an unnecessary coupling between on-chain operations and cryptographic functions. This library aims to move BLS signature operations to a secure, off-chain signing pipeline, eliminating the need for smart contract interactions for basic cryptographic operations.

```yaml
action "scalar_mul" "evm::call_contract" {
    description = "Call BN254.scalar_mul to create a signature with the private key"
    contract_address = input.bn254wrapper
    contract_abi = variable.bn254wrapper_contract.abi
    function_name = "scalar_mul"
    function_args = [
        variable.pubkey_registration_message_hash,
        evm::uint256(input.priv_key)
    ]
    signer = signer.operator
}
```

By implementing these operations in pure Rust, we can:

1. **Improve Performance**: Eliminate the overhead of EVM calls
2. **Reduce Complexity**: Simplify the pipeline by removing the need for contract interactions
3. **Move EigenLayer's operator-set handshake signing**: Move this complex process to a more secure key management workflow.

This library serves as a foundation for building a complete Rust-based pipeline for TXTX operations, making the process more efficient and maintainable.

## Features

- **G1 Operations**: Generator point retrieval, point negation, point addition, and scalar multiplication
- **G2 Operations**: Generator point retrieval and point negation
- **Pairing Operations**: Bilinear pairing checks between G1 and G2 points
- **Hashing**: Keccak-256 hashing of G1 points
- **Utilities**: Conversion between field elements and byte representations

## Usage

### G1 Operations

```rust
use din_bn254::g1::{g1_generator, g1_negate, g1_add, g1_scalar_mul};
use ark_bn254::{G1Projective, Fr};

// Get the generator point
let g = g1_generator();

// Negate a point
let neg_g = g1_negate(g);

// Add two points
let sum = g1_add(g, g);

// Scalar multiplication
let scalar = Fr::from(2u64);
let doubled = g1_scalar_mul(g, scalar);
```

### G2 Operations

```rust
use din_bn254::g2::{g2_generator, g2_negate};
use ark_bn254::G2Projective;

// Get the generator point
let g2 = g2_generator();

// Negate a point
let neg_g2 = g2_negate(g2);
```

### Pairing Operations

```rust
use din_bn254::pairing::pairing_check;
use ark_bn254::{G1Projective, G2Projective};

// Perform a pairing check
let result = pairing_check(g1, g2, h1, h2);
```

### Hashing

```rust
use din_bn254::hash::hash_g1_point;
use ark_bn254::G1Projective;

// Hash a G1 point
let hash = hash_g1_point(&g1_point);
```

### Utilities

```rust
use din_bn254::utils::fr_to_be_bytes;
use ark_bn254::Fr;

// Convert a field element to bytes
let field_element = Fr::from(123u64);
let bytes = fr_to_be_bytes(&field_element);
```

## Technical Implementation Details

### Solidity ↔ Rust Mapping

#### Solidity Side

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

#### Rust Side

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

### Data Assumptions

1. All numeric values (scalars and coordinates) are given as **decimal strings**.
2. `sig_out.value` is a **64-byte ABI-encoded result**: `X || Y` where each is 32 bytes, big-endian.
3. `pubkey_registration_message_hash.value` is a **nested JSON** array encoded as hex (e.g., `Vec<Vec<u8>>`).

### Validation Strategy

#### Property-Based Testing

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

## Known Issues and Next Steps

### Current Issues

While the scalar multiplication logic works perfectly when directly driven through the Solidity interface, **attempts to simulate the transaction using the `txtx` tool output have failed**.

Symptoms:
- No points are validated or passed through correctly
- It's unclear whether the format mismatch lies in how the call is encoded or how `sig_out` is extracted

### Next Steps

1. **Immediate Tasks**:
   - Compare direct call encoding against `txtx`-generated calldata
   - Verify the `result` vs `abi_encoded_result` on `sig_out`
   - Confirm ordering and layout of calldata and return data

2. **Future Improvements**:
   - Add more comprehensive error handling
   - Improve documentation of edge cases
   - Add benchmarks for performance comparison
   - Implement additional test cases

> **Debugging Tip**: To see the actual Solidity return bytes, inspect `.result`, not `.abi_encoded_result`. The latter may contain meta-formatting or embedded type descriptors.

## Modules

- **g1.rs**: Operations on the G1 group of the BN254 curve
- **g2.rs**: Operations on the G2 group of the BN254 curve
- **pairing.rs**: Bilinear pairing operations between G1 and G2 points
- **hash.rs**: Hashing operations for curve points
- **utils.rs**: Utility functions for working with field elements

## Relationship to EigenLayer's BN254.sol

This library is designed to be a direct Rust counterpart to EigenLayer's BN254.sol Solidity library. It implements the same operations and follows the same mathematical principles, making it suitable for computing signing proof for `registerForOperatorSet`. Note: This is not verified, and this repo is spike to test if it is viable.

## Dependencies

- [ark-bn254](https://github.com/arkworks-rs/algebra): Provides the BN254 curve implementation
- [ark-ec](https://github.com/arkworks-rs/algebra): Provides elliptic curve operations
- [ark-ff](https://github.com/arkworks-rs/algebra): Provides finite field operations
- [sha3](https://crates.io/crates/sha3): Provides Keccak-256 hashing
- [hex](https://crates.io/crates/hex): For hex encoding/decoding
- [serde_json](https://crates.io/crates/serde_json): For JSON serialization
- [proptest](https://crates.io/crates/proptest): For property-based testing

## Development

### Setup

1. Clone the repository
2. Install dependencies:
   ```bash
   cargo build
   ```
3. Run tests:
   ```bash
   cargo test
   cargo test-fuzz  # For property-based tests
   ```

### Contributing

Contributions are welcome! Please feel free to submit a Pull Request. When contributing:

1. Ensure all tests pass
2. Add tests for new functionality
3. Update documentation as needed
4. Follow the existing code style

## License

This project is licensed under the [MIT license](./LICENSE.md).
