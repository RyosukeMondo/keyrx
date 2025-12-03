//! Windows key injection using the SendInput API.
//!
//! This module provides key injection capabilities using the Windows SendInput API.
//! Injected events are marked with the LLKHF_INJECTED flag, allowing them to be
//! distinguished from real physical input.

use crate::drivers::KeyInjector;
use crate::engine::KeyCode;
use crate::error::WindowsDriverError;
use anyhow::Result;
use tracing::{debug, error};
use windows::core::Error as WinError;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VIRTUAL_KEY,
};

use super::keymap::keycode_to_vk;

/// Key injector using the Windows SendInput API.
///
/// This struct provides the ability to inject keyboard events into the system
/// as if they came from a physical keyboard. Injected events are marked with
/// the LLKHF_INJECTED flag, allowing them to be distinguished from real input.
///
/// # Extended Keys
///
/// Some keys require the KEYEVENTF_EXTENDEDKEY flag to work correctly:
/// - Arrow keys (Up, Down, Left, Right)
/// - Navigation keys (Home, End, Page Up, Page Down, Insert, Delete)
/// - Numpad Enter (distinct from main Enter)
/// - Right-side modifiers (Right Ctrl, Right Alt)
/// - Print Screen, Pause, Num Lock
pub struct SendInputInjector;

impl SendInputInjector {
    /// Create a new key injector.
    ///
    /// The injector is stateless and can be created at any time.
    pub fn new() -> Self {
        Self
    }

    /// Inject a key press or release event.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to inject
    /// * `pressed` - `true` for key down, `false` for key up
    ///
    /// # Errors
    ///
    /// Returns `WindowsDriverError::SendInputFailed` if the injection fails.
    /// This can happen if:
    /// - Another application has blocked input injection
    /// - The system is in a secure state (e.g., UAC prompt)
    /// - The calling process lacks required privileges
    pub fn inject_key(&self, key: KeyCode, pressed: bool) -> Result<(), WindowsDriverError> {
        let vk_code = keycode_to_vk(key);
        let input = build_keyboard_input(key, vk_code, pressed);

        // SAFETY: SendInput is safe to call with the following guarantees:
        // - We pass a valid INPUT array containing properly initialized INPUT structures
        // - The size parameter matches the actual size of INPUT as required by Windows
        // - The array length (1) matches the number of structures we're passing
        // - All fields in the INPUT structure are valid (virtual key, flags, etc.)
        let inputs = [input];
        let result = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };

        if result == 0 {
            // SendInput failed, get the error code from the last Win32 error
            let win_error = WinError::from_win32();
            error!(
                service = "keyrx",
                event = "sendinput_failed",
                component = "windows_injector",
                key = ?key,
                pressed = pressed,
                error = ?win_error,
                "SendInput failed"
            );
            Err(WindowsDriverError::send_input_failed(
                win_error.code().0 as u32,
            ))
        } else {
            debug!(
                service = "keyrx",
                event = "key_injected",
                component = "windows_injector",
                key = ?key,
                pressed = pressed,
                vk_code = vk_code,
                extended = is_extended_key(key),
                "Injected key event"
            );
            Ok(())
        }
    }

    /// Inject a key press followed by a key release (a complete key tap).
    ///
    /// This is a convenience method for injecting a full key press cycle.
    pub fn inject_key_tap(&self, key: KeyCode) -> Result<(), WindowsDriverError> {
        self.inject_key(key, true)?;
        self.inject_key(key, false)
    }
}

impl Default for SendInputInjector {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyInjector for SendInputInjector {
    fn inject(&mut self, key: KeyCode, pressed: bool) -> Result<()> {
        self.inject_key(key, pressed).map_err(Into::into)
    }

    fn sync(&mut self) -> Result<()> {
        // Windows SendInput processes events immediately, no sync needed
        Ok(())
    }
}

// SAFETY: SendInputInjector is safe to send between threads because:
// - It is completely stateless (zero-sized type with no fields)
// - SendInput is a thread-safe Windows API that can be called from any thread
// - No thread-local state is used or modified
// - No shared mutable state exists
unsafe impl Send for SendInputInjector {}

/// Build a Windows INPUT structure for keyboard input.
///
/// Constructs the INPUT structure with proper flags based on the key type
/// (extended vs. regular) and the press/release state.
fn build_keyboard_input(key: KeyCode, vk_code: u16, pressed: bool) -> INPUT {
    // Determine flags
    let mut flags = KEYBD_EVENT_FLAGS::default();

    // Add KEYEVENTF_KEYUP for key release
    if !pressed {
        flags |= KEYEVENTF_KEYUP;
    }

    // Add KEYEVENTF_EXTENDEDKEY for extended keys
    if is_extended_key(key) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }

    // Build the KEYBDINPUT structure
    let kbd_input = KEYBDINPUT {
        wVk: VIRTUAL_KEY(vk_code),
        wScan: 0, // Let Windows determine scan code from virtual key
        dwFlags: flags,
        time: 0,        // System will fill in the time
        dwExtraInfo: 0, // No extra info
    };

    // Build the INPUT structure with the keyboard input
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 { ki: kbd_input },
    }
}

