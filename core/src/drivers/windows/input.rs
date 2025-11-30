use super::hook::{HookManager, HOOK_THREAD_ID};
use super::injector::SendInputInjector;
use crate::{
    drivers::KeyInjector,
    engine::{InputEvent, KeyCode},
};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use std::{
    panic::{self, AssertUnwindSafe},
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread::{self, JoinHandle},
};
use tracing::{debug, error, trace, warn};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT};
pub struct WindowsInput<I: KeyInjector = SendInputInjector> {
    hook_thread: Option<JoinHandle<()>>,
    rx: Receiver<InputEvent>,
    tx: Sender<InputEvent>,
    running: Arc<AtomicBool>,
    injector: I,
    panic_error: Arc<AtomicBool>,
}
impl WindowsInput {
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
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
    pub fn receiver(&self) -> &Receiver<InputEvent> {
        &self.rx
    }
    pub fn injector(&self) -> &I {
        &self.injector
    }
    pub fn injector_mut(&mut self) -> &mut I {
        &mut self.injector
    }
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
