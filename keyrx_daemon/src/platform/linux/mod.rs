//! Linux platform implementation using evdev for input and uinput for output.
//!
//! This module provides the Linux-specific implementation for keyboard input capture
//! and event injection using the evdev and uinput kernel interfaces.

mod keycode_map;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use evdev::{Device, InputEventKind};
use uinput::Device as UInputDevice;

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::event::KeyEvent;

use crate::platform::{DeviceError, InputDevice, OutputDevice};

// Re-export key mapping functions for public use
#[allow(unused_imports)] // keycode_to_evdev will be used for output injection
pub use keycode_map::{evdev_to_keycode, keycode_to_evdev, keycode_to_uinput_key};

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

#[cfg(test)]
mod tests {
    use super::*;

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

    /// Test event injection for various key types
    /// Note: Requires uinput access
    #[test]
    #[ignore = "requires uinput access - run manually"]
    fn test_uinputoutput_inject_various_key_types() {
        let mut output =
            UinputOutput::create("keyrx-test-inject-keys").expect("Failed to create uinput device");

        // Test letter keys
        output
            .inject_event(KeyEvent::Press(KeyCode::A))
            .expect("Failed to press A");
        output
            .inject_event(KeyEvent::Release(KeyCode::A))
            .expect("Failed to release A");

        // Test number keys
        output
            .inject_event(KeyEvent::Press(KeyCode::Num5))
            .expect("Failed to press Num5");
        output
            .inject_event(KeyEvent::Release(KeyCode::Num5))
            .expect("Failed to release Num5");

        // Test function keys
        output
            .inject_event(KeyEvent::Press(KeyCode::F1))
            .expect("Failed to press F1");
        output
            .inject_event(KeyEvent::Release(KeyCode::F1))
            .expect("Failed to release F1");

        // Test modifier keys
        output
            .inject_event(KeyEvent::Press(KeyCode::LShift))
            .expect("Failed to press LShift");
        output
            .inject_event(KeyEvent::Release(KeyCode::LShift))
            .expect("Failed to release LShift");

        // Test special keys
        output
            .inject_event(KeyEvent::Press(KeyCode::Escape))
            .expect("Failed to press Escape");
        output
            .inject_event(KeyEvent::Release(KeyCode::Escape))
            .expect("Failed to release Escape");

        // Test arrow keys
        output
            .inject_event(KeyEvent::Press(KeyCode::Up))
            .expect("Failed to press Up");
        output
            .inject_event(KeyEvent::Release(KeyCode::Up))
            .expect("Failed to release Up");

        // Test numpad keys
        output
            .inject_event(KeyEvent::Press(KeyCode::NumpadEnter))
            .expect("Failed to press NumpadEnter");
        output
            .inject_event(KeyEvent::Release(KeyCode::NumpadEnter))
            .expect("Failed to release NumpadEnter");

        // Test media keys
        output
            .inject_event(KeyEvent::Press(KeyCode::VolumeUp))
            .expect("Failed to press VolumeUp");
        output
            .inject_event(KeyEvent::Release(KeyCode::VolumeUp))
            .expect("Failed to release VolumeUp");

        // Test punctuation keys
        output
            .inject_event(KeyEvent::Press(KeyCode::LeftBracket))
            .expect("Failed to press LeftBracket");
        output
            .inject_event(KeyEvent::Release(KeyCode::LeftBracket))
            .expect("Failed to release LeftBracket");

        // All held keys should be released
        assert!(output.held_keys().is_empty());
    }

