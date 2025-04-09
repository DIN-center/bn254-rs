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

    info!("Starting server at http://127.0.0.1:8080");
    
    HttpServer::new(move || {
        App::new()
            .app_data(store.clone())
            .service(
                web::scope("/api")
                    .route("/keys/{eoa_address}", web::get().to(handlers::get_key_pair))
                    .route("/keys", web::get().to(handlers::list_key_pairs))
                    .route("/scalar_mul", web::post().to(handlers::scalar_mul))
                    .route("/sign", web::post().to(handlers::sign))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
} 