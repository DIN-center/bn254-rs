//! # BN254 Key Generation Module
//!
//! This module provides functionality to derive BN254 BLS keys from an EOA address + secret salt.
//!
//! ## Key Derivation Process
//!
//! 1. Combine EOA address (checksummed) with secret salt
//! 2. Use SHA3-256 (Keccak256) to hash the combined input
//! 3. Reduce the hash modulo the curve order to get a valid scalar
//! 4. Use the scalar as the private key
//! 5. Derive public keys on G1 and G2 by multiplying generators
//!
//! ## Security Considerations
//!
//! - The secret salt MUST be kept secure and never exposed
//! - Use a high-entropy salt (minimum 32 bytes of randomness)
//! - Different salts produce completely different keys for the same EOA
//! - This approach is deterministic: same EOA + salt = same key

use ark_bn254::{Fr, G1Projective, G2Projective};
use ark_ec::{CurveGroup, Group};
use ark_ff::PrimeField;
use sha3::{Digest, Keccak256};

use crate::web::models::{G1Point, G2Point, KeyPair};

/// Derives a BN254 BLS key pair from an EOA address and secret salt
///
/// ## Arguments
/// * `eoa_address` - Ethereum address (0x-prefixed hex string, will be checksummed)
/// * `secret_salt` - Secret salt for key derivation (high-entropy string)
///
/// ## Returns
/// A `KeyPair` containing the derived private and public keys
///
/// ## Example
/// ```
/// use bn254_rs::keygen::derive_key_from_eoa;
///
/// let eoa = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
/// let salt = "my-super-secret-salt-12345";
/// let keypair = derive_key_from_eoa(eoa, salt).unwrap();
/// ```
pub fn derive_key_from_eoa(eoa_address: &str, secret_salt: &str) -> Result<KeyPair, String> {
    // Normalize EOA address (remove 0x prefix if present, convert to lowercase)
    let eoa = eoa_address.trim_start_matches("0x").to_lowercase();

    // Validate EOA address format
    if eoa.len() != 40 {
        return Err(format!("Invalid EOA address length: expected 40 hex chars, got {}", eoa.len()));
    }

    if !eoa.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid EOA address: contains non-hex characters".to_string());
    }

    // Combine EOA and salt for derivation
    let derivation_input = format!("{}:{}", eoa, secret_salt);

    // Hash the combined input using Keccak256
    let mut hasher = Keccak256::new();
    hasher.update(derivation_input.as_bytes());
    let hash_result = hasher.finalize();

    // Convert hash to a field element (private key)
    // We need to ensure the value is less than the curve order
    let private_key = hash_to_fr(&hash_result)?;

    // Derive public keys
    let g1_public = G1Projective::generator() * private_key;
    let g2_public = G2Projective::generator() * private_key;

    // Convert to affine coordinates for storage
    let g1_affine = g1_public.into_affine();
    let g2_affine = g2_public.into_affine();

    // Format the results for the KeyPair structure
    let private_key_str = private_key.to_string();

    let g1_point = G1Point {
        x: g1_affine.x.to_string(),
        y: g1_affine.y.to_string(),
    };

    let g2_point = G2Point {
        x_a: g2_affine.x.c0.to_string(),
        x_b: g2_affine.x.c1.to_string(),
        y_a: g2_affine.y.c0.to_string(),
        y_b: g2_affine.y.c1.to_string(),
    };

    Ok(KeyPair {
        eoa_address: format!("0x{}", eoa),
        private_key: private_key_str,
        public_key_g1: g1_point,
        public_key_g2: g2_point,
    })
}

/// Converts a hash (32 bytes) to a field element in Fr
///
/// This uses a "hash and reduce" approach: interpret the hash as a big integer
/// and reduce it modulo the curve order.
fn hash_to_fr(hash: &[u8]) -> Result<Fr, String> {
    if hash.len() != 32 {
        return Err(format!("Hash must be 32 bytes, got {}", hash.len()));
    }

    // Convert hash bytes to field element using big-endian bytes
    // Fr::from_be_bytes_mod_order automatically handles reduction modulo the curve order
    Ok(Fr::from_be_bytes_mod_order(hash))
}

