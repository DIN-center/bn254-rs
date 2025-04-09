use actix_web::{web, App, HttpServer};
use log::info;
use crate::web::handlers::{scalar_mul, sign};
use crate::web::store::Store;

/// Start the web server
pub async fn run_server(store: Store) -> std::io::Result<()> {
    let store = web::Data::new(store);
    
    info!("Starting server at http://127.0.0.1:8080");
    
    HttpServer::new(move || {
        App::new()
            .app_data(store.clone())
            .service(
                web::scope("/api")
                    .route("/scalar_mul", web::post().to(scalar_mul))
                    .route("/sign", web::post().to(sign))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
} 