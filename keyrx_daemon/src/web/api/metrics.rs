//! Health check and metrics endpoints.

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};

// Needed for route chaining: .route("/metrics/events", get(...).delete(...))
#[allow(unused_imports)]
use axum::routing::delete;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::{DaemonError, SocketError};
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

/// Enhanced health check response with version and system info
#[derive(Serialize)]
struct HealthCheckResponse {
    status: String,
    version: String,
    build_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    git_hash: Option<String>,
    platform: String,
    admin_rights: bool,
    hook_installed: bool,
}

/// GET /api/health - Enhanced health check with runtime information
async fn health_check() -> Json<HealthCheckResponse> {
    use crate::version;

    // Check if running as admin
    let admin_rights = check_admin_privileges();

    // Get hook installation status
    let hook_installed = check_hook_installed();

    Json(HealthCheckResponse {
        status: "ok".to_string(),
        version: version::VERSION.to_string(),
        build_time: version::BUILD_DATE.to_string(),
        git_hash: Some(version::GIT_HASH.to_string()).filter(|s| !s.is_empty() && s != "unknown"),
        platform: std::env::consts::OS.to_string(),
        admin_rights,
        hook_installed,
    })
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

async fn get_status(
    State(state): State<Arc<crate::web::AppState>>,
) -> Result<Json<StatusResponse>, DaemonError> {
    // Check if test mode is enabled
    if let Some(socket_path) = &state.test_mode_socket {
        // Test mode: use IPC to query daemon status with timeout
        use crate::ipc::{unix_socket::UnixSocketIpc, DaemonIpc, IpcRequest, IpcResponse};
        use std::time::Duration;

        let socket_path = socket_path.clone();
        let result = tokio::time::timeout(Duration::from_secs(5), async move {
            tokio::task::spawn_blocking(move || {
                let mut ipc = UnixSocketIpc::new(socket_path);
                ipc.send_request(&IpcRequest::GetStatus)
            })
            .await
        })
        .await;

        let (daemon_running, uptime_secs, active_profile, device_count) = match result {
            Ok(Ok(Ok(IpcResponse::Status {
                running,
                uptime_secs: uptime,
                active_profile: profile,
                device_count: count,
            }))) => (running, Some(uptime), profile, Some(count)),
            Ok(Ok(Err(e))) => {
                log::warn!("IPC error querying daemon status: {}", e);
                (false, None, None, None)
            }
            Ok(Err(e)) => {
                log::warn!("Failed to join IPC task: {}", e);
                (false, None, None, None)
            }
            Err(_) => {
                log::warn!("IPC timeout querying daemon status");
                (false, None, None, None)
            }
            _ => (false, None, None, None),
        };

        Ok(Json(StatusResponse {
            status: "running".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            daemon_running,
            uptime_secs,
            active_profile,
            device_count,
        }))
    } else {
        // Production mode: try to query daemon via IPC
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
async fn get_latency_stats() -> Result<Json<LatencyStatsResponse>, DaemonError> {
    use crate::error::WebError;

    let socket_path = std::path::PathBuf::from(DEFAULT_SOCKET_PATH);
    let mut ipc = crate::ipc::unix_socket::UnixSocketIpc::new(socket_path);

    let response = ipc
        .send_request(&IpcRequest::GetLatencyMetrics)
        .map_err(|_| SocketError::NotConnected)?;

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
        IpcResponse::Error { code, message } => Err(WebError::InvalidRequest {
            reason: format!("Daemon error {}: {}", code, message),
        }
        .into()),
        _ => Err(WebError::InvalidRequest {
            reason: "Unexpected response from daemon".to_string(),
        }
        .into()),
    }
}

#[derive(Deserialize)]
struct EventLogQuery {
    count: Option<usize>,
}

/// GET /api/metrics/events - Get event log
async fn get_event_log(Query(params): Query<EventLogQuery>) -> Result<Json<Value>, DaemonError> {
    use crate::error::WebError;

    let count = params.count.unwrap_or(100);

    let socket_path = std::path::PathBuf::from(DEFAULT_SOCKET_PATH);
    let mut ipc = crate::ipc::unix_socket::UnixSocketIpc::new(socket_path);

    let response = ipc
        .send_request(&IpcRequest::GetEventsTail { count })
        .map_err(|_| SocketError::NotConnected)?;

    match response {
        IpcResponse::Events { events } => Ok(Json(json!({
            "count": events.len(),
            "events": events,
        }))),
        IpcResponse::Error { code, message } => Err(WebError::InvalidRequest {
            reason: format!("Daemon error {}: {}", code, message),
        }
        .into()),
        _ => Err(WebError::InvalidRequest {
            reason: "Unexpected response from daemon".to_string(),
        }
        .into()),
    }
}

/// DELETE /api/metrics/events - Clear event log
async fn clear_event_log() -> Result<Json<Value>, DaemonError> {
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
async fn get_daemon_state() -> Result<Json<DaemonStateResponse>, DaemonError> {
    use crate::error::WebError;

    let socket_path = std::path::PathBuf::from(DEFAULT_SOCKET_PATH);
    let mut ipc = crate::ipc::unix_socket::UnixSocketIpc::new(socket_path);

    let response = ipc
        .send_request(&IpcRequest::GetState)
        .map_err(|_| SocketError::NotConnected)?;

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
        IpcResponse::Error { code, message } => Err(WebError::InvalidRequest {
            reason: format!("Daemon error {}: {}", code, message),
        }
        .into()),
        _ => Err(WebError::InvalidRequest {
            reason: "Unexpected response from daemon".to_string(),
        }
        .into()),
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

/// Check if running with administrator privileges
#[cfg(target_os = "windows")]
fn check_admin_privileges() -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
        let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

        let result = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size,
            &mut size,
        );

        CloseHandle(token);
        result != 0 && elevation.TokenIsElevated != 0
    }
}

#[cfg(not(target_os = "windows"))]
fn check_admin_privileges() -> bool {
    // On Linux, check if running as root
    unsafe { libc::geteuid() == 0 }
}

/// Check if key blocker hook is installed
#[cfg(target_os = "windows")]
fn check_hook_installed() -> bool {
    use crate::platform::windows::platform_state::PlatformState;

    if let Some(state_arc) = PlatformState::get() {
        if let Ok(state) = state_arc.lock() {
            return state.key_blocker.is_some();
        }
    }
    false
}

#[cfg(not(target_os = "windows"))]
fn check_hook_installed() -> bool {
    // On Linux, evdev grab is always available if daemon is running
    true
}
