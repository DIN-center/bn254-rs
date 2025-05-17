use actix_web::{web, App, HttpServer};
use log::{info, LevelFilter};
use env_logger::Builder;
use crate::web::handlers::{get_key_pair, list_key_pairs, scalar_mul, sign, get_registration_params};
use crate::web::store::Store;

/// Start the web server
pub async fn run_server(store: Store) -> std::io::Result<()> {
    // Configure logging
    Builder::new()
        .filter_level(LevelFilter::Debug)
        .format_timestamp_millis()
        .init();
    
    let store = web::Data::new(store);
    
    info!("Starting server at http://127.0.0.1:8080");
    
    HttpServer::new(move || {
        App::new()
            .app_data(store.clone())
            .route("/key/{eoa_address}", web::get().to(get_key_pair))
            .route("/keys", web::get().to(list_key_pairs))
            .route("/scalar_mul", web::post().to(scalar_mul))
            .route("/sign", web::post().to(sign))
            .route("/registration_params", web::post().to(get_registration_params))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
} 