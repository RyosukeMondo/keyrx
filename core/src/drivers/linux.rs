//! Linux input driver using evdev/uinput.

use crate::engine::{InputEvent, KeyCode, OutputAction};
use crate::traits::InputSource;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::path::Path;
use tracing::{debug, warn};

const UINPUT_PATH: &str = "/dev/uinput";

/// Linux input source using evdev for capture and uinput for injection.
#[derive(Default)]
pub struct LinuxInput {
    running: bool,
    /// Placeholder for evdev device handle (full integration is post-MVP).
    _evdev_device: Option<()>,
}

impl LinuxInput {
    /// Create a new Linux input source.
    pub fn new() -> Result<Self> {
        Ok(Self {
            running: false,
            _evdev_device: None,
        })
    }

    /// Check if uinput device is accessible.
    fn check_uinput_accessible() -> Result<()> {
        let path = Path::new(UINPUT_PATH);

        if !path.exists() {
            bail!(
                "uinput device not found at {UINPUT_PATH}\n\n\
                 Remediation:\n  \
                 1. Load the uinput kernel module: sudo modprobe uinput\n  \
                 2. If that fails, check if uinput is built into your kernel\n  \
                 3. Ensure your kernel supports CONFIG_INPUT_UINPUT"
            );
        }

        // Check if readable/writable by attempting to open for read
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
        {
            Ok(_) => {
                debug!("Successfully accessed {UINPUT_PATH}");
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                bail!(
                    "Permission denied accessing {UINPUT_PATH}\n\n\
                     Remediation:\n  \
                     1. Add your user to the 'input' group: sudo usermod -aG input $USER\n  \
                     2. Create a udev rule for uinput access:\n     \
                        echo 'KERNEL==\"uinput\", MODE=\"0660\", GROUP=\"input\"' | \
                        sudo tee /etc/udev/rules.d/99-uinput.rules\n  \
                     3. Reload udev rules: sudo udevadm control --reload-rules && \
                        sudo udevadm trigger\n  \
                     4. Log out and log back in for group changes to take effect\n  \
                     5. Alternatively, run with sudo (not recommended for regular use)"
                );
            }
            Err(e) => {
                bail!(
                    "Failed to access {UINPUT_PATH}: {e}\n\n\
                     Remediation:\n  \
                     1. Check device permissions: ls -la {UINPUT_PATH}\n  \
                     2. Check if you have read/write access to the device\n  \
                     3. Ensure the uinput module is loaded: lsmod | grep uinput"
                );
            }
        }
    }
}

#[async_trait]
impl InputSource for LinuxInput {
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>> {
        if !self.running {
            warn!("poll_events called while not running");
            return Ok(vec![]);
        }

        // Stub: Return empty vec. Full evdev integration is post-MVP.
        // In the future, this would read events from the evdev device.
        Ok(vec![])
    }

    async fn send_output(&mut self, action: OutputAction) -> Result<()> {
        if !self.running {
            warn!("send_output called while not running");
            return Ok(());
        }

        // Stub: Log the action. Full uinput integration is post-MVP.
        debug!("Would send output: {:?}", action);
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        if self.running {
            warn!("LinuxInput already running");
            return Ok(());
        }

        // Verify uinput is accessible before starting
        Self::check_uinput_accessible().context("Failed to start Linux input source")?;

        self.running = true;
        debug!("LinuxInput started successfully");

        // Stub: Full evdev device opening is post-MVP.
        // In the future, this would:
        // 1. Open evdev device for keyboard input
        // 2. Create uinput virtual device for output injection
        // 3. Grab the keyboard to intercept all events

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if !self.running {
            debug!("LinuxInput already stopped");
            return Ok(());
        }

        self.running = false;
        debug!("LinuxInput stopped");

        // Stub: Full cleanup is post-MVP.
        // In the future, this would:
        // 1. Release the keyboard grab
        // 2. Close the evdev device
        // 3. Destroy the uinput virtual device

        Ok(())
    }
}

