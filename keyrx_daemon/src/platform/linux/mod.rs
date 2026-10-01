//! Linux platform implementation using evdev for input and uinput for output.
//!
//! This module provides the Linux-specific implementation for keyboard input capture
//! and event injection using the evdev and uinput kernel interfaces.
//!
//! # System Tray Support
//!
//! The [`tray`] module provides system tray functionality via the StatusNotifierItem
//! D-Bus protocol (using the `ksni` crate).

mod device_discovery;
mod emergency_stop;
mod hotplug;
mod input_capture;
mod input_sync;
mod keycode_map;
mod output_injection;
pub mod tray;
pub(crate) mod uinput_device;

// Re-export public types
pub use emergency_stop::CHORD_TEXT as EMERGENCY_CHORD_TEXT;
pub use input_capture::EvdevInput;
pub use output_injection::UinputOutput;
pub use tray::LinuxSystemTray;

// Re-export key mapping functions for public use
#[allow(unused_imports)] // keycode_to_evdev will be used for output injection
pub use keycode_map::{evdev_to_keycode, keycode_to_evdev, keycode_to_uinput_key};

use keyrx_core::config::DeviceConfig;

use crate::device_manager::{DeviceManager, RefreshResult};
use crate::platform::{DeviceError, InputDevice, OutputDevice, ProcessResult};
use emergency_stop::EmergencyChord;
use hotplug::HotplugWatcher;

/// Linux platform structure for keyboard input/output operations.
///
/// This struct manages multiple keyboard input devices via `DeviceManager` and
/// a single uinput output device for event injection. It provides a unified
/// interface for keyboard remapping on Linux with multi-device support.
///
/// # Multi-Device Support
///
/// The platform can manage multiple input keyboards simultaneously, each with
/// its own device ID. Events from each device are tagged with the device ID
/// using `KeyEvent::with_device_id()`, enabling per-device configuration in
/// Rhai scripts.
///
/// # System Tray Support
///
/// The platform optionally manages a system tray icon that provides "Reload"
/// and "Exit" menu items. The tray is initialized during `init()` and is
/// optional - the daemon will continue to function on headless systems where
/// the tray is not available.
///
/// # Example
///
/// ```no_run
/// use keyrx_daemon::platform::linux::LinuxPlatform;
/// use keyrx_core::config::DeviceConfig;
///
/// let configs = vec![/* device configurations */];
/// let mut platform = LinuxPlatform::new();
///
/// // Initialize with device configurations
/// platform.init(&configs)?;
///
/// // Get list of device IDs for Rhai scripts
/// let device_ids = platform.device_ids();
///
/// // Process events from all devices
/// platform.process_events()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct LinuxPlatform {
    /// Device manager for handling multiple input keyboards.
    device_manager: Option<DeviceManager>,
    /// Virtual output device for injecting remapped events.
    output_device: Option<UinputOutput>,
    /// Which keyboards to grab (glob on the device name; `*` = all).
    device_pattern: String,
    /// Name of the virtual output device.
    output_name: String,
    /// The `device_start` blocks from the last successful
    /// [`reconfigure`](Self::reconfigure): hotplug re-grabs against these
    /// without the caller re-supplying them on every rescan.
    active_configs: Vec<DeviceConfig>,
    /// Watches `/dev/input` for plugged-in/unplugged keyboards. `None` when
    /// inotify could not be started (logged once at startup); hotplug is
    /// then simply off.
    hotplug: Option<HotplugWatcher>,
    /// Detects the emergency escape chord on the raw (pre-remap) stream.
    emergency: EmergencyChord,
    /// Set by every [`Self::reconfigure`] (explicit or from a hotplug
    /// rescan) and taken by [`Platform::take_devices_changed`], so the
    /// event loop knows to republish the captured-device set (H8) even
    /// when the rescan happened with no caller of its own to do so.
    devices_changed: bool,
}

impl LinuxPlatform {
    /// Creates a new LinuxPlatform instance with no devices attached.
    #[must_use]
    pub fn new() -> Self {
        Self::scoped("*", "keyrx")
    }

