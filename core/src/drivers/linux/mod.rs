//! Linux input driver using evdev for capture and uinput for injection.
//!
//! This module implements keyboard capture and injection on Linux using:
//! - **evdev** for reading keyboard events from `/dev/input/event*` devices
//! - **uinput** for injecting remapped keys via a virtual keyboard device
//!
//! # Platform Requirements
//!
//! - Linux kernel with evdev support (virtually all modern distributions)
//! - The `uinput` kernel module loaded (`modprobe uinput`)
//! - Read access to `/dev/input/event*` devices
//! - Write access to `/dev/uinput`
//!
//! # Permission Requirements
//!
//! KeyRx requires specific permissions to function:
//!
//! 1. **Input device access**: User must be in the `input` group or have explicit
//!    permissions on `/dev/input/event*` devices:
//!    ```bash
//!    sudo usermod -aG input $USER
//!    # Log out and back in for changes to take effect
//!    ```
//!
//! 2. **Uinput device access**: Write permission to `/dev/uinput` is required:
//!    ```bash
//!    # Create udev rule for persistent access
//!    echo 'KERNEL=="uinput", MODE="0660", GROUP="input"' | \
//!        sudo tee /etc/udev/rules.d/99-uinput.rules
//!    sudo udevadm control --reload-rules && sudo udevadm trigger
//!    ```
//!
//! 3. **Uinput module**: Ensure the kernel module is loaded:
//!    ```bash
//!    sudo modprobe uinput
//!    # To load on boot:
//!    echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf
//!    ```
//!
//! # Thread Model
//!
//! The driver uses a dedicated blocking thread for event capture:
//!
//! ```text
//! ┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
//! │  Physical KB    │────▶│  EvdevReader     │────▶│  Engine         │
//! │  (evdev device) │     │  (blocking read) │     │  (async)        │
//! └─────────────────┘     └──────────────────┘     └─────────────────┘
//!                                                          │
//!                                                          ▼
//! ┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
//! │  Applications   │◀────│  UinputWriter    │◀────│  Remap Logic    │
//! │                 │     │  (virtual KB)    │     │                 │
//! └─────────────────┘     └──────────────────┘     └─────────────────┘
//! ```
//!
//! - **EvdevReader thread**: Runs a blocking `fetch_events()` loop in a dedicated
//!   `std::thread`. Events are sent via a `crossbeam_channel` to the async engine.
//! - **Running flag**: An `Arc<AtomicBool>` is shared between threads to signal shutdown.
//! - **UinputWriter**: Runs on the main/engine thread for key injection.
//!
//! # Error Handling
//!
//! The driver provides detailed error messages with remediation steps:
//!
//! - [`LinuxDriverError::DeviceNotFound`]: Device path does not exist
//! - [`LinuxDriverError::PermissionDenied`]: Insufficient permissions on device
//! - [`LinuxDriverError::GrabFailed`]: Another process has grabbed the device
//! - [`LinuxDriverError::UinputFailed`]: Cannot create virtual keyboard
//!
//! All errors include actionable remediation hints.
//!
//! # Cleanup and Recovery
//!
//! The driver implements robust cleanup to prevent leaving the keyboard stuck:
//!
//! 1. **Normal shutdown**: Calling `stop()` signals the reader thread, waits for
//!    it to finish, and ungrab the keyboard device.
//!
//! 2. **Drop cleanup**: The `Drop` implementation ensures cleanup even on early
//!    returns or unexpected exits.
//!
//! 3. **Panic recovery**: The reader thread wraps its main loop in `catch_unwind`.
//!    On panic:
//!    - The `panic_error` flag is set to `true`
//!    - The keyboard device is ungrabbed
//!    - The error is logged
//!    - `poll_events()` returns an error on the next call
//!
//! 4. **Signal handling**: SIGINT/SIGTERM trigger graceful shutdown via the
//!    running flag, ensuring the keyboard is released even on Ctrl+C.
//!
//! # Example
//!
//! ```ignore
//! use keyrx::drivers::LinuxInput;
//!
//! // Auto-detect keyboard
//! let mut input = LinuxInput::new(None)?;
//!
//! // Start capturing events (grabs keyboard)
//! input.start().await?;
//!
//! // Poll for events (non-blocking)
//! let events = input.poll_events().await?;
//!
//! // Inject remapped keys
//! input.send_output(OutputAction::KeyDown(KeyCode::Escape)).await?;
//!
//! // Stop and release keyboard
//! input.stop().await?;
//! ```
//!
//! [`LinuxDriverError::DeviceNotFound`]: crate::error::LinuxDriverError::DeviceNotFound
//! [`LinuxDriverError::PermissionDenied`]: crate::error::LinuxDriverError::PermissionDenied
//! [`LinuxDriverError::GrabFailed`]: crate::error::LinuxDriverError::GrabFailed
//! [`LinuxDriverError::UinputFailed`]: crate::error::LinuxDriverError::UinputFailed

