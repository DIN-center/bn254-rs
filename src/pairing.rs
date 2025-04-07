//! Module for bilinear pairing operations on the BN254 curve.
//! 
//! This module provides functions for performing pairing checks on points
//! from the G1 and G2 groups of the BN254 curve. The pairing operation
//! is a bilinear map that takes points from G1 and G2 and produces an
//! element in the target field.

use ark_bn254::{Bn254, G1Projective, G2Projective};
use ark_ec::pairing::{Pairing};
use ark_ff::One;

/// Performs a pairing check between two pairs of points.
/// 
/// This function checks if e(a1, a2) * e(b1, b2) = 1, where e is the
/// bilinear pairing operation. This is equivalent to checking if
/// e(a1, a2) = e(-b1, b2).
/// 
/// # Arguments
/// * `a1` - First point from G1
/// * `a2` - First point from G2
/// * `b1` - Second point from G1
/// * `b2` - Second point from G2
/// 
/// # Returns
/// `true` if the pairing check passes, `false` otherwise
pub fn pairing_check(
    a1: G1Projective,
    a2: G2Projective,
    b1: G1Projective,
    b2: G2Projective,
) -> bool {
    let p1 = Bn254::pairing(a1, a2).0;
    let p2 = Bn254::pairing(b1, b2).0;

    p1 * p2 == <Bn254 as Pairing>::TargetField::one()
}

