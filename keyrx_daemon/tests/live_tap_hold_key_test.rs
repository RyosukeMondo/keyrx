//! End-to-end: `tap_hold("CapsLock", "VK_Escape", "VK_LCtrl", 200)` through the
//! real Linux event loop. Tap -> Escape; hold -> a REAL LCtrl is pressed by the
//! idle-tick timeout and released with the key; Ctrl+C works while held.
//!
//! The daemon is scoped to a uniquely named virtual keyboard and output device
//! (never the user's keyboards). Skipped without access to /dev/uinput.

#![cfg(target_os = "linux")]

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::{KeyEvent, KeyEventType};
use keyrx_daemon::config::{ProfileManager, ProfileTemplate};
use keyrx_daemon::daemon::{ConfigSource, Daemon, DaemonError};
use keyrx_daemon::platform::linux::LinuxPlatform;
use keyrx_daemon::test_utils::{OutputCapture, VirtualKeyboard};
use tempfile::TempDir;

const CONFIG: &str = r#"
device_start("keyrx-live-hrm*");
  tap_hold("CapsLock", "VK_Escape", "VK_LCtrl", 200);
device_end();
"#;

struct Harness {
    keyboard: VirtualKeyboard,
    capture: OutputCapture,
    running: Arc<std::sync::atomic::AtomicBool>,
    thread: Option<JoinHandle<Result<(), DaemonError>>>,
    _dir: TempDir,
}

impl Harness {
    fn start() -> Self {
        let dir = TempDir::new().unwrap();
        let manager = ProfileManager::new(dir.path().to_path_buf()).expect("profile manager");
        manager.create("hrm", ProfileTemplate::Blank).expect("create");
        manager.set_config("hrm", CONFIG).expect("config");
        assert!(manager.activate("hrm").expect("activate").success);

        let keyboard = VirtualKeyboard::create("keyrx-live-hrm-kbd").expect("keyboard");
        std::thread::sleep(Duration::from_millis(200)); // let udev register it
        let output_name = format!("keyrx-live-hrm-out-{}", std::process::id());
        let platform = Box::new(LinuxPlatform::scoped(keyboard.name(), &output_name));
        let mut daemon = Daemon::new(
            platform,
            ConfigSource::ActiveProfile,
            dir.path().to_path_buf(),
        )
        .expect("daemon starts");
        assert_eq!(daemon.device_count(), 1, "must grab only the test keyboard");
        let running = daemon.running_flag();
        let thread = Some(std::thread::spawn(move || daemon.run()));
        let capture =
            OutputCapture::find_by_name(&output_name, Duration::from_secs(5)).expect("output");
        Self {
            keyboard,
            capture,
            running,
            thread,
            _dir: dir,
        }
    }

    fn send(&mut self, key: KeyCode, press: bool) {
        let event = if press {
            KeyEvent::press(key)
        } else {
            KeyEvent::release(key)
        };
        self.keyboard.inject(event).expect("inject");
    }

    /// Everything the daemon emitted since the last call, as (key, is_press).
    fn output(&mut self) -> Vec<(KeyCode, bool)> {
        self.capture
            .collect_events(Duration::from_millis(300))
            .expect("capture")
            .iter()
            .map(|e| (e.keycode(), e.event_type() == KeyEventType::Press))
            .collect()
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

#[test]
fn tap_types_escape_and_hold_presses_a_real_ctrl() {
    keyrx_daemon::skip_if_no_uinput!();
    let mut h = Harness::start();

    // Tap: Escape.
    h.capture.drain().expect("drain");
    h.send(KeyCode::CapsLock, true);
    std::thread::sleep(Duration::from_millis(30));
    h.send(KeyCode::CapsLock, false);
    assert_eq!(
        h.output(),
        vec![(KeyCode::Escape, true), (KeyCode::Escape, false)]
    );

    // Hold past the threshold with no other key: LCtrl down by the idle tick,
    // up on release.
    h.send(KeyCode::CapsLock, true);
    std::thread::sleep(Duration::from_millis(450));
    h.send(KeyCode::CapsLock, false);
    assert_eq!(
        h.output(),
        vec![(KeyCode::LCtrl, true), (KeyCode::LCtrl, false)]
    );

    // Ctrl+C: hold Caps, type C, release Caps.
    h.send(KeyCode::CapsLock, true);
    std::thread::sleep(Duration::from_millis(300));
    h.send(KeyCode::C, true);
    h.send(KeyCode::C, false);
    h.send(KeyCode::CapsLock, false);
    assert_eq!(
        h.output(),
        vec![
            (KeyCode::LCtrl, true),
            (KeyCode::C, true),
            (KeyCode::C, false),
            (KeyCode::LCtrl, false),
        ]
    );
}
