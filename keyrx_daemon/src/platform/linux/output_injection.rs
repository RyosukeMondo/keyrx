//! Linux output injection using uinput.
//!
//! This module provides keyboard event injection via virtual uinput devices.

use std::collections::HashSet;

use uinput::Device as UInputDevice;

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::event::KeyEvent;

use crate::platform::{DeviceError, OutputDevice};

use super::keycode_map::keycode_to_uinput_key;
use super::uinput_device;

/// Virtual keyboard device for injecting keyboard events via uinput.
///
/// `UinputOutput` creates a virtual keyboard device using the Linux uinput
/// kernel interface. Events injected through this device appear to applications
/// as if they came from a real keyboard.
///
/// # Device Access
///
/// The uinput device is accessed via `/dev/uinput`. By default, this requires
/// root access or membership in the `uinput` group with appropriate udev rules.
///
/// # Udev Rules Setup
///
/// To allow non-root access, create `/etc/udev/rules.d/99-keyrx.rules`:
/// ```text
/// KERNEL=="uinput", MODE="0660", GROUP="uinput", OPTIONS+="static_node=uinput"
/// ```
///
/// Then add your user to the `uinput` group:
/// ```sh
/// sudo groupadd -f uinput
/// sudo usermod -aG uinput $USER
/// # Log out and back in for changes to take effect
/// ```
///
/// # Example
///
/// ```no_run
/// use keyrx_daemon::platform::linux::UinputOutput;
/// use keyrx_daemon::platform::OutputDevice;
/// use keyrx_core::runtime::event::KeyEvent;
/// use keyrx_core::config::KeyCode;
///
/// // Create virtual keyboard
/// let mut output = UinputOutput::create("keyrx")?;
///
/// // Inject a key press/release sequence
/// output.inject_event(KeyEvent::Press(KeyCode::A))?;
/// output.inject_event(KeyEvent::Release(KeyCode::A))?;
/// # Ok::<(), keyrx_daemon::platform::DeviceError>(())
/// ```
pub struct UinputOutput {
    /// The underlying uinput device handle.
    /// Wrapped in Option to allow taking ownership during destroy().
    device: Option<UInputDevice>,
    /// Name of the virtual device for identification.
    name: String,
    /// Set of currently held (pressed but not yet released) keys.
    /// Used during cleanup to release any keys still held when the device is destroyed.
    held_keys: HashSet<KeyCode>,
}

