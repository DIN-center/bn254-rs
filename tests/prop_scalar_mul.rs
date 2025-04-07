use ark_bn254::{Fq, Fr, G1Projective};
use ark_ec::{CurveGroup, Group};
use ark_ff::{BigInteger, One, PrimeField, Zero};
use ethers::abi::Abi;
use ethers::contract::ContractFactory;
use ethers::prelude::*;
use ethers::utils::Anvil;
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use std::ops::Mul;
use std::{sync::Arc, time::Duration};

// Load the compiled Solidity contract ABI from the JSON file
const ABI_JSON: &str = include_str!("../out/BN254Wrapper.sol/BN254Wrapper.json");

// Number of random test cases to generate
const POINTS_TO_TEST: usize = 5;

// Convert a BN254 field element (Fq) to Ethereum U256 format
fn fq_to_u256(f: Fq) -> U256 {
    let mut bytes = Vec::from(f.into_bigint().to_bytes_be());
    while bytes.len() < 32 {
        bytes.insert(0, 0);
    }
    U256::from_big_endian(&bytes)
}

// Convert a BN254 scalar (Fr) to Ethereum U256 format
fn fr_to_u256(fr: Fr) -> U256 {
    let mut bytes = Vec::from(fr.into_bigint().to_bytes_be());
    while bytes.len() < 32 {
        bytes.insert(0, 0);
    }
    U256::from_big_endian(&bytes)
}

// Structure to deserialize the contract artifact JSON
#[derive(serde::Deserialize)]
struct ContractArtifact {
    abi: Abi,
    bytecode: BytecodeObject,
}

// Structure to deserialize the bytecode object from the contract artifact
#[derive(serde::Deserialize)]
struct BytecodeObject {
    object: String,
}

// Main test function that verifies scalar multiplication results match between Rust and Solidity
#[tokio::test]
async fn test_prop_scalar_mul_matches_solidity() -> anyhow::Result<()> {
    // Set up local Ethereum test environment using Anvil
    let anvil = Anvil::new().spawn();
    let wallet: LocalWallet =
        LocalWallet::from(anvil.keys()[0].clone()).with_chain_id(anvil.chain_id());

    // Configure the Ethereum client with appropriate signer settings
    let provider =
        Provider::<Http>::try_from(anvil.endpoint())?.interval(Duration::from_millis(10));
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    // Load and parse the contract artifact
    let artifact: ContractArtifact = serde_json::from_str(ABI_JSON)?;
    let abi = artifact.abi;
    let bytecode = artifact.bytecode.object;

    // Deploy the contract to the local test network
    let factory = ContractFactory::new(abi.clone(), bytecode.parse()?, client.clone());
    let contract = factory.deploy(())?.send().await?;

    // Get the generator point of G1 group
    let base = <G1Projective as Group>::generator().into_affine();
    let g1_x = base.x;
    let g1_y = base.y;

    // === Test edge cases for scalar multiplication ===
    // Test with special scalar values: 0, 1, -1, and 2
    let edge_scalars = [Fr::zero(), Fr::one(), -Fr::one(), Fr::from(2u64)];

    for (i, scalar) in edge_scalars.iter().enumerate() {
        // Compute scalar multiplication result in Rust
        let result = G1Projective::from(base).mul(*scalar).into_affine();
        let expected_x = fq_to_u256(result.x);
        let expected_y = fq_to_u256(result.y);

        // Print test parameters for debugging
        println!("Edge case #{}:", i + 1);
        println!("  x: 0x{:x}", fq_to_u256(g1_x));
        println!("  y: 0x{:x}", fq_to_u256(g1_y));
        println!("  s: 0x{:x}", fr_to_u256(*scalar));

        // Call the Solidity contract's scalar multiplication function
        let sol_result: (U256, U256) = contract
            .method::<_, (U256, U256)>(
                "scalar_mul",
                ((fq_to_u256(g1_x), fq_to_u256(g1_y)), fr_to_u256(*scalar)),
            )?
            .call()
            .await?;

        // Print results for comparison
        println!("  expected_x: 0x{:x}", expected_x);
        println!("  expected_y: 0x{:x}", expected_y);
        println!("  sol_x:      0x{:x}", sol_result.0);
        println!("  sol_y:      0x{:x}\n", sol_result.1);

        // Verify that Solidity results match Rust results
        assert_eq!(
            sol_result.0,
            expected_x,
            "[edge {}] Mismatch in X coordinate",
            i + 1
        );
        assert_eq!(
            sol_result.1,
            expected_y,
            "[edge {}] Mismatch in Y coordinate",
            i + 1
        );
    }

    // === Property-based random fuzzing tests ===
    // Generate random 32-byte values to use as scalars
    let strategy = any::<[u8; 32]>();
    let mut runner = proptest::test_runner::TestRunner::default();

    // Test multiple random scalar values
    for i in 0..POINTS_TO_TEST {
        let tree = strategy.new_tree(&mut runner).unwrap();
        let s = tree.current();
        let scalar = Fr::from_be_bytes_mod_order(&s);

        // Compute scalar multiplication result in Rust
        let result = G1Projective::from(base).mul(scalar).into_affine();
        let expected_x = fq_to_u256(result.x);
        let expected_y = fq_to_u256(result.y);

        // Print test parameters for debugging
        println!("Testing point #{}:", i + 1);
        println!("  x: 0x{:x}", fq_to_u256(g1_x));
        println!("  y: 0x{:x}", fq_to_u256(g1_y));
        println!("  s: 0x{:x}", fr_to_u256(scalar));

        // Call the Solidity contract's scalar multiplication function
        let sol_result: (U256, U256) = contract
            .method::<_, (U256, U256)>(
                "scalar_mul",
                ((fq_to_u256(g1_x), fq_to_u256(g1_y)), fr_to_u256(scalar)),
            )?
            .call()
            .await?;

        // Print results for comparison
        println!("  expected_x: 0x{:x}", expected_x);
        println!("  expected_y: 0x{:x}", expected_y);
        println!("  sol_x:      0x{:x}", sol_result.0);
        println!("  sol_y:      0x{:x}\n", sol_result.1);

        // Verify that Solidity results match Rust results
        assert_eq!(sol_result.0, expected_x, "Mismatch in X coordinate");
        assert_eq!(sol_result.1, expected_y, "Mismatch in Y coordinate");
    }

    Ok(())
}
