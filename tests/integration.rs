use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use tower::util::ServiceExt;
use bn254_rs::web::handlers;
use bn254_rs::web::models::G1Point;
use bn254_rs::web::store::Store;
use bn254_rs::{G1Point as CryptoG1Point, G2Point, pairing_check, hash_g1_point};
use hex;
use serde_json;
use std::sync::Arc;

#[tokio::test]
async fn test_g1_scalar_mul_vs_add() {
    let g = CryptoG1Point::generator();
    let double = g.add(&g);
    let s2 = g.scalar_mul(2u64.into());
    assert_eq!(double, s2);
}

#[tokio::test]
async fn test_pairing_identity() {
    let g1 = CryptoG1Point::generator();
    let g2 = G2Point::generator();
    let neg = g1.negate();

    let result = pairing_check(g1, g2, neg, g2);
    assert!(result);
}

#[tokio::test]
async fn test_hash_g1_point() {
    let g = CryptoG1Point::generator();
    let hash = hash_g1_point(&g);
    println!("hash: 0x{}", hex::encode(hash));
}

