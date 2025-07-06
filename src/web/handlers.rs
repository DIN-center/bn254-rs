//! # API Handlers Module
//! 
//! This module contains all HTTP request handlers for the BN254 key management service.
//! Each handler implements a specific API endpoint with proper error handling and logging.
//! 
//! ## Handler Functions
//! 
//! - `get_key_pair`: Retrieve public key components for an operator
//! - `list_key_pairs`: List all available key pairs
//! - `scalar_mul`: Perform scalar multiplication on a G1 point
//! - `sign`: Sign a message with an operator's BLS private key
//! - `get_registration_params`: Generate operator registration parameters
//! 
//! ## Maintenance Notes
//! 
//! ### Adding New Handlers
//! 1. Define the handler function with proper tracing
//! 2. Use structured logging with tracing macros
//! 3. Return appropriate HTTP status codes
//! 4. Add the route in server.rs
//! 
//! ### Error Handling Guidelines
//! - 400 Bad Request: Invalid input format or parsing errors
//! - 404 Not Found: Requested key/operator not found
//! - 500 Internal Server Error: Unexpected processing errors
//! 
//! ### API Format
//! The `/sign` endpoint:
//! - Accepts `message` field: 128-char hex string (concatenated x,y coordinates)
//! - Response uses array format for G2 points
//! 
//! ### Performance Considerations
//! - All key lookups are O(1) from in-memory HashMap
//! - Cryptographic operations are the main bottleneck
//! - Consider adding caching for repeated operations
//! 
//! ## Security Notes
//! - Private keys never leave the store module
//! - All inputs are validated before processing
//! - Hex strings are parsed safely with proper error handling

use crate::web::models::{
    G1Point, RegistrationParamsRequest, RegistrationParamsResponse, ScalarMulRequest,
    ScalarMulResponse, SignRequest, SignResponse,
};
use crate::web::store::Store;
use ark_bn254::{Fq, G1Projective};
use ark_ec::CurveGroup;
use ark_ff::{BigInteger, One, PrimeField, Zero};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json;
use hex;
use tracing::{debug, error, info, trace};
use std::str::FromStr;
use std::sync::Arc;
// Add these imports for ABI encoding
use ethers::abi::{encode, Token};
use ethers::types::{Bytes, U256};

/// Parse a field element from a string
/// 
/// Supports multiple formats:
/// - Hex with 0x prefix: "0x1234abcd..."
/// - Hex without prefix: "1234abcd..."
/// - Decimal: "123456789"
/// 
/// # Maintenance Notes
/// 
/// This function is critical for input parsing. Any changes must maintain:
/// - Support for both hex formats (with/without 0x)
/// - Proper error messages for debugging
/// - Safe handling of arbitrary length inputs
/// 
/// # Examples
/// 
/// ```
/// let fq = parse_fq_from_str("0x1234")?;
/// let fq = parse_fq_from_str("1234")?;
/// let fq = parse_fq_from_str("123456")?;
/// ```
fn parse_fq_from_str(s: &str) -> Result<Fq, String> {
    // Try to detect if it's a hex string (with or without 0x prefix)
    let is_hex = if let Some(hex) = s.strip_prefix("0x") {
        // Has 0x prefix
        Some(hex)
    } else if s.chars().all(|c| c.is_ascii_hexdigit()) {
        // Looks like a hex string without 0x prefix
        // Accept any even-length hex string, not just 64 chars
        if s.len() % 2 == 0 {
            Some(s)
        } else {
            return Err(format!("Hex string has odd length: {}", s.len()));
        }
    } else {
        None
    };
    
    if let Some(hex) = is_hex {
        let bytes = match hex::decode(hex) {
            Ok(b) => b,
            Err(e) => return Err(format!("Failed to decode hex '{}': {:?}", hex, e)),
        };
        Ok(Fq::from_be_bytes_mod_order(&bytes))
    } else {
        // Try to parse as decimal
        Fq::from_str(s).map_err(|e| format!("Failed to parse '{}' as decimal: {:?}", s, e))
    }
}

