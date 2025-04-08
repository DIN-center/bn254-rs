use ark_bn254::Fr;
use bn254_rs::*;

#[test]
fn test_g1_scalar_mul_vs_add() {
    let g = G1Point::generator();
    let double = g.add(&g);
    let s2 = g.scalar_mul(Fr::from(2u64));
    assert_eq!(double, s2);
}

#[test]
fn test_pairing_identity() {
    let g1 = G1Point::generator();
    let g2 = G2Point::generator();
    let neg = g1.negate();

    let result = pairing_check(g1, g2, neg, g2);
    assert!(result);
}

#[test]
fn test_hash_g1_point() {
    let g = G1Point::generator();
    let hash = hash_g1_point(&g);
    println!("hash: 0x{}", hex::encode(hash));
}