    /// The production platform, optionally narrowed by the environment so a
    /// SECOND instance (a test or scratch daemon next to your real one) is
    /// safe and recognisable:
    ///
    /// - `KEYRX_DEVICE_SCOPE` - only keyboards whose name matches this glob
    ///   are ever grabbed (default `*`). It can only narrow what the loaded
    ///   profile's `device_start` patterns select, never widen it.
    /// - `KEYRX_OUTPUT_NAME` - name of the virtual output keyboard (default
    ///   `keyrx`), so tools can tell instances apart.
    #[must_use]
    pub fn from_env() -> Self {
        let scope = std::env::var("KEYRX_DEVICE_SCOPE").unwrap_or_else(|_| "*".to_string());
        let output = std::env::var("KEYRX_OUTPUT_NAME").unwrap_or_else(|_| "keyrx".to_string());
        if scope != "*" || output != "keyrx" {
            log::info!("Platform scope from environment: devices '{scope}', output '{output}'");
        }
        Self::scoped(&scope, &output)
    }

    /// Creates a platform that grabs only keyboards matching `device_pattern`
    /// and injects through an output device called `output_name`. Lets tests
    /// drive a real daemon without touching the user's keyboards.
    #[must_use]
    pub fn scoped(device_pattern: &str, output_name: &str) -> Self {
        Self {
            device_manager: None,
            output_device: None,
            device_pattern: device_pattern.to_string(),
            output_name: output_name.to_string(),
            active_configs: Vec::new(),
            hotplug: None,
            emergency: EmergencyChord::new(),
            devices_changed: false,
        }
    }

