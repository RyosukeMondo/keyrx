//! Configuration management endpoints for the active profile.
//!
//! Thin adapters over [`ConfigService`](crate::services::ConfigService), the
//! same service WS-RPC uses: it resolves the active profile from the profile
//! manager, and every write compiles the profile and reloads the daemon if the
//! profile is loaded.

use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use super::error::ApiError;
use crate::config::rhai_generator::KeyAction;
use crate::services::config_service::ConfigError;
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/config", get(get_config).put(update_config))
        .route("/config/key-mappings", post(set_key_mapping))
        .route("/config/key-mappings/:id", delete(delete_key_mapping))
        .route("/layers", get(list_layers))
}

fn to_api_error(e: ConfigError) -> ApiError {
    match e {
        ConfigError::ProfileNotFound(_) | ConfigError::FileNotFound => {
            ApiError::NotFound(e.to_string())
        }
        ConfigError::ConfigTooLarge
        | ConfigError::InvalidConfig(_)
        | ConfigError::LayerNotFound(_)
        | ConfigError::InvalidKeyName(_) => ApiError::BadRequest(e.to_string()),
        ConfigError::IoError(_) | ConfigError::GeneratorError(_) => {
            ApiError::InternalError(e.to_string())
        }
    }
}

/// GET /api/config - Base mappings and layers of the active profile
async fn get_config(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let (profile, generator) = state
        .config_service
        .load_active()
        .await
        .map_err(to_api_error)?;
    let base_mappings = generator
        .get_layer_mappings("base")
        .map_err(|e| ApiError::InternalError(e.to_string()))?;
    let layers: Vec<Value> = generator
        .list_layers()
        .into_iter()
        .map(|(id, count)| json!({ "id": id, "mapping_count": count }))
        .collect();
    Ok(Json(json!({
        "profile": profile,
        "base_mappings": base_mappings,
        "layers": layers,
    })))
}

/// POST /api/config/key-mappings - Set key mapping
#[derive(Deserialize)]
struct SetKeyMappingRequest {
    layer: String,
    key: String,
    action_type: String, // "simple", "tap_hold"
    // For simple remap
    output: Option<String>,
    // For tap-hold
    tap: Option<String>,
    hold: Option<String>,
    threshold_ms: Option<u16>,
}

impl SetKeyMappingRequest {
    fn action(&self) -> Result<KeyAction, ApiError> {
        let missing = |field: &str| {
            ApiError::BadRequest(format!("Missing '{field}' field for {}", self.action_type))
        };
        match self.action_type.as_str() {
            "simple" => Ok(KeyAction::SimpleRemap {
                output: self.output.clone().ok_or_else(|| missing("output"))?,
            }),
            "tap_hold" => Ok(KeyAction::TapHold {
                tap: self.tap.clone().ok_or_else(|| missing("tap"))?,
                hold: self.hold.clone().ok_or_else(|| missing("hold"))?,
                threshold_ms: self.threshold_ms.unwrap_or(200),
            }),
            other => Err(ApiError::BadRequest(format!(
                "Unsupported action type: {other}. Use 'simple' or 'tap_hold'"
            ))),
        }
    }
}

async fn set_key_mapping(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SetKeyMappingRequest>,
) -> Result<Json<Value>, ApiError> {
    let action = payload.action()?;
    state
        .config_service
        .set_key_mapping(payload.layer, payload.key, action)
        .await
        .map_err(to_api_error)?;
    Ok(Json(json!({ "success": true })))
}

/// DELETE /api/config/key-mappings/:id - Delete key mapping
/// Format: layer:key (e.g., "base:A" or "MD_00:Space")
async fn delete_key_mapping(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let Some((layer, key)) = id.split_once(':').filter(|(_, k)| !k.contains(':')) else {
        return Err(ApiError::BadRequest(
            "Invalid mapping ID. Use format 'layer:key' (e.g., 'base:A')".to_string(),
        ));
    };
    state
        .config_service
        .delete_key_mapping(layer.to_string(), key.to_string())
        .await
        .map_err(to_api_error)?;
    Ok(Json(json!({ "success": true })))
}

/// PUT /api/config - Replace the active profile's Rhai source
#[derive(Deserialize)]
struct UpdateConfigRequest {
    content: String,
}

async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateConfigRequest>,
) -> Result<Json<Value>, ApiError> {
    state
        .config_service
        .update_config(payload.content)
        .await
        .map_err(to_api_error)?;
    let profile = state
        .config_service
        .get_config()
        .await
        .map(|c| c.profile)
        .map_err(to_api_error)?;
    Ok(Json(json!({
        "success": true,
        "message": "Configuration saved and compiled",
        "profile": profile,
    })))
}

#[derive(Serialize)]
struct LayerInfo {
    id: String,
    mapping_count: usize,
    mappings: Vec<String>,
}

/// GET /api/layers - List layers with their mappings
async fn list_layers(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let (_, generator) = state
        .config_service
        .load_active()
        .await
        .map_err(to_api_error)?;
    let base = generator
        .get_layer_mappings("base")
        .map_err(|e| ApiError::InternalError(e.to_string()))?;
    let mut layers = vec![LayerInfo {
        id: "base".to_string(),
        mapping_count: base.len(),
        mappings: base,
    }];
    for (id, mapping_count) in generator.list_layers() {
        let mappings = generator.get_layer_mappings(&id).unwrap_or_default();
        layers.push(LayerInfo {
            id,
            mapping_count,
            mappings,
        });
    }
    Ok(Json(json!({ "layers": layers })))
}
