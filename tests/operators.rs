#![allow(dead_code)]
use std::fs;

use ark_bn254::{Fq, Fq2, G1Affine, G2Affine};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::PrimeField;
use num_bigint::BigUint;
use num_traits::Num;
use serde::Deserialize;

use bn254_rs::{G1Point, G2Point, pairing_check};

/// Structure representing a G1 point in JSON format.
/// Used for deserializing G1 points from JSON files.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct G1Json {
    #[serde(rename = "X")]
    x: String,
    #[serde(rename = "Y")]
    y: String,
}

/// Structure representing a G2 point in JSON format.
/// Used for deserializing G2 points from JSON files.
/// Note that G2 points have coordinates in Fq2, which are represented as arrays of two strings.
#[derive(Deserialize)]
struct G2Json {
    #[serde(rename = "X")]
    x: [String; 2],
    #[serde(rename = "Y")]
    y: [String; 2],
}

/// Structure representing a BLS wallet with private key and public keys.
/// Contains both G1 and G2 representations of the public key.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BlsWallet {
    private_key: String,
    public_key_g1: G1Json,
    public_key_g2: G2Json,
}

/// Structure representing a standard Ethereum wallet.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Wallet {
    private_key: String,
    address: String,
}

/// Structure representing an operator with both standard and BLS wallets.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Operator {
    wallet: Wallet,
    bls_wallet: BlsWallet,
}

/// Converts a decimal string to an Fq field element.
/// 
/// # Arguments
/// * `s` - A string representing a decimal number
/// 
/// # Returns
/// An Fq field element
fn parse_decimal_fq(s: &str) -> Fq {
    let int = BigUint::from_str_radix(s, 10).expect("Invalid decimal input");
    let bytes = int.to_bytes_be();
    let mut buf = [0u8; 32];
    buf[32 - bytes.len()..].copy_from_slice(&bytes);
    Fq::from_be_bytes_mod_order(&buf)
}

/// Converts a G1Json structure to a G1Point.
/// 
/// # Arguments
/// * `p` - A G1Json structure containing x and y coordinates as strings
/// 
/// # Returns
/// A G1Point
fn g1_from_json(p: &G1Json) -> G1Point {
    G1Point::from_projective(ark_bn254::G1Projective::from(G1Affine::new_unchecked(
        parse_decimal_fq(&p.x),
        parse_decimal_fq(&p.y)
    )))
}

/// Converts a G2Json structure to a G2Point.
/// 
/// # Important Note on Coordinate Ordering
/// 
/// There is a critical difference in how Fq2 coordinates are represented:
/// - Solidity/EVM: [imaginary, real]
/// - Arkworks: (real, imaginary)
/// 
/// This function handles the reordering of coordinates to ensure compatibility
/// between the two representations.
/// 
/// # Arguments
/// * `p` - A G2Json structure containing x and y coordinates as arrays of strings
/// 
/// # Returns
/// A G2Point
fn g2_from_json(p: &G2Json) -> G2Point {
    // Solidity gives [imaginary, real], Arkworks expects (real, imaginary)
    let x_c0 = parse_decimal_fq(&p.x[1]); // real part
    let x_c1 = parse_decimal_fq(&p.x[0]); // imaginary part
    let y_c0 = parse_decimal_fq(&p.y[1]);
    let y_c1 = parse_decimal_fq(&p.y[0]);

    G2Point::from_projective(ark_bn254::G2Projective::from(G2Affine::new(
        Fq2::new(x_c0, x_c1),
        Fq2::new(y_c0, y_c1)
    )))
}

/// Test function that verifies operator points from a JSON file.
/// 
/// This test:
/// 1. Loads operator data from a JSON file
/// 2. Converts G1 and G2 points from JSON format
/// 3. Verifies that points are on the curve
/// 4. Performs pairing checks to validate the BLS signature scheme
#[test]
fn test_operator_points() {
    let json = fs::read_to_string("testdata/operators.json").unwrap();
    let operators: Vec<Operator> = serde_json::from_str(&json).unwrap();

    for (i, op) in operators.iter().enumerate() {
        println!("Testing operator {}", i);
        let g1 = g1_from_json(&op.bls_wallet.public_key_g1);
        let g2 = g2_from_json(&op.bls_wallet.public_key_g2);

        println!("  G2 X: {:?}", op.bls_wallet.public_key_g2.x);
        println!("  G2 Y: {:?}", op.bls_wallet.public_key_g2.y);
        println!("  G2 is_on_curve: {}", g2.inner().into_affine().is_on_curve());

        assert!(g1.inner().into_affine().is_on_curve(), "G1 point {} not on curve", i);
        assert!(g2.inner().into_affine().is_on_curve(), "G2 point {} not on curve", i);
        assert!(!g1.inner().into_affine().is_zero(), "G1 point {} is at infinity", i);
        assert!(!g2.inner().into_affine().is_zero(), "G2 point {} is at infinity", i);

        // Verify the pairing check: e(g1, g2) * e(-g1, g2) = 1
        // This is a fundamental property of BLS signatures
        let g1_neg = g1.negate();
        let result = pairing_check(g1, g2, g1_neg, g2);

        assert!(result, "Pairing check failed at {}", i);
    }
}
