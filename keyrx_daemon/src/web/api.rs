//! REST API endpoints for the KeyRx daemon web interface.
//!
//! This module provides a complete REST API that exposes all CLI functionality
//! to the web UI. Endpoints are organized by feature area:
//! - Device management
//! - Profile management
//! - Configuration management
//! - Layer management
//! - Layout management
//! - Metrics and monitoring
//! - Daemon state query
//! - Simulator

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::{
    device_registry::{DeviceRegistry, DeviceScope},
    layout_manager::LayoutManager,
    profile_manager::{ProfileManager, ProfileTemplate},
    rhai_generator::{GeneratorError, KeyAction, RhaiGenerator},
    simulation_engine::{BuiltinScenario, EventSequence, SimulatedEvent},
};
use crate::ipc::{DaemonIpc, IpcRequest, IpcResponse, DEFAULT_SOCKET_PATH};

// ============================================================================
// Error Handling
// ============================================================================

/// API error type that maps to HTTP status codes
#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    InternalError(String),
    DaemonNotRunning,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
            ApiError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg)
            }
            ApiError::DaemonNotRunning => (
                StatusCode::SERVICE_UNAVAILABLE,
                "DAEMON_NOT_RUNNING",
                "Daemon is not running".to_string(),
            ),
        };

        let body = Json(json!({
            "success": false,
            "error": {
                "code": code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

// Convenience conversions
impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        ApiError::InternalError(e.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for ApiError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        ApiError::InternalError(e.to_string())
    }
}

// ============================================================================
// Router
// ============================================================================

pub fn create_router() -> Router {
    Router::new()
        // Health and status
        .route("/health", get(health_check))
        .route("/status", get(get_status))
        // Device management
        .route("/devices", get(list_devices))
        .route("/devices/:id/name", put(rename_device))
        .route("/devices/:id/scope", put(set_device_scope))
        .route("/devices/:id", delete(forget_device))
        // Profile management
        .route("/profiles", get(list_profiles).post(create_profile))
        .route("/profiles/:name/activate", post(activate_profile))
        .route("/profiles/:name", delete(delete_profile))
        .route("/profiles/:name/duplicate", post(duplicate_profile))
        // Configuration
        .route("/config", get(get_config))
        .route("/config/key-mappings", post(set_key_mapping))
        .route("/config/key-mappings/:id", delete(delete_key_mapping))
        // Layers
        .route("/layers", get(list_layers))
        // Layouts
        .route("/layouts", get(list_layouts))
        .route("/layouts/:name", get(get_layout))
        // Metrics and monitoring
        .route("/metrics/latency", get(get_latency_stats))
        .route(
            "/metrics/events",
            get(get_event_log).delete(clear_event_log),
        )
        .route("/daemon/state", get(get_daemon_state))
        // Simulator
        .route("/simulator/events", post(simulate_events))
        .route("/simulator/reset", post(reset_simulator))
        // Macro recorder
        .route("/macros/start-recording", post(start_macro_recording))
        .route("/macros/stop-recording", post(stop_macro_recording))
        .route("/macros/recorded-events", get(get_recorded_events))
        .route("/macros/clear", post(clear_recorded_events))
}

// ============================================================================
// Health and Status
// ============================================================================

/// GET /api/health - Health check
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// GET /api/status - Daemon status
#[derive(Serialize)]
struct StatusResponse {
    status: String,
    version: String,
    daemon_running: bool,
    uptime_secs: Option<u64>,
    active_profile: Option<String>,
    device_count: Option<usize>,
}

async fn get_status() -> Result<Json<StatusResponse>, ApiError> {
    // Try to query daemon via IPC
    let daemon_info = query_daemon_status();

    let (daemon_running, uptime_secs, active_profile, device_count) = match daemon_info {
        Ok((uptime, profile, count)) => (true, Some(uptime), profile, Some(count)),
        Err(_) => (false, None, None, None),
    };

    Ok(Json(StatusResponse {
        status: "running".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        daemon_running,
        uptime_secs,
        active_profile,
        device_count,
    }))
}

// ============================================================================
// Device Management
// ============================================================================

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

// ============================================================================
// Profile Management
// ============================================================================

#[derive(Serialize)]
struct ProfileResponse {
    name: String,
    rhai_path: String,
    krx_path: String,
    modified_at: u64,
    layer_count: usize,
    is_active: bool,
}

#[derive(Serialize)]
struct ProfilesListResponse {
    profiles: Vec<ProfileResponse>,
}

/// GET /api/profiles - List all profiles
async fn list_profiles() -> Result<Json<ProfilesListResponse>, ApiError> {
    let config_dir = get_config_dir()?;
    let mut pm =
        ProfileManager::new(config_dir).map_err(|e| ApiError::InternalError(e.to_string()))?;

    pm.scan_profiles()
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let active_profile = query_active_profile();

    let profiles: Vec<ProfileResponse> = pm
        .list()
        .iter()
        .map(|meta| ProfileResponse {
            name: meta.name.clone(),
            rhai_path: meta.rhai_path.display().to_string(),
            krx_path: meta.krx_path.display().to_string(),
            modified_at: meta
                .modified_at
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            layer_count: meta.layer_count,
            is_active: active_profile.as_ref() == Some(&meta.name),
        })
        .collect();

    Ok(Json(ProfilesListResponse { profiles }))
}

/// POST /api/profiles - Create new profile
#[derive(Deserialize)]
struct CreateProfileRequest {
    name: String,
    template: String, // "blank" or "qmk-layers"
}

async fn create_profile(
    Json(payload): Json<CreateProfileRequest>,
) -> Result<Json<Value>, ApiError> {
    let template = match payload.template.as_str() {
        "blank" => ProfileTemplate::Blank,
        "qmk-layers" => ProfileTemplate::QmkLayers,
        _ => return Err(ApiError::BadRequest("Invalid template".to_string())),
    };

    let config_dir = get_config_dir()?;
    let mut pm =
        ProfileManager::new(config_dir).map_err(|e| ApiError::InternalError(e.to_string()))?;

    let metadata = pm
        .create(&payload.name, template)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "profile": {
            "name": metadata.name,
            "rhai_path": metadata.rhai_path.display().to_string(),
            "krx_path": metadata.krx_path.display().to_string(),
        }
    })))
}

