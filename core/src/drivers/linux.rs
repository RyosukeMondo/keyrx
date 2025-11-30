//! Linux input driver using evdev/uinput.

use crate::drivers::DeviceInfo;
use crate::engine::{InputEvent, KeyCode, OutputAction};
use crate::error::LinuxDriverError;
use crate::traits::InputSource;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use crossbeam_channel::Sender;
use evdev::{uinput::VirtualDeviceBuilder, AttributeSet, Key};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tracing::{debug, error, trace, warn};

const UINPUT_PATH: &str = "/dev/uinput";
const UINPUT_DEVICE_NAME: &str = "KeyRx Virtual Keyboard";

/// Reader for keyboard events from an evdev device.
///
/// `EvdevReader` provides exclusive access to a keyboard device through the evdev
/// subsystem. It uses the EVIOCGRAB ioctl to grab the device, preventing other
/// applications from receiving keyboard events while KeyRx is active.
///
/// # Thread Safety
///
/// The `running` flag is shared across threads using `Arc<AtomicBool>` to allow
/// clean shutdown from the main thread.
// TODO: Remove allow(dead_code) once LinuxInput uses EvdevReader (task 5.2)
#[allow(dead_code)]
pub struct EvdevReader {
    /// The evdev device handle for reading keyboard events.
    device: evdev::Device,
    /// Channel sender for forwarding events to the async engine.
    tx: Sender<InputEvent>,
    /// Shared flag to signal when the reader should stop.
    running: Arc<AtomicBool>,
    /// Path to the device (for error messages and logging).
    device_path: PathBuf,
}

#[allow(dead_code)]
impl EvdevReader {
    /// Create a new EvdevReader for the given device path.
    ///
    /// # Arguments
    ///
    /// * `device_path` - Path to the evdev device (e.g., `/dev/input/event0`)
    /// * `tx` - Channel sender for forwarding input events
    /// * `running` - Shared flag for controlling the read loop
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The device path does not exist
    /// - Permission is denied when opening the device
    /// - The device cannot be opened for other reasons
    pub fn new(
        device_path: PathBuf,
        tx: Sender<InputEvent>,
        running: Arc<AtomicBool>,
    ) -> Result<Self> {
        let device = evdev::Device::open(&device_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                LinuxDriverError::device_not_found(&device_path)
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                LinuxDriverError::permission_denied(&device_path)
            } else {
                LinuxDriverError::GrabFailed(e)
            }
        })?;

        debug!(
            "Opened evdev device: {} at {}",
            device.name().unwrap_or("Unknown"),
            device_path.display()
        );

        Ok(Self {
            device,
            tx,
            running,
            device_path,
        })
    }

    /// Grab exclusive access to the keyboard device.
    ///
    /// While grabbed, the keyboard events are only sent to KeyRx and not to
    /// other applications. This is essential for key remapping to work properly.
    ///
    /// # Errors
    ///
    /// Returns `LinuxDriverError::GrabFailed` if:
    /// - Another process has already grabbed the device
    /// - The user lacks sufficient permissions
    pub fn grab(&mut self) -> Result<()> {
        self.device
            .grab()
            .map_err(|e| LinuxDriverError::grab_failed(std::io::Error::other(e.to_string())))?;
        debug!("Grabbed keyboard device: {}", self.device_path.display());
        Ok(())
    }

    /// Release the keyboard grab.
    ///
    /// This restores normal keyboard operation, allowing other applications
    /// to receive keyboard events again. Called automatically during shutdown.
    ///
    /// # Errors
    ///
    /// Returns an error if the ungrab operation fails. This is rare and usually
    /// indicates a system-level issue.
    pub fn ungrab(&mut self) -> Result<()> {
        self.device
            .ungrab()
            .map_err(|e| LinuxDriverError::grab_failed(std::io::Error::other(e.to_string())))?;
        debug!("Released keyboard device: {}", self.device_path.display());
        Ok(())
    }

    /// Check if the reader should continue running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Get a reference to the underlying evdev device.
    pub fn device(&self) -> &evdev::Device {
        &self.device
    }

    /// Get a mutable reference to the underlying evdev device.
    pub fn device_mut(&mut self) -> &mut evdev::Device {
        &mut self.device
    }

    /// Get the channel sender for forwarding events.
    pub fn sender(&self) -> &Sender<InputEvent> {
        &self.tx
    }

    /// Get the device path.
    pub fn device_path(&self) -> &Path {
        &self.device_path
    }

    /// Spawn a blocking read thread that captures keyboard events.
    ///
    /// This method consumes the `EvdevReader` and moves it into a dedicated thread
    /// that continuously reads events from the evdev device. Events are converted
    /// to `InputEvent` and sent through the channel.
    ///
    /// # Returns
    ///
    /// Returns a `JoinHandle` that can be used to wait for the thread to complete.
    /// The thread will exit when:
    /// - The `running` flag is set to `false`
    /// - A critical error occurs (e.g., device disconnected)
    /// - The channel receiver is dropped
    ///
    /// # Event Processing
    ///
    /// Only `EV_KEY` events are processed. Event values:
    /// - 0: Key released
    /// - 1: Key pressed
    /// - 2: Key repeat (ignored, we synthesize repeats differently)
    pub fn spawn(mut self) -> JoinHandle<()> {
        let device_path = self.device_path.clone();

        thread::spawn(move || {
            debug!("EvdevReader thread started for {}", device_path.display());

            while self.running.load(Ordering::Relaxed) {
                // fetch_events blocks until events are available
                match self.device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            // Only process EV_KEY events
                            if event.event_type() != evdev::EventType::KEY {
                                continue;
                            }

                            // value: 0 = release, 1 = press, 2 = repeat
                            let value = event.value();
                            if value == 2 {
                                // Skip repeat events - we handle repeats differently
                                trace!("Skipping repeat event for key {}", event.code());
                                continue;
                            }

                            let pressed = value == 1;
                            let key_code = evdev_to_keycode(event.code());

                            // Extract timestamp from event as microseconds since UNIX epoch
                            let timestamp = event
                                .timestamp()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_micros() as u64)
                                .unwrap_or(0);

                            let input_event = InputEvent {
                                key: key_code,
                                pressed,
                                timestamp,
                            };

                            trace!(
                                "Read event: {:?} {} at {}",
                                key_code,
                                if pressed { "down" } else { "up" },
                                timestamp
                            );

                            // Send event to channel
                            if let Err(e) = self.tx.send(input_event) {
                                // Channel closed, receiver dropped - exit thread
                                debug!("Event channel closed, stopping reader: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        // Check if we should continue
                        if !self.running.load(Ordering::Relaxed) {
                            break;
                        }

                        // Log error but continue - might be temporary
                        error!("Error reading events from {}: {}", device_path.display(), e);

                        // Small sleep to avoid busy loop on persistent errors
                        thread::sleep(std::time::Duration::from_millis(10));
                    }
                }
            }

            // Clean up: ungrab the device
            if let Err(e) = self.ungrab() {
                warn!(
                    "Failed to ungrab device {} on shutdown: {}",
                    device_path.display(),
                    e
                );
            }

            debug!("EvdevReader thread stopped for {}", device_path.display());
        })
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
// TODO: Remove allow(dead_code) once LinuxInput uses UinputWriter (task 5.2)
#[allow(dead_code)]
pub struct UinputWriter {
    /// The virtual uinput device for key injection.
    device: evdev::uinput::VirtualDevice,
}