impl UinputOutput {
    /// Creates a new virtual keyboard device with the specified name.
    ///
    /// The device is configured with full keyboard capabilities (all KEY_* events)
    /// and will appear in `/dev/input/` once created.
    ///
    /// # Arguments
    ///
    /// * `name` - Name for the virtual device (visible in device listings)
    ///
    /// # Returns
    ///
    /// * `Ok(UinputOutput)` - Successfully created the virtual device
    /// * `Err(DeviceError::PermissionDenied)` - Cannot access /dev/uinput
    /// * `Err(DeviceError::Io)` - Other I/O error during device creation
    ///
    /// # Permissions
    ///
    /// Creating a uinput device typically requires:
    /// - Running as root, OR
    /// - Membership in the `uinput` group, OR
    /// - Appropriate udev rules granting access to /dev/uinput
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_daemon::platform::linux::UinputOutput;
    /// use keyrx_daemon::platform::DeviceError;
    ///
    /// match UinputOutput::create("my-virtual-keyboard") {
    ///     Ok(device) => println!("Created virtual device: {}", device.name()),
    ///     Err(DeviceError::PermissionDenied(msg)) => {
    ///         eprintln!("Permission denied: {}", msg);
    ///         eprintln!("Try: sudo groupadd -f uinput && sudo usermod -aG uinput $USER");
    ///     }
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub fn create(name: &str) -> Result<Self, DeviceError> {
        // Full EV_KEY + EV_REL capability range (see `uinput_device` docs):
        // a grabbed keyboard can carry vendor/consumer keys or, if it
        // doubles as a pointer, relative motion, and none of that may be
        // silently dropped just because it has no `KeyCode` (H6).
        let device = uinput_device::create_full_capability_device(name).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                DeviceError::PermissionDenied(
                    "cannot access /dev/uinput: permission denied.\n\
                    To fix this, either:\n\
                    1. Run as root, OR\n\
                    2. Create udev rules:\n\
                       echo 'KERNEL==\"uinput\", MODE=\"0660\", GROUP=\"uinput\"' | \\\n\
                       sudo tee /etc/udev/rules.d/99-keyrx.rules\n\
                       sudo groupadd -f uinput\n\
                       sudo usermod -aG uinput $USER\n\
                       (log out and back in)"
                        .to_string(),
                )
            } else {
                DeviceError::Io(std::io::Error::other(format!(
                    "failed to create uinput device: {}",
                    e
                )))
            }
        })?;

        Ok(Self {
            device: Some(device),
            name: name.to_string(),
            held_keys: HashSet::new(),
        })
    }

    /// Returns the name of the virtual device.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_daemon::platform::linux::UinputOutput;
    ///
    /// let output = UinputOutput::create("keyrx-virtual-keyboard")?;
    /// assert_eq!(output.name(), "keyrx-virtual-keyboard");
    /// # Ok::<(), keyrx_daemon::platform::DeviceError>(())
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether the device has been destroyed.
    ///
    /// After calling `destroy()`, this returns `true` and subsequent operations
    /// will fail with `DeviceError::InjectionFailed`.
    #[must_use]
    pub fn is_destroyed(&self) -> bool {
        self.device.is_none()
    }

    /// Returns the set of currently held keys.
    ///
    /// This is primarily useful for debugging and testing. Keys are tracked
    /// during `inject_event()` calls: Press adds keys, Release removes them.
    #[must_use]
    pub fn held_keys(&self) -> &HashSet<KeyCode> {
        &self.held_keys
    }

    /// Destroys the virtual device, releasing any held keys first.
    ///
    /// This method performs cleanup in the following order:
    /// 1. Releases any keys that are currently held (pressed but not released)
    /// 2. Removes the virtual device from the system
    ///
    /// After calling this method, the device is no longer usable and any
    /// subsequent `inject_event()` calls will return an error.
    ///
    /// # Automatic Cleanup
    ///
    /// This method is called automatically when the `UinputOutput` is dropped,
    /// ensuring proper cleanup even if the caller forgets to call `destroy()`
    /// or if the program panics.
    ///
    /// # Errors
    ///
    /// - `DeviceError::InjectionFailed`: Failed to release a held key
    ///
    /// Note: Errors during cleanup are logged but do not prevent the device
    /// from being destroyed. The device will be removed regardless.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use keyrx_daemon::platform::linux::UinputOutput;
    /// use keyrx_daemon::platform::OutputDevice;
    /// use keyrx_core::runtime::event::KeyEvent;
    /// use keyrx_core::config::KeyCode;
    ///
    /// let mut output = UinputOutput::create("keyrx")?;
    ///
    /// // Press some keys
    /// output.inject_event(KeyEvent::Press(KeyCode::A))?;
    ///
    /// // Destroy will release A before removing the device
    /// output.destroy()?;
    ///
    /// // Device is no longer usable
    /// assert!(output.is_destroyed());
    /// # Ok::<(), keyrx_daemon::platform::DeviceError>(())
    /// ```
    pub fn destroy(&mut self) -> Result<(), DeviceError> {
        // Take ownership of the device (if not already destroyed)
        let Some(device) = self.device.take() else {
            // Already destroyed, nothing to do
            return Ok(());
        };

        // Collect held keys to release (clone to avoid borrow issues)
        let keys_to_release: Vec<KeyCode> = self.held_keys.iter().copied().collect();

        // Release any held keys before destroying
        // We need to temporarily put the device back to use it
        self.device = Some(device);

        for keycode in keys_to_release {
            let key = keycode_to_uinput_key(keycode);

            // Try to release the key, log errors but continue cleanup
            if let Some(ref mut dev) = self.device {
                if let Err(e) = dev.release(&key) {
                    // Log at debug level - cleanup errors shouldn't be fatal
                    log::debug!("Failed to release key {:?} during cleanup: {}", keycode, e);
                }
                // Try to synchronize after each release
                if let Err(e) = dev.synchronize() {
                    log::debug!("Failed to synchronize after releasing {:?}: {}", keycode, e);
                }
            }
        }

        // Clear the held keys set
        self.held_keys.clear();

        // Now take the device again to let it be dropped (which calls UI_DEV_DESTROY)
        let _ = self.device.take();

        Ok(())
    }
}

