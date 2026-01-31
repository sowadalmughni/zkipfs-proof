use axum::{
    routing::{get, post, delete},
    Router,
    http::Method,
    middleware,
};
use tower_http::cors::{CorsLayer, Any};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tokio::net::TcpListener;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

mod db;
mod state;
mod auth;
mod handlers;

use crate::state::AppState;
use crate::db::Db;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    // Initialize Database (SQLite)
    // Ensure the data directory exists or use a local file
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://zkipfs.db".to_string());
    
    let db = match Db::new(&db_url).await {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize state
    let state = AppState {
        jobs: Arc::new(RwLock::new(HashMap::new())),
        db,
    };

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    // Rate Limiter Configuration (Global fallback)
    // Limit: 60 requests per second per IP (Burstable)
    // For Enterprise API, we might use ApiKeyExtractor in the router layer.
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(60)
            .burst_size(10)
            .finish()
            .unwrap(),
    );

    // Routes
    let app = Router::new()
        // Public / Legacy Routes (Rate limited by IP)
        .route("/generate", post(handlers::generate_proof)) 
        .route("/status/:id", get(handlers::get_status))
        .route("/health", get(|| async { "OK" }))
        // Enterprise API v1 (Authenticated & Rate Limited)
        .nest("/api/v1", Router::new() 
            .route("/keys", post(handlers::create_api_key))
            .route("/keys", get(handlers::list_api_keys)) // In real app, restrict this!
            .route("/keys/:id", delete(handlers::revoke_api_key))
            .route("/generate", post(handlers::generate_proof)) // Authenticated generation
            // Add Auth Middleware to this nested router
            .layer(middleware::from_fn_with_state(state.clone(), auth::auth::auth_middleware))
        )
        .layer(GovernorLayer {
            config: Box::leak(governor_conf),
        })
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("Server listening on {}", addr);
    
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
