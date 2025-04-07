# bn254-rs

A Rust implementation of BN254 curve operations that mirrors EigenLayer's BN254.sol library.

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

## Modules

- **g1.rs**: Operations on the G1 group of the BN254 curve
- **g2.rs**: Operations on the G2 group of the BN254 curve
- **pairing.rs**: Bilinear pairing operations between G1 and G2 points
- **hash.rs**: Hashing operations for curve points
- **utils.rs**: Utility functions for working with field elements

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

## Relationship to EigenLayer's BN254.sol

This library is designed to be a direct Rust counterpart to EigenLayer's BN254.sol Solidity library. It implements the same operations and follows the same mathematical principles, making it suitable for computing signing proof for `registerForOperatorSet`. Note: This is not verified, and this repo is spike to test if it is viable.


## Dependencies

- [ark-bn254](https://github.com/arkworks-rs/algebra): Provides the BN254 curve implementation
- [ark-ec](https://github.com/arkworks-rs/algebra): Provides elliptic curve operations
- [ark-ff](https://github.com/arkworks-rs/algebra): Provides finite field operations
- [sha3](https://crates.io/crates/sha3): Provides Keccak-256 hashing

## License

This project is licensed under the [MIT license](./LICENSE.md).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