    /// Initializes the platform with input and output devices.
    ///
    /// This method discovers keyboards matching the provided device configurations,
    /// creates a virtual output device for event injection, and grabs exclusive
    /// access to all managed input devices.
    ///
    /// # Arguments
    ///
    /// * `configs` - Slice of device configurations to match against discovered keyboards
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No matching keyboard devices are found
    /// - Cannot access input devices (permission denied)
    /// - Cannot create virtual output device
    /// - Cannot grab exclusive access to devices
    ///
    /// # Example
    ///
    /// ```ignore
    /// use keyrx_daemon::platform::linux::LinuxPlatform;
    /// use keyrx_core::config::DeviceConfig;
    ///
    /// let configs = vec![DeviceConfig::default()];
    /// let mut platform = LinuxPlatform::new();
    /// platform.init(&configs)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn init(&mut self, configs: &[DeviceConfig]) -> Result<(), Box<dyn std::error::Error>> {
        self.ensure_output_device()?;
        let result = self.reconfigure(configs)?;
        if self.device_count() == 0 {
            return Err(format!(
                "no keyboard devices could be grabbed (0 matched and opened, {} matched but \
                 denied - see the warnings above for why)",
                result.denied
            )
            .into());
        }
        Ok(())
    }

    /// One-time setup: the virtual output device, an empty device manager,
    /// and (best-effort) the hotplug watcher. Safe to call more than once.
    ///
    /// # Errors
    ///
    /// Returns an error if the virtual output device cannot be created
    /// (e.g. `/dev/uinput` is not accessible).
    fn ensure_output_device(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.device_manager.is_none() {
            self.device_manager = Some(DeviceManager::empty());
        }
        if self.output_device.is_none() {
            let output_device = UinputOutput::create(&self.output_name)?;
            log::info!("Created virtual output device: {}", output_device.name());
            self.output_device = Some(output_device);
        }
        if self.hotplug.is_none() {
            self.hotplug = HotplugWatcher::start();
        }
        Ok(())
    }

    /// Re-evaluates which physical keyboards are grabbed against `configs`
    /// (an empty slice falls back to `"*"` - grab-everything pass-through).
    /// Remembers `configs` as [`Self::active_configs`] so a later hotplug
    /// rescan can reuse them. Never fails just because 0 devices ended up
    /// grabbed; see [`Self::init`] for the fatal startup variant.
    ///
    /// # Errors
    ///
    /// Returns an error only if enumerating `/dev/input` itself fails.
    pub fn reconfigure(
        &mut self,
        configs: &[DeviceConfig],
    ) -> Result<RefreshResult, Box<dyn std::error::Error>> {
        self.ensure_output_device()?;
        let owned_wildcard;
        let effective: &[DeviceConfig] = if configs.is_empty() {
            owned_wildcard = wildcard_configs();
            &owned_wildcard
        } else {
            configs
        };
        let device_manager = self
            .device_manager
            .as_mut()
            .ok_or("device manager missing after ensure_output_device")?;
        let result =
            device_manager.reconcile(effective, &self.device_pattern, Some(&self.output_name))?;
        self.active_configs = effective.to_vec();
        if result.added > 0 || result.removed > 0 {
            self.devices_changed = true;
            log::info!(
                "Device reconfigure: {} grabbed, {} released, {} device(s) now managed",
                result.added,
                result.removed,
                device_manager.device_count()
            );
        }
        Ok(result)
    }

    /// Drains the hotplug watcher and, if `/dev/input` actually changed,
    /// re-grabs against [`Self::active_configs`]. Only called from the idle
    /// branch of [`capture_input`](crate::platform::Platform::capture_input)
    /// (no event was ready), so it never adds latency to the hot path.
    fn rescan_if_hotplugged(&mut self) {
        let changed = self.hotplug.as_ref().is_some_and(HotplugWatcher::drain);
        if !changed {
            return;
        }
        let configs = self.active_configs.clone();
        if let Err(e) = self.reconfigure(&configs) {
            log::warn!("Hotplug rescan failed: {e}");
        }
    }

    /// Returns the next pending key event, if any, without blocking. A
    /// [`PendingCapture::Raw`] item (H6: a code this daemon has no
    /// `KeyCode` for, or a non-key event) is forwarded straight to the
    /// output device right here and never returned - that is what keeps it
    /// in its original position relative to the key events around it,
    /// since both come off the same per-device queue in arrival order.
    fn next_raw_event(
        &mut self,
    ) -> crate::platform::PlatformResult<Option<keyrx_core::runtime::event::KeyEvent>> {
        use crate::platform::PlatformError;

        loop {
            let device_manager = self.device_manager.as_mut().ok_or_else(|| {
                PlatformError::InitializationFailed {
                    reason: "device manager not initialized".to_string(),
                }
            })?;
            match take_next_pending(device_manager)? {
                None => return Ok(None),
                Some(input_capture::PendingCapture::Key(event)) => return Ok(Some(event)),
                Some(input_capture::PendingCapture::Raw(raw)) => {
                    if let Some(output) = self.output_device.as_mut() {
                        if let Err(e) = output.inject_raw(raw.event_type, raw.code, raw.value) {
                            log::warn!("Failed to forward raw event {raw:?}: {e}");
                        }
                    }
                    // Not a key event; keep looking.
                }
            }
        }
    }

    /// Feeds `event` to the emergency-escape detector before handing it back
    /// to the caller for remapping. On a completed chord, ungrabs every
    /// device (best-effort) and turns the event into
    /// [`PlatformError::EmergencyStop`] so the event loop stops instead of
    /// remapping it.
    fn finish_capture(
        &mut self,
        event: keyrx_core::runtime::event::KeyEvent,
    ) -> crate::platform::PlatformResult<keyrx_core::runtime::event::KeyEvent> {
        use crate::platform::PlatformError;

        if self.emergency.observe(&event) {
            log::error!(
                "EMERGENCY ESCAPE chord ({}+{}+{}) held: releasing every grabbed keyboard \
                 and stopping the event loop.",
                format_args!("{:?}", emergency_stop::CHORD[0]),
                format_args!("{:?}", emergency_stop::CHORD[1]),
                format_args!("{:?}", emergency_stop::CHORD[2]),
            );
            if let Err(e) = self.release_all_devices() {
                log::warn!("Emergency stop: failed to release a device cleanly: {e}");
            }
            return Err(PlatformError::EmergencyStop);
        }
        Ok(event)
    }

    /// Releases exclusive access to all managed input devices.
    ///
    /// # Errors
    ///
    /// Returns an error if releasing any device fails.
    pub fn release_all_devices(&mut self) -> Result<(), DeviceError> {
        if let Some(ref mut device_manager) = self.device_manager {
            for device in device_manager.devices_mut() {
                device.input_mut().release()?;
            }
        }
        Ok(())
    }

    /// Returns the list of device IDs for all managed devices.
    ///
    /// These IDs can be used in Rhai scripts for per-device configuration.
    #[must_use]
    pub fn device_ids(&self) -> Vec<String> {
        self.device_manager
            .as_ref()
            .map(|dm| dm.device_ids())
            .unwrap_or_default()
    }

    /// Returns the number of managed devices.
    #[must_use]
    pub fn device_count(&self) -> usize {
        self.device_manager
            .as_ref()
            .map(|dm| dm.device_count())
            .unwrap_or(0)
    }

    /// Runs the main event processing loop.
    ///
    /// This method polls all managed input devices for events, tags each event
    /// with the source device's ID, processes it through the runtime, and injects
    /// the output events via the virtual output device. It also polls the system
    /// tray (if available) for menu events.
    ///
    /// # Event Processing
    ///
    /// For each device, the method:
    /// 1. Reads the next event from the input device
    /// 2. Tags the event with the device ID using `with_device_id()`
    /// 3. Processes the event through the device's key lookup and state
    /// 4. Injects output events to the virtual output device
    ///
    /// Additionally, the system tray is polled for menu events:
    /// - `TrayControlEvent::Reload` returns `ProcessResult::ReloadRequested`
    /// - `TrayControlEvent::Exit` returns `ProcessResult::ExitRequested`
    ///
    /// # Returns
    ///
    /// - `Ok(ProcessResult::Continue)`: Normal operation, continue processing
    /// - `Ok(ProcessResult::ReloadRequested)`: User clicked "Reload" in tray menu
    /// - `Ok(ProcessResult::ExitRequested)`: User clicked "Exit" in tray menu
    /// - `Err(...)`: An error occurred during processing
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Reading from an input device fails
    /// - Injecting an output event fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// use keyrx_daemon::platform::linux::{LinuxPlatform, ProcessResult};
    /// use keyrx_core::config::DeviceConfig;
    ///
    /// let configs = vec![DeviceConfig::default()];
    /// let mut platform = LinuxPlatform::new();
    /// platform.init(&configs)?;
    ///
    /// loop {
    ///     match platform.process_events()? {
    ///         ProcessResult::Continue => {}
    ///         ProcessResult::ReloadRequested => {
    ///             println!("Reload requested");
    ///             // Reload configuration...
    ///         }
    ///         ProcessResult::ExitRequested => {
    ///             println!("Exit requested");
    ///             platform.shutdown()?;
    ///             break;
    ///         }
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn process_events(&mut self) -> Result<ProcessResult, Box<dyn std::error::Error>> {
        use keyrx_core::runtime::event::process_event;

        // Note: System tray is now managed in main.rs

        let device_manager = self
            .device_manager
            .as_mut()
            .ok_or_else(|| DeviceError::NotFound("device manager not initialized".to_string()))?;

        let output_device = self
            .output_device
            .as_mut()
            .ok_or_else(|| DeviceError::NotFound("output device not initialized".to_string()))?;

        // Process one event from each device that has events available
        for device in device_manager.devices_mut() {
            // Try to get the next event from this device (non-blocking would be ideal)
            match device.input_mut().next_event() {
                Ok(event) => {
                    // Tag the event with the device ID
                    let device_id = device.device_id();
                    let tagged_event = event.with_device_id(device_id);

                    // Process the event through the device's lookup and state
                    let (lookup, state) = device.lookup_and_state_mut();
                    let output_events = process_event(tagged_event, lookup, state);

                    // Inject output events
                    for output_event in output_events {
                        output_device.inject_event(output_event)?;
                    }
                }
                Err(DeviceError::EndOfStream) => {
                    // No more events from this device right now
                    continue;
                }
                Err(e) => {
                    // Log error but continue with other devices
                    log::warn!("Error reading from device: {}", e);
                }
            }
        }

        Ok(ProcessResult::Continue)
    }

    /// Shuts down the platform, releasing all resources.
    ///
    /// This method:
    /// 1. Shuts down the system tray (if available)
    /// 2. Releases exclusive access to all input devices
    /// 3. Destroys the virtual output device
    ///
    /// # Errors
    ///
    /// Returns an error if releasing devices fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use keyrx_daemon::platform::linux::LinuxPlatform;
    /// use keyrx_core::config::DeviceConfig;
    ///
    /// let configs = vec![DeviceConfig::default()];
    /// let mut platform = LinuxPlatform::new();
    /// platform.init(&configs)?;
    /// // ... process events ...
    /// platform.shutdown()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Note: System tray is now managed in main.rs and shutdown there

        // Release exclusive access to input devices
        self.release_all_devices()?;

        // Output device cleanup happens automatically via Drop

        log::info!("Platform shutdown complete");
        Ok(())
    }
}