/// OutputDevice trait implementation for UinputOutput.
///
/// Enables keyboard event injection to the system via a virtual uinput device.
///
/// # Event Injection
///
/// Each event is converted to the appropriate uinput key and injected with:
/// - `Press`: Sends a key down event
/// - `Release`: Sends a key up event
///
/// After each event, `synchronize()` is called to ensure the event is delivered
/// immediately to applications.
///
/// # Example
///
/// ```no_run
/// use keyrx_daemon::platform::linux::UinputOutput;
/// use keyrx_daemon::platform::OutputDevice;
/// use keyrx_core::runtime::event::KeyEvent;
/// use keyrx_core::config::KeyCode;
///
/// let mut output = UinputOutput::create("keyrx")?;
///
/// // Type "ab" by pressing and releasing each key
/// output.inject_event(KeyEvent::Press(KeyCode::A))?;
/// output.inject_event(KeyEvent::Release(KeyCode::A))?;
/// output.inject_event(KeyEvent::Press(KeyCode::B))?;
/// output.inject_event(KeyEvent::Release(KeyCode::B))?;
/// # Ok::<(), keyrx_daemon::platform::DeviceError>(())
/// ```
impl OutputDevice for UinputOutput {
    /// Injects a keyboard event into the virtual device.
    ///
    /// # Arguments
    ///
    /// * `event` - The keyboard event to inject (Press or Release)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Event successfully injected
    /// * `Err(DeviceError::InjectionFailed)` - Failed to inject the event or device destroyed
    ///
    /// # Key Tracking
    ///
    /// This method tracks which keys are currently held (pressed but not released).
    /// This information is used during `destroy()` to release any held keys before
    /// removing the virtual device.
    ///
    /// # Synchronization
    ///
    /// Each event is followed by a `synchronize()` call to ensure immediate
    /// delivery. This matches the behavior expected by applications which
    /// typically receive events with EV_SYN/SYN_REPORT markers.
    fn inject_event(&mut self, event: KeyEvent) -> Result<(), DeviceError> {
        // Get a mutable reference to the device, failing if destroyed
        let device = self
            .device
            .as_mut()
            .ok_or_else(|| DeviceError::InjectionFailed("device has been destroyed".to_string()))?;

        let keycode = event.keycode();
        let key = keycode_to_uinput_key(keycode);

        if event.is_press() {
            device
                .press(&key)
                .map_err(|e| DeviceError::InjectionFailed(format!("failed to press key: {}", e)))?;
            // Track this key as held
            self.held_keys.insert(keycode);
        } else {
            device.release(&key).map_err(|e| {
                DeviceError::InjectionFailed(format!("failed to release key: {}", e))
            })?;
            // Remove from held keys
            self.held_keys.remove(&keycode);
        }

        // Synchronize to ensure event is delivered immediately
        device.synchronize().map_err(|e| {
            DeviceError::InjectionFailed(format!("failed to synchronize events: {}", e))
        })?;

        Ok(())
    }
}

impl UinputOutput {
    /// Forwards a captured event this daemon could not turn into a
    /// `KeyCode`-based [`KeyEvent`] straight to the output device,
    /// unchanged (H6): an `EV_KEY` code with no `KeyCode` (brightness,
    /// mic-mute, vendor keys) or a non-key event (e.g. `EV_REL` from a
    /// keyboard that doubles as a pointer). The device is created with
    /// every `EV_KEY`/`EV_REL` capability enabled (see
    /// [`super::uinput_device`]), so this does not fail just because the
    /// code was never registered.
    ///
    /// Unlike [`inject_event`](OutputDevice::inject_event), this does not
    /// track `held_keys`: a raw code has no `KeyCode` to track or release
    /// by. A raw key left physically held across a config swap is a known,
    /// accepted gap - vendor/media keys are normally momentary, unlike the
    /// mapped keys `held_keys` exists for.
    pub fn inject_raw(
        &mut self,
        event_type: u16,
        code: u16,
        value: i32,
    ) -> Result<(), DeviceError> {
        let device = self
            .device
            .as_mut()
            .ok_or_else(|| DeviceError::InjectionFailed("device has been destroyed".to_string()))?;

        uinput_device::write_raw(device, event_type, code, value).map_err(|e| {
            DeviceError::InjectionFailed(format!("failed to forward raw event: {}", e))
        })
    }
}

