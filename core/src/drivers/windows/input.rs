#![allow(unsafe_code)]
use super::device::list_keyboards;
use super::hook_thread::spawn_hook_thread;
use super::injector::SendInputInjector;
use crate::drivers::common::cache::{KeymapCache, LruKeymapCache};
use crate::identity::windows::extract_serial_number;
use crate::identity::DeviceIdentity;
use crate::metrics::{MetricsCollector, Operation};
use crate::{
    drivers::KeyInjector,
    engine::{InputEvent, KeyCode},
};
use anyhow::{bail, Result};
use crossbeam_channel::{Receiver, Sender};
use std::{
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
    sync::Arc,
    thread::JoinHandle,
};
use tracing::{debug, error, trace, warn};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT};
pub struct WindowsInput<I: KeyInjector = SendInputInjector> {
    hook_thread: Option<JoinHandle<()>>,
    rx: Receiver<InputEvent>,
    tx: Sender<InputEvent>,
    pub(crate) running: Arc<AtomicBool>,
    injector: I,
    panic_error: Arc<AtomicBool>,
    thread_id_store: Arc<AtomicU32>,
    pub(crate) device_identity: Option<DeviceIdentity>,
    metrics: Arc<dyn MetricsCollector>,
    cache: Arc<LruKeymapCache>,
}
impl WindowsInput {
    pub fn new() -> Result<Self> {
        Self::new_with_metrics(crate::metrics::default_noop_collector())
    }

    pub fn new_with_metrics(metrics: Arc<dyn MetricsCollector>) -> Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let running = Arc::new(AtomicBool::new(false));
        let panic_error = Arc::new(AtomicBool::new(false));
        let thread_id_store = Arc::new(AtomicU32::new(0));
        let injector = SendInputInjector::new();

        // Initialize keymap cache with capacity for 256 entries
        // This should cover all standard keys with room for device-specific mappings
        let cache = LruKeymapCache::new(256)
            .ok_or_else(|| anyhow::anyhow!("Failed to create keymap cache"))?;
        let cache = Arc::new(cache);

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
            thread_id_store,
            device_identity: Self::detect_device_identity(),
            metrics,
            cache,
        })
    }
}
impl<I: KeyInjector> WindowsInput<I> {
    pub fn new_with_injector(injector: I) -> Result<Self> {
        Self::new_with_injector_and_metrics(injector, crate::metrics::default_noop_collector())
    }

    pub fn new_with_injector_and_metrics(
        injector: I,
        metrics: Arc<dyn MetricsCollector>,
    ) -> Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let running = Arc::new(AtomicBool::new(false));
        let panic_error = Arc::new(AtomicBool::new(false));
        let thread_id_store = Arc::new(AtomicU32::new(0));

        // Initialize keymap cache with capacity for 256 entries
        // This should cover all standard keys with room for device-specific mappings
        let cache = LruKeymapCache::new(256)
            .ok_or_else(|| anyhow::anyhow!("Failed to create keymap cache"))?;
        let cache = Arc::new(cache);

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
            thread_id_store,
            device_identity: Self::detect_device_identity(),
            metrics,
            cache,
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
    pub(crate) fn inject_key(&mut self, key: KeyCode, pressed: bool) -> Result<()> {
        Ok(self.injector.inject(key, pressed)?)
    }

    fn detect_device_identity() -> Option<DeviceIdentity> {
        let device = list_keyboards().ok()?.into_iter().next()?;
        let path = device.path.to_string_lossy().to_string();
        let serial = extract_serial_number(&path).ok()?;

        Some(DeviceIdentity::new(
            device.vendor_id,
            device.product_id,
            serial,
        ))
    }

    pub fn invalidate_cache(&self, device_id: &str) {
        self.cache.invalidate_device(device_id);
    }

    fn log_drop_start(&self) {
        debug!(
            service = "keyrx",
            event = "windows_drop_stopping",
            component = "windows_input",
            "WindowsInput::drop - stopping driver"
        );
    }

