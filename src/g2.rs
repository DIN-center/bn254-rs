//! Module for operations on the G2 group of the BN254 elliptic curve.
//! 
//! This module provides basic operations for working with points in the G2 group:
//! - Getting the generator point
//! - Point negation
//! 
//! The G2 group is the second group in the BN254 pairing-friendly elliptic curve,
//! which is used in conjunction with G1 for bilinear pairings.
//! 
//! # Examples
//! 
//! ```
//! use bn254_rs::G2Point;
//! 
//! // Get the generator point
//! let g2 = G2Point::generator();
//! 
//! // Negate a point
//! let neg_g2 = g2.negate();
//! ```

use ark_bn254::G2Projective;
use ark_ec::Group;

/// A point on the G2 group of the BN254 curve.
/// 
/// This type wraps the underlying `G2Projective` type from the ark-bn254 crate
/// and provides methods for common operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct G2Point(pub(crate) G2Projective);

impl G2Point {
    /// Returns the generator point of the G2 group in BN254 curve.
    /// This is a fixed point that can be used to generate all other points in G2
    /// through scalar multiplication.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bn254_rs::G2Point;
    /// 
    /// let g2 = G2Point::generator();
    /// ```
    pub fn generator() -> Self {
        Self(G2Projective::generator())
    }

    /// Creates a new G2Point from a G2Projective.
    /// 
    /// This is primarily used in tests and for advanced operations.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bn254_rs::G2Point;
    /// use ark_bn254::G2Projective;
    /// 
    /// let g = G2Projective::generator();
    /// let point = G2Point::from_projective(g);
    /// ```
    pub fn from_projective(p: G2Projective) -> Self {
        Self(p)
    }

    /// Negates a point in the G2 group.
    /// In elliptic curve arithmetic, negation of a point (x,y) is (x,-y).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bn254_rs::G2Point;
    /// 
    /// let g2 = G2Point::generator();
    /// let neg_g2 = g2.negate();
    /// ```
    pub fn negate(&self) -> Self {
        Self(-self.0)
    }

    /// Returns the underlying G2Projective point.
    /// 
    /// This is primarily used internally and for advanced operations.
    pub fn inner(&self) -> &G2Projective {
        &self.0
    }
}

// For backward compatibility
pub fn g2_generator() -> G2Projective {
    G2Point::generator().0
}

pub fn g2_negate(p: G2Projective) -> G2Projective {
    G2Point(p).negate().0
}
