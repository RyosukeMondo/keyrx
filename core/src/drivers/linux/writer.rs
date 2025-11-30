//! Uinput writer for keyboard event injection.
//!
//! This module provides `UinputWriter` for injecting keyboard events
//! via the Linux uinput subsystem. It implements the [`KeyInjector`] trait
//! for integration with the KeyRx remapping engine.

use crate::drivers::KeyInjector;
use crate::engine::KeyCode;
use crate::error::LinuxDriverError;
use anyhow::{Context, Result};
use evdev::{
    uinput::VirtualDeviceBuilder, AttributeSet, EventType, InputEvent as EvdevInputEvent, Key,
};
use tracing::{debug, trace};

const UINPUT_DEVICE_NAME: &str = "KeyRx Virtual Keyboard";

/// Convert KeyCode to evdev key code.
/// This uses the engine's KeyCode type directly.
/// TODO: Remove after Task 12 unifies KeyCode types.
fn keycode_to_evdev(key: KeyCode) -> u16 {
    match key {
        KeyCode::Escape => 1,
        KeyCode::Key1 => 2,
        KeyCode::Key2 => 3,
        KeyCode::Key3 => 4,
        KeyCode::Key4 => 5,
        KeyCode::Key5 => 6,
        KeyCode::Key6 => 7,
        KeyCode::Key7 => 8,
        KeyCode::Key8 => 9,
        KeyCode::Key9 => 10,
        KeyCode::Key0 => 11,
        KeyCode::Minus => 12,
        KeyCode::Equal => 13,
        KeyCode::Backspace => 14,
        KeyCode::Tab => 15,
        KeyCode::Q => 16,
        KeyCode::W => 17,
        KeyCode::E => 18,
        KeyCode::R => 19,
        KeyCode::T => 20,
        KeyCode::Y => 21,
        KeyCode::U => 22,
        KeyCode::I => 23,
        KeyCode::O => 24,
        KeyCode::P => 25,
        KeyCode::LeftBracket => 26,
        KeyCode::RightBracket => 27,
        KeyCode::Enter => 28,
        KeyCode::LeftCtrl => 29,
        KeyCode::A => 30,
        KeyCode::S => 31,
        KeyCode::D => 32,
        KeyCode::F => 33,
        KeyCode::G => 34,
        KeyCode::H => 35,
        KeyCode::J => 36,
        KeyCode::K => 37,
        KeyCode::L => 38,
        KeyCode::Semicolon => 39,
        KeyCode::Apostrophe => 40,
        KeyCode::Grave => 41,
        KeyCode::LeftShift => 42,
        KeyCode::Backslash => 43,
        KeyCode::Z => 44,
        KeyCode::X => 45,
        KeyCode::C => 46,
        KeyCode::V => 47,
        KeyCode::B => 48,
        KeyCode::N => 49,
        KeyCode::M => 50,
        KeyCode::Comma => 51,
        KeyCode::Period => 52,
        KeyCode::Slash => 53,
        KeyCode::RightShift => 54,
        KeyCode::NumpadMultiply => 55,
        KeyCode::LeftAlt => 56,
        KeyCode::Space => 57,
        KeyCode::CapsLock => 58,
        KeyCode::F1 => 59,
        KeyCode::F2 => 60,
        KeyCode::F3 => 61,
        KeyCode::F4 => 62,
        KeyCode::F5 => 63,
        KeyCode::F6 => 64,
        KeyCode::F7 => 65,
        KeyCode::F8 => 66,
        KeyCode::F9 => 67,
        KeyCode::F10 => 68,
        KeyCode::NumLock => 69,
        KeyCode::ScrollLock => 70,
        KeyCode::Numpad7 => 71,
        KeyCode::Numpad8 => 72,
        KeyCode::Numpad9 => 73,
        KeyCode::NumpadSubtract => 74,
        KeyCode::Numpad4 => 75,
        KeyCode::Numpad5 => 76,
        KeyCode::Numpad6 => 77,
        KeyCode::NumpadAdd => 78,
        KeyCode::Numpad1 => 79,
        KeyCode::Numpad2 => 80,
        KeyCode::Numpad3 => 81,
        KeyCode::Numpad0 => 82,
        KeyCode::NumpadDecimal => 83,
        KeyCode::F11 => 87,
        KeyCode::F12 => 88,
        KeyCode::NumpadEnter => 96,
        KeyCode::RightCtrl => 97,
        KeyCode::NumpadDivide => 98,
        KeyCode::PrintScreen => 99, // KEY_SYSRQ
        KeyCode::RightAlt => 100,
        KeyCode::Home => 102,
        KeyCode::Up => 103,
        KeyCode::PageUp => 104,
        KeyCode::Left => 105,
        KeyCode::Right => 106,
        KeyCode::End => 107,
        KeyCode::Down => 108,
        KeyCode::PageDown => 109,
        KeyCode::Insert => 110,
        KeyCode::Delete => 111,
        KeyCode::VolumeMute => 113,
        KeyCode::VolumeDown => 114,
        KeyCode::VolumeUp => 115,
        KeyCode::Pause => 119,
        KeyCode::LeftMeta => 125,
        KeyCode::RightMeta => 126,
        KeyCode::MediaNext => 163,
        KeyCode::MediaPlayPause => 164,
        KeyCode::MediaPrev => 165,
        KeyCode::MediaStop => 166,
        KeyCode::Unknown(code) => code,
    }
}

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
        // Build the set of keys to register
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
    /// This includes all keys that correspond to `KeyCode` variants,
    /// ensuring we can emit any key that might be remapped.
    fn build_key_set() -> AttributeSet<Key> {
        let mut keys = AttributeSet::<Key>::new();

        // Letters A-Z
        keys.insert(Key::KEY_A);
        keys.insert(Key::KEY_B);
        keys.insert(Key::KEY_C);
        keys.insert(Key::KEY_D);
        keys.insert(Key::KEY_E);
        keys.insert(Key::KEY_F);
        keys.insert(Key::KEY_G);
        keys.insert(Key::KEY_H);
        keys.insert(Key::KEY_I);
        keys.insert(Key::KEY_J);
        keys.insert(Key::KEY_K);
        keys.insert(Key::KEY_L);
        keys.insert(Key::KEY_M);
        keys.insert(Key::KEY_N);
        keys.insert(Key::KEY_O);
        keys.insert(Key::KEY_P);
        keys.insert(Key::KEY_Q);
        keys.insert(Key::KEY_R);
        keys.insert(Key::KEY_S);
        keys.insert(Key::KEY_T);
        keys.insert(Key::KEY_U);
        keys.insert(Key::KEY_V);
        keys.insert(Key::KEY_W);
        keys.insert(Key::KEY_X);
        keys.insert(Key::KEY_Y);
        keys.insert(Key::KEY_Z);

        // Numbers 0-9 (top row)
        keys.insert(Key::KEY_0);
        keys.insert(Key::KEY_1);
        keys.insert(Key::KEY_2);
        keys.insert(Key::KEY_3);
        keys.insert(Key::KEY_4);
        keys.insert(Key::KEY_5);
        keys.insert(Key::KEY_6);
        keys.insert(Key::KEY_7);
        keys.insert(Key::KEY_8);
        keys.insert(Key::KEY_9);

        // Function keys F1-F12
        keys.insert(Key::KEY_F1);
        keys.insert(Key::KEY_F2);
        keys.insert(Key::KEY_F3);
        keys.insert(Key::KEY_F4);
        keys.insert(Key::KEY_F5);
        keys.insert(Key::KEY_F6);
        keys.insert(Key::KEY_F7);
        keys.insert(Key::KEY_F8);
        keys.insert(Key::KEY_F9);
        keys.insert(Key::KEY_F10);
        keys.insert(Key::KEY_F11);
        keys.insert(Key::KEY_F12);

        // Modifier keys
        keys.insert(Key::KEY_LEFTSHIFT);
        keys.insert(Key::KEY_RIGHTSHIFT);
        keys.insert(Key::KEY_LEFTCTRL);
        keys.insert(Key::KEY_RIGHTCTRL);
        keys.insert(Key::KEY_LEFTALT);
        keys.insert(Key::KEY_RIGHTALT);
        keys.insert(Key::KEY_LEFTMETA);
        keys.insert(Key::KEY_RIGHTMETA);

        // Navigation keys
        keys.insert(Key::KEY_UP);
        keys.insert(Key::KEY_DOWN);
        keys.insert(Key::KEY_LEFT);
        keys.insert(Key::KEY_RIGHT);
        keys.insert(Key::KEY_HOME);
        keys.insert(Key::KEY_END);
        keys.insert(Key::KEY_PAGEUP);
        keys.insert(Key::KEY_PAGEDOWN);

        // Editing keys
        keys.insert(Key::KEY_INSERT);
        keys.insert(Key::KEY_DELETE);
        keys.insert(Key::KEY_BACKSPACE);

        // Whitespace keys
        keys.insert(Key::KEY_SPACE);
        keys.insert(Key::KEY_TAB);
        keys.insert(Key::KEY_ENTER);

        // Lock keys
        keys.insert(Key::KEY_CAPSLOCK);
        keys.insert(Key::KEY_NUMLOCK);
        keys.insert(Key::KEY_SCROLLLOCK);

        // Escape and Print Screen area
        keys.insert(Key::KEY_ESC);
        keys.insert(Key::KEY_SYSRQ); // Print Screen
        keys.insert(Key::KEY_PAUSE);

        // Punctuation and symbols
        keys.insert(Key::KEY_GRAVE);
        keys.insert(Key::KEY_MINUS);
        keys.insert(Key::KEY_EQUAL);
        keys.insert(Key::KEY_LEFTBRACE);
        keys.insert(Key::KEY_RIGHTBRACE);
        keys.insert(Key::KEY_BACKSLASH);
        keys.insert(Key::KEY_SEMICOLON);
        keys.insert(Key::KEY_APOSTROPHE);
        keys.insert(Key::KEY_COMMA);
        keys.insert(Key::KEY_DOT);
        keys.insert(Key::KEY_SLASH);

        // Numpad keys
        keys.insert(Key::KEY_KP0);
        keys.insert(Key::KEY_KP1);
        keys.insert(Key::KEY_KP2);
        keys.insert(Key::KEY_KP3);
        keys.insert(Key::KEY_KP4);
        keys.insert(Key::KEY_KP5);
        keys.insert(Key::KEY_KP6);
        keys.insert(Key::KEY_KP7);
        keys.insert(Key::KEY_KP8);
        keys.insert(Key::KEY_KP9);
        keys.insert(Key::KEY_KPPLUS);
        keys.insert(Key::KEY_KPMINUS);
        keys.insert(Key::KEY_KPASTERISK);
        keys.insert(Key::KEY_KPSLASH);
        keys.insert(Key::KEY_KPENTER);
        keys.insert(Key::KEY_KPDOT);

        // Media keys
        keys.insert(Key::KEY_VOLUMEUP);
        keys.insert(Key::KEY_VOLUMEDOWN);
        keys.insert(Key::KEY_MUTE);
        keys.insert(Key::KEY_PLAYPAUSE);
        keys.insert(Key::KEY_STOPCD);
        keys.insert(Key::KEY_NEXTSONG);
        keys.insert(Key::KEY_PREVIOUSSONG);

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
