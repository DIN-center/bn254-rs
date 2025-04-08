mod solidity;

use ark_bn254::{Fq, Fr, G1Affine, G1Projective};
use ark_ec::CurveGroup;
use ark_ff::{BigInteger, PrimeField};
use hex;
use serde::Deserialize;
use solidity::{call_scalar_mul_solidity, deploy_bn254_wrapper};
use std::{error::Error, fs, ops::Mul};

use bn254_rs::G1Point;

#[derive(Debug, Deserialize)]
struct Wrapper {
    for_testing: ForTesting,
}

#[derive(Debug, Deserialize)]
struct ForTesting {
    value: SignatureTestInput,
}

#[derive(Debug, Deserialize)]
struct SignatureTestInput {
    g1: [String; 2],
    #[allow(dead_code)]
    g2: [[String; 2]; 2],
    priv_key: String,
    sig_out: String,
    #[allow(dead_code)]
    call_pubkey_registration_message_hash_result: String,
}

fn fq_from_hex(s: &str) -> Result<Fq, Box<dyn Error>> {
    let bytes = hex::decode(s.trim_start_matches("0x"))?;
    Ok(Fq::from_be_bytes_mod_order(&bytes))
}

fn fr_from_hex(s: &str) -> Result<Fr, Box<dyn Error>> {
    let bytes = hex::decode(s.trim_start_matches("0x"))?;
    Ok(Fr::from_be_bytes_mod_order(&bytes))
}

fn g1_from_abi_encoded_hex(s: &str) -> Result<G1Affine, Box<dyn Error>> {
    let bytes = hex::decode(s.trim_start_matches("0x"))?;
    if bytes.len() != 64 {
        return Err("Expected 64-byte ABI-encoded G1Point".into());
    }
    let x = Fq::from_be_bytes_mod_order(&bytes[..32]);
    let y = Fq::from_be_bytes_mod_order(&bytes[32..]);
    Ok(G1Affine::new_unchecked(x, y))
}

fn format_g1_point(point: &G1Affine) -> String {
    let x_hex = hex::encode(point.x.into_bigint().to_bytes_be());
    let y_hex = hex::encode(point.y.into_bigint().to_bytes_be());
    format!("\n  X = 0x{}\n  Y = 0x{}", x_hex, y_hex)
}

#[tokio::test]
async fn test_scalar_mul_signature_from_txtx() -> Result<(), Box<dyn Error>> {
    let data = fs::read_to_string("testdata/sign.json")?;
    let wrapper: Wrapper = serde_json::from_str(&data)?;
    let input = wrapper.for_testing.value;

    let g1_x = fq_from_hex(&input.g1[0])?;
    let g1_y = fq_from_hex(&input.g1[1])?;
    let priv_key = fr_from_hex(&input.priv_key)?;
    let base_point = G1Point::from_projective(G1Projective::from(G1Affine::new_unchecked(g1_x, g1_y)));
    let expected = G1Projective::from(base_point.inner().into_affine()).mul(priv_key).into_affine();

    println!(
        "--- Scalar multiplication result rust ---\n{}",
        format_g1_point(&expected)
    );

    // Deploy the contract once and reuse it
    let (_anvil, contract, _client) = deploy_bn254_wrapper().await?;
    let sol_point = call_scalar_mul_solidity(&contract, base_point.inner().into_affine(), priv_key).await?;

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