impl Default for LinuxPlatform {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: LinuxPlatform is used in a single-threaded context in practice.
// The DeviceManager and UinputOutput are thread-safe.
// System tray is now managed separately in main.rs.
unsafe impl Send for LinuxPlatform {}
unsafe impl Sync for LinuxPlatform {}

// Platform trait implementation
impl crate::platform::Platform for LinuxPlatform {
    /// One-time setup only (output device, hotplug watcher). Does **not**
    /// grab any keyboard - the caller must follow up with
    /// [`reconfigure_devices`](crate::platform::Platform::reconfigure_devices)
    /// once it knows the live config's `device_start` patterns, so only
    /// devices a pattern actually matches get grabbed (an empty/no config
    /// falls back to `"*"`, i.e. grab-everything pass-through).
    fn initialize(&mut self) -> crate::platform::PlatformResult<()> {
        use crate::platform::PlatformError;

        self.ensure_output_device()
            .map_err(|e| PlatformError::InitializationFailed {
                reason: e.to_string(),
            })
    }

    fn reconfigure_devices(
        &mut self,
        configs: &[DeviceConfig],
    ) -> crate::platform::PlatformResult<()> {
        use crate::platform::PlatformError;

        self.reconfigure(configs)
            .map(|_| ())
            .map_err(|e| PlatformError::InitializationFailed {
                reason: e.to_string(),
            })
    }