// Helper function to convert Fq to U256
fn fq_to_u256(fq: &Fq) -> U256 {
    // Convert the field element to big-endian bytes
    let mut bytes = vec![0u8; 32];
    let fq_bytes = fq.into_bigint().to_bytes_be();

    // Copy to the end of the array to ensure big-endian ordering with leading zeros
    if fq_bytes.len() <= 32 {
        bytes[32 - fq_bytes.len()..].copy_from_slice(&fq_bytes);
    } else {
        // Truncate if somehow larger than 32 bytes (shouldn't happen with ark_bn254::Fq)
        bytes.copy_from_slice(&fq_bytes[fq_bytes.len() - 32..]);
    }

    U256::from_big_endian(&bytes)
}

// Function to ABI encode a G1 point (X, Y coordinates)
fn abi_encode_g1_point(x: &Fq, y: &Fq) -> String {
    // Convert field elements to U256
    let x_u256 = fq_to_u256(x);
    let y_u256 = fq_to_u256(y);

    // Create the ABI tokens - G1 points are encoded as tuples of two uint256 values
    let tokens = vec![Token::Uint(x_u256), Token::Uint(y_u256)];

    // Pack according to Ethereum ABI
    let encoded = Bytes::from(encode(&tokens));

    // Return as 0x-prefixed hex string
    format!("0x{}", hex::encode(encoded))
}

// Helper function to convert Fq to hex string for JSON representation
fn fq_to_hex_string(fq: &Fq) -> String {
    let u256 = fq_to_u256(fq);
    format!("0x{:064x}", u256)
}

// Function to create a formatted PubkeyRegistrationParams from the response data
fn create_formatted_params(
    signature: &G1Point, 
    g1: &G1Point, 
    g2: &crate::web::models::G2Point
) -> serde_json::Value {
    use serde_json::json;
    
    // Parse the G1 and G2 points
    let sig_x = Fq::from_str(&signature.x).unwrap_or_else(|_| Fq::zero());
    let sig_y = Fq::from_str(&signature.y).unwrap_or_else(|_| Fq::zero());
    
    let g1_x = Fq::from_str(&g1.x).unwrap_or_else(|_| Fq::zero());
    let g1_y = Fq::from_str(&g1.y).unwrap_or_else(|_| Fq::zero());
    
    let g2_x_a = Fq::from_str(&g2.x_a).unwrap_or_else(|_| Fq::zero());
    let g2_x_b = Fq::from_str(&g2.x_b).unwrap_or_else(|_| Fq::zero());
    let g2_y_a = Fq::from_str(&g2.y_a).unwrap_or_else(|_| Fq::zero());
    let g2_y_b = Fq::from_str(&g2.y_b).unwrap_or_else(|_| Fq::zero());
    
    // Create the formatted JSON structure
    json!({
        "internalType": "struct IBLSApkRegistryTypes.PubkeyRegistrationParams",
        "name": "params",
        "type": "tuple",
        "components": [
            {
                "internalType": "struct BN254.G1Point",
                "name": "pubkeyRegistrationSignature",
                "type": "tuple",
                "components": [
                    {
                        "internalType": "uint256",
                        "name": "X",
                        "type": "uint256",
                        "value": fq_to_hex_string(&sig_x)
                    },
                    {
                        "internalType": "uint256",
                        "name": "Y",
                        "type": "uint256",
                        "value": fq_to_hex_string(&sig_y)
                    }
                ]
            },
            {
                "internalType": "struct BN254.G1Point",
                "name": "pubkeyG1",
                "type": "tuple",
                "components": [
                    {
                        "internalType": "uint256",
                        "name": "X",
                        "type": "uint256",
                        "value": fq_to_hex_string(&g1_x)
                    },
                    {
                        "internalType": "uint256",
                        "name": "Y",
                        "type": "uint256",
                        "value": fq_to_hex_string(&g1_y)
                    }
                ]
            },
            {
                "internalType": "struct BN254.G2Point",
                "name": "pubkeyG2",
                "type": "tuple",
                "components": [
                    {
                        "internalType": "uint256[2]",
                        "name": "X",
                        "type": "uint256[2]",
                        "value": [
                            fq_to_hex_string(&g2_x_a),
                            fq_to_hex_string(&g2_x_b)
                        ]
                    },
                    {
                        "internalType": "uint256[2]",
                        "name": "Y",
                        "type": "uint256[2]",
                        "value": [
                            fq_to_hex_string(&g2_y_a),
                            fq_to_hex_string(&g2_y_b)
                        ]
                    }
                ]
            }
        ]
    })
}

