//! Daemon run command handler.
//!
//! This module delegates to platform-specific implementations for running the daemon.

use std::path::PathBuf;
use std::sync::Arc;

use crate::cli::dispatcher::exit_codes;
use crate::container::ServiceContainerBuilder;

/// Handle the run command - delegates to platform-specific implementation.
///
/// # Arguments
///
/// * `config` - Optional path to configuration file
/// * `debug` - Enable debug logging
/// * `test_mode` - Enable test mode (no keyboard capture)
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err((exit_code, message))` on failure.
pub fn handle_run(
    config: Option<PathBuf>,
    debug: bool,
    test_mode: bool,
) -> Result<(), (i32, String)> {
    // Validate test mode early for release builds
    #[cfg(not(debug_assertions))]
    if test_mode {
        return Err((
            exit_codes::CONFIG_ERROR,
            "Test mode is only available in debug builds".to_string(),
        ));
    }

    // Resolve config path: use provided path or active profile
    let config_path = resolve_config_path(config)?;

    // Get config directory for ServiceContainer
    let config_dir = {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("keyrx");
        path
    };

    // Build ServiceContainer with all dependencies wired
    let mut builder = ServiceContainerBuilder::new(config_dir);

    // Configure test mode if enabled
    if test_mode {
        let test_socket = PathBuf::from(format!("/tmp/keyrx-test-{}.sock", std::process::id()));
        builder = builder.with_test_mode_socket(test_socket.clone());
    }

    // Build container - this replaces 20+ lines of manual service instantiation
    let container = Arc::new(builder.build().map_err(|e| {
        (
            exit_codes::CONFIG_ERROR,
            format!("Failed to initialize services: {}", e),
        )
    })?);

    // Delegate to platform-specific handler with ServiceContainer
    #[cfg(target_os = "linux")]
    return crate::daemon::platform_runners::linux::run_daemon(
        &config_path,
        debug,
        test_mode,
        container,
    );

    #[cfg(target_os = "windows")]
    return crate::daemon::platform_runners::windows::run_daemon(
        &config_path,
        debug,
        test_mode,
        container,
    );

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    Err((
        exit_codes::CONFIG_ERROR,
        "The 'run' command is only available on Linux and Windows. \
         Build with --features linux or --features windows to enable."
            .to_string(),
    ))
}

/// Resolve configuration file path from optional argument or active profile.
fn resolve_config_path(config: Option<PathBuf>) -> Result<PathBuf, (i32, String)> {
    match config {
        Some(path) => Ok(path),
        None => {
            // Get config directory
            let mut config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            config_dir.push("keyrx");

            // Try to initialize ProfileManager and get/create default profile
            use crate::config::{ProfileManager, ProfileTemplate};
            match ProfileManager::new(config_dir.clone()) {
                Ok(manager) => {
                    // Check if we have an active profile
                    match manager.get_active() {
                        Ok(Some(active)) => {
                            eprintln!("[INFO] Using active profile: {}", active);
                            let mut profile_path = config_dir.clone();
                            profile_path.push("profiles");
                            profile_path.push(format!("{}.krx", active));
                            Ok(profile_path)
                        }
                        Ok(None) => {
                            // No active profile - try to create and activate default
                            eprintln!(
                                "[INFO] No active profile found. Creating default profile..."
                            );

                            // Create default profile with blank template if it doesn't exist
                            let profile_exists = manager.get("default").is_some();
                            if !profile_exists {
                                eprintln!("[INFO] Creating default profile with blank template...");
                                if let Err(e) = manager.create("default", ProfileTemplate::Blank) {
                                    eprintln!(
                                        "[WARN] Failed to create default profile: {}. Running in pass-through mode.",
                                        e
                                    );
                                }
                            } else {
                                eprintln!("[INFO] Default profile exists, activating...");
                            }

                            // Activate default profile
                            if let Err(e) = manager.activate("default") {
                                eprintln!(
                                    "[WARN] Failed to activate default profile: {}. Running in pass-through mode.",
                                    e
                                );
                            }

                            // Return path to default.krx
                            let mut profile_path = config_dir.clone();
                            profile_path.push("profiles");
                            profile_path.push("default.krx");
                            eprintln!(
                                "[INFO] Using default profile at: {}",
                                profile_path.display()
                            );
                            Ok(profile_path)
                        }
                        Err(e) => {
                            // Error reading active profile - fall back
                            eprintln!(
                                "[WARN] Failed to read active profile: {}. Using default.krx fallback.",
                                e
                            );
                            let mut default_path = config_dir;
                            default_path.push("default.krx");
                            Ok(default_path)
                        }
                    }
                }
                Err(e) => {
                    // Failed to initialize ProfileManager - fall back to old behavior
                    eprintln!(
                        "[WARN] Failed to initialize ProfileManager: {}. Using default.krx fallback.",
                        e
                    );
                    let mut default_path = config_dir;
                    default_path.push("default.krx");
                    Ok(default_path)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_config_path_with_explicit_path() {
        let path = PathBuf::from("/tmp/test.krx");
        let result = resolve_config_path(Some(path.clone()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), path);
    }
}
