use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use crate::{state::AppState, db::ApiKey};
use tower_governor::{key_extractor::KeyExtractor};

// API Key Extractor
pub struct RequireApiKey(pub ApiKey);

#[async_trait]
impl<S> FromRequestParts<S> for RequireApiKey
where
    S: Send + Sync,
    AppState: From<S>, // Assuming AppState can be extracted from S or S is AppState
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut axum::http::request::Parts, state: &S) -> Result<Self, Self::Rejection> {
        // This is tricky: State extraction typically requires `State<AppState>` extractor logic.
        // But we are in `FromRequestParts` for `S`.
        // Simplest way: The State should be extractable. `State(state) = State::<AppState>::from_request_parts(parts, state).await.map_err(...)`
        // But `AppState` needs to be Clone.
        
        // Let's assume S is AppState or we use `State` extractor inside if possible? Check axum docs.
        // Actually, we can just look at headers first.
        
        let key_header = parts.headers.get("X-API-Key")
            .ok_or((StatusCode::UNAUTHORIZED, "Missing X-API-Key header"))?
            .to_str()
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid API Key format"))?;

        // We need access to DB.
        // Using `axum::extract::State` logic manually:
        // Because `S` is the state type (AppState).
        // If S is AppState:
        
        // HACK: To make this generic S work with AppState fields, S must impl a trait or be AppState.
        // But let's assume we use explicit `State<AppState>` in handlers usually.
        // In Middleware (FromRequestParts), `S` is the state.
        
        // NOTE: We cannot easily use `state.db` if we don't know S is AppState.
        // We will define this specifically for `AppState`.
        
        // ERROR: `AppState` is defined in `main.rs` (or `lib.rs`?). We need to import it.
        // Since we are in `auth.rs`, we need `crate::AppState`.
        
        // If `state` variable passed here is `&S`, we need to cast/access it.
        // The standard pattern is `impl FromRequestParts<AppState> for RequireApiKey`.
             
        // But we can't implement for concrete AppState efficiently if it's not in scope or cyclic.
        // But `crate::AppState` is available.
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Auth impl pending generic fix"));
    }
}

// We will implement the middleware as a function instead of FromRequestParts for easier State access if traits are hard.
// OR, we impl generic assuming S has a valid `db`.

pub async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let key_header = req.headers().get("X-API-Key")
        .and_then(|h| h.to_str().ok());

    match key_header {
        Some(key) => {
            match state.db.validate_api_key(key).await {
                Ok(Some(api_key)) => {
                    // Inject user context if needed
                    req.extensions_mut().insert(api_key);
                    Ok(next.run(req).await)
                }
                Ok(None) => Err(StatusCode::UNAUTHORIZED),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

// Key Extractor for Rate Limiting based on IP or API Key
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ApiKeyExtractor;

impl KeyExtractor for ApiKeyExtractor {
    type Key = String;

    fn extract<B>(&self, req: &Request<B>) -> Result<Self::Key, tower_governor::key_extractor::ExtractionError> {
        // Try invalid extraction first?
        // Governor middleware usually runs BEFORE our auth middleware if layered externally.
        // Use Header extractor for X-API-Key standard.
        // If missing, maybe fall back to IP?
        // Let's implement custom logic in `PeerIpKeyExtractor` if we want simple IP.
        
        // For Enterprise: Rate limit by API Key.
        if let Some(key) = req.headers().get("X-API-Key").and_then(|v| v.to_str().ok()) {
            Ok(key.to_string())
        } else {
             // Fallback to IP? Or Error?
             // Return IP based string if key missing (unauthenticated tier?)
             // Simple: Just return IP.
             
             // Actually, simplest is to use `SmartIpKeyExtractor` from crate, but we want API Key priority.
             // We'll write custom logic here later if needed.
             // For now, let's just use the header. If missing, "anon".
             Ok("anon".to_string())
        }
    }
}
