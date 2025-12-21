#[cfg(feature = "web")]
use axum::{routing::get, Router};

#[cfg(feature = "web")]
#[allow(dead_code)]
pub fn create_router() -> Router {
    Router::new().route("/events", get(websocket_handler))
}

#[cfg(feature = "web")]
#[allow(dead_code)]
async fn websocket_handler() -> &'static str {
    "WebSocket endpoint placeholder"
}
