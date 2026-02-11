//! Daemon lifecycle management for keyrx.
//!
//! This module provides the core daemon functionality including:
//!
//! - [`Daemon`]: Main daemon struct coordinating all components
//! - Signal handling for graceful shutdown and reload
//! - Event loop processing
//! - State management
//!
//! # Signal Handling
//!
//! The daemon responds to the following signals:
//!
//! - **SIGTERM**: Graceful shutdown - stops event processing and releases all resources
//! - **SIGINT**: Same as SIGTERM (Ctrl+C handling)
//! - **SIGHUP**: Configuration reload - reloads .krx file without restarting
//!
//! # Daemon Lifecycle
//!
//! 1. **Initialization**: Load configuration, discover devices, create uinput output
//! 2. **Signal Setup**: Install handlers for SIGTERM, SIGINT, SIGHUP
//! 3. **Event Loop**: Process keyboard events from all managed devices
//! 4. **Shutdown**: Release devices, destroy virtual output, exit cleanly
//!
//! # Example
//!
//! ```ignore
//! use std::path::Path;
//! use keyrx_daemon::daemon::Daemon;
//!
//! // Initialize daemon with configuration
//! let mut daemon = Daemon::new(Path::new("config.krx"))?;
//!
//! // Run the event loop (blocks until shutdown signal)
//! daemon.run()?;
//!
//! // Shutdown is automatic via Drop trait
//! # Ok::<(), keyrx_daemon::daemon::DaemonError>(())
//! ```

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use keyrx_core::config::DeviceConfig;
use log::{info, warn};

use crate::config_loader::load_config;
use crate::error::ConfigError;
use crate::platform::{Platform, PlatformError};

use state::convert_archived_device_config;

// Submodules
pub mod event_broadcaster;
pub mod event_loop;
pub mod factory;
pub mod metrics;
pub mod platform_runners;
pub mod platform_setup;
pub mod remapping_state;
pub mod shared_state;
pub mod signals;
pub mod state;

// Re-exports for public API
pub use event_broadcaster::{start_latency_broadcast_task, EventBroadcaster};
pub use event_loop::process_one_event;
pub use metrics::{LatencyRecorder, LatencySnapshot, MetricsAggregator};
pub use remapping_state::RemappingState;
pub use shared_state::DaemonSharedState;
pub use signals::{install_signal_handlers, SignalHandler};
pub use state::ReloadState;

/// Returns the current time in microseconds since UNIX epoch.
///
/// This is used for tap-hold timeout checking.
#[allow(dead_code)]
fn current_time_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

/// Errors that can occur during daemon operations.
#[derive(Debug, Error)]
pub enum DaemonError {
    /// Failed to install signal handlers.
    #[error("failed to install signal handlers: {0}")]
    SignalError(#[from] io::Error),

    /// Configuration loading error (file not found, parse error).
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Platform error.
    #[error("platform error: {0}")]
    Platform(#[from] PlatformError),

    /// Permission error (cannot grab device, cannot create uinput).
    #[error("permission error: {0}")]
    PermissionError(String),

    /// Runtime error during event processing.
    #[error("runtime error: {0}")]
    RuntimeError(String),
}

/// Exit codes for daemon termination.
///
/// These codes follow Unix conventions and are documented in the requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCode {
    /// Successful termination.
    Success = 0,
    /// Configuration error (file not found, parse error).
    ConfigError = 1,
    /// Permission error (cannot grab device, cannot create uinput).
    PermissionError = 2,
    /// Runtime error (device disconnected with no fallback).
    RuntimeError = 3,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code as i32
    }
}

/// The main keyrx daemon.
///
/// `Daemon` coordinates all components for keyboard event processing:
///
/// - **Configuration**: Loads and manages the .krx configuration file
/// - **Device Manager**: Discovers and manages input keyboard devices
/// - **Output Device**: Creates virtual keyboard for injecting remapped events
/// - **Signal Handling**: Responds to SIGTERM, SIGINT (shutdown), SIGHUP (reload)
///
/// # Initialization Order
///
/// The daemon initializes components in this order:
///
/// 1. Load configuration from .krx file
/// 2. Discover and match input devices
/// 3. Create uinput virtual keyboard
/// 4. Install signal handlers
///
/// This order ensures we fail fast on configuration errors before grabbing devices.
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
/// use keyrx_daemon::daemon::Daemon;
///
/// // Initialize daemon
/// let mut daemon = Daemon::new(Path::new("config.krx"))?;
///
/// // Check device count
/// println!("Managing {} devices", daemon.device_count());
///
/// // Run event loop (blocks until shutdown)
/// daemon.run()?;
/// # Ok::<(), keyrx_daemon::daemon::DaemonError>(())
/// ```
pub struct Daemon {
    /// Path to the configuration file (for reload support).
    config_path: PathBuf,