    fn take_devices_changed(&mut self) -> bool {
        std::mem::take(&mut self.devices_changed)
    }

    fn take_input_overflows(&mut self) -> u64 {
        self.device_manager.as_mut().map_or(0, |dm| {
            dm.devices_mut()
                .map(|device| device.input_mut().take_overflows())
                .sum()
        })
    }

    fn capture_input(
        &mut self,
    ) -> crate::platform::PlatformResult<keyrx_core::runtime::event::KeyEvent> {
        use crate::platform::PlatformError;

        if let Some(event) = self.next_raw_event()? {
            return self.finish_capture(event);
        }
        {
            let device_manager = self.device_manager.as_ref().ok_or_else(|| {
                PlatformError::InitializationFailed {
                    reason: "device manager not initialized".to_string(),
                }
            })?;
            wait_for_input(device_manager, INPUT_WAIT)?;
        }
        self.rescan_if_hotplugged();
        match self.next_raw_event()? {
            Some(event) => self.finish_capture(event),
            None => Err(PlatformError::NoInput),
        }
    }

    fn inject_output(
        &mut self,
        event: keyrx_core::runtime::event::KeyEvent,
    ) -> crate::platform::PlatformResult<()> {
        use crate::platform::PlatformError;

        let output_device =
            self.output_device
                .as_mut()
                .ok_or_else(|| PlatformError::InitializationFailed {
                    reason: "output device not initialized".to_string(),
                })?;

        output_device
            .inject_event(event)
            .map_err(|e| PlatformError::InjectionFailed {
                reason: e.to_string(),
                suggestion: "Check uinput device permissions and kernel module".to_string(),
            })
    }

    fn list_devices(&self) -> crate::platform::PlatformResult<Vec<crate::platform::DeviceInfo>> {
        use crate::platform::{DeviceInfo, PlatformError};

        let device_manager =
            self.device_manager
                .as_ref()
                .ok_or_else(|| PlatformError::InitializationFailed {
                    reason: "device manager not initialized".to_string(),
                })?;

        // devices() returns an iterator, so we can map over it directly
        let devices = device_manager
            .devices()
            .map(|device| {
                let info = device.info();
                DeviceInfo {
                    id: device.device_id().to_string(),
                    name: info.name.clone(),
                    path: info.path.to_string_lossy().to_string(),
                    // KeyboardInfo doesn't have USB IDs, use placeholders
                    vendor_id: 0,
                    product_id: 0,
                }
            })
            .collect();

        Ok(devices)
    }

