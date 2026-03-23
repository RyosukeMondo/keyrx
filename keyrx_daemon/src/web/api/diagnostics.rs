//! Diagnostics endpoint for comprehensive system health information.

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::error::DaemonError;
use crate::version;
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/diagnostics", get(get_diagnostics))
        .route("/diagnostics/ime", get(get_ime_status))
        .route("/diagnostics/full", get(get_full_diagnostics))
        .route("/diagnostics/routes", get(get_routes_info))
        .route("/diagnostics/frontend", get(get_frontend_status))
        .route("/diagnostics/build", get(get_build_info))
        .route("/debug/state", get(get_debug_state))
        .route("/debug/config/:name", get(get_debug_config))
        .route("/debug/log-level", post(set_debug_log_level))
        .route("/debug/suspend", post(set_suspend_state))
        .route("/keyboard/labels", get(get_keyboard_labels))
}

/// Comprehensive diagnostics information
#[derive(Serialize, Deserialize)]
pub struct DiagnosticsResponse {
    /// Daemon version from Cargo.toml
    pub version: String,
    /// Build timestamp
    pub build_time: String,
    /// Git commit hash
    pub git_hash: String,
    /// Binary file modification timestamp (if available)
    pub binary_timestamp: Option<String>,
    /// Whether running with administrator privileges
    pub admin_status: bool,
    /// Key blocker hook installation status
    pub hook_status: HookStatus,
    /// Platform information
    pub platform_info: PlatformInfo,
    /// Memory usage information
    pub memory_usage: MemoryUsage,
    /// Configuration validation status
    pub config_validation_status: ConfigStatus,
}

/// Hook installation status
#[derive(Serialize, Deserialize)]
pub struct HookStatus {
    /// Whether the hook is installed
    pub installed: bool,
    /// Number of keys currently being remapped
    pub remapped_keys_count: usize,
}

/// Platform information
#[derive(Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Operating system name
    pub os: String,
    /// System architecture
    pub arch: String,
}

/// Memory usage information
#[derive(Serialize, Deserialize)]
pub struct MemoryUsage {
    /// Process memory usage in bytes
    pub process_memory_bytes: u64,
    /// Process memory usage in human-readable format
    pub process_memory_human: String,
}

/// Configuration validation status
#[derive(Serialize, Deserialize)]
pub struct ConfigStatus {
    /// Whether configuration is valid
    pub valid: bool,
    /// Validation message or error
    pub message: String,
}

/// GET /api/diagnostics - Get comprehensive system diagnostics
/// Simple build information for quick verification
#[derive(Serialize, Deserialize)]
pub struct BuildInfo {
    pub version: String,
    pub build_time: String,
    pub git_hash: String,
    pub binary_timestamp: Option<String>,
}

/// Get simple build information (lightweight endpoint for verification)
async fn get_build_info() -> Result<Json<BuildInfo>, DaemonError> {
    tokio::task::spawn_blocking(move || {
        let binary_timestamp = get_binary_timestamp();

        Ok::<Json<BuildInfo>, DaemonError>(Json(BuildInfo {
            version: version::VERSION.to_string(),
            build_time: version::BUILD_DATE.to_string(),
            git_hash: version::GIT_HASH.to_string(),
            binary_timestamp,
        }))
    })
    .await
    .map_err(|e| {
        DaemonError::from(crate::error::ConfigError::ParseError {
            path: std::path::PathBuf::from("build-info"),
            reason: format!("Task join error: {}", e),
        })
    })?
}

async fn get_diagnostics() -> Result<Json<DiagnosticsResponse>, DaemonError> {
    tokio::task::spawn_blocking(move || {
        let binary_timestamp = get_binary_timestamp();
        let admin_status = check_admin_status();
        let hook_status = get_hook_status();
        let platform_info = PlatformInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        };
        let memory_usage = get_memory_usage();
        let config_validation_status = check_config_validation();
        Ok::<Json<DiagnosticsResponse>, DaemonError>(Json(DiagnosticsResponse {
            version: version::VERSION.to_string(),
            build_time: version::BUILD_DATE.to_string(),
            git_hash: version::GIT_HASH.to_string(),
            binary_timestamp,
            admin_status,
            hook_status,
            platform_info,
            memory_usage,
            config_validation_status,
        }))
    })
    .await
    .map_err(|e| {
        use crate::error::ConfigError;
        DaemonError::from(ConfigError::ParseError {
            path: std::path::PathBuf::from("diagnostics"),
            reason: format!("Task join error: {}", e),
        })
    })?
}

