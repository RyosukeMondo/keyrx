#[cfg(feature = "web")]
use axum::{Router, routing::get, Json};
#[cfg(feature = "web")]
use serde_json::{json, Value};

#[cfg(feature = "web")]
pub fn create_router() -> Router {
    Router::new()
        .route("/status", get(get_status))
        .route("/config", get(get_config))
}

#[cfg(feature = "web")]
async fn get_status() -> Json<Value> {
    Json(json!({
        "status": "running",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[cfg(feature = "web")]
async fn get_config() -> Json<Value> {
    Json(json!({
        "config": "placeholder"
    }))
}