    /// Path to the keyrx config directory (~/.config/keyrx).
    config_dir: PathBuf,

    /// Platform abstraction for input/output operations.
    platform: Box<dyn Platform>,

    /// Running flag for event loop control.
    running: Arc<AtomicBool>,

    /// Signal handler for reload detection.
    signal_handler: SignalHandler,

    /// Event broadcaster for WebSocket real-time updates (optional).
    event_broadcaster: Option<EventBroadcaster>,

    /// Lock-free latency recorder for metrics collection.
    ///
    /// This is shared between the event loop (writing samples) and
    /// the broadcast task (reading statistics). Lock-free design
    /// ensures no mutex contention on the hot path.
    latency_recorder: Arc<LatencyRecorder>,

    /// Remapping state for key remapping (KeyLookup + DeviceState).
    ///
    /// This is `Some` when a profile is active and remapping is enabled.
    /// It is `None` in pass-through mode (no active profile).
    remapping_state: Option<RemappingState>,
}

impl Daemon {
    /// Creates a new daemon instance with the specified platform and configuration file.
    ///
    /// This method performs the initialization sequence:
    ///
    /// 1. Accepts a platform implementation via dependency injection
    /// 2. Initializes the platform
    /// 3. Installs signal handlers for graceful shutdown and reload
    ///
    /// # Arguments
    ///
    /// * `platform` - Platform implementation for input/output operations
    /// * `config_path` - Path to the .krx configuration file (for reload support)
    ///
    /// # Returns
    ///
    /// * `Ok(Daemon)` - Successfully initialized daemon
    /// * `Err(DaemonError::Platform)` - Platform initialization failed
    /// * `Err(DaemonError::SignalError)` - Failed to install signal handlers
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::daemon::Daemon;
    /// use keyrx_daemon::platform::create_platform;
    ///
    /// let platform = create_platform()?;
    /// match Daemon::new(platform, Path::new("config.krx")) {
    ///     Ok(daemon) => {
    ///         println!("Daemon initialized successfully");
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Failed to initialize daemon: {}", e);
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(mut platform: Box<dyn Platform>, config_path: &Path) -> Result<Self, DaemonError> {
        info!(
            "Initializing keyrx daemon with config: {}",
            config_path.display()
        );

        // Determine config directory (~/.config/keyrx)
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("keyrx");

        // Step 1: Initialize the platform
        info!("Initializing platform...");
        platform.initialize()?;
        info!("Platform initialized");

        // Step 2: Install signal handlers
        info!("Installing signal handlers...");
        let running = Arc::new(AtomicBool::new(true));
        let signal_handler = install_signal_handlers(Arc::clone(&running))?;
        info!("Signal handlers installed");

        // Create lock-free latency recorder for metrics collection
        let latency_recorder = Arc::new(LatencyRecorder::new());

        // Step 3: Load active profile and create remapping state (if any)
        let remapping_state = match Self::load_active_profile_config(&config_dir) {
            Ok(Some(device_config)) => {
                info!("Loaded active profile, creating remapping state");
                Some(RemappingState::new(&device_config))
            }
            Ok(None) => {
                info!("No active profile found, running in pass-through mode");
                None
            }
            Err(e) => {
                warn!(
                    "Failed to load active profile: {}. Running in pass-through mode",
                    e
                );
                None
            }
        };

        info!("Daemon initialization complete");

        Ok(Self {
            config_path: config_path.to_path_buf(),
            config_dir,
            platform,
            running,
            signal_handler,
            event_broadcaster: None,
            latency_recorder,
            remapping_state,
        })
    }

    /// Sets the event broadcaster for real-time WebSocket updates.
    ///
    /// This method allows injecting an EventBroadcaster after daemon creation.
    /// The broadcaster will receive key events and state updates during event processing.
    pub fn set_event_broadcaster(&mut self, broadcaster: EventBroadcaster) {
        self.event_broadcaster = Some(broadcaster);
    }