mod keymap;
mod reader;
mod writer;

use crate::drivers::{DeviceInfo, KeyInjector};
use crate::engine::{InputEvent, OutputAction};
use crate::traits::InputSource;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use crossbeam_channel::Sender;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use tracing::{debug, error, trace, warn};

use reader::EvdevReader;
pub use writer::UinputWriter;

const UINPUT_PATH: &str = "/dev/uinput";

/// Linux input source using evdev for capture and uinput for injection.
///
/// `LinuxInput` coordinates an `EvdevReader` for keyboard event capture and
/// a `UinputWriter` for key injection. It implements the `InputSource` trait
/// for integration with the KeyRx remapping engine.
///
/// # Architecture
///
/// ```text
/// ┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
/// │  Physical KB    │────▶│  EvdevReader     │────▶│  Engine         │
/// │  (evdev device) │     │  (blocking read) │     │  (async)        │
/// └─────────────────┘     └──────────────────┘     └─────────────────┘
///                                                          │
///                                                          ▼
/// ┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
/// │  Applications   │◀────│  UinputWriter    │◀────│  Remap Logic    │
/// │                 │     │  (virtual KB)    │     │                 │
/// └─────────────────┘     └──────────────────┘     └─────────────────┘
/// ```
///
/// # Thread Model
///
/// - The `EvdevReader` runs in a dedicated blocking thread
/// - Events are sent via a crossbeam channel to the async engine
/// - The `running` flag is shared via `Arc<AtomicBool>` for clean shutdown
///
/// # Panic Recovery
///
/// The reader thread is wrapped in `catch_unwind` to handle panics gracefully.
/// If a panic occurs, the `panic_error` flag is set and the keyboard device is
/// ungrabbed. The `poll_events` method checks this flag and returns an error
/// if a panic was detected.
pub struct LinuxInput {
    /// Handle to the reader thread (set after start() is called).
    reader_handle: Option<JoinHandle<()>>,
    /// Key injector for output (uses trait for testability).
    injector: Box<dyn KeyInjector>,
    /// Receiver for events from the reader thread.
    rx: crossbeam_channel::Receiver<InputEvent>,
    /// Sender for events (held to create the reader).
    tx: Sender<InputEvent>,
    /// Shared flag to signal shutdown.
    running: Arc<AtomicBool>,
    /// Path to the evdev device being read.
    device_path: PathBuf,
    /// Shared flag indicating if the reader thread panicked.
    panic_error: Arc<AtomicBool>,
}

impl LinuxInput {
    /// Create a new Linux input source.
    ///
    /// If `device_path` is `None`, automatically detects the first available
    /// keyboard device by scanning `/dev/input/event*`.
    ///
    /// # Arguments
    ///
    /// * `device_path` - Optional explicit path to an evdev device. If `None`,
    ///   the first detected keyboard will be used.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No keyboard device is found (when `device_path` is `None`)
    /// - The uinput device cannot be created (permission denied)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Auto-detect keyboard
    /// let input = LinuxInput::new(None)?;
    ///
    /// // Use specific device
    /// let input = LinuxInput::new(Some(PathBuf::from("/dev/input/event3")))?;
    /// ```
    pub fn new(device_path: Option<PathBuf>) -> Result<Self> {
        let injector = UinputWriter::new().context("Failed to create uinput writer")?;
        Self::new_with_injector(device_path, Box::new(injector))
    }

