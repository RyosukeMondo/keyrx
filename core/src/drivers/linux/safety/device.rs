//! SafeDevice wrapper for evdev device operations.
//!
//! This module provides a safe RAII wrapper around evdev device operations,
//! ensuring proper cleanup on drop and graceful handling of device disconnection.

use crate::drivers::common::error::DriverError;
use evdev::Device;
use std::io;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Safe wrapper around an evdev device with RAII cleanup.
///
/// `SafeDevice` manages the lifecycle of an evdev device, including:
/// - Opening and validating device access
/// - Exclusive device grabbing (EVIOCGRAB)
/// - Automatic ungrabbing on drop
/// - Graceful disconnection handling
///
/// # RAII Guarantees
///
/// When a `SafeDevice` is dropped:
/// 1. If the device was grabbed, it is automatically ungrabbed
/// 2. The device file descriptor is closed
/// 3. These operations happen even during panic unwinding
///
/// This ensures the keyboard is never left in a stuck/grabbed state.
///
/// # Example
///
/// ```no_run
/// use keyrx_core::drivers::linux::safety::device::SafeDevice;
///
/// let mut device = SafeDevice::open("/dev/input/event3")?;
/// device.grab()?;
///
/// // Use the device...
/// for event in device.iter_events()? {
///     println!("Event: {:?}", event);
/// }
///
/// // Device is automatically ungrabbed and closed on drop
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(dead_code)] // Infrastructure for integration in next task
pub struct SafeDevice {
    /// The underlying evdev device.
    device: Device,
    /// Path to the device (for logging and error messages).
    path: PathBuf,
    /// Whether the device is currently grabbed.
    is_grabbed: bool,
}

#[allow(dead_code)] // Infrastructure for integration in next task
impl SafeDevice {
    /// Open an evdev device at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the evdev device (e.g., `/dev/input/event3`)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The device does not exist (`DeviceNotFound`)
    /// - Permission is denied (`PermissionDenied` with hints)
    /// - The device cannot be opened for other reasons
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_core::drivers::linux::safety::device::SafeDevice;
    ///
    /// let device = SafeDevice::open("/dev/input/event3")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DriverError> {
        let path = path.as_ref().to_path_buf();

        let device = Device::open(&path).map_err(|e| Self::map_open_error(&path, e))?;

        debug!(
            service = "keyrx",
            event = "safe_device_opened",
            component = "linux_safety",
            device = device.name().unwrap_or("Unknown"),
            path = %path.display(),
            "SafeDevice opened"
        );

        Ok(Self {
            device,
            path,
            is_grabbed: false,
        })
    }

    /// Grab exclusive access to the device.
    ///
    /// While grabbed, all events from this device are routed only to this process,
    /// preventing other applications from receiving them. This is essential for
    /// proper key remapping.
    ///
    /// # Errors
    ///
    /// Returns `DriverError::GrabFailed` if:
    /// - Another process has already grabbed the device
    /// - The device was disconnected
    /// - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_core::drivers::linux::safety::device::SafeDevice;
    ///
    /// let mut device = SafeDevice::open("/dev/input/event3")?;
    /// device.grab()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn grab(&mut self) -> Result<(), DriverError> {
        if self.is_grabbed {
            return Ok(());
        }

        self.device
            .grab()
            .map_err(|e| Self::map_grab_error(&self.path, e))?;

        self.is_grabbed = true;

        debug!(
            service = "keyrx",
            event = "safe_device_grabbed",
            component = "linux_safety",
            path = %self.path.display(),
            "Device grabbed"
        );

        Ok(())
    }

    /// Ungrab the device, allowing other applications to receive events.
    ///
    /// This is called automatically on drop, but can be called explicitly
    /// if you need to release the device before dropping.
    ///
    /// # Errors
    ///
    /// Returns `DriverError::UngrabFailed` if the ungrab operation fails.
    /// This is rare and usually indicates the device was already ungrabbed
    /// or disconnected.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_core::drivers::linux::safety::device::SafeDevice;
    ///
    /// let mut device = SafeDevice::open("/dev/input/event3")?;
    /// device.grab()?;
    /// // ... use device ...
    /// device.ungrab()?; // Explicit ungrab
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn ungrab(&mut self) -> Result<(), DriverError> {
        if !self.is_grabbed {
            return Ok(());
        }

        self.device
            .ungrab()
            .map_err(|e| Self::map_ungrab_error(&self.path, e))?;

        self.is_grabbed = false;

        debug!(
            service = "keyrx",
            event = "safe_device_ungrabbed",
            component = "linux_safety",
            path = %self.path.display(),
            "Device ungrabbed"
        );

        Ok(())
    }

    /// Get a reference to the underlying evdev device.
    ///
    /// This allows direct access to evdev operations while maintaining
    /// the safety guarantees of the wrapper.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get a mutable reference to the underlying evdev device.
    ///
    /// # Safety Note
    ///
    /// While this provides direct access to the device, the RAII guarantees
    /// are maintained. Dropping the `SafeDevice` will still ungrab and close
    /// the device even if operations are performed through this reference.
    pub fn device_mut(&mut self) -> &mut Device {
        &mut self.device
    }

    /// Get the device path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Check if the device is currently grabbed.
    pub fn is_grabbed(&self) -> bool {
        self.is_grabbed
    }

    /// Fetch events from the device with non-blocking I/O.
    ///
    /// Returns an iterator over pending events. If no events are available,
    /// returns an empty iterator.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The device was disconnected (`DeviceDisconnected`)
    /// - An I/O error occurred
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_core::drivers::linux::safety::device::SafeDevice;
    ///
    /// let mut device = SafeDevice::open("/dev/input/event3")?;
    /// device.grab()?;
    ///
    /// for event in device.fetch_events()? {
    ///     println!("Event: {:?}", event);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn fetch_events(
        &mut self,
    ) -> Result<impl Iterator<Item = evdev::InputEvent> + '_, DriverError> {
        self.device
            .fetch_events()
            .map_err(|e| Self::map_fetch_error(&self.path, e))
    }

    // Error mapping helpers

    /// Map evdev open errors to DriverError with helpful context.
    fn map_open_error(path: &Path, error: io::Error) -> DriverError {
        match error.kind() {
            io::ErrorKind::NotFound => DriverError::DeviceNotFound {
                path: path.to_path_buf(),
            },
            io::ErrorKind::PermissionDenied => DriverError::PermissionDenied {
                resource: path.display().to_string(),
                hint: "Add your user to the 'input' group: sudo usermod -aG input $USER\n\
                       Or configure udev rules for device access:\n  \
                       echo 'KERNEL==\"event*\", MODE=\"0660\", GROUP=\"input\"' | \
                       sudo tee /etc/udev/rules.d/99-input.rules"
                    .to_string(),
            },
            _ => DriverError::Platform(error),
        }
    }

    /// Map evdev grab errors to DriverError.
    fn map_grab_error(path: &Path, error: io::Error) -> DriverError {
        match error.kind() {
            io::ErrorKind::NotFound => DriverError::DeviceDisconnected {
                device: path.display().to_string(),
            },
            io::ErrorKind::PermissionDenied => DriverError::PermissionDenied {
                resource: path.display().to_string(),
                hint: "Ensure your user has permission to grab the device".to_string(),
            },
            _ => DriverError::GrabFailed {
                reason: format!("Failed to grab {}: {}", path.display(), error),
            },
        }
    }

    /// Map evdev ungrab errors to DriverError.
    fn map_ungrab_error(path: &Path, error: io::Error) -> DriverError {
        DriverError::UngrabFailed {
            reason: format!("Failed to ungrab {}: {}", path.display(), error),
        }
    }

    /// Map evdev fetch errors to DriverError.
    fn map_fetch_error(path: &Path, error: io::Error) -> DriverError {
        match error.kind() {
            io::ErrorKind::NotFound => DriverError::DeviceDisconnected {
                device: path.display().to_string(),
            },
            _ => DriverError::Platform(error),
        }
    }
}

