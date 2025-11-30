//! Evdev reader for keyboard event capture.
//!
//! This module provides `EvdevReader` for reading keyboard events from
//! evdev devices on Linux.

use super::keymap::evdev_to_keycode;
use crate::drivers::common::extract_panic_message;
use crate::engine::InputEvent;
use crate::error::LinuxDriverError;
use anyhow::Result;
use crossbeam_channel::Sender;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tracing::{debug, error, trace};

/// Reader for keyboard events from an evdev device.
///
/// `EvdevReader` provides exclusive access to a keyboard device through the evdev
/// subsystem. It uses the EVIOCGRAB ioctl to grab the device, preventing other
/// applications from receiving keyboard events while KeyRx is active.
///
/// # Thread Safety
///
/// The `running` flag is shared across threads using `Arc<AtomicBool>` to allow
/// clean shutdown from the main thread.
///
/// # Panic Recovery
///
/// The reader thread is wrapped in `catch_unwind` to handle panics gracefully.
/// If a panic occurs, the `panic_error` flag is set and the keyboard device is
/// ungrabbed to prevent leaving the keyboard in a stuck state.
pub struct EvdevReader {
    /// The evdev device handle for reading keyboard events.
    device: evdev::Device,
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
    ) -> Result<Self> {
        let device = evdev::Device::open(&device_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                LinuxDriverError::device_not_found(&device_path)
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                LinuxDriverError::permission_denied(&device_path)
            } else {
                LinuxDriverError::GrabFailed(e)
            }
        })?;

        debug!(
            "Opened evdev device: {} at {}",
            device.name().unwrap_or("Unknown"),
            device_path.display()
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
        })
    }

    /// Grab exclusive access to the keyboard device.
    ///
    /// While grabbed, the keyboard events are only sent to KeyRx and not to
    /// other applications. This is essential for key remapping to work properly.
    ///
    /// # Errors
    ///
    /// Returns `LinuxDriverError::GrabFailed` if:
    /// - Another process has already grabbed the device
    /// - The user lacks sufficient permissions
    pub fn grab(&mut self) -> Result<()> {
        self.device
            .grab()
            .map_err(|e| LinuxDriverError::grab_failed(std::io::Error::other(e.to_string())))?;
        debug!("Grabbed keyboard device: {}", self.device_path.display());
        Ok(())
    }

    /// Release the keyboard grab.
    ///
    /// This restores normal keyboard operation, allowing other applications
    /// to receive keyboard events again. Called automatically during shutdown.
    ///
    /// # Errors
    ///
    /// Returns an error if the ungrab operation fails. This is rare and usually
    /// indicates a system-level issue.
    pub fn ungrab(&mut self) -> Result<()> {
        self.device
            .ungrab()
            .map_err(|e| LinuxDriverError::grab_failed(std::io::Error::other(e.to_string())))?;
        debug!("Released keyboard device: {}", self.device_path.display());
        Ok(())
    }

    /// Check if the reader should continue running.
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Get a reference to the underlying evdev device.
    #[allow(dead_code)]
    pub fn device(&self) -> &evdev::Device {
        &self.device
    }

    /// Get a mutable reference to the underlying evdev device.
    #[allow(dead_code)]
    pub fn device_mut(&mut self) -> &mut evdev::Device {
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
    /// The thread code is wrapped in `catch_unwind` to handle panics gracefully.
    /// If a panic occurs:
    /// - The `panic_error` flag is set to `true`
    /// - The keyboard device is ungrabbed
    /// - The error is logged
    /// - The thread exits cleanly
    pub fn spawn(mut self) -> JoinHandle<()> {
        let device_path = self.device_path.clone();
        let panic_error = self.panic_error.clone();
        let running = self.running.clone();

        thread::spawn(move || {
            debug!("EvdevReader thread started for {}", device_path.display());

            // Wrap the main loop in catch_unwind for panic recovery
            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                self.run_loop();
            }));

            // Handle panic recovery
            if let Err(panic_info) = result {
                self.handle_panic(&device_path, panic_error, running, &*panic_info);
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
                    Err(Some(e))
                }
            }
        }
    }

    /// Process a batch of evdev key events.
    ///
    /// Returns `false` if the reader should stop (channel closed), `true` otherwise.
    fn process_events(&self, events: &[evdev::InputEvent]) -> bool {
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
        panic_info: &(dyn std::any::Any + Send),
    ) {
        // Set the panic error flag so main thread can detect it
        panic_error.store(true, Ordering::SeqCst);
        running.store(false, Ordering::Relaxed);

        // Log the panic
        let panic_msg = extract_panic_message(panic_info);
        error!(
            "EvdevReader thread panicked for {}: {}",
            device_path.display(),
            panic_msg
        );

        // CRITICAL: Ungrab the keyboard even on panic
        if let Err(e) = self.ungrab() {
            error!(
                "Failed to ungrab device {} after panic: {}",
                device_path.display(),
                e
            );
        } else {
            debug!(
                "Successfully ungrabbed device {} after panic",
                device_path.display()
            );
        }

        debug!(
            "EvdevReader thread exiting after panic for {}",
            device_path.display()
        );
    }

    /// Handle normal shutdown - ungrab the device.
    fn handle_normal_shutdown(&mut self, device_path: &Path) {
        if let Err(e) = self.ungrab() {
            tracing::warn!(
                "Failed to ungrab device {} on shutdown: {}",
                device_path.display(),
                e
            );
        }
        debug!("EvdevReader thread stopped for {}", device_path.display());
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
            "Read event: {:?} {} (scan_code={}, repeat={}) at {} from {}",
            input_event.key,
            if input_event.pressed { "down" } else { "up" },
            input_event.scan_code,
            input_event.is_repeat,
            input_event.timestamp_us,
            input_event.device_id.as_deref().unwrap_or("unknown-device")
        );

        if tx.send(input_event).is_err() {
            debug!("Event channel closed, stopping reader");
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

    error!("Error reading events from {}: {}", device_path.display(), e);

    // Small sleep to avoid busy loop on persistent errors
    thread::sleep(std::time::Duration::from_millis(10));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::KeyCode;
    use crossbeam_channel::unbounded;
    use std::sync::atomic::AtomicBool;

    fn sample_event(code: u16, value: i32) -> evdev::InputEvent {
        evdev::InputEvent::new(evdev::EventType::KEY, code, value)
    }

    #[test]
    fn build_input_event_sets_flags_and_metadata() {
        let key_down = build_input_event("dev0", &sample_event(30, 1));
        assert_eq!(key_down.key, KeyCode::A);
        assert!(key_down.pressed);
        assert!(!key_down.is_repeat);
        assert_eq!(key_down.device_id.as_deref(), Some("dev0"));

        let key_repeat = build_input_event("dev0", &sample_event(30, 2));
        assert!(key_repeat.pressed);
        assert!(key_repeat.is_repeat);

        let key_up = build_input_event("dev0", &sample_event(30, 0));
        assert!(!key_up.pressed);
        assert!(!key_up.is_repeat);
    }

    #[test]
    fn process_events_internal_sends_all_events() {
        let (tx, rx) = unbounded();
        let events = vec![sample_event(1, 1), sample_event(1, 0)];

        let keep_running =
            process_events_internal(&tx, |event| build_input_event("dev1", event), &events);
        assert!(keep_running);

        let received: Vec<_> = rx.try_iter().collect();
        assert_eq!(received.len(), 2);
        assert_eq!(received[0].key, KeyCode::Escape);
        assert!(received[0].pressed);
        assert_eq!(received[1].key, KeyCode::Escape);
        assert!(!received[1].pressed);
    }

    #[test]
    fn process_events_internal_stops_on_disconnected_channel() {
        let (tx, rx) = unbounded();
        drop(rx);

        let events = vec![sample_event(1, 1)];
        let keep_running =
            process_events_internal(&tx, |event| build_input_event("dev1", event), &events);
        assert!(!keep_running);
    }

    #[test]
    fn handle_read_error_internal_respects_running_flag() {
        let running = Arc::new(AtomicBool::new(false));
        let path = PathBuf::from("/dev/input/event-test");
        let err = std::io::Error::new(std::io::ErrorKind::Other, "boom");

        // When not running, returns false without sleeping
        assert!(!handle_read_error_internal(&running, &path, &err));

        running.store(true, Ordering::Relaxed);
        assert!(handle_read_error_internal(&running, &path, &err));
    }
}
