//! # Key Store Module V3
//!
//! This module manages BLS key pairs by combining:
//! 1. A static BLS key pool (key_0 through key_49) 
//! 2. An EOA-to-key mapping file passed via --db flag
//!
//! ## Workflow
//! ```
//! bn254-rs --db /path/to/eoa-keymap.json
//! ```
//!
//! ## EOA Keymap Format
//! ```json
//! {
//!   "0x70997970C51812dc3A010C7d01b50e0d17dc79C8": "key_0",
//!   "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC": "key_1",
//!   "0x90F79bf6EB2c4f870365E785982E1f101E93b906": "key_2"
//! }
//! ```

use crate::web::models::{G1Point, G2Point, KeyPair};
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use tracing::{debug, info, trace, warn};

// Path to the embedded BLS key pool
const BLS_KEY_POOL_PATH: &str = "src/web/bn254pool/keys.json";

/// In-memory store for operator key pairs
pub struct Store {
    /// Map from EOA address to KeyPair
    players: HashMap<String, KeyPair>,
    /// Map from key_N format to KeyPair
    key_index: HashMap<String, KeyPair>,
}

impl Store {
    /// Create a new store by loading keys from default JSON file (backward compatibility)
    /// 
    /// This method is kept for backward compatibility but will attempt to use
    /// the new mapping format if the file contains EOA-to-key mappings.
    pub fn from_file(path: &str) -> Result<Self> {
        // First try to load as EOA mapping
        match Self::from_mapping(path) {
            Ok(store) => Ok(store),
            Err(_) => {
                // Fall back to loading all keys from pool without mapping
                warn!("Failed to load as EOA mapping, loading all keys from pool");
                Self::load_all_from_pool()
            }
        }
    }
    
    /// Load all keys from the pool without EOA mapping (for testing)
    fn load_all_from_pool() -> Result<Self> {
        let bls_pool = Self::load_bls_pool()?;
        let mut players = HashMap::new();
        let mut key_index = HashMap::new();
        
        // Create dummy EOA addresses for all keys
        for (key_id, bls_data) in bls_pool {
            let dummy_eoa = format!("0x{}", key_id);
            let key_pair = KeyPair {
                eoa_address: dummy_eoa.clone(),
                private_key: bls_data.priv_key,
                public_key_g1: G1Point {
                    x: bls_data.g1_x,
                    y: bls_data.g1_y,
                },
                public_key_g2: G2Point {
                    x_a: bls_data.g2_x_0,
                    x_b: bls_data.g2_x_1,
                    y_a: bls_data.g2_y_0,
                    y_b: bls_data.g2_y_1,
                },
            };
            
            players.insert(dummy_eoa, key_pair.clone());
            key_index.insert(key_id, key_pair);
        }
        
        Ok(Self { players, key_index })
    }
    /// Create a new store by loading the EOA mapping and BLS key pool
    ///
    /// ## Arguments
    /// * `mapping_path` - Path to the EOA-to-key mapping JSON file
    pub fn from_mapping(mapping_path: &str) -> Result<Self> {
        info!("Initializing key store with mapping from: {}", mapping_path);
        
        // Load the BLS key pool first
        let bls_pool = Self::load_bls_pool()?;
        
        // Load the EOA-to-key mapping
        let eoa_mapping = Self::load_eoa_mapping(mapping_path)?;
        
        // Construct the final store
        let mut players = HashMap::new();
        let mut key_index = HashMap::new();
        
        // Process each EOA mapping
        for (eoa_address, key_id) in eoa_mapping {
            debug!("Processing mapping: {} -> {}", eoa_address, key_id);
            
            // Get the BLS key from the pool
            if let Some(bls_data) = bls_pool.get(&key_id) {
                let key_pair = KeyPair {
                    eoa_address: eoa_address.clone(),
                    private_key: bls_data.priv_key.clone(),
                    public_key_g1: G1Point {
                        x: bls_data.g1_x.clone(),
                        y: bls_data.g1_y.clone(),
                    },
                    public_key_g2: G2Point {
                        x_a: bls_data.g2_x_0.clone(),
                        x_b: bls_data.g2_x_1.clone(),
                        y_a: bls_data.g2_y_0.clone(),
                        y_b: bls_data.g2_y_1.clone(),
                    },
                };
                
                // Add to both indices
                players.insert(eoa_address.clone(), key_pair.clone());
                key_index.insert(key_id.clone(), key_pair);
                
                debug!("Added key pair for EOA {} using {}", eoa_address, key_id);
            } else {
                warn!("BLS key {} not found in pool for EOA {}", key_id, eoa_address);
            }
        }
        
        info!(
            "Key store initialized with {} mappings",
            players.len()
        );
        
        Ok(Self { players, key_index })
    }
    
