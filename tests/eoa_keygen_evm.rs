//! Test module for EOA-based key derivation with EVM precompile verification
//!
//! This test demonstrates the full flow:
//! 1. Derive BLS keys from an EOA address using HKDF
//! 2. Sign a message (perform scalar multiplication on a G1 point)
//! 3. Verify the signature using the EVM pairing precompile via Solidity
//!
//! The verification uses the BLS signature pairing check:
//! e(signature, G2_generator) = e(message_hash, pubkey_G2)
//!
//! Which is equivalent to checking:
//! e(signature, G2_generator) * e(-message_hash, pubkey_G2) = 1

mod solidity;

use ark_bn254::{Fq, Fr, G1Affine, G1Projective, G2Affine};
use ark_ec::{AffineRepr, CurveGroup, Group};
use ark_ff::PrimeField;
use bn254_rs::keygen::derive_key_from_eoa;
use ethers::prelude::*;
use solidity::{deploy_bn254_wrapper, fq_to_u256};
use std::ops::{Mul, Neg};
use std::str::FromStr;

/// Converts a decimal string to Fr
fn fr_from_decimal_str(s: &str) -> anyhow::Result<Fr> {
    Fr::from_str(s).map_err(|e| anyhow::anyhow!("Failed to parse Fr: {:?}", e))
}

/// Converts a decimal string to Fq
fn fq_from_decimal_str(s: &str) -> anyhow::Result<Fq> {
    Fq::from_str(s).map_err(|e| anyhow::anyhow!("Failed to parse Fq: {:?}", e))
}

/// Calls the pairing precompile via the BN254Wrapper contract
async fn verify_signature_with_evm(
    contract: &ethers::contract::Contract<SignerMiddleware<Provider<Http>, LocalWallet>>,
    signature: G1Affine,
    message_hash: G1Affine,
    pubkey_g2: G2Affine,
) -> anyhow::Result<bool> {
    // Get the G2 generator
    let g2_gen = G2Affine::generator();

    // Convert signature to U256
    let sig_x = fq_to_u256(signature.x);
    let sig_y = fq_to_u256(signature.y);

    // Convert G2 generator to U256 arrays
    // Ethereum expects (x1, x0) and (y1, y0) ordering for G2 points
    let g2_gen_x = [
        fq_to_u256(g2_gen.x.c1), // x1 (imaginary part)
        fq_to_u256(g2_gen.x.c0), // x0 (real part)
    ];
    let g2_gen_y = [
        fq_to_u256(g2_gen.y.c1), // y1 (imaginary part)
        fq_to_u256(g2_gen.y.c0), // y0 (real part)
    ];

    // Negate message_hash for the pairing check
    let neg_message_hash = message_hash.into_group().neg().into_affine();
    let neg_msg_x = fq_to_u256(neg_message_hash.x);
    let neg_msg_y = fq_to_u256(neg_message_hash.y);

    // Convert pubkey_g2 to U256 arrays
    // Ethereum expects (x1, x0) and (y1, y0) ordering for G2 points
    let pk_x = [
        fq_to_u256(pubkey_g2.x.c1), // x1 (imaginary part)
        fq_to_u256(pubkey_g2.x.c0), // x0 (real part)
    ];
    let pk_y = [
        fq_to_u256(pubkey_g2.y.c1), // y1 (imaginary part)
        fq_to_u256(pubkey_g2.y.c0), // y0 (real part)
    ];

    // Call pairing(signature, G2_gen, -message_hash, pubkey_g2)
    // This checks: e(signature, G2_gen) * e(-message_hash, pubkey_g2) = 1
    let result: bool = contract
        .method(
            "pairing",
            (
                (sig_x, sig_y),  // signature (G1)
                (g2_gen_x, g2_gen_y), // G2 generator
                (neg_msg_x, neg_msg_y), // -message_hash (G1)
                (pk_x, pk_y),    // pubkey_g2 (G2)
            ),
        )?
        .call()
        .await?;

    Ok(result)
}

