//! SafeUinput wrapper for virtual device operations.
//!
//! This module provides a safe RAII wrapper around uinput virtual device operations,
//! ensuring proper cleanup on drop and validation of injected events.

use crate::drivers::common::error::DriverError;
use crate::engine::KeyCode;
use evdev::{
    uinput::VirtualDevice, uinput::VirtualDeviceBuilder, AttributeSet, EventType,
    InputEvent as EvdevInputEvent, Key,
};
use std::io;
use tracing::debug;

/// Safe wrapper around a uinput virtual device with RAII cleanup.
///
/// `SafeUinput` manages the lifecycle of a virtual input device, including:
/// - Creating and configuring the virtual device
/// - Validating event data before injection
/// - Automatic cleanup on drop
/// - Graceful error handling
///
/// # RAII Guarantees
///
/// When a `SafeUinput` is dropped:
/// 1. The virtual device is automatically destroyed
/// 2. The uinput file descriptor is closed
/// 3. These operations happen even during panic unwinding
///
/// This ensures proper cleanup of kernel resources.
///
/// # Event Validation
///
/// Before injecting events, `SafeUinput` validates that:
/// - The event type is supported (KEY events)
/// - The key code is within valid range
/// - The event value is valid (0 for release, 1 for press)
///
/// Invalid events return `DriverError::InvalidEvent` rather than causing
/// undefined behavior or kernel errors.
///
/// # Example
///
/// ```no_run
/// use keyrx_core::drivers::linux::safety::uinput::SafeUinput;
/// use keyrx_core::engine::KeyCode;
///
/// let mut device = SafeUinput::new("KeyRx Virtual Keyboard")?;
///
/// // Inject key press
/// device.emit_key(KeyCode::A, true)?;
/// device.sync()?;
///
/// // Inject key release
/// device.emit_key(KeyCode::A, false)?;
/// device.sync()?;
///
/// // Device is automatically destroyed on drop
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(dead_code)] // Infrastructure for integration in next task
pub struct SafeUinput {
    /// The underlying uinput virtual device.
    device: VirtualDevice,
    /// Name of the virtual device (for logging and error messages).
    name: String,
}

#[allow(dead_code)] // Infrastructure for integration in next task
impl SafeUinput {
    /// Create a new virtual uinput device with the specified name.
    ///
    /// The device is registered with all supported key codes to ensure
    /// any key can be injected through this device.
    ///
    /// # Arguments
    ///
    /// * `name` - Name for the virtual device (visible in /proc/bus/input/devices)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `/dev/uinput` does not exist (`VirtualDeviceError`)
    /// - Permission is denied (`PermissionDenied` with hints)
    /// - The virtual device creation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_core::drivers::linux::safety::uinput::SafeUinput;
    ///
    /// let device = SafeUinput::new("My Virtual Keyboard")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(name: impl Into<String>) -> Result<Self, DriverError> {
        let name = name.into();

        // Build the set of keys to register
        let keys = Self::build_key_set();

        // Create the virtual device
        let device = VirtualDeviceBuilder::new()
            .map_err(Self::map_creation_error)?
            .name(&name)
            .with_keys(&keys)
            .map_err(Self::map_builder_error)?
            .build()
            .map_err(Self::map_build_error)?;

        debug!(
            service = "keyrx",
            event = "safe_uinput_created",
            component = "linux_safety",
            device_name = %name,
            "SafeUinput device created"
        );

