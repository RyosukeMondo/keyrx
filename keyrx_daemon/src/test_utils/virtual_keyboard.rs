//! Virtual keyboard for test input injection.
//!
//! This module provides [`VirtualKeyboard`] for creating a virtual input device
//! that can inject key events into the kernel for E2E testing.
//!
//! # Usage
//!
//! ```ignore
//! use keyrx_daemon::test_utils::VirtualKeyboard;
//! use keyrx_core::runtime::event::KeyEvent;
//! use keyrx_core::config::KeyCode;
//!
//! // Create a virtual keyboard with unique name
//! let mut keyboard = VirtualKeyboard::create("test")?;
//!
//! // Inject key events
//! keyboard.inject(KeyEvent::Press(KeyCode::A))?;
//! keyboard.inject(KeyEvent::Release(KeyCode::A))?;
//!
//! // Device is automatically cleaned up on drop
//! ```
//!
//! # Requirements
//!
//! - Linux with uinput support (`/dev/uinput`)
//! - Read/write access to uinput (typically requires `input` group membership or root)

use std::time::SystemTime;

#[cfg(target_os = "linux")]
use uinput::Device as UInputDevice;

#[cfg(target_os = "linux")]
use crate::platform::linux::keycode_to_uinput_key;
#[cfg(target_os = "windows")]
use crate::platform::windows::keycode::keycode_to_vk;
use crate::test_utils::VirtualDeviceError;
use keyrx_core::config::KeyCode;
use keyrx_core::runtime::event::KeyEvent;

#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

/// Markers for events during E2E testing
#[cfg(target_os = "windows")]
const TEST_SIMULATED_PHYSICAL_MARKER: usize = 0x54455354; // "TEST"

/// A virtual keyboard device for injecting test key events.
///
/// Uses Linux's uinput subsystem to create a virtual input device that appears
/// to the system as a real keyboard. Events injected through this device flow
/// through the kernel's input subsystem and can be captured by applications
/// (including the keyrx daemon for E2E testing).
///
/// # Device Naming
///
/// Each virtual keyboard is created with a unique name that includes:
/// - A user-provided base name
/// - A timestamp-based suffix for uniqueness
///
/// This ensures parallel tests can each create their own virtual keyboard
/// without name collisions.
///
/// # Cleanup
///
/// The device is automatically destroyed when the `VirtualKeyboard` is dropped,
/// ensuring no orphaned virtual devices are left in `/dev/input/`.
///
/// # Example
///
/// ```ignore
/// let mut keyboard = VirtualKeyboard::create("e2e-test")?;
/// println!("Created: {}", keyboard.name());
///
/// // Inject a key tap (press + release)
/// keyboard.inject(KeyEvent::Press(KeyCode::A))?;
/// keyboard.inject(KeyEvent::Release(KeyCode::A))?;
/// ```
pub struct VirtualKeyboard {
    /// The underlying uinput device handle.
    /// Wrapped in Option to allow taking ownership during destroy.
    #[cfg(target_os = "linux")]
    device: Option<UInputDevice>,
    /// Full name of the virtual device (includes unique suffix).
    name: String,
}

