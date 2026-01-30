//! Virtual keyboard device for testing
//!
//! This module provides a virtual keyboard that can simulate key presses
//! for E2E testing without requiring actual hardware.

use keyrx_core::config::KeyCode;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE,
};

use super::keycode::keycode_to_scancode;

/// Virtual keyboard for simulating key presses in tests
pub struct VirtualKeyboard {
    // Track which keys are currently pressed
    pressed_keys: std::collections::HashSet<u32>,
}

impl VirtualKeyboard {
    /// Create a new virtual keyboard
    pub fn new() -> Self {
        Self {
            pressed_keys: std::collections::HashSet::new(),
        }
    }

    /// Simulate a key press (down)
    ///
    /// # Arguments
    ///
    /// * `key` - The key to press
    ///
    /// # Returns
    ///
    /// Number of events successfully sent (should be 1)
    pub fn press_key(&mut self, key: KeyCode) -> Result<u32, String> {
        let scan_code = keycode_to_scancode(key)
            .ok_or_else(|| format!("Cannot convert {:?} to scan code", key))?;

        // Check if extended key (has 0xE000 prefix)
        let is_extended = (scan_code & 0xE000) == 0xE000;
        let base_scan_code = (scan_code & 0xFF) as u16;

        let mut input: INPUT = unsafe { std::mem::zeroed() };
        input.r#type = INPUT_KEYBOARD;
        input.Anonymous.ki = KEYBDINPUT {
            wVk: 0, // We use scan codes, not virtual keys
            wScan: base_scan_code,
            dwFlags: KEYEVENTF_SCANCODE
                | if is_extended { 0x0001 } else { 0 }, // KEYEVENTF_EXTENDEDKEY
            time: 0,
            dwExtraInfo: 0,
        };

        let sent = unsafe { SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32) };

        if sent == 1 {
            self.pressed_keys.insert(scan_code);
            Ok(sent)
        } else {
            Err(format!("SendInput failed for {:?}", key))
        }
    }

    /// Simulate a key release (up)
    ///
    /// # Arguments
    ///
    /// * `key` - The key to release
    ///
    /// # Returns
    ///
    /// Number of events successfully sent (should be 1)
    pub fn release_key(&mut self, key: KeyCode) -> Result<u32, String> {
        let scan_code = keycode_to_scancode(key)
            .ok_or_else(|| format!("Cannot convert {:?} to scan code", key))?;

        let is_extended = (scan_code & 0xE000) == 0xE000;
        let base_scan_code = (scan_code & 0xFF) as u16;

        let mut input: INPUT = unsafe { std::mem::zeroed() };
        input.r#type = INPUT_KEYBOARD;
        input.Anonymous.ki = KEYBDINPUT {
            wVk: 0,
            wScan: base_scan_code,
            dwFlags: KEYEVENTF_SCANCODE
                | KEYEVENTF_KEYUP
                | if is_extended { 0x0001 } else { 0 },
            time: 0,
            dwExtraInfo: 0,
        };

        let sent = unsafe { SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32) };

        if sent == 1 {
            self.pressed_keys.remove(&scan_code);
            Ok(sent)
        } else {
            Err(format!("SendInput failed for {:?}", key))
        }
    }

    /// Simulate a key tap (press + release)
    ///
    /// # Arguments
    ///
    /// * `key` - The key to tap
    /// * `duration_ms` - How long to hold the key (simulates realistic typing)
    ///
    /// # Returns
    ///
    /// Number of events successfully sent (should be 2)
    pub fn tap_key(&mut self, key: KeyCode, duration_ms: u64) -> Result<u32, String> {
        self.press_key(key)?;
        std::thread::sleep(std::time::Duration::from_millis(duration_ms));
        self.release_key(key)?;
        Ok(2)
    }

    /// Type a sequence of keys
    ///
    /// # Arguments
    ///
    /// * `keys` - The keys to type
    /// * `delay_ms` - Delay between keys
    pub fn type_keys(&mut self, keys: &[KeyCode], delay_ms: u64) -> Result<(), String> {
        for key in keys {
            self.tap_key(*key, 50)?; // 50ms press duration
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
        Ok(())
    }

    /// Release all currently pressed keys
    pub fn release_all(&mut self) -> Result<(), String> {
        let pressed: Vec<u32> = self.pressed_keys.iter().copied().collect();
        for scan_code in pressed {
            // Convert scan code back to KeyCode (best effort)
            // For now, just release using scan code directly
            let is_extended = (scan_code & 0xE000) == 0xE000;
            let base_scan_code = (scan_code & 0xFF) as u16;

            let mut input: INPUT = unsafe { std::mem::zeroed() };
            input.r#type = INPUT_KEYBOARD;
            input.Anonymous.ki = KEYBDINPUT {
                wVk: 0,
                wScan: base_scan_code,
                dwFlags: KEYEVENTF_SCANCODE
                    | KEYEVENTF_KEYUP
                    | if is_extended { 0x0001 } else { 0 },
                time: 0,
                dwExtraInfo: 0,
            };

            unsafe {
                SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
            }
        }
        self.pressed_keys.clear();
        Ok(())
    }
}

impl Drop for VirtualKeyboard {
    fn drop(&mut self) {
        // Release all keys when dropped
        let _ = self.release_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_keyboard_creation() {
        let kb = VirtualKeyboard::new();
        assert_eq!(kb.pressed_keys.len(), 0);
    }

    #[test]
    fn test_scan_code_conversion() {
        // Verify our scan code conversions are correct
        let w_scan = keycode_to_scancode(KeyCode::W).expect("W should have scan code");
        assert_eq!(w_scan, 0x11);

        let e_scan = keycode_to_scancode(KeyCode::E).expect("E should have scan code");
        assert_eq!(e_scan, 0x12);

        let o_scan = keycode_to_scancode(KeyCode::O).expect("O should have scan code");
        assert_eq!(o_scan, 0x18);
    }
}
