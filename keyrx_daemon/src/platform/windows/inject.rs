use crate::platform::windows::keycode::{keycode_to_scancode, keycode_to_vk};
use std::mem::size_of;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

// Marker for events injected by the daemon
const DAEMON_OUTPUT_MARKER: usize = 0x4441454D; // "DAEM"

pub struct EventInjector;

impl EventInjector {
    #[allow(dead_code)]
    pub fn inject(&self, event: &keyrx_core::runtime::KeyEvent) -> Result<(), String> {
        let keycode = event.keycode();
        let is_release = event.is_release();

        // Use our hardcoded scan code table for injection (layout-independent).
        // MapVirtualKeyW is NOT used because VK_OEM_* → scan code mappings
        // are layout-dependent (e.g., VK_OEM_6 → different scan codes on JIS vs US).
        let full_scan = keycode_to_scancode(keycode)
            .ok_or_else(|| format!("Unmapped keycode: {:?}", keycode))?;
        let vk = keycode_to_vk(keycode).unwrap_or(0);
        let scan = (full_scan & 0xFF) as u16;
        let is_extended = full_scan > 0xFF;

        unsafe {
            let mut input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: std::mem::zeroed(),
            };

            input.Anonymous.ki = KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: (if is_release { KEYEVENTF_KEYUP } else { 0 }) | KEYEVENTF_SCANCODE,
                time: 0,
                dwExtraInfo: DAEMON_OUTPUT_MARKER,
            };

            if is_extended {
                input.Anonymous.ki.dwFlags |= KEYEVENTF_EXTENDEDKEY;
            }

            if SendInput(1, &input, size_of::<INPUT>() as i32) == 0 {
                log::error!("SendInput failed: {}", std::io::Error::last_os_error());
                return Err("SendInput failed".to_string());
            }
        }

        Ok(())
    }
}