impl VirtualKeyboard {
    /// Creates a new virtual keyboard with a unique name.
    ///
    /// The actual device name will include a timestamp-based suffix to allow
    /// parallel test execution without name collisions.
    ///
    /// # Arguments
    ///
    /// * `base_name` - Base name for the virtual device (will have suffix appended)
    ///
    /// # Returns
    ///
    /// A new `VirtualKeyboard` instance, or an error if creation fails.
    ///
    /// # Errors
    ///
    /// - [`VirtualDeviceError::PermissionDenied`] if uinput is not accessible
    /// - [`VirtualDeviceError::CreationFailed`] if device creation fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let keyboard = VirtualKeyboard::create("my-test")?;
    /// // Device name will be something like "my-test-1703456789123"
    /// println!("Created: {}", keyboard.name());
    /// ```
    pub fn create(base_name: &str) -> Result<Self, VirtualDeviceError> {
        // Generate a unique name with timestamp suffix
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        let unique_name = format!("{}-{}", base_name, timestamp);

        #[cfg(target_os = "linux")]
        {
            // Create uinput device with keyboard capabilities
            let device = uinput::default()
                .map_err(|e| {
                    let err_str = e.to_string();
                    if err_str.contains("Permission denied") || err_str.contains("EACCES") {
                        VirtualDeviceError::uinput_permission_denied()
                    } else {
                        VirtualDeviceError::CreationFailed {
                            message: format!("failed to open uinput: {}", e),
                        }
                    }
                })?
                .name(&unique_name)
                .map_err(|e| {
                    VirtualDeviceError::creation_failed(format!("failed to set name: {}", e))
                })?
                // Enable all keyboard events for full capability
                .event(uinput::event::Keyboard::All)
                .map_err(|e| {
                    VirtualDeviceError::creation_failed(format!(
                        "failed to enable keyboard events: {}",
                        e
                    ))
                })?
                .create()
                .map_err(|e| {
                    VirtualDeviceError::creation_failed(format!("failed to create device: {}", e))
                })?;

            Ok(Self {
                device: Some(device),
                name: unique_name,
            })
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, we don't need to create a device node,
            // we'll just use SendInput with a special marker.
            Ok(Self { name: unique_name })
        }
    }

