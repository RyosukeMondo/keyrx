//! Evdev reader for keyboard event capture.
//!
//! This module provides `EvdevReader` for reading keyboard events from
//! evdev devices on Linux.

use crate::drivers::common::extract_panic_message;
use crate::engine::{InputEvent, KeyCode};
use crate::error::LinuxDriverError;
use anyhow::Result;
use crossbeam_channel::Sender;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tracing::{debug, error, trace};

/// Convert evdev key code to KeyCode.
/// This uses the engine's KeyCode type directly.
/// TODO: Remove after Task 12 unifies KeyCode types.
fn evdev_to_keycode(code: u16) -> KeyCode {
    match code {
        1 => KeyCode::Escape,
        2 => KeyCode::Key1,
        3 => KeyCode::Key2,
        4 => KeyCode::Key3,
        5 => KeyCode::Key4,
        6 => KeyCode::Key5,
        7 => KeyCode::Key6,
        8 => KeyCode::Key7,
        9 => KeyCode::Key8,
        10 => KeyCode::Key9,
        11 => KeyCode::Key0,
        12 => KeyCode::Minus,
        13 => KeyCode::Equal,
        14 => KeyCode::Backspace,
        15 => KeyCode::Tab,
        16 => KeyCode::Q,
        17 => KeyCode::W,
        18 => KeyCode::E,
        19 => KeyCode::R,
        20 => KeyCode::T,
        21 => KeyCode::Y,
        22 => KeyCode::U,
        23 => KeyCode::I,
        24 => KeyCode::O,
        25 => KeyCode::P,
        26 => KeyCode::LeftBracket,
        27 => KeyCode::RightBracket,
        28 => KeyCode::Enter,
        29 => KeyCode::LeftCtrl,
        30 => KeyCode::A,
        31 => KeyCode::S,
        32 => KeyCode::D,
        33 => KeyCode::F,
        34 => KeyCode::G,
        35 => KeyCode::H,
        36 => KeyCode::J,
        37 => KeyCode::K,
        38 => KeyCode::L,
        39 => KeyCode::Semicolon,
        40 => KeyCode::Apostrophe,
        41 => KeyCode::Grave,
        42 => KeyCode::LeftShift,
        43 => KeyCode::Backslash,
        44 => KeyCode::Z,
        45 => KeyCode::X,
        46 => KeyCode::C,
        47 => KeyCode::V,
        48 => KeyCode::B,
        49 => KeyCode::N,
        50 => KeyCode::M,
        51 => KeyCode::Comma,
        52 => KeyCode::Period,
        53 => KeyCode::Slash,
        54 => KeyCode::RightShift,
        55 => KeyCode::NumpadMultiply,
        56 => KeyCode::LeftAlt,
        57 => KeyCode::Space,
        58 => KeyCode::CapsLock,
        59 => KeyCode::F1,
        60 => KeyCode::F2,
        61 => KeyCode::F3,
        62 => KeyCode::F4,
        63 => KeyCode::F5,
        64 => KeyCode::F6,
        65 => KeyCode::F7,
        66 => KeyCode::F8,
        67 => KeyCode::F9,
        68 => KeyCode::F10,
        69 => KeyCode::NumLock,
        70 => KeyCode::ScrollLock,
        71 => KeyCode::Numpad7,
        72 => KeyCode::Numpad8,
        73 => KeyCode::Numpad9,
        74 => KeyCode::NumpadSubtract,
        75 => KeyCode::Numpad4,
        76 => KeyCode::Numpad5,
        77 => KeyCode::Numpad6,
        78 => KeyCode::NumpadAdd,
        79 => KeyCode::Numpad1,
        80 => KeyCode::Numpad2,
        81 => KeyCode::Numpad3,
        82 => KeyCode::Numpad0,
        83 => KeyCode::NumpadDecimal,
        87 => KeyCode::F11,
        88 => KeyCode::F12,
        96 => KeyCode::NumpadEnter,
        97 => KeyCode::RightCtrl,
        98 => KeyCode::NumpadDivide,
        100 => KeyCode::RightAlt,
        102 => KeyCode::Home,
        103 => KeyCode::Up,
        104 => KeyCode::PageUp,
        105 => KeyCode::Left,
        106 => KeyCode::Right,
        107 => KeyCode::End,
        108 => KeyCode::Down,
        109 => KeyCode::PageDown,
        110 => KeyCode::Insert,
        111 => KeyCode::Delete,
        113 => KeyCode::VolumeMute,
        114 => KeyCode::VolumeDown,
        115 => KeyCode::VolumeUp,
        125 => KeyCode::LeftMeta,
        126 => KeyCode::RightMeta,
        163 => KeyCode::MediaNext,
        164 => KeyCode::MediaPlayPause,
        165 => KeyCode::MediaPrev,
        166 => KeyCode::MediaStop,
        _ => KeyCode::Unknown(code),
    }
}

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
    fn run_loop(&mut self) {
        while self.running.load(Ordering::Relaxed) {
            // fetch_events blocks until events are available
            match self.device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        // Only process EV_KEY events
                        if event.event_type() != evdev::EventType::KEY {
                            continue;
                        }

                        // value: 0 = release, 1 = press, 2 = repeat
                        let value = event.value();
                        let is_repeat = value == 2;
                        // pressed is true for both initial press (1) and repeat (2)
                        let pressed = value == 1 || is_repeat;
                        let key_code = evdev_to_keycode(event.code());

                        // Extract timestamp from event as microseconds
                        let timestamp_us = event
                            .timestamp()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_micros() as u64)
                            .unwrap_or(0);

                        // Extract scan_code from the raw evdev event code
                        let scan_code = event.code();

                        // Create event with full metadata
                        let input_event = InputEvent {
                            key: key_code,
                            pressed,
                            timestamp_us,
                            device_id: Some(self.device_id.clone()),
                            is_repeat,
                            is_synthetic: false,
                            scan_code,
                        };

                        trace!(
                            "Read event: {:?} {} (scan_code={}, repeat={}) at {} from {}",
                            key_code,
                            if pressed { "down" } else { "up" },
                            scan_code,
                            is_repeat,
                            timestamp_us,
                            self.device_id
                        );

                        // Send event to channel
                        if self.tx.send(input_event).is_err() {
                            // Channel closed, receiver dropped - exit thread
                            debug!("Event channel closed, stopping reader");
                            return;
                        }
                    }
                }
                Err(e) => {
                    // Check if we should continue
                    if !self.running.load(Ordering::Relaxed) {
                        return;
                    }

                    // Log error but continue - might be temporary
                    error!(
                        "Error reading events from {}: {}",
                        self.device_path.display(),
                        e
                    );

                    // Small sleep to avoid busy loop on persistent errors
                    thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
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
