//! Daemon settings endpoints (REST twin of the WS-RPC `get/set_global_layout`,
//! both over [`SettingsService`](crate::services::SettingsService)).

use axum::{extract::State, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use typeshare::typeshare;

use super::error::ApiError;
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route(
        "/settings/global-layout",
        get(get_global_layout).put(set_global_layout),
    )
}

/// The default keyboard layout for devices without their own.
#[typeshare]
#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalLayout {
    /// Layout name (e.g. `ANSI_104`), or none when unset.
    pub layout: Option<String>,
}

/// GET /api/settings/global-layout
async fn get_global_layout(
    State(state): State<Arc<AppState>>,
) -> Result<Json<GlobalLayout>, ApiError> {
    let layout = state
        .settings_service
        .get_global_layout()
        .await
        .map_err(ApiError::InternalError)?;
    Ok(Json(GlobalLayout { layout }))
}

/// PUT /api/settings/global-layout
async fn set_global_layout(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<GlobalLayout>,
) -> Result<Json<GlobalLayout>, ApiError> {
    state
        .settings_service
        .set_global_layout(payload.layout.clone())
        .await
        .map_err(ApiError::BadRequest)?;
    Ok(Json(payload))
}