    /// Returns the full name of the virtual device.
    ///
    /// This includes the unique suffix appended during creation.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let keyboard = VirtualKeyboard::create("test")?;
    /// println!("Device name: {}", keyboard.name());
    /// // Prints something like: "test-1703456789123"
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether the device has been destroyed.
    ///
    /// After calling `destroy()` or when the device is dropped, this returns `true`.
    #[must_use]
    pub fn is_destroyed(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            self.device.is_none()
        }
        #[cfg(target_os = "windows")]
        {
            false
        }
    }

    /// Destroys the virtual device.
    ///
    /// This method removes the virtual device from the system. After calling,
    /// any subsequent operations will fail.
    ///
    /// This is called automatically when the `VirtualKeyboard` is dropped.
    ///
    /// # Errors
    ///
    /// Returns an error if the device was already destroyed.
    pub fn destroy(&mut self) -> Result<(), VirtualDeviceError> {
        #[cfg(target_os = "linux")]
        {
            if self.device.take().is_none() {
                return Err(VirtualDeviceError::creation_failed(
                    "device already destroyed",
                ));
            }
        }
        Ok(())
    }

    /// Injects a single key event into the virtual keyboard.
    ///
    /// The event is written to the uinput device and will be received by
    /// any application (including the keyrx daemon) that is reading from
    /// the device.
    ///
    /// # Arguments
    ///
    /// * `event` - The key event to inject (Press or Release)
    ///
    /// # Errors
    ///
    /// - [`VirtualDeviceError::CreationFailed`] if the device has been destroyed
    /// - [`VirtualDeviceError::Io`] if the write operation fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut keyboard = VirtualKeyboard::create("test")?;
    ///
    /// // Inject a key press
    /// keyboard.inject(KeyEvent::Press(KeyCode::A))?;
    ///
    /// // Inject a key release
    /// keyboard.inject(KeyEvent::Release(KeyCode::A))?;
    /// ```
    pub fn inject(&mut self, event: KeyEvent) -> Result<(), VirtualDeviceError> {
        #[cfg(target_os = "linux")]
        {
            let device = self
                .device
                .as_mut()
                .ok_or_else(|| VirtualDeviceError::creation_failed("device has been destroyed"))?;

            let key = keycode_to_uinput_key(event.keycode());

            if event.is_press() {
                device.press(&key).map_err(|e| {
                    VirtualDeviceError::Io(std::io::Error::other(format!("press failed: {}", e)))
                })?;
            } else {
                device.release(&key).map_err(|e| {
                    VirtualDeviceError::Io(std::io::Error::other(format!("release failed: {}", e)))
                })?;
            }

            // Synchronize to ensure the event is delivered immediately
            device.synchronize().map_err(|e| {
                VirtualDeviceError::Io(std::io::Error::other(format!("synchronize failed: {}", e)))
            })?;
        }

        #[cfg(target_os = "windows")]
        {
            let keycode = event.keycode();
            let is_release = event.is_release();

            let vk = keycode_to_vk(keycode).ok_or_else(|| {
                VirtualDeviceError::creation_failed(format!("Unmapped keycode: {:?}", keycode))
            })?;

            unsafe {
                let mut input = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: std::mem::zeroed(),
                };

                input.Anonymous.ki = KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: if is_release { KEYEVENTF_KEYUP } else { 0 },
                    time: 0,
                    dwExtraInfo: TEST_SIMULATED_PHYSICAL_MARKER,
                };

                // Set extended key flag for certain keys
                if is_extended_key(vk) {
                    input.Anonymous.ki.dwFlags |= KEYEVENTF_EXTENDEDKEY;
                }

                if SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) == 0 {
                    return Err(VirtualDeviceError::Io(std::io::Error::other(
                        "SendInput failed",
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub fn is_extended_key(vk: u16) -> bool {
    matches!(
        vk,
        VK_RMENU
            | VK_RCONTROL
            | VK_INSERT
            | VK_DELETE
            | VK_HOME
            | VK_END
            | VK_PRIOR
            | VK_NEXT
            | VK_LEFT
            | VK_RIGHT
            | VK_UP
            | VK_DOWN
            | VK_NUMLOCK
            | VK_SNAPSHOT
    )
}

impl VirtualKeyboard {
    /// Injects a sequence of key events with optional delay between them.
    ///
    /// This is a convenience method for injecting multiple events. If `delay`
    /// is `Some`, the method will sleep between each event.
    ///
    /// # Arguments
    ///
    /// * `events` - Slice of key events to inject in order
    /// * `delay` - Optional delay between events (None for no delay)
    ///
    /// # Errors
    ///
    /// Returns an error if any event injection fails. Events before the
    /// failure will have been injected.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::time::Duration;
    ///
    /// let mut keyboard = VirtualKeyboard::create("test")?;
    ///
    /// // Type "ab" with 10ms delay between events
    /// let events = vec![
    ///     KeyEvent::Press(KeyCode::A),
    ///     KeyEvent::Release(KeyCode::A),
    ///     KeyEvent::Press(KeyCode::B),
    ///     KeyEvent::Release(KeyCode::B),
    /// ];
    /// keyboard.inject_sequence(&events, Some(Duration::from_millis(10)))?;
    /// ```
    pub fn inject_sequence(
        &mut self,
        events: &[KeyEvent],
        delay: Option<std::time::Duration>,
    ) -> Result<(), VirtualDeviceError> {
        for (i, event) in events.iter().enumerate() {
            self.inject(event.clone())?;

            // Add delay between events (but not after the last one)
            if let Some(d) = delay {
                if i < events.len() - 1 {
                    std::thread::sleep(d);
                }
            }
        }
        Ok(())
    }

    /// Creates a tap event sequence (press + release) for a key.
    ///
    /// This is a helper method that creates the events needed to simulate
    /// a key tap (pressing and releasing a key).
    ///
    /// # Arguments
    ///
    /// * `keycode` - The key to tap
    ///
    /// # Returns
    ///
    /// A vector containing Press and Release events for the key.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let events = VirtualKeyboard::tap_events(KeyCode::A);
    /// // events = [Press(A), Release(A)]
    /// ```
    #[must_use]
    pub fn tap_events(keycode: KeyCode) -> Vec<KeyEvent> {
        vec![KeyEvent::Press(keycode), KeyEvent::Release(keycode)]
    }
}

impl Drop for VirtualKeyboard {
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        {
            // Take ownership of device to let it be dropped (destroys the uinput device)
            let _ = self.device.take();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tap_events_helper() {
        let events = VirtualKeyboard::tap_events(KeyCode::A);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], KeyEvent::Press(KeyCode::A));
        assert_eq!(events[1], KeyEvent::Release(KeyCode::A));
    }

    #[test]
    fn test_tap_events_modifier() {
        let events = VirtualKeyboard::tap_events(KeyCode::LShift);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], KeyEvent::Press(KeyCode::LShift));
        assert_eq!(events[1], KeyEvent::Release(KeyCode::LShift));
    }

    /// Test that device creation fails gracefully without uinput access
    /// Note: This test is not marked #[ignore] because it tests error handling
    /// which works regardless of uinput access
    #[test]
    fn test_create_generates_unique_names() {
        // We can't test actual device creation without uinput access,
        // but we can verify the name generation logic by attempting
        // creation twice and checking that the error messages contain
        // different timestamps (if we had access) or the same error type

        // For now, just verify the function exists and returns an error type
        let result1 = VirtualKeyboard::create("test");
        // Small delay to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(2));
        let result2 = VirtualKeyboard::create("test");

        // Both should be the same type of result (either Ok or same Err variant)
        match (result1, result2) {
            (Ok(kb1), Ok(kb2)) => {
                // If we have access, names should be different
                assert_ne!(kb1.name(), kb2.name());
            }
            (Err(_), Err(_)) => {
                // Both failed (no uinput access) - this is expected
            }
            _ => {
                // One succeeded, one failed - unexpected
                panic!("Inconsistent results for same create call");
            }
        }
    }

    /// Test that create returns a VirtualKeyboard when uinput is accessible
    #[test]
    fn test_virtual_keyboard_create() {
        crate::skip_if_no_uinput!();
        let keyboard =
            VirtualKeyboard::create("test-create").expect("Failed to create virtual keyboard");

        assert!(keyboard.name().starts_with("test-create-"));
        assert!(!keyboard.is_destroyed());
    }

    /// Test that inject works correctly
    #[test]
    fn test_virtual_keyboard_inject() {
        crate::skip_if_no_uinput!();
        let mut keyboard =
            VirtualKeyboard::create("test-inject").expect("Failed to create virtual keyboard");

        // Inject press and release
        keyboard
            .inject(KeyEvent::Press(KeyCode::A))
            .expect("Failed to inject press");
        keyboard
            .inject(KeyEvent::Release(KeyCode::A))
            .expect("Failed to inject release");
    }

    /// Test that inject_sequence works correctly
    #[test]
    fn test_virtual_keyboard_inject_sequence() {
        crate::skip_if_no_uinput!();
        use std::time::Duration;

        let mut keyboard =
            VirtualKeyboard::create("test-sequence").expect("Failed to create virtual keyboard");

        let events = vec![
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Release(KeyCode::A),
            KeyEvent::Press(KeyCode::B),
            KeyEvent::Release(KeyCode::B),
        ];

        keyboard
            .inject_sequence(&events, Some(Duration::from_millis(1)))
            .expect("Failed to inject sequence");
    }

    /// Test that destroy works and prevents further injection
    #[test]
    fn test_virtual_keyboard_destroy() {
        crate::skip_if_no_uinput!();
        let mut keyboard =
            VirtualKeyboard::create("test-destroy").expect("Failed to create virtual keyboard");

        assert!(!keyboard.is_destroyed());

        keyboard.destroy().expect("Failed to destroy keyboard");
        assert!(keyboard.is_destroyed());

        // Further injection should fail
        let result = keyboard.inject(KeyEvent::Press(KeyCode::A));
        assert!(result.is_err());
    }

    /// Test that Drop cleans up the device
    #[test]
    fn test_virtual_keyboard_drop() {
        crate::skip_if_no_uinput!();
        {
            let _keyboard =
                VirtualKeyboard::create("test-drop").expect("Failed to create virtual keyboard");
            // Device will be dropped here
        }
        // If we get here without panic, drop worked correctly
    }

    /// Test that double destroy is handled gracefully
    #[test]
    fn test_virtual_keyboard_double_destroy() {
        crate::skip_if_no_uinput!();
        let mut keyboard = VirtualKeyboard::create("test-double-destroy")
            .expect("Failed to create virtual keyboard");

        keyboard.destroy().expect("First destroy should succeed");

        // Second destroy should return an error
        let result = keyboard.destroy();
        assert!(result.is_err());
    }

    /// Test device name uniqueness across rapid creates
    #[test]
    fn test_virtual_keyboard_name_uniqueness() {
        crate::skip_if_no_uinput!();
        let kb1 = VirtualKeyboard::create("test-unique").expect("Failed to create first keyboard");
        let kb2 = VirtualKeyboard::create("test-unique").expect("Failed to create second keyboard");

        // Names should be different due to timestamp suffix
        assert_ne!(kb1.name(), kb2.name());
        assert!(kb1.name().starts_with("test-unique-"));
        assert!(kb2.name().starts_with("test-unique-"));
    }
}