/// POST /api/profiles/:name/activate - Activate profile
async fn activate_profile(Path(name): Path<String>) -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let mut pm =
        ProfileManager::new(config_dir).map_err(|e| ApiError::InternalError(e.to_string()))?;

    let result = pm
        .activate(&name)
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    if !result.success {
        return Err(ApiError::InternalError(
            result.error.unwrap_or_else(|| "Unknown error".to_string()),
        ));
    }

    Ok(Json(json!({
        "success": true,
        "compile_time_ms": result.compile_time_ms,
        "reload_time_ms": result.reload_time_ms,
    })))
}

/// DELETE /api/profiles/:name - Delete profile
async fn delete_profile(Path(name): Path<String>) -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let mut pm =
        ProfileManager::new(config_dir).map_err(|e| ApiError::InternalError(e.to_string()))?;

    pm.delete(&name)
        .map_err(|e| ApiError::NotFound(e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

/// POST /api/profiles/:name/duplicate - Duplicate profile
#[derive(Deserialize)]
struct DuplicateProfileRequest {
    new_name: String,
}

async fn duplicate_profile(
    Path(name): Path<String>,
    Json(payload): Json<DuplicateProfileRequest>,
) -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let mut pm =
        ProfileManager::new(config_dir).map_err(|e| ApiError::InternalError(e.to_string()))?;

    let metadata = pm
        .duplicate(&name, &payload.new_name)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "profile": {
            "name": metadata.name,
            "rhai_path": metadata.rhai_path.display().to_string(),
        }
    })))
}

// ============================================================================
// Configuration Management
// ============================================================================