/// IME status diagnostic endpoint — returns detailed IME detection info
async fn get_ime_status() -> Json<Value> {
    #[cfg(target_os = "windows")]
    {
        let debug = crate::platform::windows::ime::query_windows_ime_debug();
        Json(serde_json::json!(debug))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Json(serde_json::json!({
            "active": null,
            "language": null,
            "platform": std::env::consts::OS,
            "note": "IME detection not implemented for this platform",
        }))
    }
}

/// Get binary file modification timestamp
fn get_binary_timestamp() -> Option<String> {
    std::env::current_exe()
        .ok()
        .and_then(|path| std::fs::metadata(path).ok())
        .and_then(|metadata| metadata.modified().ok())
        .map(|modified| {
            use std::time::SystemTime;
            let duration = modified
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default();
            // Format as RFC 3339 timestamp
            let secs = duration.as_secs();
            let datetime =
                chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, 0).unwrap_or_default();
            datetime.to_rfc3339()
        })
}

/// Check if running with administrator privileges
#[cfg(target_os = "windows")]
fn check_admin_status() -> bool {
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
fn check_admin_status() -> bool {
    // On Linux, check if running as root
    unsafe { libc::geteuid() == 0 }
}

/// Get hook installation status
#[cfg(target_os = "windows")]
fn get_hook_status() -> HookStatus {
    use crate::platform::windows::platform_state::PlatformState;

    if let Some(state_arc) = PlatformState::get() {
        if let Ok(state) = state_arc.lock() {
            if let Some(ref blocker) = state.key_blocker {
                return HookStatus {
                    installed: true,
                    remapped_keys_count: blocker.blocked_count(),
                };
            }
        }
    }

    HookStatus {
        installed: false,
        remapped_keys_count: 0,
    }
}

#[cfg(not(target_os = "windows"))]
fn get_hook_status() -> HookStatus {
    // On Linux, we don't have a hook system in the same way
    // The evdev grab is the equivalent
    HookStatus {
        installed: true,        // Assume installed if daemon is running
        remapped_keys_count: 0, // Not tracked on Linux
    }
}

/// Get process memory usage
fn get_memory_usage() -> MemoryUsage {
    // Try reading from /proc/self/status on Linux
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            let bytes = kb * 1024;
                            return MemoryUsage {
                                process_memory_bytes: bytes,
                                process_memory_human: format_bytes(bytes),
                            };
                        }
                    }
                }
            }
        }
    }

    // For Windows and fallback: return unknown
    // Note: Getting process memory on Windows requires Win32_System_ProcessStatus feature
    // which is not currently enabled. This can be added later if needed.
    MemoryUsage {
        process_memory_bytes: 0,
        process_memory_human: "Not available".to_string(),
    }
}

/// Format byte count as human-readable string (B, KB, MB, GB)
#[cfg(any(target_os = "linux", test))]
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// GET /api/diagnostics/full - Get all diagnostics including routes and frontend status
async fn get_full_diagnostics() -> Result<Json<Value>, DaemonError> {
    tokio::task::spawn_blocking(move || {
        let basic_diag = tokio::runtime::Handle::current()
            .block_on(get_diagnostics())
            .map_err(|e| {
                DaemonError::from(crate::error::ConfigError::ParseError {
                    path: std::path::PathBuf::from("full-diagnostics"),
                    reason: format!("Failed to get basic diagnostics: {}", e),
                })
            })?;

        let routes_info = get_routes_list();
        let frontend_status = get_frontend_info();

        Ok::<Json<Value>, DaemonError>(Json(serde_json::json!({
            "basic": basic_diag.0,
            "routes": routes_info,
            "frontend": frontend_status,
        })))
    })
    .await
    .map_err(|e| {
        DaemonError::from(crate::error::ConfigError::ParseError {
            path: std::path::PathBuf::from("full-diagnostics"),
            reason: format!("Task join error: {}", e),
        })
    })?
}

/// GET /api/diagnostics/routes - Get information about registered API routes
async fn get_routes_info() -> Json<Value> {
    Json(serde_json::json!(get_routes_list()))
}

