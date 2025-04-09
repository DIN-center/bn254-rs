use actix_web::{web, HttpResponse, Responder};
use crate::web::models::{ScalarMulRequest, ScalarMulResponse, SignRequest, SignResponse, G1Point};
use crate::web::store::Store;
use ark_bn254::{Fq, G1Projective};
use ark_ec::CurveGroup;
use ark_ff::One;
use std::str::FromStr;
use log::error;

/// Get a key pair by EOA address
pub async fn get_key_pair(
    store: web::Data<Store>,
    eoa_address: web::Path<String>,
) -> impl Responder {
    match store.get_key_pair(&eoa_address) {
        Some(key_pair) => HttpResponse::Ok().json(key_pair),
        None => HttpResponse::NotFound().finish(),
    }
}

/// List all key pairs
pub async fn list_key_pairs(
    store: web::Data<Store>,
) -> impl Responder {
    let key_pairs = store.list_key_pairs();
    HttpResponse::Ok().json(key_pairs)
}

/// Perform scalar multiplication
pub async fn scalar_mul(
    store: web::Data<Store>,
    req: web::Json<ScalarMulRequest>,
) -> impl Responder {
    // Get key pair from store first
    let key_pair = match store.get_key_pair(&req.eoa_address) {
        Some(kp) => kp,
        None => {
            error!("Key pair not found for address: {}", req.eoa_address);
            return HttpResponse::NotFound().finish();
        }
    };

    // Parse hash point
    let hash_x = match Fq::from_str(&req.hash_x) {
        Ok(x) => x,
        Err(_) => {
            error!("Failed to parse hash_x coordinate");
            return HttpResponse::BadRequest().finish();
        }
    };

    let hash_y = match Fq::from_str(&req.hash_y) {
        Ok(y) => y,
        Err(_) => {
            error!("Failed to parse hash_y coordinate");
            return HttpResponse::BadRequest().finish();
        }
    };

    let hash_point = G1Projective::new_unchecked(hash_x, hash_y, Fq::one());

    // Get BLS private key
    let private_key = match key_pair.to_private_key() {
        Ok(k) => k,
        Err(e) => {
            error!("Failed to parse private key: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Perform scalar multiplication (hash_point * private_key)
    let result = hash_point * private_key;
    let result_affine = result.into_affine();

    // Create response
    let response = ScalarMulResponse {
        g1: key_pair.public_key_g1.clone(),
        g2: key_pair.public_key_g2.clone(),
        signature: G1Point {
            x: result_affine.x.to_string(),
            y: result_affine.y.to_string(),
        },
    };

    HttpResponse::Ok().json(response)
}

/// Sign a message
pub async fn sign(
    store: web::Data<Store>,
    req: web::Json<SignRequest>,
) -> impl Responder {
    // Get key pair from store
    let key_pair = match store.get_key_pair(&req.eoa_address) {
        Some(kp) => kp,
        None => {
            error!("Key pair not found for address: {}", req.eoa_address);
            return HttpResponse::NotFound().finish();
        }
    };

    // Parse point to sign
    let point = match Fq::from_str(&req.point) {
        Ok(x) => G1Projective::new_unchecked(x, Fq::one(), Fq::one()),
        Err(_) => {
            error!("Failed to parse point");
            return HttpResponse::BadRequest().finish();
        }
    };

    // Get private key
    let private_key = match key_pair.to_private_key() {
        Ok(k) => k,
        Err(e) => {
            error!("Failed to parse private key: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Perform signing (scalar multiplication)
    let signature = point * private_key;
    let signature_affine = signature.into_affine();

    // Create response
    let response = SignResponse {
        product: G1Point {
            x: signature_affine.x.to_string(),
            y: signature_affine.y.to_string(),
        },
        signer_g1: key_pair.public_key_g1.clone(),
    };

    HttpResponse::Ok().json(response)
} 