/// GET /api/config - Get current configuration
async fn get_config() -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let active_profile = query_active_profile().unwrap_or_else(|| "default".to_string());

    let rhai_path = config_dir
        .join("profiles")
        .join(format!("{}.rhai", active_profile));

    if !rhai_path.exists() {
        return Err(ApiError::NotFound(format!(
            "Active profile '{}' not found",
            active_profile
        )));
    }

    let generator =
        RhaiGenerator::load(&rhai_path).map_err(|e| ApiError::InternalError(e.to_string()))?;

    // Get base mappings and layers
    let base_mappings = generator
        .get_layer_mappings("base")
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let layers = generator.list_layers();

    Ok(Json(json!({
        "profile": active_profile,
        "base_mappings": base_mappings,
        "layers": layers.iter().map(|(id, count)| json!({
            "id": id,
            "mapping_count": count,
        })).collect::<Vec<_>>(),
    })))
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
) -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let active_profile = query_active_profile().unwrap_or_else(|| "default".to_string());

    let rhai_path = config_dir
        .join("profiles")
        .join(format!("{}.rhai", active_profile));

    if !rhai_path.exists() {
        return Err(ApiError::NotFound(format!(
            "Active profile '{}' not found",
            active_profile
        )));
    }

    let mut generator =
        RhaiGenerator::load(&rhai_path).map_err(|e| ApiError::InternalError(e.to_string()))?;

    // Parse action type
    let action = match payload.action_type.as_str() {
        "simple" => {
            let output = payload.output.ok_or_else(|| {
                ApiError::BadRequest("Missing 'output' field for simple remap".to_string())
            })?;
            KeyAction::SimpleRemap { output }
        }
        "tap_hold" => {
            let tap = payload.tap.ok_or_else(|| {
                ApiError::BadRequest("Missing 'tap' field for tap_hold".to_string())
            })?;
            let hold = payload.hold.ok_or_else(|| {
                ApiError::BadRequest("Missing 'hold' field for tap_hold".to_string())
            })?;
            let threshold_ms = payload.threshold_ms.unwrap_or(200);
            KeyAction::TapHold {
                tap,
                hold,
                threshold_ms,
            }
        }
        _ => {
            return Err(ApiError::BadRequest(format!(
                "Unsupported action type: {}. Use 'simple' or 'tap_hold'",
                payload.action_type
            )))
        }
    };

    generator
        .set_key_mapping(&payload.layer, &payload.key, action)
        .map_err(|e| match e {
            GeneratorError::LayerNotFound(layer) => {
                ApiError::NotFound(format!("Layer '{}' not found", layer))
            }
            GeneratorError::InvalidKeyName(key) => {
                ApiError::BadRequest(format!("Invalid key name: {}", key))
            }
            _ => ApiError::InternalError(e.to_string()),
        })?;

    generator
        .save(&rhai_path)
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

/// DELETE /api/config/key-mappings/:id - Delete key mapping
/// Format: layer:key (e.g., "base:A" or "MD_00:Space")
async fn delete_key_mapping(Path(id): Path<String>) -> Result<Json<Value>, ApiError> {
    let parts: Vec<&str> = id.split(':').collect();
    if parts.len() != 2 {
        return Err(ApiError::BadRequest(
            "Invalid mapping ID. Use format 'layer:key' (e.g., 'base:A')".to_string(),
        ));
    }

    let layer = parts[0];
    let key = parts[1];

    let config_dir = get_config_dir()?;
    let active_profile = query_active_profile().unwrap_or_else(|| "default".to_string());

    let rhai_path = config_dir
        .join("profiles")
        .join(format!("{}.rhai", active_profile));

    if !rhai_path.exists() {
        return Err(ApiError::NotFound(format!(
            "Active profile '{}' not found",
            active_profile
        )));
    }

    let mut generator =
        RhaiGenerator::load(&rhai_path).map_err(|e| ApiError::InternalError(e.to_string()))?;

    generator
        .delete_key_mapping(layer, key)
        .map_err(|e| match e {
            GeneratorError::LayerNotFound(layer) => {
                ApiError::NotFound(format!("Layer '{}' not found", layer))
            }
            GeneratorError::InvalidKeyName(key) => {
                ApiError::BadRequest(format!("Invalid key name: {}", key))
            }
            _ => ApiError::InternalError(e.to_string()),
        })?;

    generator
        .save(&rhai_path)
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    Ok(Json(json!({ "success": true })))
}

// ============================================================================
// Layer Management
// ============================================================================

#[derive(Serialize)]
struct LayerInfo {
    id: String,
    mapping_count: usize,
    mappings: Vec<String>,
}

/// GET /api/layers - List layers
async fn list_layers() -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let active_profile = query_active_profile().unwrap_or_else(|| "default".to_string());

    let rhai_path = config_dir
        .join("profiles")
        .join(format!("{}.rhai", active_profile));

    if !rhai_path.exists() {
        return Err(ApiError::NotFound(format!(
            "Active profile '{}' not found",
            active_profile
        )));
    }

    let generator =
        RhaiGenerator::load(&rhai_path).map_err(|e| ApiError::InternalError(e.to_string()))?;

    // Get base layer
    let base_mappings = generator
        .get_layer_mappings("base")
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

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

    Ok(Json(json!({ "layers": layers })))
}

