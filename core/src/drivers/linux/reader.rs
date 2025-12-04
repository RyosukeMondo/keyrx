//! Evdev reader for keyboard event capture.
//!
//! This module provides `EvdevReader` for reading keyboard events from
//! evdev devices on Linux.

use super::keymap::evdev_to_keycode;
use super::safety::device::SafeDevice;
use crate::config::{
    EVDEV_KEY_ESC, EVDEV_KEY_LEFTALT, EVDEV_KEY_LEFTCTRL, EVDEV_KEY_LEFTSHIFT, EVDEV_KEY_RIGHTALT,
    EVDEV_KEY_RIGHTCTRL, EVDEV_KEY_RIGHTSHIFT,
};
use crate::drivers::emergency_exit::{is_bypass_active, toggle_bypass_mode};
use crate::engine::InputEvent;
use crate::errors::KeyrxError;
use crate::safety::panic_guard::PanicGuard;
use crossbeam_channel::Sender;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tracing::{debug, error, trace, warn};

/// Reader for keyboard events from an evdev device.
///
/// `EvdevReader` provides exclusive access to a keyboard device through the evdev
/// subsystem. It uses SafeDevice wrapper which automatically handles device
/// grabbing/ungrabbing with RAII guarantees.
///
/// # Thread Safety
///
/// The `running` flag is shared across threads using `Arc<AtomicBool>` to allow
/// clean shutdown from the main thread.
///
/// # Panic Recovery
///
/// The reader thread is wrapped in `PanicGuard` to handle panics gracefully.
/// If a panic occurs, the `panic_error` flag is set, backtrace is captured,
/// and the keyboard device is automatically ungrabbed via SafeDevice's Drop implementation.
pub struct EvdevReader {
    /// Safe wrapper around the evdev device with RAII cleanup.
    device: SafeDevice,
    /// Channel sender for forwarding events to the async engine.
    tx: Sender<InputEvent>,
    /// Shared flag to signal when the reader should stop.
    running: Arc<AtomicBool>,
    /// Path to the device (for error messages and logging).
    device_path: PathBuf,
    /// Shared flag to indicate if the reader thread panicked.
    panic_error: Arc<AtomicBool>,
    /// Device ID string for event metadata (derived from device_path).
    device_id: String,
    /// Modifier state tracking for emergency exit detection.
    modifier_state: ModifierStateTracker,
}

/// Tracks modifier key state for emergency exit combo detection.
#[derive(Default)]
struct ModifierStateTracker {
    ctrl_down: bool,
    shift_down: bool,
    alt_down: bool,
}

impl ModifierStateTracker {
    /// Update modifier state based on key event.
    fn update(&mut self, code: u16, pressed: bool) {
        if code == EVDEV_KEY_LEFTCTRL || code == EVDEV_KEY_RIGHTCTRL {
            self.ctrl_down = pressed;
        } else if code == EVDEV_KEY_LEFTSHIFT || code == EVDEV_KEY_RIGHTSHIFT {
            self.shift_down = pressed;
        } else if code == EVDEV_KEY_LEFTALT || code == EVDEV_KEY_RIGHTALT {
            self.alt_down = pressed;
        }
    }

    /// Check if all modifiers for emergency exit are pressed.
    fn all_modifiers_down(&self) -> bool {
        self.ctrl_down && self.shift_down && self.alt_down
    }
}

impl EvdevReader {
    /// Create a new EvdevReader for the given device path.
    ///
    /// # Arguments
    ///
    /// * `device_path` - Path to the evdev device (e.g., `/dev/input/event0`)
    /// * `tx` - Channel sender for forwarding input events
    /// * `running` - Shared flag for controlling the read loop
    /// * `panic_error` - Shared flag set to true if the reader thread panics
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The device path does not exist
    /// - Permission is denied when opening the device
    /// - The device cannot be opened for other reasons
    pub fn new(
        device_path: PathBuf,
        tx: Sender<InputEvent>,
        running: Arc<AtomicBool>,
        panic_error: Arc<AtomicBool>,
    ) -> Result<Self, KeyrxError> {
        // Use SafeDevice wrapper which provides better error messages and RAII cleanup
        let device = SafeDevice::open(&device_path)?;

        debug!(
            service = "keyrx",
            event = "evdev_opened",
            component = "linux_reader",
            device = device.device().name().unwrap_or("Unknown"),
            path = %device_path.display(),
            "Opened evdev device with SafeDevice wrapper"
        );

        // Create device_id from path for event metadata
        let device_id = device_path.to_string_lossy().to_string();

        Ok(Self {
            device,
            tx,
            running,
            device_path,
            panic_error,
            device_id,
            modifier_state: ModifierStateTracker::default(),
        })
    }

