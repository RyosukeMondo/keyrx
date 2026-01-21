//! Device management endpoints.

use axum::{
    extract::Path,
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
async fn list_devices() -> Result<Json<DevicesListResponse>, DaemonError> {
    use crate::device_manager::enumerate_keyboards;

    // Get registry path
    let config_dir = get_config_dir()?;
    let registry_path = config_dir.join("devices.json");

    // Load registry (contains user-set names and scopes)
    let registry = DeviceRegistry::load(&registry_path)?;

    // Enumerate actual connected devices
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

    Ok(Json(DevicesListResponse { devices }))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
async fn list_devices() -> Result<Json<DevicesListResponse>, DaemonError> {
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
    Path(id): Path<String>,
    Json(payload): Json<RenameDeviceRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate input parameters
    payload
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation failed: {}", e)))?;

    let config_dir = get_config_dir().map_err(|e| ApiError::InternalError(e.to_string()))?;
    let registry_path = config_dir.join("devices.json");

    let mut registry =
        DeviceRegistry::load(&registry_path).map_err(|e| ApiError::InternalError(e.to_string()))?;

    registry.rename(&id, &payload.name).map_err(|e| match e {
        DeviceValidationError::DeviceNotFound(msg) => ApiError::NotFound(msg),
        _ => ApiError::BadRequest(e.to_string()),
    })?;

    registry
        .save()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

/// PUT /api/devices/:id/layout - Set device layout
#[derive(Deserialize, Validate)]
struct SetDeviceLayoutRequest {
    #[validate(length(min = 1, max = 50))]
    layout: String,
}

async fn set_device_layout(
    Path(id): Path<String>,
    Json(payload): Json<SetDeviceLayoutRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate input parameters
    payload
        .validate()
        .map_err(|e| ApiError::BadRequest(format!("Validation failed: {}", e)))?;

    let config_dir = get_config_dir().map_err(|e| ApiError::InternalError(e.to_string()))?;
    let registry_path = config_dir.join("devices.json");

    let mut registry =
        DeviceRegistry::load(&registry_path).map_err(|e| ApiError::InternalError(e.to_string()))?;

    registry
        .set_layout(&id, &payload.layout)
        .map_err(|e| match e {
            DeviceValidationError::DeviceNotFound(msg) => ApiError::NotFound(msg),
            _ => ApiError::BadRequest(e.to_string()),
        })?;

    registry
        .save()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

/// GET /api/devices/:id/layout - Get device layout
#[derive(Serialize)]
struct GetDeviceLayoutResponse {
    layout: Option<String>,
}

async fn get_device_layout(
    Path(id): Path<String>,
) -> Result<Json<GetDeviceLayoutResponse>, ApiError> {
    let config_dir = get_config_dir().map_err(|e| ApiError::InternalError(e.to_string()))?;
    let registry_path = config_dir.join("devices.json");

    let registry =
        DeviceRegistry::load(&registry_path).map_err(|e| ApiError::InternalError(e.to_string()))?;

    let device = registry
        .get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Device not found: {}", id)))?;

    Ok(Json(GetDeviceLayoutResponse {
        layout: device.layout.clone(),
    }))
}

/// PATCH /api/devices/:id - Update device configuration
#[derive(Deserialize, Validate)]
struct UpdateDeviceConfigRequest {
    #[validate(length(min = 1, max = 50))]
    layout: Option<String>,
}

async fn update_device_config(
    Path(id): Path<String>,
    Json(payload): Json<UpdateDeviceConfigRequest>,
) -> Result<Json<Value>, DaemonError> {
    use crate::error::WebError;

    // Validate input parameters
    payload.validate().map_err(|e| WebError::InvalidRequest {
        reason: format!("Validation failed: {}", e),
    })?;

    let config_dir = get_config_dir()?;
    let registry_path = config_dir.join("devices.json");

    let mut registry = DeviceRegistry::load(&registry_path)?;

    // Auto-register device if it doesn't exist
    if registry.get(&id).is_none() {
        log::info!("Auto-registering device: {}", id);
        // Sanitize device ID for use as name: replace invalid chars with dash
        let sanitized_name = id
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
            id.clone(),
            sanitized_name, // Use sanitized ID as default name
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

    // Update layout if provided
    if let Some(layout) = &payload.layout {
        registry.set_layout(&id, layout).map_err(|e| {
            use crate::error::RegistryError;
            RegistryError::CorruptedRegistry(e.to_string())
        })?;
    }

    registry.save()?;

    Ok(Json(json!({ "success": true })))
}

/// DELETE /api/devices/:id - Forget device
async fn forget_device(Path(id): Path<String>) -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir().map_err(|e| ApiError::InternalError(e.to_string()))?;
    let registry_path = config_dir.join("devices.json");

    let mut registry =
        DeviceRegistry::load(&registry_path).map_err(|e| ApiError::InternalError(e.to_string()))?;

    registry.forget(&id).map_err(|e| match e {
        DeviceValidationError::DeviceNotFound(msg) => ApiError::NotFound(msg),
        _ => ApiError::InternalError(e.to_string()),
    })?;

    registry
        .save()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(json!({ "success": true })))
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
