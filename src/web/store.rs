//! # Key Store Module V3
//!
//! This module manages BLS key pairs by combining:
//! 1. A static BLS key pool (key_0 through key_49)
//! 2. An EOA-to-key mapping file in the data directory
//!
//! ## Workflow
//! ```
//! bn254-rs --data-dir /path/to/data
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
use ethers::signers::{coins_bip39::English, MnemonicBuilder, Signer};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, trace, warn};

// Default paths for the embedded BLS key pool
const DEFAULT_BLS_KEY_POOL_PATH_CONTAINER: &str = "/app/data/keys.json";
const DEFAULT_BLS_KEY_POOL_PATH_LOCAL: &str = "./data/keys.json";
// Expected filenames in the data directory
const EOA_KEYMAP_FILENAME: &str = "eoa-keymap.json";
const KEYS_FILENAME: &str = "keys.json";

/// Check if we're running inside a Docker container
fn is_running_in_container() -> bool {
    // Check for /.dockerenv file (common Docker indicator)
    if Path::new("/.dockerenv").exists() {
        return true;
    }
    
    // Check if /proc/1/cgroup mentions docker or containerd
    if let Ok(cgroup) = fs::read_to_string("/proc/1/cgroup") {
        if cgroup.contains("docker") || cgroup.contains("containerd") {
            return true;
        }
    }
    
    // Check if we're in the /app directory (our container's WORKDIR)
    if let Ok(current_dir) = std::env::current_dir() {
        if current_dir.to_str() == Some("/app") {
            return true;
        }
    }
    
    false
}

/// Get the default BLS key pool path based on environment
fn get_default_bls_pool_path() -> &'static str {
    if is_running_in_container() {
        DEFAULT_BLS_KEY_POOL_PATH_CONTAINER
    } else {
        DEFAULT_BLS_KEY_POOL_PATH_LOCAL
    }
}

/// In-memory store for operator key pairs
pub struct Store {
    /// Map from EOA address to KeyPair
    players: HashMap<String, KeyPair>,
    /// Map from key_N format to KeyPair
    key_index: HashMap<String, KeyPair>,
}

impl Store {
    /// Create a new store from a data directory
    ///
    /// The data directory should contain:
    /// - eoa-keymap.json: EOA to key mappings
    /// - keys.json: BLS key pool (optional, falls back to default location)
    pub fn from_data_dir(data_dir: &str) -> Result<Self> {
        if data_dir.is_empty() || data_dir == "none" {
            // No data directory specified, load all keys from default pool
            info!("No data directory specified, loading all keys from default pool");
            return Self::load_all_from_pool(None);
        }

        let data_path = Path::new(data_dir);
        if !data_path.exists() {
            return Err(anyhow::anyhow!("Data directory does not exist: {}", data_dir));
        }

        let eoa_keymap_path = data_path.join(EOA_KEYMAP_FILENAME);
        let keys_path = data_path.join(KEYS_FILENAME);

        // Use keys.json from data directory if it exists, otherwise use default
        let bls_pool_path = if keys_path.exists() {
            Some(keys_path)
        } else {
            None
        };

        if !eoa_keymap_path.exists() {
            return Err(anyhow::anyhow!(
                "Required file {} not found in data directory: {}",
                EOA_KEYMAP_FILENAME,
                data_dir
            ));
        }

        // Load with the mapping from the data directory
        info!("Loading key store from data directory: {}", data_dir);
        Self::from_mapping(eoa_keymap_path.to_str().unwrap(), bls_pool_path)
    }

    /// Create a new store by loading keys from a file (backward compatibility)
    ///
    /// This method is kept for backward compatibility.
    /// If the file is an EOA mapping, it uses that.
    /// Otherwise, it loads all keys from the default pool.
    pub fn from_file(path: &str) -> Result<Self> {
        // First check if it's a path to a directory (new format)
        if Path::new(path).is_dir() {
            return Self::from_data_dir(path);
        }
        
        // Try to load as EOA mapping file directly
        if path != "none" && Path::new(path).exists() {
            match Self::from_mapping(path, None) {
                Ok(store) => return Ok(store),
                Err(_) => {
                    warn!("Failed to load as EOA mapping, loading all keys from pool");
                }
            }
        }
        
        // Fall back to loading all keys from pool without mapping
        Self::load_all_from_pool(None)
    }

    /// Load all keys from the pool without EOA mapping
    pub fn load_all_from_pool(keys_path: Option<PathBuf>) -> Result<Self> {
        let bls_pool = Self::load_bls_pool(keys_path)?;
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
    /// * `keys_path` - Optional path to the keys.json file (uses default if None)
    pub fn from_mapping(mapping_path: &str, keys_path: Option<PathBuf>) -> Result<Self> {
        info!("Initializing key store with mapping from: {}", mapping_path);

        // Load the BLS key pool first
        let bls_pool = Self::load_bls_pool(keys_path)?;

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
                warn!(
                    "BLS key {} not found in pool for EOA {}",
                    key_id, eoa_address
                );
            }
        }

        info!("Key store initialized with {} mappings", players.len());

        Ok(Self { players, key_index })
    }

    /// Create a new store by deriving accounts from a mnemonic phrase
    ///
    /// Derives the first 26 accounts from the mnemonic and maps them to key_0 through key_25
    /// from the BLS key pool.
    ///
    /// ## Arguments
    /// * `mnemonic_phrase` - The mnemonic phrase to derive accounts from
    pub fn from_mnemonic(mnemonic_phrase: &str) -> Result<Self> {
        info!("Deriving 26 accounts from mnemonic");

        // Load the BLS key pool first
        let bls_pool = Self::load_bls_pool(None)?;

        let mut players = HashMap::new();
        let mut key_index = HashMap::new();

        // Derive the first 26 accounts (0-25)
        for i in 0..26 {
            // Build the wallet using the HD path m/44'/60'/0'/0/{i}
            let wallet = MnemonicBuilder::<English>::default()
                .phrase(mnemonic_phrase)
                .index(i as u32)?
                .build()?;

            // Get the EOA address
            let eoa_address = format!("{:#x}", wallet.address());
            let key_id = format!("key_{}", i);

            debug!("Derived account {}: {} -> {}", i, eoa_address, key_id);

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

                trace!("Added key pair for account {} at {}", i, eoa_address);
            } else {
                warn!("BLS key {} not found in pool", key_id);
            }
        }

        info!("Derived {} accounts from mnemonic", players.len());

        Ok(Self { players, key_index })
    }

    /// Load the BLS key pool from the specified path or default location
    fn load_bls_pool(keys_path: Option<PathBuf>) -> Result<HashMap<String, BLSKeyData>> {
        let pool_path = if let Some(path) = keys_path {
            path.to_string_lossy().into_owned()
        } else {
            get_default_bls_pool_path().to_string()
        };
        
        let is_container = is_running_in_container();
        debug!(
            "Loading BLS key pool from {} (container: {})",
            pool_path, is_container
        );

        let content = fs::read_to_string(&pool_path)
            .with_context(|| format!("Failed to read BLS key pool from {}", pool_path))?;

        let json: Value =
            serde_json::from_str(&content).with_context(|| "Failed to parse BLS key pool JSON")?;

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

        let mapping: HashMap<String, String> =
            serde_json::from_str(&content).with_context(|| "Failed to parse EOA mapping JSON")?;

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