    /// Loads the active profile's DeviceConfig from the .krx file.
    ///
    /// Returns `Ok(Some(config))` if an active profile exists and was loaded successfully,
    /// `Ok(None)` if no active profile is set, or `Err` on load failure.
    fn load_active_profile_config(config_dir: &Path) -> Result<Option<DeviceConfig>, DaemonError> {
        // Read the .active file to get the active profile name
        let active_file = config_dir.join(".active");
        if !active_file.exists() {
            return Ok(None);
        }

        let active_name = fs::read_to_string(&active_file)
            .map_err(|e| DaemonError::RuntimeError(format!("Failed to read .active file: {}", e)))?
            .trim()
            .to_string();

        if active_name.is_empty() {
            return Ok(None);
        }

        // Construct path to the .krx file
        let krx_path = config_dir
            .join("profiles")
            .join(format!("{}.krx", active_name));
        if !krx_path.exists() {
            warn!(
                "Active profile '{}' has no compiled .krx file at {}",
                active_name,
                krx_path.display()
            );
            return Ok(None);
        }

        // Load the .krx file
        info!(
            "Loading active profile '{}' from {}",
            active_name,
            krx_path.display()
        );
        let archived_config = load_config(&krx_path)?;

        // Get the first device config (global config)
        // Most profiles use a single wildcard pattern "*" for global remapping
        if archived_config.devices.is_empty() {
            warn!("Profile '{}' has no device configurations", active_name);
            return Ok(None);
        }

        // Convert the first (global) device config to owned type
        let device_config = convert_archived_device_config(&archived_config.devices[0]);
        info!(
            "Loaded {} key mappings from profile '{}'",
            device_config.mappings.len(),
            active_name
        );

        Ok(Some(device_config))
    }

    /// Returns the number of managed devices.
    #[must_use]
    pub fn device_count(&self) -> usize {
        // Platform trait doesn't expose device count directly
        // Return the number of devices from list_devices()
        self.platform
            .list_devices()
            .map(|devices| devices.len())
            .unwrap_or(0)
    }

    /// Returns whether the daemon is still running.
    ///
    /// This is set to `false` when a shutdown signal (SIGTERM, SIGINT) is received.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Returns the path to the configuration file.
    #[must_use]
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Returns a reference to the signal handler.
    #[must_use]
    pub fn signal_handler(&self) -> &SignalHandler {
        &self.signal_handler
    }

