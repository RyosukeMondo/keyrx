//! Platform initialization and setup.
//!
//! This module handles platform-specific initialization, including:
//! - Platform creation
//! - Permission checks
//! - Logging configuration
//! - Version information logging

use crate::daemon::ExitCode;
use crate::platform::Platform;

/// Initialize the platform with proper error handling.
///
/// # Returns
///
/// Returns a boxed Platform trait object on success.
///
/// # Errors
///
/// Returns error tuple `(exit_code, message)` if platform creation fails.
pub fn initialize_platform() -> Result<Box<dyn Platform>, (i32, String)> {
    crate::platform::create_platform().map_err(|e| {
        (
            ExitCode::RuntimeError as i32,
            format!("Failed to create platform: {}", e),
        )
    })
}

/// Initialize logging with the specified debug level.
///
/// # Arguments
///
/// * `debug` - Whether to enable debug-level logging
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub fn init_logging(debug: bool) {
    use env_logger::Builder;
    use log::LevelFilter;

    let level = if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    Builder::new()
        .filter_level(level)
        .format_timestamp_millis()
        .init();
}

/// Log startup version information and system status.
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub fn log_startup_version_info() {
    use crate::version;

    log::info!("========================================");
    log::info!("KeyRx Daemon Starting");
    log::info!("========================================");
    log::info!("Version:    {}", version::VERSION);
    log::info!("Build Time: {}", version::BUILD_DATE);
    log::info!("Git Hash:   {}", version::GIT_HASH);

    // Log binary timestamp
    if let Some(binary_ts) = get_binary_timestamp() {
        log::info!("Binary:     {}", binary_ts);
    }

    // Log admin rights status
    let admin_status = check_startup_admin_status();
    if admin_status {
        log::info!("Admin:      Running with administrator privileges");
    } else {
        log::warn!("Admin:      NOT running with administrator privileges");
        log::warn!("            Key remapping may not work for elevated applications");
    }

    // Log hook installation status
    log_hook_installation_status();

    log::info!("========================================");
}

/// Get binary timestamp for logging.
#[cfg(any(target_os = "linux", target_os = "windows"))]
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
            let secs = duration.as_secs();
            format!("{} seconds since epoch", secs)
        })
}

/// Check admin status for startup logging (Windows).
#[cfg(target_os = "windows")]
fn check_startup_admin_status() -> bool {
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

/// Check admin status for startup logging (Linux).
#[cfg(target_os = "linux")]
fn check_startup_admin_status() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Log hook installation status (Windows).
#[cfg(target_os = "windows")]
fn log_hook_installation_status() {
    log::info!("Hook:       Will attempt to install low-level keyboard hook");
}

/// Log hook installation status (Linux).
#[cfg(target_os = "linux")]
fn log_hook_installation_status() {
    log::info!("Hook:       Using evdev device grabbing (Linux)");
}

/// Log hook status after daemon initialization (Windows).
#[cfg(target_os = "windows")]
pub fn log_post_init_hook_status() {
    use crate::platform::windows::platform_state::PlatformState;

    if let Some(state_arc) = PlatformState::get() {
        if let Ok(state) = state_arc.lock() {
            if let Some(ref blocker) = state.key_blocker {
                let blocked_count = blocker.blocked_count();
                log::info!("✓ Low-level keyboard hook installed successfully");
                log::info!("  Currently blocking {} key(s)", blocked_count);
            } else {
                log::warn!("✗ Low-level keyboard hook NOT installed");
                log::warn!("  Double inputs may occur during remapping");
            }
        } else {
            log::warn!("✗ Failed to access platform state");
        }
    } else {
        log::warn!("✗ Platform state not initialized");
    }
}

/// Log hook status after daemon initialization (Linux).
#[cfg(target_os = "linux")]
pub fn log_post_init_hook_status() {
    log::info!("✓ Device grabbing configured (evdev)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_platform() {
        // Platform creation depends on OS features
        // This test verifies the function signature is correct
        let _result = initialize_platform();
    }
}
