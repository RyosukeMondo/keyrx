//! Device management endpoints.

use axum::{
    extract::Path,
    routing::{delete, get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use super::error::ApiError;
use crate::config::device_registry::{DeviceRegistry, DeviceScope};
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/devices", get(list_devices))
        .route("/devices/:id/name", put(rename_device))
        .route("/devices/:id/scope", put(set_device_scope))
        .route("/devices/:id/layout", put(set_device_layout))
        .route("/devices/:id/layout", get(get_device_layout))
        .route("/devices/:id", delete(forget_device))
}

#[derive(Serialize)]
struct DeviceResponse {
    id: String,
    name: String,
    path: String,
    serial: Option<String>,
    active: bool,
    scope: Option<String>,
    layout: Option<String>,
}

#[derive(Serialize)]
struct DevicesListResponse {
    devices: Vec<DeviceResponse>,
}

/// GET /api/devices - List all connected devices
#[cfg(any(target_os = "linux", target_os = "windows"))]
async fn list_devices() -> Result<Json<DevicesListResponse>, ApiError> {
    use crate::device_manager::enumerate_keyboards;

    // Get registry path
    let config_dir = get_config_dir()?;
    let registry_path = config_dir.join("devices.json");

    // Load registry (contains user-set names and scopes)
    let registry = DeviceRegistry::load(&registry_path)
        .map_err(|e| ApiError::InternalError(format!("Failed to load device registry: {}", e)))?;

    // Enumerate actual connected devices
    let keyboards = enumerate_keyboards()
        .map_err(|e| ApiError::InternalError(format!("Failed to enumerate keyboards: {}", e)))?;

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
                scope: registry_entry.map(|e| match e.scope {
                    DeviceScope::Global => "global".to_string(),
                    DeviceScope::DeviceSpecific => "device-specific".to_string(),
                }),
                layout: registry_entry.and_then(|e| e.layout.clone()),
            }
        })
        .collect();

    Ok(Json(DevicesListResponse { devices }))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
async fn list_devices() -> Result<Json<DevicesListResponse>, ApiError> {
    Ok(Json(DevicesListResponse {
        devices: Vec::new(),
    }))
}

/// PUT /api/devices/:id/name - Rename a device
#[derive(Deserialize)]
struct RenameDeviceRequest {
    name: String,
}

async fn rename_device(
    Path(id): Path<String>,
    Json(payload): Json<RenameDeviceRequest>,
) -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let registry_path = config_dir.join("devices.json");

    let mut registry = DeviceRegistry::load(&registry_path)
        .map_err(|e| ApiError::InternalError(format!("Failed to load device registry: {}", e)))?;

    registry
        .rename(&id, &payload.name)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    registry
        .save()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

/// PUT /api/devices/:id/scope - Set device scope
#[derive(Deserialize)]
struct SetDeviceScopeRequest {
    scope: String, // "global" or "device-specific"
}

async fn set_device_scope(
    Path(id): Path<String>,
    Json(payload): Json<SetDeviceScopeRequest>,
) -> Result<Json<Value>, ApiError> {
    let scope = match payload.scope.as_str() {
        "global" => DeviceScope::Global,
        "device-specific" => DeviceScope::DeviceSpecific,
        _ => return Err(ApiError::BadRequest("Invalid scope value".to_string())),
    };

    let config_dir = get_config_dir()?;
    let registry_path = config_dir.join("devices.json");

    let mut registry = DeviceRegistry::load(&registry_path)
        .map_err(|e| ApiError::InternalError(format!("Failed to load device registry: {}", e)))?;

    registry
        .set_scope(&id, scope)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    registry
        .save()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

/// PUT /api/devices/:id/layout - Set device layout
#[derive(Deserialize)]
struct SetDeviceLayoutRequest {
    layout: String,
}

async fn set_device_layout(
    Path(id): Path<String>,
    Json(payload): Json<SetDeviceLayoutRequest>,
) -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let registry_path = config_dir.join("devices.json");

    let mut registry = DeviceRegistry::load(&registry_path)
        .map_err(|e| ApiError::InternalError(format!("Failed to load device registry: {}", e)))?;

    registry
        .set_layout(&id, &payload.layout)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

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
    let config_dir = get_config_dir()?;
    let registry_path = config_dir.join("devices.json");

    let registry = DeviceRegistry::load(&registry_path)
        .map_err(|e| ApiError::InternalError(format!("Failed to load device registry: {}", e)))?;

    let device = registry
        .get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("Device not found: {}", id)))?;

    Ok(Json(GetDeviceLayoutResponse {
        layout: device.layout.clone(),
    }))
}

/// DELETE /api/devices/:id - Forget device
async fn forget_device(Path(id): Path<String>) -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let registry_path = config_dir.join("devices.json");

    let mut registry = DeviceRegistry::load(&registry_path)
        .map_err(|e| ApiError::InternalError(format!("Failed to load device registry: {}", e)))?;

    registry
        .forget(&id)
        .map_err(|e| ApiError::NotFound(e.to_string()))?;

    registry
        .save()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

/// Get config directory path
fn get_config_dir() -> Result<std::path::PathBuf, ApiError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| ApiError::InternalError("Cannot determine home directory".to_string()))?;

    Ok(std::path::PathBuf::from(home).join(".config/keyrx"))
}