    /// Load the BLS key pool from the embedded file
    fn load_bls_pool() -> Result<HashMap<String, BLSKeyData>> {
        debug!("Loading BLS key pool from {}", BLS_KEY_POOL_PATH);
        
        let content = fs::read_to_string(BLS_KEY_POOL_PATH)
            .with_context(|| format!("Failed to read BLS key pool from {}", BLS_KEY_POOL_PATH))?;
        
        let json: Value = serde_json::from_str(&content)
            .with_context(|| "Failed to parse BLS key pool JSON")?;
        
        let mut pool = HashMap::new();
        
        if let Some(bls_keys) = json.get("bls_keys").and_then(|v| v.as_object()) {
            for (key_id, key_data) in bls_keys {
                if let Ok(bls_data) = serde_json::from_value::<BLSKeyData>(key_data.clone()) {
                    pool.insert(key_id.clone(), bls_data);
                    trace!("Loaded BLS key: {}", key_id);
                } else {
                    warn!("Failed to parse BLS key data for {}", key_id);
                }
            }
        }
        
        info!("Loaded {} BLS keys from pool", pool.len());
        Ok(pool)
    }
    
    /// Load the EOA-to-key mapping file
    fn load_eoa_mapping(path: &str) -> Result<HashMap<String, String>> {
        debug!("Loading EOA mapping from {}", path);
        
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read EOA mapping from {}", path))?;
        
        let mapping: HashMap<String, String> = serde_json::from_str(&content)
            .with_context(|| "Failed to parse EOA mapping JSON")?;
        
        info!("Loaded {} EOA mappings", mapping.len());
        Ok(mapping)
    }
    
    /// Get a key pair by EOA address or key ID
    pub fn get_key_pair(&self, identifier: &str) -> Option<&KeyPair> {
        debug!(identifier = %identifier, "Looking up key pair");
        
        // First check if it's a key_N format
        if identifier.starts_with("key_") {
            if let Some(key_pair) = self.key_index.get(identifier) {
                debug!(identifier = %identifier, "Key pair found via key index");
                return Some(key_pair);
            }
        }
        
        // Fall back to EOA lookup
        let key_pair = self.players.get(identifier);
        
        if key_pair.is_some() {
            debug!(identifier = %identifier, "Key pair found via EOA address");
        } else {
            debug!(identifier = %identifier, "Key pair not found");
        }
        
        key_pair
    }
    
    /// List all key pairs
    pub fn list_key_pairs(&self) -> Vec<&KeyPair> {
        debug!("Listing all key pairs, count: {}", self.players.len());
        self.players.values().collect()
    }
    
    /// List all key pairs as a map including both EOA and key_N indices
    pub fn list_all_keys(&self) -> HashMap<String, &KeyPair> {
        let mut all_keys = HashMap::new();
        
        // Add all EOA mappings
        for (eoa, key_pair) in &self.players {
            all_keys.insert(eoa.clone(), key_pair);
        }
        
        // Add all key_N mappings
        for (key_id, key_pair) in &self.key_index {
            all_keys.insert(key_id.clone(), key_pair);
        }
        
        all_keys
    }
}

/// Internal structure for BLS key data from the pool
#[derive(serde::Deserialize, Clone)]
struct BLSKeyData {
    priv_key: String,
    g1_x: String,
    g1_y: String,
    g2_x_0: String,
    g2_x_1: String,
    g2_y_0: String,
    g2_y_1: String,
}