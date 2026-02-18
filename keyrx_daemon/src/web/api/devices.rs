//! Device management endpoints.

use axum::{
    extract::{Path, State},
    routing::{delete, get, patch, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use validator::Validate;

use crate::config::device_registry::{DeviceEntry, DeviceRegistry, DeviceValidationError};
use crate::error::DaemonError;
use crate::web::api::error::ApiError;
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/devices", get(list_devices))
        .route("/devices/:id/name", put(rename_device))
        .route("/devices/:id/layout", put(set_device_layout))
        .route("/devices/:id/layout", get(get_device_layout))
        .route("/devices/:id", patch(update_device_config))
        .route("/devices/:id", delete(forget_device))
}

#[derive(Serialize)]
struct DeviceResponse {
    id: String,
    name: String,
    path: String,
    serial: Option<String>,
    active: bool,
    layout: Option<String>,
}

#[derive(Serialize)]
struct DevicesListResponse {
    devices: Vec<DeviceResponse>,
}

/// GET /api/devices - List all connected devices
#[cfg(any(target_os = "linux", target_os = "windows"))]
async fn list_devices(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DevicesListResponse>, DaemonError> {
    use crate::device_manager::enumerate_keyboards;
    use crate::error::ConfigError;

    let registry_path = state.device_service.registry_path().to_path_buf();

    tokio::task::spawn_blocking(move || {
        let registry = DeviceRegistry::load(&registry_path)?;

        let keyboards = enumerate_keyboards().map_err(|e| {
            use crate::error::PlatformError;
            PlatformError::DeviceError(e.to_string())
        })?;

        let devices: Vec<DeviceResponse> = keyboards
            .into_iter()
            .map(|kb| {
                let id = kb.device_id();
                let registry_entry = registry.get(&id);

                DeviceResponse {
                    id: id.clone(),
                    name: registry_entry
                        .map(|e| e.name.clone())
                        .unwrap_or_else(|| kb.name.clone()),
                    path: kb.path.display().to_string(),
                    serial: kb.serial,
                    active: true,
                    layout: registry_entry.and_then(|e| e.layout.clone()),
                }
            })
            .collect();

        Ok::<Json<DevicesListResponse>, DaemonError>(Json(DevicesListResponse { devices }))
    })
    .await
    .map_err(|e| ConfigError::ParseError {
        path: std::path::PathBuf::from("devices"),
        reason: format!("Task join error: {}", e),
    })?
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
async fn list_devices(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<DevicesListResponse>, DaemonError> {
    Ok(Json(DevicesListResponse {
        devices: Vec::new(),
    }))
}

/// PUT /api/devices/:id/name - Rename a device
#[derive(Deserialize, Validate)]
struct RenameDeviceRequest {
    #[validate(length(min = 1, max = 100))]
    name: String,
}

async fn rename_device(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<RenameDeviceRequest>,
) -> Result<Json<Value>, ApiError> {
    payload
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation failed: {}", e)))?;

    let id_clone = id.clone();
    let name_clone = payload.name.clone();
    let registry_path = state.device_service.registry_path().to_path_buf();

    tokio::task::spawn_blocking(move || {
        let mut registry = DeviceRegistry::load(&registry_path)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        registry
            .rename(&id_clone, &name_clone)
            .map_err(|e| match e {
                DeviceValidationError::DeviceNotFound(msg) => ApiError::NotFound(msg),
                _ => ApiError::BadRequest(e.to_string()),
            })?;

        registry
            .save()
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        Ok::<(), ApiError>(())
    })
    .await
    .map_err(|e| ApiError::InternalError(format!("Task join error: {}", e)))??;

    // Broadcast event to WebSocket subscribers
    use crate::web::rpc_types::ServerMessage;
    let event = ServerMessage::Event {
        channel: "devices".to_string(),
        data: serde_json::json!({
            "action": "renamed",
            "id": id,
            "name": payload.name
        }),
    };
    if let Err(e) = state.event_broadcaster.send(event) {
        log::warn!("Failed to broadcast device renamed event: {}", e);
    }

    Ok(Json(json!({ "success": true })))
}

/// PUT /api/devices/:id/layout - Set device layout
#[derive(Deserialize, Validate)]
struct SetDeviceLayoutRequest {
    #[validate(length(min = 1, max = 50))]
    layout: String,
}

async fn set_device_layout(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<SetDeviceLayoutRequest>,
) -> Result<Json<Value>, ApiError> {
    payload
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation failed: {}", e)))?;

    let id_clone = id.clone();
    let layout_clone = payload.layout.clone();
    let registry_path = state.device_service.registry_path().to_path_buf();

    tokio::task::spawn_blocking(move || {
        let mut registry = DeviceRegistry::load(&registry_path)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        registry
            .set_layout(&id_clone, &layout_clone)
            .map_err(|e| match e {
                DeviceValidationError::DeviceNotFound(msg) => ApiError::NotFound(msg),
                _ => ApiError::BadRequest(e.to_string()),
            })?;

        registry
            .save()
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        Ok::<Json<Value>, ApiError>(Json(json!({ "success": true })))
    })
    .await
    .map_err(|e| ApiError::InternalError(format!("Task join error: {}", e)))?
}

/// GET /api/devices/:id/layout - Get device layout
#[derive(Serialize)]
struct GetDeviceLayoutResponse {
    layout: Option<String>,
}

async fn get_device_layout(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<GetDeviceLayoutResponse>, ApiError> {
    let id_clone = id.clone();
    let registry_path = state.device_service.registry_path().to_path_buf();

    tokio::task::spawn_blocking(move || {
        let registry = DeviceRegistry::load(&registry_path)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        let device = registry
            .get(&id_clone)
            .ok_or_else(|| ApiError::NotFound(format!("Device not found: {}", id_clone)))?;

        Ok::<Json<GetDeviceLayoutResponse>, ApiError>(Json(GetDeviceLayoutResponse {
            layout: device.layout.clone(),
        }))
    })
    .await
    .map_err(|e| ApiError::InternalError(format!("Task join error: {}", e)))?
}

/// PATCH /api/devices/:id - Update device configuration
#[derive(Deserialize, Validate)]
struct UpdateDeviceConfigRequest {
    #[validate(length(min = 1, max = 50))]
    layout: Option<String>,
}

async fn update_device_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDeviceConfigRequest>,
) -> Result<Json<Value>, DaemonError> {
    use crate::error::{ConfigError, WebError};

    payload.validate().map_err(|e| WebError::InvalidRequest {
        reason: format!("Validation failed: {}", e),
    })?;

    let id_clone = id.clone();
    let layout_clone = payload.layout.clone();
    let registry_path = state.device_service.registry_path().to_path_buf();

    tokio::task::spawn_blocking(move || {
        let mut registry = DeviceRegistry::load(&registry_path)?;

        // Auto-register device if it doesn't exist
        if registry.get(&id_clone).is_none() {
            log::info!("Auto-registering device: {}", id_clone);
            let sanitized_name = id_clone
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                        c
                    } else {
                        '-'
                    }
                })
                .collect::<String>();
            let entry = DeviceEntry::new(
                id_clone.clone(),
                sanitized_name,
                None,
                None,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
            registry.register(entry).map_err(|e| {
                use crate::error::RegistryError;
                RegistryError::CorruptedRegistry(e.to_string())
            })?;
        }

        if let Some(layout) = &layout_clone {
            registry.set_layout(&id_clone, layout).map_err(|e| {
                use crate::error::RegistryError;
                RegistryError::CorruptedRegistry(e.to_string())
            })?;
        }

        registry.save()?;

        Ok::<(), DaemonError>(())
    })
    .await
    .map_err(|e| ConfigError::ParseError {
        path: std::path::PathBuf::from("devices"),
        reason: format!("Task join error: {}", e),
    })??;

    // Broadcast event to WebSocket subscribers
    use crate::web::rpc_types::ServerMessage;
    let event = ServerMessage::Event {
        channel: "devices".to_string(),
        data: serde_json::json!({
            "action": "updated",
            "id": id,
            "layout": payload.layout
        }),
    };
    if let Err(e) = state.event_broadcaster.send(event) {
        log::warn!("Failed to broadcast device updated event: {}", e);
    }

    Ok(Json(json!({ "success": true })))
}

/// DELETE /api/devices/:id - Forget device
async fn forget_device(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let id_clone = id.clone();
    let registry_path = state.device_service.registry_path().to_path_buf();

    tokio::task::spawn_blocking(move || {
        let mut registry = DeviceRegistry::load(&registry_path)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        registry.forget(&id_clone).map_err(|e| match e {
            DeviceValidationError::DeviceNotFound(msg) => ApiError::NotFound(msg),
            _ => ApiError::InternalError(e.to_string()),
        })?;

        registry
            .save()
            .map_err(|e| ApiError::InternalError(e.to_string()))?;

        Ok::<Json<Value>, ApiError>(Json(json!({ "success": true })))
    })
    .await
    .map_err(|e| ApiError::InternalError(format!("Task join error: {}", e)))?
}