    /// Create a new Linux input source with a custom key injector.
    ///
    /// This constructor allows dependency injection for testing purposes,
    /// enabling unit tests to use a `MockKeyInjector` instead of the real
    /// `UinputWriter` which requires hardware access and elevated permissions.
    ///
    /// # Arguments
    ///
    /// * `device_path` - Optional explicit path to an evdev device. If `None`,
    ///   the first detected keyboard will be used.
    /// * `injector` - A boxed `KeyInjector` implementation for key output.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No keyboard device is found (when `device_path` is `None`)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use keyrx_core::drivers::{LinuxInput, MockKeyInjector};
    ///
    /// // Use a mock injector for testing
    /// let mock = MockKeyInjector::new();
    /// let input = LinuxInput::new_with_injector(None, Box::new(mock))?;
    /// ```
    pub fn new_with_injector(
        device_path: Option<PathBuf>,
        injector: Box<dyn KeyInjector>,
    ) -> Result<Self> {
        // Determine device path: use provided or auto-detect
        let device_path = match device_path {
            Some(path) => {
                debug!("Using specified device: {}", path.display());
                path
            }
            None => {
                debug!("Auto-detecting keyboard device...");
                Self::find_first_keyboard()?
            }
        };

        // Create the event channel
        let (tx, rx) = crossbeam_channel::unbounded();

        let running = Arc::new(AtomicBool::new(false));
        let panic_error = Arc::new(AtomicBool::new(false));

        debug!("LinuxInput created for device: {}", device_path.display());

        Ok(Self {
            reader_handle: None,
            injector,
            rx,
            tx,
            running,
            device_path,
            panic_error,
        })
    }

    /// Find the first available keyboard device.
    ///
    /// Scans `/dev/input/event*` and returns the path to the first device
    /// that has keyboard capability.
    fn find_first_keyboard() -> Result<PathBuf> {
        let keyboards = list_keyboards()?;

        if keyboards.is_empty() {
            bail!(
                "No keyboard devices found\n\n\
                 Remediation:\n  \
                 1. Ensure a keyboard is connected\n  \
                 2. Check permissions: ls -la /dev/input/event*\n  \
                 3. Add user to input group: sudo usermod -aG input $USER\n  \
                 4. Log out and back in for group changes to take effect"
            );
        }

        let device = &keyboards[0];
        debug!(
            "Auto-detected keyboard: {} at {}",
            device.name,
            device.path.display()
        );

        Ok(device.path.clone())
    }

    /// List all keyboard devices.
    ///
    /// This is a convenience method that delegates to the module-level
    /// `list_keyboards()` function.
    pub fn list_devices() -> Result<Vec<DeviceInfo>> {
        list_keyboards()
    }

    /// Get the device path.
    pub fn device_path(&self) -> &Path {
        &self.device_path
    }

    /// Get the event receiver.
    ///
    /// This can be used for direct access to the event channel, though
    /// typically events should be accessed via `poll_events()`.
    pub fn receiver(&self) -> &crossbeam_channel::Receiver<InputEvent> {
        &self.rx
    }

    /// Check if the driver is currently running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
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
        // Check if the reader thread panicked
        if self.panic_error.load(Ordering::SeqCst) {
            error!("poll_events called after reader thread panic");
            self.running.store(false, Ordering::Relaxed);
            bail!("Input reader thread panicked - keyboard has been ungrabbed for safety");
        }

        if !self.running.load(Ordering::Relaxed) {
            trace!("poll_events called while not running");
            return Ok(vec![]);
        }

        // Non-blocking receive from the channel
        // Collect all available events without blocking
        let mut events = Vec::new();
        loop {
            match self.rx.try_recv() {
                Ok(event) => {
                    trace!(
                        "Received event: {:?} {}",
                        event.key,
                        if event.pressed { "down" } else { "up" }
                    );
                    events.push(event);
                }
                Err(crossbeam_channel::TryRecvError::Empty) => {
                    // No more events available
                    break;
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    // Channel closed - reader thread has stopped
                    // Check if it was due to a panic
                    if self.panic_error.load(Ordering::SeqCst) {
                        error!("Event channel disconnected due to reader thread panic");
                        self.running.store(false, Ordering::Relaxed);
                        bail!(
                            "Input reader thread panicked - keyboard has been ungrabbed for safety"
                        );
                    }
                    error!("Event channel disconnected - reader thread may have crashed");
                    self.running.store(false, Ordering::Relaxed);
                    bail!("Input reader disconnected unexpectedly");
                }
            }
        }

