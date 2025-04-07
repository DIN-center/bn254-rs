//! Module for operations on the G1 group of the BN254 elliptic curve.
//! 
//! This module provides basic operations for working with points in the G1 group:
//! - Getting the generator point
//! - Point negation
//! - Point addition
//! - Scalar multiplication
//! 
//! The BN254 curve is a pairing-friendly elliptic curve that is widely used in
//! zero-knowledge proof systems and other cryptographic applications.

use ark_bn254::{G1Projective, Fr};
use ark_ec::Group;

/// Returns the generator point of the G1 group in BN254 curve.
/// This is a fixed point that can be used to generate all other points in G1
/// through scalar multiplication.
pub fn g1_generator() -> G1Projective {
    G1Projective::generator()
}

/// Negates a point in the G1 group.
/// In elliptic curve arithmetic, negation of a point (x,y) is (x,-y).
/// 
/// # Arguments
/// * `p` - The point to negate
/// 
/// # Returns
/// The negation of the input point
pub fn g1_negate(p: G1Projective) -> G1Projective {
    -p
}

/// Adds two points in the G1 group.
/// Implements the group operation for points on the BN254 curve.
/// 
/// # Arguments
/// * `p1` - First point to add
/// * `p2` - Second point to add
/// 
/// # Returns
/// The sum of the two input points
pub fn g1_add(p1: G1Projective, p2: G1Projective) -> G1Projective {
    p1 + p2
}

/// Performs scalar multiplication of a point in G1.
/// Multiplies a point by a scalar value (field element) using the
/// double-and-add algorithm.
/// 
/// # Arguments
/// * `p` - The point to multiply
/// * `s` - The scalar value to multiply by
/// 
/// # Returns
/// The result of the scalar multiplication
pub fn g1_scalar_mul(p: G1Projective, s: Fr) -> G1Projective {
    p * s
}