#[tokio::test]
async fn test_eoa_keygen_sign_and_verify_with_evm() -> anyhow::Result<()> {
    println!("\n=== EOA Key Derivation + EVM Signature Verification Test ===\n");

    // Step 1: Derive keys from an EOA address
    let eoa_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"; // Hardhat default account
    let secret_salt = "test-salt-for-evm-verification";

    println!("Step 1: Deriving keys from EOA");
    println!("  EOA Address: {}", eoa_address);
    println!("  Secret Salt: {}", secret_salt);

    let keypair = derive_key_from_eoa(eoa_address, secret_salt)
        .map_err(|e| anyhow::anyhow!("Key derivation failed: {}", e))?;

    println!("\nDerived Keys:");
    println!("  Private Key: {}", keypair.private_key);
    println!("  Public Key G1:");
    println!("    X: {}", keypair.public_key_g1.x);
    println!("    Y: {}", keypair.public_key_g1.y);
    println!("  Public Key G2:");
    println!("    X: [{}, {}]", keypair.public_key_g2.x_a, keypair.public_key_g2.x_b);
    println!("    Y: [{}, {}]", keypair.public_key_g2.y_a, keypair.public_key_g2.y_b);

    // Step 2: Parse the keys to curve points
    let private_key = fr_from_decimal_str(&keypair.private_key)?;

    let pubkey_g1_x = fq_from_decimal_str(&keypair.public_key_g1.x)?;
    let pubkey_g1_y = fq_from_decimal_str(&keypair.public_key_g1.y)?;
    let pubkey_g1 = G1Affine::new_unchecked(pubkey_g1_x, pubkey_g1_y);

    let pubkey_g2_x0 = fq_from_decimal_str(&keypair.public_key_g2.x_a)?;
    let pubkey_g2_x1 = fq_from_decimal_str(&keypair.public_key_g2.x_b)?;
    let pubkey_g2_y0 = fq_from_decimal_str(&keypair.public_key_g2.y_a)?;
    let pubkey_g2_y1 = fq_from_decimal_str(&keypair.public_key_g2.y_b)?;

    use ark_bn254::Fq2;
    let pubkey_g2_x = Fq2::new(pubkey_g2_x0, pubkey_g2_x1);
    let pubkey_g2_y = Fq2::new(pubkey_g2_y0, pubkey_g2_y1);
    let pubkey_g2 = G2Affine::new_unchecked(pubkey_g2_x, pubkey_g2_y);

    // Verify that the derived public key is correct
    let expected_pubkey_g1 = G1Projective::generator().mul(private_key).into_affine();
    assert_eq!(pubkey_g1, expected_pubkey_g1, "G1 public key mismatch");
    println!("\n✓ G1 public key derivation verified");

    // Step 3: Create a message to sign (use a simple hash value)
    let message = "Hello, EVM!";
    let message_bytes = message.as_bytes();
    let message_scalar = Fr::from_be_bytes_mod_order(message_bytes);

    println!("\nStep 2: Creating message to sign");
    println!("  Message: \"{}\"", message);
    println!("  Message Scalar: {:?}", message_scalar.into_bigint());

    // Hash to G1 point (message_hash = message_scalar * G1_generator)
    let message_hash = G1Projective::generator().mul(message_scalar).into_affine();

    println!("  Message Hash (G1 point):");
    println!("    X: 0x{:064x}", fq_to_u256(message_hash.x));
    println!("    Y: 0x{:064x}", fq_to_u256(message_hash.y));

    // Step 4: Sign the message (signature = private_key * message_hash)
    println!("\nStep 3: Signing the message");
    let signature = G1Projective::from(message_hash).mul(private_key).into_affine();

    println!("  Signature (G1 point):");
    println!("    X: 0x{:064x}", fq_to_u256(signature.x));
    println!("    Y: 0x{:064x}", fq_to_u256(signature.y));

    // Step 5: Deploy contract and verify signature using EVM pairing precompile
    println!("\nStep 4: Deploying BN254Wrapper contract");
    let (_anvil, contract, _) = deploy_bn254_wrapper().await?;
    println!("  Contract deployed at: {}", contract.address());

    println!("\nStep 5: Verifying signature with EVM pairing precompile");
    println!("  Checking: e(signature, G2_gen) = e(message_hash, pubkey_G2)");

    let is_valid = verify_signature_with_evm(&contract, signature, message_hash, pubkey_g2).await?;

    println!("\n=== Verification Result ===");
    println!("  Signature Valid: {}", is_valid);

    assert!(is_valid, "Signature verification failed!");
    println!("\n✓ Signature verified successfully using EVM pairing precompile!");

    // Step 6: Test with invalid signature (should fail)
    println!("\nStep 6: Testing with invalid signature (negative test)");
    let invalid_signature = G1Projective::generator().mul(Fr::from(999u64)).into_affine();
    let is_invalid = verify_signature_with_evm(&contract, invalid_signature, message_hash, pubkey_g2).await?;

    println!("  Invalid Signature Valid: {}", is_invalid);
    assert!(!is_invalid, "Invalid signature should not verify!");
    println!("✓ Invalid signature correctly rejected!");

    println!("\n=== Test Completed Successfully ===\n");

    Ok(())
}