    /// Returns the running flag for external coordination.
    #[must_use]
    pub fn running_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.running)
    }

    /// Returns a clone of the latency recorder Arc.
    ///
    /// This is used to share the recorder with the latency broadcast task.
    /// The recorder is lock-free and safe for concurrent access.
    #[must_use]
    pub fn latency_recorder(&self) -> Arc<LatencyRecorder> {
        Arc::clone(&self.latency_recorder)
    }

    /// Reloads the configuration from disk.
    ///
    /// This method reads the active profile from the `.active` file and
    /// rebuilds the remapping state. Called when SIGHUP is received or
    /// when profile activation triggers a reload.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::daemon::Daemon;
    /// use keyrx_daemon::platform::create_platform;
    ///
    /// let platform = create_platform()?;
    /// let mut daemon = Daemon::new(platform, Path::new("config.krx"))?;
    ///
    /// match daemon.reload() {
    ///     Ok(()) => println!("Configuration reloaded successfully"),
    ///     Err(e) => eprintln!("Reload failed: {}", e),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn reload(&mut self) -> Result<(), DaemonError> {
        info!("Reloading configuration from active profile...");

        match Self::load_active_profile_config(&self.config_dir) {
            Ok(Some(device_config)) => {
                let mapping_count = device_config.mappings.len();
                if let Some(ref mut state) = self.remapping_state {
                    // Update existing state
                    state.reload(&device_config);
                    info!("Remapping state reloaded with {} mappings", mapping_count);
                } else {
                    // Create new state
                    self.remapping_state = Some(RemappingState::new(&device_config));
                    info!(
                        "Created new remapping state with {} mappings",
                        mapping_count
                    );
                }
                Ok(())
            }
            Ok(None) => {
                info!("No active profile found, switching to pass-through mode");
                self.remapping_state = None;
                Ok(())
            }
            Err(e) => {
                warn!("Failed to reload configuration: {}", e);
                Err(e)
            }
        }
    }

    /// Runs the main event processing loop.
    ///
    /// This method captures keyboard events from the platform, processes them,
    /// and injects output events. The loop continues until a shutdown signal
    /// (SIGTERM or SIGINT) is received.
    ///
    /// # Event Processing Flow
    ///
    /// For each input event:
    /// 1. Capture event from platform (blocking)
    /// 2. Inject the event back through the platform
    ///
    /// **Note**: The current implementation is simplified and does not perform
    /// key remapping. Full remapping support would require the Platform trait
    /// to expose device state and lookup tables, or for the Daemon to manage
    /// remapping state independently.
    ///
    /// # Signal Handling
    ///
    /// - **SIGTERM/SIGINT**: Sets the running flag to false, causing graceful exit
    /// - **SIGHUP**: Triggers configuration reload (logs but requires restart for full effect)
    ///
    /// # Errors
    ///
    /// - `DaemonError::Platform`: Platform error during event capture or injection
    /// - `DaemonError::RuntimeError`: Critical error during event processing
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::daemon::Daemon;
    /// use keyrx_daemon::platform::create_platform;
    ///
    /// let platform = create_platform()?;
    /// let mut daemon = Daemon::new(platform, Path::new("config.krx"))?;
    ///
    /// // This blocks until shutdown signal received
    /// daemon.run()?;
    ///
    /// println!("Daemon stopped gracefully");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    /// Process a single event from the platform (non-blocking).
    ///
    /// This method is designed for Windows where the event loop must be
    /// integrated with the system message pump. Call this repeatedly in
    /// the Windows message loop to process keyboard events.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - An event was processed
    /// * `Ok(false)` - No event was available
    /// * `Err(...)` - A fatal error occurred
    pub fn process_one_event(&mut self) -> Result<bool, DaemonError> {
        event_loop::process_one_event(
            &mut self.platform,
            self.event_broadcaster.as_ref(),
            self.remapping_state.as_mut(),
            Some(&self.latency_recorder),
        )
    }

    pub fn run(&mut self) -> Result<(), DaemonError> {
        // Clone config_dir for the reload closure (avoids borrowing self)
        let config_dir = self.config_dir.clone();

        // Create reload callback that reloads from active profile
        // Note: Due to borrow constraints, this callback cannot directly update
        // self.remapping_state. For now, it loads the new config and logs.
        // Full hot-reload would require Arc<RwLock> for the remapping state.
        let reload_fn = move || -> Result<(), DaemonError> {
            info!("Reload signal received, reloading configuration...");
            match Daemon::load_active_profile_config(&config_dir) {
                Ok(Some(device_config)) => {
                    info!(
                        "Loaded active profile with {} mappings. Note: Full hot-reload requires daemon restart.",
                        device_config.mappings.len()
                    );
                    Ok(())
                }
                Ok(None) => {
                    info!("No active profile found after reload signal");
                    Ok(())
                }
                Err(e) => {
                    warn!("Failed to reload configuration: {}", e);
                    Err(e)
                }
            }
        };

        event_loop::run_event_loop(
            &mut self.platform,
            Arc::clone(&self.running),
            &self.signal_handler,
            reload_fn,
            self.event_broadcaster.as_ref(),
            self.remapping_state.as_mut(),
            Some(&self.latency_recorder),
        )
    }

    /// Performs graceful shutdown of the daemon.
    ///
    /// This method shuts down the platform and releases all resources.
    ///
    /// # Error Handling
    ///
    /// Errors during shutdown are logged but do not prevent continued cleanup.
    /// This ensures that all resources are released even if some operations fail.
    ///
    /// # Automatic Cleanup
    ///
    /// This method is called automatically by the `Drop` implementation, so
    /// cleanup occurs even on panic or unexpected termination.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::daemon::Daemon;
    /// use keyrx_daemon::platform::create_platform;
    ///
    /// let platform = create_platform()?;
    /// let mut daemon = Daemon::new(platform, Path::new("config.krx"))?;
    ///
    /// // Run the event loop
    /// daemon.run()?;
    ///
    /// // Explicit shutdown (optional - Drop will call this automatically)
    /// daemon.shutdown();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn shutdown(&mut self) {
        info!("Initiating graceful shutdown...");

        // Shutdown the platform
        info!("Shutting down platform...");
        match self.platform.shutdown() {
            Ok(()) => {
                info!("Platform shutdown successfully");
            }
            Err(e) => {
                // Log warning but continue
                warn!("Failed to shutdown platform: {}", e);
            }
        }

        // Mark daemon as stopped
        self.running.store(false, Ordering::SeqCst);

        info!("Shutdown complete");
    }
}

