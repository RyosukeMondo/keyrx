//! Layout management endpoints.

use axum::{extract::Path, routing::get, Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;

use super::error::ApiError;
use crate::config::layout_manager::LayoutManager;
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/layouts", get(list_layouts))
        .route("/layouts/:name", get(get_layout))
}

/// GET /api/layouts - List keyboard layouts
async fn list_layouts() -> Result<Json<Value>, ApiError> {
    // Wrap all blocking operations in spawn_blocking to prevent runtime starvation
    tokio::task::spawn_blocking(move || {
        let config_dir = get_config_dir()?;
        let lm = LayoutManager::new(config_dir.join("layouts"))
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        let layouts: Vec<String> = lm.list().iter().map(|layout| layout.name.clone()).collect();

        Ok::<Json<Value>, ApiError>(Json(json!({
            "layouts": layouts
        })))
    })
    .await
    .map_err(|e| ApiError::InternalError(format!("Task join error: {}", e)))?
}

/// GET /api/layouts/:name - Get layout KLE JSON
async fn get_layout(Path(name): Path<String>) -> Result<Json<Value>, ApiError> {
    // Wrap all blocking operations in spawn_blocking to prevent runtime starvation
    tokio::task::spawn_blocking(move || {
        let config_dir = get_config_dir()?;
        let lm = LayoutManager::new(config_dir.join("layouts"))
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        let layout = lm
            .get(&name)
            .ok_or_else(|| ApiError::NotFound(format!("Layout '{}' not found", name)))?;

        Ok::<Json<Value>, ApiError>(Json(layout.kle_json.clone()))
    })
    .await
    .map_err(|e| ApiError::InternalError(format!("Task join error: {}", e)))?
}

/// Get config directory path (cross-platform)
fn get_config_dir() -> Result<std::path::PathBuf, ApiError> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| ApiError::InternalError("Cannot determine config directory".to_string()))?;

    Ok(config_dir.join("keyrx"))
}
