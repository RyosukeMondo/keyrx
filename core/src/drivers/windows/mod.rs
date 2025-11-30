//! Windows input driver using low-level keyboard hooks and SendInput.
//!
//! Implements keyboard capture via WH_KEYBOARD_LL hook and injection via SendInput API.
//! Requires Windows 7+, no admin privileges needed (except for capturing elevated apps).
//! Uses a dedicated hook thread with Windows message pump for event handling.

mod device;
mod hook;
mod injector;
mod keymap;

pub use device::list_keyboards;

use crate::drivers::KeyInjector;
use crate::engine::{InputEvent, KeyCode, OutputAction};
use crate::traits::InputSource;
use anyhow::Result;
use async_trait::async_trait;
use crossbeam_channel::{Receiver, Sender};
use std::panic::{self, AssertUnwindSafe};
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
/// Captures keyboard events via WH_KEYBOARD_LL hook in dedicated thread,
/// sends events via channel, injects keys via SendInput. Supports panic
/// recovery and dependency injection for testing.
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
    pub fn new() -> Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let running = Arc::new(AtomicBool::new(false));
        let panic_error = Arc::new(AtomicBool::new(false));
        let injector = SendInputInjector::new();

        debug!(
            service = "keyrx",
            event = "windows_input_created",
            component = "windows_input",
            injector = "sendinput",
            "WindowsInput created"
        );

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
    /// Create a new Windows input source with a custom key injector (for testing).
    pub fn new_with_injector(injector: I) -> Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let running = Arc::new(AtomicBool::new(false));
        let panic_error = Arc::new(AtomicBool::new(false));

        debug!(
            service = "keyrx",
            event = "windows_input_created",
            component = "windows_input",
            injector = "custom",
            "WindowsInput created with custom injector"
        );

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
            debug!(
                service = "keyrx",
                event = "windows_drop_stopping",
                component = "windows_input",
                "WindowsInput::drop - stopping driver"
            );
            self.running.store(false, Ordering::Relaxed);

            // Post WM_QUIT to the hook thread to break out of the message loop
            let thread_id = HOOK_THREAD_ID.load(Ordering::SeqCst);
            if thread_id != 0 {
                // SAFETY: PostThreadMessageW is safe to call with a valid thread ID
                let result =
                    unsafe { PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
                if result.is_err() {
                    warn!(
                        service = "keyrx",
                        event = "windows_drop_post_quit_failed",
                        component = "windows_input",
                        thread_id = thread_id,
                        "WindowsInput::drop - Failed to post WM_QUIT to hook thread"
                    );
                }
            }

            // Wait for the hook thread to finish
            if let Some(handle) = self.hook_thread.take() {
                debug!(
                    service = "keyrx",
                    event = "windows_drop_join_hook",
                    component = "windows_input",
                    "WindowsInput::drop - waiting for hook thread"
                );
                match handle.join() {
                    Ok(()) => debug!(
                        service = "keyrx",
                        event = "windows_drop_hook_stopped",
                        component = "windows_input",
                        status = "clean",
                        "WindowsInput::drop - hook thread finished cleanly"
                    ),
                    Err(e) => warn!(
                        service = "keyrx",
                        event = "windows_drop_hook_panic",
                        component = "windows_input",
                        error = ?e,
                        "WindowsInput::drop - hook thread panicked"
                    ),
                }
            }

            // Drain any remaining events
            while self.rx.try_recv().is_ok() {}

            debug!(
                service = "keyrx",
                event = "windows_drop_complete",
                component = "windows_input",
                "WindowsInput::drop - cleanup complete"
            );
        }
    }
}

