//! Health check and metrics endpoints.

use axum::{extract::Query, routing::get, Json, Router};

// Needed for route chaining: .route("/metrics/events", get(...).delete(...))
#[allow(unused_imports)]
use axum::routing::delete;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use super::error::ApiError;
use crate::ipc::{DaemonIpc, IpcRequest, IpcResponse, DEFAULT_SOCKET_PATH};
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_check))
        .route("/version", get(get_version))
        .route("/status", get(get_status))
        .route("/metrics/latency", get(get_latency_stats))
        .route(
            "/metrics/events",
            get(get_event_log).delete(clear_event_log),
        )
        .route("/daemon/state", get(get_daemon_state))
}

/// GET /api/health - Health check
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Version information response
#[derive(Serialize)]
struct VersionInfo {
    /// Daemon version from Cargo.toml
    version: String,
    /// Build timestamp (RFC3339 format)
    build_time: String,
    /// Git commit hash (short)
    #[serde(skip_serializing_if = "Option::is_none")]
    git_hash: Option<String>,
    /// Target platform
    platform: String,
}

/// GET /api/version - Get version and build information
async fn get_version() -> Json<VersionInfo> {
    Json(VersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_time: env!("BUILD_TIMESTAMP").to_string(),
        git_hash: option_env!("GIT_HASH").map(|s| s.to_string()),
        platform: std::env::consts::OS.to_string(),
    })
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
