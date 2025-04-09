//! Test module for verifying BLS signature operations in the EigenLayer protocol.
//! 
//! This module tests the compatibility between our Rust implementation and EigenLayer's
//! Solidity implementation of BN254 curve operations, specifically focusing on the
//! operator registration process where:
//! 1. EigenLayer provides a G1 point for the operator to sign
//! 2. The operator uses their private key to create a signature via scalar multiplication
//! 3. The signature is verified on-chain
//! 
//! The test uses real test vectors from a successful operator registration transaction
//! to ensure our Rust implementation produces exactly the same results as the Solidity contract.

mod solidity;

use ark_bn254::{Fq, Fr, G1Affine, G1Projective};
use ark_ec::CurveGroup;
use ark_ff::{BigInteger, PrimeField};
use hex;
use serde::Deserialize;
use solidity::{call_scalar_mul_solidity, deploy_bn254_wrapper};
use std::{error::Error, fs, ops::Mul};

/// Wrapper structure for deserializing the test data JSON file
#[derive(Debug, Deserialize)]
struct Wrapper {
    for_testing: ForTesting,
}

/// Container for the test input values
#[derive(Debug, Deserialize)]
struct ForTesting {
    value: SignatureTestInput,
}

/// Test input data structure containing all the values needed to verify
/// the BLS signature operation
#[derive(Debug, Deserialize)]
struct SignatureTestInput {
    /// Operator's G1 point
    #[allow(dead_code)]
    g1: [String; 2],
    /// Operator's G2 point
    #[allow(dead_code)]
    g2: [[String; 2]; 2],
    /// Operator's private key
    priv_key: String,
    /// The expected signature output from the transaction
    sig_out: String,
    /// The G1 point that the operator needs to sign (message hash)
    /// This is the same G1 point that Eigen Labs provides to the operator
    /// during the BLS handshake
    call_pubkey_registration_message_hash_result: String,
}

/// Converts a hexadecimal string to a field element in Fq
/// 
/// # Arguments
/// * `s` - Hex string, optionally prefixed with "0x"
/// 
/// # Returns
/// * `Result<Fq, Box<dyn Error>>` - The field element or an error
#[allow(dead_code)]
fn fq_from_hex(s: &str) -> Result<Fq, Box<dyn Error>> {
    let bytes = hex::decode(s.trim_start_matches("0x"))?;
    Ok(Fq::from_be_bytes_mod_order(&bytes))
}

/// Converts a hexadecimal string to a scalar field element in Fr
/// 
/// # Arguments
/// * `s` - Hex string, optionally prefixed with "0x"
/// 
/// # Returns
/// * `Result<Fr, Box<dyn Error>>` - The scalar field element or an error
fn fr_from_hex(s: &str) -> Result<Fr, Box<dyn Error>> {
    let bytes = hex::decode(s.trim_start_matches("0x"))?;
    Ok(Fr::from_be_bytes_mod_order(&bytes))
}

/// Converts an ABI-encoded G1 point from hex to affine coordinates
/// 
/// The ABI encoding is a 64-byte array containing:
/// - bytes 0-31: X coordinate
/// - bytes 32-63: Y coordinate
/// 
/// # Arguments
/// * `s` - Hex string of the ABI-encoded point
/// 
/// # Returns
/// * `Result<G1Affine, Box<dyn Error>>` - The point in affine coordinates or an error
fn g1_from_abi_encoded_hex(s: &str) -> Result<G1Affine, Box<dyn Error>> {
    let bytes = hex::decode(s.trim_start_matches("0x"))?;
    if bytes.len() != 64 {
        return Err("Expected 64-byte ABI-encoded G1Point".into());
    }
    let x = Fq::from_be_bytes_mod_order(&bytes[..32]);
    let y = Fq::from_be_bytes_mod_order(&bytes[32..]);
    Ok(G1Affine::new_unchecked(x, y))
}

/// Formats a G1 point for debug output
/// 
/// # Arguments
/// * `point` - The G1 point to format
/// 
/// # Returns
/// A string containing the X and Y coordinates in hex
fn format_g1_point(point: &G1Affine) -> String {
    let x_hex = hex::encode(point.x.into_bigint().to_bytes_be());
    let y_hex = hex::encode(point.y.into_bigint().to_bytes_be());
    format!("\n  X = 0x{}\n  Y = 0x{}", x_hex, y_hex)
}

/// Tests that our Rust scalar multiplication matches both:
/// 1. The Solidity contract's direct output
/// 2. The actual transaction output from EigenLayer
///
/// This test verifies that our implementation can reproduce the exact same
/// signature that was accepted by the EigenLayer contract in a real transaction.
#[tokio::test]
async fn test_scalar_mul_signature_from_txtx() -> Result<(), Box<dyn Error>> {
    let data = fs::read_to_string("testdata/sign.json")?;
    let wrapper: Wrapper = serde_json::from_str(&data)?;
    let input = wrapper.for_testing.value;

    // The G1 point shared between Eigen Labs and the contract during the initial BLS handshake
    let eigen_labs_g1_point = g1_from_abi_encoded_hex(&input.call_pubkey_registration_message_hash_result)?;

    // The Operator's private key used for signing
    let priv_key = fr_from_hex(&input.priv_key)?;

    // Calculate the expected signature by performing scalar multiplication
    // of the Eigen Labs G1 point with the Operator's private key
    let expected = G1Projective::from(eigen_labs_g1_point).mul(priv_key).into_affine();
    println!(
        "--- Scalar multiplication result rust ---\n{}",
        format_g1_point(&expected)
    );

    // Deploy the contract once and reuse it
    let (_anvil, contract, _client) = deploy_bn254_wrapper().await?;
    let sol_point = call_scalar_mul_solidity(&contract, eigen_labs_g1_point, priv_key).await?;

    println!(
        "--- Solidity result (via contract) ---\n{}",
        format_g1_point(&sol_point)
    );

    assert_eq!(expected, sol_point, "Mismatch: Rust vs Solidity contract");

    let sig_out_point = g1_from_abi_encoded_hex(&input.sig_out)?;
    println!(
        "\n--- sig_out decoded G1 ---{}",
        format_g1_point(&sig_out_point)
    );

    if expected != sig_out_point {
        panic!(
            "Mismatch with sig_out:\nExpected (Rust result): {}\nActual (sig_out): {}",
            format_g1_point(&expected),
            format_g1_point(&sig_out_point),
        );
    }

    Ok(())
}