/// Drop implementation to ensure automatic cleanup.
///
/// When a `UinputOutput` is dropped (goes out of scope, or program panics),
/// this implementation ensures that:
/// 1. Any held keys are released
/// 2. The virtual device is properly destroyed via `UI_DEV_DESTROY` ioctl
///
/// This prevents orphaned virtual devices in `/dev/input/` and ensures that
/// applications don't see stuck keys after the daemon exits.
impl Drop for UinputOutput {
    fn drop(&mut self) {
        // Call destroy to release held keys and cleanup
        // Errors during drop are logged but cannot be propagated
        if let Err(e) = self.destroy() {
            log::warn!("Error during UinputOutput cleanup: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;

    /// Checks if input devices are accessible for reading.
    fn can_access_input_devices() -> bool {
        for i in 0..20 {
            let path = format!("/dev/input/event{}", i);
            if OpenOptions::new().read(true).open(&path).is_ok() {
                return true;
            }
        }
        false
    }

    /// Checks if uinput and input devices are accessible.
    fn can_access_uinput() -> bool {
        let uinput_ok = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/uinput")
            .is_ok();
        uinput_ok && can_access_input_devices()
    }

    /// Test that UinputOutput::create returns PermissionDenied when not running as root
    /// Note: This test only works when NOT running as root without udev rules
    #[test]
    #[ignore = "requires non-root user without uinput access - run manually"]
    fn test_uinputoutput_create_permission_denied() {
        let result = UinputOutput::create("test-keyboard");

        match result {
            Err(DeviceError::PermissionDenied(msg)) => {
                assert!(msg.contains("permission denied"));
                assert!(msg.contains("udev rules") || msg.contains("uinput"));
            }
            Ok(_) => {
                // We have permission, test is inconclusive
                println!("User has uinput access, cannot test permission denied");
            }
            Err(e) => panic!("Expected PermissionDenied or Ok, got {:?}", e),
        }
    }

    /// Test that UinputOutput::create creates a virtual device
    /// Note: Requires root or udev rules for uinput access
    #[test]
    fn test_uinputoutput_create() {
        if !can_access_uinput() {
            eprintln!("SKIPPED: uinput/input not accessible");
            return;
        }
        let output =
            UinputOutput::create("keyrx-test-keyboard").expect("Failed to create uinput device");

        assert_eq!(output.name(), "keyrx-test-keyboard");
        assert!(!output.is_destroyed());
        assert!(output.held_keys().is_empty());
    }

    /// Test that inject_event fails after destroy
    /// Note: Requires uinput access
    #[test]
    fn test_uinputoutput_inject_after_destroy_fails() {
        if !can_access_uinput() {
            eprintln!("SKIPPED: uinput/input not accessible");
            return;
        }
        let mut output =
            UinputOutput::create("keyrx-test-inject-fail").expect("Failed to create uinput device");

        // Destroy the device
        output.destroy().expect("Failed to destroy device");
        assert!(output.is_destroyed());

        // Try to inject after destroy - should fail
        let result = output.inject_event(KeyEvent::Press(KeyCode::A));
        assert!(result.is_err());
        match result {
            Err(DeviceError::InjectionFailed(msg)) => {
                assert!(msg.contains("destroyed"));
            }
            Err(e) => panic!("Expected InjectionFailed, got {:?}", e),
            Ok(_) => panic!("Expected error, got Ok"),
        }
    }

    /// Test that held_keys tracks pressed keys
    /// Note: Requires uinput access
    #[test]
    fn test_uinputoutput_held_keys_tracking() {
        if !can_access_uinput() {
            eprintln!("SKIPPED: uinput/input not accessible");
            return;
        }
        let mut output =
            UinputOutput::create("keyrx-test-held").expect("Failed to create uinput device");

        // Initially no held keys
        assert!(output.held_keys().is_empty());

        // Press A
        output
            .inject_event(KeyEvent::Press(KeyCode::A))
            .expect("Failed to press A");
        assert!(output.held_keys().contains(&KeyCode::A));
        assert_eq!(output.held_keys().len(), 1);

        // Press B
        output
            .inject_event(KeyEvent::Press(KeyCode::B))
            .expect("Failed to press B");
        assert!(output.held_keys().contains(&KeyCode::A));
        assert!(output.held_keys().contains(&KeyCode::B));
        assert_eq!(output.held_keys().len(), 2);

        // Release A
        output
            .inject_event(KeyEvent::Release(KeyCode::A))
            .expect("Failed to release A");
        assert!(!output.held_keys().contains(&KeyCode::A));
        assert!(output.held_keys().contains(&KeyCode::B));
        assert_eq!(output.held_keys().len(), 1);

        // Release B
        output
            .inject_event(KeyEvent::Release(KeyCode::B))
            .expect("Failed to release B");
        assert!(output.held_keys().is_empty());
    }

    /// Test that destroy releases held keys
    /// Note: Requires uinput access
    #[test]
    fn test_uinputoutput_destroy_releases_held_keys() {
        if !can_access_uinput() {
            eprintln!("SKIPPED: uinput/input not accessible");
            return;
        }
        let mut output =
            UinputOutput::create("keyrx-test-destroy").expect("Failed to create uinput device");

        // Press some keys but don't release
        output
            .inject_event(KeyEvent::Press(KeyCode::A))
            .expect("Failed to press A");
        output
            .inject_event(KeyEvent::Press(KeyCode::LShift))
            .expect("Failed to press LShift");

        assert_eq!(output.held_keys().len(), 2);

        // Destroy should clear held keys
        output.destroy().expect("Failed to destroy device");

        assert!(output.is_destroyed());
        assert!(output.held_keys().is_empty());
    }

    /// Test that calling destroy twice is safe
    /// Note: Requires uinput access
    #[test]
    fn test_uinputoutput_destroy_twice_is_safe() {
        if !can_access_uinput() {
            eprintln!("SKIPPED: uinput/input not accessible");
            return;
        }
        let mut output = UinputOutput::create("keyrx-test-destroy-twice")
            .expect("Failed to create uinput device");

        // First destroy
        output.destroy().expect("First destroy failed");
        assert!(output.is_destroyed());

        // Second destroy should be a no-op
        output.destroy().expect("Second destroy failed");
        assert!(output.is_destroyed());
    }

    /// Test that Drop trait calls destroy
    /// Note: Requires uinput access
    #[test]
    fn test_uinputoutput_drop_calls_destroy() {
        if !can_access_uinput() {
            eprintln!("SKIPPED: uinput/input not accessible");
            return;
        }
        // Create a device with some held keys
        {
            let mut output =
                UinputOutput::create("keyrx-test-drop").expect("Failed to create uinput device");

            output
                .inject_event(KeyEvent::Press(KeyCode::A))
                .expect("Failed to press A");

            // Drop will be called at end of scope
        }

        // If we get here without panic, Drop worked correctly
        // We can't easily verify the device was destroyed, but no panic is good
    }

    /// Test event injection for various key types
    /// Note: Requires uinput access
    #[test]
    fn test_uinputoutput_inject_various_key_types() {
        if !can_access_uinput() {
            eprintln!("SKIPPED: uinput/input not accessible");
            return;
        }
        let mut output =
            UinputOutput::create("keyrx-test-inject-keys").expect("Failed to create uinput device");

        // Test letter keys
        output
            .inject_event(KeyEvent::Press(KeyCode::A))
            .expect("Failed to press A");
        output
            .inject_event(KeyEvent::Release(KeyCode::A))
            .expect("Failed to release A");

        // Test number keys
        output
            .inject_event(KeyEvent::Press(KeyCode::Num5))
            .expect("Failed to press Num5");
        output
            .inject_event(KeyEvent::Release(KeyCode::Num5))
            .expect("Failed to release Num5");

        // Test function keys
        output
            .inject_event(KeyEvent::Press(KeyCode::F1))
            .expect("Failed to press F1");
        output
            .inject_event(KeyEvent::Release(KeyCode::F1))
            .expect("Failed to release F1");

        // Test modifier keys
        output
            .inject_event(KeyEvent::Press(KeyCode::LShift))
            .expect("Failed to press LShift");
        output
            .inject_event(KeyEvent::Release(KeyCode::LShift))
            .expect("Failed to release LShift");

        // All held keys should be released
        assert!(output.held_keys().is_empty());
    }

    /// Test that multiple UinputOutput devices can coexist
    /// Note: Requires uinput access
    #[test]
    fn test_uinputoutput_multiple_devices() {
        if !can_access_uinput() {
            eprintln!("SKIPPED: uinput/input not accessible");
            return;
        }
        let output1 = UinputOutput::create("keyrx-test-multi-1")
            .expect("Failed to create first uinput device");
        let output2 = UinputOutput::create("keyrx-test-multi-2")
            .expect("Failed to create second uinput device");

        assert_eq!(output1.name(), "keyrx-test-multi-1");
        assert_eq!(output2.name(), "keyrx-test-multi-2");

        assert!(!output1.is_destroyed());
        assert!(!output2.is_destroyed());

        // Both should be independently usable

        drop(output1);
        drop(output2);
    }
}
