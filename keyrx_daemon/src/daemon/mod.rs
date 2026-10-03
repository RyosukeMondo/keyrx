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
//! - **SIGHUP**: Configuration reload - re-reads the live config without restarting
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
//! use keyrx_daemon::daemon::{ConfigSource, Daemon};
//! use keyrx_daemon::platform::create_platform;
//!
//! // Start from the active profile in ~/.config/keyrx
//! let config_dir = keyrx_daemon::cli::config_dir::get_config_dir()?;
//! let mut daemon = Daemon::new(create_platform()?, ConfigSource::ActiveProfile, config_dir)?;
//!
//! // Run the event loop (blocks until shutdown signal)
//! daemon.run()?;
//!
//! // Shutdown is automatic via Drop trait
//! # Ok::<(), keyrx_daemon::daemon::DaemonError>(())
//! ```

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use keyrx_core::config::DeviceConfig;
use log::{error, info, warn};

use crate::error::ConfigError;
use crate::platform::held_outputs::HeldOutputs;
use crate::platform::min_key_down::MinKeyDown;
use crate::platform::{Platform, PlatformError};
use options::{OptionOverrides, RuntimeOptions};

// Submodules
pub mod config_watch;
pub mod event_broadcaster;
pub mod event_loop;
pub mod live_config;
pub mod metrics;
pub mod options;
pub mod overflow_log;
pub mod platform_runners;
pub mod platform_setup;
pub mod remapping_state;
pub mod shared_state;
pub mod signals;
pub mod state;
pub mod telemetry;

// Re-exports for public API
pub use event_broadcaster::{start_latency_broadcast_task, EventBroadcaster};
pub use event_loop::process_one_event;
pub use live_config::{ConfigSource, LiveConfig, LoadedConfig};
pub use metrics::{LatencyRecorder, LatencySnapshot, MetricsAggregator};
pub use remapping_state::RemappingState;
pub use shared_state::DaemonSharedState;
pub use signals::{install_signal_handlers, SignalHandler};
pub use state::ReloadState;
pub use telemetry::{DaemonTelemetry, TelemetryState};

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

impl DaemonError {
    /// The process exit code this error maps to (single rule for every runner).
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Config(_) => ExitCode::ConfigError,
            Self::PermissionError(_) => ExitCode::PermissionError,
            Self::Platform(e) if e.is_permission_denied() => ExitCode::PermissionError,
            Self::Platform(_) | Self::SignalError(_) | Self::RuntimeError(_) => {
                ExitCode::RuntimeError
            }
        }
    }
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
/// Owns the platform (input capture + output injection), the live remapping
/// state, and the single sources of truth shared with the web/IPC threads:
/// [`DaemonSharedState`] (status, reload flag, activation requests) and
/// [`DaemonTelemetry`] (live state, latency, recent events).
///
/// Which configuration is live is decided in one place, [`LiveConfig`]; see
/// [`live_config`] for what `run --config` means and how activation works.
pub struct Daemon {
    /// Which configuration is loaded, and how reloads resolve.
    live: LiveConfig,

    /// Platform abstraction for input/output operations.
    platform: Box<dyn Platform>,

    /// Running flag for event loop control.
    running: Arc<AtomicBool>,

    /// Signal handler; its reload flag is the one `DaemonSharedState` raises.
    signal_handler: SignalHandler,

    /// Event broadcaster for WebSocket real-time updates (optional).
    event_broadcaster: Option<EventBroadcaster>,

    /// Lock-free latency recorder (owned by `telemetry`, cached for the hot path).
    latency_recorder: Arc<LatencyRecorder>,

    /// Live remapping state; `None` when no config is live (no keyboard is grabbed). Owned by the event
    /// loop while it runs — reloads hand a replacement back to the loop.
    remapping_state: Option<RemappingState>,

    /// Live telemetry (state/latency/recent events) shared with IPC/web consumers.
    telemetry: Arc<DaemonTelemetry>,

    /// Status + control shared with the web server and IPC server.
    shared_state: Arc<DaemonSharedState>,
}

