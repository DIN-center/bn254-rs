//! # API Models Module
//!
//! This module defines all request and response data structures for the BN254 key management API.
//!
//! ## Data Structure Notes
//!
//! ### G2 Point Representation
//! - Internal format: `G2Point` with x_a, x_b, y_a, y_b fields
//! - Response format: `G2PointArray` with x: [a,b], y: [a,b] for client compatibility
//!
//! ## Maintenance Guidelines
//!
//! ### Adding New Fields
//! - Use `#[serde(default)]` for optional fields
//! - Maintain backward compatibility by keeping old fields
//! - Document the purpose and format of each field
//!
//! ### Changing Response Formats
//! - Consider creating new endpoints instead of breaking changes
//! - If changes are necessary, support both formats temporarily
//! - Document migration path for clients
//!
//! ## Security Considerations
//! - Private keys are stored as strings, never exposed in responses
//! - All string fields should be validated before use
//! - Consider field size limits to prevent DoS

use ark_bn254::{Fq, Fr};
use ark_ff::One;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::str::FromStr;

/// Represents a key pair stored in the system
///
/// ## Format
/// - `eoa_address`: Ethereum address (0x-prefixed hex)
/// - `private_key`: BLS private key as decimal string
/// - `public_key_g1`: G1 point coordinates
/// - `public_key_g2`: G2 point coordinates
///
/// ## Maintenance Note
/// This structure matches the JSON format in data/players.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    pub eoa_address: String,
    pub private_key: String,
    pub public_key_g1: G1Point,
    pub public_key_g2: G2Point,
}

/// Request for scalar multiplication operation
///
/// ## Usage
/// Multiply a G1 point (hash_x, hash_y) by the operator's private key
///
/// ## Format
/// - `eoa_address`: Operator's Ethereum address
/// - `hash_x`, `hash_y`: G1 point coordinates as hex or decimal strings
#[derive(Debug, Serialize, Deserialize)]
pub struct ScalarMulRequest {
    pub eoa_address: String,
    pub hash_x: String,
    pub hash_y: String,
}

/// Response for scalar multiplication
///
/// ## Fields
/// - `g1`, `g2`: Operator's public key components
/// - `signature`: Result of scalar multiplication
/// - `abi_encoded_result`: Ethereum ABI-encoded signature
#[derive(Debug, Serialize, Deserialize)]
pub struct ScalarMulResponse {
    pub g1: G1Point,
    pub g2: G2Point,
    pub signature: G1Point,
    pub abi_encoded_result: String,
}

impl fmt::Display for ScalarMulResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            // For {:?} format, show full details
            write!(
                f,
                "ScalarMulResponse {{ g1: {}, g2: {}, signature: {}, abi_encoded_result: {} }}",
                self.g1, self.g2, self.signature, self.abi_encoded_result
            )
        } else {
            // For normal {} format, show simplified view
            write!(
                f,
                "ScalarMulResponse {{ signature: {}, abi_encoded_result: {} }}",
                self.signature, self.abi_encoded_result
            )
        }
    }
}

/// Request for signing a G1 point with an EOA's private key
///
/// ## Format
/// - `eoa_address`: Operator's Ethereum address
/// - `message`: 128-character hex string containing concatenated x,y coordinates (64 chars each)
#[derive(Debug, Serialize, Deserialize)]
pub struct SignRequest {
    pub eoa_address: String,
    pub message: String,
}

/// Request for getting registration parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationParamsRequest {
    pub eoa_address: String,
    pub message_hash: String,
}

/// Solidity compatible BN254 G1Point structure
#[derive(Debug, Serialize, Deserialize)]
pub struct SolG1Point {
    pub x: String,
    pub y: String,
}

/// Solidity compatible BN254 G2Point structure with X and Y as arrays
#[derive(Debug, Serialize, Deserialize)]
pub struct SolG2Point {
    pub x: [String; 2],
    pub y: [String; 2],
}