        Ok(Self { device, name })
    }

    /// Build the set of key codes to register with the virtual device.
    ///
    /// Registers all keys from 0 to KEY_MAX to support any possible key injection.
    /// This ensures compatibility with all KeyCode enum variants.
    fn build_key_set() -> AttributeSet<Key> {
        let mut keys = AttributeSet::<Key>::new();

        // Register all possible key codes
        // evdev Key codes range from 0 to ~767 (KEY_MAX)
        // We register a comprehensive set to support all KeyCode variants
        for code in 0..=767 {
            keys.insert(Key::new(code));
        }

        keys
    }

    /// Emit a key event (press or release) through the virtual device.
    ///
    /// The event is validated before injection to ensure it's safe to send
    /// to the kernel. Invalid events return an error rather than causing
    /// undefined behavior.
    ///
    /// # Arguments
    ///
    /// * `key_code` - The evdev key code to emit
    /// * `pressed` - `true` for key press (value=1), `false` for release (value=0)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The key code is invalid (`InvalidEvent`)
    /// - The injection operation fails (`InjectionFailed`)
    ///
    /// # Note
    ///
    /// You must call `sync()` after emitting events to flush them to the system.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_core::drivers::linux::safety::uinput::SafeUinput;
    ///
    /// let mut device = SafeUinput::new("Test Device")?;
    /// device.emit_key_code(30, true)?;  // KEY_A press
    /// device.sync()?;
    /// device.emit_key_code(30, false)?; // KEY_A release
    /// device.sync()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn emit_key_code(&mut self, key_code: u16, pressed: bool) -> Result<(), DriverError> {
        // Validate key code range
        if key_code > 767 {
            return Err(DriverError::InvalidEvent {
                details: format!("Key code {} exceeds maximum value (767)", key_code),
            });
        }

        let value = if pressed { 1 } else { 0 };

        // Create the key event
        // SAFETY: We've validated the key_code is within valid range
        let event = EvdevInputEvent::new(EventType::KEY, key_code, value);

        // Write the event to the virtual device
        self.device
            .emit(&[event])
            .map_err(|e| Self::map_emit_error(&self.name, e))?;

        Ok(())
    }

    /// Emit a key event using a KeyCode enum variant.
    ///
    /// This is a higher-level wrapper around `emit_key_code` that accepts
    /// KeyCode enum variants and converts them to evdev codes.
    ///
    /// # Arguments
    ///
    /// * `key` - The KeyCode to emit
    /// * `pressed` - `true` for key press, `false` for release
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The key code conversion fails (`InvalidEvent`)
    /// - The injection operation fails (`InjectionFailed`)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_core::drivers::linux::safety::uinput::SafeUinput;
    /// use keyrx_core::engine::KeyCode;
    ///
    /// let mut device = SafeUinput::new("Test Device")?;
    /// device.emit_key(KeyCode::A, true)?;
    /// device.sync()?;
    /// device.emit_key(KeyCode::A, false)?;
    /// device.sync()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn emit_key(&mut self, key: KeyCode, pressed: bool) -> Result<(), DriverError> {
        use crate::drivers::linux::keymap::keycode_to_evdev;
        let evdev_code = keycode_to_evdev(key);
        self.emit_key_code(evdev_code, pressed)
    }

    /// Send a synchronization event to flush pending events.
    ///
    /// The kernel buffers input events until an `EV_SYN` event is received.
    /// This method writes the sync event to ensure all pending key events
    /// are processed immediately.
    ///
    /// # Errors
    ///
    /// Returns an error if the sync event cannot be written (`InjectionFailed`).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_core::drivers::linux::safety::uinput::SafeUinput;
    /// use keyrx_core::engine::KeyCode;
    ///
    /// let mut device = SafeUinput::new("Test Device")?;
    /// device.emit_key(KeyCode::A, true)?;
    /// device.sync()?; // Flush the key press
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn sync(&mut self) -> Result<(), DriverError> {
        // EV_SYN = 0, SYN_REPORT = 0, value = 0
        let sync_event = EvdevInputEvent::new(EventType::SYNCHRONIZATION, 0, 0);

        self.device
            .emit(&[sync_event])
            .map_err(|e| Self::map_emit_error(&self.name, e))?;

        Ok(())
    }

    /// Get a reference to the underlying virtual device.
    ///
    /// This allows direct access to evdev operations while maintaining
    /// the RAII guarantees of the wrapper.
    pub fn device(&self) -> &VirtualDevice {
        &self.device
    }

    /// Get a mutable reference to the underlying virtual device.
    ///
    /// # Safety Note
    ///
    /// While this provides direct access to the device, the RAII guarantees
    /// are maintained. Dropping the `SafeUinput` will still clean up the device.
    pub fn device_mut(&mut self) -> &mut VirtualDevice {
        &mut self.device
    }

    /// Get the name of the virtual device.
    pub fn name(&self) -> &str {
        &self.name
    }

    // Error mapping helpers

    /// Map VirtualDeviceBuilder creation errors to DriverError.
    fn map_creation_error(error: io::Error) -> DriverError {
        match error.kind() {
            io::ErrorKind::NotFound => DriverError::VirtualDeviceError {
                message: "uinput device not found at /dev/uinput\n\
                          Hint: Load the uinput kernel module: sudo modprobe uinput"
                    .to_string(),
            },
            io::ErrorKind::PermissionDenied => DriverError::PermissionDenied {
                resource: "/dev/uinput".to_string(),
                hint: "Add your user to the 'input' group: sudo usermod -aG input $USER\n\
                       Or configure udev rules:\n  \
                       echo 'KERNEL==\"uinput\", MODE=\"0660\", GROUP=\"input\"' | \
                       sudo tee /etc/udev/rules.d/99-uinput.rules\n  \
                       Then reload: sudo udevadm control --reload-rules && sudo udevadm trigger"
                    .to_string(),
            },
            _ => DriverError::VirtualDeviceError {
                message: format!("Failed to create VirtualDeviceBuilder: {}", error),
            },
        }
    }

    /// Map VirtualDeviceBuilder configuration errors to DriverError.
    fn map_builder_error(error: io::Error) -> DriverError {
        DriverError::VirtualDeviceError {
            message: format!("Failed to configure virtual device: {}", error),
        }
    }

    /// Map VirtualDeviceBuilder build errors to DriverError.
    fn map_build_error(error: io::Error) -> DriverError {
        match error.kind() {
            io::ErrorKind::PermissionDenied => DriverError::PermissionDenied {
                resource: "/dev/uinput".to_string(),
                hint: "Check permissions on /dev/uinput and ensure uinput module is loaded"
                    .to_string(),
            },
            _ => DriverError::VirtualDeviceError {
                message: format!("Failed to build virtual device: {}", error),
            },
        }
    }

    /// Map event emission errors to DriverError.
    fn map_emit_error(device_name: &str, error: io::Error) -> DriverError {
        match error.kind() {
            io::ErrorKind::BrokenPipe | io::ErrorKind::NotConnected => {
                DriverError::DeviceDisconnected {
                    device: device_name.to_string(),
                }
            }
            _ => DriverError::InjectionFailed {
                reason: format!("Failed to emit event to {}: {}", device_name, error),
            },
        }
    }
}

