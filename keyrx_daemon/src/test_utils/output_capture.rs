//! Output capture for reading daemon's virtual keyboard events.
//!
//! This module provides [`OutputCapture`] for finding and reading events
//! from the daemon's virtual output keyboard device.

use super::VirtualDeviceError;

/// Captures output events from the daemon's virtual keyboard.
///
/// Finds and opens the daemon's output device by name, then provides
/// methods for reading events with timeout handling.
///
/// # Example
///
/// ```ignore
/// let capture = OutputCapture::find_by_name("keyrx Virtual Keyboard", Duration::from_secs(5))?;
/// let events = capture.collect_events(Duration::from_millis(100))?;
/// ```
pub struct OutputCapture {
    // Will hold the evdev device handle
    #[allow(dead_code)]
    name: String,
}

impl OutputCapture {
    /// Finds and opens an output device by name.
    ///
    /// Polls for the device existence until found or timeout expires.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the device to find
    /// * `timeout` - Maximum time to wait for the device
    ///
    /// # Returns
    ///
    /// An `OutputCapture` instance connected to the device, or an error.
    ///
    /// # Errors
    ///
    /// - [`VirtualDeviceError::NotFound`] if device not found within timeout
    /// - [`VirtualDeviceError::PermissionDenied`] if device is not accessible
    #[allow(dead_code)]
    pub fn find_by_name(
        _name: &str,
        _timeout: std::time::Duration,
    ) -> Result<Self, VirtualDeviceError> {
        // TODO: Implement in Task 5
        Err(VirtualDeviceError::creation_failed("not yet implemented"))
    }
}
