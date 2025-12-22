//! Platform abstraction layer for keyboard input/output.
//!
//! This module defines traits for input and output devices, providing a
//! platform-agnostic interface for event processing. Platform-specific
//! implementations (Linux, Windows, Mock) implement these traits.

use keyrx_core::runtime::event::KeyEvent;
use thiserror::Error;

#[cfg(feature = "linux")]
pub mod linux;

#[cfg(feature = "windows")]
pub mod windows;

pub mod mock;

#[cfg(feature = "linux")]
pub use linux::LinuxPlatform;

#[cfg(feature = "windows")]
pub use windows::WindowsPlatform;

#[allow(unused_imports)] // Will be used in tasks #17-20
pub use mock::{MockInput, MockOutput};

/// Errors that can occur during device operations.
#[derive(Debug, Error)]
#[allow(dead_code)] // Will be used in tasks #14-20
pub enum DeviceError {
    /// Device was not found or could not be opened.
    #[error("device not found: {0}")]
    NotFound(String),

    /// Permission denied when accessing device.
    /// This typically occurs when the user lacks privileges to access input devices.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// End of event stream reached.
    /// For input devices, this indicates no more events are available.
    /// This is a normal termination condition, not an error.
    #[error("end of stream")]
    EndOfStream,

    /// Failed to inject event into output device.
    #[error("event injection failed: {0}")]
    InjectionFailed(String),

    /// I/O error occurred during device operation.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Input device trait for capturing keyboard events.
///
/// # Device Ownership
///
/// Input devices support exclusive access via the `grab()` and `release()` methods:
/// - `grab()`: Obtains exclusive access to the device, preventing other applications
///   from receiving events from this device.
/// - `release()`: Releases exclusive access, allowing other applications to receive events.
///
/// # Example
///
/// ```no_run
/// use keyrx_daemon::platform::{InputDevice, DeviceError};
///
/// fn process_input<I: InputDevice>(mut input: I) -> Result<(), DeviceError> {
///     input.grab()?;
///
///     loop {
///         match input.next_event() {
///             Ok(event) => println!("Event: {:?}", event),
///             Err(DeviceError::EndOfStream) => break,
///             Err(e) => return Err(e),
///         }
///     }
///
///     input.release()?;
///     Ok(())
/// }
/// ```
#[allow(dead_code)] // Will be implemented in tasks #14-16
pub trait InputDevice {
    /// Retrieves the next keyboard event from the device.
    ///
    /// # Returns
    ///
    /// - `Ok(KeyEvent)`: Successfully read an event
    /// - `Err(DeviceError::EndOfStream)`: No more events available (normal termination)
    /// - `Err(DeviceError::Io)`: I/O error occurred
    /// - `Err(DeviceError::PermissionDenied)`: Insufficient permissions
    fn next_event(&mut self) -> Result<KeyEvent, DeviceError>;

    /// Obtains exclusive access to the device.
    ///
    /// After calling this method, other applications will not receive events from
    /// this device until `release()` is called. This is essential for key remapping
    /// to prevent the original keystrokes from reaching applications.
    ///
    /// # Platform Notes
    ///
    /// - Linux: Uses `EVIOCGRAB` ioctl
    /// - Windows: Uses low-level keyboard hooks with event suppression
    /// - Mock: Sets an internal flag for testing
    ///
    /// # Errors
    ///
    /// - `DeviceError::PermissionDenied`: Insufficient privileges
    /// - `DeviceError::Io`: Underlying system call failed
    fn grab(&mut self) -> Result<(), DeviceError>;

    /// Releases exclusive access to the device.
    ///
    /// After calling this method, other applications can receive events from this device.
    ///
    /// # Errors
    ///
    /// - `DeviceError::Io`: Underlying system call failed
    fn release(&mut self) -> Result<(), DeviceError>;
}

/// Output device trait for injecting keyboard events.
///
/// Output devices emit synthetic keyboard events that appear to applications
/// as if they came from a real keyboard.
///
/// # Example
///
/// ```no_run
/// use keyrx_daemon::platform::{OutputDevice, DeviceError};
/// use keyrx_core::runtime::event::KeyEvent;
/// use keyrx_core::config::KeyCode;
///
/// fn inject_events<O: OutputDevice>(mut output: O) -> Result<(), DeviceError> {
///     output.inject_event(KeyEvent::Press(KeyCode::A))?;
///     output.inject_event(KeyEvent::Release(KeyCode::A))?;
///     Ok(())
/// }
/// ```
#[allow(dead_code)] // Will be implemented in tasks #15-16
pub trait OutputDevice {
    /// Injects a keyboard event into the output device.
    ///
    /// The injected event will be visible to all applications as if it came from
    /// a physical keyboard.
    ///
    /// # Platform Notes
    ///
    /// - Linux: Uses uinput subsystem to create virtual keyboard
    /// - Windows: Uses `SendInput` API
    /// - Mock: Appends event to internal buffer for testing
    ///
    /// # Errors
    ///
    /// - `DeviceError::InjectionFailed`: Failed to inject event
    /// - `DeviceError::Io`: Underlying system call failed
    fn inject_event(&mut self, event: KeyEvent) -> Result<(), DeviceError>;
}

#[allow(dead_code)]
pub enum Platform {
    #[cfg(feature = "linux")]
    Linux(LinuxPlatform),
    #[cfg(feature = "windows")]
    Windows(WindowsPlatform),
    #[cfg(not(any(feature = "linux", feature = "windows")))]
    Unsupported,
}

impl Platform {
    #[allow(dead_code)]
    pub fn new() -> Self {
        #[cfg(feature = "linux")]
        {
            Platform::Linux(LinuxPlatform::new())
        }
        #[cfg(all(feature = "windows", not(feature = "linux")))]
        {
            Platform::Windows(WindowsPlatform::new())
        }
        #[cfg(not(any(feature = "linux", feature = "windows")))]
        {
            Platform::Unsupported
        }
    }

    #[allow(dead_code)]
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            #[cfg(feature = "linux")]
            Platform::Linux(p) => p.init(),
            #[cfg(feature = "windows")]
            Platform::Windows(p) => p.init(),
            #[cfg(not(any(feature = "linux", feature = "windows")))]
            Platform::Unsupported => Ok(()),
        }
    }

    #[allow(dead_code)]
    pub fn process_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            #[cfg(feature = "linux")]
            Platform::Linux(p) => p.process_events(),
            #[cfg(feature = "windows")]
            Platform::Windows(p) => p.process_events(),
            #[cfg(not(any(feature = "linux", feature = "windows")))]
            Platform::Unsupported => Ok(()),
        }
    }
}

impl Default for Platform {
    fn default() -> Self {
        Self::new()
    }
}
