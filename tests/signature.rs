use ark_bn254::{Fq, Fr, G1Affine, G1Projective};
use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ethers::abi::{decode, ParamType, Token};
use hex;
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::ops::Mul;
use std::str::FromStr;

/// This test deserializes a scalar multiplication output from a Solidity contract
/// and checks whether the result matches the same operation performed in Rust using arkworks.
///
/// FORMAT ASSUMPTIONS:
/// - `priv_key`, `g1_x`, `g1_y` are uint256 **decimal strings**
/// - `sig_out.value` is the hex-encoded `.result` from a Solidity function returning `(uint256 x, uint256 y)`
/// - `pubkey_registration_message_hash.value` is hex-encoded JSON (UTF-8) of `Vec<Vec<u8>>`

#[derive(Debug, Deserialize)]
struct SignatureTestInput {
    priv_key: String,
    g1_x: String,
    g1_y: String,
    sig_out: HexJsonBlob,
    pubkey_registration_message_hash: HexJsonBlob,
}

#[derive(Debug, Deserialize)]
struct HexJsonBlob {
    value: String,
}

/// Parse Fr from decimal string (Solidity uint256)
fn parse_decimal_fr(s: &str) -> Result<Fr, Box<dyn Error>> {
    Fr::from_str(s).map_err(|_| "Invalid decimal Fr".into())
}

/// Parse Fq from decimal string (Solidity uint256)
fn parse_decimal_fq(s: &str) -> Result<Fq, Box<dyn Error>> {
    Fq::from_str(s).map_err(|_| "Invalid decimal Fq".into())
}

/// Decode ABI-encoded `(uint256 x, uint256 y)` returned from Solidity as `G1Point`
fn decode_abi_g1_point(hex_str: &str) -> Result<G1Affine, Box<dyn Error>> {
    let raw = hex::decode(hex_str.trim_start_matches("0x"))?;
    let tokens = decode(&[ParamType::Uint(256), ParamType::Uint(256)], &raw)?;

    let x = match &tokens[0] {
        Token::Uint(x) => {
            let mut buf = [0u8; 32];
            x.to_big_endian(&mut buf);
            Fq::from_be_bytes_mod_order(&buf)
        }
        _ => return Err("Expected Uint for X".into()),
    };

    let y = match &tokens[1] {
        Token::Uint(y) => {
            let mut buf = [0u8; 32];
            y.to_big_endian(&mut buf);
            Fq::from_be_bytes_mod_order(&buf)
        }
        _ => return Err("Expected Uint for Y".into()),
    };

    Ok(G1Affine::new_unchecked(x, y))
}

/// Decode hex string containing UTF-8 JSON-encoded nested Vec<Vec<u8>>
fn parse_nested_u8_array_from_hex_json(s: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let decoded = hex::decode(s.trim_start_matches("0x"))?;
    let json_str = std::str::from_utf8(&decoded)?;
    let nested: Vec<Vec<u8>> = serde_json::from_str(json_str)?;
    Ok(nested.into_iter().flatten().collect())
}

#[test]
fn test_scalar_mul_signature() -> Result<(), Box<dyn Error>> {
    // Read JSON input exported from txtx runbook
    let data = fs::read_to_string("testdata/sign.json")?;
    let input: SignatureTestInput = serde_json::from_str(&data)?;

    // Parse scalar and coordinates
    let priv_key = parse_decimal_fr(&input.priv_key)?;
    let g1_x = parse_decimal_fq(&input.g1_x)?;
    let g1_y = parse_decimal_fq(&input.g1_y)?;
    let point = G1Affine::new_unchecked(g1_x, g1_y);

    // Scalar multiplication in Rust
    let result = G1Projective::from(point).mul(priv_key).into_affine();
    println!("\n--- Scalar mul result ---\n{:?}\n", result);

    // Decode Solidity output as ABI-encoded G1Point (uint256 x, uint256 y)
    let expected = decode_abi_g1_point(&input.sig_out.value)?;
    println!(
        "--- Solidity sig_out (decoded G1Point) ---\n{:?}\n",
        expected
    );

    // Validate
    assert_eq!(result, expected, "Signature mismatch");

    // Optional sanity check: decode the pubkey registration message hash
    let _message_hash =
        parse_nested_u8_array_from_hex_json(&input.pubkey_registration_message_hash.value)?;

    Ok(())
}