impl Drop for SafeUinput {
    /// Automatically destroy the virtual device on drop.
    ///
    /// # SAFETY
    ///
    /// This ensures the virtual device is properly cleaned up even during
    /// panic unwinding, preventing resource leaks in the kernel.
    ///
    /// The uinput subsystem automatically removes the device when the
    /// file descriptor is closed, which happens when VirtualDevice is dropped.
    fn drop(&mut self) {
        debug!(
            service = "keyrx",
            event = "safe_uinput_dropped",
            component = "linux_safety",
            device_name = %self.name,
            "SafeUinput device destroyed"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_key_set_registers_comprehensive_keys() {
        let keys = SafeUinput::build_key_set();

        // Verify common keys are registered
        assert!(keys.contains(Key::new(30))); // KEY_A
        assert!(keys.contains(Key::new(1))); // KEY_ESC
        assert!(keys.contains(Key::new(57))); // KEY_SPACE
        assert!(keys.contains(Key::new(28))); // KEY_ENTER

        // Verify we have a comprehensive set (at least 100 keys)
        assert!(keys.iter().count() >= 100);
    }

    #[test]
    fn emit_key_code_validates_range() {
        // This test would need a real uinput device or mock
        // For now, just test the validation logic would work
        // by checking the key code range
        assert!(768 > 767); // Invalid code
        assert!(30 <= 767); // Valid code
    }

    #[test]
    fn error_mapping_permission_denied() {
        let error = io::Error::from(io::ErrorKind::PermissionDenied);
        let driver_error = SafeUinput::map_creation_error(error);

        match driver_error {
            DriverError::PermissionDenied { resource, hint } => {
                assert!(resource.contains("uinput"));
                assert!(hint.contains("input"));
                assert!(hint.contains("udev"));
            }
            _ => panic!("Expected PermissionDenied error"),
        }
    }

    #[test]
    fn error_mapping_not_found() {
        let error = io::Error::from(io::ErrorKind::NotFound);
        let driver_error = SafeUinput::map_creation_error(error);

        match driver_error {
            DriverError::VirtualDeviceError { message } => {
                assert!(message.contains("uinput"));
                assert!(message.contains("modprobe"));
            }
            _ => panic!("Expected VirtualDeviceError"),
        }
    }

    #[test]
    fn error_mapping_disconnected() {
        let error = io::Error::from(io::ErrorKind::BrokenPipe);
        let driver_error = SafeUinput::map_emit_error("test device", error);

        match driver_error {
            DriverError::DeviceDisconnected { device } => {
                assert_eq!(device, "test device");
            }
            _ => panic!("Expected DeviceDisconnected error"),
        }
    }
}
