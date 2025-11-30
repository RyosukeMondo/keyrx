//! Windows input driver using low-level keyboard hooks and SendInput.
//!
//! This module implements keyboard capture and injection on Windows using:
//! - **WH_KEYBOARD_LL** (low-level keyboard hook) for capturing all keyboard input
//! - **SendInput API** for injecting remapped keys back into the system
//!
//! # Platform Requirements
//!
//! - Windows 7 or later (tested on Windows 10/11)
//! - The `windows` crate for Win32 API bindings
//! - No administrator privileges required (in most cases)
//!
//! # Permission Requirements
//!
//! Unlike Linux, Windows keyboard hooks generally work without special permissions:
//!
//! 1. **Normal user**: Low-level keyboard hooks can be installed by any user-mode
//!    application. No administrator privileges are required.
//!
//! 2. **Antivirus software**: Some security software may block keyboard hooks.
//!    If installation fails:
//!    - Add an exception for `keyrx.exe` in your antivirus settings
//!    - Check Windows Security / Virus & threat protection settings
//!
//! 3. **UAC prompts**: Hooks cannot capture input during UAC dialogs (by design).
//!    Keys pressed during UAC prompts will be missed.
//!
//! 4. **Elevated applications**: Hooks from a non-elevated process cannot capture
//!    input to elevated (admin) applications. If you need to remap keys in admin
//!    apps, run KeyRx as administrator.
//!
//! # Thread Model
//!
//! The driver uses a dedicated thread for the hook and message pump:
//!
//! ```text
//! ┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
//! │  Physical KB    │────▶│  Hook Thread     │────▶│  Engine         │
//! │  (WH_KEYBOARD_LL)     │  (message pump)  │     │  (async)        │
//! └─────────────────┘     └──────────────────┘     └─────────────────┘
//!                                                          │
//!                                                          ▼
//! ┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
//! │  Applications   │◀────│  SendInput       │◀────│  Remap Logic    │
//! │                 │     │                  │     │                 │
//! └─────────────────┘     └──────────────────┘     └─────────────────┘
//! ```
//!
//! - **Hook thread**: A dedicated `std::thread` that installs the hook and runs
//!   the Windows message pump. Required because hooks dispatch via the message queue.
//! - **Message pump**: Uses `PeekMessageW` in a loop with 1ms sleep to avoid busy-waiting.
//! - **Event channel**: Events flow via `crossbeam_channel` to the async engine.
//! - **Running flag**: `Arc<AtomicBool>` shared between threads for shutdown signaling.
//!
//! # Error Handling
//!
//! The driver provides detailed error messages:
//!
//! - [`WindowsDriverError::HookInstallFailed`]: SetWindowsHookExW failed
//!   (check antivirus, or another hook may be blocking)
//! - [`WindowsDriverError::SendInputFailed`]: Cannot inject keys
//!   (target app may be elevated, or input is blocked)
//! - [`WindowsDriverError::MessagePumpPanic`]: Hook thread crashed unexpectedly
//!
//! # Cleanup and Recovery
//!
//! The driver implements robust cleanup to prevent keyboard issues:
//!
//! 1. **Normal shutdown**: Calling `stop()` posts `WM_QUIT` to the hook thread,
//!    which uninstalls the hook and exits the message loop.
//!
//! 2. **Drop cleanup**: The `Drop` implementation ensures cleanup even on early
//!    returns or unexpected exits.
//!
//! 3. **Panic recovery**: The hook thread wraps its main loop in `catch_unwind`.
//!    On panic:
//!    - The `panic_error` flag is set to `true`
//!    - The keyboard hook is uninstalled via `UnhookWindowsHookEx`
//!    - The error is logged
//!    - `poll_events()` returns an error on the next call
//!
//! 4. **Ctrl+C handling**: Uses `SetConsoleCtrlHandler` to catch Ctrl+C and
//!    trigger graceful shutdown.
//!
//! # Hook Callback Performance
//!
//! The hook callback (`low_level_keyboard_proc`) must complete quickly:
//!
//! - Windows requires hooks to process within ~100ms
//! - Slow processing causes visible keyboard lag
//! - The callback only extracts event data and sends via channel (non-blocking)
//! - Heavy processing happens in the engine thread, not the callback
//!
//! # Extended Keys
//!
//! Some keys require the `KEYEVENTF_EXTENDEDKEY` flag for proper injection:
//!
//! - Arrow keys (Up, Down, Left, Right)
//! - Navigation cluster (Home, End, Insert, Delete, Page Up/Down)
//! - Right-side modifiers (Right Ctrl, Right Alt)
//! - Numpad Enter and Numpad Divide
//! - Windows keys, Print Screen, Pause, Num Lock
//!
//! # Example
//!
//! ```ignore
//! use keyrx::drivers::WindowsInput;
//!
//! // Create Windows input source
//! let mut input = WindowsInput::new()?;
//!
//! // Start capturing events (installs hook)
//! input.start().await?;
//!
//! // Poll for events (non-blocking)
//! let events = input.poll_events().await?;
//!
//! // Inject remapped keys
//! input.send_output(OutputAction::KeyDown(KeyCode::Escape)).await?;
//!
//! // Stop and uninstall hook
//! input.stop().await?;
//! ```
//!
//! [`WindowsDriverError::HookInstallFailed`]: crate::error::WindowsDriverError::HookInstallFailed
//! [`WindowsDriverError::SendInputFailed`]: crate::error::WindowsDriverError::SendInputFailed
//! [`WindowsDriverError::MessagePumpPanic`]: crate::error::WindowsDriverError::MessagePumpPanic

