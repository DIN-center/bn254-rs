use ark_bn254::{Fq, Fq2, Fr, G1Affine, G2Affine};
use ark_ec::AffineRepr;
use ark_ff::{BigInteger, PrimeField};
use bn254_rs::keygen::derive_key_from_eoa;
use ethers::abi::Abi;
use ethers::contract::{Contract, ContractFactory};
use ethers::prelude::*;
use ethers::types::U256;
use ethers::utils::{Anvil, AnvilInstance};
use std::{str::FromStr, sync::Arc, time::Duration};

const VALIDATOR_ABI_JSON: &str = include_str!("../out/BN254KeyValidator.sol/BN254KeyValidator.json");

#[derive(serde::Deserialize)]
struct ContractArtifact {
    abi: Abi,
    bytecode: BytecodeObject,
}

#[derive(serde::Deserialize)]
struct BytecodeObject {
    object: String,
}

/// Deploys the BN254KeyValidator contract to a local Anvil instance
async fn deploy_key_validator() -> anyhow::Result<(
    AnvilInstance,
    Contract<SignerMiddleware<Provider<Http>, LocalWallet>>,
    Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
)> {
    let anvil = Anvil::new().spawn();
    let endpoint = anvil.endpoint();

    let wallet: LocalWallet =
        LocalWallet::from(anvil.keys()[0].clone()).with_chain_id(anvil.chain_id());

    let provider =
        Provider::<Http>::try_from(endpoint.clone())?.interval(Duration::from_millis(10));
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    let artifact: ContractArtifact = serde_json::from_str(VALIDATOR_ABI_JSON)?;
    let abi = artifact.abi;
    let bytecode = artifact.bytecode.object;

    let factory = ContractFactory::new(abi, bytecode.parse()?, client.clone());
    let contract = factory.deploy(())?.send().await?;

    Ok((anvil, contract, client))
}

/// Converts a BN254 field element (Fq) to a U256 (Ethereum-compatible)
fn fq_to_u256(f: Fq) -> U256 {
    let bytes = f.into_bigint().to_bytes_be();
    U256::from_big_endian(&bytes)
}

/// Converts a BN254 scalar (Fr) to a U256 (Ethereum-compatible)
#[allow(dead_code)]
fn fr_to_u256(fr: Fr) -> U256 {
    let bytes = fr.into_bigint().to_bytes_be();
    U256::from_big_endian(&bytes)
}

/// Calls `validateKeyPair` on the BN254KeyValidator contract
async fn validate_keypair_solidity(
    contract: &Contract<SignerMiddleware<Provider<Http>, LocalWallet>>,
    pubkey_g1: G1Affine,
    pubkey_g2: G2Affine,
) -> anyhow::Result<bool> {
    let g1_x = fq_to_u256(pubkey_g1.x);
    let g1_y = fq_to_u256(pubkey_g1.y);

    // G2 points are encoded as (x0, x1) and (y0, y1) where the point is (x0 + i*x1, y0 + i*y1)
    // However, Ethereum expects (x1, x0) and (y1, y0) ordering for G2
    let g2_x = [
        fq_to_u256(pubkey_g2.x.c1), // x1 (imaginary part)
        fq_to_u256(pubkey_g2.x.c0), // x0 (real part)
    ];
    let g2_y = [
        fq_to_u256(pubkey_g2.y.c1), // y1 (imaginary part)
        fq_to_u256(pubkey_g2.y.c0), // y0 (real part)
    ];

    let is_valid: bool = contract
        .method("validateKeyPair", ((g1_x, g1_y), (g2_x, g2_y)))?
        .call()
        .await?;

    Ok(is_valid)
}

#[tokio::test]
async fn test_validate_derived_key() -> anyhow::Result<()> {
    println!("\n=== Testing Key Derivation Validation ===\n");

    // Deploy the validator contract
    let (_anvil, contract, _client) = deploy_key_validator().await?;
    println!("BN254KeyValidator deployed at: {}", contract.address());

    // Derive a key from an EOA address
    let eoa = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
    let salt = "test-validation-salt";
    let keypair = derive_key_from_eoa(eoa, salt).map_err(|e| anyhow::anyhow!(e))?;

    println!("Derived keypair for EOA: {}", eoa);
    println!("Private key: {}", keypair.private_key);
    println!("G1 public key: ({}, {})", keypair.public_key_g1.x, keypair.public_key_g1.y);
    println!(
        "G2 public key: (({}, {}), ({}, {}))",
        keypair.public_key_g2.x_a,
        keypair.public_key_g2.x_b,
        keypair.public_key_g2.y_a,
        keypair.public_key_g2.y_b
    );

    // Parse the public keys back to affine points
    let g1_x = Fq::from_str(&keypair.public_key_g1.x).unwrap();
    let g1_y = Fq::from_str(&keypair.public_key_g1.y).unwrap();
    let g1_point = G1Affine::new_unchecked(g1_x, g1_y);

    let g2_x_c0 = Fq::from_str(&keypair.public_key_g2.x_a).unwrap();
    let g2_x_c1 = Fq::from_str(&keypair.public_key_g2.x_b).unwrap();
    let g2_y_c0 = Fq::from_str(&keypair.public_key_g2.y_a).unwrap();
    let g2_y_c1 = Fq::from_str(&keypair.public_key_g2.y_b).unwrap();
    let g2_point = G2Affine::new_unchecked(
        Fq2::new(g2_x_c0, g2_x_c1),
        Fq2::new(g2_y_c0, g2_y_c1),
    );

    // Validate the keypair using the Solidity contract
    println!("\nValidating keypair using pairing precompile...");
    let is_valid = validate_keypair_solidity(&contract, g1_point, g2_point).await?;

    println!("Validation result: {}", if is_valid { "✓ VALID" } else { "✗ INVALID" });

    assert!(is_valid, "Keypair validation failed! G1 and G2 keys don't match.");

    println!("\n✓ Key derivation validated successfully via pairing check!\n");

    Ok(())
}