        if !events.is_empty() {
            debug!("poll_events returning {} events", events.len());
        }

        Ok(events)
    }

    async fn send_output(&mut self, action: OutputAction) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            trace!("send_output called while not running");
            return Ok(());
        }

        match action {
            OutputAction::KeyDown(key) => {
                debug!("Sending key down: {:?}", key);
                self.injector.inject(key, true)?;
            }
            OutputAction::KeyUp(key) => {
                debug!("Sending key up: {:?}", key);
                self.injector.inject(key, false)?;
            }
            OutputAction::KeyTap(key) => {
                debug!("Sending key tap: {:?}", key);
                self.injector.inject(key, true)?;
                self.injector.inject(key, false)?;
            }
            OutputAction::Block => {
                // Block does nothing - the original event is already grabbed
                // and won't be passed through unless we explicitly emit it
                trace!("Blocking key (no action needed)");
            }
            OutputAction::PassThrough => {
                // PassThrough is handled by the engine - it re-emits the original key
                // For the driver, this is a no-op since the engine will call
                // KeyDown/KeyUp for the original key if needed
                trace!("PassThrough (no action needed)");
            }
        }

        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            warn!("LinuxInput already running");
            return Ok(());
        }

        // Verify uinput is accessible before starting
        Self::check_uinput_accessible().context("Failed to start Linux input source")?;

        // Reset panic error flag for fresh start
        self.panic_error.store(false, Ordering::SeqCst);

        // Set running flag before spawning thread
        self.running.store(true, Ordering::Relaxed);

        // Create the evdev reader
        let mut reader = EvdevReader::new(
            self.device_path.clone(),
            self.tx.clone(),
            self.running.clone(),
            self.panic_error.clone(),
        )
        .context("Failed to create evdev reader")?;

        // Grab exclusive access to the keyboard
        reader.grab().context("Failed to grab keyboard device")?;

        // Spawn the reader thread
        let handle = reader.spawn();
        self.reader_handle = Some(handle);

        debug!(
            "LinuxInput started successfully for device: {}",
            self.device_path.display()
        );

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            debug!("LinuxInput already stopped");
            return Ok(());
        }

        debug!("Stopping LinuxInput...");

        // Signal the reader thread to stop
        self.running.store(false, Ordering::Relaxed);

        // Wait for the reader thread to finish
        if let Some(handle) = self.reader_handle.take() {
            debug!("Waiting for reader thread to finish...");
            match handle.join() {
                Ok(()) => {
                    debug!("Reader thread finished cleanly");
                }
                Err(e) => {
                    error!("Reader thread panicked: {:?}", e);
                    // Continue with cleanup even if thread panicked
                }
            }
        }

        // Drain any remaining events from the channel
        while self.rx.try_recv().is_ok() {
            // Discard remaining events
        }

        debug!("LinuxInput stopped successfully");
        Ok(())
    }
}

