//! Linux platform implementation using evdev for input and uinput for output.
//!
//! This module provides the Linux-specific implementation for keyboard input capture
//! and event injection using the evdev and uinput kernel interfaces.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use evdev::{Device, InputEventKind, Key};
use uinput::event::keyboard::{Key as UKey, KeyPad, Keyboard, Misc};
use uinput::Device as UInputDevice;

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::event::KeyEvent;

use crate::platform::{DeviceError, InputDevice, OutputDevice};

/// Linux platform structure for keyboard input/output operations.
///
/// This struct wraps the evdev input device and uinput output device,
/// providing a unified interface for keyboard remapping on Linux.
#[allow(dead_code)] // Fields will be used in tasks #3-4 (EvdevInput) and #6-8 (UinputOutput)
pub struct LinuxPlatform {
    input_device: Option<Device>,
    output_device: Option<UInputDevice>,
}

impl LinuxPlatform {
    /// Creates a new LinuxPlatform instance with no devices attached.
    #[must_use]
    pub fn new() -> Self {
        Self {
            input_device: None,
            output_device: None,
        }
    }

    /// Initializes the platform with input and output devices.
    ///
    /// # Errors
    ///
    /// Returns an error if device initialization fails.
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Placeholder for Linux input/output device initialization
        // Will be implemented in tasks #3-4 (EvdevInput) and #6-8 (UinputOutput)
        Ok(())
    }

    /// Runs the main event processing loop.
    ///
    /// # Errors
    ///
    /// Returns an error if event processing fails.
    pub fn process_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Placeholder for event processing loop
        // Will be implemented in task #17 (Daemon event loop)
        Ok(())
    }
}

impl Default for LinuxPlatform {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper for evdev input device with keyrx interface.
///
/// `EvdevInput` provides a high-level interface for capturing keyboard events
/// from Linux input devices via the evdev subsystem. It supports exclusive
/// access (grab) to prevent events from reaching other applications.
///
/// # Device Access
///
/// Input devices are accessed via `/dev/input/event*` device nodes.
/// By default, these require root or membership in the `input` group.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use keyrx_daemon::platform::linux::EvdevInput;
///
/// // Open keyboard device
/// let mut keyboard = EvdevInput::open(Path::new("/dev/input/event0"))?;
///
/// // Print device info
/// println!("Device: {}", keyboard.name());
/// if let Some(serial) = keyboard.serial() {
///     println!("Serial: {}", serial);
/// }
///
/// // Grab exclusive access (other apps won't receive events)
/// // keyboard.grab()?;  // Implemented in task #4
/// # Ok::<(), keyrx_daemon::platform::DeviceError>(())
/// ```
#[allow(dead_code)] // Will be used in task #4 (InputDevice trait implementation)
pub struct EvdevInput {
    /// The underlying evdev device handle.
    device: Device,
    /// Whether we have exclusive (grabbed) access to the device.
    grabbed: bool,
    /// Path to the device node (for identification).
    path: PathBuf,
}

#[allow(dead_code)] // Methods will be used in task #4 (InputDevice trait implementation)
impl EvdevInput {
    /// Opens an evdev input device by path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the device node (e.g., `/dev/input/event0`)
    ///
    /// # Returns
    ///
    /// * `Ok(EvdevInput)` - Successfully opened the device
    /// * `Err(DeviceError::NotFound)` - Device does not exist
    /// * `Err(DeviceError::PermissionDenied)` - Insufficient permissions
    /// * `Err(DeviceError::Io)` - Other I/O error
    ///
    /// # Permissions
    ///
    /// Accessing input devices typically requires:
    /// - Running as root, OR
    /// - Membership in the `input` group, OR
    /// - Appropriate udev rules granting access
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::platform::linux::EvdevInput;
    /// use keyrx_daemon::platform::DeviceError;
    ///
    /// match EvdevInput::open(Path::new("/dev/input/event0")) {
    ///     Ok(device) => println!("Opened: {}", device.name()),
    ///     Err(DeviceError::PermissionDenied(msg)) => {
    ///         eprintln!("Permission denied: {}", msg);
    ///         eprintln!("Try adding your user to the 'input' group");
    ///     }
    ///     Err(DeviceError::NotFound(msg)) => {
    ///         eprintln!("Device not found: {}", msg);
    ///     }
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub fn open(path: &Path) -> Result<Self, DeviceError> {
        let device = Device::open(path).map_err(|e| {
            let path_str = path.display().to_string();
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    DeviceError::NotFound(format!("device not found: {}", path_str))
                }
                std::io::ErrorKind::PermissionDenied => DeviceError::PermissionDenied(format!(
                    "cannot access {}: permission denied. Try adding user to 'input' group",
                    path_str
                )),
                _ => DeviceError::Io(e),
            }
        })?;

        Ok(Self {
            device,
            grabbed: false,
            path: path.to_path_buf(),
        })
    }

    /// Creates an `EvdevInput` from an existing evdev device.
    ///
    /// This is useful when you've already opened a device through other means
    /// (e.g., device enumeration) and want to wrap it in the keyrx interface.
    ///
    /// # Arguments
    ///
    /// * `device` - An already-opened evdev device
    ///
    /// # Note
    ///
    /// The path will be extracted from the device if available, otherwise
    /// set to an empty path.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use evdev::Device;
    /// use keyrx_daemon::platform::linux::EvdevInput;
    ///
    /// // Open device with evdev directly
    /// let evdev_device = Device::open("/dev/input/event0")?;
    ///
    /// // Wrap in EvdevInput
    /// let input = EvdevInput::from_device(evdev_device);
    /// println!("Device: {}", input.name());
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn from_device(device: Device) -> Self {
        // Try to get the device path, falling back to empty if unavailable
        let path = device
            .physical_path()
            .map(PathBuf::from)
            .unwrap_or_default();

        Self {
            device,
            grabbed: false,
            path,
        }
    }

    /// Returns the device name as reported by the kernel.
    ///
    /// This is typically a human-readable name like "AT Translated Set 2 keyboard"
    /// or "Logitech USB Keyboard".
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::platform::linux::EvdevInput;
    ///
    /// let keyboard = EvdevInput::open(Path::new("/dev/input/event0"))?;
    /// println!("Device name: {}", keyboard.name());
    /// # Ok::<(), keyrx_daemon::platform::DeviceError>(())
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        self.device.name().unwrap_or("Unknown Device")
    }

    /// Returns the device serial number, if available.
    ///
    /// Not all devices report a serial number. USB devices typically do,
    /// while built-in laptop keyboards often don't.
    ///
    /// # Returns
    ///
    /// * `Some(&str)` - Serial number if reported by device
    /// * `None` - Device doesn't have a serial number
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::platform::linux::EvdevInput;
    ///
    /// let keyboard = EvdevInput::open(Path::new("/dev/input/event0"))?;
    /// if let Some(serial) = keyboard.serial() {
    ///     println!("Serial: {}", serial);
    /// } else {
    ///     println!("No serial number available");
    /// }
    /// # Ok::<(), keyrx_daemon::platform::DeviceError>(())
    /// ```
    #[must_use]
    pub fn serial(&self) -> Option<&str> {
        // evdev crate's uniq() method returns the unique identifier (serial)
        self.device.unique_name()
    }

    /// Returns the path to the device node.
    ///
    /// This is the path used to open the device (e.g., `/dev/input/event0`).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::platform::linux::EvdevInput;
    ///
    /// let keyboard = EvdevInput::open(Path::new("/dev/input/event0"))?;
    /// println!("Path: {}", keyboard.path().display());
    /// # Ok::<(), keyrx_daemon::platform::DeviceError>(())
    /// ```
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns whether the device is currently grabbed (exclusive access).
    ///
    /// When a device is grabbed, events from it are not delivered to other
    /// applications. This is essential for key remapping to prevent the
    /// original keystroke from reaching applications.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use keyrx_daemon::platform::linux::EvdevInput;
    ///
    /// let keyboard = EvdevInput::open(Path::new("/dev/input/event0"))?;
    /// assert!(!keyboard.is_grabbed());
    /// // After grab(): keyboard.is_grabbed() would return true
    /// # Ok::<(), keyrx_daemon::platform::DeviceError>(())
    /// ```
    #[must_use]
    pub fn is_grabbed(&self) -> bool {
        self.grabbed
    }

    /// Returns a reference to the underlying evdev device.
    ///
    /// This allows direct access to evdev functionality not exposed
    /// through the `EvdevInput` interface.
    #[must_use]
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Returns a mutable reference to the underlying evdev device.
    ///
    /// This allows direct access to evdev functionality not exposed
    /// through the `EvdevInput` interface.
    pub fn device_mut(&mut self) -> &mut Device {
        &mut self.device
    }
}

