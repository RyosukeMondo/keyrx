//! Configuration management endpoints.

use axum::{
    extract::Path,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::config::rhai_generator::{KeyAction, RhaiGenerator};
use crate::error::DaemonError;
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/config", get(get_config).put(update_config))
        .route("/config/key-mappings", post(set_key_mapping))
        .route("/config/key-mappings/:id", delete(delete_key_mapping))
        .route("/layers", get(list_layers))
}

/// GET /api/config - Get current configuration
async fn get_config() -> Result<Json<Value>, DaemonError> {
    use crate::error::ConfigError;

    // Wrap all blocking operations in spawn_blocking to prevent runtime starvation
    tokio::task::spawn_blocking(move || {
        let config_dir = get_config_dir()?;
        let active_profile = query_active_profile().unwrap_or_else(|| "default".to_string());

        let rhai_path = config_dir
            .join("profiles")
            .join(format!("{}.rhai", active_profile));

        if !rhai_path.exists() {
            return Err(ConfigError::FileNotFound { path: rhai_path }.into());
        }

        let generator =
            RhaiGenerator::load(&rhai_path).map_err(|e| ConfigError::Generator(e.to_string()))?;

        // Get base mappings and layers
        let base_mappings = generator
            .get_layer_mappings("base")
            .map_err(|e| ConfigError::Generator(e.to_string()))?;

        let layers = generator.list_layers();

        Ok::<Json<Value>, DaemonError>(Json(json!({
            "profile": active_profile,
            "base_mappings": base_mappings,
            "layers": layers.iter().map(|(id, count)| json!({
                "id": id,
                "mapping_count": count,
            })).collect::<Vec<_>>(),
        })))
    })
    .await
    .map_err(|e| ConfigError::ParseError {
        path: std::path::PathBuf::from("config"),
        reason: format!("Task join error: {}", e),
    })?
}

/// POST /api/config/key-mappings - Set key mapping
#[derive(Deserialize)]
struct SetKeyMappingRequest {
    layer: String,
    key: String,
    action_type: String, // "simple", "tap_hold", "macro"
    // For simple remap
    output: Option<String>,
    // For tap-hold
    tap: Option<String>,
    hold: Option<String>,
    threshold_ms: Option<u16>,
    // For macros - simplified as string sequence for now (not yet implemented)
    #[allow(dead_code)]
    macro_sequence: Option<String>,
}

async fn set_key_mapping(
    Json(payload): Json<SetKeyMappingRequest>,
) -> Result<Json<Value>, DaemonError> {
    use crate::error::{ConfigError, WebError};

    // Wrap all blocking operations in spawn_blocking
    tokio::task::spawn_blocking(move || {
        let config_dir = get_config_dir()?;
        let active_profile = query_active_profile().unwrap_or_else(|| "default".to_string());

        let rhai_path = config_dir
            .join("profiles")
            .join(format!("{}.rhai", active_profile));

        if !rhai_path.exists() {
            return Err(ConfigError::FileNotFound { path: rhai_path }.into());
        }

        let mut generator =
            RhaiGenerator::load(&rhai_path).map_err(|e| ConfigError::Generator(e.to_string()))?;

        // Parse action type
        let action = match payload.action_type.as_str() {
            "simple" => {
                let output = payload.output.ok_or_else(|| WebError::InvalidRequest {
                    reason: "Missing 'output' field for simple remap".to_string(),
                })?;
                KeyAction::SimpleRemap { output }
            }
            "tap_hold" => {
                let tap = payload.tap.ok_or_else(|| WebError::InvalidRequest {
                    reason: "Missing 'tap' field for tap_hold".to_string(),
                })?;
                let hold = payload.hold.ok_or_else(|| WebError::InvalidRequest {
                    reason: "Missing 'hold' field for tap_hold".to_string(),
                })?;
                let threshold_ms = payload.threshold_ms.unwrap_or(200);
                KeyAction::TapHold {
                    tap,
                    hold,
                    threshold_ms,
                }
            }
            _ => {
                return Err(WebError::InvalidRequest {
                    reason: format!(
                        "Unsupported action type: {}. Use 'simple' or 'tap_hold'",
                        payload.action_type
                    ),
                }
                .into())
            }
        };

        generator
            .set_key_mapping(&payload.layer, &payload.key, action)
            .map_err(|e| ConfigError::Generator(e.to_string()))?;

        generator
            .save(&rhai_path)
            .map_err(|e| ConfigError::Generator(e.to_string()))?;

        Ok::<Json<Value>, DaemonError>(Json(json!({ "success": true })))
    })
    .await
    .map_err(|e| ConfigError::ParseError {
        path: std::path::PathBuf::from("config"),
        reason: format!("Task join error: {}", e),
    })?
}