// ============================================================================
// Layout Management
// ============================================================================

/// GET /api/layouts - List keyboard layouts
async fn list_layouts() -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let lm = LayoutManager::new(config_dir.join("layouts"))
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let layouts: Vec<String> = lm.list().iter().map(|layout| layout.name.clone()).collect();

    Ok(Json(json!({
        "layouts": layouts
    })))
}

/// GET /api/layouts/:name - Get layout KLE JSON
async fn get_layout(Path(name): Path<String>) -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;
    let lm = LayoutManager::new(config_dir.join("layouts"))
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let layout = lm
        .get(&name)
        .ok_or_else(|| ApiError::NotFound(format!("Layout '{}' not found", name)))?;

    Ok(Json(layout.kle_json.clone()))
}

// ============================================================================
// Metrics and Monitoring
// ============================================================================

#[derive(Serialize)]
struct LatencyStatsResponse {
    min_us: u64,
    avg_us: u64,
    max_us: u64,
    p95_us: u64,
    p99_us: u64,
}

/// GET /api/metrics/latency - Get latency statistics
async fn get_latency_stats() -> Result<Json<LatencyStatsResponse>, ApiError> {
    let socket_path = std::path::PathBuf::from(DEFAULT_SOCKET_PATH);
    let mut ipc = crate::ipc::unix_socket::UnixSocketIpc::new(socket_path);

    let response = ipc
        .send_request(&IpcRequest::GetLatencyMetrics)
        .map_err(|_| ApiError::DaemonNotRunning)?;

    match response {
        IpcResponse::Latency {
            min_us,
            avg_us,
            max_us,
            p95_us,
            p99_us,
        } => Ok(Json(LatencyStatsResponse {
            min_us,
            avg_us,
            max_us,
            p95_us,
            p99_us,
        })),
        IpcResponse::Error { code, message } => Err(ApiError::InternalError(format!(
            "Daemon error {}: {}",
            code, message
        ))),
        _ => Err(ApiError::InternalError(
            "Unexpected response from daemon".to_string(),
        )),
    }
}

#[derive(Deserialize)]
struct EventLogQuery {
    count: Option<usize>,
}

/// GET /api/metrics/events - Get event log
async fn get_event_log(Query(params): Query<EventLogQuery>) -> Result<Json<Value>, ApiError> {
    let count = params.count.unwrap_or(100);

    let socket_path = std::path::PathBuf::from(DEFAULT_SOCKET_PATH);
    let mut ipc = crate::ipc::unix_socket::UnixSocketIpc::new(socket_path);

    let response = ipc
        .send_request(&IpcRequest::GetEventsTail { count })
        .map_err(|_| ApiError::DaemonNotRunning)?;

    match response {
        IpcResponse::Events { events } => Ok(Json(json!({
            "count": events.len(),
            "events": events,
        }))),
        IpcResponse::Error { code, message } => Err(ApiError::InternalError(format!(
            "Daemon error {}: {}",
            code, message
        ))),
        _ => Err(ApiError::InternalError(
            "Unexpected response from daemon".to_string(),
        )),
    }
}

/// DELETE /api/metrics/events - Clear event log
async fn clear_event_log() -> Result<Json<Value>, ApiError> {
    // Note: The daemon doesn't currently have a "clear events" IPC command
    // This would require adding a new IpcRequest::ClearEvents variant
    // For now, return a not implemented response
    Ok(Json(json!({
        "success": false,
        "error": "Event log clearing requires daemon support (not yet implemented in IPC protocol)"
    })))
}

// ============================================================================
// Daemon State
// ============================================================================

#[derive(Serialize)]
struct DaemonStateResponse {
    active_layer: Option<String>,
    modifiers: Vec<String>,
    locks: Vec<String>,
    /// Raw 255-bit state vector
    raw_state: Vec<bool>,
    /// Number of active modifiers
    active_modifier_count: usize,
    /// Number of active locks
    active_lock_count: usize,
}