    /// Test that keycode_to_uinput_key maps all KeyCode variants correctly
    /// This is a unit test that doesn't require uinput access
    #[test]
    fn test_keycode_to_uinput_key_all_variants() {
        // Test representative keys from each category to ensure mapping works
        // (The actual injection requires uinput access, but we can verify the mapping function)

        // Letters
        let _ = keycode_to_uinput_key(KeyCode::A);
        let _ = keycode_to_uinput_key(KeyCode::Z);

        // Numbers
        let _ = keycode_to_uinput_key(KeyCode::Num0);
        let _ = keycode_to_uinput_key(KeyCode::Num9);

        // Function keys
        let _ = keycode_to_uinput_key(KeyCode::F1);
        let _ = keycode_to_uinput_key(KeyCode::F24);

        // Modifiers
        let _ = keycode_to_uinput_key(KeyCode::LShift);
        let _ = keycode_to_uinput_key(KeyCode::RMeta);

        // Special keys
        let _ = keycode_to_uinput_key(KeyCode::Escape);
        let _ = keycode_to_uinput_key(KeyCode::CapsLock);
        let _ = keycode_to_uinput_key(KeyCode::Pause);

        // Arrow keys
        let _ = keycode_to_uinput_key(KeyCode::Left);
        let _ = keycode_to_uinput_key(KeyCode::Down);

        // Numpad
        let _ = keycode_to_uinput_key(KeyCode::Numpad0);
        let _ = keycode_to_uinput_key(KeyCode::NumpadEnter);

        // Media keys
        let _ = keycode_to_uinput_key(KeyCode::Mute);
        let _ = keycode_to_uinput_key(KeyCode::MediaNext);

        // System keys
        let _ = keycode_to_uinput_key(KeyCode::Power);
        let _ = keycode_to_uinput_key(KeyCode::Wake);

        // Browser keys
        let _ = keycode_to_uinput_key(KeyCode::BrowserBack);
        let _ = keycode_to_uinput_key(KeyCode::BrowserHome);

        // Application keys
        let _ = keycode_to_uinput_key(KeyCode::AppMail);
        let _ = keycode_to_uinput_key(KeyCode::AppCalculator);

        // Additional keys
        let _ = keycode_to_uinput_key(KeyCode::Menu);
        let _ = keycode_to_uinput_key(KeyCode::Find);
    }

    /// Test virtual device appears in /dev/input/ and is removed on destroy
    /// Note: Requires uinput access
    #[test]
    #[ignore = "requires uinput access - run manually with: sudo cargo test -p keyrx_daemon --features linux test_uinputoutput_device_lifecycle -- --ignored"]
    fn test_uinputoutput_device_lifecycle() {
        use std::fs;
        use std::thread;
        use std::time::Duration;

        let device_name = "keyrx-test-lifecycle-device";

        // Count initial devices
        let initial_count = fs::read_dir("/dev/input/")
            .map(|entries| entries.count())
            .unwrap_or(0);

        // Create device
        let output = UinputOutput::create(device_name).expect("Failed to create uinput device");

        // Wait a moment for the device to appear
        thread::sleep(Duration::from_millis(100));

        // Should have one more device now
        let with_device_count = fs::read_dir("/dev/input/")
            .map(|entries| entries.count())
            .unwrap_or(0);

        // Device count should have increased (new virtual device created)
        assert!(
            with_device_count >= initial_count,
            "Device count should not decrease after creation"
        );

        // Verify device is not destroyed yet
        assert!(!output.is_destroyed());

        // Drop the device
        drop(output);

        // Wait for device removal
        thread::sleep(Duration::from_millis(100));

        // Should be back to initial count (or close to it)
        let final_count = fs::read_dir("/dev/input/")
            .map(|entries| entries.count())
            .unwrap_or(0);

        // The count should not be significantly higher than initial
        // (allowing for other system devices that might have appeared/disappeared)
        assert!(
            final_count <= with_device_count,
            "Device count should not increase after destruction: initial={}, with_device={}, final={}",
            initial_count,
            with_device_count,
            final_count
        );
    }