/// DELETE /api/config/key-mappings/:id - Delete key mapping
/// Format: layer:key (e.g., "base:A" or "MD_00:Space")
async fn delete_key_mapping(Path(id): Path<String>) -> Result<Json<Value>, DaemonError> {
    use crate::error::{ConfigError, WebError};

    let parts: Vec<&str> = id.split(':').collect();
    if parts.len() != 2 {
        return Err(WebError::InvalidRequest {
            reason: "Invalid mapping ID. Use format 'layer:key' (e.g., 'base:A')".to_string(),
        }
        .into());
    }

    let layer = parts[0].to_string();
    let key = parts[1].to_string();

    // Wrap all blocking operations in spawn_blocking
    tokio::task::spawn_blocking(move || {
        let config_dir = get_config_dir()?;
        let active_profile = query_active_profile().unwrap_or_else(|| "default".to_string());

        let rhai_path = config_dir
            .join("profiles")
            .join(format!("{}.rhai", active_profile));

        if !rhai_path.exists() {
            return Err(ConfigError::FileNotFound { path: rhai_path }.into());
        }

        let mut generator =
            RhaiGenerator::load(&rhai_path).map_err(|e| ConfigError::Generator(e.to_string()))?;

        generator
            .delete_key_mapping(&layer, &key)
            .map_err(|e| ConfigError::Generator(e.to_string()))?;

        generator
            .save(&rhai_path)
            .map_err(|e| ConfigError::Generator(e.to_string()))?;

        Ok::<Json<Value>, DaemonError>(Json(json!({ "success": true })))
    })
    .await
    .map_err(|e| ConfigError::ParseError {
        path: std::path::PathBuf::from("config"),
        reason: format!("Task join error: {}", e),
    })?
}

/// PUT /api/config - Update configuration (save raw Rhai content)
#[derive(Deserialize)]
struct UpdateConfigRequest {
    content: String,
}

async fn update_config(
    Json(payload): Json<UpdateConfigRequest>,
) -> Result<Json<Value>, DaemonError> {
    use crate::error::ConfigError;

    // Wrap all blocking operations in spawn_blocking
    tokio::task::spawn_blocking(move || {
        let config_dir = get_config_dir()?;
        let active_profile = query_active_profile().unwrap_or_else(|| "default".to_string());

        let rhai_path = config_dir
            .join("profiles")
            .join(format!("{}.rhai", active_profile));

        // Write the configuration content to the file
        std::fs::write(&rhai_path, payload.content.as_bytes()).map_err(ConfigError::Io)?;

        // Validate the configuration by attempting to load it
        // This ensures syntax errors are caught
        match RhaiGenerator::load(&rhai_path) {
            Ok(_) => Ok::<Json<Value>, DaemonError>(Json(json!({
                "success": true,
                "message": "Configuration saved successfully",
                "profile": active_profile,
            }))),
            Err(e) => {
                // If validation fails, the file has been written but is invalid
                // Return success=true but include validation error
                Ok(Json(json!({
                    "success": true,
                    "message": "Configuration saved but has validation errors",
                    "profile": active_profile,
                    "validation_error": e.to_string(),
                })))
            }
        }
    })
    .await
    .map_err(|e| ConfigError::ParseError {
        path: std::path::PathBuf::from("config"),
        reason: format!("Task join error: {}", e),
    })?
}

#[derive(Serialize)]
struct LayerInfo {
    id: String,
    mapping_count: usize,
    mappings: Vec<String>,
}

/// GET /api/layers - List layers
async fn list_layers() -> Result<Json<Value>, DaemonError> {
    use crate::error::ConfigError;

    // Wrap all blocking operations in spawn_blocking
    tokio::task::spawn_blocking(move || {
        let config_dir = get_config_dir()?;
        let active_profile = query_active_profile().unwrap_or_else(|| "default".to_string());

        let rhai_path = config_dir
            .join("profiles")
            .join(format!("{}.rhai", active_profile));

        if !rhai_path.exists() {
            return Err(ConfigError::FileNotFound { path: rhai_path }.into());
        }

        let generator =
            RhaiGenerator::load(&rhai_path).map_err(|e| ConfigError::Generator(e.to_string()))?;

        // Get base layer
        let base_mappings = generator
            .get_layer_mappings("base")
            .map_err(|e| ConfigError::Generator(e.to_string()))?;

        let mut layers = vec![LayerInfo {
            id: "base".to_string(),
            mapping_count: base_mappings.len(),
            mappings: base_mappings,
        }];

        // Get all other layers
        for (layer_id, mapping_count) in generator.list_layers() {
            let mappings = generator
                .get_layer_mappings(&layer_id)
                .unwrap_or_else(|_| vec![]);

            layers.push(LayerInfo {
                id: layer_id,
                mapping_count,
                mappings,
            });
        }

        Ok::<Json<Value>, DaemonError>(Json(json!({ "layers": layers })))
    })
    .await
    .map_err(|e| ConfigError::ParseError {
        path: std::path::PathBuf::from("config"),
        reason: format!("Task join error: {}", e),
    })?
}

/// Get config directory path (cross-platform)
fn get_config_dir() -> Result<std::path::PathBuf, DaemonError> {
    use crate::error::ConfigError;

    let config_dir = dirs::config_dir().ok_or_else(|| ConfigError::ParseError {
        path: std::path::PathBuf::from("~"),
        reason: "Cannot determine config directory".to_string(),
    })?;

    Ok(config_dir.join("keyrx"))
}

/// Query active profile name
fn query_active_profile() -> Option<String> {
    use crate::ipc::{DaemonIpc, IpcRequest, IpcResponse, DEFAULT_SOCKET_PATH};

    let socket_path = std::path::PathBuf::from(DEFAULT_SOCKET_PATH);
    let mut ipc = crate::ipc::unix_socket::UnixSocketIpc::new(socket_path);

    let response = ipc.send_request(&IpcRequest::GetStatus).ok()?;

    match response {
        IpcResponse::Status {
            running: _,
            uptime_secs: _,
            active_profile,
            device_count: _,
        } => active_profile,
        _ => None,
    }
}
