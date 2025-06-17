//! # Web Service Module
//! 
//! This module implements the BN254 key management web service.
//! It provides a RESTful API for BLS signature operations compatible with EigenLayer AVS operators.
//! 
//! ## Module Structure
//! 
//! - `server`: HTTP server setup and middleware configuration
//! - `handlers`: API endpoint implementations
//! - `models`: Request/response data structures
//! - `store`: Key storage and retrieval logic
//! 
//! ## Maintenance Notes
//! 
//! ### Adding New Endpoints
//! 1. Define request/response models in `models.rs`
//! 2. Implement handler function in `handlers.rs`
//! 3. Add route in `server.rs`
//! 4. Update API documentation in README.md
//! 
//! ### API Design
//! - Port 3000 for service endpoints
//! - `/sign` endpoint expects `message` field with concatenated G1 coordinates
//! - G2 points in responses use array format for x and y coordinates
//! 
//! ### Error Handling
//! - All handlers return `impl IntoResponse` for flexible error handling
//! - HTTP status codes are used to indicate error types:
//!   - 400: Bad Request (invalid input)
//!   - 404: Not Found (key not found)
//!   - 500: Internal Server Error (processing errors)
//! 
//! ### Performance Considerations
//! - Key store is loaded once at startup and kept in memory
//! - No database queries during request handling
//! - All cryptographic operations are CPU-bound
//! 
//! ## Example Usage
//! ```rust
//! // Start the server
//! web::start_server().await?;
//! ```

pub mod models;
pub mod store;
pub mod handlers;
pub mod server;

use tracing::{info, error};
use std::io;

/// Start the web service with a custom database path, bind address, and port
/// 
/// This function:
/// 1. Initializes the key store from the specified database file
/// 2. Starts the HTTP server on the specified address and port
/// 3. Sets up all API routes with tracing middleware
/// 
/// # Arguments
/// 
/// * `db_path` - Path to the JSON database file containing operator keys
/// * `bind_addr` - IP address to bind the server to (e.g., "0.0.0.0" or "127.0.0.1")
/// * `port` - Port number to bind the server to
/// 
/// # Errors
/// 
/// Returns an error if:
/// - The key store file cannot be read or parsed
/// - The server cannot bind to the specified address and port
/// - Any other I/O error occurs
/// 
/// # Maintenance Notes
/// 
/// If the key store location changes:
/// 1. Update the path in `store.rs`
/// 2. Update documentation in README.md
/// 3. Ensure the new location is accessible to the service
/// 
/// Port Configuration:
/// - Port is now configurable via CLI argument
/// - Default port is 3000 for backward compatibility
/// - Bind address defaults to 0.0.0.0 for Docker compatibility
pub async fn start_server(db_path: &str, bind_addr: &str, port: u16) -> io::Result<()> {
    // Initialize store from JSON file
    // The store loads all operator keys into memory at startup
    let store = match store::Store::from_file(db_path) {
        Ok(store) => store,
        Err(e) => {
            error!("Failed to initialize store from {}: {}", db_path, e);
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }
    };

    info!("Loaded key store from: {}", db_path);
    info!("Starting server at http://{}:{}", bind_addr, port);
    
    // Run the server with the initialized store, bind address, and port
    // This will set up all routes and start listening for connections
    server::run_server(store, bind_addr, port).await
}