use serde::{Deserialize, Serialize};
use ark_bn254::{Fr, Fq};
use ark_ff::One;
use std::str::FromStr;

/// Represents a key pair in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    pub eoa_address: String,
    pub private_key: String,
    pub public_key_g1: G1Point,
    pub public_key_g2: G2Point,
}

/// Request for scalar multiplication
#[derive(Debug, Serialize, Deserialize)]
pub struct ScalarMulRequest {
    pub eoa_address: String,
    pub hash_x: String,
    pub hash_y: String,
}

/// Response for scalar multiplication
#[derive(Debug, Serialize, Deserialize)]
pub struct ScalarMulResponse {
    pub g1: G1Point,
    pub g2: G2Point,
    pub signature: G1Point,
}

/// Request for signing a G1 point with an EOA's private key
#[derive(Debug, Serialize, Deserialize)]
pub struct SignRequest {
    pub eoa_address: String,
    pub point: String,
}

/// G1 point coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct G1Point {
    pub x: String,
    pub y: String,
}

/// G2 point coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct G2Point {
    pub x_a: String,
    pub x_b: String,
    pub y_a: String,
    pub y_b: String,
}

/// Response for signing a G1 point
#[derive(Debug, Serialize, Deserialize)]
pub struct SignResponse {
    pub product: G1Point,
    pub signer_g1: G1Point,
}

impl G1Point {
    pub fn to_g1_point(&self) -> Result<ark_bn254::G1Projective, String> {
        let x = Fq::from_str(&self.x).map_err(|_| "Failed to parse x coordinate".to_string())?;
        let y = Fq::from_str(&self.y).map_err(|_| "Failed to parse y coordinate".to_string())?;
        Ok(ark_bn254::G1Projective::new_unchecked(x, y, Fq::one()))
    }
}

impl G2Point {
    pub fn to_g2_point(&self) -> Result<ark_bn254::G2Projective, String> {
        let x_a = Fq::from_str(&self.x_a).map_err(|_| "Failed to parse x_a coordinate".to_string())?;
        let x_b = Fq::from_str(&self.x_b).map_err(|_| "Failed to parse x_b coordinate".to_string())?;
        let y_a = Fq::from_str(&self.y_a).map_err(|_| "Failed to parse y_a coordinate".to_string())?;
        let y_b = Fq::from_str(&self.y_b).map_err(|_| "Failed to parse y_b coordinate".to_string())?;
        Ok(ark_bn254::G2Projective::new_unchecked(
            ark_bn254::Fq2::new(x_a, x_b),
            ark_bn254::Fq2::new(y_a, y_b),
            ark_bn254::Fq2::one(),
        ))
    }
}

impl KeyPair {
    pub fn to_private_key(&self) -> Result<Fr, String> {
        Fr::from_str(&self.private_key).map_err(|_| "Failed to parse private key".to_string())
    }
} 