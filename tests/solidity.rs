use ark_bn254::{Fq, Fr, G1Affine};
use ark_ff::{BigInteger, PrimeField};
use ethers::abi::Abi;
use ethers::contract::{Contract, ContractFactory};
use ethers::prelude::*;
use ethers::types::U256;
use ethers::utils::{Anvil, AnvilInstance};
use std::{sync::Arc, time::Duration};
use ark_ec::AffineRepr;

const ABI_JSON: &str = include_str!("../out/BN254Wrapper.sol/BN254Wrapper.json");

#[derive(serde::Deserialize)]
struct ContractArtifact {
    abi: Abi,
    bytecode: BytecodeObject,
}

#[derive(serde::Deserialize)]
struct BytecodeObject {
    object: String,
}

/// Deploys the Solidity BN254Wrapper contract to a local Anvil instance
pub async fn deploy_bn254_wrapper() -> anyhow::Result<(
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

    let artifact: ContractArtifact = serde_json::from_str(ABI_JSON)?;
    let abi = artifact.abi;
    let bytecode = artifact.bytecode.object;

    let factory = ContractFactory::new(abi, bytecode.parse()?, client.clone());
    let contract = factory.deploy(())?.send().await?;

    Ok((anvil, contract, client))
}

/// Converts a BN254 field element (Fq) to a U256 (Ethereum-compatible)
pub fn fq_to_u256(f: Fq) -> U256 {
    let bytes = f.into_bigint().to_bytes_be();
    U256::from_big_endian(&bytes)
}

/// Converts a BN254 scalar (Fr) to a U256 (Ethereum-compatible)
pub fn fr_to_u256(fr: Fr) -> U256 {
    let bytes = fr.into_bigint().to_bytes_be();
    U256::from_big_endian(&bytes)
}

/// Calls `scalar_mul` on the BN254Wrapper contract and returns the resulting G1Affine
#[allow(dead_code)]
pub async fn call_scalar_mul_solidity(
    contract: &Contract<SignerMiddleware<Provider<Http>, LocalWallet>>,
    g1: G1Affine,
    scalar: Fr,
) -> anyhow::Result<G1Affine> {
    let g1_x = fq_to_u256(g1.x);
    let g1_y = fq_to_u256(g1.y);
    let s = fr_to_u256(scalar);

    let (x, y): (U256, U256) = contract
        .method("scalar_mul", ((g1_x, g1_y), s))?
        .call()
        .await?;

    let mut x_bytes = [0u8; 32];
    let mut y_bytes = [0u8; 32];
    x.to_big_endian(&mut x_bytes);
    y.to_big_endian(&mut y_bytes);

    let fq_x = Fq::from_be_bytes_mod_order(&x_bytes);
    let fq_y = Fq::from_be_bytes_mod_order(&y_bytes);
    Ok(G1Affine::new_unchecked(fq_x, fq_y))
}

/// Calls `scalar_mul` and returns U256 (raw) instead of G1Affine
#[allow(dead_code)]
pub async fn call_scalar_mul_solidity_raw(
    contract: &Contract<SignerMiddleware<Provider<Http>, LocalWallet>>,
    x: ark_bn254::Fq,
    y: ark_bn254::Fq,
    scalar: ark_bn254::Fr,
) -> anyhow::Result<(U256, U256)> {
    let g1_x = fq_to_u256(x);
    let g1_y = fq_to_u256(y);
    let s = fr_to_u256(scalar);

    let result: (U256, U256) = contract
        .method("scalar_mul", ((g1_x, g1_y), s))?
        .call()
        .await?;

    Ok(result)
}

#[tokio::test]
async fn test_scalar_mul_equivalence() -> anyhow::Result<()> {
    // Deploy contract
    let (anvil, contract, _) = deploy_bn254_wrapper().await?;
    println!("Contract deployed at: {}", contract.address());

    // Get generator point from ark-bn254
    let g1 = G1Affine::generator();
    println!("\nGenerator point from ark-bn254:");
    println!("  x: {:?}", g1.x);
    println!("  y: {:?}", g1.y);

    // Convert to U256 for contract call
    let g1_x = fq_to_u256(g1.x);
    let g1_y = fq_to_u256(g1.y);
    println!("\nConverted to U256:");
    println!("  x: 0x{:064x}", g1_x);
    println!("  y: 0x{:064x}", g1_y);

    let scalar = Fr::from(42u64);

    // Perform scalar multiplication in Rust
    let rust_result = g1 * scalar;
    println!("\nRust result:");
    println!("  x: {:?}", rust_result.x);
    println!("  y: {:?}", rust_result.y);

    // Call contract's scalar_mul
    let solidity_result = call_scalar_mul_solidity(&contract, g1, scalar).await?;
    println!("\nSolidity result:");
    println!("  x: {:?}", solidity_result.x);
    println!("  y: {:?}", solidity_result.y);

    // Compare results
    assert_eq!(rust_result.x, solidity_result.x, "X coordinates don't match");
    assert_eq!(rust_result.y, solidity_result.y, "Y coordinates don't match");

    // Test encoding
    let rust_x = fq_to_u256(rust_result.x);
    let rust_y = fq_to_u256(rust_result.y);
    let solidity_x = fq_to_u256(solidity_result.x);
    let solidity_y = fq_to_u256(solidity_result.y);

    println!("\nEncoded results:");
    println!("Rust:");
    println!("  x: 0x{:064x}", rust_x);
    println!("  y: 0x{:064x}", rust_y);
    println!("Solidity:");
    println!("  x: 0x{:064x}", solidity_x);
    println!("  y: 0x{:064x}", solidity_y);

    assert_eq!(rust_x, solidity_x, "Encoded X coordinates don't match");
    assert_eq!(rust_y, solidity_y, "Encoded Y coordinates don't match");

    Ok(())
}