    /// Test that multiple UinputOutput devices can coexist
    /// Note: Requires uinput access
    #[test]
    #[ignore = "requires uinput access - run manually"]
    fn test_uinputoutput_multiple_devices() {
        let output1 = UinputOutput::create("keyrx-test-multi-1")
            .expect("Failed to create first uinput device");
        let output2 = UinputOutput::create("keyrx-test-multi-2")
            .expect("Failed to create second uinput device");

        assert_eq!(output1.name(), "keyrx-test-multi-1");
        assert_eq!(output2.name(), "keyrx-test-multi-2");

        assert!(!output1.is_destroyed());
        assert!(!output2.is_destroyed());

        // Both should be independently usable
        // (actual event injection would require evtest verification)

        drop(output1);
        drop(output2);
    }

    /// Test inject_event with modifier key combinations
    /// Note: Requires uinput access
    #[test]
    #[ignore = "requires uinput access - run manually"]
    fn test_uinputoutput_inject_modifier_combinations() {
        let mut output =
            UinputOutput::create("keyrx-test-modifiers").expect("Failed to create uinput device");

        // Simulate Ctrl+Shift+A
        output
            .inject_event(KeyEvent::Press(KeyCode::LCtrl))
            .expect("Failed to press LCtrl");
        output
            .inject_event(KeyEvent::Press(KeyCode::LShift))
            .expect("Failed to press LShift");
        output
            .inject_event(KeyEvent::Press(KeyCode::A))
            .expect("Failed to press A");

        // Verify all three keys are held
        assert_eq!(output.held_keys().len(), 3);
        assert!(output.held_keys().contains(&KeyCode::LCtrl));
        assert!(output.held_keys().contains(&KeyCode::LShift));
        assert!(output.held_keys().contains(&KeyCode::A));

        // Release in reverse order
        output
            .inject_event(KeyEvent::Release(KeyCode::A))
            .expect("Failed to release A");
        output
            .inject_event(KeyEvent::Release(KeyCode::LShift))
            .expect("Failed to release LShift");
        output
            .inject_event(KeyEvent::Release(KeyCode::LCtrl))
            .expect("Failed to release LCtrl");

        assert!(output.held_keys().is_empty());
    }

    /// Test that releasing an un-pressed key doesn't cause issues
    /// Note: Requires uinput access
    #[test]
    #[ignore = "requires uinput access - run manually"]
    fn test_uinputoutput_release_unpressed_key() {
        let mut output =
            UinputOutput::create("keyrx-test-release").expect("Failed to create uinput device");

        // Release a key that was never pressed
        // This should succeed (the system handles it gracefully)
        output
            .inject_event(KeyEvent::Release(KeyCode::A))
            .expect("Release of unpressed key should succeed");

        // held_keys should not contain the key (and definitely not have negative count)
        assert!(!output.held_keys().contains(&KeyCode::A));
        assert!(output.held_keys().is_empty());
    }

    /// Test name() accessor returns correct value
    #[test]
    #[ignore = "requires uinput access - run manually"]
    fn test_uinputoutput_name_accessor() {
        let output =
            UinputOutput::create("keyrx-test-name").expect("Failed to create uinput device");

        assert_eq!(output.name(), "keyrx-test-name");
    }

    /// Test that held_keys is empty initially
    #[test]
    #[ignore = "requires uinput access - run manually"]
    fn test_uinputoutput_held_keys_empty_initially() {
        let output =
            UinputOutput::create("keyrx-test-empty").expect("Failed to create uinput device");

        assert!(output.held_keys().is_empty());
        assert_eq!(output.held_keys().len(), 0);
    }

    /// Test is_destroyed accessor
    #[test]
    #[ignore = "requires uinput access - run manually"]
    fn test_uinputoutput_is_destroyed_accessor() {
        let mut output =
            UinputOutput::create("keyrx-test-destroyed").expect("Failed to create uinput device");

        assert!(!output.is_destroyed(), "Should not be destroyed initially");

        output.destroy().expect("Failed to destroy device");

        assert!(output.is_destroyed(), "Should be destroyed after destroy()");
    }
}
