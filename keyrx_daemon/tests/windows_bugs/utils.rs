use keyrx_core::config::KeyCode;
use keyrx_daemon::platform::windows::keycode::{keycode_to_scancode, keycode_to_vk};
use std::mem::size_of;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

#[allow(dead_code)]
pub struct VirtualWindowsKeyboard {
    dev_name: String,
}

impl VirtualWindowsKeyboard {
    #[allow(dead_code)]
    pub fn new(name: &str) -> Self {
        Self {
            dev_name: name.to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn press(&self, keycode: KeyCode) -> Result<(), String> {
        self.send_input(keycode, false)
    }

    #[allow(dead_code)]
    pub fn release(&self, keycode: KeyCode) -> Result<(), String> {
        self.send_input(keycode, true)
    }

    #[allow(dead_code)]
    fn send_input(&self, keycode: KeyCode, is_release: bool) -> Result<(), String> {
        // Use hardcoded scan code table (layout-independent)
        let full_scan = keycode_to_scancode(keycode)
            .ok_or_else(|| format!("Unmapped keycode: {:?}", keycode))?;
        let vk = keycode_to_vk(keycode).unwrap_or(0);
        inject_keyboard_event(vk, (full_scan & 0xFF) as u16, full_scan > 0xFF, is_release)
    }
}

#[allow(dead_code)]
pub fn inject_keyboard_event(
    vk: u16,
    scan: u16,
    is_extended: bool,
    is_release: bool,
) -> Result<(), String> {
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
            dwExtraInfo: 0,
        };

        if is_extended {
            input.Anonymous.ki.dwFlags |= KEYEVENTF_EXTENDEDKEY;
        }

        if SendInput(1, &input, size_of::<INPUT>() as i32) == 0 {
            return Err(format!(
                "SendInput failed: {}",
                std::io::Error::last_os_error()
            ));
        }
    }
    Ok(())
}