impl Daemon {
    /// Creates a daemon that starts from `source`.
    ///
    /// The configuration is read before any device is grabbed, so a broken
    /// `--config` file fails fast. A missing or broken *active profile* is not
    /// fatal: the daemon starts with **no config live**, which grabs no
    /// keyboard (a broken profile must never capture devices the user did not
    /// scope), keeps serving IPC/web so the profile can be fixed, and reports
    /// the reason as `config_error` in status.
    ///
    /// # Errors
    ///
    /// - `DaemonError::Config` / `RuntimeError`: an explicit `--config` file
    ///   cannot be loaded
    /// - `DaemonError::Platform`: platform initialization failed
    /// - `DaemonError::SignalError`: signal handlers could not be installed
    pub fn new(
        platform: Box<dyn Platform>,
        source: ConfigSource,
        config_dir: PathBuf,
    ) -> Result<Self, DaemonError> {
        let options = RuntimeOptions::from_environment(&config_dir, &OptionOverrides::default())
            .map_err(DaemonError::RuntimeError)?;
        Self::with_options(platform, source, config_dir, &options)
    }

    /// Like [`Self::new`] with the runtime tuning (minimum key-down time)
    /// already resolved, e.g. including command-line overrides.
    ///
    /// # Errors
    ///
    /// Same as [`Self::new`].
    pub fn with_options(
        platform: Box<dyn Platform>,
        source: ConfigSource,
        config_dir: PathBuf,
        options: &RuntimeOptions,
    ) -> Result<Self, DaemonError> {
        // Track held output keys so a config swap can release them.
        let mut platform: Box<dyn Platform> = Box::new(HeldOutputs::new(platform));
        if !options.min_key_down.is_zero() {
            platform = Box::new(MinKeyDown::new(platform, options.min_key_down));
        }
        info!("Initializing keyrx daemon from {source:?}");
        let mut live = LiveConfig::new(config_dir);
        let (loaded, config_error) = Self::load_startup_config(&live, &source)?;

        platform.initialize()?;
        let running = Arc::new(AtomicBool::new(true));
        let signal_handler = install_signal_handlers(Arc::clone(&running))?;

        // Telemetry owns the latency recorder: one recorder, one aggregator.
        let telemetry = Arc::new(DaemonTelemetry::new());
        let latency_recorder = telemetry.latency_recorder();

        let shared_state = Arc::new(
            DaemonSharedState::new(Arc::clone(&running), None, PathBuf::new(), 0)
                .sharing_reload_flag(signal_handler.reload_state().flag()),
        );
        // Grabs only devices the live config's `device_start` patterns
        // actually match (no config grabs nothing) and publishes the count.
        let remapping_state = apply_loaded(&mut platform, &mut live, &shared_state, loaded);
        shared_state.set_config_error(config_error);

        info!("Daemon initialization complete");
        Ok(Self {
            live,
            platform,
            running,
            signal_handler,
            event_broadcaster: None,
            latency_recorder,
            remapping_state,
            telemetry,
            shared_state,
        })
    }

    /// Reads the startup config. A broken/unreadable *active profile* yields
    /// `(None, Some(reason))` - no config live, reason surfaced in status.
    fn load_startup_config(
        live: &LiveConfig,
        source: &ConfigSource,
    ) -> Result<(Option<LoadedConfig>, Option<String>), DaemonError> {
        match (source, live.load(source)) {
            (_, Ok(None)) => {
                info!("No active profile; no config is live and no keyboard is grabbed");
                Ok((None, None))
            }
            (_, Ok(loaded)) => Ok((loaded, None)),
            (ConfigSource::ActiveProfile, Err(e)) => {
                let reason = format!("failed to load the active profile: {e}");
                error!("{reason}; no keyboard is grabbed until the profile is fixed or another is activated");
                Ok((None, Some(reason)))
            }
            (_, Err(e)) => Err(e),
        }
    }

    /// Sets the event broadcaster for real-time WebSocket updates.
    pub fn set_event_broadcaster(&mut self, broadcaster: EventBroadcaster) {
        self.event_broadcaster = Some(broadcaster);
    }

    /// Returns the number of managed devices.
    #[must_use]
    pub fn device_count(&self) -> usize {
        self.shared_state.get_device_count()
    }

    /// Returns whether the daemon is still running.
    ///
    /// This is set to `false` when a shutdown signal (SIGTERM, SIGINT) is received.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// The loaded configuration file, if any (`None`: no config is live).
    #[must_use]
    pub fn config_path(&self) -> Option<&Path> {
        self.live.loaded().map(|l| l.path.as_path())
    }