/// Convert evdev key code to KeyRx KeyCode.
/// This is a stub mapping - full implementation post-MVP.
#[allow(dead_code)]
fn evdev_to_keycode(code: u16) -> KeyCode {
    // Common evdev key codes (from linux/input-event-codes.h)
    match code {
        1 => KeyCode::Escape,
        2 => KeyCode::Key1,
        3 => KeyCode::Key2,
        4 => KeyCode::Key3,
        5 => KeyCode::Key4,
        6 => KeyCode::Key5,
        7 => KeyCode::Key6,
        8 => KeyCode::Key7,
        9 => KeyCode::Key8,
        10 => KeyCode::Key9,
        11 => KeyCode::Key0,
        12 => KeyCode::Minus,
        13 => KeyCode::Equal,
        14 => KeyCode::Backspace,
        15 => KeyCode::Tab,
        16 => KeyCode::Q,
        17 => KeyCode::W,
        18 => KeyCode::E,
        19 => KeyCode::R,
        20 => KeyCode::T,
        21 => KeyCode::Y,
        22 => KeyCode::U,
        23 => KeyCode::I,
        24 => KeyCode::O,
        25 => KeyCode::P,
        26 => KeyCode::LeftBracket,
        27 => KeyCode::RightBracket,
        28 => KeyCode::Enter,
        29 => KeyCode::LeftCtrl,
        30 => KeyCode::A,
        31 => KeyCode::S,
        32 => KeyCode::D,
        33 => KeyCode::F,
        34 => KeyCode::G,
        35 => KeyCode::H,
        36 => KeyCode::J,
        37 => KeyCode::K,
        38 => KeyCode::L,
        39 => KeyCode::Semicolon,
        40 => KeyCode::Apostrophe,
        41 => KeyCode::Grave,
        42 => KeyCode::LeftShift,
        43 => KeyCode::Backslash,
        44 => KeyCode::Z,
        45 => KeyCode::X,
        46 => KeyCode::C,
        47 => KeyCode::V,
        48 => KeyCode::B,
        49 => KeyCode::N,
        50 => KeyCode::M,
        51 => KeyCode::Comma,
        52 => KeyCode::Period,
        53 => KeyCode::Slash,
        54 => KeyCode::RightShift,
        55 => KeyCode::NumpadMultiply,
        56 => KeyCode::LeftAlt,
        57 => KeyCode::Space,
        58 => KeyCode::CapsLock,
        59 => KeyCode::F1,
        60 => KeyCode::F2,
        61 => KeyCode::F3,
        62 => KeyCode::F4,
        63 => KeyCode::F5,
        64 => KeyCode::F6,
        65 => KeyCode::F7,
        66 => KeyCode::F8,
        67 => KeyCode::F9,
        68 => KeyCode::F10,
        69 => KeyCode::NumLock,
        70 => KeyCode::ScrollLock,
        71 => KeyCode::Numpad7,
        72 => KeyCode::Numpad8,
        73 => KeyCode::Numpad9,
        74 => KeyCode::NumpadSubtract,
        75 => KeyCode::Numpad4,
        76 => KeyCode::Numpad5,
        77 => KeyCode::Numpad6,
        78 => KeyCode::NumpadAdd,
        79 => KeyCode::Numpad1,
        80 => KeyCode::Numpad2,
        81 => KeyCode::Numpad3,
        82 => KeyCode::Numpad0,
        83 => KeyCode::NumpadDecimal,
        87 => KeyCode::F11,
        88 => KeyCode::F12,
        96 => KeyCode::NumpadEnter,
        97 => KeyCode::RightCtrl,
        98 => KeyCode::NumpadDivide,
        100 => KeyCode::RightAlt,
        102 => KeyCode::Home,
        103 => KeyCode::Up,
        104 => KeyCode::PageUp,
        105 => KeyCode::Left,
        106 => KeyCode::Right,
        107 => KeyCode::End,
        108 => KeyCode::Down,
        109 => KeyCode::PageDown,
        110 => KeyCode::Insert,
        111 => KeyCode::Delete,
        113 => KeyCode::VolumeMute,
        114 => KeyCode::VolumeDown,
        115 => KeyCode::VolumeUp,
        125 => KeyCode::LeftMeta,
        126 => KeyCode::RightMeta,
        163 => KeyCode::MediaNext,
        164 => KeyCode::MediaPlayPause,
        165 => KeyCode::MediaPrev,
        166 => KeyCode::MediaStop,
        _ => KeyCode::Unknown(code),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_input_default() {
        let input = LinuxInput::default();
        assert!(!input.running);
    }

    #[test]
    fn linux_input_new() {
        let input = LinuxInput::new().unwrap();
        assert!(!input.running);
    }

    #[test]
    fn evdev_keycode_mapping() {
        assert_eq!(evdev_to_keycode(1), KeyCode::Escape);
        assert_eq!(evdev_to_keycode(30), KeyCode::A);
        assert_eq!(evdev_to_keycode(58), KeyCode::CapsLock);
        assert_eq!(evdev_to_keycode(57), KeyCode::Space);
        assert_eq!(evdev_to_keycode(28), KeyCode::Enter);
        assert_eq!(evdev_to_keycode(9999), KeyCode::Unknown(9999));
    }
}
