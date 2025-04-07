//! Module for operations on the G2 group of the BN254 elliptic curve.
//! 
//! This module provides basic operations for working with points in the G2 group:
//! - Getting the generator point
//! - Point negation
//! 
//! The G2 group is the second group in the BN254 pairing-friendly elliptic curve,
//! which is used in conjunction with G1 for bilinear pairings.

use ark_bn254::G2Projective;
use ark_ec::Group;

/// Returns the generator point of the G2 group in BN254 curve.
/// This is a fixed point that can be used to generate all other points in G2
/// through scalar multiplication.
/// 
/// # Returns
/// The generator point of the G2 group
pub fn g2_generator() -> G2Projective {
    G2Projective::generator()
}

/// Negates a point in the G2 group.
/// In elliptic curve arithmetic, negation of a point (x,y) is (x,-y).
/// 
/// # Arguments
/// * `p` - The point to negate
/// 
/// # Returns
/// The negation of the input point
pub fn g2_negate(p: G2Projective) -> G2Projective {
    -p
}
