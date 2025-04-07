//! Module for hashing operations on BN254 curve points.
//! 
//! This module provides functions for hashing points in the G1 group of the BN254 curve.
//! It uses the Keccak-256 hash function (SHA-3) to produce a 32-byte hash output.

use ark_bn254::G1Projective;
use ark_ec::CurveGroup;
use ark_ff::{BigInteger, PrimeField}; 
use sha3::{Digest, Keccak256};

/// Hashes a G1 point to a 32-byte array using Keccak-256.
/// The hash is computed by concatenating the big-endian representations of
/// the point's x and y coordinates and hashing the result.
/// 
/// # Arguments
/// * `p` - The G1 point to hash
/// 
/// # Returns
/// A 32-byte array containing the hash of the point
pub fn hash_g1_point(p: &G1Projective) -> [u8; 32] {
    let aff = p.into_affine();
    let mut hasher = Keccak256::new();
    hasher.update(aff.x.into_bigint().to_bytes_be());
    hasher.update(aff.y.into_bigint().to_bytes_be());
    let result = hasher.finalize();
    result.into()
}