    fn shutdown(&mut self) -> crate::platform::PlatformResult<()> {
        use crate::platform::PlatformError;

        // Call existing shutdown method
        self.shutdown()
            .map_err(|e| PlatformError::Io(std::io::Error::other(e.to_string())))
    }
}

/// Longest `capture_input` waits for input before returning `NoInput`, which
/// bounds how late the event loop services reloads and tap-hold timeouts.
const INPUT_WAIT: std::time::Duration = std::time::Duration::from_millis(10);

/// The single `"*"` block used when no config is loaded (pass-through):
/// grab and pass through every keyboard in scope, same as before a config
/// existed.
fn wildcard_configs() -> Vec<DeviceConfig> {
    use keyrx_core::config::mappings::DeviceIdentifier;
    vec![DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: "*".to_string(),
        },
        mappings: vec![],
    }]
}

/// Returns the next pending item (key event or raw passthrough, see
/// [`input_capture::PendingCapture`]) from any device without blocking.
///
/// A device whose read errors (not [`DeviceError::EndOfStream`], which just
/// means "no event right now") is dropped immediately with one log line -
/// almost always `ENODEV` from an unplugged keyboard - instead of that one
/// dead fd starving every other device's events behind it every poll. It is
/// re-grabbed automatically if it comes back (hotplug).
fn take_next_pending(
    device_manager: &mut DeviceManager,
) -> crate::platform::PlatformResult<Option<input_capture::PendingCapture>> {
    let mut found = None;
    let mut vanished: Vec<String> = Vec::new();
    for device in device_manager.devices_mut() {
        match device.input_mut().next_pending() {
            Ok(input_capture::PendingCapture::Key(event)) => {
                found = Some(input_capture::PendingCapture::Key(
                    event.with_device_id(device.device_id()),
                ));
                break;
            }
            Ok(raw @ input_capture::PendingCapture::Raw(_)) => {
                found = Some(raw);
                break;
            }
            Err(DeviceError::EndOfStream) => continue,
            Err(e) => {
                log::warn!(
                    "Keyboard '{}' ({}) disappeared: {e}. Releasing it.",
                    device.info().name,
                    device.info().path.display()
                );
                vanished.push(device.device_id());
            }
        }
    }
    for id in vanished {
        device_manager.drop_by_id(&id);
    }
    Ok(found)
}

/// Blocks until any device is readable or `timeout` elapses.
///
/// Devices are non-blocking, so this is the only place the capture thread
/// waits: an idle keyboard can no longer starve the others.
fn wait_for_input(
    device_manager: &DeviceManager,
    timeout: std::time::Duration,
) -> crate::platform::PlatformResult<()> {
    use nix::poll::{poll, PollFd, PollFlags, PollTimeout};

    let mut fds: Vec<PollFd> = device_manager
        .devices()
        .map(|d| PollFd::new(d.input().poll_fd(), PollFlags::POLLIN))
        .collect();
    let timeout_ms = u16::try_from(timeout.as_millis()).unwrap_or(u16::MAX);
    match poll(&mut fds, PollTimeout::from(timeout_ms)) {
        Ok(_) | Err(nix::errno::Errno::EINTR) => Ok(()),
        Err(e) => Err(crate::platform::PlatformError::Io(e.into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Platform;

    #[test]
    fn test_platform_trait_usage() {
        // Verify LinuxPlatform can be used as Box<dyn Platform>
        let platform: Box<dyn Platform> = Box::new(LinuxPlatform::new());
        let _ = platform; // Compile-time check that trait object works
    }

    #[test]
    fn test_linux_platform_implements_platform() {
        // Verify LinuxPlatform implements all Platform trait methods
        let platform = LinuxPlatform::new();

        // Test that we can call trait methods
        // Note: These will fail without actual devices, but the type-checking is what matters
        let _ = platform.list_devices();
    }
}