/// Health check endpoint
/// 
/// Returns service status and key store information
pub async fn health_check(State(store): State<Arc<Store>>) -> impl IntoResponse {
    debug!("Health check requested");
    
    let key_count = store.key_map.len();
    let status = if key_count > 0 { "healthy" } else { "degraded" };
    
    Json(serde_json::json!({
        "status": status,
        "service": "bn254-signer",
        "version": env!("CARGO_PKG_VERSION"),
        "keys_loaded": key_count,
        "message": if key_count > 0 {
            format!("Service healthy with {} keys loaded", key_count)
        } else {
            "Service running but no keys loaded".to_string()
        }
    }))
}

/// Get a key pair by EOA address
pub async fn get_key_pair(
    State(store): State<Arc<Store>>,
    Path(eoa_address): Path<String>,
) -> impl IntoResponse {
    info!(
        eoa_address = %eoa_address,
        "Received request for key pair"
    );
    trace!("Looking up key pair in store");

    match store.get_key_pair(&eoa_address) {
        Some(key_pair) => {
            info!(
                eoa_address = %eoa_address,
                "Key pair found"
            );
            trace!(
                g1.x = %key_pair.public_key_g1.x,
                g1.y = %key_pair.public_key_g1.y,
                g2.x_a = %key_pair.public_key_g2.x_a,
                g2.x_b = %key_pair.public_key_g2.x_b,
                g2.y_a = %key_pair.public_key_g2.y_a,
                g2.y_b = %key_pair.public_key_g2.y_b,
                "Returning key pair details"
            );
            (StatusCode::OK, Json(key_pair)).into_response()
        }
        None => {
            error!(
                eoa_address = %eoa_address,
                "Key pair not found"
            );
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

/// List all key pairs
pub async fn list_key_pairs(State(store): State<Arc<Store>>) -> impl IntoResponse {
    info!("Received request to list all key pairs");

    let key_pairs = store.list_key_pairs();
    let count = key_pairs.len();

    info!("Returning {} key pairs", count);
    debug!(
        "Key pair addresses: {:?}",
        key_pairs
            .iter()
            .map(|kp| &kp.eoa_address)
            .collect::<Vec<_>>()
    );

    (StatusCode::OK, Json(key_pairs)).into_response()
}

/// Perform scalar multiplication
pub async fn scalar_mul(
    State(store): State<Arc<Store>>,
    Json(req): Json<ScalarMulRequest>,
) -> impl IntoResponse {
    info!(
        "Received scalar_mul request for EOA address: {}",
        req.eoa_address
    );
    debug!("Hash point: x={}, y={}", req.hash_x, req.hash_y);

    // Get key pair from store first
    let key_pair = match store.get_key_pair(&req.eoa_address) {
        Some(kp) => {
            info!("Found key pair for EOA address: {}", req.eoa_address);
            kp
        }
        None => {
            error!("Key pair not found for address: {}", req.eoa_address);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    // Parse hash point
    let hash_x = match parse_fq_from_str(&req.hash_x) {
        Ok(x) => x,
        Err(e) => {
            error!("Failed to parse hash_x coordinate: {}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let hash_y = match parse_fq_from_str(&req.hash_y) {
        Ok(y) => y,
        Err(e) => {
            error!("Failed to parse hash_y coordinate: {}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let hash_point = G1Projective::new_unchecked(hash_x, hash_y, Fq::one());

    // Get BLS private key
    let private_key = match key_pair.to_private_key() {
        Ok(k) => k,
        Err(e) => {
            error!("Failed to parse private key: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Perform scalar multiplication (hash_point * private_key)
    debug!("Performing scalar multiplication");
    let result = hash_point * private_key;
    let result_affine = result.into_affine();
    debug!(
        "Scalar multiplication result: x={}, y={}",
        result_affine.x, result_affine.y
    );

    // Generate ABI-encoded result
    debug!("Generating ABI-encoded result");
    let abi_encoded_result = abi_encode_g1_point(&result_affine.x, &result_affine.y);
    debug!("ABI-encoded result: {}", abi_encoded_result);

    // Create response - add the abi_encoded_result field
    let response = ScalarMulResponse {
        g1: key_pair.public_key_g1.clone(),
        g2: key_pair.public_key_g2.clone(),
        signature: G1Point {
            x: result_affine.x.to_string(),
            y: result_affine.y.to_string(),
        },
        // Add the ABI-encoded result
        abi_encoded_result,
    };

    // Concise response summary
    debug!("Response: {}", response);
    // Detailed debug logging with alternate format
    debug!("Response Details: {:?}", &response);

    info!(
        "Successfully completed scalar multiplication for EOA: {}",
        req.eoa_address
    );
    (StatusCode::OK, Json(response)).into_response()
}

/// Sign a message with BLS private key
/// 
/// ## Input Format
/// - `eoa_address`: Operator's Ethereum address
/// - `message`: 128-character hex string containing concatenated x,y coordinates (64 chars each)
/// 
/// ## Response Format
/// ```json
/// {
///   "signature": {"x": "0x...", "y": "0x..."},
///   "g1": {"x": "0x...", "y": "0x..."},
///   "g2": {"x": ["0x...", "0x..."], "y": ["0x...", "0x..."]},
///   "abi_encoded_result": "0x..."
/// }
/// ```
/// 
/// ## Error Cases
/// - 400: Invalid hex format or length
/// - 404: EOA address not found
/// - 500: Private key parsing error
/// 
/// ## Example Usage
/// ```bash
/// curl -X POST http://localhost:3000/sign \
///   -d '{"eoa_address": "0x...", "message": "1234...abcd"}'
/// ```
pub async fn sign(
    State(store): State<Arc<Store>>,
    Json(req): Json<SignRequest>,
) -> impl IntoResponse {
    info!(
        eoa_address = %req.eoa_address,
        message = %req.message,
        "Received sign request"
    );

    // Get key pair from store
    let key_pair = match store.get_key_pair(&req.eoa_address) {
        Some(kp) => {
            info!("Found key pair for EOA address: {}", req.eoa_address);
            kp
        }
        None => {
            error!("Key pair not found for address: {}", req.eoa_address);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    // Parse message as concatenated hex string of x and y coordinates
    trace!(
        message_value = %req.message,
        message_len = req.message.len(),
        "Parsing message as concatenated G1 point"
    );
    
    // Remove 0x prefix if present
    let hex_str = req.message.strip_prefix("0x").unwrap_or(&req.message);
    
    // Expect 128 hex chars (64 bytes total)
    if hex_str.len() != 128 {
        error!(
            hex_len = hex_str.len(),
            "Invalid message length for G1 point (expected 128 hex chars)"
        );
        return StatusCode::BAD_REQUEST.into_response();
    }
    
    // Split into x and y coordinates (32 bytes each)
    let x_hex = &hex_str[0..64];
    let y_hex = &hex_str[64..128];
    
    let x = match parse_fq_from_str(x_hex) {
        Ok(x) => x,
        Err(e) => {
            error!(error = %e, "Failed to parse x coordinate from message");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };
    
    let y = match parse_fq_from_str(y_hex) {
        Ok(y) => y,
        Err(e) => {
            error!(error = %e, "Failed to parse y coordinate from message");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };
    
    debug!("Successfully parsed message as G1 point");
    let point = G1Projective::new_unchecked(x, y, Fq::one());

    // Get private key
    let private_key = match key_pair.to_private_key() {
        Ok(k) => k,
        Err(e) => {
            error!("Failed to parse private key: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Perform signing (scalar multiplication)
    debug!("Performing signing operation (scalar multiplication)");
    let signature = point * private_key;
    let signature_affine = signature.into_affine();
    debug!(
        "Signature result: x={}, y={}",
        signature_affine.x, signature_affine.y
    );

    // Generate ABI-encoded result
    debug!("Generating ABI-encoded result");
    let abi_encoded_result = abi_encode_g1_point(&signature_affine.x, &signature_affine.y);
    debug!("ABI-encoded result: {}", abi_encoded_result);

    // Create response with backward compatible field names
    let response = SignResponse {
        signature: G1Point {
            x: signature_affine.x.to_string(),
            y: signature_affine.y.to_string(),
        },
        g1: key_pair.public_key_g1.clone(),
        g2: key_pair.public_key_g2.clone().into(),  // Convert to array format
        // Add the ABI-encoded result
        abi_encoded_result,
    };

    // Concise response summary
    debug!("Response: {}", response);
    // Detailed debug logging with alternate format
    debug!("Response Details: {:?}", &response);

    info!(
        "Successfully completed signing for EOA: {}",
        req.eoa_address
    );
    (StatusCode::OK, Json(response)).into_response()
}

/// Get registration parameters for an operator
pub async fn get_registration_params(
    State(store): State<Arc<Store>>,
    Json(req): Json<RegistrationParamsRequest>,
) -> Response {
    info!(
        "Received registration params request for EOA: {}",
        req.eoa_address
    );
    debug!("Message hash: {}", req.message_hash);

    // Get key pair from store
    let key_pair = match store.get_key_pair(&req.eoa_address) {
        Some(kp) => {
            info!("Found key pair for EOA: {}", req.eoa_address);
            debug!(
                "G1 point: x={}, y={}",
                kp.public_key_g1.x, kp.public_key_g1.y
            );
            debug!(
                "G2 point: x_a={}, x_b={}, y_a={}, y_b={}",
                kp.public_key_g2.x_a,
                kp.public_key_g2.x_b,
                kp.public_key_g2.y_a,
                kp.public_key_g2.y_b
            );
            kp
        }
        None => {
            error!("Key pair not found for address: {}", req.eoa_address);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    // Parse message hash
    let hash = match parse_fq_from_str(&req.message_hash) {
        Ok(h) => {
            debug!("Successfully parsed message hash: {}", h);
            h
        }
        Err(e) => {
            error!("Failed to parse message hash: {:?}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    // Create hash point
    let hash_point = G1Projective::new_unchecked(hash, Fq::one(), Fq::one());
    debug!("Created hash point: x={}, y={}", hash_point.x, hash_point.y);

    // Get private key
    let private_key = match key_pair.to_private_key() {
        Ok(k) => {
            debug!("Successfully parsed private key");
            k
        }
        Err(e) => {
            error!("Failed to parse private key: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Perform signing (scalar multiplication)
    let signature = hash_point * private_key;
    let signature_affine = signature.into_affine();
    debug!(
        "Generated signature: x={}, y={}",
        signature_affine.x, signature_affine.y
    );

    // Generate ABI-encoded signature
    let abi_encoded_signature = abi_encode_g1_point(&signature_affine.x, &signature_affine.y);
    debug!("ABI encoded signature: {}", abi_encoded_signature);

    // Create the signature G1Point
    let signature_g1 = G1Point {
        x: signature_affine.x.to_string(),
        y: signature_affine.y.to_string(),
    };
    
    // Create the formatted params structure
    let formatted_params = create_formatted_params(
        &signature_g1,
        &key_pair.public_key_g1,
        &key_pair.public_key_g2
    );
    
    // Create response with all parameters
    let response = RegistrationParamsResponse {
        signature: signature_g1,
        g1: key_pair.public_key_g1.clone(),
        g2: key_pair.public_key_g2.clone(),
        // Add the ABI-encoded signature
        abi_encoded_signature,
        // Add the formatted params
        formatted_params,
    };

    info!(
        "Successfully generated registration parameters for EOA: {}",
        req.eoa_address
    );
    // Concise response summary
    debug!("Response: {}", response);
    // Detailed debug logging with alternate format
    debug!("Response Details: {:?}", &response);

    (StatusCode::OK, Json(response)).into_response()
}
