//! End-to-end coverage for the Linux device-handling fixes:
//!
//! - Only devices a `device_start` pattern matches are grabbed (previously
//!   every keyboard was grabbed and unmatched ones were passed through).
//! - A keyboard plugged in after the daemon starts is picked up without a
//!   restart (hotplug via inotify on `/dev/input`), and one unplugged is
//!   dropped cleanly without breaking the event loop for the rest.
//! - The emergency escape chord (LCtrl+RCtrl+Escape) ungrabs every device
//!   and stops the event loop.
//! - H6: an `EV_KEY` code with no `KeyCode` (vendor/consumer keys) and a
//!   non-key event (e.g. `EV_REL`) on a grabbed node are forwarded to the
//!   output device unchanged instead of being silently dropped.
//! - H8: `DeviceService`'s `active` field reflects what the daemon actually
//!   has grabbed - not "every device the OS can see" - and stays correct
//!   after a hotplug rescan.
//!
//! Like `live_profile_switch_test.rs`, every daemon here is scoped
//! (`LinuxPlatform::scoped`) to virtual keyboards created by the test itself
//! (all sharing a unique per-test name prefix), so it can never touch the
//! user's real keyboard or another test's devices. Requires /dev/uinput and
//! /dev/input access (the `input` group); skipped otherwise.

#![cfg(target_os = "linux")]

use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::event::KeyEvent;
use keyrx_core::runtime::KeyEventType;
use keyrx_daemon::config::{ProfileManager, ProfileTemplate};
use keyrx_daemon::daemon::{ConfigSource, Daemon, DaemonError, DaemonSharedState};
use keyrx_daemon::platform::linux::{evdev_to_keycode, LinuxPlatform};
use keyrx_daemon::test_utils::{
    can_access_input_devices, can_access_uinput, OutputCapture, VirtualKeyboard,
};
use tempfile::TempDir;

/// A running in-process daemon, scoped to every keyboard whose name starts
/// with `scope_prefix` (so a test can add/remove virtual keyboards under
/// that prefix and the daemon will react to them).
struct Harness {
    capture: OutputCapture,
    /// Kept for parity with the other harnesses and possible future
    /// assertions; not read by the current tests.
    _shared: Arc<DaemonSharedState>,
    running: Arc<std::sync::atomic::AtomicBool>,
    thread: Option<JoinHandle<Result<(), DaemonError>>>,
}

impl Harness {
    fn start(scope_prefix: &str, config_dir: &Path) -> Self {
        let output_name = format!("keyrx-hp-out-{scope_prefix}-{}", std::process::id());
        let platform = Box::new(LinuxPlatform::scoped(
            &format!("{scope_prefix}*"),
            &output_name,
        ));

        let mut daemon = Daemon::new(
            platform,
            ConfigSource::ActiveProfile,
            config_dir.to_path_buf(),
        )
        .expect("daemon starts");
        let _shared = daemon.shared_state();
        let running = daemon.running_flag();
        let thread = Some(std::thread::spawn(move || daemon.run()));

        let capture =
            OutputCapture::find_by_name(&output_name, Duration::from_secs(5)).expect("output");
        Self {
            capture,
            _shared,
            running,
            thread,
        }
    }

    /// Taps `key` on `keyboard` and returns what the daemon emitted.
    fn tap(&mut self, keyboard: &mut VirtualKeyboard, key: KeyCode) -> Vec<(KeyCode, bool)> {
        self.capture.drain().expect("drain");
        let events = VirtualKeyboard::tap_events(key);
        keyboard
            .inject_sequence(&events, Some(Duration::from_millis(10)))
            .expect("inject");
        self.capture
            .collect_events(Duration::from_millis(400))
            .expect("capture")
            .iter()
            .map(|e| (e.keycode(), e.event_type() == KeyEventType::Press))
            .collect()
    }

    /// Like [`Self::tap`], but asserts nothing at all reaches the daemon's
    /// output - the keyboard must not be grabbed (or must be pass-through
    /// with no mapping firing and the device itself unreachable).
    fn tap_and_expect_silence(&mut self, keyboard: &mut VirtualKeyboard, key: KeyCode) {
        self.capture.drain().expect("drain");
        let events = VirtualKeyboard::tap_events(key);
        keyboard
            .inject_sequence(&events, Some(Duration::from_millis(10)))
            .expect("inject");
        let seen = self
            .capture
            .collect_events(Duration::from_millis(300))
            .expect("capture");
        assert!(
            seen.is_empty(),
            "expected the daemon's output to stay silent (device should not be grabbed), got {seen:?}"
        );
    }
}

