use axum::{
    routing::{get, post},
    Router,
    extract::{State, Path},
    response::{Json, IntoResponse},
    http::{StatusCode, Method},
};
use tower_http::cors::{CorsLayer, Any};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tokio::net::TcpListener;
use serde::{Serialize, Deserialize};
use zkipfs_proof_core::{ProofConfig, ContentSelection};

mod handlers;

#[derive(Clone)]
pub struct AppState {
    pub jobs: Arc<RwLock<HashMap<String, Job>>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "status", content = "result")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed(serde_json::Value),
    Failed(String),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Job {
    pub id: String,
    pub status: JobStatus,
    pub created_at: u64,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Initialize state
    let state = AppState {
        jobs: Arc::new(RwLock::new(HashMap::new())),
    };

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    // Setup router
    let app = Router::new()
        .route("/generate", post(handlers::generate_proof))
        .route("/status/:id", get(handlers::get_status))
        .route("/health", get(|| async { "OK" }))
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("Server listening on {}", addr);
    
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
