//! A Rust implementation of BN254 curve operations that mirrors EigenLayer's BN254.sol library.
//! 
//! This library provides a pure Rust implementation of the BN254 elliptic curve operations,
//! designed to be compatible with EigenLayer's [BN254.sol](https://github.com/Layr-Labs/eigenlayer-middleware/blob/dev/src/libraries/BN254.sol) Solidity library.
//! 
//! # Examples
//! 
//! ```
//! use bn254_rs::{G1Point, G2Point, pairing_check};
//! use ark_bn254::Fr;
//! 
//! // Get generator points
//! let g1 = G1Point::generator();
//! let g2 = G2Point::generator();
//! 
//! // Perform scalar multiplication
//! let scalar = Fr::from(2u64);
//! let doubled = g1.scalar_mul(scalar);
//! 
//! // Perform a pairing check
//! let result = pairing_check(g1, g2, g1.negate(), g2);
//! assert!(result);
//! ```

pub mod g1;
pub mod g2;
pub mod pairing;
pub mod hash;
pub mod utils;
pub mod web;

// Re-export the main types
pub use g1::{G1Point, g1_generator, g1_negate, g1_add, g1_scalar_mul};
pub use g2::{G2Point, g2_generator, g2_negate};
pub use pairing::{pairing_check, pairing_check_raw};
pub use hash::{hash_g1_point, hash_g1_point_raw};
pub use utils::fr_to_be_bytes;