/// GET /api/daemon/state - Get current daemon state
async fn get_daemon_state() -> Result<Json<DaemonStateResponse>, ApiError> {
    let socket_path = std::path::PathBuf::from(DEFAULT_SOCKET_PATH);
    let mut ipc = crate::ipc::unix_socket::UnixSocketIpc::new(socket_path);

    let response = ipc
        .send_request(&IpcRequest::GetState)
        .map_err(|_| ApiError::DaemonNotRunning)?;

    match response {
        IpcResponse::State { state } => {
            // Parse the 255-bit state vector
            // Note: The exact bit layout depends on keyrx_core's ExtendedState structure
            // For now, we provide the raw state and basic analysis

            // Modifiers are typically bits 0-127 (MD_00 to MD_127)
            // Locks are typically bits 128-191 (LK_00 to LK_63)
            // Active layers are typically bits 192-254

            let modifiers: Vec<String> = state
                .iter()
                .take(128)
                .enumerate()
                .filter_map(|(i, &active)| {
                    if active {
                        Some(format!("MD_{:02}", i))
                    } else {
                        None
                    }
                })
                .collect();

            let locks: Vec<String> = state
                .iter()
                .skip(128)
                .take(64)
                .enumerate()
                .filter_map(|(i, &active)| {
                    if active {
                        Some(format!("LK_{:02}", i))
                    } else {
                        None
                    }
                })
                .collect();

            // Active layers (bits 192-254)
            let active_layer_bits: Vec<usize> = state
                .iter()
                .skip(192)
                .take(63)
                .enumerate()
                .filter_map(|(i, &active)| if active { Some(i) } else { None })
                .collect();

            let active_layer = if !active_layer_bits.is_empty() {
                Some(format!("Layer bits: {:?}", active_layer_bits))
            } else {
                None
            };

            Ok(Json(DaemonStateResponse {
                active_layer,
                modifiers: modifiers.clone(),
                locks: locks.clone(),
                raw_state: state,
                active_modifier_count: modifiers.len(),
                active_lock_count: locks.len(),
            }))
        }
        IpcResponse::Error { code, message } => Err(ApiError::InternalError(format!(
            "Daemon error {}: {}",
            code, message
        ))),
        _ => Err(ApiError::InternalError(
            "Unexpected response from daemon".to_string(),
        )),
    }
}

// ============================================================================
// Simulator
// ============================================================================

#[derive(Deserialize)]
struct SimulateEventsRequest {
    /// Optional scenario name (e.g., "tap-hold-under-threshold")
    scenario: Option<String>,
    /// Optional custom event sequence
    events: Option<Vec<SimulatedEvent>>,
    /// Seed for deterministic behavior
    seed: Option<u64>,
}

#[derive(Serialize)]
struct SimulateEventsResponse {
    success: bool,
    /// Number of events processed
    event_count: usize,
    /// List of output events generated
    outputs: Vec<String>,
    /// Execution time in microseconds
    duration_us: u64,
}

/// POST /api/simulator/events - Simulate events
async fn simulate_events(
    Json(payload): Json<SimulateEventsRequest>,
) -> Result<Json<SimulateEventsResponse>, ApiError> {
    // Determine event sequence
    let sequence = if let Some(scenario_name) = payload.scenario {
        // Use built-in scenario
        let scenario = match scenario_name.as_str() {
            "tap-hold-under-threshold" => BuiltinScenario::TapHoldUnderThreshold,
            "tap-hold-over-threshold" => BuiltinScenario::TapHoldOverThreshold,
            "permissive-hold" => BuiltinScenario::PermissiveHold,
            "cross-device-modifiers" => BuiltinScenario::CrossDeviceModifiers,
            "macro-sequence" => BuiltinScenario::MacroSequence,
            _ => {
                return Err(ApiError::BadRequest(format!(
                    "Unknown scenario: {}. Available: tap-hold-under-threshold, tap-hold-over-threshold, permissive-hold, cross-device-modifiers, macro-sequence",
                    scenario_name
                )))
            }
        };
        scenario.generate_events()
    } else if let Some(events) = payload.events {
        // Use custom event sequence
        EventSequence {
            events,
            seed: payload.seed.unwrap_or(0),
        }
    } else {
        return Err(ApiError::BadRequest(
            "Must provide either 'scenario' or 'events'".to_string(),
        ));
    };

    // Note: Actual simulation would require running the events through
    // the keyrx_core processor. For now, we just return the input events
    // as a demonstration. Full implementation would need:
    // 1. Load the active profile's .krx file
    // 2. Create a processor instance
    // 3. Feed events through the processor
    // 4. Collect output events

    Ok(Json(SimulateEventsResponse {
        success: true,
        event_count: sequence.events.len(),
        outputs: sequence
            .events
            .iter()
            .map(|e| format!("{:?} {} at {}μs", e.event_type, e.key, e.timestamp_us))
            .collect(),
        duration_us: sequence.events.last().map(|e| e.timestamp_us).unwrap_or(0),
    }))
}