/// GET /api/diagnostics/frontend - Get frontend bundle status
async fn get_frontend_status() -> Json<Value> {
    Json(serde_json::json!(get_frontend_info()))
}

/// Get list of registered API routes
fn get_routes_list() -> Value {
    let api = vec![
        "health",
        "version",
        "status",
        "metrics/latency",
        "metrics/events",
        "daemon/state",
        "diagnostics",
        "diagnostics/full",
        "diagnostics/routes",
        "diagnostics/frontend",
        "diagnostics/build",
        "devices",
        "devices/:device_id",
        "devices/:device_id/layout",
        "profiles",
        "profiles/:name",
        "profiles/:name/config",
        "profiles/:name/activate",
        "profiles/active",
        "config/:profile/layers",
        "config/:profile/layers/:layer",
        "layouts",
        "layouts/:layout_name",
        "simulator/press",
        "simulator/release",
        "simulator/sequence",
        "macros/start",
        "macros/stop",
        "macros/events",
        "macros/save",
        "debug/state",
        "debug/config/:name",
        "debug/log-level",
        "debug/suspend",
        "keyboard/labels",
    ];
    let api_routes: Vec<String> = api.into_iter().map(|r| format!("/api/{r}")).collect();
    serde_json::json!({
        "api_routes": api_routes,
        "websocket_routes": ["/ws", "/ws-rpc"],
        "frontend_routes": ["/", "/home", "/devices", "/profiles",
            "/profiles/:name/config", "/config", "/metrics", "/simulator"],
        "note": "Frontend routes are handled by React Router via SPA fallback"
    })
}

/// Get frontend bundle information
fn get_frontend_info() -> Value {
    let ui_dist_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../keyrx_ui/dist");
    serde_json::json!({
        "ui_embedded": true,
        "ui_dist_path": ui_dist_path.to_string_lossy(),
        "ui_dist_exists": ui_dist_path.exists(),
        "spa_fallback": "Enabled - all non-API routes serve index.html",
        "bundle_files": ["/index.html", "/assets/index-*.js",
            "/assets/vendor-*.js", "/assets/index-*.css"],
    })
}

/// Check configuration validation status
fn check_config_validation() -> ConfigStatus {
    use crate::config::ProfileManager;

    let Some(mut config_dir) = dirs::config_dir() else {
        return ConfigStatus {
            valid: false,
            message: "Cannot determine config directory".into(),
        };
    };
    config_dir.push("keyrx");

    let profile_manager = match ProfileManager::new(config_dir) {
        Ok(mgr) => mgr,
        Err(e) => {
            return ConfigStatus {
                valid: false,
                message: format!("Failed to initialize ProfileManager: {e}"),
            }
        }
    };

    match profile_manager.get_active() {
        Ok(Some(name)) => match profile_manager.get(&name) {
            Some(_) => ConfigStatus {
                valid: true,
                message: format!("Active profile '{name}' is valid"),
            },
            None => ConfigStatus {
                valid: false,
                message: format!("Active profile '{name}' not found"),
            },
        },
        Ok(None) => ConfigStatus {
            valid: true,
            message: "No active profile (running in pass-through mode)".into(),
        },
        Err(e) => ConfigStatus {
            valid: false,
            message: format!("Error reading active profile: {e}"),
        },
    }
}

/// GET /api/debug/state - Comprehensive daemon state snapshot for debugging
async fn get_debug_state(State(state): State<Arc<AppState>>) -> Json<Value> {
    let daemon_info = match state.daemon_state.as_ref() {
        Some(daemon) => {
            let active = daemon.get_active_profile();
            let mapping_count = active
                .as_ref()
                .map(|n| count_mappings_for_profile(&state, n))
                .unwrap_or(0);
            serde_json::json!({
                "running": daemon.is_running(), "uptime_secs": daemon.uptime_secs(),
                "active_profile": active, "device_count": daemon.get_device_count(),
                "mapping_count": mapping_count, "suspended": daemon.is_suspended(),
            })
        }
        None => serde_json::json!({
            "running": false, "uptime_secs": null, "active_profile": null,
            "device_count": 0, "mapping_count": 0,
            "note": "daemon_state not available (test mode or Linux IPC)",
        }),
    };
    let config_info = build_config_info(&state).await;
    let profiles_info = build_profiles_info(&state).await;
    let hook = get_hook_status();
    Json(serde_json::json!({
        "daemon": daemon_info,
        "config": config_info,
        "profiles": profiles_info,
        "hook": { "installed": hook.installed, "remapped_keys_count": hook.remapped_keys_count },
        "ws_info": {
            "daemon_query_available": state.daemon_query.is_some(),
            "daemon_state_available": state.daemon_state.is_some(),
        },
    }))
}