/// Polls `condition` until it is true or `timeout` elapses (asserting on
/// timeout). A free function, not a method, so the closure can freely borrow
/// (and mutate through) the harness without fighting a receiver borrow.
fn wait_until(mut condition: impl FnMut() -> bool, timeout: Duration, what: &str) {
    let deadline = Instant::now() + timeout;
    while !condition() {
        assert!(Instant::now() < deadline, "timed out waiting for: {what}");
        std::thread::sleep(Duration::from_millis(20));
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

fn devices_accessible() -> bool {
    if can_access_uinput() && can_access_input_devices() {
        return true;
    }
    eprintln!("SKIPPED: needs /dev/uinput and /dev/input access (input group)");
    false
}

/// Compiles a blank profile with the given Rhai `device_start`/`map` body as
/// the active profile.
fn active_profile_with(dir: &Path, rhai_body: &str) -> Arc<ProfileManager> {
    let manager = Arc::new(ProfileManager::new(dir.to_path_buf()).expect("profile manager"));
    manager
        .create("t", ProfileTemplate::Blank)
        .expect("create profile");
    manager.set_config("t", rhai_body).expect("set config");
    manager.activate("t").expect("activate");
    manager
}

fn new_keyboard(tag: &str, name: &str) -> VirtualKeyboard {
    let kb = VirtualKeyboard::create(&format!("keyrx-hp-{tag}-{name}")).expect("virtual keyboard");
    std::thread::sleep(Duration::from_millis(200)); // let udev register it
    kb
}

// ---------------------------------------------------------------------
// Lead #2: only devices a device_start pattern matches are grabbed.
// ---------------------------------------------------------------------

#[test]
fn devices_outside_every_pattern_are_not_grabbed_or_passed_through() {
    if !devices_accessible() {
        return;
    }
    let tag = format!("filt-{}", std::process::id());
    let dir = TempDir::new().unwrap();
    // Only "*-match" is remapped; "*-other" matches no block at all.
    let _manager = active_profile_with(
        dir.path(),
        &format!(
            r#"
device_start("*{tag}-match*");
    map("VK_A", "VK_B");
device_end();
"#
        ),
    );

    let mut matched = new_keyboard(&tag, "match");
    let mut other = new_keyboard(&tag, "other");
    let mut h = Harness::start(&format!("keyrx-hp-{tag}-"), dir.path());

    assert_eq!(h.tap(&mut matched, KeyCode::A), tapped(KeyCode::B));
    h.tap_and_expect_silence(&mut other, KeyCode::A);
}

// ---------------------------------------------------------------------
// Lead #1: hotplug - a keyboard plugged in after startup is picked up
// without a restart, and one unplugged is dropped cleanly.
// ---------------------------------------------------------------------

#[test]
fn a_keyboard_plugged_in_after_startup_is_grabbed_without_restart() {
    if !devices_accessible() {
        return;
    }
    let tag = format!("plug-{}", std::process::id());
    let dir = TempDir::new().unwrap();
    let _manager = active_profile_with(
        dir.path(),
        r#"
device_start("*");
    map("VK_A", "VK_B");
device_end();
"#,
    );

    let mut first = new_keyboard(&tag, "first");
    let mut h = Harness::start(&format!("keyrx-hp-{tag}-"), dir.path());
    assert_eq!(h.tap(&mut first, KeyCode::A), tapped(KeyCode::B));

    // Plug in a second keyboard AFTER the daemon is already running.
    let mut second = new_keyboard(&tag, "second");
    wait_until(
        || h.tap(&mut second, KeyCode::A) == tapped(KeyCode::B),
        Duration::from_secs(5),
        "hotplugged keyboard to be grabbed and remapped",
    );

    // Unplugging it must not break the loop for the surviving keyboard.
    drop(second);
    assert_eq!(h.tap(&mut first, KeyCode::A), tapped(KeyCode::B));
    assert!(
        h.running.load(Ordering::SeqCst),
        "event loop must still be running after a device vanished"
    );
}

// ---------------------------------------------------------------------
// Lead #5: emergency escape chord (LCtrl+RCtrl+Escape).
// ---------------------------------------------------------------------

#[test]
fn emergency_chord_releases_devices_and_stops_the_loop() {
    if !devices_accessible() {
        return;
    }
    let tag = format!("esc-{}", std::process::id());
    let dir = TempDir::new().unwrap();
    // CapsLock is remapped; Ctrl/Escape are left alone so the chord is on
    // untouched physical keys.
    let _manager = active_profile_with(
        dir.path(),
        r#"
device_start("*");
    map("VK_CapsLock", "VK_B");
device_end();
"#,
    );

    let mut kbd = new_keyboard(&tag, "a");
    let mut h = Harness::start(&format!("keyrx-hp-{tag}-"), dir.path());
    assert_eq!(h.tap(&mut kbd, KeyCode::CapsLock), tapped(KeyCode::B));
    assert!(h.running.load(Ordering::SeqCst));

    kbd.inject(KeyEvent::press(KeyCode::LCtrl))
        .expect("press LCtrl");
    kbd.inject(KeyEvent::press(KeyCode::RCtrl))
        .expect("press RCtrl");
    kbd.inject(KeyEvent::press(KeyCode::Escape))
        .expect("press Escape");

    wait_until(
        || !h.running.load(Ordering::SeqCst),
        Duration::from_secs(3),
        "event loop to stop after the emergency chord",
    );
}

// ---------------------------------------------------------------------
// H6: events this daemon cannot map (no KeyCode, or non-key entirely) are
// forwarded to the output device instead of being silently dropped.
// ---------------------------------------------------------------------

/// Polls the capture device's raw evdev stream (bypassing `KeyCode`
/// decoding entirely - `OutputCapture::collect_events` only surfaces
/// events `evdev_to_keycode` recognizes) for the first `(event_type,
/// code)` match, returning its value.
fn wait_for_raw_event(
    capture: &mut OutputCapture,
    event_type: u16,
    code: u16,
    timeout: Duration,
) -> Option<i32> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Ok(events) = capture.device_mut().fetch_events() {
            for ev in events {
                if ev.event_type().0 == event_type && ev.code() == code {
                    return Some(ev.value());
                }
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    None
}

#[test]
fn unmappable_events_are_forwarded_to_the_output_device_instead_of_dropped() {
    if !devices_accessible() {
        return;
    }
    // Sanity check the premise: keyrx has no KeyCode for KEY_BRIGHTNESSUP.
    assert!(evdev_to_keycode(evdev::Key::KEY_BRIGHTNESSUP.code()).is_none());

    let tag = format!("raw-{}", std::process::id());
    let dir = TempDir::new().unwrap();
    let _manager = active_profile_with(
        dir.path(),
        r#"
device_start("*");
    map("VK_A", "VK_B");
device_end();
"#,
    );

    let mut kbd = new_keyboard(&tag, "a");
    let mut h = Harness::start(&format!("keyrx-hp-{tag}-"), dir.path());
    h.capture.drain().expect("drain");

    // An EV_KEY code with no KeyCode (vendor/consumer key): press, release.
    kbd.inject_raw(
        evdev::EventType::KEY.0,
        evdev::Key::KEY_BRIGHTNESSUP.code(),
        1,
    )
    .expect("inject brightness press");
    kbd.inject_raw(
        evdev::EventType::KEY.0,
        evdev::Key::KEY_BRIGHTNESSUP.code(),
        0,
    )
    .expect("inject brightness release");

    // A non-key event entirely: relative pointer motion (a keyboard with an
    // integrated trackpoint/combo receiver).
    kbd.inject_raw(
        evdev::EventType::RELATIVE.0,
        evdev::RelativeAxisType::REL_X.0,
        5,
    )
    .expect("inject REL_X");

    assert_eq!(
        wait_for_raw_event(
            &mut h.capture,
            evdev::EventType::KEY.0,
            evdev::Key::KEY_BRIGHTNESSUP.code(),
            Duration::from_secs(2),
        ),
        Some(1),
        "KEY_BRIGHTNESSUP press must reach the output device, not be dropped"
    );
    assert_eq!(
        wait_for_raw_event(
            &mut h.capture,
            evdev::EventType::KEY.0,
            evdev::Key::KEY_BRIGHTNESSUP.code(),
            Duration::from_secs(2),
        ),
        Some(0),
        "KEY_BRIGHTNESSUP release must reach the output device, not be dropped"
    );
    assert_eq!(
        wait_for_raw_event(
            &mut h.capture,
            evdev::EventType::RELATIVE.0,
            evdev::RelativeAxisType::REL_X.0,
            Duration::from_secs(2),
        ),
        Some(5),
        "REL_X must reach the output device, not be dropped"
    );

    // The daemon keeps remapping normal keys on the same device afterwards.
    assert_eq!(h.tap(&mut kbd, KeyCode::A), tapped(KeyCode::B));
}

// ---------------------------------------------------------------------
// H8: DeviceService.active reflects what the daemon actually grabbed, not
// every device the OS can see - and stays correct after a hotplug rescan.
// ---------------------------------------------------------------------

/// Finds `name`'s `DeviceInfo` in `devices` (device names are unique in
/// this test - every virtual keyboard here carries a per-test tag).
fn find_device<'a>(
    devices: &'a [keyrx_daemon::services::device_service::DeviceInfo],
    name: &str,
) -> &'a keyrx_daemon::services::device_service::DeviceInfo {
    devices.iter().find(|d| d.name == name).unwrap_or_else(|| {
        panic!("device '{name}' not in DeviceService::list_devices(): {devices:?}")
    })
}

#[test]
fn device_service_active_matches_what_the_daemon_actually_grabbed() {
    if !devices_accessible() {
        return;
    }
    let tag = format!("actv-{}", std::process::id());
    let dir = TempDir::new().unwrap();
    let _manager = active_profile_with(
        dir.path(),
        r#"
device_start("*");
    map("VK_A", "VK_B");
device_end();
"#,
    );

    let grabbed = new_keyboard(&tag, "grabbed");
    // This one exists on the system but is outside the daemon's scope
    // prefix, so it is never grabbed - `active` must stay false for it.
    let outside_scope = VirtualKeyboard::create(&format!("keyrx-hp-other-{tag}")).expect("kbd");
    std::thread::sleep(Duration::from_millis(200));

    let h = Harness::start(&format!("keyrx-hp-{tag}-"), dir.path());

    let device_service =
        keyrx_daemon::services::DeviceService::new(dir.path().join("device-service-scratch"));
    device_service.attach_daemon_state(Arc::clone(&h._shared));

    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let devices = runtime
        .block_on(device_service.list_devices())
        .expect("list_devices");

    assert!(
        find_device(&devices, grabbed.name()).active,
        "the device the daemon grabbed must report active: true"
    );
    assert!(
        !find_device(&devices, outside_scope.name()).active,
        "a device outside the daemon's scope must report active: false, not just \
         'the OS can see it' (H8)"
    );

    drop(outside_scope);
    drop(grabbed);
}

#[test]
fn device_service_active_updates_after_hotplug() {
    if !devices_accessible() {
        return;
    }
    let tag = format!("actvhp-{}", std::process::id());
    let dir = TempDir::new().unwrap();
    let _manager = active_profile_with(
        dir.path(),
        r#"
device_start("*");
    map("VK_A", "VK_B");
device_end();
"#,
    );

    // A keyboard present before startup, same as the other hotplug test -
    // keeps the daemon's device_manager non-empty from the start so this
    // test only exercises "hotplug adds a second device", not "grab the
    // very first device from zero" (a separate, already-covered path).
    let mut first = new_keyboard(&tag, "first");
    let mut h = Harness::start(&format!("keyrx-hp-{tag}-"), dir.path());
    assert_eq!(h.tap(&mut first, KeyCode::A), tapped(KeyCode::B));

    let device_service =
        keyrx_daemon::services::DeviceService::new(dir.path().join("device-service-scratch"));
    device_service.attach_daemon_state(Arc::clone(&h._shared));
    let runtime = tokio::runtime::Runtime::new().expect("runtime");

    // Plug in AFTER the daemon started; the hotplug rescan must grab it and
    // publish the updated set for DeviceService to see (H8: device_count -
    // and now the active set - must refresh on hotplug too, not just
    // startup/reload).
    let mut plugged = new_keyboard(&tag, "late");
    wait_until(
        || {
            let devices = runtime.block_on(device_service.list_devices()).unwrap();
            devices
                .iter()
                .find(|d| d.name == plugged.name())
                .is_some_and(|d| d.active)
        },
        Duration::from_secs(5),
        "DeviceService to report the hotplugged device active",
    );

    // And tapping it actually goes through the live daemon.
    assert_eq!(h.tap(&mut plugged, KeyCode::A), tapped(KeyCode::B));
}
