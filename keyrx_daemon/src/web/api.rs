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

#[cfg(all(test, feature = "web"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_router() {
        let router = create_router();
        // Just verify router can be created
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[tokio::test]
    async fn test_get_status() {
        let result = get_status().await;
        let value = result.0;
        assert_eq!(value["status"], "running");
    }

    #[tokio::test]
    async fn test_get_config() {
        let result = get_config().await;
        let value = result.0;
        assert_eq!(value["config"], "placeholder");
    }
}
