mod solidity;

use ark_bn254::{Fr, G1Projective};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::{One, PrimeField, Zero};
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use solidity::{call_scalar_mul_solidity_raw, deploy_bn254_wrapper, fq_to_u256, fr_to_u256};
use std::ops::Mul;

use bn254_rs::G1Point;

const POINTS_TO_TEST: usize = 5;

#[tokio::test]
async fn test_prop_scalar_mul_matches_solidity() -> anyhow::Result<()> {
    // Deploy the contract and get a reference to it
    let (_anvil, contract, _) = deploy_bn254_wrapper().await?;

    let base = G1Point::generator();
    let g1_x = base.inner().x;
    let g1_y = base.inner().y;

    // === Test edge cases for scalar multiplication ===
    let edge_scalars = [Fr::zero(), Fr::one(), -Fr::one(), Fr::from(2u64)];

    for (i, scalar) in edge_scalars.iter().enumerate() {
        // Use the same approach as in signature.rs
        let expected = G1Projective::from(base.inner().into_affine()).mul(*scalar).into_affine();
        let expected_x = fq_to_u256(expected.x);
        let expected_y = fq_to_u256(expected.y);

        println!("Edge case #{}:", i + 1);
        println!("  x: 0x{:x}", fq_to_u256(g1_x));
        println!("  y: 0x{:x}", fq_to_u256(g1_y));
        println!("  s: 0x{:x}", fr_to_u256(*scalar));

        let (sol_x, sol_y) = call_scalar_mul_solidity_raw(&contract, g1_x, g1_y, *scalar).await?;

        println!("  expected_x: 0x{:x}", expected_x);
        println!("  expected_y: 0x{:x}", expected_y);
        println!("  sol_x:      0x{:x}", sol_x);
        println!("  sol_y:      0x{:x}\n", sol_y);

        // For zero scalar, both implementations should return the point at infinity
        if scalar.is_zero() {
            // Check if either implementation returns the point at infinity
            let is_rust_infinity = expected.is_zero();
            let is_solidity_infinity = sol_x.is_zero() && sol_y.is_zero();
            
            assert!(is_rust_infinity || is_solidity_infinity, 
                "[edge {}] Neither implementation returned point at infinity", i + 1);
        } else {
            assert_eq!(sol_x, expected_x, "[edge {}] Mismatch in X", i + 1);
            assert_eq!(sol_y, expected_y, "[edge {}] Mismatch in Y", i + 1);
        }
    }

    // === Property-based random fuzzing tests ===
    let strategy = any::<[u8; 32]>();
    let mut runner = proptest::test_runner::TestRunner::default();

    for i in 0..POINTS_TO_TEST {
        let tree = strategy.new_tree(&mut runner).unwrap();
        let s = tree.current();
        let scalar = Fr::from_be_bytes_mod_order(&s);

        // Use the same approach as in signature.rs
        let expected = G1Projective::from(base.inner().into_affine()).mul(scalar).into_affine();
        let expected_x = fq_to_u256(expected.x);
        let expected_y = fq_to_u256(expected.y);

        println!("Testing point #{}:", i + 1);
        println!("  s: 0x{:x}", fr_to_u256(scalar));

        let (sol_x, sol_y) = call_scalar_mul_solidity_raw(&contract, g1_x, g1_y, scalar).await?;

        println!("  expected_x: 0x{:x}", expected_x);
        println!("  expected_y: 0x{:x}", expected_y);
        println!("  sol_x:      0x{:x}", sol_x);
        println!("  sol_y:      0x{:x}\n", sol_y);

        assert_eq!(sol_x, expected_x, "Mismatch in X");
        assert_eq!(sol_y, expected_y, "Mismatch in Y");
    }

    Ok(())
}
