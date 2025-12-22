//! Virtual keyboard for test input injection.
//!
//! This module provides [`VirtualKeyboard`] for creating a virtual input device
//! that can inject key events into the kernel for E2E testing.

use super::VirtualDeviceError;

/// A virtual keyboard device for injecting test key events.
///
/// Uses Linux's uinput subsystem to create a virtual input device that appears
/// to the system as a real keyboard. Events injected through this device flow
/// through the kernel's input subsystem and can be captured by applications.
///
/// # Example
///
/// ```ignore
/// let keyboard = VirtualKeyboard::create("test-keyboard")?;
/// keyboard.inject(KeyEvent::Press(KeyCode::A))?;
/// keyboard.inject(KeyEvent::Release(KeyCode::A))?;
/// ```
pub struct VirtualKeyboard {
    // Will hold the uinput device handle
    #[allow(dead_code)]
    name: String,
}

impl VirtualKeyboard {
    /// Creates a new virtual keyboard with the given name.
    ///
    /// The actual device name will include a unique suffix to allow
    /// parallel test execution.
    ///
    /// # Arguments
    ///
    /// * `name` - Base name for the virtual device
    ///
    /// # Returns
    ///
    /// A new `VirtualKeyboard` instance, or an error if creation fails.
    ///
    /// # Errors
    ///
    /// - [`VirtualDeviceError::PermissionDenied`] if uinput is not accessible
    /// - [`VirtualDeviceError::CreationFailed`] if device creation fails
    #[allow(dead_code)]
    pub fn create(_name: &str) -> Result<Self, VirtualDeviceError> {
        // TODO: Implement in Task 2
        Err(VirtualDeviceError::creation_failed("not yet implemented"))
    }
}
