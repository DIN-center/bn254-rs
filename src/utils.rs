//! Module for utility functions related to BN254 curve operations.
//! 
//! This module provides helper functions for working with field elements
//! and their byte representations in the BN254 curve.

use ark_ff::{BigInteger, PrimeField};
use ark_bn254::Fr;

/// Converts a field element to a 32-byte array in big-endian format.
/// 
/// This function takes a field element and returns its big-endian byte
/// representation, padded to 32 bytes if necessary.
/// 
/// # Arguments
/// * `f` - The field element to convert
/// 
/// # Returns
/// A 32-byte array containing the big-endian representation of the field element
pub fn fr_to_be_bytes(f: &Fr) -> [u8; 32] {
    let mut out = [0u8; 32];
    let bytes = f.into_bigint().to_bytes_be();
    out[32 - bytes.len()..].copy_from_slice(&bytes);
    out
}
