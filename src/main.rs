use bn254_rs::web;
use log::info;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init();

    // Start the server
    info!("Starting BN254 web service...");
    web::start_server().await
} 