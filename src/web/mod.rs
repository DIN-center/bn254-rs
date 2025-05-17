pub mod models;
pub mod store;
pub mod handlers;

use actix_web::{web, App, HttpServer};
use log::{info, error};

pub async fn start_server() -> std::io::Result<()> {
    // Initialize store
    let store = match store::Store::new() {
        Ok(store) => store,
        Err(e) => {
            error!("Failed to initialize store: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
        }
    };
    let store = web::Data::new(store);
    
    // Initialize Anvil contract
    handlers::init_contract();

    info!("🚀 Starting BN254 Web Service");
    info!("📡 Server running at: http://127.0.0.1:8080");
    info!("Available endpoints:");
    info!("  GET  /api/keys/{{eoa_address}} - Get key pair by EOA address");
    info!("  GET  /api/keys              - List all key pairs");
    info!("  POST /api/scalar_mul        - Perform scalar multiplication");
    info!("  POST /api/sign              - Sign a message");
    info!("  POST /api/registration_params - Get registration parameters");
    
    HttpServer::new(move || {
        App::new()
            .app_data(store.clone())
            .service(
                web::scope("/api")
                    .route("/keys/{eoa_address}", web::get().to(handlers::get_key_pair))
                    .route("/keys", web::get().to(handlers::list_key_pairs))
                    .route("/scalar_mul", web::post().to(handlers::scalar_mul))
                    .route("/sign", web::post().to(handlers::sign))
                    .route("/registration_params", web::post().to(handlers::get_registration_params_anvil))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
} 