/// InputDevice trait implementation for EvdevInput.
///
/// Enables keyboard event capture from real Linux input devices using the evdev subsystem.
///
/// # Event Filtering
///
/// Only EV_KEY events are processed:
/// - value=1: Key press (→ `KeyEvent::Press`)
/// - value=0: Key release (→ `KeyEvent::Release`)
/// - value=2: Key repeat (ignored - handled by applications)
///
/// # Exclusive Access
///
/// The `grab()` method uses the `EVIOCGRAB` ioctl to obtain exclusive access
/// to the device. While grabbed, other applications (including X11/Wayland)
/// will not receive events from this device.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use keyrx_daemon::platform::linux::EvdevInput;
/// use keyrx_daemon::platform::{InputDevice, DeviceError};
///
/// let mut keyboard = EvdevInput::open(Path::new("/dev/input/event0"))?;
///
/// // Grab exclusive access for remapping
/// keyboard.grab()?;
///
/// loop {
///     match keyboard.next_event() {
///         Ok(event) => {
///             println!("Event: {:?}", event);
///             // Process and remap the event...
///         }
///         Err(DeviceError::EndOfStream) => break,
///         Err(e) => return Err(e),
///     }
/// }
///
/// keyboard.release()?;
/// # Ok::<(), DeviceError>(())
/// ```
#[allow(dead_code)] // Will be used in task #17 (Daemon event loop)
impl InputDevice for EvdevInput {
    /// Reads the next keyboard event from the device.
    ///
    /// This method blocks until a key press or release event is available.
    /// Repeat events (value=2) are automatically filtered out.
    ///
    /// # Returns
    ///
    /// - `Ok(KeyEvent::Press(keycode))` for key press events
    /// - `Ok(KeyEvent::Release(keycode))` for key release events
    /// - `Err(DeviceError::EndOfStream)` when no more events (device disconnected)
    /// - `Err(DeviceError::Io)` on I/O errors
    ///
    /// # Unknown Keys
    ///
    /// Keys that don't map to a `KeyCode` are skipped (the method continues
    /// reading until it finds a known key). This allows unknown keys to be
    /// handled at a higher level (passthrough to output).
    fn next_event(&mut self) -> Result<KeyEvent, DeviceError> {
        loop {
            // Fetch events from the device
            // evdev::Device::fetch_events returns an iterator over events
            let events = self.device.fetch_events().map_err(|e| {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    DeviceError::EndOfStream
                } else {
                    DeviceError::Io(e)
                }
            })?;

            for event in events {
                // Only process EV_KEY events (keyboard key presses/releases)
                if let InputEventKind::Key(key) = event.kind() {
                    let value = event.value();

                    // value: 0 = release, 1 = press, 2 = repeat (ignored)
                    match value {
                        1 => {
                            // Key press
                            if let Some(keycode) = evdev_to_keycode(key.code()) {
                                return Ok(KeyEvent::Press(keycode));
                            }
                            // Unknown key - continue reading for known keys
                        }
                        0 => {
                            // Key release
                            if let Some(keycode) = evdev_to_keycode(key.code()) {
                                return Ok(KeyEvent::Release(keycode));
                            }
                            // Unknown key - continue reading for known keys
                        }
                        2 => {
                            // Key repeat - ignore, continue reading
                        }
                        _ => {
                            // Unknown event value - ignore
                        }
                    }
                }
                // Non-key events (EV_SYN, EV_MSC, etc.) are ignored
            }
            // If we processed all events and found no key events, loop to fetch more
        }
    }

    /// Grabs exclusive access to the device using EVIOCGRAB ioctl.
    ///
    /// After calling this method, the kernel will not deliver events from this
    /// device to other applications. This is essential for key remapping to
    /// prevent the original keystrokes from reaching applications.
    ///
    /// # Platform Details
    ///
    /// Uses the evdev crate's built-in grab functionality which wraps the
    /// `EVIOCGRAB` ioctl with value 1 to acquire exclusive access.
    ///
    /// # Errors
    ///
    /// - `DeviceError::PermissionDenied` if the process lacks CAP_SYS_ADMIN
    /// - `DeviceError::Io` for other ioctl failures
    fn grab(&mut self) -> Result<(), DeviceError> {
        if self.grabbed {
            return Ok(()); // Already grabbed
        }

        self.device.grab().map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                DeviceError::PermissionDenied(format!(
                    "cannot grab device {}: permission denied. \
                     Try running as root or with CAP_SYS_ADMIN",
                    self.path.display()
                ))
            } else {
                DeviceError::Io(e)
            }
        })?;

        self.grabbed = true;
        Ok(())
    }

    /// Releases exclusive access to the device.
    ///
    /// After calling this method, other applications will receive events from
    /// this device again. This should be called during graceful shutdown to
    /// restore normal keyboard operation.
    ///
    /// # Platform Details
    ///
    /// Uses the evdev crate's ungrab functionality which wraps the
    /// `EVIOCGRAB` ioctl with value 0 to release exclusive access.
    fn release(&mut self) -> Result<(), DeviceError> {
        if !self.grabbed {
            return Ok(()); // Not grabbed
        }

        self.device.ungrab().map_err(DeviceError::Io)?;
        self.grabbed = false;
        Ok(())
    }
}

// ============================================================================
// UinputOutput - Virtual Keyboard for Event Injection
// ============================================================================

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
#[allow(dead_code)] // Methods will be used in task #17 (Daemon event loop)
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

