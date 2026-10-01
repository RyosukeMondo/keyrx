//! Daemon run command handler.
//!
//! This module delegates to platform-specific implementations for running the daemon.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::cli::dispatcher::exit_codes;
use crate::container::ServiceContainerBuilder;
use crate::daemon::ConfigSource;

/// Handle the run command - delegates to platform-specific implementation.
///
/// # Arguments
///
/// * `config` - Optional path to configuration file
/// * `log` - Logging verbosity (`--debug`, `--log-keys`)
/// * `watch` - Hot-reload the loaded profile when its source changes
/// * `test_mode` - Enable test mode (no keyboard capture)
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err((exit_code, message))` on failure.
pub fn handle_run(
    config: Option<PathBuf>,
    log: crate::daemon::platform_setup::LogOptions,
    watch: bool,
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

    let config_dir = crate::cli::config_dir::get_config_dir()
        .map_err(|e| (exit_codes::CONFIG_ERROR, format!("Error: {e}")))?;

    // `--config FILE` overrides the active profile at startup; without it the
    // daemon follows the active profile (see `daemon::live_config`).
    let source = match config {
        Some(path) => {
            validate_config_file(&path)?;
            ConfigSource::File(path)
        }
        None => {
            ensure_active_profile(&config_dir);
            ConfigSource::ActiveProfile
        }
    };

    // Build ServiceContainer with all dependencies wired
    let mut builder = ServiceContainerBuilder::new(config_dir.clone());

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
        source, config_dir, log, watch, test_mode, container,
    );

    #[cfg(target_os = "windows")]
    return crate::daemon::platform_runners::windows::run_daemon(
        source, config_dir, log, watch, test_mode, container,
    );

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    Err((
        exit_codes::CONFIG_ERROR,
        "The 'run' command is only available on Linux and Windows. \
         Build with --features linux or --features windows to enable."
            .to_string(),
    ))
}

/// Fails fast on an explicit `--config` that is missing or not a valid `.krx`.
fn validate_config_file(path: &Path) -> Result<(), (i32, String)> {
    let err = |msg: String| (exit_codes::CONFIG_ERROR, msg);
    if !path.is_file() {
        return Err(err(format!(
            "Error: Config file not found: {}",
            path.display()
        )));
    }
    let bytes =
        std::fs::read(path).map_err(|e| err(format!("Error: Cannot read config file: {e}")))?;
    keyrx_compiler::serialize::deserialize(&bytes)
        .map_err(|e| err(format!("Error: Invalid config file: {e}")))?;
    Ok(())
}

/// First run without `--config`: create and activate a blank `default`
/// profile so the UI has something to edit. Failures only mean no config is live.
fn ensure_active_profile(config_dir: &Path) {
    use crate::config::{ProfileManager, ProfileTemplate};

    let manager = match ProfileManager::new(config_dir.to_path_buf()) {
        Ok(manager) => manager,
        Err(e) => {
            log::warn!("Cannot read profiles ({e}); starting with no config live");
            return;
        }
    };
    if let Ok(Some(active)) = manager.get_active() {
        log::info!("Starting from active profile '{active}'");
        return;
    }
    log::info!("No active profile; activating a blank 'default' profile");
    if manager.get("default").is_none() {
        if let Err(e) = manager.create("default", ProfileTemplate::Blank) {
            log::warn!("Failed to create default profile: {e}");
            return;
        }
    }
    if let Err(e) = manager.activate("default") {
        log::warn!("Failed to activate default profile: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_config_must_exist() {
        let err = validate_config_file(Path::new("/nonexistent/test.krx")).unwrap_err();
        assert_eq!(err.0, exit_codes::CONFIG_ERROR);
        assert!(err.1.contains("not found"));
    }

    #[test]
    fn test_explicit_config_must_be_krx() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bogus.krx");
        std::fs::write(&path, b"not a krx").unwrap();
        let err = validate_config_file(&path).unwrap_err();
        assert!(err.1.contains("Invalid config file"));
    }
}