    fn post_quit_for_drop(&self) {
        let thread_id = self.thread_id_store.load(Ordering::SeqCst);
        if thread_id == 0 {
            return;
        }
        // SAFETY: PostThreadMessageW is safe to call with a valid thread ID.
        // The thread_id is either 0 (checked above) or a valid Windows thread ID
        // stored by the hook thread during startup. WM_QUIT with null parameters
        // is a standard message that causes the message loop to exit.
        let result = unsafe { PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
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

    fn join_hook_thread_for_drop(&mut self) {
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
    }

    pub(crate) fn drain_events(&mut self) {
        while self.rx.try_recv().is_ok() {}
    }

    fn log_drop_complete(&self) {
        debug!(
            service = "keyrx",
            event = "windows_drop_complete",
            component = "windows_input",
            "WindowsInput::drop - cleanup complete"
        );
    }

    pub(crate) fn fail_if_hook_panicked(&mut self) -> Result<()> {
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
        Ok(())
    }

    pub(crate) fn is_inactive(&self) -> bool {
        !self.running.load(Ordering::Relaxed)
    }

    pub(crate) fn log_poll_when_inactive(&self) {
        trace!(
            service = "keyrx",
            event = "windows_poll_events_inactive",
            component = "windows_input",
            "poll_events called while not running"
        );
    }

    pub(crate) fn next_event(&mut self) -> Result<Option<InputEvent>> {
        let start = std::time::Instant::now();
        let result = match self.rx.try_recv() {
            Ok(event) => {
                /*
                trace!(
                    service = "keyrx",
                    event = "windows_input_event_received",
                    component = "windows_input",
                    key = ?event.key,
                    pressed = event.pressed,
                    "Received input event"
                );
                */
                Ok(Some(event))
            }
            Err(crossbeam_channel::TryRecvError::Empty) => Ok(None),
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                self.handle_disconnected_channel()
            }
        };

        // Only record latency if we actually got an event
        if matches!(result, Ok(Some(_))) {
            let elapsed_micros = start.elapsed().as_micros() as u64;
            self.metrics
                .record_latency(Operation::DriverRead, elapsed_micros);
        }

        result
    }

    pub(crate) fn log_polled_events(&self, count: usize) {
        if count > 0 {
            /*
            debug!(
                service = "keyrx",
                event = "windows_poll_events",
                component = "windows_input",
                count = count,
                "Returning polled events"
            );
            */
        }
    }

    pub(crate) fn log_inactive_send(&self) {
        trace!(
            service = "keyrx",
            event = "windows_send_output_inactive",
            component = "windows_input",
            "send_output called while not running"
        );
    }

    pub(crate) fn inject_key_action(
        &mut self,
        key: KeyCode,
        pressed: bool,
        event: &'static str,
    ) -> Result<()> {
        debug!(
            service = "keyrx",
            event = event,
            component = "windows_input",
            key = ?key,
            pressed = pressed,
            "Sending key action"
        );

        let start = std::time::Instant::now();
        let result = self.inject_key(key, pressed);
        let elapsed_micros = start.elapsed().as_micros() as u64;
        self.metrics
            .record_latency(Operation::DriverWrite, elapsed_micros);

        result
    }

    pub(crate) fn tap_key_action(&mut self, key: KeyCode) -> Result<()> {
        debug!(
            service = "keyrx",
            event = "windows_key_tap",
            component = "windows_input",
            key = ?key,
            "Sending key tap"
        );

        let start = std::time::Instant::now();
        self.inject_key(key, true)?;
        self.inject_key(key, false)?;
        let elapsed_micros = start.elapsed().as_micros() as u64;
        self.metrics
            .record_latency(Operation::DriverWrite, elapsed_micros);

        Ok(())
    }

    pub(crate) fn log_block_action(&self) {
        trace!(
            service = "keyrx",
            event = "windows_block_action",
            component = "windows_input",
            "Blocking key (no action needed)"
        );
    }

    pub(crate) fn log_passthrough_action(&self) {
        /*
        trace!(
            service = "keyrx",
            event = "windows_passthrough_action",
            component = "windows_input",
            "PassThrough (no action needed)"
        );
        */
    }

    pub(crate) fn log_start_skipped(&self) {
        warn!(
            service = "keyrx",
            event = "windows_start_skipped",
            component = "windows_input",
            reason = "already_running",
            "WindowsInput already running"
        );
    }

    pub(crate) fn log_stop_skipped(&self) {
        debug!(
            service = "keyrx",
            event = "windows_stop_skipped",
            component = "windows_input",
            reason = "already_stopped",
            "WindowsInput already stopped"
        );
    }

    pub(crate) fn prepare_start(&mut self) {
        self.panic_error.store(false, Ordering::SeqCst);
        self.running.store(true, Ordering::Relaxed);
    }

    pub(crate) fn log_started(&self) {
        debug!(
            service = "keyrx",
            event = "windows_started",
            component = "windows_input",
            "WindowsInput started successfully"
        );
    }

    pub(crate) fn log_stopped(&self) {
        debug!(
            service = "keyrx",
            event = "windows_stopped",
            component = "windows_input",
            "WindowsInput stopped successfully"
        );
    }

    fn handle_disconnected_channel(&mut self) -> Result<Option<InputEvent>> {
        if self.panic_error.load(Ordering::SeqCst) {
            error!(
                service = "keyrx",
                event = "windows_channel_disconnected",
                component = "windows_input",
                reason = "hook_panic",
                "Event channel disconnected due to hook thread panic"
            );
            self.running.store(false, Ordering::Relaxed);
            bail!("Hook thread panicked - keyboard hook has been uninstalled for safety");
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

    pub(crate) fn log_starting(&self) {
        debug!(
            service = "keyrx",
            event = "windows_starting",
            component = "windows_input",
            "Starting WindowsInput"
        );
    }

    pub(crate) fn spawn_hook_thread(&mut self) {
        let running = self.running.clone();
        let panic_error = self.panic_error.clone();
        let thread_id_store = self.thread_id_store.clone();
        let tx = self.tx.clone();
        let cache = self.cache.clone();
        let identity = self.device_identity.clone();
        self.hook_thread = Some(spawn_hook_thread(
            running,
            panic_error,
            thread_id_store,
            tx,
            cache,
            identity,
        ));
    }

    pub(crate) fn wait_for_hook_start(&mut self) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_millis(50));
        if !self.running.load(Ordering::Relaxed) {
            if let Some(handle) = self.hook_thread.take() {
                let _ = handle.join();
            }
            bail!("Failed to start keyboard hook");
        }
        Ok(())
    }

    pub(crate) fn post_quit_for_stop(&self) {
        let thread_id = self.thread_id_store.load(Ordering::SeqCst);
        if thread_id == 0 {
            return;
        }
        // SAFETY: PostThreadMessageW is safe to call with a valid thread ID.
        // The thread_id is either 0 (checked above) or a valid Windows thread ID
        // stored by the hook thread during startup. WM_QUIT with null parameters
        // is a standard message that causes the message loop to exit.
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

    pub(crate) fn join_hook_thread_for_stop(&mut self) {
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
                }
            }
        }
    }
}

impl Default for WindowsInput {
    fn default() -> Self {
        // WindowsInput::new only fails if channel creation fails, which is extremely rare
        // and would indicate a critical system resource issue. In such cases, panicking
        // in Default is acceptable as the system cannot continue anyway.
        // We use expect with a clear message to aid debugging if this ever occurs.
        #[allow(clippy::expect_used)]
        Self::new().expect(
            "WindowsInput creation failed - critical system resource exhaustion (channel creation)",
        )
    }
}
impl<I: KeyInjector> Drop for WindowsInput<I> {
    fn drop(&mut self) {
        if !self.running.load(Ordering::Relaxed) {
            return;
        }
        self.log_drop_start();
        self.running.store(false, Ordering::Relaxed);
        self.post_quit_for_drop();
        self.join_hook_thread_for_drop();
        self.drain_events();

        // Invalidate cache entries for Windows device
        self.cache.invalidate_device("windows");
        debug!(
            service = "keyrx",
            event = "windows_cache_invalidated_drop",
            component = "windows_input",
            "Invalidated cache entries on drop"
        );

        self.log_drop_complete();
    }
}