mod hook;
mod injector;
mod keymap;

use crate::drivers::{DeviceInfo, KeyInjector};
use crate::engine::{InputEvent, KeyCode, OutputAction};
use crate::traits::InputSource;
use anyhow::Result;
use async_trait::async_trait;
use crossbeam_channel::{Receiver, Sender};
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tracing::{debug, error, trace, warn};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT};

use hook::{HookManager, HOOK_THREAD_ID};
pub use injector::SendInputInjector;

/// Windows input source using low-level keyboard hook.
///
/// `WindowsInput` coordinates keyboard event capture via a low-level keyboard hook
/// and key injection via `SendInput`. It implements the `InputSource` trait for
/// integration with the KeyRx remapping engine.
///
/// # Architecture
///
/// ```text
/// ┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
/// │  Physical KB    │────▶│  Hook Thread     │────▶│  Engine         │
/// │  (WH_KEYBOARD_LL)     │  (message pump)  │     │  (async)        │
/// └─────────────────┘     └──────────────────┘     └─────────────────┘
///                                                          │
///                                                          ▼
/// ┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
/// │  Applications   │◀────│  SendInput       │◀────│  Remap Logic    │
/// │                 │     │                  │     │                 │
/// └─────────────────┘     └──────────────────┘     └─────────────────┘
/// ```
///
/// # Thread Model
///
/// - The hook and message pump run in a dedicated thread (required by Windows)
/// - Events are sent via a crossbeam channel to the async engine
/// - The `running` flag is shared via `Arc<AtomicBool>` for clean shutdown
/// - WM_QUIT is posted to the hook thread to stop the message pump
///
/// # Panic Recovery
///
/// The hook thread is wrapped in `catch_unwind` to handle panics gracefully.
/// If a panic occurs, the `panic_error` flag is set and the keyboard hook is
/// uninstalled. The `poll_events` method checks this flag and returns an error
/// if a panic was detected.
///
/// # Type Parameter
///
/// * `I` - The key injector implementation. Defaults to `SendInputInjector` for
///   production use. For testing, use `MockKeyInjector` via `new_with_injector()`.
pub struct WindowsInput<I: KeyInjector = SendInputInjector> {
    /// Handle to the hook thread (set after start() is called).
    hook_thread: Option<JoinHandle<()>>,
    /// Receiver for events from the hook callback.
    rx: Receiver<InputEvent>,
    /// Sender for events (held to pass to the hook thread).
    tx: Sender<InputEvent>,
    /// Shared flag to signal shutdown.
    running: Arc<AtomicBool>,
    /// Key injector for sending output.
    injector: I,
    /// Shared flag indicating if the hook thread panicked.
    panic_error: Arc<AtomicBool>,
}