#[tokio::test]
async fn test_validate_multiple_keys() -> anyhow::Result<()> {
    println!("\n=== Testing Multiple Key Validations ===\n");

    // Deploy the validator contract
    let (_anvil, contract, _client) = deploy_key_validator().await?;

    let test_cases = vec![
        ("0x70997970C51812dc3A010C7d01b50e0d17dc79C8", "salt-1"),
        ("0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC", "salt-2"),
        ("0x90F79bf6EB2c4f870365E785982E1f101E93b906", "salt-3"),
    ];

    for (eoa, salt) in test_cases {
        println!("Testing EOA: {}", eoa);

        // Derive key
        let keypair = derive_key_from_eoa(eoa, salt).map_err(|e| anyhow::anyhow!(e))?;

        // Parse to affine points
        let g1_x = Fq::from_str(&keypair.public_key_g1.x).unwrap();
        let g1_y = Fq::from_str(&keypair.public_key_g1.y).unwrap();
        let g1_point = G1Affine::new_unchecked(g1_x, g1_y);

        let g2_x_c0 = Fq::from_str(&keypair.public_key_g2.x_a).unwrap();
        let g2_x_c1 = Fq::from_str(&keypair.public_key_g2.x_b).unwrap();
        let g2_y_c0 = Fq::from_str(&keypair.public_key_g2.y_a).unwrap();
        let g2_y_c1 = Fq::from_str(&keypair.public_key_g2.y_b).unwrap();
        let g2_point = G2Affine::new_unchecked(
            Fq2::new(g2_x_c0, g2_x_c1),
            Fq2::new(g2_y_c0, g2_y_c1),
        );

        // Validate
        let is_valid = validate_keypair_solidity(&contract, g1_point, g2_point).await?;

        println!("  Result: {}", if is_valid { "✓ VALID" } else { "✗ INVALID" });
        assert!(is_valid, "Keypair validation failed for EOA {}", eoa);
    }

    println!("\n✓ All keys validated successfully!\n");

    Ok(())
}

#[tokio::test]
async fn test_invalid_keypair_detection() -> anyhow::Result<()> {
    println!("\n=== Testing Invalid Keypair Detection ===\n");

    // Deploy the validator contract
    let (_anvil, contract, _client) = deploy_key_validator().await?;

    // Derive two different keys
    let eoa1 = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
    let eoa2 = "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC";
    let salt = "test-salt";

    let keypair1 = derive_key_from_eoa(eoa1, salt).map_err(|e| anyhow::anyhow!(e))?;
    let keypair2 = derive_key_from_eoa(eoa2, salt).map_err(|e| anyhow::anyhow!(e))?;

    println!("Derived two different keypairs");
    println!("Testing mismatched pair (G1 from keypair1, G2 from keypair2)...");

    // Parse G1 from keypair1
    let g1_x = Fq::from_str(&keypair1.public_key_g1.x).unwrap();
    let g1_y = Fq::from_str(&keypair1.public_key_g1.y).unwrap();
    let g1_point = G1Affine::new_unchecked(g1_x, g1_y);

    // Parse G2 from keypair2 (different key!)
    let g2_x_c0 = Fq::from_str(&keypair2.public_key_g2.x_a).unwrap();
    let g2_x_c1 = Fq::from_str(&keypair2.public_key_g2.x_b).unwrap();
    let g2_y_c0 = Fq::from_str(&keypair2.public_key_g2.y_a).unwrap();
    let g2_y_c1 = Fq::from_str(&keypair2.public_key_g2.y_b).unwrap();
    let g2_point = G2Affine::new_unchecked(
        Fq2::new(g2_x_c0, g2_x_c1),
        Fq2::new(g2_y_c0, g2_y_c1),
    );

    // Validate the mismatched pair
    let is_valid = validate_keypair_solidity(&contract, g1_point, g2_point).await?;

    println!("Validation result: {}", if is_valid { "✗ VALID (unexpected!)" } else { "✓ INVALID (as expected)" });

    assert!(!is_valid, "Mismatched keypair should be invalid!");

    println!("\n✓ Invalid keypair correctly detected!\n");

    Ok(())
}

#[tokio::test]
async fn test_validate_generator_points() -> anyhow::Result<()> {
    println!("\n=== Testing Generator Points Validation ===\n");

    // Deploy the validator contract
    let (_anvil, contract, _client) = deploy_key_validator().await?;

    // Test with the generator points (should be valid with scalar = 1)
    let g1_gen = G1Affine::generator();
    let g2_gen = G2Affine::generator();

    println!("Testing generator points:");
    println!("G1 generator: ({}, {})", g1_gen.x, g1_gen.y);
    println!("G2 generator: (({}, {}), ({}, {}))",
        g2_gen.x.c0, g2_gen.x.c1, g2_gen.y.c0, g2_gen.y.c1);

    let is_valid = validate_keypair_solidity(&contract, g1_gen, g2_gen).await?;

    println!("Validation result: {}", if is_valid { "✓ VALID" } else { "✗ INVALID" });

    assert!(is_valid, "Generator points should form a valid keypair!");

    println!("\n✓ Generator points validated successfully!\n");

    Ok(())
}
