//! Linux input capture using evdev.
//!
//! This module provides keyboard event capture from Linux input devices via the evdev subsystem.

use std::collections::VecDeque;
use std::os::fd::{AsRawFd, BorrowedFd};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use evdev::{Device, InputEventKind};

use keyrx_core::runtime::event::KeyEvent;

use crate::platform::{DeviceError, InputDevice};

use super::keycode_map::evdev_to_keycode;

/// A single `(type, code, value)` triple this daemon could not turn into a
/// `KeyCode`-based [`KeyEvent`]: an `EV_KEY` code with no `KeyCode`
/// (brightness, mic-mute, vendor/consumer keys) or a non-key event type
/// entirely (e.g. `EV_REL` from a keyboard that doubles as a pointer).
/// Forwarded to the output device unchanged instead of being silently
/// dropped (H6) - see [`PendingCapture`] and
/// [`super::LinuxPlatform::next_raw_event`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RawPassthroughEvent {
    pub event_type: u16,
    pub code: u16,
    pub value: i32,
}

/// One item read off a device, in the order the kernel delivered it:
/// either a mapped key press/release, or a [`RawPassthroughEvent`] this
/// daemon does not interpret. Keeping both in one queue (rather than two
/// separate ones) is what lets the caller forward raw items in their
/// original position relative to the key events around them.
#[derive(Debug, Clone)]
pub(crate) enum PendingCapture {
    Key(KeyEvent),
    Raw(RawPassthroughEvent),
}