/// Count mappings for a profile by reading its .krx file from disk.
fn count_mappings_for_profile(state: &AppState, name: &str) -> usize {
    let profiles_dir = state.profile_service.profile_manager().profiles_dir();
    let krx_path = profiles_dir.join(format!("{name}.krx"));
    let Ok(data) = std::fs::read(&krx_path) else {
        return 0;
    };
    let Ok(archived) = keyrx_compiler::serialize::deserialize(&data) else {
        return 0;
    };
    use rkyv::Deserialize;
    let config: keyrx_core::config::ConfigRoot =
        archived.deserialize(&mut rkyv::Infallible).unwrap();
    config.devices.iter().map(|d| d.mappings.len()).sum()
}

/// Build config info for the active profile.
async fn build_config_info(state: &AppState) -> Value {
    let active_name = state.profile_service.get_active_profile().await;
    let Some(name) = active_name else {
        return serde_json::json!({
            "active_profile": null,
            "source": null,
            "file_size_bytes": null,
            "last_modified": null,
        });
    };

    let profiles_dir = state.profile_service.profile_manager().profiles_dir();
    let rhai_path = profiles_dir.join(format!("{name}.rhai"));

    let source = state.profile_service.get_profile_config(&name).await.ok();
    let (file_size, last_modified) = std::fs::metadata(&rhai_path)
        .ok()
        .map(|m| {
            let size = m.len();
            let modified = m
                .modified()
                .ok()
                .and_then(|t| {
                    let d = t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok()?;
                    chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0)
                })
                .map(|dt| dt.to_rfc3339());
            (Some(size), modified)
        })
        .unwrap_or((None, None));

    serde_json::json!({
        "active_profile": name,
        "source": source,
        "file_size_bytes": file_size,
        "last_modified": last_modified,
    })
}

/// Build profiles listing with file sizes.
async fn build_profiles_info(state: &AppState) -> Value {
    let profiles_dir = state.profile_service.profile_manager().profiles_dir();
    let Ok(entries) = std::fs::read_dir(&profiles_dir) else {
        return serde_json::json!([]);
    };

    let mut profiles = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rhai") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        profiles.push(serde_json::json!({
            "name": name,
            "file_size_bytes": file_size,
        }));
    }

    serde_json::json!(profiles)
}

/// GET /api/debug/config/:name - Raw .rhai source code for a profile
async fn get_debug_config(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Value>, DaemonError> {
    let source = state
        .profile_service
        .get_profile_config(&name)
        .await
        .map_err(|e| {
            DaemonError::from(crate::error::ConfigError::ParseError {
                path: std::path::PathBuf::from(format!("{name}.rhai")),
                reason: format!("Profile not found: {e}"),
            })
        })?;

    Ok(Json(serde_json::json!({
        "name": name,
        "source": source,
    })))
}

/// Request body for changing log level.
#[derive(Deserialize)]
struct LogLevelRequest {
    level: String,
}

/// POST /api/debug/log-level - Change runtime log level
async fn set_debug_log_level(
    Json(payload): Json<LogLevelRequest>,
) -> Result<Json<Value>, DaemonError> {
    let level = match payload.level.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        other => {
            return Err(DaemonError::from(crate::error::ConfigError::ParseError {
                path: std::path::PathBuf::from("log-level"),
                reason: format!(
                    "Invalid log level '{other}'. \
                         Use: trace, debug, info, warn, error"
                ),
            }));
        }
    };

    log::set_max_level(level);
    log::info!("Log level changed to: {}", payload.level);

    Ok(Json(serde_json::json!({
        "level": payload.level.to_lowercase(),
        "applied": true,
    })))
}

/// Request body for suspend/resume.
#[derive(Deserialize)]
struct SuspendRequest {
    suspended: bool,
}

