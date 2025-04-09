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
//! 
//! # Examples
//! 
//! ```
//! use bn254_rs::G1Point;
//! use ark_bn254::Fr;
//! 
//! // Get the generator point
//! let g = G1Point::generator();
//! 
//! // Negate a point
//! let neg_g = g.negate();
//! 
//! // Add two points
//! let sum = g.add(&g);
//! 
//! // Scalar multiplication
//! let scalar = Fr::from(2u64);
//! let doubled = g.scalar_mul(scalar);
//! ```

use ark_bn254::{G1Projective, Fr};
use ark_ec::Group;
use ark_ff::Zero;

/// A point on the G1 group of the BN254 curve.
/// 
/// This type wraps the underlying `G1Projective` type from the ark-bn254 crate
/// and provides methods for common operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct G1Point(pub(crate) G1Projective);

impl G1Point {
    /// Returns the generator point of the G1 group in BN254 curve.
    /// This is a fixed point that can be used to generate all other points in G1
    /// through scalar multiplication.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bn254_rs::G1Point;
    /// 
    /// let g = G1Point::generator();
    /// ```
    pub fn generator() -> Self {
        Self(G1Projective::generator())
    }

    /// Creates a new G1Point from a G1Projective.
    /// 
    /// This is primarily used in tests and for advanced operations.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bn254_rs::G1Point;
    /// use ark_bn254::G1Projective;
    /// use ark_ec::Group;
    /// 
    /// let g = G1Projective::generator();
    /// let point = G1Point::from_projective(g);
    /// ```
    pub fn from_projective(p: G1Projective) -> Self {
        Self(p)
    }

    /// Negates a point in the G1 group.
    /// In elliptic curve arithmetic, negation of a point (x,y) is (x,-y).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bn254_rs::G1Point;
    /// 
    /// let g = G1Point::generator();
    /// let neg_g = g.negate();
    /// ```
    pub fn negate(&self) -> Self {
        Self(-self.0)
    }

    /// Adds two points in the G1 group.
    /// Implements the group operation for points on the BN254 curve.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bn254_rs::G1Point;
    /// 
    /// let g = G1Point::generator();
    /// let sum = g.add(&g);
    /// ```
    pub fn add(&self, other: &Self) -> Self {
        Self(self.0 + other.0)
    }

    /// Performs scalar multiplication of a point in G1.
    /// Multiplies a point by a scalar value (field element) using the
    /// double-and-add algorithm.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bn254_rs::G1Point;
    /// use ark_bn254::Fr;
    /// 
    /// let g = G1Point::generator();
    /// let scalar = Fr::from(2u64);
    /// let doubled = g.scalar_mul(scalar);
    /// ```
    pub fn scalar_mul(&self, scalar: Fr) -> Self {
        // Special case for zero scalar - return point at infinity (0,0)
        if scalar.is_zero() {
            Self(G1Projective::zero())
        } else {
            // For non-zero scalars, use the standard scalar multiplication
            Self(self.0 * scalar)
        }
    }

    /// Returns the underlying G1Projective point.
    /// 
    /// This is primarily used internally and for advanced operations.
    pub fn inner(&self) -> &G1Projective {
        &self.0
    }
}

// For backward compatibility
pub fn g1_generator() -> G1Projective {
    G1Point::generator().0
}

pub fn g1_negate(p: G1Projective) -> G1Projective {
    G1Point(p).negate().0
}

pub fn g1_add(p1: G1Projective, p2: G1Projective) -> G1Projective {
    G1Point(p1).add(&G1Point(p2)).0
}

pub fn g1_scalar_mul(p: G1Projective, s: Fr) -> G1Projective {
    G1Point(p).scalar_mul(s).0
}

