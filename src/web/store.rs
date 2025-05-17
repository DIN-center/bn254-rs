//! # Key Store Module
//!
//! This module manages the in-memory storage of BLS key pairs for operators.
//! Keys are loaded from a JSON file at startup and kept in memory for fast access.
//!
//! ## Data Format
//!
//! The store expects a JSON file at `./data/players.json` with the following structure:
//! ```json
//! {
//!   "operator_name": {
//!     "pub": "0x...",  // EOA address
//!     "bls": {
//!       "priv_key": "...",  // Private key as decimal string
//!       "g1_x": "...", "g1_y": "...",  // G1 public key
//!       "g2_x_0": "...", "g2_x_1": "...",  // G2 public key
//!       "g2_y_0": "...", "g2_y_1": "..."
//!     }
//!   }
//! }
//! ```
//!
//! ## Maintenance Notes
//!
//! ### File Location
//! - Current: `./data/players.json`
//! - Path is hardcoded - changes require updates in multiple places
//! - File must exist and be readable at startup
//!
//! ### Error Handling
//! - Missing file: Service will fail to start
//! - Invalid JSON: Service will fail to start
//! - Missing fields: Player is skipped with warning
//!
//! ### Performance
//! - All keys loaded into memory at startup
//! - O(1) lookup by EOA address
//! - No runtime file I/O after initialization
//!
//! ### Security
//! - Private keys stored in plaintext in JSON
//! - File permissions should restrict access
//! - Consider encryption for production use
//!
//! ## Future Improvements
//! - Support for encrypted key storage
//! - Dynamic key loading/reloading
//! - Database backend option
//! - Key rotation support

use crate::web::models::{G1Point, G2Point, KeyPair};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use tracing::{debug, error, info, trace, warn};

/// In-memory store for operator key pairs
///
/// ## Implementation Details
/// - Uses HashMap for O(1) lookups by EOA address
/// - Loaded once at startup from JSON file
/// - Thread-safe when wrapped in Arc (as done in server)
pub struct Store {
    players: HashMap<String, KeyPair>,
}

impl Store {
    /// Create a new store by loading keys from default JSON file
    ///
    /// This is a convenience method that loads from `data/players.json`
    #[allow(unused)]
    pub fn new() -> Result<Self> {
        Self::from_file("data/players.json")
    }

    /// Create a new store by loading keys from specified JSON file
    ///
    /// ## File Format
    /// See module documentation for expected JSON structure
    ///
    /// ## Error Cases
    /// - File not found: Returns IO error
    /// - Invalid JSON: Returns parse error
    /// - Missing required fields: Player skipped, warning logged
    ///
    /// ## Arguments
    /// * `path` - Path to the JSON file containing key pairs
    pub fn from_file(path: &str) -> Result<Self> {
        info!("Initializing key store from: {}", path);
        debug!("Loading key pairs from {}", path);

        let file_content = match fs::read_to_string(path) {
            Ok(content) => {
                debug!("Successfully read {} file", path);
                content
            }
            Err(e) => {
                error!("Failed to read {} file: {}", path, e);
                return Err(e.into());
            }
        };

        let json: Value = match serde_json::from_str(&file_content) {
            Ok(parsed) => {
                debug!("Successfully parsed JSON data from {}", path);
                parsed
            }
            Err(e) => {
                error!("Failed to parse JSON data from {}: {}", path, e);
                return Err(e.into());
            }
        };

        let mut players = HashMap::new();

        if let Value::Object(obj) = json {
            info!("Found {} players in the JSON data", obj.len());

            for (name, player) in obj {
                debug!("Processing player: {}", name);

                if let Value::Object(player_obj) = player {
                    let eoa_address = player_obj["pub"].as_str().unwrap_or_default().to_string();

                    if eoa_address.is_empty() {
                        warn!("Empty EOA address for player {}, skipping", name);
                        continue;
                    }

                    let private_key = player_obj["bls"]["priv_key"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();

                    if private_key.is_empty() {
                        warn!("Empty private key for player {}, skipping", name);
                        continue;
                    }

                    debug!("Creating key pair for EOA address: {}", eoa_address);

                    let key_pair = KeyPair {
                        eoa_address: eoa_address.clone(),
                        private_key,
                        public_key_g1: G1Point {
                            x: player_obj["bls"]["g1_x"]
                                .as_str()
                                .unwrap_or_default()
                                .to_string(),
                            y: player_obj["bls"]["g1_y"]
                                .as_str()
                                .unwrap_or_default()
                                .to_string(),
                        },
                        public_key_g2: G2Point {
                            x_a: player_obj["bls"]["g2_x_0"]
                                .as_str()
                                .unwrap_or_default()
                                .to_string(),
                            x_b: player_obj["bls"]["g2_x_1"]
                                .as_str()
                                .unwrap_or_default()
                                .to_string(),
                            y_a: player_obj["bls"]["g2_y_0"]
                                .as_str()
                                .unwrap_or_default()
                                .to_string(),
                            y_b: player_obj["bls"]["g2_y_1"]
                                .as_str()
                                .unwrap_or_default()
                                .to_string(),
                        },
                    };

                    players.insert(eoa_address, key_pair);
                    debug!("Added key pair for player {}", name);
                } else {
                    warn!("Player {} is not an object, skipping", name);
                }
            }
        } else {
            warn!("JSON data is not an object");
        }

        info!(
            "Key store initialized with {} key pairs from {}",
            players.len(),
            path
        );
        Ok(Self { players })
    }

    /// Get a key pair by EOA address
    pub fn get_key_pair(&self, eoa_address: &str) -> Option<&KeyPair> {
        debug!(eoa_address = %eoa_address, "Looking up key pair");
        trace!("Total key pairs in store: {}", self.players.len());

        let key_pair = self.players.get(eoa_address);

        if key_pair.is_some() {
            debug!(eoa_address = %eoa_address, "Key pair found");
        } else {
            debug!(eoa_address = %eoa_address, "Key pair not found");
            trace!(
                "Available EOA addresses: {:?}",
                self.players.keys().collect::<Vec<_>>()
            );
        }

        key_pair
    }

    /// List all key pairs
    pub fn list_key_pairs(&self) -> Vec<&KeyPair> {
        debug!("Listing all key pairs, count: {}", self.players.len());
        self.players.values().collect()
    }
}