#[async_trait]
impl<I: KeyInjector + 'static> InputSource for WindowsInput<I> {
    async fn poll_events(&mut self) -> Result<Vec<InputEvent>> {
        // Check if the hook thread panicked
        if self.panic_error.load(Ordering::SeqCst) {
            error!(
                service = "keyrx",
                event = "windows_hook_panic_detected",
                component = "windows_input",
                "poll_events called after hook thread panic"
            );
            self.running.store(false, Ordering::Relaxed);
            anyhow::bail!("Hook thread panicked - keyboard hook has been uninstalled for safety");
        }

        if !self.running.load(Ordering::Relaxed) {
            trace!(
                service = "keyrx",
                event = "windows_poll_events_inactive",
                component = "windows_input",
                "poll_events called while not running"
            );
            return Ok(vec![]);
        }

        // Non-blocking receive from the channel
        // Collect all available events without blocking
        let mut events = Vec::new();
        loop {
            match self.rx.try_recv() {
                Ok(event) => {
                    trace!(
                        service = "keyrx",
                        event = "windows_input_event_received",
                        component = "windows_input",
                        key = ?event.key,
                        pressed = event.pressed,
                        "Received input event"
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
                        error!(
                            service = "keyrx",
                            event = "windows_channel_disconnected",
                            component = "windows_input",
                            reason = "hook_panic",
                            "Event channel disconnected due to hook thread panic"
                        );
                        self.running.store(false, Ordering::Relaxed);
                        anyhow::bail!(
                            "Hook thread panicked - keyboard hook has been uninstalled for safety"
                        );
                    }
                    error!(
                        service = "keyrx",
                        event = "windows_channel_disconnected",
                        component = "windows_input",
                        reason = "unexpected_disconnect",
                        "Event channel disconnected - hook thread may have crashed"
                    );
                    self.running.store(false, Ordering::Relaxed);
                    anyhow::bail!("Input hook disconnected unexpectedly");
                }
            }
        }

        if !events.is_empty() {
            debug!(
                service = "keyrx",
                event = "windows_poll_events",
                component = "windows_input",
                count = events.len(),
                "Returning polled events"
            );
        }

        Ok(events)
    }

    async fn send_output(&mut self, action: OutputAction) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            trace!(
                service = "keyrx",
                event = "windows_send_output_inactive",
                component = "windows_input",
                "send_output called while not running"
            );
            return Ok(());
        }

        match action {
            OutputAction::KeyDown(key) => {
                debug!(
                    service = "keyrx",
                    event = "windows_key_down",
                    component = "windows_input",
                    key = ?key,
                    "Sending key down"
                );
                self.inject_key(key, true)?;
            }
            OutputAction::KeyUp(key) => {
                debug!(
                    service = "keyrx",
                    event = "windows_key_up",
                    component = "windows_input",
                    key = ?key,
                    "Sending key up"
                );
                self.inject_key(key, false)?;
            }
            OutputAction::KeyTap(key) => {
                debug!(
                    service = "keyrx",
                    event = "windows_key_tap",
                    component = "windows_input",
                    key = ?key,
                    "Sending key tap"
                );
                self.inject_key(key, true)?;
                self.inject_key(key, false)?;
            }
            OutputAction::Block => {
                // Block does nothing - the original event is already captured
                // and won't be passed through unless we explicitly emit it
                trace!(
                    service = "keyrx",
                    event = "windows_block_action",
                    component = "windows_input",
                    "Blocking key (no action needed)"
                );
            }
            OutputAction::PassThrough => {
                // PassThrough is handled by the engine - it re-emits the original key
                // For the driver, this is a no-op since the engine will call
                // KeyDown/KeyUp for the original key if needed
                trace!(
                    service = "keyrx",
                    event = "windows_passthrough_action",
                    component = "windows_input",
                    "PassThrough (no action needed)"
                );
            }
        }

        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            warn!(
                service = "keyrx",
                event = "windows_start_skipped",
                component = "windows_input",
                reason = "already_running",
                "WindowsInput already running"
            );
            return Ok(());
        }

        debug!(
            service = "keyrx",
            event = "windows_starting",
            component = "windows_input",
            "Starting WindowsInput"
        );

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
            debug!(
                service = "keyrx",
                event = "windows_hook_thread_started",
                component = "windows_input",
                "Hook thread started"
            );

            // Create the hook manager
            let mut hook_manager = HookManager::new(running.clone());

            // Install the hook
            match hook_manager.install(tx) {
                Ok(()) => {
                    debug!(
                        service = "keyrx",
                        event = "windows_keyboard_hook_installed",
                        component = "windows_input",
                        "Keyboard hook installed successfully"
                    );
                }
                Err(e) => {
                    error!(
                        service = "keyrx",
                        event = "windows_keyboard_hook_install_failed",
                        component = "windows_input",
                        error = ?e,
                        "Failed to install keyboard hook"
                    );
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
                error!(
                    service = "keyrx",
                    event = "windows_hook_thread_panic",
                    component = "windows_input",
                    error = panic_msg,
                    "Hook thread panicked"
                );

                // CRITICAL: Uninstall the hook even on panic
                // This is the primary goal of panic recovery
                hook_manager.uninstall();
                debug!(
                    service = "keyrx",
                    event = "windows_hook_uninstalled_after_panic",
                    component = "windows_input",
                    "Hook uninstalled after panic"
                );

                debug!(
                    service = "keyrx",
                    event = "windows_hook_thread_exit_after_panic",
                    component = "windows_input",
                    "Hook thread exiting after panic"
                );
                return;
            }

            // Normal shutdown: uninstall the hook (also done in Drop, but explicit is clearer)
            hook_manager.uninstall();

            debug!(
                service = "keyrx",
                event = "windows_hook_thread_finished",
                component = "windows_input",
                "Hook thread finished"
            );
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

        debug!(
            service = "keyrx",
            event = "windows_started",
            component = "windows_input",
            "WindowsInput started successfully"
        );
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            debug!(
                service = "keyrx",
                event = "windows_stop_skipped",
                component = "windows_input",
                reason = "already_stopped",
                "WindowsInput already stopped"
            );
            return Ok(());
        }

        debug!(
            service = "keyrx",
            event = "windows_stopping",
            component = "windows_input",
            "Stopping WindowsInput"
        );

        // Signal the hook thread to stop
        self.running.store(false, Ordering::Relaxed);

        // Post WM_QUIT to the hook thread to break out of the message loop
        let thread_id = HOOK_THREAD_ID.load(Ordering::SeqCst);
        if thread_id != 0 {
            // SAFETY: PostThreadMessageW is safe to call with a valid thread ID
            let result = unsafe { PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
            if result.is_err() {
                warn!(
                    service = "keyrx",
                    event = "windows_post_quit_failed",
                    component = "windows_input",
                    thread_id = thread_id,
                    "Failed to post WM_QUIT to hook thread"
                );
            } else {
                debug!(
                    service = "keyrx",
                    event = "windows_post_quit_sent",
                    component = "windows_input",
                    thread_id = thread_id,
                    "Posted WM_QUIT to hook thread"
                );
            }
        }

        // Wait for the hook thread to finish
        if let Some(handle) = self.hook_thread.take() {
            debug!(
                service = "keyrx",
                event = "windows_join_hook",
                component = "windows_input",
                "Waiting for hook thread to finish"
            );
            match handle.join() {
                Ok(()) => {
                    debug!(
                        service = "keyrx",
                        event = "windows_hook_thread_stopped",
                        component = "windows_input",
                        status = "clean",
                        "Hook thread finished cleanly"
                    );
                }
                Err(e) => {
                    error!(
                        service = "keyrx",
                        event = "windows_hook_thread_panic",
                        component = "windows_input",
                        error = ?e,
                        "Hook thread panicked"
                    );
                    // Continue with cleanup even if thread panicked
                }
            }
        }

        // Drain any remaining events from the channel
        while self.rx.try_recv().is_ok() {
            // Discard remaining events
        }

        debug!(
            service = "keyrx",
            event = "windows_stopped",
            component = "windows_input",
            "WindowsInput stopped successfully"
        );
        Ok(())
    }
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
