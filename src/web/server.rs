use axum::{
    routing::{get, post},
    Router,
    response::Response,
    middleware::{self, Next},
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tower_http::trace::{TraceLayer, DefaultOnResponse, DefaultMakeSpan};
use tracing::{info, debug, Level};
use crate::web::handlers::{health_check, get_key_pair, list_key_pairs, scalar_mul, sign, get_registration_params};
use crate::web::store::Store;

// Log response middleware with request details
async fn log_response_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let query = req.uri().query().map(|q| q.to_owned());
    
    debug!(
        method = %method,
        path = %path,
        query = ?query,
        "Incoming request"
    );
    
    // Process the request
    let response = next.run(req).await;
    
    let status = response.status();
    
    if status.is_success() {
        debug!(
            method = %method,
            path = %path,
            status = %status,
            "Request completed successfully"
        );
    } else {
        info!(
            method = %method,
            path = %path,
            status = %status,
            "Request completed with error"
        );
    }
    
    Ok(response)
}

/// Start the web server on the specified host and port
pub async fn run_server(store: Store, host: &str, port: u16) -> std::io::Result<()> {
    // Logging is already initialized in main
    
    let shared_state = Arc::new(store);
    
    let bind_addr = format!("{}:{}", host, port);
    info!("Starting server at http://{}", bind_addr);
    
    debug!("Configuring application routes");
    
    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/key/:eoa_address", get(get_key_pair))
        .route("/keys", get(list_key_pairs))
        .route("/scalar_mul", post(scalar_mul))
        .route("/sign", post(sign))
        .route("/registration_params", post(get_registration_params))
        .layer(middleware::from_fn(log_response_middleware))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new()
                    .level(Level::DEBUG)
                    .include_headers(true))
                .on_response(DefaultOnResponse::new()
                    .level(Level::DEBUG)
                    .include_headers(true)
                    .latency_unit(tower_http::LatencyUnit::Micros))
        )
        .with_state(shared_state);
        
    debug!("Routes configured successfully");

    // Run the server
    debug!("Creating TCP listener on {}", bind_addr);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Server listening on http://{}", bind_addr);
    
    debug!("Starting axum server");
    axum::serve(listener, app).await?;
    info!("Server shutdown");
    
    Ok(())
}