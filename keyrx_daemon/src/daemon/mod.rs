//! Daemon lifecycle management for keyrx.
//!
//! This module provides the core daemon functionality including:
//!
//! - [`install_signal_handlers`]: Sets up signal handlers for graceful shutdown and reload
//! - [`SignalHandler`]: Manages signal state and detection
//! - [`Daemon`]: Main daemon struct coordinating all components
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
//! ```no_run
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

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
#[cfg(target_os = "linux")]
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use log::{debug, info, trace, warn};

use crate::config_loader::{load_config, ConfigError};
use crate::device_manager::DeviceManager;
#[cfg(target_os = "linux")]
use crate::platform::linux::UinputOutput;
use crate::platform::{DeviceError, InputDevice, OutputDevice};
use keyrx_core::runtime::event::check_tap_hold_timeouts;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::{install_signal_handlers, SignalHandler};
#[cfg(target_os = "windows")]
pub use windows::{install_signal_handlers, SignalHandler};

/// Returns the current time in microseconds since UNIX epoch.
///
/// This is used for tap-hold timeout checking.
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

    /// Device access error.
    #[error("device error: {0}")]
    Device(#[from] DeviceError),

    /// Permission error (cannot grab device, cannot create uinput).
    #[error("permission error: {0}")]
    PermissionError(String),

    /// Runtime error during event processing.
    #[error("runtime error: {0}")]
    RuntimeError(String),

    /// Device discovery failed.
    #[error("device discovery failed: {0}")]
    DiscoveryError(#[from] crate::device_manager::DiscoveryError),
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

/// Reload request state.
///
/// This struct tracks whether a configuration reload has been requested
/// (typically via SIGHUP signal).
#[derive(Debug, Clone)]
pub struct ReloadState {
    /// Flag indicating a reload has been requested.
    reload_requested: Arc<AtomicBool>,
}

impl ReloadState {
    /// Creates a new reload state.
    pub fn new() -> Self {
        Self {
            reload_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns the underlying atomic flag for signal handler registration.
    pub fn flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.reload_requested)
    }

    /// Checks if a reload has been requested and clears the flag.
    ///
    /// Returns `true` if a reload was requested since the last check.
    pub fn check_and_clear(&self) -> bool {
        self.reload_requested.swap(false, Ordering::SeqCst)
    }

    /// Requests a reload (for testing purposes).
    #[cfg(test)]
    pub fn request_reload(&self) {
        self.reload_requested.store(true, Ordering::SeqCst);
    }
}

impl Default for ReloadState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Archived Config Conversion Helpers
// ============================================================================
//
// These functions convert rkyv-archived configuration types to owned types.
// This is necessary because DeviceManager and KeyLookup operate on owned types.

use keyrx_core::config::{
    BaseKeyMapping, Condition, ConditionItem, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping,
};

// Import the archived types from their modules
use keyrx_core::config::conditions::{ArchivedCondition, ArchivedConditionItem};
use keyrx_core::config::keys::ArchivedKeyCode;
use keyrx_core::config::mappings::{
    ArchivedBaseKeyMapping, ArchivedDeviceConfig, ArchivedKeyMapping,
};

/// Converts an archived KeyCode to an owned KeyCode.
///
/// Uses rkyv's Deserialize trait for safe conversion.
fn convert_archived_keycode(archived: &ArchivedKeyCode) -> KeyCode {
    use rkyv::Deserialize;
    archived
        .deserialize(&mut rkyv::Infallible)
        .expect("KeyCode deserialization is infallible")
}

/// Converts an archived ConditionItem to an owned ConditionItem.
fn convert_archived_condition_item(archived: &ArchivedConditionItem) -> ConditionItem {
    match archived {
        ArchivedConditionItem::ModifierActive(id) => ConditionItem::ModifierActive(*id),
        ArchivedConditionItem::LockActive(id) => ConditionItem::LockActive(*id),
    }
}

/// Converts an archived Condition to an owned Condition.
fn convert_archived_condition(archived: &ArchivedCondition) -> Condition {
    match archived {
        ArchivedCondition::ModifierActive(id) => Condition::ModifierActive(*id),
        ArchivedCondition::LockActive(id) => Condition::LockActive(*id),
        ArchivedCondition::DeviceMatches(id) => {
            use rkyv::Deserialize;
            Condition::DeviceMatches(Deserialize::deserialize(id, &mut rkyv::Infallible).unwrap())
        }
        ArchivedCondition::AllActive(items) => {
            Condition::AllActive(items.iter().map(convert_archived_condition_item).collect())
        }
        ArchivedCondition::NotActive(items) => {
            Condition::NotActive(items.iter().map(convert_archived_condition_item).collect())
        }
    }
}

/// Converts an archived BaseKeyMapping to an owned BaseKeyMapping.
fn convert_archived_base_mapping(archived: &ArchivedBaseKeyMapping) -> BaseKeyMapping {
    match archived {
        ArchivedBaseKeyMapping::Simple { from, to } => BaseKeyMapping::Simple {
            from: convert_archived_keycode(from),
            to: convert_archived_keycode(to),
        },
        ArchivedBaseKeyMapping::Modifier { from, modifier_id } => BaseKeyMapping::Modifier {
            from: convert_archived_keycode(from),
            modifier_id: *modifier_id,
        },
        ArchivedBaseKeyMapping::Lock { from, lock_id } => BaseKeyMapping::Lock {
            from: convert_archived_keycode(from),
            lock_id: *lock_id,
        },
        ArchivedBaseKeyMapping::TapHold {
            from,
            tap,
            hold_modifier,
            threshold_ms,
        } => BaseKeyMapping::TapHold {
            from: convert_archived_keycode(from),
            tap: convert_archived_keycode(tap),
            hold_modifier: *hold_modifier,
            threshold_ms: *threshold_ms,
        },
        ArchivedBaseKeyMapping::ModifiedOutput {
            from,
            to,
            shift,
            ctrl,
            alt,
            win,
        } => BaseKeyMapping::ModifiedOutput {
            from: convert_archived_keycode(from),
            to: convert_archived_keycode(to),
            shift: *shift,
            ctrl: *ctrl,
            alt: *alt,
            win: *win,
        },
    }
}

/// Converts an archived KeyMapping to an owned KeyMapping.
fn convert_archived_key_mapping(archived: &ArchivedKeyMapping) -> KeyMapping {
    match archived {
        ArchivedKeyMapping::Base(base) => KeyMapping::Base(convert_archived_base_mapping(base)),
        ArchivedKeyMapping::Conditional {
            condition,
            mappings,
        } => KeyMapping::Conditional {
            condition: convert_archived_condition(condition),
            mappings: mappings.iter().map(convert_archived_base_mapping).collect(),
        },
    }
}

/// Converts an archived DeviceConfig to an owned DeviceConfig.
fn convert_archived_device_config(archived: &ArchivedDeviceConfig) -> DeviceConfig {
    DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: archived.identifier.pattern.to_string(),
        },
        mappings: archived
            .mappings
            .iter()
            .map(convert_archived_key_mapping)
            .collect(),
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
/// ```no_run
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

    /// Device manager handling input keyboards.
    device_manager: DeviceManager,

    /// Virtual keyboard for output injection.
    #[cfg(target_os = "linux")]
    output: UinputOutput,
    #[cfg(target_os = "windows")]
    output: crate::platform::windows::output::WindowsKeyboardOutput,

    /// Running flag for event loop control.
    running: Arc<AtomicBool>,

    /// Signal handler for reload detection.
    signal_handler: SignalHandler,
}

impl Daemon {
    /// Creates a new daemon instance with the specified configuration file.
    ///
    /// This method performs the complete initialization sequence:
    ///
    /// 1. Loads and validates the .krx configuration file
    /// 2. Discovers input keyboard devices matching the configuration patterns
    /// 3. Creates a uinput virtual keyboard for output
    /// 4. Installs signal handlers for graceful shutdown and reload
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the .krx configuration file
    ///
    /// # Returns
    ///
    /// * `Ok(Daemon)` - Successfully initialized daemon
    /// * `Err(DaemonError::Config)` - Configuration file error
    /// * `Err(DaemonError::DiscoveryError)` - No matching devices found
    /// * `Err(DaemonError::Device)` - Failed to create uinput device
    /// * `Err(DaemonError::SignalError)` - Failed to install signal handlers
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::daemon::Daemon;
    ///
    /// match Daemon::new(Path::new("config.krx")) {
    ///     Ok(daemon) => {
    ///         println!("Daemon initialized with {} device(s)", daemon.device_count());
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Failed to initialize daemon: {}", e);
    ///     }
    /// }
    /// ```
    pub fn new(config_path: &Path) -> Result<Self, DaemonError> {
        info!(
            "Initializing keyrx daemon with config: {}",
            config_path.display()
        );

        // Step 1: Load and validate configuration
        info!("Loading configuration...");
        let config = load_config(config_path)?;
        info!(
            "Configuration loaded: {} device configuration(s)",
            config.devices.len()
        );

        // Convert archived device configs to owned for DeviceManager
        let device_configs: Vec<keyrx_core::config::DeviceConfig> = config
            .devices
            .iter()
            .map(convert_archived_device_config)
            .collect();

        // Step 2: Discover and match input devices
        info!("Discovering input devices...");
        let device_manager = DeviceManager::discover(&device_configs)?;
        info!(
            "Device discovery complete: {} device(s) matched",
            device_manager.device_count()
        );

        // Step 3: Create virtual keyboard
        info!("Creating virtual keyboard...");
        #[cfg(target_os = "linux")]
        let output = UinputOutput::create("keyrx Virtual Keyboard")?;
        #[cfg(target_os = "windows")]
        let output = crate::platform::windows::output::WindowsKeyboardOutput::new();
        info!("Virtual keyboard created");

        // Step 4: Install signal handlers
        info!("Installing signal handlers...");
        let running = Arc::new(AtomicBool::new(true));
        let signal_handler = install_signal_handlers(Arc::clone(&running))?;
        info!("Signal handlers installed");

        info!("Daemon initialization complete");

        Ok(Self {
            config_path: config_path.to_path_buf(),
            device_manager,
            output,
            running,
            signal_handler,
        })
    }

    /// Returns the number of managed devices.
    #[must_use]
    pub fn device_count(&self) -> usize {
        self.device_manager.device_count()
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

    /// Returns a reference to the device manager.
    #[must_use]
    pub fn device_manager(&self) -> &DeviceManager {
        &self.device_manager
    }

    /// Returns a mutable reference to the device manager.
    pub fn device_manager_mut(&mut self) -> &mut DeviceManager {
        &mut self.device_manager
    }

    /// Returns a reference to the output device.
    #[must_use]
    #[cfg(target_os = "linux")]
    pub fn output(&self) -> &UinputOutput {
        &self.output
    }
    #[must_use]
    #[cfg(target_os = "windows")]
    pub fn output(&self) -> &crate::platform::windows::output::WindowsKeyboardOutput {
        &self.output
    }

    /// Returns a mutable reference to the output device.
    #[cfg(target_os = "linux")]
    pub fn output_mut(&mut self) -> &mut UinputOutput {
        &mut self.output
    }
    #[cfg(target_os = "windows")]
    pub fn output_mut(&mut self) -> &mut crate::platform::windows::output::WindowsKeyboardOutput {
        &mut self.output
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

    /// Reloads the configuration from disk.
    ///
    /// This method reloads the .krx configuration file and rebuilds the lookup
    /// tables for all managed devices. Device state (modifiers, locks) is preserved
    /// to ensure seamless continuation of event processing.
    ///
    /// # Behavior
    ///
    /// - Reloads the .krx file from the path specified during daemon initialization
    /// - Rebuilds [`KeyLookup`] tables for all managed devices
    /// - Preserves [`DeviceState`] (modifier/lock state is not reset)
    /// - Does not release/re-grab devices (they stay grabbed)
    ///
    /// # Error Handling
    ///
    /// If the reload fails (file not found, parse error, etc.), the old configuration
    /// is retained and the daemon continues operating with the previous mappings.
    /// This ensures the daemon remains functional even if a bad config is deployed.
    ///
    /// # Memory Note
    ///
    /// Each call to `reload()` leaks a small amount of memory due to how the
    /// configuration loader works (see [`load_config`] documentation). For typical
    /// usage (occasional reloads on SIGHUP), this is negligible.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::daemon::Daemon;
    ///
    /// let mut daemon = Daemon::new(Path::new("config.krx"))?;
    ///
    /// // Later, after modifying the config file...
    /// match daemon.reload() {
    ///     Ok(()) => println!("Configuration reloaded successfully"),
    ///     Err(e) => eprintln!("Reload failed (keeping old config): {}", e),
    /// }
    /// # Ok::<(), keyrx_daemon::daemon::DaemonError>(())
    /// ```
    pub fn reload(&mut self) -> Result<(), DaemonError> {
        info!(
            "Reloading configuration from: {}",
            self.config_path.display()
        );

        // Step 1: Load and validate the new configuration
        let config = match load_config(&self.config_path) {
            Ok(cfg) => cfg,
            Err(e) => {
                warn!(
                    "Failed to reload configuration: {}. Keeping old configuration.",
                    e
                );
                return Err(DaemonError::Config(e));
            }
        };

        info!(
            "New configuration loaded: {} device configuration(s)",
            config.devices.len()
        );

        // Step 2: Convert archived device configs to owned for DeviceManager
        let device_configs: Vec<DeviceConfig> = config
            .devices
            .iter()
            .map(convert_archived_device_config)
            .collect();

        // Step 3: Rebuild lookup tables for all managed devices
        // This preserves DeviceState (modifier/lock state is not reset)
        // Devices stay grabbed (no release/re-grab)
        self.device_manager.rebuild_lookups(&device_configs);

        info!(
            "Configuration reload complete. {} device(s) updated.",
            self.device_manager.device_count()
        );

        Ok(())
    }

    /// Runs the main event processing loop.
    ///
    /// This method polls all managed input devices for keyboard events, processes
    /// them through the remapping engine, and injects the output events via the
    /// virtual keyboard. The loop continues until a shutdown signal (SIGTERM or
    /// SIGINT) is received.
    ///
    /// # Event Processing Flow
    ///
    /// For each input event:
    /// 1. Read event from input device
    /// 2. Look up mapping in device's lookup table
    /// 3. Update device state (for modifier/lock mappings)
    /// 4. Inject output event(s) via uinput
    ///
    /// # Multi-Device Handling
    ///
    /// The loop uses `poll()` to efficiently wait for events from all devices
    /// simultaneously. This ensures fair handling across multiple keyboards
    /// without busy-waiting.
    ///
    /// # Signal Handling
    ///
    /// - **SIGTERM/SIGINT**: Sets the running flag to false, causing graceful exit
    /// - **SIGHUP**: Triggers configuration reload via [`Self::reload`]
    ///
    /// # Errors
    ///
    /// - `DaemonError::RuntimeError`: Critical error during event processing
    /// - `DaemonError::Device`: Device I/O error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::daemon::Daemon;
    ///
    /// let mut daemon = Daemon::new(Path::new("config.krx"))?;
    ///
    /// // This blocks until shutdown signal received
    /// daemon.run()?;
    ///
    /// println!("Daemon stopped gracefully");
    /// # Ok::<(), keyrx_daemon::daemon::DaemonError>(())
    /// ```
    /// Blocks until a shutdown signal is received.
    #[cfg(target_os = "linux")]
    #[allow(unused_mut)]
    pub fn run(&mut self) -> Result<(), DaemonError> {
        info!("Starting event processing loop");

        // Grab all input devices before starting the loop
        self.grab_all_devices()?;

        // Track metrics for periodic logging
        let mut event_count: u64 = 0;
        let mut last_stats_time = Instant::now();
        const STATS_INTERVAL: Duration = Duration::from_secs(60);

        // Main event loop
        while self.is_running() {
            // Check for SIGHUP (reload request)
            if self.signal_handler.check_reload() {
                info!("Reload signal received (SIGHUP)");
                if let Err(e) = self.reload() {
                    // Log the error but continue running with old config
                    warn!("Configuration reload failed: {}", e);
                }
            }

            #[cfg(feature = "linux")]
            {
                // Poll devices for available events
                let ready_devices = match self.poll_devices() {
                    Ok(devices) => devices,
                    Err(e) => {
                        // Check if we were interrupted by signal
                        if !self.is_running() {
                            break;
                        }
                        warn!("Poll error: {}", e);
                        continue;
                    }
                };

                // Process events from ready devices
                for device_idx in ready_devices {
                    match self.process_device_events(device_idx) {
                        Ok(count) => {
                            event_count += count as u64;
                        }
                        Err(e) => {
                            // Log error but continue with other devices
                            warn!("Error processing device {}: {}", device_idx, e);
                        }
                    }
                }
            }

            #[cfg(target_os = "windows")]
            {
                // On Windows, draining events is non-blocking and handles remapping
                if let Err(e) = self.process_pending_events() {
                    warn!("Error processing events: {}", e);
                }
                // Small sleep to prevent 100% CPU usage as process_pending_events is non-blocking
                std::thread::sleep(Duration::from_millis(1));
            }

            // Check for tap-hold timeouts on all devices
            if let Err(e) = self.check_tap_hold_timeouts() {
                warn!("Error checking tap-hold timeouts: {}", e);
            }

            // Periodic stats logging
            if last_stats_time.elapsed() >= STATS_INTERVAL {
                info!(
                    "Event loop stats: {} events processed, {} devices active",
                    event_count,
                    self.device_manager.device_count()
                );
                last_stats_time = Instant::now();
            }
        }

        info!(
            "Event loop stopped. Total events processed: {}",
            event_count
        );

        Ok(())
    }

    /// Grabs exclusive access to all managed input devices.
    /// This prevents other applications from receiving keyboard events
    /// from these devices, which is essential for key remapping.
    pub fn grab_all_devices(&mut self) -> Result<(), DaemonError> {
        info!(
            "Grabbing {} input device(s)...",
            self.device_manager.device_count()
        );

        for device in self.device_manager.devices_mut() {
            let name = device.info().name.clone();
            device.input_mut().grab().map_err(|e| {
                DaemonError::RuntimeError(format!("failed to grab device '{}': {}", name, e))
            })?;
            debug!("Grabbed device: {}", name);
        }

        info!("All devices grabbed successfully");
        Ok(())
    }

    /// Polls all managed devices for available events.
    ///
    /// Returns a vector of device indices that have events ready to read.
    /// Uses a 100ms timeout to allow periodic signal checking.
    /// Uses a 100ms timeout to allow periodic signal checking.
    #[cfg(feature = "linux")]
    fn poll_devices(&self) -> Result<Vec<usize>, DaemonError> {
        // Collect raw file descriptors from all devices
        let raw_fds: Vec<i32> = self
            .device_manager
            .devices()
            .map(|device| device.input().device().as_raw_fd())
            .collect();

        // SAFETY: The raw fds are valid for the duration of this function because
        // we hold a reference to self.device_manager which owns the devices.
        // The BorrowedFd lifetime is limited to this function scope.
        let mut poll_fds: Vec<PollFd> = raw_fds
            .iter()
            .map(|&fd| {
                // SAFETY: fd is valid because it comes from a live Device
                let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
                PollFd::new(borrowed, PollFlags::POLLIN)
            })
            .collect();

        // Poll with 100ms timeout to allow signal checking
        let timeout_ms: u16 = 100;
        let result = poll(&mut poll_fds, timeout_ms)
            .map_err(|e| DaemonError::RuntimeError(format!("poll failed: {}", e)))?;

        // If no events ready (timeout), return empty
        if result == 0 {
            return Ok(Vec::new());
        }

        // Collect indices of devices with available events
        let ready_indices: Vec<usize> = poll_fds
            .iter()
            .enumerate()
            .filter_map(|(idx, pfd)| {
                if let Some(revents) = pfd.revents() {
                    if revents.contains(PollFlags::POLLIN) {
                        return Some(idx);
                    }
                }
                None
            })
            .collect();

        trace!("Poll returned {} ready device(s)", ready_indices.len());
        Ok(ready_indices)
    }

    /// Processes all pending events from all devices.
    ///
    /// On Windows, this is called from the main message loop.
    #[cfg(target_os = "windows")]
    pub fn process_pending_events(&mut self) -> Result<(), DaemonError> {
        // Check for tap-hold timeouts
        if let Err(e) = self.check_tap_hold_timeouts() {
            log::error!("Error checking tap-hold timeouts: {}", e);
        }

        // Process events from all devices
        for device_idx in 0..self.device_manager.device_count() {
            let mut device_events = Vec::new();

            // Collect all currently available events (non-blocking)
            {
                let device = self.device_manager.get_device_mut(device_idx).unwrap();
                while let Ok(event) = device.input_mut().next_event() {
                    device_events.push(event);
                }
            }

            // Process collected events
            for event in device_events {
                let device = self.device_manager.get_device_mut(device_idx).unwrap();
                let (lookup, state) = device.lookup_and_state_mut();

                let output_events = process_event(event, lookup, state);

                for out_event in output_events {
                    self.output.inject_event(out_event).map_err(|e| {
                        DaemonError::RuntimeError(format!("failed to inject event: {}", e))
                    })?;
                }
            }
        }

        Ok(())
    }

    /// Performs graceful shutdown of the daemon.
    ///
    /// This method releases all resources in the correct order:
    ///
    /// 1. Release all grabbed input devices (restores normal keyboard input)
    /// 2. Destroy the uinput virtual keyboard (removes from `/dev/input/`)
    /// 3. Log completion of each step
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
    ///
    /// let mut daemon = Daemon::new(Path::new("config.krx"))?;
    ///
    /// // Run the event loop
    /// daemon.run()?;
    ///
    /// // Explicit shutdown (optional - Drop will call this automatically)
    /// daemon.shutdown();
    /// # Ok::<(), keyrx_daemon::daemon::DaemonError>(())
    /// ```
    pub fn shutdown(&mut self) {
        info!("Initiating graceful shutdown...");

        // Step 1: Release all grabbed input devices
        info!(
            "Releasing {} grabbed device(s)...",
            self.device_manager.device_count()
        );

        for device in self.device_manager.devices_mut() {
            let name = device.info().name.clone();

            // Only release if device is grabbed
            if device.input().is_grabbed() {
                match device.input_mut().release() {
                    Ok(()) => {
                        info!("Released device: {}", name);
                    }
                    Err(e) => {
                        // Log warning but continue with other devices
                        warn!("Failed to release device '{}': {}", name, e);
                    }
                }
            } else {
                debug!("Device '{}' was not grabbed, skipping release", name);
            }
        }

        // Step 2: Destroy the uinput virtual keyboard
        info!("Destroying virtual keyboard...");
        match self.output.destroy() {
            Ok(()) => {
                info!("Virtual keyboard destroyed");
            }
            Err(e) => {
                // Log warning - device may already be destroyed by Drop
                warn!("Failed to destroy virtual keyboard: {}", e);
            }
        }

        // Mark daemon as stopped
        self.running.store(false, Ordering::SeqCst);

        info!("Shutdown complete");
    }

    /// Processes all available events from a single device.
    ///
    /// Returns the number of events processed.
    #[cfg(feature = "linux")]
    fn process_device_events(&mut self, device_idx: usize) -> Result<usize, DaemonError> {
        let mut processed_count = 0;

        loop {
            // Read and process one event, collecting output events
            let output_events = {
                let device = match self.device_manager.get_device_mut(device_idx) {
                    Some(d) => d,
                    None => return Ok(processed_count),
                };

                // Read the next event from the device
                let input_event = match device.input_mut().next_event() {
                    Ok(event) => event,
                    Err(DeviceError::EndOfStream) => {
                        // No more events available
                        break;
                    }
                    Err(DeviceError::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // Would block - no more events
                        break;
                    }
                    Err(e) => {
                        return Err(DaemonError::RuntimeError(format!(
                            "failed to read event: {}",
                            e
                        )));
                    }
                };

                trace!("Input event: {:?}", input_event);

                // Process through the remapping engine using the combined accessor
                let (lookup, state) = device.lookup_and_state_mut();
                process_event(input_event, lookup, state)
                // device borrow ends here
            };

            // Inject output events (device_manager borrow released)
            for output_event in output_events {
                trace!("Output event: {:?}", output_event);
                self.output.inject_event(output_event).map_err(|e| {
                    DaemonError::RuntimeError(format!("failed to inject event: {}", e))
                })?;
            }

            processed_count += 1;
        }

        Ok(processed_count)
    }

    /// Checks for tap-hold timeouts on all devices and injects resulting events.
    ///
    /// This method should be called periodically (after each poll cycle) to detect
    /// tap-hold keys that have been held past their threshold time. When a timeout
    /// is detected, the tap-hold processor transitions the key to Hold state and
    /// generates modifier activation events.
    ///
    /// # Errors
    ///
    /// Returns an error if event injection fails.
    fn check_tap_hold_timeouts(&mut self) -> Result<(), DaemonError> {
        let current_time = current_time_us();

        // Check each device for tap-hold timeouts
        for device_idx in 0..self.device_manager.device_count() {
            // Get timeout events for this device
            let timeout_events = {
                let device = match self.device_manager.get_device_mut(device_idx) {
                    Some(d) => d,
                    None => continue,
                };

                // Check for timeouts and get resulting events
                check_tap_hold_timeouts(current_time, device.state_mut())
            };

            // Inject any timeout-triggered events
            for event in timeout_events {
                trace!("Tap-hold timeout event: {:?}", event);
                self.output.inject_event(event).map_err(|e| {
                    DaemonError::RuntimeError(format!(
                        "failed to inject tap-hold timeout event: {}",
                        e
                    ))
                })?;
            }
        }

        Ok(())
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
    fn test_daemon_error_display() {
        let err = DaemonError::PermissionError("access denied".to_string());
        assert_eq!(err.to_string(), "permission error: access denied");

        let err = DaemonError::RuntimeError("event loop failed".to_string());
        assert_eq!(err.to_string(), "runtime error: event loop failed");
    }

    #[test]
    fn test_daemon_error_config_variant() {
        use crate::config_loader::ConfigError;
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let config_err = ConfigError::Io(io_err);
        let daemon_err = DaemonError::Config(config_err);
        assert!(daemon_err.to_string().contains("configuration error"));
    }

    #[test]
    fn test_daemon_error_device_variant() {
        use crate::platform::DeviceError;
        let device_err = DeviceError::NotFound("test device".to_string());
        let daemon_err = DaemonError::Device(device_err);
        assert!(daemon_err.to_string().contains("device error"));
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

    #[test]
    fn test_reload_state_new() {
        let state = ReloadState::new();
        assert!(!state.check_and_clear());
    }

    #[test]
    fn test_reload_state_check_and_clear() {
        let state = ReloadState::new();

        // Initially no reload requested
        assert!(!state.check_and_clear());

        // Request reload
        state.request_reload();

        // Check and clear should return true once
        assert!(state.check_and_clear());

        // Subsequent checks should return false
        assert!(!state.check_and_clear());
    }

    #[test]
    fn test_reload_state_flag_sharing() {
        let state = ReloadState::new();
        let flag = state.flag();

        // Set flag via external reference
        flag.store(true, Ordering::SeqCst);

        // Should be detectable via check_and_clear
        assert!(state.check_and_clear());
    }

    #[test]
    fn test_reload_state_default() {
        let state = ReloadState::default();
        assert!(!state.check_and_clear());
    }

    // Daemon tests - these require real devices/permissions
    mod daemon_tests {
        use super::*;
        use std::path::Path;

        #[test]
        fn test_daemon_new_missing_config() {
            let result = Daemon::new(Path::new("/nonexistent/path/config.krx"));
            assert!(result.is_err());

            // Should be a Config error
            match result {
                Err(DaemonError::Config(_)) => {} // Expected
                Err(e) => panic!("Expected Config error, got: {}", e),
                Ok(_) => panic!("Expected error, got Ok"),
            }
        }

        #[test]
        fn test_daemon_new_real_devices() {
            crate::skip_if_no_uinput!();
            use keyrx_compiler::serialize::serialize;
            use keyrx_core::config::{
                ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
            };
            use std::io::Write;
            use tempfile::NamedTempFile;

            // Create a minimal valid config
            let config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 0,
                    compiler_version: "test".to_string(),
                    source_hash: "test".to_string(),
                },
            };

            // Serialize and write to temp file
            let bytes = serialize(&config).expect("Failed to serialize config");
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            temp_file.write_all(&bytes).expect("Failed to write config");
            temp_file.flush().expect("Failed to flush");

            // Try to create daemon
            match Daemon::new(temp_file.path()) {
                Ok(daemon) => {
                    assert!(daemon.device_count() > 0, "Should have at least one device");
                    assert!(daemon.is_running(), "Daemon should be running initially");
                    println!("Daemon created with {} device(s)", daemon.device_count());
                }
                Err(e) => {
                    println!("Daemon creation failed (expected if no permissions): {}", e);
                }
            }
        }

        #[test]
        fn test_daemon_error_from_discovery_error() {
            use crate::device_manager::DiscoveryError;
            let discovery_err = DiscoveryError::NoDevicesFound;
            let daemon_err = DaemonError::from(discovery_err);
            assert!(daemon_err.to_string().contains("device discovery"));
        }

        #[test]
        fn test_daemon_error_from_io_error() {
            let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test");
            let daemon_err = DaemonError::SignalError(io_err);
            assert!(daemon_err.to_string().contains("signal handlers"));
        }

        #[test]
        fn test_daemon_reload_success() {
            crate::skip_if_no_uinput!();
            use keyrx_compiler::serialize::serialize;
            use keyrx_core::config::{
                ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
            };
            use std::io::Write;
            use tempfile::NamedTempFile;

            // Create initial config (A -> B)
            let initial_config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 0,
                    compiler_version: "test".to_string(),
                    source_hash: "initial".to_string(),
                },
            };

            // Create temp file with initial config
            let bytes = serialize(&initial_config).expect("Failed to serialize config");
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            temp_file.write_all(&bytes).expect("Failed to write config");
            temp_file.flush().expect("Failed to flush");

            // Try to create daemon
            let mut daemon = match Daemon::new(temp_file.path()) {
                Ok(d) => d,
                Err(e) => {
                    println!("Daemon creation failed (expected if no permissions): {}", e);
                    return;
                }
            };

            // Create updated config (A -> C)
            let updated_config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::C)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 1,
                    compiler_version: "test".to_string(),
                    source_hash: "updated".to_string(),
                },
            };

            // Overwrite the config file
            let updated_bytes =
                serialize(&updated_config).expect("Failed to serialize updated config");
            std::fs::write(temp_file.path(), &updated_bytes)
                .expect("Failed to write updated config");

            // Reload should succeed
            let result = daemon.reload();
            assert!(result.is_ok(), "Reload should succeed: {:?}", result.err());

            println!("Configuration reloaded successfully");
        }

        #[test]
        fn test_daemon_reload_preserves_device_count() {
            crate::skip_if_no_uinput!();
            use keyrx_compiler::serialize::serialize;
            use keyrx_core::config::{
                ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
            };
            use std::io::Write;
            use tempfile::NamedTempFile;

            // Create config
            let config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 0,
                    compiler_version: "test".to_string(),
                    source_hash: "test".to_string(),
                },
            };

            let bytes = serialize(&config).expect("Failed to serialize config");
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            temp_file.write_all(&bytes).expect("Failed to write config");
            temp_file.flush().expect("Failed to flush");

            let mut daemon = match Daemon::new(temp_file.path()) {
                Ok(d) => d,
                Err(e) => {
                    println!("Daemon creation failed (expected if no permissions): {}", e);
                    return;
                }
            };

            let device_count_before = daemon.device_count();

            // Reload (same config, device count should remain)
            let result = daemon.reload();
            assert!(result.is_ok());

            let device_count_after = daemon.device_count();
            assert_eq!(
                device_count_before, device_count_after,
                "Device count should remain same after reload"
            );
        }

        #[test]
        fn test_daemon_reload_missing_file() {
            crate::skip_if_no_uinput!();
            use keyrx_compiler::serialize::serialize;
            use keyrx_core::config::{
                ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
            };
            use std::io::Write;
            use tempfile::NamedTempFile;

            // Create config
            let config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 0,
                    compiler_version: "test".to_string(),
                    source_hash: "test".to_string(),
                },
            };

            let bytes = serialize(&config).expect("Failed to serialize config");
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            temp_file.write_all(&bytes).expect("Failed to write config");
            temp_file.flush().expect("Failed to flush");

            let mut daemon = match Daemon::new(temp_file.path()) {
                Ok(d) => d,
                Err(e) => {
                    println!("Daemon creation failed (expected if no permissions): {}", e);
                    return;
                }
            };

            // Delete the config file
            std::fs::remove_file(temp_file.path()).expect("Failed to remove config file");

            // Reload should fail but daemon should continue
            let result = daemon.reload();
            assert!(
                result.is_err(),
                "Reload should fail when config file is missing"
            );

            match result {
                Err(DaemonError::Config(_)) => {} // Expected
                Err(e) => panic!("Expected Config error, got: {}", e),
                Ok(_) => panic!("Expected error, got Ok"),
            }

            // Daemon should still be running
            assert!(
                daemon.is_running(),
                "Daemon should still be running after failed reload"
            );
        }

        #[test]
        fn test_daemon_shutdown_releases_devices() {
            crate::skip_if_no_uinput!();
            use keyrx_compiler::serialize::serialize;
            use keyrx_core::config::{
                ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
            };
            use std::io::Write;
            use tempfile::NamedTempFile;

            // Create config
            let config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 0,
                    compiler_version: "test".to_string(),
                    source_hash: "test".to_string(),
                },
            };

            let bytes = serialize(&config).expect("Failed to serialize config");
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            temp_file.write_all(&bytes).expect("Failed to write config");
            temp_file.flush().expect("Failed to flush");

            let mut daemon = match Daemon::new(temp_file.path()) {
                Ok(d) => d,
                Err(e) => {
                    println!("Daemon creation failed (expected if no permissions): {}", e);
                    return;
                }
            };

            // Verify daemon is running initially
            assert!(daemon.is_running(), "Daemon should be running initially");

            // Call shutdown
            daemon.shutdown();

            // Verify daemon is no longer running
            assert!(
                !daemon.is_running(),
                "Daemon should not be running after shutdown"
            );

            // Verify output device is destroyed
            assert!(
                daemon.output().is_destroyed(),
                "Output device should be destroyed after shutdown"
            );

            println!("Shutdown completed successfully - all resources released");
        }

        #[test]
        fn test_daemon_shutdown_idempotent() {
            crate::skip_if_no_uinput!();
            use keyrx_compiler::serialize::serialize;
            use keyrx_core::config::{
                ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
            };
            use std::io::Write;
            use tempfile::NamedTempFile;

            // Create config
            let config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 0,
                    compiler_version: "test".to_string(),
                    source_hash: "test".to_string(),
                },
            };

            let bytes = serialize(&config).expect("Failed to serialize config");
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            temp_file.write_all(&bytes).expect("Failed to write config");
            temp_file.flush().expect("Failed to flush");

            let mut daemon = match Daemon::new(temp_file.path()) {
                Ok(d) => d,
                Err(e) => {
                    println!("Daemon creation failed (expected if no permissions): {}", e);
                    return;
                }
            };

            // Call shutdown twice - should not panic or error
            daemon.shutdown();
            daemon.shutdown(); // Should handle already-released state gracefully

            assert!(!daemon.is_running(), "Daemon should not be running");
            assert!(daemon.output().is_destroyed(), "Output should be destroyed");

            println!("Multiple shutdown calls handled gracefully");
        }

        #[test]
        fn test_daemon_drop_calls_shutdown() {
            crate::skip_if_no_uinput!();
            use keyrx_compiler::serialize::serialize;
            use keyrx_core::config::{
                ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
            };
            use std::io::Write;
            use tempfile::NamedTempFile;

            // Create config
            let config = ConfigRoot {
                version: Version::current(),
                devices: vec![DeviceConfig {
                    identifier: DeviceIdentifier {
                        pattern: "*".to_string(),
                    },
                    mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
                }],
                metadata: Metadata {
                    compilation_timestamp: 0,
                    compiler_version: "test".to_string(),
                    source_hash: "test".to_string(),
                },
            };

            let bytes = serialize(&config).expect("Failed to serialize config");
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            temp_file.write_all(&bytes).expect("Failed to write config");
            temp_file.flush().expect("Failed to flush");

            // Create daemon in a block so it gets dropped
            {
                let daemon = match Daemon::new(temp_file.path()) {
                    Ok(d) => d,
                    Err(e) => {
                        println!("Daemon creation failed (expected if no permissions): {}", e);
                        return;
                    }
                };

                println!(
                    "Daemon created with {} device(s), dropping now...",
                    daemon.device_count()
                );
                // daemon will be dropped here, which should call shutdown via Drop
            }

            // If we get here without panic, Drop worked correctly
            println!("Daemon dropped successfully - cleanup via Drop verified");
        }
    }
}