    /// Grab exclusive access to the keyboard device.
    ///
    /// While grabbed, the keyboard events are only sent to KeyRx and not to
    /// other applications. This is essential for key remapping to work properly.
    ///
    /// # Errors
    ///
    /// Returns `DriverError::GrabFailed` if:
    /// - Another process has already grabbed the device
    /// - The user lacks sufficient permissions
    pub fn grab(&mut self) -> Result<(), KeyrxError> {
        // SafeDevice handles the actual grabbing with proper error messages
        self.device.grab()?;
        debug!(
            service = "keyrx",
            event = "evdev_grabbed",
            component = "linux_reader",
            path = %self.device_path.display(),
            "Grabbed keyboard device"
        );
        Ok(())
    }

    /// Release the keyboard grab.
    ///
    /// This restores normal keyboard operation, allowing other applications
    /// to receive keyboard events again. Called automatically during shutdown
    /// and via SafeDevice's Drop implementation.
    ///
    /// # Errors
    ///
    /// Returns an error if the ungrab operation fails. This is rare and usually
    /// indicates a system-level issue.
    pub fn ungrab(&mut self) -> Result<(), KeyrxError> {
        // SafeDevice handles the actual ungrabbing with proper error messages
        self.device.ungrab()?;
        debug!(
            service = "keyrx",
            event = "evdev_released",
            component = "linux_reader",
            path = %self.device_path.display(),
            "Released keyboard device"
        );
        Ok(())
    }

    /// Check if the reader should continue running.
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Get a reference to the underlying SafeDevice wrapper.
    #[allow(dead_code)]
    pub fn safe_device(&self) -> &SafeDevice {
        &self.device
    }

    /// Get a mutable reference to the underlying SafeDevice wrapper.
    #[allow(dead_code)]
    pub fn safe_device_mut(&mut self) -> &mut SafeDevice {
        &mut self.device
    }

    /// Get the channel sender for forwarding events.
    #[allow(dead_code)]
    pub fn sender(&self) -> &Sender<InputEvent> {
        &self.tx
    }

    /// Get the device path.
    #[allow(dead_code)]
    pub fn device_path(&self) -> &Path {
        &self.device_path
    }

    /// Spawn a blocking read thread that captures keyboard events.
    ///
    /// This method consumes the `EvdevReader` and moves it into a dedicated thread
    /// that continuously reads events from the evdev device. Events are converted
    /// to `InputEvent` and sent through the channel.
    ///
    /// # Returns
    ///
    /// Returns a `JoinHandle` that can be used to wait for the thread to complete.
    /// The thread will exit when:
    /// - The `running` flag is set to `false`
    /// - A critical error occurs (e.g., device disconnected)
    /// - The channel receiver is dropped
    /// - A panic occurs (the keyboard will be ungrabbed before the thread exits)
    ///
    /// # Event Processing
    ///
    /// Only `EV_KEY` events are processed. Event values:
    /// - 0: Key released
    /// - 1: Key pressed
    /// - 2: Key repeat (ignored, we synthesize repeats differently)
    ///
    /// # Panic Recovery
    ///
    /// The thread code is wrapped in `PanicGuard` to handle panics gracefully.
    /// If a panic occurs:
    /// - The `panic_error` flag is set to `true`
    /// - The keyboard device is ungrabbed
    /// - The error is logged with backtrace
    /// - The thread exits cleanly
    pub fn spawn(mut self) -> JoinHandle<()> {
        let device_path = self.device_path.clone();
        let panic_error = self.panic_error.clone();
        let running = self.running.clone();

        thread::spawn(move || {
            debug!(
                service = "keyrx",
                event = "evdev_reader_started",
                component = "linux_reader",
                path = %device_path.display(),
                "EvdevReader thread started"
            );

            // Wrap the main loop in PanicGuard for panic recovery with backtrace logging
            let result = PanicGuard::new("linux_evdev_reader").execute(|| {
                self.run_loop();
            });

            // Handle panic recovery
            if let Err(panic_error_info) = result {
                self.handle_panic(&device_path, panic_error, running, panic_error_info);
                return;
            }

            // Normal shutdown: ungrab the device
            self.handle_normal_shutdown(&device_path);
        })
    }

