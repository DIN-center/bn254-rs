use std::collections::HashMap;
use serde_json::Value;
use std::fs;
use crate::web::models::{KeyPair, G1Point, G2Point};
use anyhow::Result;

/// A simple in-memory store for key pairs
pub struct Store {
    players: HashMap<String, KeyPair>,
}

impl Store {
    /// Create a new store
    pub fn new() -> Result<Self> {
        let file_content = fs::read_to_string("src/web/players.json")?;
        let json: Value = serde_json::from_str(&file_content)?;
        
        let mut players = HashMap::new();
        
        if let Value::Object(obj) = json {
            for (_name, player) in obj {
                if let Value::Object(player_obj) = player {
                    let eoa_address = player_obj["pub"].as_str().unwrap_or_default().to_string();
                    let private_key = player_obj["bls"]["priv_key"].as_str().unwrap_or_default().to_string();
                    
                    let key_pair = KeyPair {
                        eoa_address: eoa_address.clone(),
                        private_key,
                        public_key_g1: G1Point {
                            x: player_obj["bls"]["g1_x"].as_str().unwrap_or_default().to_string(),
                            y: player_obj["bls"]["g1_y"].as_str().unwrap_or_default().to_string(),
                        },
                        public_key_g2: G2Point {
                            x_a: player_obj["bls"]["g2_x_0"].as_str().unwrap_or_default().to_string(),
                            x_b: player_obj["bls"]["g2_x_1"].as_str().unwrap_or_default().to_string(),
                            y_a: player_obj["bls"]["g2_y_0"].as_str().unwrap_or_default().to_string(),
                            y_b: player_obj["bls"]["g2_y_1"].as_str().unwrap_or_default().to_string(),
                        },
                    };
                    
                    players.insert(eoa_address, key_pair);
                }
            }
        }
        
        Ok(Self { players })
    }

    /// Get a key pair by EOA address
    pub fn get_key_pair(&self, eoa_address: &str) -> Option<&KeyPair> {
        self.players.get(eoa_address)
    }

    /// List all key pairs
    pub fn list_key_pairs(&self) -> Vec<&KeyPair> {
        self.players.values().collect()
    }
} 