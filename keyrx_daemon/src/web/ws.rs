#[cfg(feature = "web")]
use axum::{Router, routing::get};

#[cfg(feature = "web")]
pub fn create_router() -> Router {
    Router::new()
        .route("/events", get(websocket_handler))
}

#[cfg(feature = "web")]
async fn websocket_handler() -> &'static str {
    "WebSocket endpoint placeholder"
}