/// Solidity compatible PubkeyRegistrationParams structure
#[derive(Debug, Serialize, Deserialize)]
pub struct PubkeyRegistrationParams {
    pub pubkey_registration_signature: SolG1Point,
    pub pubkey_g1: SolG1Point,
    pub pubkey_g2: SolG2Point,
}

/// Response containing all registration parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationParamsResponse {
    pub signature: G1Point,
    pub g1: G1Point,
    pub g2: G2Point,
    pub abi_encoded_signature: String,
    pub formatted_params: Value,
}

impl fmt::Display for RegistrationParamsResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            // For {:?} format, show full details
            write!(f, "RegistrationParamsResponse {{ signature: {}, g1: {}, g2: {}, abi_encoded_signature: {}, with formatted_params }}", 
                   self.signature, self.g1, self.g2, self.abi_encoded_signature)
        } else {
            // For normal {} format, show simplified view
            write!(f, "RegistrationParamsResponse {{ signature: {}, abi_encoded_signature: {}, with formatted_params }}", 
                   self.signature, self.abi_encoded_signature)
        }
    }
}

/// G1 point coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct G1Point {
    pub x: String,
    pub y: String,
}

impl fmt::Display for G1Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "G1Point {{ x: {}, y: {} }}", self.x, self.y)
    }
}

/// G2 point coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct G2Point {
    pub x_a: String,
    pub x_b: String,
    pub y_a: String,
    pub y_b: String,
}

impl fmt::Display for G2Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "G2Point {{ x_a: {}, x_b: {}, y_a: {}, y_b: {} }}",
            self.x_a, self.x_b, self.y_a, self.y_b
        )
    }
}

/// Response for signing a G1 point
#[derive(Debug, Serialize, Deserialize)]
pub struct SignResponse {
    pub signature: G1Point,
    pub g1: G1Point,
    pub g2: G2PointArray, // Array format for client compatibility
    pub abi_encoded_result: String,
}

/// G2 point with array format as expected by clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct G2PointArray {
    pub x: [String; 2],
    pub y: [String; 2],
}

impl From<G2Point> for G2PointArray {
    fn from(p: G2Point) -> Self {
        G2PointArray {
            x: [p.x_a, p.x_b],
            y: [p.y_a, p.y_b],
        }
    }
}

impl fmt::Display for SignResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            // For {:?} format, show full details
            write!(
                f,
                "SignResponse {{ signature: {}, g1: {}, g2: {:?}, abi_encoded_result: {} }}",
                self.signature, self.g1, self.g2, self.abi_encoded_result
            )
        } else {
            // For normal {} format, show simplified view
            write!(
                f,
                "SignResponse {{ signature: {}, abi_encoded_result: {} }}",
                self.signature, self.abi_encoded_result
            )
        }
    }
}

impl G1Point {
    #[allow(unused)]
    pub fn to_g1_point(&self) -> Result<ark_bn254::G1Projective, String> {
        let x = Fq::from_str(&self.x).map_err(|_| "Failed to parse x coordinate".to_string())?;
        let y = Fq::from_str(&self.y).map_err(|_| "Failed to parse y coordinate".to_string())?;
        Ok(ark_bn254::G1Projective::new_unchecked(x, y, Fq::one()))
    }
}

impl G2Point {
    #[allow(unused)]
    pub fn to_g2_point(&self) -> Result<ark_bn254::G2Projective, String> {
        let x_a =
            Fq::from_str(&self.x_a).map_err(|_| "Failed to parse x_a coordinate".to_string())?;
        let x_b =
            Fq::from_str(&self.x_b).map_err(|_| "Failed to parse x_b coordinate".to_string())?;
        let y_a =
            Fq::from_str(&self.y_a).map_err(|_| "Failed to parse y_a coordinate".to_string())?;
        let y_b =
            Fq::from_str(&self.y_b).map_err(|_| "Failed to parse y_b coordinate".to_string())?;
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
