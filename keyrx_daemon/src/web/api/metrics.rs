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
use std::sync::Arc;
use typeshare::typeshare;

use crate::web::events::{DaemonState, KeyEventData, LatencyStats};
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
    uptime_secs: u64,
    active_profile: Option<String>,
    device_count: usize,
    input_overflows: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    config_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_device: Option<crate::platform::OutputDeviceInfo>,
    web_server: crate::web_server_status::WebServerStatus,
}

async fn get_status(State(state): State<Arc<AppState>>) -> Json<StatusResponse> {
    let status = state.daemon_query.get_status();
    Json(StatusResponse {
        status: "running".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        daemon_running: status.daemon_running,
        uptime_secs: status.uptime_secs,
        active_profile: status.active_profile,
        device_count: status.device_count,
        input_overflows: status.input_overflows,
        config_error: status.config_error,
        output_device: status.output_device,
        web_server: status.web_server,
    })
}

/// GET /api/metrics/latency - Latency statistics (same type as the WS feed)
async fn get_latency_stats(State(state): State<Arc<AppState>>) -> Json<LatencyStats> {
    Json(state.daemon_query.get_latency_stats())
}

#[derive(Deserialize)]
struct EventLogQuery {
    count: Option<usize>,
}

/// GET /api/metrics/events - Most recent key events, oldest first (same
/// record as the WS `event` feed)
async fn get_event_log(
    State(state): State<Arc<AppState>>,
    Query(params): Query<EventLogQuery>,
) -> Json<Vec<KeyEventData>> {
    Json(
        state
            .daemon_query
            .get_recent_events(params.count.unwrap_or(100)),
    )
}

/// Result of clearing the event log.
#[typeshare]
#[derive(Serialize)]
pub struct ClearEventsResult {
    pub success: bool,
    /// Number of events removed.
    #[typeshare(serialized_as = "number")]
    pub cleared: usize,
}

/// DELETE /api/metrics/events - Clear event log
async fn clear_event_log(State(state): State<Arc<AppState>>) -> Json<ClearEventsResult> {
    Json(ClearEventsResult {
        success: true,
        cleared: state.daemon_query.clear_events(),
    })
}

/// GET /api/daemon/state - Current modifier/lock/layer state (same type as
/// the WS `state` feed)
async fn get_daemon_state(State(state): State<Arc<AppState>>) -> Json<DaemonState> {
    Json(state.daemon_query.get_daemon_state())
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
    nix::unistd::geteuid().is_root()
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
