//! Build/version info, admin & hook status, memory usage, IME detection, and
//! config validation — the data behind the core `/diagnostics*` endpoints.

use axum::Json;
use serde_json::Value;

use crate::error::DaemonError;
use crate::version;

use super::types::{
    BuildInfo, ConfigStatus, DiagnosticsResponse, HookStatus, MemoryUsage, PlatformInfo,
};

/// Get simple build information (lightweight endpoint for verification)
pub(super) async fn get_build_info() -> Result<Json<BuildInfo>, DaemonError> {
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

/// GET /api/diagnostics - Get comprehensive system diagnostics
pub(super) async fn get_diagnostics() -> Result<Json<DiagnosticsResponse>, DaemonError> {
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
pub(super) async fn get_ime_status() -> Json<Value> {
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
    nix::unistd::geteuid().is_root()
}

/// Get hook installation status
#[cfg(target_os = "windows")]
pub(super) fn get_hook_status() -> HookStatus {
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
pub(super) fn get_hook_status() -> HookStatus {
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
pub(super) fn format_bytes(bytes: u64) -> String {
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

/// Check configuration validation status
fn check_config_validation() -> ConfigStatus {
    use crate::config::ProfileManager;

    let Ok(config_dir) = crate::cli::config_dir::get_config_dir() else {
        return ConfigStatus {
            valid: false,
            message: "Cannot determine config directory".into(),
        };
    };

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
            message: "No active profile (no config is live; no keyboard is grabbed)".into(),
        },
        Err(e) => ConfigStatus {
            valid: false,
            message: format!("Error reading active profile: {e}"),
        },
    }
}
