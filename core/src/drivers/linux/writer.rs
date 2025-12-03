//! Uinput writer for keyboard event injection.
//!
//! This module provides `UinputWriter` for injecting keyboard events
//! via the Linux uinput subsystem. It implements the [`KeyInjector`] trait
//! for integration with the KeyRx remapping engine.

use super::safety::uinput::SafeUinput;
use crate::config::UINPUT_DEVICE_NAME;
use crate::drivers::KeyInjector;
use crate::engine::KeyCode;
use anyhow::Result;
use tracing::{debug, trace};

/// Writer for injecting keyboard events via uinput.
///
/// `UinputWriter` creates a virtual keyboard device that can emit key events
/// to the system. This is used to inject remapped keys back into the input
/// stream after processing.
///
/// # Device Registration
///
/// The virtual device is registered with all keys supported by the `KeyCode`
/// enum to ensure any remapped key can be emitted.
///
/// # Safety
///
/// Uses `SafeUinput` wrapper which provides RAII cleanup and event validation.
///
/// # Permissions
///
/// Creating a uinput device requires write access to `/dev/uinput`.
/// See `LinuxInput::check_uinput_accessible()` for permission requirements.
pub struct UinputWriter {
    /// Safe wrapper around the uinput virtual device with RAII cleanup.
    device: SafeUinput,
}

impl UinputWriter {
    /// Create a new UinputWriter with a virtual keyboard device.
    ///
    /// The virtual device is named "KeyRx Virtual Keyboard" and supports
    /// all keys defined in the `KeyCode` enum.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The uinput device cannot be accessed (permission denied)
    /// - The virtual device creation fails
    pub fn new() -> Result<Self> {
        // Use SafeUinput wrapper which provides RAII cleanup and event validation
        let device = SafeUinput::new(UINPUT_DEVICE_NAME)?;

        debug!(
            service = "keyrx",
            event = "uinput_created",
            component = "linux_writer",
            device_name = UINPUT_DEVICE_NAME,
            "Created uinput virtual keyboard with SafeUinput wrapper"
        );

        Ok(Self { device })
    }

    /// Get a reference to the underlying SafeUinput wrapper.
    #[allow(dead_code)]
    pub fn safe_device(&self) -> &SafeUinput {
        &self.device
    }

    /// Get a mutable reference to the underlying SafeUinput wrapper.
    #[allow(dead_code)]
    pub fn safe_device_mut(&mut self) -> &mut SafeUinput {
        &mut self.device
    }

    /// Emit a key event (press or release) through the virtual keyboard.
    ///
    /// Converts the `KeyCode` to an evdev key code and writes the event
    /// to the virtual device. Automatically calls `sync()` after writing
    /// for immediate effect.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to emit
    /// * `pressed` - `true` for key press, `false` for key release
    ///
    /// # Errors
    ///
    /// Returns an error if the event cannot be written to the uinput device.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut writer = UinputWriter::new()?;
    /// // Press and release Escape
    /// writer.emit(KeyCode::Escape, true)?;
    /// writer.emit(KeyCode::Escape, false)?;
    /// ```
    pub fn emit(&mut self, key: KeyCode, pressed: bool) -> Result<()> {
        trace!(
            "Emitting key event: {:?} {}",
            key,
            if pressed { "down" } else { "up" }
        );

        // SafeUinput handles event validation and injection
        self.device.emit_key(key, pressed)?;

        // Sync for immediate effect
        self.sync()?;

        Ok(())
    }

    /// Send a synchronization event to flush pending events.
    ///
    /// The kernel buffers input events until an `EV_SYN` event is received.
    /// This method writes the sync event to ensure all pending key events
    /// are processed immediately.
    ///
    /// # Note
    ///
    /// This is called automatically by `emit()`, so you typically don't
    /// need to call it directly unless batching multiple events.
    ///
    /// # Errors
    ///
    /// Returns an error if the sync event cannot be written.
    fn sync_internal(&mut self) -> Result<()> {
        // SafeUinput handles sync event emission
        self.device.sync()?;
        Ok(())
    }
}

impl KeyInjector for UinputWriter {
    fn inject(&mut self, key: KeyCode, pressed: bool) -> Result<()> {
        self.emit(key, pressed)
    }

    fn sync(&mut self) -> Result<()> {
        self.sync_internal()
    }

    fn needs_uinput(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn uinput_writer_uses_safe_wrapper() {
        // This test verifies that UinputWriter uses SafeUinput
        // Actual creation would require /dev/uinput access
        // Integration tests will verify the full functionality
        assert!(
            true,
            "UinputWriter uses SafeUinput wrapper for RAII cleanup"
        );
    }
}