/// Converts a `SystemTime` to microseconds since UNIX epoch.
///
/// This is used to extract timestamps from evdev events for tap-hold
/// timing calculations. Falls back to 0 if the conversion fails
/// (e.g., for times before UNIX epoch).
fn systemtime_to_micros(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

/// errno `ENODEV` on Linux: the device was unplugged.
const ENODEV: i32 = 19;

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
/// use keyrx_daemon::platform::InputDevice;
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
/// keyboard.grab()?;
/// # Ok::<(), keyrx_daemon::platform::DeviceError>(())
/// ```
pub struct EvdevInput {
    /// The underlying evdev device handle.
    device: Device,
    /// Whether we have exclusive (grabbed) access to the device.
    grabbed: bool,
    /// Path to the device node (for identification).
    path: PathBuf,
    /// Events already read from the kernel but not yet returned. One
    /// `read()` can return several events; all of them are kept here, in
    /// order, so none are lost (a lost release is a stuck key) and raw
    /// passthrough items keep their position relative to the key events
    /// around them.
    pending: VecDeque<PendingCapture>,
}

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

        set_nonblocking(&device)?;
        Ok(Self {
            device,
            grabbed: false,
            path: path.to_path_buf(),
            pending: VecDeque::new(),
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

        if let Err(e) = set_nonblocking(&device) {
            log::warn!("Could not make {} non-blocking: {e}", path.display());
        }
        Self {
            device,
            grabbed: false,
            path,
            pending: VecDeque::new(),
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
impl EvdevInput {
    /// Borrowed fd for `poll(2)` across devices.
    pub fn poll_fd(&self) -> BorrowedFd<'_> {
        // SAFETY: `self.device` owns this fd and outlives the returned borrow.
        unsafe { BorrowedFd::borrow_raw(self.device.as_raw_fd()) }
    }

    /// Reads everything the kernel has buffered (non-blocking) and queues
    /// it, in order. Returns `Ok` with nothing queued when idle.
    fn read_available(&mut self) -> Result<(), DeviceError> {
        let events = match self.device.fetch_events() {
            Ok(events) => events,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
            Err(e) => return Err(DeviceError::Io(e)),
        };
        for event in events {
            match event.kind() {
                InputEventKind::Synchronization(_) | InputEventKind::Misc(_) => continue,
                InputEventKind::Key(key) => {
                    let Some(keycode) = evdev_to_keycode(key.code()) else {
                        // No `KeyCode` for this code (brightness, mic-mute,
                        // vendor/consumer keys, mouse buttons on a combo
                        // device, ...): forward it raw instead of dropping
                        // it (H6). We don't interpret its value (press,
                        // release or autorepeat), so all three pass through.
                        self.pending
                            .push_back(PendingCapture::Raw(RawPassthroughEvent {
                                event_type: event.event_type().0,
                                code: key.code(),
                                value: event.value(),
                            }));
                        continue;
                    };
                    let timestamp_us = systemtime_to_micros(event.timestamp());
                    // value: 1 = press, 0 = release, 2 = autorepeat (ignored
                    // - the desktop repeats a mapped key on its own; see
                    // module docs).
                    let key_event = match event.value() {
                        1 => KeyEvent::press(keycode),
                        0 => KeyEvent::release(keycode),
                        _ => continue,
                    };
                    self.pending
                        .push_back(PendingCapture::Key(key_event.with_timestamp(timestamp_us)));
                }
                _ => {
                    // Non-key event on a grabbed keyboard node (EV_REL from
                    // a keyboard with an integrated pointer/trackpoint,
                    // EV_ABS, EV_SW, EV_LED, ...): forward it raw (H6).
                    self.pending
                        .push_back(PendingCapture::Raw(RawPassthroughEvent {
                            event_type: event.event_type().0,
                            code: event.code(),
                            value: event.value(),
                        }));
                }
            }
        }
        Ok(())
    }

    /// Like [`InputDevice::next_event`], but returns the next queued item
    /// as-is instead of dropping [`PendingCapture::Raw`] entries: this is
    /// what lets a raw passthrough event keep its original position
    /// relative to the key events around it. The only production caller is
    /// `LinuxPlatform::next_raw_event`.
    pub(crate) fn next_pending(&mut self) -> Result<PendingCapture, DeviceError> {
        if self.pending.is_empty() {
            self.read_available()?;
        }
        self.pending.pop_front().ok_or(DeviceError::EndOfStream)
    }
}

/// Puts the device fd in non-blocking mode so one idle keyboard can never
/// stall reads from the others; waiting is done with `poll(2)` instead.
fn set_nonblocking(device: &Device) -> Result<(), DeviceError> {
    use nix::fcntl::{fcntl, FcntlArg, OFlag};
    let fd = device.as_raw_fd();
    let flags = fcntl(fd, FcntlArg::F_GETFL).map_err(|e| DeviceError::Io(e.into()))?;
    let flags = OFlag::from_bits_truncate(flags) | OFlag::O_NONBLOCK;
    fcntl(fd, FcntlArg::F_SETFL(flags)).map_err(|e| DeviceError::Io(e.into()))?;
    Ok(())
}

impl InputDevice for EvdevInput {
    /// Returns the next queued key press/release without blocking.
    ///
    /// Autorepeat (value=2) of a mapped key is skipped. Events this daemon
    /// cannot represent as a `KeyCode`-based [`KeyEvent`] are skipped too -
    /// callers that need those (the production capture path) use
    /// [`EvdevInput::next_pending`] instead, which returns them as
    /// [`PendingCapture::Raw`].
    ///
    /// # Returns
    ///
    /// - `Ok(event)` for the oldest pending press/release
    /// - `Err(DeviceError::EndOfStream)` when no input is available right now
    /// - `Err(DeviceError::Io)` on I/O errors (e.g. device unplugged)
    fn next_event(&mut self) -> Result<KeyEvent, DeviceError> {
        loop {
            if let Some(item) = self.pending.pop_front() {
                match item {
                    PendingCapture::Key(event) => return Ok(event),
                    PendingCapture::Raw(_) => continue,
                }
            }
            self.read_available()?;
            if self.pending.is_empty() {
                return Err(DeviceError::EndOfStream);
            }
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

        match self.device.ungrab() {
            Ok(()) => {}
            // The device is gone (unplugged): the kernel dropped the grab
            // with it, so there is nothing left to release.
            Err(e) if e.raw_os_error() == Some(ENODEV) => {}
            Err(e) => return Err(DeviceError::Io(e)),
        }
        self.grabbed = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::time::Duration;

    /// Checks if input devices are accessible for reading.
    fn can_access_input_devices() -> bool {
        for i in 0..20 {
            let path = format!("/dev/input/event{}", i);
            if OpenOptions::new().read(true).open(&path).is_ok() {
                return true;
            }
        }
        false
    }

    // ============================================
    // Timestamp Conversion Tests
    // ============================================

    /// Test systemtime_to_micros with a known timestamp
    #[test]
    fn test_systemtime_to_micros_valid() {
        // Create a SystemTime 1 second after UNIX epoch
        let time = UNIX_EPOCH + Duration::from_secs(1);
        let micros = systemtime_to_micros(time);
        assert_eq!(micros, 1_000_000);

        // Create a SystemTime 1.5 seconds after UNIX epoch
        let time = UNIX_EPOCH + Duration::from_micros(1_500_000);
        let micros = systemtime_to_micros(time);
        assert_eq!(micros, 1_500_000);
    }

    /// Test systemtime_to_micros at UNIX epoch
    #[test]
    fn test_systemtime_to_micros_epoch() {
        let micros = systemtime_to_micros(UNIX_EPOCH);
        assert_eq!(micros, 0);
    }

    /// Test systemtime_to_micros with current time (should be non-zero)
    #[test]
    fn test_systemtime_to_micros_now() {
        let now = SystemTime::now();
        let micros = systemtime_to_micros(now);
        // Should be a large number (billions of microseconds since 1970)
        assert!(
            micros > 1_000_000_000_000_000,
            "Timestamp should be in reasonable range"
        );
    }

    // ============================================
    // EvdevInput Tests
    // ============================================

    /// Test that opening a non-existent device returns NotFound error
    #[test]
    fn test_evdevinput_open_not_found() {
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
    #[test]
    fn test_evdevinput_from_device() {
        // Runtime skip if no input device access
        let has_input_access = (0..20).any(|i| {
            OpenOptions::new()
                .read(true)
                .open(format!("/dev/input/event{}", i))
                .is_ok()
        });
        if !has_input_access {
            eprintln!(
                "SKIPPED: test_evdevinput_from_device - input devices not accessible (add user to 'input' group or run with sudo)"
            );
            return;
        }
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
        // Skip test if running as root (root can access all devices)
        if std::process::Command::new("id")
            .arg("-u")
            .output()
            .map(|o| o.stdout.starts_with(b"0"))
            .unwrap_or(false)
        {
            eprintln!("Skipping test: running as root, cannot test permission denied");
            return;
        }

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

        // If we get here, all devices were accessible - this is fine when
        // running with proper group permissions
        eprintln!("Test inconclusive: all devices accessible (user likely in input group)");
    }

    /// Test that is_grabbed returns false initially
    #[test]
    fn test_evdevinput_not_grabbed_initially() {
        if !can_access_input_devices() {
            eprintln!("SKIPPED: input devices not accessible");
            return;
        }

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
    fn test_evdevinput_accessors() {
        if !can_access_input_devices() {
            eprintln!("SKIPPED: input devices not accessible");
            return;
        }

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

    /// Regression: reads used to block, so one idle keyboard stalled every
    /// other grabbed keyboard, and events after the first key of a `read()`
    /// batch were held back (a release stuck until the next keystroke).
    #[test]
    fn test_nonblocking_reads_and_no_lost_batch_events() {
        crate::skip_if_no_uinput!();
        use crate::test_utils::output_capture::OutputCapture;
        use crate::test_utils::VirtualKeyboard;
        use keyrx_core::config::KeyCode;
        use std::time::{Duration, Instant};

        let mut keyboard = VirtualKeyboard::create("nonblocking-capture-test").unwrap();
        std::thread::sleep(Duration::from_millis(250));
        let path = OutputCapture::find_by_name(keyboard.name(), Duration::from_secs(5))
            .unwrap()
            .device_path()
            .to_path_buf();
        let mut input = EvdevInput::open(&path).unwrap();

        // Idle device: returns at once instead of blocking.
        let start = Instant::now();
        assert!(matches!(input.next_event(), Err(DeviceError::EndOfStream)));
        assert!(start.elapsed() < Duration::from_millis(100));

        // Press + release land in one kernel read; both must be delivered.
        keyboard.inject(KeyEvent::press(KeyCode::A)).unwrap();
        keyboard.inject(KeyEvent::release(KeyCode::A)).unwrap();
        std::thread::sleep(Duration::from_millis(50));
        let first = input.next_event().unwrap();
        let second = input.next_event().unwrap();
        assert!(
            first.is_press() && first.keycode() == KeyCode::A,
            "{first:?}"
        );
        assert!(
            !second.is_press() && second.keycode() == KeyCode::A,
            "{second:?}"
        );
        assert!(matches!(input.next_event(), Err(DeviceError::EndOfStream)));
    }

    /// Releasing the grab of an unplugged device is not an error: the kernel
    /// already dropped it (it used to log a spurious "Failed to release").
    #[test]
    fn release_of_an_unplugged_device_succeeds() {
        use crate::test_utils::output_capture::OutputCapture;
        use crate::test_utils::VirtualKeyboard;
        use std::time::Duration;

        let mut keyboard = VirtualKeyboard::create("release-unplugged-test").unwrap();
        std::thread::sleep(Duration::from_millis(250));
        let path = OutputCapture::find_by_name(keyboard.name(), Duration::from_secs(5))
            .unwrap()
            .device_path()
            .to_path_buf();
        let mut input = EvdevInput::open(&path).unwrap();
        input.grab().unwrap();
        keyboard.destroy().unwrap();
        std::thread::sleep(Duration::from_millis(250));

        input.release().expect("ENODEV on release must be ignored");
        assert!(!input.is_grabbed());
    }
}