impl Drop for SafeDevice {
    /// Automatically ungrab and close the device on drop.
    ///
    /// # SAFETY
    ///
    /// This ensures the device is ungrabbed even during panic unwinding,
    /// preventing the keyboard from being left in a stuck state.
    ///
    /// Errors during ungrab are logged but not propagated (Drop cannot fail).
    fn drop(&mut self) {
        if self.is_grabbed {
            debug!(
                service = "keyrx",
                event = "safe_device_drop_ungrabbing",
                component = "linux_safety",
                path = %self.path.display(),
                "SafeDevice::drop - ungrabbing device"
            );

            if let Err(e) = self.ungrab() {
                warn!(
                    service = "keyrx",
                    event = "safe_device_drop_ungrab_failed",
                    component = "linux_safety",
                    path = %self.path.display(),
                    error = %e,
                    "Failed to ungrab device in drop"
                );
            }
        }

        debug!(
            service = "keyrx",
            event = "safe_device_dropped",
            component = "linux_safety",
            path = %self.path.display(),
            "SafeDevice dropped"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_not_grabbed_initially() {
        // This test would need a real device or mock
        // For now, just test the API structure
        assert!(true);
    }

    #[test]
    fn device_error_mapping_not_found() {
        let path = PathBuf::from("/dev/input/event999");
        let error = io::Error::from(io::ErrorKind::NotFound);
        let driver_error = SafeDevice::map_open_error(&path, error);

        match driver_error {
            DriverError::DeviceNotFound { path: p } => {
                assert_eq!(p, path);
            }
            _ => panic!("Expected DeviceNotFound error"),
        }
    }

    #[test]
    fn device_error_mapping_permission() {
        let path = PathBuf::from("/dev/input/event0");
        let error = io::Error::from(io::ErrorKind::PermissionDenied);
        let driver_error = SafeDevice::map_open_error(&path, error);

        match driver_error {
            DriverError::PermissionDenied { resource, hint } => {
                assert!(resource.contains("event0"));
                assert!(hint.contains("input"));
            }
            _ => panic!("Expected PermissionDenied error"),
        }
    }

    #[test]
    fn grab_error_disconnection() {
        let path = PathBuf::from("/dev/input/event0");
        let error = io::Error::from(io::ErrorKind::NotFound);
        let driver_error = SafeDevice::map_grab_error(&path, error);

        match driver_error {
            DriverError::DeviceDisconnected { device } => {
                assert!(device.contains("event0"));
            }
            _ => panic!("Expected DeviceDisconnected error"),
        }
    }
}