    /// Main event reading loop.
    ///
    /// Continuously reads events from the evdev device and processes them.
    /// Exits when `running` is set to false or the channel is closed.
    fn run_loop(&mut self) {
        while self.running.load(Ordering::Relaxed) {
            match self.fetch_key_events() {
                Ok(events) => {
                    if !self.process_events(&events) {
                        return;
                    }
                }
                Err(Some(e)) => {
                    if !self.handle_read_error(&e) {
                        return;
                    }
                }
                Err(None) => {
                    // Shutdown requested during fetch
                    return;
                }
            }
        }
    }

    /// Fetch key events from the device, filtering to only EV_KEY events.
    ///
    /// Returns `Ok(events)` on success, `Err(Some(e))` on read error,
    /// or `Err(None)` if shutdown was requested.
    fn fetch_key_events(&mut self) -> Result<Vec<evdev::InputEvent>, Option<std::io::Error>> {
        // Use SafeDevice's fetch_events method which handles disconnection gracefully
        match self.device.fetch_events() {
            Ok(events) => {
                let key_events: Vec<_> = events
                    .filter(|e| e.event_type() == evdev::EventType::KEY)
                    .collect();
                Ok(key_events)
            }
            Err(e) => {
                if !self.running.load(Ordering::Relaxed) {
                    Err(None)
                } else {
                    // Convert DriverError to io::Error for compatibility
                    // SafeDevice already provides good error messages
                    Err(Some(std::io::Error::other(e.to_string())))
                }
            }
        }
    }

    /// Process a batch of evdev key events.
    ///
    /// Returns `false` if the reader should stop (channel closed or emergency exit triggered),
    /// `true` otherwise.
    fn process_events(&mut self, events: &[evdev::InputEvent]) -> bool {
        // EMERGENCY EXIT CHECK - must be FIRST before any other processing
        // Check for Ctrl+Alt+Shift+Escape to toggle bypass mode
        for event in events {
            let code = event.code();
            let value = event.value();
            let pressed = value == 1; // 1 = press, 0 = release, 2 = repeat

            // Update modifier state for all events (including releases)
            self.modifier_state.update(code, pressed);

            // Check for emergency exit combo: Escape pressed with all modifiers down
            if pressed && code == EVDEV_KEY_ESC && self.modifier_state.all_modifiers_down() {
                let new_state = toggle_bypass_mode();
                if new_state {
                    // Bypass mode activated - ungrab device so keys flow to OS
                    warn!(
                        service = "keyrx",
                        event = "emergency_exit_triggered",
                        component = "linux_reader",
                        path = %self.device_path.display(),
                        "Emergency exit triggered - ungrabbing device"
                    );
                    if let Err(e) = self.ungrab() {
                        error!(
                            service = "keyrx",
                            event = "emergency_exit_ungrab_failed",
                            component = "linux_reader",
                            error = %e,
                            "Failed to ungrab device on emergency exit"
                        );
                    }
                    // Signal to stop processing (reader will exit)
                    self.running.store(false, Ordering::Relaxed);
                    return false;
                }
            }
        }

        // If bypass mode is active, don't process events normally
        if is_bypass_active() {
            return true;
        }

        process_events_internal(&self.tx, |event| self.convert_event(event), events)
    }

    /// Convert an evdev event to an InputEvent.
    fn convert_event(&self, event: &evdev::InputEvent) -> InputEvent {
        build_input_event(&self.device_id, event)
    }

    /// Handle a read error from the evdev device.
    ///
    /// Returns `false` if the reader should stop, `true` to continue.
    fn handle_read_error(&self, e: &std::io::Error) -> bool {
        handle_read_error_internal(&self.running, &self.device_path, e)
    }