impl WindowsInput {
    /// Create a new Windows input source with the default SendInput injector.
    ///
    /// This initializes the input source but does not start the hook.
    /// Call [`start()`](Self::start) to begin capturing keyboard events.
    ///
    /// # Returns
    ///
    /// Returns a new `WindowsInput` instance ready to be started.
    pub fn new() -> Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let running = Arc::new(AtomicBool::new(false));
        let panic_error = Arc::new(AtomicBool::new(false));
        let injector = SendInputInjector::new();

        debug!("WindowsInput created");

        Ok(Self {
            hook_thread: None,
            rx,
            tx,
            running,
            injector,
            panic_error,
        })
    }
}

impl<I: KeyInjector> WindowsInput<I> {
    /// Create a new Windows input source with a custom key injector.
    ///
    /// This constructor allows dependency injection of the key injector,
    /// enabling unit testing without hardware access.
    ///
    /// # Arguments
    ///
    /// * `injector` - The key injector implementation to use
    ///
    /// # Example
    ///
    /// ```ignore
    /// use keyrx_core::drivers::{WindowsInput, MockKeyInjector};
    ///
    /// let mock = MockKeyInjector::new();
    /// let input = WindowsInput::new_with_injector(mock)?;
    /// ```
    pub fn new_with_injector(injector: I) -> Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let running = Arc::new(AtomicBool::new(false));
        let panic_error = Arc::new(AtomicBool::new(false));

        debug!("WindowsInput created with custom injector");

        Ok(Self {
            hook_thread: None,
            rx,
            tx,
            running,
            injector,
            panic_error,
        })
    }

    /// Check if the driver is currently running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Get the event receiver.
    ///
    /// This can be used for direct access to the event channel, though
    /// typically events should be accessed via `poll_events()`.
    pub fn receiver(&self) -> &Receiver<InputEvent> {
        &self.rx
    }

    /// Get a reference to the key injector.
    ///
    /// This can be used for testing to inspect the injector state.
    pub fn injector(&self) -> &I {
        &self.injector
    }

    /// Get a mutable reference to the key injector.
    ///
    /// This can be used for testing to inspect or modify the injector state.
    pub fn injector_mut(&mut self) -> &mut I {
        &mut self.injector
    }

    /// Helper method to inject a key using the injector.
    fn inject_key(&mut self, key: KeyCode, pressed: bool) -> Result<()> {
        self.injector.inject(key, pressed)
    }
}

impl Default for WindowsInput {
    fn default() -> Self {
        Self::new().expect("WindowsInput::new should not fail")
    }
}

