//! An in-process production daemon wired to one virtual keyboard, for live
//! end-to-end tests (activation, SYN_DROPPED recovery, ...).
//!
//! The daemon is scoped to a uniquely named virtual input keyboard and a
//! uniquely named virtual output device, so it never grabs the user's
//! keyboards or types into their desktop. Keys are injected through uinput
//! and read back from the daemon's output device.
#![cfg(target_os = "linux")]
#![allow(dead_code)]

use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEventType;
use keyrx_daemon::daemon::{ConfigSource, Daemon, DaemonError, DaemonSharedState};
use keyrx_daemon::platform::linux::LinuxPlatform;
use keyrx_daemon::test_utils::{OutputCapture, VirtualKeyboard};

/// A running in-process daemon wired to one virtual keyboard.
pub struct Harness {
    pub keyboard: VirtualKeyboard,
    pub capture: OutputCapture,
    pub shared: Arc<DaemonSharedState>,
    pub running: Arc<std::sync::atomic::AtomicBool>,
    pub output_name: String,
    pub thread: Option<JoinHandle<Result<(), DaemonError>>>,
}

impl Harness {
    pub fn start(tag: &str, source: ConfigSource, config_dir: &Path) -> Self {
        Self::start_expecting(tag, source, config_dir, 1)
    }

    /// Like [`Self::start`], but asserts the daemon grabbed `grabbed` devices
    /// at startup (0 when no valid config is live).
    pub fn start_expecting(
        tag: &str,
        source: ConfigSource,
        config_dir: &Path,
        grabbed: usize,
    ) -> Self {
        let keyboard = VirtualKeyboard::create(&format!("keyrx-live-{tag}")).expect("keyboard");
        std::thread::sleep(Duration::from_millis(200)); // let udev register it
        let output_name = format!("keyrx-live-out-{tag}-{}", std::process::id());
        let platform = Box::new(LinuxPlatform::scoped(keyboard.name(), &output_name));

        let mut daemon =
            Daemon::new(platform, source, config_dir.to_path_buf()).expect("daemon starts");
        assert_eq!(
            daemon.device_count(),
            grabbed,
            "daemon must grab exactly the expected test keyboards"
        );
        let shared = daemon.shared_state();
        let running = daemon.running_flag();
        let thread = Some(std::thread::spawn(move || daemon.run()));

        let capture =
            OutputCapture::find_by_name(&output_name, Duration::from_secs(5)).expect("output");
        Self {
            keyboard,
            capture,
            output_name,
            shared,
            running,
            thread,
        }
    }

    /// Taps `key` and returns what the daemon emitted, as (key, is_press).
    pub fn tap(&mut self, key: KeyCode) -> Vec<(KeyCode, bool)> {
        self.capture.drain().expect("drain");
        let events = VirtualKeyboard::tap_events(key);
        self.keyboard
            .inject_sequence(&events, Some(Duration::from_millis(10)))
            .expect("inject");
        self.capture
            .collect_events(Duration::from_millis(300))
            .expect("capture")
            .iter()
            .map(|e| (e.keycode(), e.event_type() == KeyEventType::Press))
            .collect()
    }

    /// Replaces the output reader with a fresh one. A reader that fell behind
    /// during a flood has its own queue of compensation events; a new one
    /// starts from a clean slate so later assertions see only new output.
    pub fn reopen_capture(&mut self) {
        self.capture =
            OutputCapture::find_by_name(&self.output_name, Duration::from_secs(5)).expect("output");
    }

    pub fn wait_for_profile(&self, name: &str) {
        let deadline = Instant::now() + Duration::from_secs(3);
        while self.shared.get_active_profile().as_deref() != Some(name) {
            assert!(
                Instant::now() < deadline,
                "daemon never loaded '{name}' (status: {:?})",
                self.shared.get_active_profile()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let result = thread.join().expect("daemon thread panicked");
            if !std::thread::panicking() {
                result.expect("daemon run failed");
            }
        }
    }
}

pub fn tapped(key: KeyCode) -> Vec<(KeyCode, bool)> {
    vec![(key, true), (key, false)]
}