/// POST /api/debug/suspend - Suspend or resume key remapping
async fn set_suspend_state(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SuspendRequest>,
) -> Result<Json<Value>, DaemonError> {
    match state.daemon_state.as_ref() {
        Some(daemon) => {
            daemon.set_suspended(payload.suspended);
            log::info!(
                "Daemon {} via REST API",
                if payload.suspended {
                    "suspended"
                } else {
                    "resumed"
                }
            );
            Ok(Json(serde_json::json!({
                "suspended": payload.suspended,
                "applied": true,
            })))
        }
        None => Err(DaemonError::from(crate::error::ConfigError::ParseError {
            path: std::path::PathBuf::from("suspend"),
            reason: "Daemon state not available".to_string(),
        })),
    }
}

/// Response for keyboard label detection
#[derive(Serialize, Deserialize)]
pub struct KeyboardLabelsResponse {
    /// Detected keyboard layout name
    pub detected_layout: String,
    /// Map of KeyCode name -> display label
    pub labels: std::collections::HashMap<String, String>,
}

/// GET /api/keyboard/labels - Detect keyboard layout and return display labels
#[cfg(target_os = "windows")]
async fn get_keyboard_labels() -> Result<Json<KeyboardLabelsResponse>, DaemonError> {
    tokio::task::spawn_blocking(move || {
        let labels = detect_keyboard_labels();
        Ok::<Json<KeyboardLabelsResponse>, DaemonError>(Json(labels))
    })
    .await
    .map_err(|e| {
        DaemonError::from(crate::error::ConfigError::ParseError {
            path: std::path::PathBuf::from("keyboard-labels"),
            reason: format!("Task join error: {}", e),
        })
    })?
}

#[cfg(not(target_os = "windows"))]
async fn get_keyboard_labels() -> Result<Json<KeyboardLabelsResponse>, DaemonError> {
    Ok(Json(KeyboardLabelsResponse {
        detected_layout: "Unknown (non-Windows)".to_string(),
        labels: std::collections::HashMap::new(),
    }))
}

/// Detect the current keyboard layout and compute display labels
/// for each scan code.
#[cfg(target_os = "windows")]
fn detect_keyboard_labels() -> KeyboardLabelsResponse {
    use std::collections::HashMap;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetKeyboardLayout, MapVirtualKeyW, ToUnicodeEx, MAPVK_VSC_TO_VK_EX,
    };

    let hkl = unsafe { GetKeyboardLayout(0) };

    // Extract layout ID for display name
    let layout_id = (hkl as usize) & 0xFFFF;
    let detected_layout = match layout_id {
        0x0411 => "Japanese (109-key)".to_string(),
        0x0409 => "US English (ANSI)".to_string(),
        0x0809 => "UK English (ISO)".to_string(),
        0x0407 => "German (QWERTZ)".to_string(),
        0x040C => "French (AZERTY)".to_string(),
        _ => format!("Layout 0x{:04X}", layout_id),
    };

    // Scan codes to probe (all layout-dependent keys)
    let scan_label_pairs: &[(u32, &str)] = &[
        (0x29, "Grave"),
        (0x0C, "Minus"),
        (0x0D, "Equal"),
        (0x1A, "LeftBracket"),
        (0x1B, "RightBracket"),
        (0x2B, "Backslash"),
        (0x27, "Semicolon"),
        (0x28, "Quote"),
        (0x33, "Comma"),
        (0x34, "Period"),
        (0x35, "Slash"),
        (0x02, "Num1"),
        (0x03, "Num2"),
        (0x04, "Num3"),
        (0x05, "Num4"),
        (0x06, "Num5"),
        (0x07, "Num6"),
        (0x08, "Num7"),
        (0x09, "Num8"),
        (0x0A, "Num9"),
        (0x0B, "Num0"),
    ];

    let mut labels = HashMap::new();
    let key_state = [0u8; 256];

    for &(scancode, name) in scan_label_pairs {
        let vk = unsafe { MapVirtualKeyW(scancode, MAPVK_VSC_TO_VK_EX) };
        if vk == 0 {
            continue;
        }

        let mut buf = [0u16; 4];
        let result = unsafe {
            ToUnicodeEx(
                vk,
                scancode,
                key_state.as_ptr(),
                buf.as_mut_ptr(),
                buf.len() as i32,
                0,
                hkl,
            )
        };

        if result > 0 {
            let label = String::from_utf16_lossy(&buf[..result as usize]);
            labels.insert(name.to_string(), label);
        }
    }

    KeyboardLabelsResponse {
        detected_layout,
        labels,
    }
}

#[cfg(test)]
mod tests;