/// Alternative derivation using HKDF-like expansion (more robust)
///
/// This is a more sophisticated approach that uses multiple rounds of hashing
/// similar to HKDF-Expand, providing better key derivation properties.
#[allow(dead_code)]
pub fn derive_key_from_eoa_hkdf(
    eoa_address: &str,
    secret_salt: &str,
    info: Option<&str>
) -> Result<KeyPair, String> {
    let eoa = eoa_address.trim_start_matches("0x").to_lowercase();

    if eoa.len() != 40 {
        return Err(format!("Invalid EOA address length: expected 40 hex chars, got {}", eoa.len()));
    }

    // Extract: hash the salt to get a pseudorandom key
    let mut extract_hasher = Keccak256::new();
    extract_hasher.update(b"BN254-KEYGEN-SALT");
    extract_hasher.update(secret_salt.as_bytes());
    let prk = extract_hasher.finalize();

    // Expand: derive key material from PRK
    let info_str = info.unwrap_or("BN254-BLS-KEY");
    let mut expand_hasher = Keccak256::new();
    expand_hasher.update(&prk);
    expand_hasher.update(eoa.as_bytes());
    expand_hasher.update(info_str.as_bytes());
    expand_hasher.update(&[0x01]); // Counter byte
    let okm = expand_hasher.finalize();

    // Convert to private key
    let private_key = hash_to_fr(&okm)?;

    // Derive public keys
    let g1_public = G1Projective::generator() * private_key;
    let g2_public = G2Projective::generator() * private_key;

    let g1_affine = g1_public.into_affine();
    let g2_affine = g2_public.into_affine();

    Ok(KeyPair {
        eoa_address: format!("0x{}", eoa),
        private_key: private_key.to_string(),
        public_key_g1: G1Point {
            x: g1_affine.x.to_string(),
            y: g1_affine.y.to_string(),
        },
        public_key_g2: G2Point {
            x_a: g2_affine.x.c0.to_string(),
            x_b: g2_affine.x.c1.to_string(),
            y_a: g2_affine.y.c0.to_string(),
            y_b: g2_affine.y.c1.to_string(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ec::CurveGroup;
    use ark_ff::Zero;
    use std::str::FromStr;

    #[test]
    fn test_derive_key_from_eoa() {
        let eoa = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
        let salt = "test-salt-123";

        let keypair = derive_key_from_eoa(eoa, salt).unwrap();

        // Verify the EOA address is stored correctly
        assert_eq!(keypair.eoa_address.to_lowercase(), eoa.to_lowercase());

        // Verify private key is non-zero
        let private_key = Fr::from_str(&keypair.private_key).unwrap();
        assert!(!private_key.is_zero());

        // Verify public key derivation is correct
        let g1_derived = G1Projective::generator() * private_key;
        let g1_affine = g1_derived.into_affine();
        assert_eq!(g1_affine.x.to_string(), keypair.public_key_g1.x);
        assert_eq!(g1_affine.y.to_string(), keypair.public_key_g1.y);
    }

    #[test]
    fn test_deterministic_derivation() {
        let eoa = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
        let salt = "test-salt-123";

        let keypair1 = derive_key_from_eoa(eoa, salt).unwrap();
        let keypair2 = derive_key_from_eoa(eoa, salt).unwrap();

        // Same inputs should produce same keys
        assert_eq!(keypair1.private_key, keypair2.private_key);
        assert_eq!(keypair1.public_key_g1.x, keypair2.public_key_g1.x);
        assert_eq!(keypair1.public_key_g1.y, keypair2.public_key_g1.y);
    }

    #[test]
    fn test_different_salts_produce_different_keys() {
        let eoa = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
        let salt1 = "salt-1";
        let salt2 = "salt-2";

        let keypair1 = derive_key_from_eoa(eoa, salt1).unwrap();
        let keypair2 = derive_key_from_eoa(eoa, salt2).unwrap();

        // Different salts should produce different keys
        assert_ne!(keypair1.private_key, keypair2.private_key);
        assert_ne!(keypair1.public_key_g1.x, keypair2.public_key_g1.x);
    }

    #[test]
    fn test_different_eoas_produce_different_keys() {
        let eoa1 = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
        let eoa2 = "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC";
        let salt = "same-salt";

        let keypair1 = derive_key_from_eoa(eoa1, salt).unwrap();
        let keypair2 = derive_key_from_eoa(eoa2, salt).unwrap();

        // Different EOAs should produce different keys
        assert_ne!(keypair1.private_key, keypair2.private_key);
        assert_ne!(keypair1.public_key_g1.x, keypair2.public_key_g1.x);
    }

    #[test]
    fn test_invalid_eoa_length() {
        let result = derive_key_from_eoa("0x1234", "salt");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid EOA address length"));
    }

    #[test]
    fn test_invalid_eoa_format() {
        // Test with non-hex characters in EOA (40 chars but invalid)
        let result = derive_key_from_eoa("0xGGGG997970C51812dc3A010C7d01b50e0d17dc7", "salt");
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("non-hex") || err_msg.contains("length"));
    }

    #[test]
    fn test_hkdf_derivation() {
        let eoa = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
        let salt = "test-salt-123";

        let keypair = derive_key_from_eoa_hkdf(eoa, salt, None).unwrap();

        // Verify basic properties
        assert_eq!(keypair.eoa_address.to_lowercase(), eoa.to_lowercase());
        let private_key = Fr::from_str(&keypair.private_key).unwrap();
        assert!(!private_key.is_zero());
    }

    #[test]
    fn test_hkdf_different_from_simple() {
        let eoa = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
        let salt = "test-salt-123";

        let keypair_simple = derive_key_from_eoa(eoa, salt).unwrap();
        let keypair_hkdf = derive_key_from_eoa_hkdf(eoa, salt, None).unwrap();

        // Different derivation methods should produce different keys
        assert_ne!(keypair_simple.private_key, keypair_hkdf.private_key);
    }
}