    /// The keyrx config directory the daemon resolves profiles in.
    #[must_use]
    pub fn config_dir(&self) -> &Path {
        self.live.config_dir()
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
    #[must_use]
    pub fn latency_recorder(&self) -> Arc<LatencyRecorder> {
        Arc::clone(&self.latency_recorder)
    }

    /// Returns a clone of the shared telemetry Arc.
    #[must_use]
    pub fn telemetry(&self) -> Arc<DaemonTelemetry> {
        Arc::clone(&self.telemetry)
    }

    /// The status/control state shared with the web and IPC servers.
    #[must_use]
    pub fn shared_state(&self) -> Arc<DaemonSharedState> {
        Arc::clone(&self.shared_state)
    }

    /// Reloads now: switches to a pending activation if one was requested,
    /// otherwise re-reads the loaded source. On error the current mappings stay.
    pub fn reload(&mut self) -> Result<(), DaemonError> {
        let new_state = reload_remapping(&mut self.platform, &mut self.live, &self.shared_state)?;
        release_held_outputs(&mut self.platform);
        self.remapping_state = new_state;
        self.telemetry.update_state(TelemetryState::empty());
        Ok(())
    }

    /// Services a pending reload request (SIGHUP, activation, config saved).
    ///
    /// For message-pump platforms (Windows); the Linux event loop does the
    /// same inside [`event_loop::run_event_loop`]. Returns whether one was pending.
    pub fn service_reload_request(&mut self) -> bool {
        if !self.signal_handler.check_reload() {
            return false;
        }
        if let Err(e) = self.reload() {
            warn!("Configuration reload failed, keeping current mappings: {e}");
        }
        true
    }

    /// Process a single event from the platform (non-blocking).
    ///
    /// For Windows, where events are pumped from the message loop.
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
            Some(&self.telemetry),
        )
    }

    /// Check and process tap-hold timeouts.
    ///
    /// Must be called periodically (~10ms) on Windows where the event loop
    /// doesn't have a built-in timeout check like the Linux event loop does.
    /// Without this, tap-hold keys only resolve via permissive hold (another
    /// key press), never via timeout — breaking hold-to-activate-layer behavior.
    pub fn check_tap_hold_timeouts(&mut self) {
        if let Some(ref mut remap_state) = self.remapping_state {
            let timeout_events = remap_state.tick(event_loop::current_timestamp_us());
            for output_event in &timeout_events {
                if let Err(e) = self.platform.inject_output(output_event.clone()) {
                    log::warn!("Failed to inject timeout event: {}", e);
                }
            }
        }
    }

    /// Runs the event loop until a shutdown signal (Linux).
    ///
    /// Reload requests are serviced inside the loop: the new remapping state
    /// replaces the old one before the next event is processed.
    pub fn run(&mut self) -> Result<(), DaemonError> {
        let live = &mut self.live;
        let shared_state = &self.shared_state;
        event_loop::run_event_loop(
            &mut self.platform,
            Arc::clone(&self.running),
            &self.signal_handler,
            |platform| reload_remapping(platform, live, shared_state),
            self.event_broadcaster.as_ref(),
            &mut self.remapping_state,
            Some(&self.latency_recorder),
            Some(&self.telemetry),
            Some(shared_state.as_ref()),
        )
    }

    /// Performs graceful shutdown of the daemon.
    ///
    /// Shuts down the platform (releases grabbed devices, destroys the virtual
    /// output). Errors are logged; cleanup continues. Called by `Drop`.
    pub fn shutdown(&mut self) {
        info!("Initiating graceful shutdown...");
        release_held_outputs(&mut self.platform);
        match self.platform.shutdown() {
            Ok(()) => info!("Platform shutdown successfully"),
            Err(e) => warn!("Failed to shutdown platform: {}", e),
        }
        self.running.store(false, Ordering::SeqCst);
        info!("Shutdown complete");
    }
}

/// Releases output keys still held under the outgoing config (see
/// [`HeldOutputs`]). Failure is logged: a stuck key is bad, but not a reason
/// to keep the old config.
pub(crate) fn release_held_outputs(platform: &mut Box<dyn Platform>) {
    match platform.release_held_outputs() {
        Ok(0) => {}
        Ok(n) => info!("Released {n} held output key(s) before the config swap"),
        Err(e) => warn!("Failed to release held output keys: {e}"),
    }
}

/// Publishes `platform`'s currently captured devices (grabbed on Linux; the
/// full `list_devices()` set on platforms that don't grab per device) to
/// `shared_state`: the ONE source `DeviceService::list_devices` reads for
/// device count and per-device "active" status (H8). Called after startup,
/// every reload/profile activation ([`apply_loaded`]) and, on Linux, every
/// hotplug rescan (see [`event_loop::run_event_loop`]) - so a device never
/// shows as active a moment longer than the daemon actually has it.
pub(crate) fn publish_device_state(shared_state: &DaemonSharedState, platform: &dyn Platform) {
    let devices = platform.list_devices().unwrap_or_default();
    shared_state.set_device_count(devices.len());
    shared_state.set_output_device(platform.output_device());
    shared_state.set_active_devices(devices.into_iter().map(|d| d.id));
}

