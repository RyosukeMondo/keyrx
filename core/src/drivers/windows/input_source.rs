use super::input::WindowsInput;
use crate::{
    drivers::KeyInjector,
    engine::{InputEvent, KeyCode, OutputAction},
    traits::InputSource,
};
use anyhow::{bail, Result};
use async_trait::async_trait;
use std::sync::atomic::Ordering;
use tracing::{debug, error, trace, warn};

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
            bail!("Hook thread panicked - keyboard hook has been uninstalled for safety");
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
                        bail!(
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
                    bail!("Input hook disconnected unexpectedly");
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
            bail!("Failed to start keyboard hook");
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
        self.running.store(false, Ordering::Relaxed);
        let thread_id = HOOK_THREAD_ID.load(Ordering::SeqCst);
        if thread_id != 0 {
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
                    error!(service = "keyrx", event = "windows_hook_thread_panic", component = "windows_input", error = ?e, "Hook thread panicked");
                }
            }
        }
        while self.rx.try_recv().is_ok() {}
        debug!(
            service = "keyrx",
            event = "windows_stopped",
            component = "windows_input",
            "WindowsInput stopped successfully"
        );
        Ok(())
    }
}