#[tokio::test]
async fn test_multiple_eoa_signatures() -> anyhow::Result<()> {
    println!("\n=== Multiple EOA Signature Test ===\n");

    // Deploy contract once for all tests
    let (_anvil, contract, _) = deploy_bn254_wrapper().await?;
    println!("Contract deployed at: {}\n", contract.address());

    let test_cases = vec![
        ("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", "salt-1", "Message 1"),
        ("0x70997970C51812dc3A010C7d01b50e0d17dc79C8", "salt-2", "Message 2"),
        ("0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC", "salt-3", "Message 3"),
    ];

    for (i, (eoa, salt, message)) in test_cases.iter().enumerate() {
        println!("Test Case #{}: EOA = {}", i + 1, eoa);

        // Derive keys
        let keypair = derive_key_from_eoa(eoa, salt)
            .map_err(|e| anyhow::anyhow!("Key derivation failed: {}", e))?;
        let private_key = fr_from_decimal_str(&keypair.private_key)?;

        let pubkey_g2_x0 = fq_from_decimal_str(&keypair.public_key_g2.x_a)?;
        let pubkey_g2_x1 = fq_from_decimal_str(&keypair.public_key_g2.x_b)?;
        let pubkey_g2_y0 = fq_from_decimal_str(&keypair.public_key_g2.y_a)?;
        let pubkey_g2_y1 = fq_from_decimal_str(&keypair.public_key_g2.y_b)?;

        use ark_bn254::Fq2;
        let pubkey_g2_x = Fq2::new(pubkey_g2_x0, pubkey_g2_x1);
        let pubkey_g2_y = Fq2::new(pubkey_g2_y0, pubkey_g2_y1);
        let pubkey_g2 = G2Affine::new_unchecked(pubkey_g2_x, pubkey_g2_y);

        // Create message hash
        let message_scalar = Fr::from_be_bytes_mod_order(message.as_bytes());
        let message_hash = G1Projective::generator().mul(message_scalar).into_affine();

        // Sign
        let signature = G1Projective::from(message_hash).mul(private_key).into_affine();

        // Verify
        let is_valid = verify_signature_with_evm(&contract, signature, message_hash, pubkey_g2).await?;

        println!("  Message: \"{}\"", message);
        println!("  Signature Valid: {}", is_valid);
        assert!(is_valid, "Signature verification failed for test case #{}", i + 1);
        println!("  ✓ Verified\n");
    }

    println!("=== All Test Cases Passed ===\n");

    Ok(())
}
