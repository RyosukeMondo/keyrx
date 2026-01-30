//! Diagnostics endpoint for comprehensive system health information.

use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::DaemonError;
use crate::version;
use crate::web::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/diagnostics", get(get_diagnostics))
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
    /// Number of keys currently being blocked
    pub blocked_keys_count: usize,
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
async fn get_diagnostics() -> Result<Json<DiagnosticsResponse>, DaemonError> {
    // Wrap all operations in spawn_blocking for consistency with other endpoints
    tokio::task::spawn_blocking(move || {
        // Get binary timestamp
        let binary_timestamp = get_binary_timestamp();

        // Get admin status
        let admin_status = check_admin_status();

        // Get hook status
        let hook_status = get_hook_status();

        // Get platform info
        let platform_info = PlatformInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        };

        // Get memory usage
        let memory_usage = get_memory_usage();

        // Get config validation status
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
            let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, 0)
                .unwrap_or_default();
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
                    blocked_keys_count: blocker.blocked_count(),
                };
            }
        }
    }

    HookStatus {
        installed: false,
        blocked_keys_count: 0,
    }
}

#[cfg(not(target_os = "windows"))]
fn get_hook_status() -> HookStatus {
    // On Linux, we don't have a hook system in the same way
    // The evdev grab is the equivalent
    HookStatus {
        installed: true, // Assume installed if daemon is running
        blocked_keys_count: 0, // Not tracked on Linux
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

/// Format bytes in human-readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Check configuration validation status
fn check_config_validation() -> ConfigStatus {
    use crate::config::ProfileManager;

    // Get config directory
    let config_dir = match dirs::config_dir() {
        Some(mut dir) => {
            dir.push("keyrx");
            dir
        }
        None => {
            return ConfigStatus {
                valid: false,
                message: "Cannot determine config directory".to_string(),
            }
        }
    };

    // Try to load ProfileManager
    let profile_manager = match ProfileManager::new(config_dir.clone()) {
        Ok(mgr) => mgr,
        Err(e) => {
            return ConfigStatus {
                valid: false,
                message: format!("Failed to initialize ProfileManager: {}", e),
            }
        }
    };

    // Check if there's an active profile
    match profile_manager.get_active() {
        Ok(Some(active_name)) => {
            // Try to load the active profile
            match profile_manager.get(&active_name) {
                Some(_) => ConfigStatus {
                    valid: true,
                    message: format!("Active profile '{}' is valid", active_name),
                },
                None => ConfigStatus {
                    valid: false,
                    message: format!("Active profile '{}' not found", active_name),
                },
            }
        }
        Ok(None) => ConfigStatus {
            valid: true,
            message: "No active profile (running in pass-through mode)".to_string(),
        },
        Err(e) => ConfigStatus {
            valid: false,
            message: format!("Error reading active profile: {}", e),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_platform_info() {
        let platform_info = PlatformInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        };
        assert!(!platform_info.os.is_empty());
        assert!(!platform_info.arch.is_empty());
    }

    #[test]
    fn test_diagnostics_response_serialization() {
        let response = DiagnosticsResponse {
            version: "0.1.0".to_string(),
            build_time: "2024-01-01".to_string(),
            git_hash: "abc123".to_string(),
            binary_timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            admin_status: true,
            hook_status: HookStatus {
                installed: true,
                blocked_keys_count: 5,
            },
            platform_info: PlatformInfo {
                os: "windows".to_string(),
                arch: "x86_64".to_string(),
            },
            memory_usage: MemoryUsage {
                process_memory_bytes: 10485760,
                process_memory_human: "10.00 MB".to_string(),
            },
            config_validation_status: ConfigStatus {
                valid: true,
                message: "Configuration is valid".to_string(),
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"version\":\"0.1.0\""));
        assert!(json.contains("\"admin_status\":true"));
    }
}
