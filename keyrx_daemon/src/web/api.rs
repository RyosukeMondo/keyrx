#[cfg(feature = "web")]
use axum::{routing::get, Json, Router};
#[cfg(feature = "web")]
use serde_json::{json, Value};

#[cfg(feature = "web")]
#[allow(dead_code)]
pub fn create_router() -> Router {
    Router::new()
        .route("/status", get(get_status))
        .route("/config", get(get_config))
}

#[cfg(feature = "web")]
#[allow(dead_code)]
async fn get_status() -> Json<Value> {
    Json(json!({
        "status": "running",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[cfg(feature = "web")]
#[allow(dead_code)]
async fn get_config() -> Json<Value> {
    Json(json!({
        "config": "placeholder"
    }))
}