/// Drop implementation to ensure automatic cleanup on daemon exit.
///
/// When a `Daemon` is dropped (goes out of scope, program exits, or panic occurs),
/// this implementation ensures that:
///
/// 1. All grabbed input devices are released (restores normal keyboard input)
/// 2. The virtual keyboard is destroyed (removes from `/dev/input/`)
///
/// This prevents:
/// - Orphaned device grabs that would block keyboard input
/// - Orphaned virtual devices in `/dev/input/`
/// - Stuck keys in applications
///
/// # Note
///
/// The `shutdown()` method is called automatically. If `shutdown()` was already
/// called manually, it will safely handle the already-released/destroyed state.
impl Drop for Daemon {
    fn drop(&mut self) {
        // Call shutdown to release all resources
        // shutdown() handles already-released devices gracefully
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Requires Platform refactoring"]
    fn test_daemon_error_display() {
        let err = DaemonError::PermissionError("access denied".to_string());
        assert_eq!(err.to_string(), "permission error: access denied");

        let err = DaemonError::RuntimeError("event loop failed".to_string());
        assert_eq!(err.to_string(), "runtime error: event loop failed");
    }

    #[test]
    #[ignore = "Requires Platform refactoring"]
    fn test_daemon_error_config_variant() {
        use crate::error::ConfigError;
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let config_err = ConfigError::Io(io_err);
        let daemon_err = DaemonError::Config(config_err);
        assert!(daemon_err.to_string().contains("configuration error"));
    }

    #[test]
    #[ignore = "Requires Platform refactoring"]
    fn test_daemon_error_platform_variant() {
        use crate::platform::PlatformError;
        let platform_err = PlatformError::DeviceNotFound("test device".to_string());
        let daemon_err = DaemonError::Platform(platform_err);
        assert!(daemon_err.to_string().contains("platform error"));
    }

    #[test]
    fn test_exit_code_values() {
        assert_eq!(ExitCode::Success as u8, 0);
        assert_eq!(ExitCode::ConfigError as u8, 1);
        assert_eq!(ExitCode::PermissionError as u8, 2);
        assert_eq!(ExitCode::RuntimeError as u8, 3);
    }

    #[test]
    fn test_exit_code_to_i32() {
        assert_eq!(i32::from(ExitCode::Success), 0);
        assert_eq!(i32::from(ExitCode::ConfigError), 1);
        assert_eq!(i32::from(ExitCode::PermissionError), 2);
        assert_eq!(i32::from(ExitCode::RuntimeError), 3);
    }

    // Daemon tests - these require real devices/permissions
    mod daemon_tests {
        use super::*;

        // Note: These tests are temporarily disabled during Platform trait refactoring
        // TODO: Update tests to use mock Platform implementation

        #[test]
        #[ignore = "Requires Platform refactoring"]
        fn test_daemon_new_missing_config() {
            // Test disabled - needs Platform mock
            // let platform = create_platform().unwrap();
            // let result = Daemon::new(platform, Path::new("/nonexistent/path/config.krx"));
            // assert!(result.is_err());
        }

        #[test]
        #[ignore = "Requires Platform refactoring - needs MockPlatform"]
        fn test_daemon_new_real_devices() {
            // Test disabled - needs Platform mock parameter
            // TODO: Update to create platform and pass to Daemon::new(platform, path)
        }

        #[test]
        #[ignore = "Requires Platform refactoring"]
        fn test_daemon_error_from_discovery_error() {
            // Test disabled - DiscoveryError no longer in DaemonError
        }

        #[test]
        #[ignore = "Requires Platform refactoring"]
        fn test_daemon_error_from_io_error() {
            let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test");
            let daemon_err = DaemonError::SignalError(io_err);
            assert!(daemon_err.to_string().contains("signal handlers"));
        }

        #[test]
        #[ignore = "Requires Platform refactoring - needs MockPlatform"]
        fn test_daemon_reload_success() {
            // Test disabled - needs Platform mock parameter
            // TODO: Update to create platform and pass to Daemon::new(platform, path)
        }
    }
}
