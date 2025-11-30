//! Uinput writer for keyboard event injection.
//!
//! This module provides `UinputWriter` for injecting keyboard events
//! via the Linux uinput subsystem. It implements the [`KeyInjector`] trait
//! for integration with the KeyRx remapping engine.

use super::keymap::{all_evdev_codes, keycode_to_evdev};
use crate::drivers::KeyInjector;
use crate::engine::KeyCode;
use crate::error::LinuxDriverError;
use anyhow::{Context, Result};
use evdev::{
    uinput::VirtualDeviceBuilder, AttributeSet, EventType, InputEvent as EvdevInputEvent, Key,
};
use tracing::{debug, trace};

const UINPUT_DEVICE_NAME: &str = "KeyRx Virtual Keyboard";

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
/// # Permissions
///
/// Creating a uinput device requires write access to `/dev/uinput`.
/// See `LinuxInput::check_uinput_accessible()` for permission requirements.
pub struct UinputWriter {
    /// The virtual uinput device for key injection.
    device: evdev::uinput::VirtualDevice,
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
        // Build the set of keys to register from the centralized keycodes module
        let keys = Self::build_key_set();

        let device = VirtualDeviceBuilder::new()
            .context("Failed to create VirtualDeviceBuilder")?
            .name(UINPUT_DEVICE_NAME)
            .with_keys(&keys)
            .map_err(|e| LinuxDriverError::uinput_failed(std::io::Error::other(e.to_string())))?
            .build()
            .map_err(|e| LinuxDriverError::uinput_failed(std::io::Error::other(e.to_string())))?;

        debug!("Created uinput virtual keyboard: {}", UINPUT_DEVICE_NAME);

        Ok(Self { device })
    }

    /// Build the set of evdev keys to register with the virtual device.
    ///
    /// Uses `all_evdev_codes()` from the centralized keycodes module
    /// to ensure all supported keys are registered.
    fn build_key_set() -> AttributeSet<Key> {
        let mut keys = AttributeSet::<Key>::new();

        // Use centralized evdev codes from keycodes.rs (SSOT)
        for evdev_code in all_evdev_codes() {
            keys.insert(Key::new(evdev_code));
        }

        keys
    }

    /// Get a reference to the underlying virtual device.
    #[allow(dead_code)]
    pub fn device(&self) -> &evdev::uinput::VirtualDevice {
        &self.device
    }

    /// Get a mutable reference to the underlying virtual device.
    #[allow(dead_code)]
    pub fn device_mut(&mut self) -> &mut evdev::uinput::VirtualDevice {
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
        let evdev_code = keycode_to_evdev(key);
        let value = if pressed { 1 } else { 0 };

        // Create the key event
        // EV_KEY type = 1, the code is the key code, value is 1 (press) or 0 (release)
        let event = EvdevInputEvent::new(EventType::KEY, evdev_code, value);

        trace!(
            "Emitting key event: {:?} {} (evdev code: {})",
            key,
            if pressed { "down" } else { "up" },
            evdev_code
        );

        // Write the event to the virtual device
        self.device
            .emit(&[event])
            .map_err(|e| LinuxDriverError::uinput_failed(std::io::Error::other(e.to_string())))?;

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
        // EV_SYN = 0, SYN_REPORT = 0, value = 0
        let sync_event = EvdevInputEvent::new(EventType::SYNCHRONIZATION, 0, 0);

        self.device
            .emit(&[sync_event])
            .map_err(|e| LinuxDriverError::uinput_failed(std::io::Error::other(e.to_string())))?;

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
}
