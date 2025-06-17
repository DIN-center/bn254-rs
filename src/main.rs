//! # BN254 Key Management Service
//! 
//! This is the main entry point for the BN254 key management web service.
//! The service provides a RESTful API for BLS signature operations using the BN254 curve.
//! 
//! ## Architecture Overview
//! 
//! The service consists of:
//! - Web server running on port 3000 (for backward compatibility)
//! - RESTful API endpoints for key management and signing operations
//! - In-memory key store loaded from `./data/players.json`
//! 
//! ## Maintenance Notes
//! 
//! ### Port Configuration
//! The service runs on port 3000 to maintain backward compatibility with existing clients.
//! If you need to change the port, update:
//! 1. `src/web/server.rs` - Change the bind address
//! 2. `README.md` - Update all documentation references
//! 3. Notify all client applications of the change
//! 
//! ### Logging Configuration
//! - Default log level: info
//! - Can be overridden via CLI: `--log-level debug`
//! - Can be overridden via env: `RUST_LOG=debug`
//! - Environment variable takes precedence over CLI argument
//! 
//! ### Error Handling
//! The service will exit with code 1 if:
//! - The web server fails to start (port already in use, etc.)
//! - The key store file cannot be loaded
//! 
//! ## Usage
//! ```bash
//! # Run with default settings
//! cargo run --bin bn254-rs
//! 
//! # Run with custom database file
//! cargo run --bin bn254-rs -- --db /path/to/keys.json
//! 
//! # Run on a different port
//! cargo run --bin bn254-rs -- --port 8080
//! 
//! # Run on all interfaces (for Docker)
//! cargo run --bin bn254-rs -- --host 0.0.0.0
//! 
//! # Run with debug logging
//! cargo run --bin bn254-rs -- --log-level debug
//! 
//! # Run with all custom options
//! cargo run --bin bn254-rs -- --db /path/to/keys.json --host 0.0.0.0 --port 8080 --log-level debug
//! 
//! # Run with environment variable
//! RUST_LOG=trace cargo run --bin bn254-rs
//! ```

mod web;

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Command-line arguments for the BN254 Key Management Service
#[derive(Parser, Debug)]
#[command(
    name = "bn254-rs",
    author,
    version,
    about = "BN254 Key Management Service - RESTful API for BLS operations",
    long_about = "A web service that provides BLS signature operations using the BN254 elliptic curve.\n\
                  Maintains backward compatibility with EigenLayer AVS operators."
)]
struct Args {
    /// Set the log level (error, warn, info, debug, trace)
    /// 
    /// Examples:
    /// - error: Only show errors
    /// - warn: Show warnings and errors
    /// - info: Show informational messages (default)
    /// - debug: Show debug information including request/response details
    /// - trace: Show all information including internal function calls
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    /// Path to the JSON database file containing operator keys
    /// 
    /// This file should contain the BLS key pairs for operators.
    /// If not specified, defaults to "players.json" in the current directory.
    #[arg(short, long, default_value = "players.json")]
    db: String,
    
    /// Port to run the web server on
    /// 
    /// The service will bind to this port on the specified address.
    /// If not specified, defaults to port 3000.
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
    
    /// Host address to bind the server to
    /// 
    /// Use 0.0.0.0 to listen on all interfaces, or 127.0.0.1 for localhost only.
    /// If not specified, defaults to 0.0.0.0 for Docker compatibility.
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    // Initialize tracing with the following precedence:
    // 1. RUST_LOG environment variable (if set)
    // 2. --log-level CLI argument
    // 3. Default to "info"
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&args.log_level))
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting BN254 Key Management Service with log level: {}", args.log_level);
    tracing::info!("Using database file: {}", args.db);
    tracing::info!("Server will listen on {}:{}", args.host, args.port);
    
    // Start the web server
    // This will:
    // 1. Load the key store from the specified database file
    // 2. Start the HTTP server on the specified address and port
    // 3. Set up all API routes
    if let Err(e) = web::start_server(&args.db, &args.host, args.port).await {
        tracing::error!("Error running web service: {}", e);
        std::process::exit(1);
    }
} 