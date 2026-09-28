//! End-to-end: activating a profile changes the live key remapping.
//!
//! A real daemon (`Daemon::run`, the Linux production event loop) is scoped to
//! a virtual input keyboard and a uniquely named virtual output device, so the
//! test never grabs the user's keyboards or types into their desktop. Keys are
//! injected through uinput and read back from the daemon's output device.
//!
//! Requires access to /dev/uinput and /dev/input (the `input` group); skipped
//! otherwise.

#![cfg(target_os = "linux")]

use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEventType;
use keyrx_daemon::config::{ProfileManager, ProfileTemplate};
use keyrx_daemon::daemon::{ConfigSource, Daemon, DaemonError, DaemonSharedState};
use keyrx_daemon::platform::linux::LinuxPlatform;
use keyrx_daemon::services::ProfileService;
use keyrx_daemon::test_utils::{OutputCapture, VirtualKeyboard};
use tempfile::TempDir;

const CAPS_TO_LCTRL: &str = r#"
device_start("*");
  map("VK_CapsLock", "VK_LCtrl");
device_end();
"#;

/// A running in-process daemon wired to one virtual keyboard.
struct Harness {
    keyboard: VirtualKeyboard,
    capture: OutputCapture,
    shared: Arc<DaemonSharedState>,
    running: Arc<std::sync::atomic::AtomicBool>,
    thread: Option<JoinHandle<Result<(), DaemonError>>>,
}

impl Harness {
    fn start(tag: &str, source: ConfigSource, config_dir: &Path) -> Self {
        let keyboard = VirtualKeyboard::create(&format!("keyrx-live-{tag}")).expect("keyboard");
        std::thread::sleep(Duration::from_millis(200)); // let udev register it
        let output_name = format!("keyrx-live-out-{tag}-{}", std::process::id());
        let platform = Box::new(LinuxPlatform::scoped(keyboard.name(), &output_name));

        let mut daemon =
            Daemon::new(platform, source, config_dir.to_path_buf()).expect("daemon starts");
        assert_eq!(
            daemon.device_count(),
            1,
            "daemon must grab only the test keyboard"
        );
        let shared = daemon.shared_state();
        let running = daemon.running_flag();
        let thread = Some(std::thread::spawn(move || daemon.run()));

        let capture =
            OutputCapture::find_by_name(&output_name, Duration::from_secs(5)).expect("output");
        Self {
            keyboard,
            capture,
            shared,
            running,
            thread,
        }
    }

    /// Taps `key` and returns what the daemon emitted, as (key, is_press).
    fn tap(&mut self, key: KeyCode) -> Vec<(KeyCode, bool)> {
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

    fn wait_for_profile(&self, name: &str) {
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

fn tapped(key: KeyCode) -> Vec<(KeyCode, bool)> {
    vec![(key, true), (key, false)]
}

/// Profiles "a" (CapsLock→Escape, active) and "b" (CapsLock→LCtrl), compiled
/// by the real ProfileManager.
fn profiles(dir: &Path) -> Arc<ProfileManager> {
    let manager = Arc::new(ProfileManager::new(dir.to_path_buf()).expect("profile manager"));
    manager
        .create("a", ProfileTemplate::CapslockEscape)
        .expect("create a");
    manager
        .create("b", ProfileTemplate::Blank)
        .expect("create b");
    manager.set_config("b", CAPS_TO_LCTRL).expect("config b");
    manager
}

/// Activates `name` the way REST, MCP and WS-RPC do.
fn activate(service: &ProfileService, name: &str) {
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let result = runtime
        .block_on(service.activate_profile(name))
        .expect("activation");
    assert!(result.success, "activation of '{name}' failed: {result:?}");
}

#[test]
fn activating_a_profile_changes_live_remapping_without_restart() {
    keyrx_daemon::skip_if_no_uinput!();
    let dir = TempDir::new().unwrap();
    let manager = profiles(dir.path());
    assert!(manager.activate("a").expect("activate a").success);

    let mut h = Harness::start("switch", ConfigSource::ActiveProfile, dir.path());
    assert_eq!(h.shared.get_active_profile().as_deref(), Some("a"));
    assert_eq!(h.tap(KeyCode::CapsLock), tapped(KeyCode::Escape));

    let service = ProfileService::new(Arc::clone(&manager));
    service.attach_daemon_state(Arc::clone(&h.shared));
    activate(&service, "b");
    h.wait_for_profile("b");
    assert_eq!(h.tap(KeyCode::CapsLock), tapped(KeyCode::LCtrl));

    // Unmapped keys still pass through after the switch.
    assert_eq!(h.tap(KeyCode::F24), tapped(KeyCode::F24));
}

#[test]
fn explicit_config_applies_without_active_profile_and_activation_overrides_it() {
    keyrx_daemon::skip_if_no_uinput!();
    let dir = TempDir::new().unwrap();
    let manager = profiles(dir.path());
    // `run --config FILE` with no .active: the file must be live. (Activating
    // "a" is just the easy way to compile it; the activation is then undone.)
    assert!(manager.activate("a").expect("compile a").success);
    std::fs::remove_file(dir.path().join(".active")).unwrap();
    let file: PathBuf = dir.path().join("config.krx");
    std::fs::copy(dir.path().join("profiles/a.krx"), &file).unwrap();

    let mut h = Harness::start("explicit", ConfigSource::File(file.clone()), dir.path());
    assert_eq!(h.shared.get_active_profile(), None);
    assert_eq!(h.shared.get_config_path(), file);
    assert_eq!(h.tap(KeyCode::CapsLock), tapped(KeyCode::Escape));

    let service = ProfileService::new(manager);
    service.attach_daemon_state(Arc::clone(&h.shared));
    activate(&service, "b");
    h.wait_for_profile("b");
    assert_eq!(h.tap(KeyCode::CapsLock), tapped(KeyCode::LCtrl));
}

/// G7: switching profiles while a remapped key is held must not leave the old
/// output stuck. Hold CapsLock (→ Escape down), switch to "b", and Escape must
/// come up at the swap, before CapsLock is released.
#[test]
fn profile_switch_releases_outputs_held_under_the_old_mapping() {
    keyrx_daemon::skip_if_no_uinput!();
    let dir = TempDir::new().unwrap();
    let manager = profiles(dir.path());
    assert!(manager.activate("a").expect("activate a").success);
    let mut h = Harness::start("held", ConfigSource::ActiveProfile, dir.path());

    h.capture.drain().expect("drain");
    h.keyboard
        .inject(keyrx_core::runtime::KeyEvent::press(KeyCode::CapsLock))
        .expect("press");
    let down = h
        .capture
        .collect_events(Duration::from_millis(300))
        .expect("capture");
    assert_eq!(down.len(), 1);
    assert_eq!(down[0].keycode(), KeyCode::Escape);

    let service = ProfileService::new(manager);
    service.attach_daemon_state(Arc::clone(&h.shared));
    activate(&service, "b");
    h.wait_for_profile("b");
    // The daemon releases held outputs right after publishing the switch.
    let all: Vec<(KeyCode, bool)> = h
        .capture
        .collect_events(Duration::from_millis(300))
        .expect("capture")
        .iter()
        .map(|e| (e.keycode(), e.event_type() == KeyEventType::Press))
        .collect();
    assert!(
        all.contains(&(KeyCode::Escape, false)),
        "Escape was left held after the switch: {all:?}"
    );
    h.keyboard
        .inject(keyrx_core::runtime::KeyEvent::release(KeyCode::CapsLock))
        .expect("release");
}
