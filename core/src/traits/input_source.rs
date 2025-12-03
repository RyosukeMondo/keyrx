//! Input source trait for OS-level keyboard hooks.
//!
//! This module defines the [`InputSource`] trait, which abstracts over platform-specific
//! keyboard input capture mechanisms. Implementations handle the low-level details of
//! intercepting keyboard events and injecting synthetic key presses.
//!
//! # Thread Safety
//!
//! The `InputSource` trait requires `Send` because implementations are typically used
//! across async task boundaries. The remapping engine runs on a dedicated task and
//! communicates with the input source across thread boundaries.
//!
//! # Lifecycle
//!
//! An input source follows this lifecycle:
//!
//! 1. Create the input source (platform-specific initialization)
//! 2. Call [`InputSource::start`] to begin capturing keyboard events
//! 3. Poll for events with [`InputSource::poll_events`] in a loop
//! 4. Send output actions with [`InputSource::send_output`] as needed
//! 5. Call [`InputSource::stop`] to release resources and stop capture
//!
//! # Example Implementation Sketch
//!
//! ```ignore
//! use keyrx_core::traits::InputSource;
//! use keyrx_core::engine::{InputEvent, OutputAction};
//! use async_trait::async_trait;
//! use anyhow::Result;
//!
//! struct MyInput {
//!     // Platform-specific handles
//! }
//!
//! #[async_trait]
//! impl InputSource for MyInput {
//!     async fn start(&mut self) -> Result<()> {
//!         // Initialize keyboard hook/device
//!         // Return Err if device is busy or permissions are insufficient
//!         Ok(())
//!     }
//!
//!     async fn poll_events(&mut self) -> Result<Vec<InputEvent>> {
//!         // Read pending events from kernel/hook
//!         // Return empty vec if no events (non-blocking)
//!         Ok(vec![])
//!     }
//!
//!     async fn send_output(&mut self, action: OutputAction) -> Result<()> {
//!         // Inject synthetic key press/release
//!         Ok(())
//!     }
//!
//!     async fn stop(&mut self) -> Result<()> {
//!         // Release hook/device
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # Platform Implementations
//!
//! - **Windows**: Uses `WH_KEYBOARD_LL` low-level keyboard hook via Win32 API
//! - **Linux**: Uses evdev for input capture and uinput for synthetic events
//! - **Testing**: Use [`MockInput`](crate::mocks::MockInput) for unit tests

use crate::engine::{InputEvent, OutputAction};
use crate::errors::KeyrxError;
use async_trait::async_trait;

/// Trait for input sources (keyboard hooks, virtual devices).
///
/// This trait abstracts platform-specific keyboard input handling, allowing the
/// remapping engine to work identically across Windows, Linux, and in tests.
///
/// # Thread Safety
///
/// Implementations must be `Send` because the remapping engine may run on a different
/// thread than where the input source was created. This is required for async runtime
/// compatibility.
///
/// # Error Handling
///
/// All methods return `Result<T>` using the `anyhow` crate. Implementations should:
/// - Return meaningful error messages that help diagnose the issue
/// - Not panic on recoverable errors (device busy, temporary failures)
/// - Clean up resources appropriately when errors occur
///
/// # Implementations
///
/// - `WindowsInput`: WH_KEYBOARD_LL hook on Windows
/// - `LinuxInput`: evdev/uinput on Linux
/// - [`MockInput`](crate::mocks::MockInput): Test mock for simulation
#[async_trait]
pub trait InputSource: Send {
    /// Poll for pending input events.
    ///
    /// Returns all keyboard events that have occurred since the last poll.
    /// This method should be non-blocking; if no events are pending, return
    /// an empty vector.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<InputEvent>)` - Zero or more input events. Empty vec means no events.
    /// - `Err(_)` - Device error (disconnected, permission revoked, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The input device has been disconnected
    /// - The keyboard hook was uninstalled unexpectedly
    /// - An I/O error occurred reading from the device
    ///
    /// # Panics
    ///
    /// Should not panic. Device errors should be returned as `Err`.
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>, KeyrxError>;

    /// Send an output action to the OS.
    ///
    /// Injects a synthetic keyboard event or performs a control action.
    ///
    /// # Arguments
    ///
    /// * `action` - The output action to perform:
    ///   - `OutputAction::KeyDown(key)` - Simulate key press
    ///   - `OutputAction::KeyUp(key)` - Simulate key release
    ///   - `OutputAction::KeyTap(key)` - Press and release key
    ///   - `OutputAction::Block` - Consume/block the original input
    ///   - `OutputAction::PassThrough` - Let the original input through unchanged
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Action was successfully sent to the OS
    /// - `Err(_)` - Failed to inject the event
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The virtual input device is not available
    /// - The OS rejected the synthetic event
    /// - Permission to inject input was denied
    async fn send_output(&mut self, action: OutputAction) -> Result<(), KeyrxError>;

    /// Start capturing input.
    ///
    /// Initializes the keyboard hook or opens the input device. This must be
    /// called before [`poll_events`](Self::poll_events) or [`send_output`](Self::send_output).
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Capture started successfully
    /// - `Err(_)` - Failed to start capture
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The input device is busy or locked by another process
    /// - Insufficient permissions (e.g., not running as root on Linux)
    /// - The required kernel modules are not loaded (Linux)
    /// - Another keyboard hook is already installed (Windows)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut input = LinuxInput::new()?;
    /// input.start().await?;
    /// // Now ready to poll events and send output
    /// ```
    async fn start(&mut self) -> Result<(), KeyrxError>;

    /// Stop capturing input.
    ///
    /// Releases the keyboard hook or closes the input device. After calling
    /// this method, [`poll_events`](Self::poll_events) and [`send_output`](Self::send_output)
    /// should not be called until [`start`](Self::start) is called again.
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Capture stopped and resources released
    /// - `Err(_)` - Error during cleanup (resources may still be held)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to uninstall the keyboard hook
    /// - Failed to close the device file
    ///
    /// Note: Even if `stop` returns an error, the implementation should make
    /// a best effort to release resources. Subsequent calls to `start` should
    /// attempt to reinitialize cleanly.
    async fn stop(&mut self) -> Result<(), KeyrxError>;
}