#[allow(dead_code)] // Methods will be used in task #17 (Daemon event loop)
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
        // Create uinput device with keyboard capabilities
        let device = uinput::default()
            .map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("Permission denied") || err_str.contains("EACCES") {
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
                    DeviceError::Io(std::io::Error::other(format!("uinput open failed: {}", e)))
                }
            })?
            .name(name)
            .map_err(|e| {
                DeviceError::Io(std::io::Error::other(format!(
                    "failed to set device name: {}",
                    e
                )))
            })?
            // Enable all keyboard events
            .event(uinput::event::Keyboard::All)
            .map_err(|e| {
                DeviceError::Io(std::io::Error::other(format!(
                    "failed to configure keyboard events: {}",
                    e
                )))
            })?
            .create()
            .map_err(|e| {
                DeviceError::Io(std::io::Error::other(format!(
                    "failed to create uinput device: {}",
                    e
                )))
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
                    eprintln!(
                        "Warning: failed to release key {:?} during cleanup: {}",
                        keycode, e
                    );
                }
                // Try to synchronize after each release
                if let Err(e) = dev.synchronize() {
                    eprintln!(
                        "Warning: failed to synchronize after releasing {:?}: {}",
                        keycode, e
                    );
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

/// Maps a keyrx KeyCode to a uinput Keyboard variant.
///
/// This is used by the OutputDevice implementation to convert keyrx KeyCodes
/// to the uinput crate's key type for event injection.
///
/// The uinput crate organizes keys into different enums:
/// - `Key`: Basic keyboard keys (letters, numbers, F-keys, modifiers, arrows, etc.)
/// - `KeyPad`: Numpad keys
/// - `Misc`: Media keys, system keys, browser keys, and special function keys
fn keycode_to_uinput_key(keycode: KeyCode) -> Keyboard {
    match keycode {
        // Letters A-Z
        KeyCode::A => Keyboard::Key(UKey::A),
        KeyCode::B => Keyboard::Key(UKey::B),
        KeyCode::C => Keyboard::Key(UKey::C),
        KeyCode::D => Keyboard::Key(UKey::D),
        KeyCode::E => Keyboard::Key(UKey::E),
        KeyCode::F => Keyboard::Key(UKey::F),
        KeyCode::G => Keyboard::Key(UKey::G),
        KeyCode::H => Keyboard::Key(UKey::H),
        KeyCode::I => Keyboard::Key(UKey::I),
        KeyCode::J => Keyboard::Key(UKey::J),
        KeyCode::K => Keyboard::Key(UKey::K),
        KeyCode::L => Keyboard::Key(UKey::L),
        KeyCode::M => Keyboard::Key(UKey::M),
        KeyCode::N => Keyboard::Key(UKey::N),
        KeyCode::O => Keyboard::Key(UKey::O),
        KeyCode::P => Keyboard::Key(UKey::P),
        KeyCode::Q => Keyboard::Key(UKey::Q),
        KeyCode::R => Keyboard::Key(UKey::R),
        KeyCode::S => Keyboard::Key(UKey::S),
        KeyCode::T => Keyboard::Key(UKey::T),
        KeyCode::U => Keyboard::Key(UKey::U),
        KeyCode::V => Keyboard::Key(UKey::V),
        KeyCode::W => Keyboard::Key(UKey::W),
        KeyCode::X => Keyboard::Key(UKey::X),
        KeyCode::Y => Keyboard::Key(UKey::Y),
        KeyCode::Z => Keyboard::Key(UKey::Z),

        // Numbers 0-9 (top row)
        KeyCode::Num0 => Keyboard::Key(UKey::_0),
        KeyCode::Num1 => Keyboard::Key(UKey::_1),
        KeyCode::Num2 => Keyboard::Key(UKey::_2),
        KeyCode::Num3 => Keyboard::Key(UKey::_3),
        KeyCode::Num4 => Keyboard::Key(UKey::_4),
        KeyCode::Num5 => Keyboard::Key(UKey::_5),
        KeyCode::Num6 => Keyboard::Key(UKey::_6),
        KeyCode::Num7 => Keyboard::Key(UKey::_7),
        KeyCode::Num8 => Keyboard::Key(UKey::_8),
        KeyCode::Num9 => Keyboard::Key(UKey::_9),

        // Function keys F1-F12
        KeyCode::F1 => Keyboard::Key(UKey::F1),
        KeyCode::F2 => Keyboard::Key(UKey::F2),
        KeyCode::F3 => Keyboard::Key(UKey::F3),
        KeyCode::F4 => Keyboard::Key(UKey::F4),
        KeyCode::F5 => Keyboard::Key(UKey::F5),
        KeyCode::F6 => Keyboard::Key(UKey::F6),
        KeyCode::F7 => Keyboard::Key(UKey::F7),
        KeyCode::F8 => Keyboard::Key(UKey::F8),
        KeyCode::F9 => Keyboard::Key(UKey::F9),
        KeyCode::F10 => Keyboard::Key(UKey::F10),
        KeyCode::F11 => Keyboard::Key(UKey::F11),
        KeyCode::F12 => Keyboard::Key(UKey::F12),

        // Extended function keys F13-F24
        KeyCode::F13 => Keyboard::Key(UKey::F13),
        KeyCode::F14 => Keyboard::Key(UKey::F14),
        KeyCode::F15 => Keyboard::Key(UKey::F15),
        KeyCode::F16 => Keyboard::Key(UKey::F16),
        KeyCode::F17 => Keyboard::Key(UKey::F17),
        KeyCode::F18 => Keyboard::Key(UKey::F18),
        KeyCode::F19 => Keyboard::Key(UKey::F19),
        KeyCode::F20 => Keyboard::Key(UKey::F20),
        KeyCode::F21 => Keyboard::Key(UKey::F21),
        KeyCode::F22 => Keyboard::Key(UKey::F22),
        KeyCode::F23 => Keyboard::Key(UKey::F23),
        KeyCode::F24 => Keyboard::Key(UKey::F24),

        // Modifier keys
        KeyCode::LShift => Keyboard::Key(UKey::LeftShift),
        KeyCode::RShift => Keyboard::Key(UKey::RightShift),
        KeyCode::LCtrl => Keyboard::Key(UKey::LeftControl),
        KeyCode::RCtrl => Keyboard::Key(UKey::RightControl),
        KeyCode::LAlt => Keyboard::Key(UKey::LeftAlt),
        KeyCode::RAlt => Keyboard::Key(UKey::RightAlt),
        KeyCode::LMeta => Keyboard::Key(UKey::LeftMeta),
        KeyCode::RMeta => Keyboard::Key(UKey::RightMeta),

        // Special keys
        KeyCode::Escape => Keyboard::Key(UKey::Esc),
        KeyCode::Enter => Keyboard::Key(UKey::Enter),
        KeyCode::Backspace => Keyboard::Key(UKey::BackSpace),
        KeyCode::Tab => Keyboard::Key(UKey::Tab),
        KeyCode::Space => Keyboard::Key(UKey::Space),
        KeyCode::CapsLock => Keyboard::Key(UKey::CapsLock),
        KeyCode::NumLock => Keyboard::Key(UKey::NumLock),
        KeyCode::ScrollLock => Keyboard::Key(UKey::ScrollLock),
        KeyCode::PrintScreen => Keyboard::Key(UKey::SysRq),
        KeyCode::Pause => Keyboard::Misc(Misc::Pause),
        KeyCode::Insert => Keyboard::Key(UKey::Insert),
        KeyCode::Delete => Keyboard::Key(UKey::Delete),
        KeyCode::Home => Keyboard::Key(UKey::Home),
        KeyCode::End => Keyboard::Key(UKey::End),
        KeyCode::PageUp => Keyboard::Key(UKey::PageUp),
        KeyCode::PageDown => Keyboard::Key(UKey::PageDown),

        // Arrow keys
        KeyCode::Left => Keyboard::Key(UKey::Left),
        KeyCode::Right => Keyboard::Key(UKey::Right),
        KeyCode::Up => Keyboard::Key(UKey::Up),
        KeyCode::Down => Keyboard::Key(UKey::Down),

        // Punctuation and symbols
        KeyCode::LeftBracket => Keyboard::Key(UKey::LeftBrace),
        KeyCode::RightBracket => Keyboard::Key(UKey::RightBrace),
        KeyCode::Backslash => Keyboard::Key(UKey::BackSlash),
        KeyCode::Semicolon => Keyboard::Key(UKey::SemiColon),
        KeyCode::Quote => Keyboard::Key(UKey::Apostrophe),
        KeyCode::Comma => Keyboard::Key(UKey::Comma),
        KeyCode::Period => Keyboard::Key(UKey::Dot),
        KeyCode::Slash => Keyboard::Key(UKey::Slash),
        KeyCode::Grave => Keyboard::Key(UKey::Grave),
        KeyCode::Minus => Keyboard::Key(UKey::Minus),
        KeyCode::Equal => Keyboard::Key(UKey::Equal),

        // Numpad keys (use KeyPad enum)
        KeyCode::Numpad0 => Keyboard::KeyPad(KeyPad::_0),
        KeyCode::Numpad1 => Keyboard::KeyPad(KeyPad::_1),
        KeyCode::Numpad2 => Keyboard::KeyPad(KeyPad::_2),
        KeyCode::Numpad3 => Keyboard::KeyPad(KeyPad::_3),
        KeyCode::Numpad4 => Keyboard::KeyPad(KeyPad::_4),
        KeyCode::Numpad5 => Keyboard::KeyPad(KeyPad::_5),
        KeyCode::Numpad6 => Keyboard::KeyPad(KeyPad::_6),
        KeyCode::Numpad7 => Keyboard::KeyPad(KeyPad::_7),
        KeyCode::Numpad8 => Keyboard::KeyPad(KeyPad::_8),
        KeyCode::Numpad9 => Keyboard::KeyPad(KeyPad::_9),
        KeyCode::NumpadDivide => Keyboard::KeyPad(KeyPad::Slash),
        KeyCode::NumpadMultiply => Keyboard::KeyPad(KeyPad::Asterisk),
        KeyCode::NumpadSubtract => Keyboard::KeyPad(KeyPad::Minus),
        KeyCode::NumpadAdd => Keyboard::KeyPad(KeyPad::Plus),
        KeyCode::NumpadEnter => Keyboard::KeyPad(KeyPad::Enter),
        KeyCode::NumpadDecimal => Keyboard::KeyPad(KeyPad::Dot),

        // Media keys (use Misc enum)
        KeyCode::Mute => Keyboard::Misc(Misc::Mute),
        KeyCode::VolumeDown => Keyboard::Misc(Misc::VolumeDown),
        KeyCode::VolumeUp => Keyboard::Misc(Misc::VolumeUp),
        KeyCode::MediaPlayPause => Keyboard::Misc(Misc::PlayPause),
        KeyCode::MediaStop => Keyboard::Misc(Misc::StopCD),
        KeyCode::MediaPrevious => Keyboard::Misc(Misc::PreviousSong),
        KeyCode::MediaNext => Keyboard::Misc(Misc::NextSong),

        // System keys (use Misc enum)
        KeyCode::Power => Keyboard::Misc(Misc::Power),
        KeyCode::Sleep => Keyboard::Misc(Misc::Sleep),
        KeyCode::Wake => Keyboard::Misc(Misc::WakeUp),

        // Browser keys (use Misc enum)
        KeyCode::BrowserBack => Keyboard::Misc(Misc::Back),
        KeyCode::BrowserForward => Keyboard::Misc(Misc::Forward),
        KeyCode::BrowserRefresh => Keyboard::Misc(Misc::Refresh),
        KeyCode::BrowserStop => Keyboard::Misc(Misc::Stop),
        KeyCode::BrowserSearch => Keyboard::Misc(Misc::Search),
        KeyCode::BrowserFavorites => Keyboard::Misc(Misc::Bookmarks),
        KeyCode::BrowserHome => Keyboard::Misc(Misc::HomePage),

        // Application keys (use Misc enum)
        KeyCode::AppMail => Keyboard::Misc(Misc::Mail),
        KeyCode::AppCalculator => Keyboard::Misc(Misc::Calc),
        KeyCode::AppMyComputer => Keyboard::Misc(Misc::Computer),

        // Additional keys (use Misc enum)
        KeyCode::Menu => Keyboard::Misc(Misc::Menu),
        KeyCode::Help => Keyboard::Misc(Misc::Help),
        KeyCode::Select => Keyboard::Misc(Misc::Select),
        KeyCode::Execute => Keyboard::Misc(Misc::Open),
        KeyCode::Undo => Keyboard::Misc(Misc::Undo),
        KeyCode::Redo => Keyboard::Misc(Misc::Redo),
        KeyCode::Cut => Keyboard::Misc(Misc::Cut),
        KeyCode::Copy => Keyboard::Misc(Misc::Copy),
        KeyCode::Paste => Keyboard::Misc(Misc::Paste),
        KeyCode::Find => Keyboard::Misc(Misc::Find),
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
#[allow(dead_code)] // Will be used in task #17 (Daemon event loop)
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

        let key = match &event {
            KeyEvent::Press(keycode) | KeyEvent::Release(keycode) => {
                keycode_to_uinput_key(*keycode)
            }
        };

        match &event {
            KeyEvent::Press(keycode) => {
                device.press(&key).map_err(|e| {
                    DeviceError::InjectionFailed(format!("failed to press key: {}", e))
                })?;
                // Track this key as held
                self.held_keys.insert(*keycode);
            }
            KeyEvent::Release(keycode) => {
                device.release(&key).map_err(|e| {
                    DeviceError::InjectionFailed(format!("failed to release key: {}", e))
                })?;
                // Remove from held keys
                self.held_keys.remove(keycode);
            }
        }

        // Synchronize to ensure event is delivered immediately
        device.synchronize().map_err(|e| {
            DeviceError::InjectionFailed(format!("failed to synchronize events: {}", e))
        })?;

        Ok(())
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
            eprintln!("Warning: error during UinputOutput cleanup: {}", e);
        }
    }
}

/// Maps an evdev key code to a keyrx KeyCode.
///
/// # Arguments
/// * `code` - The evdev key code (from linux/input-event-codes.h)
///
/// # Returns
/// * `Some(KeyCode)` if the code maps to a known key
/// * `None` if the code is unknown (passthrough handling)
///
/// # Key Categories
/// - Letters: KEY_A (30) through KEY_Z
/// - Numbers: KEY_1 (2) through KEY_0 (11)
/// - Function keys: KEY_F1 (59) through KEY_F24
/// - Modifiers: KEY_LEFTSHIFT, KEY_RIGHTSHIFT, etc.
/// - Special keys: KEY_ESC, KEY_ENTER, KEY_BACKSPACE, etc.
#[must_use]
#[allow(dead_code)] // Will be used in task #4 (InputDevice trait implementation)
pub fn evdev_to_keycode(code: u16) -> Option<KeyCode> {
    // Convert u16 to evdev Key for pattern matching
    let key = Key::new(code);

    match key {
        // Letters A-Z
        Key::KEY_A => Some(KeyCode::A),
        Key::KEY_B => Some(KeyCode::B),
        Key::KEY_C => Some(KeyCode::C),
        Key::KEY_D => Some(KeyCode::D),
        Key::KEY_E => Some(KeyCode::E),
        Key::KEY_F => Some(KeyCode::F),
        Key::KEY_G => Some(KeyCode::G),
        Key::KEY_H => Some(KeyCode::H),
        Key::KEY_I => Some(KeyCode::I),
        Key::KEY_J => Some(KeyCode::J),
        Key::KEY_K => Some(KeyCode::K),
        Key::KEY_L => Some(KeyCode::L),
        Key::KEY_M => Some(KeyCode::M),
        Key::KEY_N => Some(KeyCode::N),
        Key::KEY_O => Some(KeyCode::O),
        Key::KEY_P => Some(KeyCode::P),
        Key::KEY_Q => Some(KeyCode::Q),
        Key::KEY_R => Some(KeyCode::R),
        Key::KEY_S => Some(KeyCode::S),
        Key::KEY_T => Some(KeyCode::T),
        Key::KEY_U => Some(KeyCode::U),
        Key::KEY_V => Some(KeyCode::V),
        Key::KEY_W => Some(KeyCode::W),
        Key::KEY_X => Some(KeyCode::X),
        Key::KEY_Y => Some(KeyCode::Y),
        Key::KEY_Z => Some(KeyCode::Z),

        // Numbers 0-9 (top row)
        // Note: evdev uses KEY_1 (2) through KEY_0 (11), not KEY_0 through KEY_9
        Key::KEY_1 => Some(KeyCode::Num1),
        Key::KEY_2 => Some(KeyCode::Num2),
        Key::KEY_3 => Some(KeyCode::Num3),
        Key::KEY_4 => Some(KeyCode::Num4),
        Key::KEY_5 => Some(KeyCode::Num5),
        Key::KEY_6 => Some(KeyCode::Num6),
        Key::KEY_7 => Some(KeyCode::Num7),
        Key::KEY_8 => Some(KeyCode::Num8),
        Key::KEY_9 => Some(KeyCode::Num9),
        Key::KEY_0 => Some(KeyCode::Num0),

        // Function keys F1-F12
        Key::KEY_F1 => Some(KeyCode::F1),
        Key::KEY_F2 => Some(KeyCode::F2),
        Key::KEY_F3 => Some(KeyCode::F3),
        Key::KEY_F4 => Some(KeyCode::F4),
        Key::KEY_F5 => Some(KeyCode::F5),
        Key::KEY_F6 => Some(KeyCode::F6),
        Key::KEY_F7 => Some(KeyCode::F7),
        Key::KEY_F8 => Some(KeyCode::F8),
        Key::KEY_F9 => Some(KeyCode::F9),
        Key::KEY_F10 => Some(KeyCode::F10),
        Key::KEY_F11 => Some(KeyCode::F11),
        Key::KEY_F12 => Some(KeyCode::F12),

        // Extended function keys F13-F24
        Key::KEY_F13 => Some(KeyCode::F13),
        Key::KEY_F14 => Some(KeyCode::F14),
        Key::KEY_F15 => Some(KeyCode::F15),
        Key::KEY_F16 => Some(KeyCode::F16),
        Key::KEY_F17 => Some(KeyCode::F17),
        Key::KEY_F18 => Some(KeyCode::F18),
        Key::KEY_F19 => Some(KeyCode::F19),
        Key::KEY_F20 => Some(KeyCode::F20),
        Key::KEY_F21 => Some(KeyCode::F21),
        Key::KEY_F22 => Some(KeyCode::F22),
        Key::KEY_F23 => Some(KeyCode::F23),
        Key::KEY_F24 => Some(KeyCode::F24),

        // Modifier keys
        Key::KEY_LEFTSHIFT => Some(KeyCode::LShift),
        Key::KEY_RIGHTSHIFT => Some(KeyCode::RShift),
        Key::KEY_LEFTCTRL => Some(KeyCode::LCtrl),
        Key::KEY_RIGHTCTRL => Some(KeyCode::RCtrl),
        Key::KEY_LEFTALT => Some(KeyCode::LAlt),
        Key::KEY_RIGHTALT => Some(KeyCode::RAlt),
        Key::KEY_LEFTMETA => Some(KeyCode::LMeta),
        Key::KEY_RIGHTMETA => Some(KeyCode::RMeta),

        // Special keys
        Key::KEY_ESC => Some(KeyCode::Escape),
        Key::KEY_ENTER => Some(KeyCode::Enter),
        Key::KEY_BACKSPACE => Some(KeyCode::Backspace),
        Key::KEY_TAB => Some(KeyCode::Tab),
        Key::KEY_SPACE => Some(KeyCode::Space),
        Key::KEY_CAPSLOCK => Some(KeyCode::CapsLock),
        Key::KEY_NUMLOCK => Some(KeyCode::NumLock),
        Key::KEY_SCROLLLOCK => Some(KeyCode::ScrollLock),
        Key::KEY_SYSRQ => Some(KeyCode::PrintScreen),
        Key::KEY_PAUSE => Some(KeyCode::Pause),
        Key::KEY_INSERT => Some(KeyCode::Insert),
        Key::KEY_DELETE => Some(KeyCode::Delete),
        Key::KEY_HOME => Some(KeyCode::Home),
        Key::KEY_END => Some(KeyCode::End),
        Key::KEY_PAGEUP => Some(KeyCode::PageUp),
        Key::KEY_PAGEDOWN => Some(KeyCode::PageDown),

        // Arrow keys
        Key::KEY_LEFT => Some(KeyCode::Left),
        Key::KEY_RIGHT => Some(KeyCode::Right),
        Key::KEY_UP => Some(KeyCode::Up),
        Key::KEY_DOWN => Some(KeyCode::Down),

        // Punctuation and symbols
        Key::KEY_LEFTBRACE => Some(KeyCode::LeftBracket),
        Key::KEY_RIGHTBRACE => Some(KeyCode::RightBracket),
        Key::KEY_BACKSLASH => Some(KeyCode::Backslash),
        Key::KEY_SEMICOLON => Some(KeyCode::Semicolon),
        Key::KEY_APOSTROPHE => Some(KeyCode::Quote),
        Key::KEY_COMMA => Some(KeyCode::Comma),
        Key::KEY_DOT => Some(KeyCode::Period),
        Key::KEY_SLASH => Some(KeyCode::Slash),
        Key::KEY_GRAVE => Some(KeyCode::Grave),
        Key::KEY_MINUS => Some(KeyCode::Minus),
        Key::KEY_EQUAL => Some(KeyCode::Equal),

        // Numpad keys
        Key::KEY_KP0 => Some(KeyCode::Numpad0),
        Key::KEY_KP1 => Some(KeyCode::Numpad1),
        Key::KEY_KP2 => Some(KeyCode::Numpad2),
        Key::KEY_KP3 => Some(KeyCode::Numpad3),
        Key::KEY_KP4 => Some(KeyCode::Numpad4),
        Key::KEY_KP5 => Some(KeyCode::Numpad5),
        Key::KEY_KP6 => Some(KeyCode::Numpad6),
        Key::KEY_KP7 => Some(KeyCode::Numpad7),
        Key::KEY_KP8 => Some(KeyCode::Numpad8),
        Key::KEY_KP9 => Some(KeyCode::Numpad9),
        Key::KEY_KPSLASH => Some(KeyCode::NumpadDivide),
        Key::KEY_KPASTERISK => Some(KeyCode::NumpadMultiply),
        Key::KEY_KPMINUS => Some(KeyCode::NumpadSubtract),
        Key::KEY_KPPLUS => Some(KeyCode::NumpadAdd),
        Key::KEY_KPENTER => Some(KeyCode::NumpadEnter),
        Key::KEY_KPDOT => Some(KeyCode::NumpadDecimal),

        // Media keys
        Key::KEY_MUTE => Some(KeyCode::Mute),
        Key::KEY_VOLUMEDOWN => Some(KeyCode::VolumeDown),
        Key::KEY_VOLUMEUP => Some(KeyCode::VolumeUp),
        Key::KEY_PLAYPAUSE => Some(KeyCode::MediaPlayPause),
        Key::KEY_STOPCD => Some(KeyCode::MediaStop),
        Key::KEY_PREVIOUSSONG => Some(KeyCode::MediaPrevious),
        Key::KEY_NEXTSONG => Some(KeyCode::MediaNext),

        // System keys
        Key::KEY_POWER => Some(KeyCode::Power),
        Key::KEY_SLEEP => Some(KeyCode::Sleep),
        Key::KEY_WAKEUP => Some(KeyCode::Wake),

        // Browser keys
        Key::KEY_BACK => Some(KeyCode::BrowserBack),
        Key::KEY_FORWARD => Some(KeyCode::BrowserForward),
        Key::KEY_REFRESH => Some(KeyCode::BrowserRefresh),
        Key::KEY_STOP => Some(KeyCode::BrowserStop),
        Key::KEY_SEARCH => Some(KeyCode::BrowserSearch),
        Key::KEY_BOOKMARKS => Some(KeyCode::BrowserFavorites),
        Key::KEY_HOMEPAGE => Some(KeyCode::BrowserHome),

        // Application keys
        Key::KEY_MAIL => Some(KeyCode::AppMail),
        Key::KEY_CALC => Some(KeyCode::AppCalculator),
        Key::KEY_COMPUTER => Some(KeyCode::AppMyComputer),

        // Additional keys
        Key::KEY_COMPOSE => Some(KeyCode::Menu),
        Key::KEY_HELP => Some(KeyCode::Help),
        Key::KEY_SELECT => Some(KeyCode::Select),
        Key::KEY_OPEN => Some(KeyCode::Execute), // KEY_OPEN is closest match for Execute
        Key::KEY_UNDO => Some(KeyCode::Undo),
        Key::KEY_REDO => Some(KeyCode::Redo),
        Key::KEY_CUT => Some(KeyCode::Cut),
        Key::KEY_COPY => Some(KeyCode::Copy),
        Key::KEY_PASTE => Some(KeyCode::Paste),
        Key::KEY_FIND => Some(KeyCode::Find),

        // Unknown key - return None for passthrough handling
        _ => None,
    }
}

/// Maps a keyrx KeyCode to an evdev key code.
///
/// # Arguments
/// * `keycode` - The keyrx KeyCode to convert
///
/// # Returns
/// The corresponding evdev key code (u16)
///
/// # Note
/// This function covers all KeyCode variants exhaustively.
/// The mapping is the inverse of `evdev_to_keycode`.
#[must_use]
#[allow(dead_code)] // Will be used in task #7 (OutputDevice trait implementation)
pub fn keycode_to_evdev(keycode: KeyCode) -> u16 {
    match keycode {
        // Letters A-Z
        KeyCode::A => Key::KEY_A.code(),
        KeyCode::B => Key::KEY_B.code(),
        KeyCode::C => Key::KEY_C.code(),
        KeyCode::D => Key::KEY_D.code(),
        KeyCode::E => Key::KEY_E.code(),
        KeyCode::F => Key::KEY_F.code(),
        KeyCode::G => Key::KEY_G.code(),
        KeyCode::H => Key::KEY_H.code(),
        KeyCode::I => Key::KEY_I.code(),
        KeyCode::J => Key::KEY_J.code(),
        KeyCode::K => Key::KEY_K.code(),
        KeyCode::L => Key::KEY_L.code(),
        KeyCode::M => Key::KEY_M.code(),
        KeyCode::N => Key::KEY_N.code(),
        KeyCode::O => Key::KEY_O.code(),
        KeyCode::P => Key::KEY_P.code(),
        KeyCode::Q => Key::KEY_Q.code(),
        KeyCode::R => Key::KEY_R.code(),
        KeyCode::S => Key::KEY_S.code(),
        KeyCode::T => Key::KEY_T.code(),
        KeyCode::U => Key::KEY_U.code(),
        KeyCode::V => Key::KEY_V.code(),
        KeyCode::W => Key::KEY_W.code(),
        KeyCode::X => Key::KEY_X.code(),
        KeyCode::Y => Key::KEY_Y.code(),
        KeyCode::Z => Key::KEY_Z.code(),

        // Numbers 0-9 (top row)
        KeyCode::Num0 => Key::KEY_0.code(),
        KeyCode::Num1 => Key::KEY_1.code(),
        KeyCode::Num2 => Key::KEY_2.code(),
        KeyCode::Num3 => Key::KEY_3.code(),
        KeyCode::Num4 => Key::KEY_4.code(),
        KeyCode::Num5 => Key::KEY_5.code(),
        KeyCode::Num6 => Key::KEY_6.code(),
        KeyCode::Num7 => Key::KEY_7.code(),
        KeyCode::Num8 => Key::KEY_8.code(),
        KeyCode::Num9 => Key::KEY_9.code(),

        // Function keys F1-F12
        KeyCode::F1 => Key::KEY_F1.code(),
        KeyCode::F2 => Key::KEY_F2.code(),
        KeyCode::F3 => Key::KEY_F3.code(),
        KeyCode::F4 => Key::KEY_F4.code(),
        KeyCode::F5 => Key::KEY_F5.code(),
        KeyCode::F6 => Key::KEY_F6.code(),
        KeyCode::F7 => Key::KEY_F7.code(),
        KeyCode::F8 => Key::KEY_F8.code(),
        KeyCode::F9 => Key::KEY_F9.code(),
        KeyCode::F10 => Key::KEY_F10.code(),
        KeyCode::F11 => Key::KEY_F11.code(),
        KeyCode::F12 => Key::KEY_F12.code(),

        // Extended function keys F13-F24
        KeyCode::F13 => Key::KEY_F13.code(),
        KeyCode::F14 => Key::KEY_F14.code(),
        KeyCode::F15 => Key::KEY_F15.code(),
        KeyCode::F16 => Key::KEY_F16.code(),
        KeyCode::F17 => Key::KEY_F17.code(),
        KeyCode::F18 => Key::KEY_F18.code(),
        KeyCode::F19 => Key::KEY_F19.code(),
        KeyCode::F20 => Key::KEY_F20.code(),
        KeyCode::F21 => Key::KEY_F21.code(),
        KeyCode::F22 => Key::KEY_F22.code(),
        KeyCode::F23 => Key::KEY_F23.code(),
        KeyCode::F24 => Key::KEY_F24.code(),

        // Modifier keys
        KeyCode::LShift => Key::KEY_LEFTSHIFT.code(),
        KeyCode::RShift => Key::KEY_RIGHTSHIFT.code(),
        KeyCode::LCtrl => Key::KEY_LEFTCTRL.code(),
        KeyCode::RCtrl => Key::KEY_RIGHTCTRL.code(),
        KeyCode::LAlt => Key::KEY_LEFTALT.code(),
        KeyCode::RAlt => Key::KEY_RIGHTALT.code(),
        KeyCode::LMeta => Key::KEY_LEFTMETA.code(),
        KeyCode::RMeta => Key::KEY_RIGHTMETA.code(),

        // Special keys
        KeyCode::Escape => Key::KEY_ESC.code(),
        KeyCode::Enter => Key::KEY_ENTER.code(),
        KeyCode::Backspace => Key::KEY_BACKSPACE.code(),
        KeyCode::Tab => Key::KEY_TAB.code(),
        KeyCode::Space => Key::KEY_SPACE.code(),
        KeyCode::CapsLock => Key::KEY_CAPSLOCK.code(),
        KeyCode::NumLock => Key::KEY_NUMLOCK.code(),
        KeyCode::ScrollLock => Key::KEY_SCROLLLOCK.code(),
        KeyCode::PrintScreen => Key::KEY_SYSRQ.code(),
        KeyCode::Pause => Key::KEY_PAUSE.code(),
        KeyCode::Insert => Key::KEY_INSERT.code(),
        KeyCode::Delete => Key::KEY_DELETE.code(),
        KeyCode::Home => Key::KEY_HOME.code(),
        KeyCode::End => Key::KEY_END.code(),
        KeyCode::PageUp => Key::KEY_PAGEUP.code(),
        KeyCode::PageDown => Key::KEY_PAGEDOWN.code(),

        // Arrow keys
        KeyCode::Left => Key::KEY_LEFT.code(),
        KeyCode::Right => Key::KEY_RIGHT.code(),
        KeyCode::Up => Key::KEY_UP.code(),
        KeyCode::Down => Key::KEY_DOWN.code(),

        // Punctuation and symbols
        KeyCode::LeftBracket => Key::KEY_LEFTBRACE.code(),
        KeyCode::RightBracket => Key::KEY_RIGHTBRACE.code(),
        KeyCode::Backslash => Key::KEY_BACKSLASH.code(),
        KeyCode::Semicolon => Key::KEY_SEMICOLON.code(),
        KeyCode::Quote => Key::KEY_APOSTROPHE.code(),
        KeyCode::Comma => Key::KEY_COMMA.code(),
        KeyCode::Period => Key::KEY_DOT.code(),
        KeyCode::Slash => Key::KEY_SLASH.code(),
        KeyCode::Grave => Key::KEY_GRAVE.code(),
        KeyCode::Minus => Key::KEY_MINUS.code(),
        KeyCode::Equal => Key::KEY_EQUAL.code(),

        // Numpad keys
        KeyCode::Numpad0 => Key::KEY_KP0.code(),
        KeyCode::Numpad1 => Key::KEY_KP1.code(),
        KeyCode::Numpad2 => Key::KEY_KP2.code(),
        KeyCode::Numpad3 => Key::KEY_KP3.code(),
        KeyCode::Numpad4 => Key::KEY_KP4.code(),
        KeyCode::Numpad5 => Key::KEY_KP5.code(),
        KeyCode::Numpad6 => Key::KEY_KP6.code(),
        KeyCode::Numpad7 => Key::KEY_KP7.code(),
        KeyCode::Numpad8 => Key::KEY_KP8.code(),
        KeyCode::Numpad9 => Key::KEY_KP9.code(),
        KeyCode::NumpadDivide => Key::KEY_KPSLASH.code(),
        KeyCode::NumpadMultiply => Key::KEY_KPASTERISK.code(),
        KeyCode::NumpadSubtract => Key::KEY_KPMINUS.code(),
        KeyCode::NumpadAdd => Key::KEY_KPPLUS.code(),
        KeyCode::NumpadEnter => Key::KEY_KPENTER.code(),
        KeyCode::NumpadDecimal => Key::KEY_KPDOT.code(),

        // Media keys
        KeyCode::Mute => Key::KEY_MUTE.code(),
        KeyCode::VolumeDown => Key::KEY_VOLUMEDOWN.code(),
        KeyCode::VolumeUp => Key::KEY_VOLUMEUP.code(),
        KeyCode::MediaPlayPause => Key::KEY_PLAYPAUSE.code(),
        KeyCode::MediaStop => Key::KEY_STOPCD.code(),
        KeyCode::MediaPrevious => Key::KEY_PREVIOUSSONG.code(),
        KeyCode::MediaNext => Key::KEY_NEXTSONG.code(),

        // System keys
        KeyCode::Power => Key::KEY_POWER.code(),
        KeyCode::Sleep => Key::KEY_SLEEP.code(),
        KeyCode::Wake => Key::KEY_WAKEUP.code(),

        // Browser keys
        KeyCode::BrowserBack => Key::KEY_BACK.code(),
        KeyCode::BrowserForward => Key::KEY_FORWARD.code(),
        KeyCode::BrowserRefresh => Key::KEY_REFRESH.code(),
        KeyCode::BrowserStop => Key::KEY_STOP.code(),
        KeyCode::BrowserSearch => Key::KEY_SEARCH.code(),
        KeyCode::BrowserFavorites => Key::KEY_BOOKMARKS.code(),
        KeyCode::BrowserHome => Key::KEY_HOMEPAGE.code(),

        // Application keys
        KeyCode::AppMail => Key::KEY_MAIL.code(),
        KeyCode::AppCalculator => Key::KEY_CALC.code(),
        KeyCode::AppMyComputer => Key::KEY_COMPUTER.code(),

        // Additional keys
        KeyCode::Menu => Key::KEY_COMPOSE.code(),
        KeyCode::Help => Key::KEY_HELP.code(),
        KeyCode::Select => Key::KEY_SELECT.code(),
        KeyCode::Execute => Key::KEY_OPEN.code(), // Closest match for Execute
        KeyCode::Undo => Key::KEY_UNDO.code(),
        KeyCode::Redo => Key::KEY_REDO.code(),
        KeyCode::Cut => Key::KEY_CUT.code(),
        KeyCode::Copy => Key::KEY_COPY.code(),
        KeyCode::Paste => Key::KEY_PASTE.code(),
        KeyCode::Find => Key::KEY_FIND.code(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that all letter keys map correctly
    #[test]
    fn test_letter_keys_mapping() {
        // Test A-Z
        assert_eq!(evdev_to_keycode(Key::KEY_A.code()), Some(KeyCode::A));
        assert_eq!(evdev_to_keycode(Key::KEY_Z.code()), Some(KeyCode::Z));
        assert_eq!(evdev_to_keycode(Key::KEY_M.code()), Some(KeyCode::M));

        // Test round-trip
        assert_eq!(keycode_to_evdev(KeyCode::A), Key::KEY_A.code());
        assert_eq!(keycode_to_evdev(KeyCode::Z), Key::KEY_Z.code());
    }

    /// Test that number keys map correctly
    #[test]
    fn test_number_keys_mapping() {
        // Note: evdev KEY_0 is actually the '0' key, not at position 0
        assert_eq!(evdev_to_keycode(Key::KEY_1.code()), Some(KeyCode::Num1));
        assert_eq!(evdev_to_keycode(Key::KEY_0.code()), Some(KeyCode::Num0));
        assert_eq!(evdev_to_keycode(Key::KEY_5.code()), Some(KeyCode::Num5));

        // Round-trip
        assert_eq!(keycode_to_evdev(KeyCode::Num0), Key::KEY_0.code());
        assert_eq!(keycode_to_evdev(KeyCode::Num9), Key::KEY_9.code());
    }

    /// Test that function keys map correctly
    #[test]
    fn test_function_keys_mapping() {
        assert_eq!(evdev_to_keycode(Key::KEY_F1.code()), Some(KeyCode::F1));
        assert_eq!(evdev_to_keycode(Key::KEY_F12.code()), Some(KeyCode::F12));
        assert_eq!(evdev_to_keycode(Key::KEY_F24.code()), Some(KeyCode::F24));

        // Round-trip
        assert_eq!(keycode_to_evdev(KeyCode::F1), Key::KEY_F1.code());
        assert_eq!(keycode_to_evdev(KeyCode::F12), Key::KEY_F12.code());
    }

    /// Test that modifier keys map correctly
    #[test]
    fn test_modifier_keys_mapping() {
        assert_eq!(
            evdev_to_keycode(Key::KEY_LEFTSHIFT.code()),
            Some(KeyCode::LShift)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_RIGHTSHIFT.code()),
            Some(KeyCode::RShift)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_LEFTCTRL.code()),
            Some(KeyCode::LCtrl)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_RIGHTCTRL.code()),
            Some(KeyCode::RCtrl)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_LEFTALT.code()),
            Some(KeyCode::LAlt)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_RIGHTALT.code()),
            Some(KeyCode::RAlt)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_LEFTMETA.code()),
            Some(KeyCode::LMeta)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_RIGHTMETA.code()),
            Some(KeyCode::RMeta)
        );

        // Round-trip
        assert_eq!(keycode_to_evdev(KeyCode::LShift), Key::KEY_LEFTSHIFT.code());
        assert_eq!(keycode_to_evdev(KeyCode::RMeta), Key::KEY_RIGHTMETA.code());
    }

    /// Test that special keys map correctly
    #[test]
    fn test_special_keys_mapping() {
        assert_eq!(evdev_to_keycode(Key::KEY_ESC.code()), Some(KeyCode::Escape));
        assert_eq!(
            evdev_to_keycode(Key::KEY_ENTER.code()),
            Some(KeyCode::Enter)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_BACKSPACE.code()),
            Some(KeyCode::Backspace)
        );
        assert_eq!(evdev_to_keycode(Key::KEY_TAB.code()), Some(KeyCode::Tab));
        assert_eq!(
            evdev_to_keycode(Key::KEY_SPACE.code()),
            Some(KeyCode::Space)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_CAPSLOCK.code()),
            Some(KeyCode::CapsLock)
        );

        // Round-trip
        assert_eq!(keycode_to_evdev(KeyCode::Escape), Key::KEY_ESC.code());
        assert_eq!(
            keycode_to_evdev(KeyCode::CapsLock),
            Key::KEY_CAPSLOCK.code()
        );
    }

    /// Test that arrow keys map correctly
    #[test]
    fn test_arrow_keys_mapping() {
        assert_eq!(evdev_to_keycode(Key::KEY_LEFT.code()), Some(KeyCode::Left));
        assert_eq!(
            evdev_to_keycode(Key::KEY_RIGHT.code()),
            Some(KeyCode::Right)
        );
        assert_eq!(evdev_to_keycode(Key::KEY_UP.code()), Some(KeyCode::Up));
        assert_eq!(evdev_to_keycode(Key::KEY_DOWN.code()), Some(KeyCode::Down));

        // Round-trip
        assert_eq!(keycode_to_evdev(KeyCode::Left), Key::KEY_LEFT.code());
        assert_eq!(keycode_to_evdev(KeyCode::Down), Key::KEY_DOWN.code());
    }

    /// Test that numpad keys map correctly
    #[test]
    fn test_numpad_keys_mapping() {
        assert_eq!(
            evdev_to_keycode(Key::KEY_KP0.code()),
            Some(KeyCode::Numpad0)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_KP9.code()),
            Some(KeyCode::Numpad9)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_KPENTER.code()),
            Some(KeyCode::NumpadEnter)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_KPPLUS.code()),
            Some(KeyCode::NumpadAdd)
        );

        // Round-trip
        assert_eq!(keycode_to_evdev(KeyCode::Numpad0), Key::KEY_KP0.code());
        assert_eq!(
            keycode_to_evdev(KeyCode::NumpadEnter),
            Key::KEY_KPENTER.code()
        );
    }

    /// Test that punctuation keys map correctly
    #[test]
    fn test_punctuation_keys_mapping() {
        assert_eq!(
            evdev_to_keycode(Key::KEY_LEFTBRACE.code()),
            Some(KeyCode::LeftBracket)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_RIGHTBRACE.code()),
            Some(KeyCode::RightBracket)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_SEMICOLON.code()),
            Some(KeyCode::Semicolon)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_APOSTROPHE.code()),
            Some(KeyCode::Quote)
        );
        assert_eq!(
            evdev_to_keycode(Key::KEY_COMMA.code()),
            Some(KeyCode::Comma)
        );
        assert_eq!(evdev_to_keycode(Key::KEY_DOT.code()), Some(KeyCode::Period));

        // Round-trip
        assert_eq!(
            keycode_to_evdev(KeyCode::LeftBracket),
            Key::KEY_LEFTBRACE.code()
        );
        assert_eq!(keycode_to_evdev(KeyCode::Period), Key::KEY_DOT.code());
    }

    /// Test that unknown keys return None
    #[test]
    fn test_unknown_keys_return_none() {
        // Use a key code that's unlikely to be mapped (high value)
        assert_eq!(evdev_to_keycode(0xFFFF), None);
        // BTN_LEFT is a mouse button, not a keyboard key
        assert_eq!(evdev_to_keycode(0x110), None);
    }

    /// Test round-trip conversion for all KeyCode variants
    #[test]
    fn test_round_trip_all_keys() {
        let all_keys = [
            // Letters
            KeyCode::A,
            KeyCode::B,
            KeyCode::C,
            KeyCode::D,
            KeyCode::E,
            KeyCode::F,
            KeyCode::G,
            KeyCode::H,
            KeyCode::I,
            KeyCode::J,
            KeyCode::K,
            KeyCode::L,
            KeyCode::M,
            KeyCode::N,
            KeyCode::O,
            KeyCode::P,
            KeyCode::Q,
            KeyCode::R,
            KeyCode::S,
            KeyCode::T,
            KeyCode::U,
            KeyCode::V,
            KeyCode::W,
            KeyCode::X,
            KeyCode::Y,
            KeyCode::Z,
            // Numbers
            KeyCode::Num0,
            KeyCode::Num1,
            KeyCode::Num2,
            KeyCode::Num3,
            KeyCode::Num4,
            KeyCode::Num5,
            KeyCode::Num6,
            KeyCode::Num7,
            KeyCode::Num8,
            KeyCode::Num9,
            // Function keys
            KeyCode::F1,
            KeyCode::F2,
            KeyCode::F3,
            KeyCode::F4,
            KeyCode::F5,
            KeyCode::F6,
            KeyCode::F7,
            KeyCode::F8,
            KeyCode::F9,
            KeyCode::F10,
            KeyCode::F11,
            KeyCode::F12,
            KeyCode::F13,
            KeyCode::F14,
            KeyCode::F15,
            KeyCode::F16,
            KeyCode::F17,
            KeyCode::F18,
            KeyCode::F19,
            KeyCode::F20,
            KeyCode::F21,
            KeyCode::F22,
            KeyCode::F23,
            KeyCode::F24,
            // Modifiers
            KeyCode::LShift,
            KeyCode::RShift,
            KeyCode::LCtrl,
            KeyCode::RCtrl,
            KeyCode::LAlt,
            KeyCode::RAlt,
            KeyCode::LMeta,
            KeyCode::RMeta,
            // Special keys
            KeyCode::Escape,
            KeyCode::Enter,
            KeyCode::Backspace,
            KeyCode::Tab,
            KeyCode::Space,
            KeyCode::CapsLock,
            KeyCode::NumLock,
            KeyCode::ScrollLock,
            KeyCode::PrintScreen,
            KeyCode::Pause,
            KeyCode::Insert,
            KeyCode::Delete,
            KeyCode::Home,
            KeyCode::End,
            KeyCode::PageUp,
            KeyCode::PageDown,
            // Arrow keys
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Up,
            KeyCode::Down,
            // Punctuation
            KeyCode::LeftBracket,
            KeyCode::RightBracket,
            KeyCode::Backslash,
            KeyCode::Semicolon,
            KeyCode::Quote,
            KeyCode::Comma,
            KeyCode::Period,
            KeyCode::Slash,
            KeyCode::Grave,
            KeyCode::Minus,
            KeyCode::Equal,
            // Numpad
            KeyCode::Numpad0,
            KeyCode::Numpad1,
            KeyCode::Numpad2,
            KeyCode::Numpad3,
            KeyCode::Numpad4,
            KeyCode::Numpad5,
            KeyCode::Numpad6,
            KeyCode::Numpad7,
            KeyCode::Numpad8,
            KeyCode::Numpad9,
            KeyCode::NumpadDivide,
            KeyCode::NumpadMultiply,
            KeyCode::NumpadSubtract,
            KeyCode::NumpadAdd,
            KeyCode::NumpadEnter,
            KeyCode::NumpadDecimal,
            // Media keys
            KeyCode::Mute,
            KeyCode::VolumeDown,
            KeyCode::VolumeUp,
            KeyCode::MediaPlayPause,
            KeyCode::MediaStop,
            KeyCode::MediaPrevious,
            KeyCode::MediaNext,
            // System keys
            KeyCode::Power,
            KeyCode::Sleep,
            KeyCode::Wake,
            // Browser keys
            KeyCode::BrowserBack,
            KeyCode::BrowserForward,
            KeyCode::BrowserRefresh,
            KeyCode::BrowserStop,
            KeyCode::BrowserSearch,
            KeyCode::BrowserFavorites,
            KeyCode::BrowserHome,
            // Application keys
            KeyCode::AppMail,
            KeyCode::AppCalculator,
            KeyCode::AppMyComputer,
            // Additional keys
            KeyCode::Menu,
            KeyCode::Help,
            KeyCode::Select,
            KeyCode::Execute,
            KeyCode::Undo,
            KeyCode::Redo,
            KeyCode::Cut,
            KeyCode::Copy,
            KeyCode::Paste,
            KeyCode::Find,
        ];

        for keycode in all_keys {
            let evdev_code = keycode_to_evdev(keycode);
            let back = evdev_to_keycode(evdev_code);
            assert_eq!(
                back,
                Some(keycode),
                "Round-trip failed for {:?}: evdev code {} -> {:?}",
                keycode,
                evdev_code,
                back
            );
        }
    }

    // ============================================
    // EvdevInput Tests
    // ============================================

    /// Test that opening a non-existent device returns NotFound error
    #[test]
    fn test_evdevinput_open_not_found() {
        use std::path::Path;

        let result = EvdevInput::open(Path::new("/dev/input/event_nonexistent_12345"));
        assert!(result.is_err());

        match result {
            Err(DeviceError::NotFound(msg)) => {
                assert!(
                    msg.contains("event_nonexistent"),
                    "Error message should contain path"
                );
            }
            Err(e) => panic!("Expected NotFound, got {:?}", e),
            Ok(_) => panic!("Expected error, got Ok"),
        }
    }

    /// Test EvdevInput::from_device with path extraction
    /// Note: This test is marked #[ignore] as it requires a real device
    #[test]
    #[ignore = "requires real input device - run manually with: cargo test -p keyrx_daemon --features linux test_evdevinput_from_device -- --ignored"]
    fn test_evdevinput_from_device() {
        // Try to open the first available event device
        for i in 0..20 {
            let path = format!("/dev/input/event{}", i);
            if let Ok(device) = evdev::Device::open(&path) {
                let input = EvdevInput::from_device(device);

                // Verify the device was wrapped correctly
                assert!(!input.name().is_empty());
                assert!(!input.is_grabbed());

                // Note: path may not match since from_device uses physical_path
                println!(
                    "Device: {}, Serial: {:?}, Path: {}",
                    input.name(),
                    input.serial(),
                    input.path().display()
                );
                return;
            }
        }

        panic!("No input devices available for testing");
    }

    /// Test that open returns PermissionDenied for devices we can't access
    /// Note: This test only works when NOT running as root
    #[test]
    #[ignore = "requires non-root user without input group - run manually"]
    fn test_evdevinput_open_permission_denied() {
        use std::path::Path;

        // Try to find a device that exists but we can't access
        for i in 0..20 {
            let path_str = format!("/dev/input/event{}", i);
            let path = Path::new(&path_str);

            if path.exists() {
                match EvdevInput::open(path) {
                    Err(DeviceError::PermissionDenied(msg)) => {
                        assert!(msg.contains("permission denied"));
                        assert!(msg.contains("input group"));
                        return;
                    }
                    Ok(_) => {
                        // We have permission, skip to next device or test
                        continue;
                    }
                    Err(e) => {
                        panic!("Expected PermissionDenied or Ok, got {:?}", e);
                    }
                }
            }
        }

        panic!("Test inconclusive: could not find a device to test permission denied");
    }

    /// Test that is_grabbed returns false initially
    #[test]
    #[ignore = "requires real input device - run manually"]
    fn test_evdevinput_not_grabbed_initially() {
        use std::path::Path;

        for i in 0..20 {
            let path = format!("/dev/input/event{}", i);
            if let Ok(input) = EvdevInput::open(Path::new(&path)) {
                assert!(
                    !input.is_grabbed(),
                    "Device should not be grabbed initially"
                );
                return;
            }
        }

        panic!("No accessible input devices for testing");
    }

    /// Test accessor methods on a real device
    #[test]
    #[ignore = "requires real input device - run manually"]
    fn test_evdevinput_accessors() {
        use std::path::Path;

        for i in 0..20 {
            let path_str = format!("/dev/input/event{}", i);
            let path = Path::new(&path_str);
            if let Ok(input) = EvdevInput::open(path) {
                // Name should never be empty (fallback is "Unknown Device")
                assert!(!input.name().is_empty());

                // Path should match what we opened with
                assert_eq!(input.path(), path);

                // Serial may or may not be available
                println!(
                    "Device {} - Name: '{}', Serial: {:?}",
                    i,
                    input.name(),
                    input.serial()
                );

                // device() should return a valid reference
                let _device_ref = input.device();

                return;
            }
        }

        panic!("No accessible input devices for testing");
    }

    // ============================================
    // UinputOutput Tests
    // ============================================

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
    #[ignore = "requires uinput access - run manually with: sudo cargo test -p keyrx_daemon --features linux test_uinputoutput_create -- --ignored"]
    fn test_uinputoutput_create() {
        let output =
            UinputOutput::create("keyrx-test-keyboard").expect("Failed to create uinput device");

        assert_eq!(output.name(), "keyrx-test-keyboard");
        assert!(!output.is_destroyed());
        assert!(output.held_keys().is_empty());
    }

    /// Test that inject_event fails after destroy
    /// Note: Requires uinput access
    #[test]
    #[ignore = "requires uinput access - run manually"]
    fn test_uinputoutput_inject_after_destroy_fails() {
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
    #[ignore = "requires uinput access - run manually"]
    fn test_uinputoutput_held_keys_tracking() {
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
    #[ignore = "requires uinput access - run manually"]
    fn test_uinputoutput_destroy_releases_held_keys() {
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
    #[ignore = "requires uinput access - run manually"]
    fn test_uinputoutput_destroy_twice_is_safe() {
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
    #[ignore = "requires uinput access - run manually"]
    fn test_uinputoutput_drop_calls_destroy() {
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
}