impl<I: KeyInjector> Drop for WindowsInput<I> {
    fn drop(&mut self) {
        // Ensure the driver is stopped and hook is released on drop.
        // This is critical for graceful cleanup even on panics or unexpected termination.
        if self.running.load(Ordering::Relaxed) {
            debug!("WindowsInput::drop - stopping driver...");
            self.running.store(false, Ordering::Relaxed);

            // Post WM_QUIT to the hook thread to break out of the message loop
            let thread_id = HOOK_THREAD_ID.load(Ordering::SeqCst);
            if thread_id != 0 {
                // SAFETY: PostThreadMessageW is safe to call with a valid thread ID
                let result =
                    unsafe { PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
                if result.is_err() {
                    warn!("WindowsInput::drop - Failed to post WM_QUIT to hook thread");
                }
            }

            // Wait for the hook thread to finish
            if let Some(handle) = self.hook_thread.take() {
                debug!("WindowsInput::drop - waiting for hook thread...");
                match handle.join() {
                    Ok(()) => debug!("WindowsInput::drop - hook thread finished cleanly"),
                    Err(e) => warn!("WindowsInput::drop - hook thread panicked: {:?}", e),
                }
            }

            // Drain any remaining events
            while self.rx.try_recv().is_ok() {}

            debug!("WindowsInput::drop - cleanup complete");
        }
    }
}

#[async_trait]
impl<I: KeyInjector + 'static> InputSource for WindowsInput<I> {
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>> {
        // Check if the hook thread panicked
        if self.panic_error.load(Ordering::SeqCst) {
            error!("poll_events called after hook thread panic");
            self.running.store(false, Ordering::Relaxed);
            anyhow::bail!("Hook thread panicked - keyboard hook has been uninstalled for safety");
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
                    // Channel closed - hook thread has stopped
                    // Check if it was due to a panic
                    if self.panic_error.load(Ordering::SeqCst) {
                        error!("Event channel disconnected due to hook thread panic");
                        self.running.store(false, Ordering::Relaxed);
                        anyhow::bail!(
                            "Hook thread panicked - keyboard hook has been uninstalled for safety"
                        );
                    }
                    error!("Event channel disconnected - hook thread may have crashed");
                    self.running.store(false, Ordering::Relaxed);
                    anyhow::bail!("Input hook disconnected unexpectedly");
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
                self.inject_key(key, true)?;
            }
            OutputAction::KeyUp(key) => {
                debug!("Sending key up: {:?}", key);
                self.inject_key(key, false)?;
            }
            OutputAction::KeyTap(key) => {
                debug!("Sending key tap: {:?}", key);
                self.inject_key(key, true)?;
                self.inject_key(key, false)?;
            }
            OutputAction::Block => {
                // Block does nothing - the original event is already captured
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
            warn!("WindowsInput already running");
            return Ok(());
        }

        debug!("Starting WindowsInput...");

        // Reset panic error flag for fresh start
        self.panic_error.store(false, Ordering::SeqCst);

        // Set running flag before spawning thread
        self.running.store(true, Ordering::Relaxed);

        // Clone what we need for the thread
        let running = self.running.clone();
        let panic_error = self.panic_error.clone();
        let tx = self.tx.clone();

        // Spawn the hook thread
        let handle = thread::spawn(move || {
            debug!("Hook thread started");

            // Create the hook manager
            let mut hook_manager = HookManager::new(running.clone());

            // Install the hook
            match hook_manager.install(tx) {
                Ok(()) => {
                    debug!("Keyboard hook installed successfully");
                }
                Err(e) => {
                    error!("Failed to install keyboard hook: {:?}", e);
                    running.store(false, Ordering::Relaxed);
                    return;
                }
            }

            // Wrap the message loop in catch_unwind for panic recovery
            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                // Run the message loop (blocks until WM_QUIT or running=false)
                hook_manager.run_message_loop();
            }));

            // Handle panic recovery
            if let Err(panic_info) = result {
                // Set the panic error flag so main thread can detect it
                panic_error.store(true, Ordering::SeqCst);
                running.store(false, Ordering::Relaxed);

                // Log the panic
                let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };
                error!("Hook thread panicked: {}", panic_msg);

                // CRITICAL: Uninstall the hook even on panic
                // This is the primary goal of panic recovery
                hook_manager.uninstall();
                debug!("Hook uninstalled after panic");

