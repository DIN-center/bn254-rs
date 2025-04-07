use ark_bn254::Fr;
use bn254_rs::*;

#[test]
fn test_g1_scalar_mul_vs_add() {
    let g = g1_generator();
    let double = g1_add(g, g);
    let s2 = g1_scalar_mul(g, Fr::from(2u64));
    assert_eq!(double, s2);
}

#[test]
fn test_pairing_identity() {
    let g1 = g1_generator();
    let g2 = g2_generator();
    let neg = g1_negate(g1);

    let result = pairing_check(g1, g2, neg, g2);
    assert!(result);
}

#[test]
fn test_hash_g1_point() {
    let g = g1_generator();
    let hash = hash_g1_point(&g);
    println!("hash: 0x{}", hex::encode(hash));
}
