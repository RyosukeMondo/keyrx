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
mod input_capture;
mod keycode_map;
mod output_injection;
pub mod tray;

// Re-export public types
pub use input_capture::EvdevInput;
pub use output_injection::UinputOutput;
pub use tray::LinuxSystemTray;

// Re-export key mapping functions for public use
#[allow(unused_imports)] // keycode_to_evdev will be used for output injection
pub use keycode_map::{evdev_to_keycode, keycode_to_evdev, keycode_to_uinput_key};

use keyrx_core::config::DeviceConfig;

use crate::device_manager::DeviceManager;
use crate::platform::{
    DeviceError, InputDevice, OutputDevice, ProcessResult, SystemTray, TrayControlEvent,
};

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
}

impl LinuxPlatform {
    /// Creates a new LinuxPlatform instance with no devices attached.
    #[must_use]
    pub fn new() -> Self {
        Self {
            device_manager: None,
            output_device: None,
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
        // Discover and open all matching keyboard devices
        let device_manager = DeviceManager::discover(configs)?;

        eprintln!(
            "[keyrx] Discovered {} keyboard device(s)",
            device_manager.device_count()
        );
        for device in device_manager.devices() {
            eprintln!(
                "[keyrx]   - {} ({})",
                device.info().name,
                device.device_id()
            );
        }

        // Create virtual output device for event injection
        let output_device = UinputOutput::create("keyrx")?;
        eprintln!(
            "[keyrx] Created virtual output device: {}",
            output_device.name()
        );

        // Note: System tray is now managed in main.rs to ensure proper GTK event loop integration
        // LinuxPlatform no longer manages the tray directly

        self.device_manager = Some(device_manager);
        self.output_device = Some(output_device);

        // Grab exclusive access to all input devices
        self.grab_all_devices()?;

        Ok(())
    }

    /// Grabs exclusive access to all managed input devices.
    ///
    /// # Errors
    ///
    /// Returns an error if grabbing any device fails.
    fn grab_all_devices(&mut self) -> Result<(), DeviceError> {
        let device_manager = self
            .device_manager
            .as_mut()
            .ok_or_else(|| DeviceError::NotFound("device manager not initialized".to_string()))?;

        for device in device_manager.devices_mut() {
            device.input_mut().grab()?;
        }

        Ok(())
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
                    eprintln!("[keyrx] Error reading from device: {}", e);
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

        eprintln!("[keyrx] Daemon shutdown complete");
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
    fn initialize(&mut self) -> crate::platform::PlatformResult<()> {
        use crate::platform::PlatformError;
        use keyrx_core::config::mappings::DeviceIdentifier;

        // Create a wildcard configuration that matches all keyboards
        let wildcard_config = DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*".to_string(),
            },
            mappings: vec![],
        };

        // Call the existing init method
        self.init(&[wildcard_config])
            .map_err(|e| PlatformError::InitializationFailed(e.to_string()))
    }

    fn capture_input(
        &mut self,
    ) -> crate::platform::PlatformResult<keyrx_core::runtime::event::KeyEvent> {
        use crate::platform::PlatformError;

        // Get device manager
        let device_manager = self.device_manager.as_mut().ok_or_else(|| {
            PlatformError::InitializationFailed("device manager not initialized".to_string())
        })?;

        // Try to get the next event from any device
        // In the Platform trait model, we need to return ONE event, not process all devices
        for device in device_manager.devices_mut() {
            match device.input_mut().next_event() {
                Ok(event) => {
                    // Tag the event with the device ID
                    let device_id = device.device_id();
                    return Ok(event.with_device_id(device_id));
                }
                Err(DeviceError::EndOfStream) => {
                    // No events from this device, try the next one
                    continue;
                }
                Err(e) => {
                    return Err(PlatformError::Io(std::io::Error::other(e.to_string())));
                }
            }
        }

        // No events available from any device
        Err(PlatformError::DeviceNotFound(
            "No events available".to_string(),
        ))
    }

    fn inject_output(
        &mut self,
        event: keyrx_core::runtime::event::KeyEvent,
    ) -> crate::platform::PlatformResult<()> {
        use crate::platform::PlatformError;

        let output_device = self.output_device.as_mut().ok_or_else(|| {
            PlatformError::InitializationFailed("output device not initialized".to_string())
        })?;

        output_device
            .inject_event(event)
            .map_err(|e| PlatformError::InjectionFailed(e.to_string()))
    }

    fn list_devices(&self) -> crate::platform::PlatformResult<Vec<crate::platform::DeviceInfo>> {
        use crate::platform::{DeviceInfo, PlatformError};

        let device_manager = self.device_manager.as_ref().ok_or_else(|| {
            PlatformError::InitializationFailed("device manager not initialized".to_string())
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