                debug!("Hook thread exiting after panic");
                return;
            }

            // Normal shutdown: uninstall the hook (also done in Drop, but explicit is clearer)
            hook_manager.uninstall();

            debug!("Hook thread finished");
        });

        self.hook_thread = Some(handle);

        // Give the hook thread a moment to start and install the hook
        // This is a simple approach; a more robust solution would use a channel
        // to signal that the hook was successfully installed
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Check if the thread is still running (hook installation succeeded)
        if !self.running.load(Ordering::Relaxed) {
            // Hook installation failed, wait for thread to finish
            if let Some(handle) = self.hook_thread.take() {
                let _ = handle.join();
            }
            anyhow::bail!("Failed to start keyboard hook");
        }

        debug!("WindowsInput started successfully");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            debug!("WindowsInput already stopped");
            return Ok(());
        }

        debug!("Stopping WindowsInput...");

        // Signal the hook thread to stop
        self.running.store(false, Ordering::Relaxed);

        // Post WM_QUIT to the hook thread to break out of the message loop
        let thread_id = HOOK_THREAD_ID.load(Ordering::SeqCst);
        if thread_id != 0 {
            // SAFETY: PostThreadMessageW is safe to call with a valid thread ID
            let result = unsafe { PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
            if result.is_err() {
                warn!("Failed to post WM_QUIT to hook thread");
            } else {
                debug!("Posted WM_QUIT to hook thread {}", thread_id);
            }
        }

        // Wait for the hook thread to finish
        if let Some(handle) = self.hook_thread.take() {
            debug!("Waiting for hook thread to finish...");
            match handle.join() {
                Ok(()) => {
                    debug!("Hook thread finished cleanly");
                }
                Err(e) => {
                    error!("Hook thread panicked: {:?}", e);
                    // Continue with cleanup even if thread panicked
                }
            }
        }

        // Drain any remaining events from the channel
        while self.rx.try_recv().is_ok() {
            // Discard remaining events
        }

        debug!("WindowsInput stopped successfully");
        Ok(())
    }
}