/// Resolves and loads what a reload should switch to (a pending activation,
/// else the loaded source) and makes it live. Errors leave everything as is.
fn reload_remapping(
    platform: &mut Box<dyn Platform>,
    live: &mut LiveConfig,
    shared_state: &DaemonSharedState,
) -> Result<Option<RemappingState>, DaemonError> {
    let source = live.reload_source(shared_state.take_pending_activation());
    info!("Reloading configuration from {source:?}");
    let result = live.load(&source).map(|loaded| {
        let state = apply_loaded(platform, live, shared_state, loaded);
        shared_state.set_config_error(None);
        state
    });
    if let Err(e) = &result {
        shared_state.report_config_error(format!("failed to load {source:?}: {e}"));
    }
    // Success or not, the request has been dealt with: waiters (activate)
    // re-read the published profile to tell which.
    shared_state.mark_reload_serviced();
    result
}

/// Makes `loaded` the live configuration: builds its remapping state,
/// configures platform key blocking/device capture, and publishes it to
/// status.
fn apply_loaded(
    platform: &mut Box<dyn Platform>,
    live: &mut LiveConfig,
    shared_state: &DaemonSharedState,
    loaded: Option<LoadedConfig>,
) -> Option<RemappingState> {
    let devices = loaded.as_ref().map(|l| l.devices.as_slice());
    configure_platform_blocking(devices);
    // Re-evaluate which devices are grabbed (Linux) against the new
    // patterns; a no-op on platforms that don't grab per device.
    if let Err(e) = platform.reconfigure_devices(devices.unwrap_or(&[])) {
        warn!("Failed to reconfigure device capture: {e}");
    }
    publish_device_state(shared_state, platform.as_ref());
    let remapping_state = devices.map(RemappingState::from_blocks);
    shared_state.set_active_config(
        loaded.as_ref().and_then(|l| l.profile.clone()),
        loaded.as_ref().map(|l| l.path.clone()).unwrap_or_default(),
    );
    match &loaded {
        Some(l) => info!(
            "Live config: {} ({} mappings in {} block(s), profile: {})",
            l.path.display(),
            l.devices.iter().map(|d| d.mappings.len()).sum::<usize>(),
            l.devices.len(),
            l.profile.as_deref().unwrap_or("-")
        ),
        None => info!("Live config: none (no keyboard is grabbed)"),
    }
    live.set_loaded(loaded);
    remapping_state
}

/// Windows: the low-level hook must block exactly the remapped source keys
/// (of every block - the hook does not know which device a key came from),
/// otherwise the original keystroke leaks through (double input). Linux
/// instead re-evaluates which devices are grabbed (see the
/// `reconfigure_devices` call in [`apply_loaded`]).
#[cfg(target_os = "windows")]
fn configure_platform_blocking(devices: Option<&[DeviceConfig]>) {
    use crate::platform::windows::platform_state::PlatformState;
    use keyrx_core::config::{ConfigRoot, Metadata, Version};

    let config_root = devices.map(|devices| ConfigRoot {
        version: Version::current(),
        devices: devices.to_vec(),
        metadata: Metadata {
            compilation_timestamp: 0,
            compiler_version: String::new(),
            source_hash: String::new(),
        },
    });
    if let Err(e) = PlatformState::configure_blocking(config_root.as_ref()) {
        warn!("Failed to configure key blocking: {}", e);
    }
}

#[cfg(not(target_os = "windows"))]
fn configure_platform_blocking(_devices: Option<&[DeviceConfig]>) {}

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
    fn platform_errors_map_to_exit_codes_by_variant_not_text() {
        use crate::platform::PlatformError;
        let code = |e: PlatformError| DaemonError::Platform(e).exit_code();
        assert_eq!(
            code(PlatformError::PermissionDenied("x".into())),
            ExitCode::PermissionError
        );
        assert_eq!(
            code(PlatformError::Io(io::Error::from(
                io::ErrorKind::PermissionDenied
            ))),
            ExitCode::PermissionError
        );
        // Text mentioning "permission" must not be misclassified.
        assert_eq!(
            code(PlatformError::InitializationFailed {
                reason: "permission-less setup failed".into()
            }),
            ExitCode::RuntimeError
        );
        assert_eq!(
            code(PlatformError::DeviceNotFound("k".into())),
            ExitCode::RuntimeError
        );
        assert_eq!(
            DaemonError::PermissionError("p".into()).exit_code(),
            ExitCode::PermissionError
        );
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