#[allow(dead_code)]
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
    pub fn device(&self) -> &evdev::uinput::VirtualDevice {
        &self.device
    }

    /// Get a mutable reference to the underlying virtual device.
    pub fn device_mut(&mut self) -> &mut evdev::uinput::VirtualDevice {
        &mut self.device
    }
}

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

/// List all keyboard devices available on the system.
///
/// Scans `/dev/input/event*` devices and returns information about those
/// that have keyboard capability (EV_KEY with standard keyboard keys).
///
/// # Errors
///
/// Returns an error if:
/// - The `/dev/input` directory cannot be read
/// - Device enumeration fails due to permission issues
pub fn list_keyboards() -> Result<Vec<DeviceInfo>> {
    let mut keyboards = Vec::new();
    let input_dir = Path::new("/dev/input");

    if !input_dir.exists() {
        bail!(
            "/dev/input directory not found\n\n\
             Remediation:\n  \
             1. Ensure you are running on a Linux system with evdev support\n  \
             2. Check if the input subsystem is loaded"
        );
    }

    let entries = std::fs::read_dir(input_dir).context("Failed to read /dev/input directory")?;

    for entry in entries.flatten() {
        let path = entry.path();

        // Only look at event* devices
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !file_name.starts_with("event") {
            continue;
        }

        // Try to open and check if it's a keyboard
        match evdev::Device::open(&path) {
            Ok(device) => {
                // Check if device has KEY capability with common keyboard keys
                let has_keyboard_keys = device
                    .supported_keys()
                    .map(|keys| {
                        // Check for common keyboard keys (A, Enter, Space)
                        keys.contains(evdev::Key::KEY_A)
                            && keys.contains(evdev::Key::KEY_ENTER)
                            && keys.contains(evdev::Key::KEY_SPACE)
                    })
                    .unwrap_or(false);

                if has_keyboard_keys {
                    let name = device.name().unwrap_or("Unknown Device").to_string();
                    let input_id = device.input_id();

                    keyboards.push(DeviceInfo::new(
                        path,
                        name,
                        input_id.vendor(),
                        input_id.product(),
                        true,
                    ));
                }
            }
            Err(e) => {
                // Log but don't fail - device might be busy or lack permissions
                debug!("Could not open {}: {}", path.display(), e);
            }
        }
    }

    // Sort by path for consistent ordering
    keyboards.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(keyboards)
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
