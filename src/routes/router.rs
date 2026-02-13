use axum::{Router, routing::{get, post}};
use tower_http::trace::TraceLayer;

use crate::handlers::{health, handle_snmpv2c};

pub fn create_router() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health", get(health))
        .route("/home", post(handle_snmpv2c))
        .layer(TraceLayer::new_for_http())
}