/// Check if a key is an extended key that requires KEYEVENTF_EXTENDEDKEY.
///
/// Extended keys are those on the enhanced keyboard that were not on the
/// original IBM PC/XT 83-key keyboard. These include:
/// - Right-side modifiers (Right Alt, Right Ctrl)
/// - Navigation cluster (Insert, Delete, Home, End, Page Up, Page Down)
/// - Arrow keys
/// - Numpad Enter, Numpad /, Print Screen, Pause, Num Lock
pub fn is_extended_key(key: KeyCode) -> bool {
    matches!(
        key,
        // Navigation keys
        KeyCode::Insert
            | KeyCode::Delete
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::PageUp
            | KeyCode::PageDown
            // Arrow keys
            | KeyCode::Up
            | KeyCode::Down
            | KeyCode::Left
            | KeyCode::Right
            // Right-side modifiers
            | KeyCode::RightCtrl
            | KeyCode::RightAlt
            // Numpad keys that need extended flag
            | KeyCode::NumpadEnter
            | KeyCode::NumpadDivide
            // Other extended keys
            | KeyCode::PrintScreen
            | KeyCode::Pause
            | KeyCode::NumLock
            // Windows keys
            | KeyCode::LeftMeta
            | KeyCode::RightMeta
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::KeyInjector;

    #[test]
    fn send_input_injector_creation() {
        let injector = SendInputInjector::new();
        // Injector is stateless, just verify it can be created
        drop(injector);

        // Also test Default impl
        let _injector: SendInputInjector = Default::default();
    }

    #[test]
    fn send_input_injector_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<SendInputInjector>();
    }

    #[test]
    fn send_input_injector_implements_key_injector() {
        // Verify that SendInputInjector implements KeyInjector
        fn assert_key_injector<T: KeyInjector>() {}
        assert_key_injector::<SendInputInjector>();
    }

    #[test]
    fn send_input_injector_sync_is_noop() {
        let mut injector = SendInputInjector::new();
        // Sync should succeed (it's a no-op on Windows)
        let result = injector.sync();
        assert!(result.is_ok());
    }

    #[test]
    fn extended_key_detection_navigation() {
        // Navigation keys should be extended
        assert!(is_extended_key(KeyCode::Insert));
        assert!(is_extended_key(KeyCode::Delete));
        assert!(is_extended_key(KeyCode::Home));
        assert!(is_extended_key(KeyCode::End));
        assert!(is_extended_key(KeyCode::PageUp));
        assert!(is_extended_key(KeyCode::PageDown));
    }

    #[test]
    fn extended_key_detection_arrows() {
        // Arrow keys should be extended
        assert!(is_extended_key(KeyCode::Up));
        assert!(is_extended_key(KeyCode::Down));
        assert!(is_extended_key(KeyCode::Left));
        assert!(is_extended_key(KeyCode::Right));
    }

    #[test]
    fn extended_key_detection_modifiers() {
        // Right-side modifiers should be extended
        assert!(is_extended_key(KeyCode::RightCtrl));
        assert!(is_extended_key(KeyCode::RightAlt));
        assert!(is_extended_key(KeyCode::LeftMeta));
        assert!(is_extended_key(KeyCode::RightMeta));

        // Left-side modifiers should NOT be extended (except meta)
        assert!(!is_extended_key(KeyCode::LeftCtrl));
        assert!(!is_extended_key(KeyCode::LeftAlt));
        assert!(!is_extended_key(KeyCode::LeftShift));
        assert!(!is_extended_key(KeyCode::RightShift));
    }

    #[test]
    fn extended_key_detection_numpad() {
        // NumpadEnter and NumpadDivide are extended
        assert!(is_extended_key(KeyCode::NumpadEnter));
        assert!(is_extended_key(KeyCode::NumpadDivide));

        // Other numpad keys are NOT extended
        assert!(!is_extended_key(KeyCode::Numpad0));
        assert!(!is_extended_key(KeyCode::Numpad5));
        assert!(!is_extended_key(KeyCode::NumpadAdd));
        assert!(!is_extended_key(KeyCode::NumpadMultiply));
    }

    #[test]
    fn extended_key_detection_regular_keys() {
        // Regular keys should NOT be extended
        assert!(!is_extended_key(KeyCode::A));
        assert!(!is_extended_key(KeyCode::Space));
        assert!(!is_extended_key(KeyCode::Enter));
        assert!(!is_extended_key(KeyCode::Tab));
        assert!(!is_extended_key(KeyCode::Escape));
        assert!(!is_extended_key(KeyCode::CapsLock));
        assert!(!is_extended_key(KeyCode::F1));
        assert!(!is_extended_key(KeyCode::Key1));
    }

    #[test]
    fn extended_key_detection_special() {
        // PrintScreen, Pause, NumLock are extended
        assert!(is_extended_key(KeyCode::PrintScreen));
        assert!(is_extended_key(KeyCode::Pause));
        assert!(is_extended_key(KeyCode::NumLock));

        // ScrollLock and CapsLock are NOT extended
        assert!(!is_extended_key(KeyCode::ScrollLock));
        assert!(!is_extended_key(KeyCode::CapsLock));
    }
}
