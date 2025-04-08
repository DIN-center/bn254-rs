use ark_bn254::{Fq, Fr, G1Affine, G1Projective};
use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use hex;
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::ops::Mul;

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
    g2: [[String; 2]; 2],
    priv_key: String,
    sig_out: String,
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

#[test]
fn test_scalar_mul_signature_from_txtx() -> Result<(), Box<dyn Error>> {
    let data = fs::read_to_string("testdata/sign.json")?;
    let wrapper: Wrapper = serde_json::from_str(&data)?;
    let input = wrapper.for_testing.value;

    let g1_x = fq_from_hex(&input.g1[0])?;
    let g1_y = fq_from_hex(&input.g1[1])?;
    let priv_key = fr_from_hex(&input.priv_key)?;
    let base_point = G1Affine::new_unchecked(g1_x, g1_y);
    let expected = G1Projective::from(base_point).mul(priv_key).into_affine();

    println!("--- Scalar multiplication result ---\n{:?}", expected);

    let sig_out_point = g1_from_abi_encoded_hex(&input.sig_out)?;
    let call_result_point =
        g1_from_abi_encoded_hex(&input.call_pubkey_registration_message_hash_result)?;

    println!("\n--- sig_out decoded G1 ---\n{:?}", sig_out_point);
    println!("--- call_result decoded G1 ---\n{:?}", call_result_point);

    assert_eq!(expected, sig_out_point, "Mismatch with sig_out");
    assert_eq!(
        expected, call_result_point,
        "Mismatch with call_pubkey_registration_message_hash_result"
    );

    Ok(())
}
