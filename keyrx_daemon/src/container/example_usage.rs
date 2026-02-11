//! Example usage of ServiceContainer in main.rs
//!
//! This module demonstrates how to refactor main.rs to use ServiceContainer
//! for dependency injection, eliminating the 15+ direct instantiation violations.

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::Arc;

use crate::container::ServiceContainerBuilder;
use crate::web::AppState;

/// Example: Refactored handle_run() function using ServiceContainer
///
/// This shows how main.rs can be simplified from 196+ lines of service
/// instantiation to just 3 lines using the ServiceContainer pattern.
///
/// # Before (SOLID violation)
///
/// ```ignore
/// // Direct instantiation - violates DIP
/// let profile_manager = Arc::new(ProfileManager::new(config_dir.clone())?);
/// let macro_recorder = Arc::new(MacroRecorder::new());
/// let profile_service = Arc::new(ProfileService::new(Arc::clone(&profile_manager)));
/// let device_service = Arc::new(DeviceService::new(config_dir.clone()));
/// let config_service = Arc::new(ConfigService::new(Arc::clone(&profile_manager)));
/// // ... 10+ more lines
/// ```
///
/// # After (SOLID compliant)
///
/// ```ignore
/// let container = ServiceContainerBuilder::new(config_dir)
///     .build()
///     .map_err(|e| (ExitCode::ConfigError, e.to_string()))?;
/// ```
pub fn example_refactored_handle_run(
    config_dir: PathBuf,
    test_mode: bool,
) -> Result<AppState, String> {
    log::info!("Initializing services with ServiceContainer");

    // Build service container with all dependencies wired
    let mut builder = ServiceContainerBuilder::new(config_dir);

    // Configure test mode if needed
    if test_mode {
        let test_socket = PathBuf::from("/tmp/keyrx-test.sock");
        builder = builder.with_test_mode_socket(test_socket);
    }

    // Build container - this replaces 20+ lines of manual instantiation
    let container = builder.build().map_err(|e| e.to_string())?;

    // Create AppState from container
    let app_state = AppState::from_container(container, None);

    log::info!("Services initialized successfully");
    Ok(app_state)
}

/// Example: Simplified Linux test mode initialization
///
/// # Before (196 lines with duplication)
///
/// The original handle_run_test_mode() function had massive duplication
/// between Linux and Windows versions, with identical service instantiation
/// code repeated 3+ times.
///
/// # After (30 lines, no duplication)
pub fn example_refactored_test_mode(config_dir: PathBuf) -> Result<AppState, String> {
    log::info!("Starting test mode with ServiceContainer");

    // Create container with test mode enabled
    let test_socket = PathBuf::from(format!("/tmp/keyrx-test-{}.sock", std::process::id()));

    let container = ServiceContainerBuilder::new(config_dir)
        .with_test_mode_socket(test_socket.clone())
        .with_event_channel_size(1000)
        .build()
        .map_err(|e| e.to_string())?;

    // Create AppState with test mode socket
    let app_state = AppState::from_container(container, Some(test_socket));

    log::info!("Test mode initialized");
    Ok(app_state)
}

/// Example: Production mode initialization (Linux)
///
/// Shows how production mode can use the same ServiceContainer
/// without test-specific configuration.
pub fn example_production_mode(config_dir: PathBuf) -> Result<AppState, String> {
    log::info!("Starting production mode with ServiceContainer");

    // Build production container (no test mode)
    let container = ServiceContainerBuilder::new(config_dir)
        .with_event_channel_size(1000)
        .build()
        .map_err(|e| e.to_string())?;

    let app_state = AppState::from_container(container, None);

    log::info!("Production mode initialized");
    Ok(app_state)
}

/// Example: Windows initialization with port handling
///
/// Shows how Windows-specific logic (port finding, PID file) can coexist
/// with ServiceContainer-based initialization.
pub fn example_windows_mode(
    config_dir: PathBuf,
    _configured_port: u16,
) -> Result<AppState, String> {
    log::info!("Starting Windows mode with ServiceContainer");

    // ServiceContainer handles service wiring, main.rs handles platform-specific logic
    let container = ServiceContainerBuilder::new(config_dir)
        .build()
        .map_err(|e| e.to_string())?;

    let app_state = AppState::from_container(container, None);

    // Platform-specific logic (port finding, PID file) remains in main.rs
    // but service instantiation is centralized in ServiceContainer
    log::info!("Windows mode initialized");
    Ok(app_state)
}

/// Example: CLI command handler using ServiceContainer
///
/// Shows how CLI handlers like handle_profiles_command() can be simplified.
///
/// # Before
///
/// ```ignore
/// let profile_manager = Arc::new(ProfileManager::new(config_dir)?);
/// let service = ProfileService::new(profile_manager);
/// ```
///
/// # After
///
/// ```ignore
/// let container = ServiceContainerBuilder::new(config_dir).build()?;
/// let service = container.profile_service();
/// ```
pub fn example_cli_handler(config_dir: PathBuf) -> Result<(), String> {
    // Build container
    let container = ServiceContainerBuilder::new(config_dir)
        .build()
        .map_err(|e| e.to_string())?;

    // Get only the service we need (no need to instantiate everything)
    let profile_service = container.profile_service();

    // Use service
    log::info!(
        "CLI handler using profile service: {:?}",
        Arc::strong_count(&profile_service)
    );

    Ok(())
}

/// Comparison: Line counts before and after refactoring
///
/// | Component | Before | After | Reduction |
/// |-----------|--------|-------|-----------|
/// | main.rs total | 1,995 lines | ~300 lines | 85% |
/// | Linux handle_run | 223 lines | ~50 lines | 78% |
/// | Windows handle_run | 315 lines | ~50 lines | 84% |
/// | handle_run_test_mode | 196 lines | ~30 lines | 85% |
/// | handle_profiles_command | 38 lines | ~15 lines | 60% |
/// | **Total duplication removed** | **~600 lines** | - | - |
///
/// # Benefits
///
/// 1. **Single Responsibility**: ServiceContainer handles service wiring
/// 2. **DRY**: No duplication between Linux/Windows/test modes
/// 3. **Testability**: Easy to inject mock container
/// 4. **Maintainability**: Add new service in one place
/// 5. **Dependency Inversion**: main.rs depends on ServiceContainer abstraction
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_example_production_mode() {
        let temp_dir = tempdir().unwrap();
        let result = example_production_mode(temp_dir.path().to_path_buf());
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_example_test_mode() {
        let temp_dir = tempdir().unwrap();
        let result = example_refactored_test_mode(temp_dir.path().to_path_buf());
        assert!(result.is_ok());
    }

    #[test]
    fn test_example_cli_handler() {
        let temp_dir = tempdir().unwrap();
        let result = example_cli_handler(temp_dir.path().to_path_buf());
        assert!(result.is_ok());
    }
}