/// List all keyboard devices available on the system.
///
/// On Windows, this returns a single entry representing the system keyboard.
/// Windows uses a global low-level keyboard hook (WH_KEYBOARD_LL) which
/// intercepts all keyboard input regardless of which physical device generated it.
///
/// # Errors
///
/// This function currently always succeeds on Windows.
pub fn list_keyboards() -> Result<Vec<DeviceInfo>> {
    // Windows uses a global keyboard hook that captures all keyboard input.
    // We return a single "virtual" device representing the system keyboard.
    // Full HID device enumeration could be added later for device-specific handling.
    Ok(vec![DeviceInfo::new(
        PathBuf::from("\\\\?\\HID#System#Keyboard"),
        "System Keyboard (Global Hook)".to_string(),
        0, // Vendor ID not available via global hook
        0, // Product ID not available via global hook
        true,
    )])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::{InjectedKey, MockKeyInjector};

    #[test]
    fn windows_input_default() {
        let input = WindowsInput::default();
        assert!(!input.is_running());
    }

    #[test]
    fn windows_input_new() {
        let input = WindowsInput::new().unwrap();
        assert!(!input.is_running());
    }

    #[test]
    fn windows_input_has_receiver() {
        let input = WindowsInput::new().unwrap();
        // Verify we can access the receiver (channel is empty initially)
        assert!(input.receiver().try_recv().is_err());
    }

    #[test]
    fn list_keyboards_returns_system_keyboard() {
        let keyboards = list_keyboards().unwrap();
        assert_eq!(keyboards.len(), 1);
        assert!(keyboards[0].is_keyboard());
        assert!(keyboards[0].name().contains("System Keyboard"));
    }

    #[test]
    fn windows_input_with_mock_injector() {
        let mock = MockKeyInjector::new();
        let input = WindowsInput::new_with_injector(mock).unwrap();
        assert!(!input.is_running());
    }

    #[test]
    fn windows_input_mock_injector_captures_keys() {
        let mock = MockKeyInjector::new();
        let mut input = WindowsInput::new_with_injector(mock).unwrap();

        // Directly test the inject_key helper
        input.inject_key(KeyCode::A, true).unwrap();
        input.inject_key(KeyCode::A, false).unwrap();
        input.inject_key(KeyCode::Escape, true).unwrap();

        // Verify injections were captured
        let injected = input.injector().injected_keys();
        assert_eq!(injected.len(), 3);
        assert_eq!(injected[0], InjectedKey::press(KeyCode::A));
        assert_eq!(injected[1], InjectedKey::release(KeyCode::A));
        assert_eq!(injected[2], InjectedKey::press(KeyCode::Escape));
    }

    #[test]
    fn windows_input_mock_injector_sync() {
        let mock = MockKeyInjector::new();
        let mut input = WindowsInput::new_with_injector(mock).unwrap();

        // Sync is a no-op for mock, but verify it doesn't panic
        input.injector_mut().sync().unwrap();
        assert_eq!(input.injector().sync_count(), 1);
    }

    #[test]
    fn windows_input_mock_injector_was_pressed() {
        let mock = MockKeyInjector::new();
        let mut input = WindowsInput::new_with_injector(mock).unwrap();

        assert!(!input.injector().was_pressed(KeyCode::B));

        input.inject_key(KeyCode::B, true).unwrap();
        assert!(input.injector().was_pressed(KeyCode::B));
        assert!(!input.injector().was_pressed(KeyCode::C));
    }

    #[test]
    fn windows_input_mock_injector_was_tapped() {
        let mock = MockKeyInjector::new();
        let mut input = WindowsInput::new_with_injector(mock).unwrap();

        // Press and release = tap
        input.inject_key(KeyCode::Space, true).unwrap();
        assert!(!input.injector().was_tapped(KeyCode::Space)); // Not yet
        input.inject_key(KeyCode::Space, false).unwrap();
        assert!(input.injector().was_tapped(KeyCode::Space)); // Now it's a tap
    }

    #[test]
    fn windows_input_mock_injector_fail_next() {
        let mock = MockKeyInjector::new();
        let mut input = WindowsInput::new_with_injector(mock).unwrap();

        // Set up failure
        input.injector_mut().fail_next_injection();

        // This should fail
        let result = input.inject_key(KeyCode::A, true);
        assert!(result.is_err());

        // Next one should succeed
        let result = input.inject_key(KeyCode::A, true);
        assert!(result.is_ok());
    }

    #[test]
    fn windows_input_mock_injector_clear() {
        let mock = MockKeyInjector::new();
        let mut input = WindowsInput::new_with_injector(mock).unwrap();

        input.inject_key(KeyCode::A, true).unwrap();
        input.injector_mut().sync().unwrap();

        assert_eq!(input.injector().injected_keys().len(), 1);
        assert_eq!(input.injector().sync_count(), 1);

        input.injector_mut().clear();

        assert!(input.injector().injected_keys().is_empty());
        assert_eq!(input.injector().sync_count(), 0);
    }

    #[tokio::test]
    async fn windows_input_send_output_with_mock() {
        let mock = MockKeyInjector::new();
        let mut input = WindowsInput::new_with_injector(mock).unwrap();

        // Simulate running state for send_output to work
        input.running.store(true, Ordering::Relaxed);

        // Test KeyDown
        input
            .send_output(OutputAction::KeyDown(KeyCode::A))
            .await
            .unwrap();
        assert!(input.injector().was_pressed(KeyCode::A));

        // Test KeyUp
        input
            .send_output(OutputAction::KeyUp(KeyCode::B))
            .await
            .unwrap();
        assert!(input.injector().was_released(KeyCode::B));

        // Test KeyTap
        input
            .send_output(OutputAction::KeyTap(KeyCode::C))
            .await
            .unwrap();
        assert!(input.injector().was_tapped(KeyCode::C));

        // Verify total injections
        let injected = input.injector().injected_keys();
        assert_eq!(injected.len(), 4); // A down, B up, C down, C up
    }

    #[tokio::test]
    async fn windows_input_send_output_block_passthrough() {
        let mock = MockKeyInjector::new();
        let mut input = WindowsInput::new_with_injector(mock).unwrap();
        input.running.store(true, Ordering::Relaxed);

        // Block and PassThrough should not inject any keys
        input.send_output(OutputAction::Block).await.unwrap();
        input.send_output(OutputAction::PassThrough).await.unwrap();

        assert!(input.injector().injected_keys().is_empty());
    }

    #[tokio::test]
    async fn windows_input_send_output_not_running() {
        let mock = MockKeyInjector::new();
        let mut input = WindowsInput::new_with_injector(mock).unwrap();

        // Not running - send_output should be a no-op
        assert!(!input.is_running());
        input
            .send_output(OutputAction::KeyDown(KeyCode::A))
            .await
            .unwrap();

        // Nothing should be injected
        assert!(input.injector().injected_keys().is_empty());
    }
}