    /// Handle panic recovery - ungrab keyboard and set error flags.
    fn handle_panic(
        &mut self,
        device_path: &Path,
        panic_error: Arc<AtomicBool>,
        running: Arc<AtomicBool>,
        critical_error: crate::errors::critical::CriticalError,
    ) {
        // Set the panic error flag so main thread can detect it
        panic_error.store(true, Ordering::SeqCst);
        running.store(false, Ordering::Relaxed);

        // Log the panic (PanicGuard already logged with backtrace)
        error!(
            service = "keyrx",
            event = "evdev_reader_panic",
            component = "linux_reader",
            path = %device_path.display(),
            error = %critical_error,
            "EvdevReader thread panicked"
        );

        // CRITICAL: Ungrab the keyboard even on panic
        if let Err(e) = self.ungrab() {
            error!(
                service = "keyrx",
                event = "evdev_ungrab_failed",
                component = "linux_reader",
                path = %device_path.display(),
                error = %e,
                "Failed to ungrab device after panic"
            );
        } else {
            debug!(
                service = "keyrx",
                event = "evdev_ungrab_success",
                component = "linux_reader",
                path = %device_path.display(),
                "Successfully ungrabbed device after panic"
            );
        }

        debug!(
            service = "keyrx",
            event = "evdev_reader_exit_after_panic",
            component = "linux_reader",
            path = %device_path.display(),
            "EvdevReader thread exiting after panic"
        );
    }

    /// Handle normal shutdown - ungrab the device.
    fn handle_normal_shutdown(&mut self, device_path: &Path) {
        if let Err(e) = self.ungrab() {
            tracing::warn!(
                service = "keyrx",
                event = "evdev_ungrab_failed",
                component = "linux_reader",
                path = %device_path.display(),
                error = %e,
                "Failed to ungrab device on shutdown"
            );
        }
        debug!(
            service = "keyrx",
            event = "evdev_reader_stopped",
            component = "linux_reader",
            path = %device_path.display(),
            "EvdevReader thread stopped"
        );
    }
}

fn build_input_event(device_id: &str, event: &evdev::InputEvent) -> InputEvent {
    let value = event.value();
    let is_repeat = value == 2;
    let pressed = value == 1 || is_repeat;
    let key_code = evdev_to_keycode(event.code());
    let timestamp_us = event
        .timestamp()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0);
    let scan_code = event.code();

    InputEvent {
        key: key_code,
        pressed,
        timestamp_us,
        device_id: Some(device_id.to_string()),
        is_repeat,
        is_synthetic: false,
        scan_code,
    }
}

fn process_events_internal<F>(
    tx: &Sender<InputEvent>,
    convert: F,
    events: &[evdev::InputEvent],
) -> bool
where
    F: Fn(&evdev::InputEvent) -> InputEvent,
{
    for event in events {
        let input_event = convert(event);
        trace!(
            service = "keyrx",
            event = "evdev_input_read",
            component = "linux_reader",
            key = ?input_event.key,
            pressed = input_event.pressed,
            scan_code = input_event.scan_code,
            repeat = input_event.is_repeat,
            timestamp_us = input_event.timestamp_us,
            device_id = input_event.device_id.as_deref().unwrap_or("unknown-device"),
            "Read input event"
        );

        if tx.send(input_event).is_err() {
            debug!(
                service = "keyrx",
                event = "evdev_channel_closed",
                component = "linux_reader",
                "Event channel closed, stopping reader"
            );
            return false;
        }
    }
    true
}

fn handle_read_error_internal(
    running: &Arc<AtomicBool>,
    device_path: &Path,
    e: &std::io::Error,
) -> bool {
    if !running.load(Ordering::Relaxed) {
        return false;
    }

    error!(
        service = "keyrx",
        event = "evdev_read_error",
        component = "linux_reader",
        path = %device_path.display(),
        error = %e,
        "Error reading events from device"
    );

    // Small sleep to avoid busy loop on persistent errors
    thread::sleep(std::time::Duration::from_millis(10));
    true
}

#[cfg(test)]
mod tests;