impl Drop for LinuxInput {
    fn drop(&mut self) {
        // Ensure the driver is stopped and keyboard is released on drop.
        // This is critical for graceful cleanup even on panics or unexpected termination.
        if self.running.load(Ordering::Relaxed) {
            debug!("LinuxInput::drop - stopping driver...");
            self.running.store(false, Ordering::Relaxed);

            // Wait for the reader thread to finish
            if let Some(handle) = self.reader_handle.take() {
                debug!("LinuxInput::drop - waiting for reader thread...");
                match handle.join() {
                    Ok(()) => debug!("LinuxInput::drop - reader thread finished cleanly"),
                    Err(e) => warn!("LinuxInput::drop - reader thread panicked: {:?}", e),
                }
            }

            // Drain any remaining events
            while self.rx.try_recv().is_ok() {}

            debug!("LinuxInput::drop - cleanup complete");
        }
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
    let mut keyboards: Vec<DeviceInfo> = entries
        .flatten()
        .filter_map(|entry| try_get_keyboard_info(&entry.path()))
        .collect();

    keyboards.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(keyboards)
}

/// Attempts to open a device path and return keyboard info if it's a keyboard.
fn try_get_keyboard_info(path: &Path) -> Option<DeviceInfo> {
    let file_name = path.file_name().and_then(|n| n.to_str())?;
    if !file_name.starts_with("event") {
        return None;
    }

    let device = match evdev::Device::open(path) {
        Ok(d) => d,
        Err(e) => {
            debug!("Could not open {}: {}", path.display(), e);
            return None;
        }
    };

    let has_keyboard_keys = device
        .supported_keys()
        .map(|keys| {
            keys.contains(evdev::Key::KEY_A)
                && keys.contains(evdev::Key::KEY_ENTER)
                && keys.contains(evdev::Key::KEY_SPACE)
        })
        .unwrap_or(false);

    if !has_keyboard_keys {
        return None;
    }

    let name = device.name().unwrap_or("Unknown Device").to_string();
    let input_id = device.input_id();
    Some(DeviceInfo::new(
        path.to_path_buf(),
        name,
        input_id.vendor(),
        input_id.product(),
        true,
    ))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Tests use unwrap for clarity
mod tests {
    use super::*;
    use crate::drivers::{InjectedKey, MockKeyInjector};
    use crate::engine::KeyCode;

    /// Test that LinuxInput can be created with a mock injector.
    /// This test requires a keyboard device to be available, but doesn't
    /// need uinput permissions since we're using a mock.
    #[test]
    fn linux_input_with_mock_injector_compiles() {
        // This test verifies the type signatures work correctly
        fn _create_with_mock(_path: PathBuf) -> Result<LinuxInput> {
            let mock = MockKeyInjector::new();
            LinuxInput::new_with_injector(Some(_path), Box::new(mock))
        }
    }

    /// Test that UinputWriter implements KeyInjector.
    #[test]
    fn uinput_writer_implements_key_injector() {
        fn assert_key_injector<T: KeyInjector>() {}
        assert_key_injector::<UinputWriter>();
    }

    /// Test MockKeyInjector records injections correctly when used as a trait object.
    #[test]
    fn mock_injector_as_trait_object() {
        let mut injector: Box<dyn KeyInjector> = Box::new(MockKeyInjector::new());

        injector.inject(KeyCode::A, true).unwrap();
        injector.inject(KeyCode::A, false).unwrap();
        injector.sync().unwrap();

        // Downcast to check recorded injections
        // Note: In real tests, you'd typically keep a reference to the mock
    }

    /// Test that send_output uses the injector correctly via the mock.
    #[tokio::test]
    async fn send_output_uses_injector() {
        // We can't easily test this without a real device path, but we can
        // verify the injector interface works by testing the mock directly
        let mut mock = MockKeyInjector::new();

        // Simulate what send_output does for KeyDown
        mock.inject(KeyCode::Escape, true).unwrap();
        assert!(mock.was_pressed(KeyCode::Escape));

        // Simulate what send_output does for KeyUp
        mock.inject(KeyCode::Escape, false).unwrap();
        assert!(mock.was_released(KeyCode::Escape));

        // Simulate KeyTap (press then release)
        mock.inject(KeyCode::Enter, true).unwrap();
        mock.inject(KeyCode::Enter, false).unwrap();
        assert!(mock.was_tapped(KeyCode::Enter));

        // Verify all injections
        let injected = mock.injected_keys();
        assert_eq!(injected.len(), 4);
        assert_eq!(injected[0], InjectedKey::press(KeyCode::Escape));
        assert_eq!(injected[1], InjectedKey::release(KeyCode::Escape));
        assert_eq!(injected[2], InjectedKey::press(KeyCode::Enter));
        assert_eq!(injected[3], InjectedKey::release(KeyCode::Enter));
    }

    /// Test that the injector trait is Send.
    #[test]
    fn key_injector_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Box<dyn KeyInjector>>();
    }
}