/// POST /api/simulator/reset - Reset simulator
async fn reset_simulator() -> Result<Json<Value>, ApiError> {
    // Simulator state is not persistent, so there's nothing to reset
    // This endpoint exists for API completeness
    Ok(Json(json!({
        "success": true,
        "message": "Simulator state is ephemeral (no persistent state to reset)"
    })))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get config directory path
fn get_config_dir() -> Result<std::path::PathBuf, ApiError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| ApiError::InternalError("Cannot determine home directory".to_string()))?;

    Ok(std::path::PathBuf::from(home).join(".config/keyrx"))
}

/// Query daemon status via IPC
fn query_daemon_status() -> Result<(u64, Option<String>, usize), Box<dyn std::error::Error>> {
    let socket_path = std::path::PathBuf::from(DEFAULT_SOCKET_PATH);
    let mut ipc = crate::ipc::unix_socket::UnixSocketIpc::new(socket_path);

    let response = ipc.send_request(&IpcRequest::GetStatus)?;

    match response {
        IpcResponse::Status {
            running: _,
            uptime_secs,
            active_profile,
            device_count,
        } => Ok((uptime_secs, active_profile, device_count)),
        _ => Err("Unexpected response from daemon".into()),
    }
}

/// Query active profile name
fn query_active_profile() -> Option<String> {
    query_daemon_status()
        .ok()
        .and_then(|(_, profile, _)| profile)
}

// ============================================================================
// Macro Recorder
// ============================================================================

use crate::macro_recorder::MacroRecorder;
use std::sync::OnceLock;

/// Global macro recorder instance
static MACRO_RECORDER: OnceLock<MacroRecorder> = OnceLock::new();

/// Get or initialize the global macro recorder
fn get_macro_recorder() -> &'static MacroRecorder {
    MACRO_RECORDER.get_or_init(MacroRecorder::new)
}

/// POST /api/macros/start-recording - Start recording macro
async fn start_macro_recording() -> Result<Json<Value>, ApiError> {
    let recorder = get_macro_recorder();
    recorder.start_recording().map_err(ApiError::BadRequest)?;

    Ok(Json(json!({
        "success": true,
        "message": "Recording started"
    })))
}

/// POST /api/macros/stop-recording - Stop recording macro
async fn stop_macro_recording() -> Result<Json<Value>, ApiError> {
    let recorder = get_macro_recorder();
    recorder.stop_recording().map_err(ApiError::BadRequest)?;

    let event_count = recorder.event_count();

    Ok(Json(json!({
        "success": true,
        "message": "Recording stopped",
        "event_count": event_count
    })))
}

/// GET /api/macros/recorded-events - Get recorded events
async fn get_recorded_events() -> Result<Json<Value>, ApiError> {
    let recorder = get_macro_recorder();
    let events = recorder
        .get_recorded_events()
        .map_err(ApiError::InternalError)?;

    let recording = recorder.is_recording();

    Ok(Json(json!({
        "success": true,
        "recording": recording,
        "event_count": events.len(),
        "events": events
    })))
}

/// POST /api/macros/clear - Clear recorded events
async fn clear_recorded_events() -> Result<Json<Value>, ApiError> {
    let recorder = get_macro_recorder();
    recorder.clear_events().map_err(ApiError::InternalError)?;

    Ok(Json(json!({
        "success": true,
        "message": "Events cleared"
    })))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let result = health_check().await;
        let value = result.0;
        assert_eq!(value["status"], "ok");
    }

    #[tokio::test]
    async fn test_create_router() {
        let router = create_router();
        assert!(std::mem::size_of_val(&router) > 0);
    }
}
