//! G8 end to end: per-device configuration on the live Linux event loop.
//!
//! Two virtual keyboards ("...numpad..." and "...main...") feed a real daemon
//! (`Daemon::run`) scoped to them. The config has two `device_start` blocks
//! and a `when_device` condition. Before G8 the daemon applied only the first
//! block to every device and `when_device` never matched.
//!
//! Needs /dev/uinput and /dev/input access (the `input` group); skipped
//! otherwise.

#![cfg(target_os = "linux")]

use std::sync::atomic::Ordering;
use std::time::Duration;

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEventType;
use keyrx_daemon::config::{ProfileManager, ProfileTemplate};
use keyrx_daemon::daemon::{ConfigSource, Daemon};
use keyrx_daemon::platform::linux::LinuxPlatform;
use keyrx_daemon::test_utils::{OutputCapture, VirtualKeyboard};

/// numpad: A->B plus (when_device numpad) Q->W; main: A->C, Q unmapped.
const MULTI_DEVICE: &str = r#"
device_start("*keyrx-md-numpad*");
  map("VK_A", "VK_B");
device_end();

device_start("*");
  map("VK_A", "VK_C");
  when_device_start("*NUMPAD*");
    map("VK_Q", "VK_W");
  when_device_end();
device_end();
"#;

fn tap(
    kbd: &mut VirtualKeyboard,
    capture: &mut OutputCapture,
    key: KeyCode,
) -> Vec<(KeyCode, bool)> {
    capture.drain().expect("drain");
    kbd.inject_sequence(
        &VirtualKeyboard::tap_events(key),
        Some(Duration::from_millis(10)),
    )
    .expect("inject");
    capture
        .collect_events(Duration::from_millis(300))
        .expect("capture")
        .iter()
        .map(|e| (e.keycode(), e.event_type() == KeyEventType::Press))
        .collect()
}

fn tapped(key: KeyCode) -> Vec<(KeyCode, bool)> {
    vec![(key, true), (key, false)]
}

#[test]
fn each_device_uses_its_own_block_and_when_device_matches_by_name() {
    keyrx_daemon::skip_if_no_uinput!();
    let dir = tempfile::tempdir().unwrap();
    let manager = ProfileManager::new(dir.path().to_path_buf()).unwrap();
    manager.create("md", ProfileTemplate::Blank).unwrap();
    manager.set_config("md", MULTI_DEVICE).unwrap();
    assert!(manager.activate("md").unwrap().success);

    // Distinct stems per run, so parallel runs never grab each other's devices.
    let tag = std::process::id();
    let mut numpad = VirtualKeyboard::create(&format!("keyrx-md-numpad-{tag}")).unwrap();
    let mut main = VirtualKeyboard::create(&format!("keyrx-md-main-{tag}")).unwrap();
    std::thread::sleep(Duration::from_millis(200));

    let output = format!("keyrx-md-out-{tag}");
    let platform = LinuxPlatform::scoped(&format!("keyrx-md-*-{tag}*"), &output);
    let mut daemon = Daemon::new(
        Box::new(platform),
        ConfigSource::ActiveProfile,
        dir.path().to_path_buf(),
    )
    .expect("daemon starts");
    assert_eq!(
        daemon.device_count(),
        2,
        "daemon must grab exactly the two test keyboards"
    );
    let running = daemon.running_flag();
    let thread = std::thread::spawn(move || daemon.run());
    let mut capture = OutputCapture::find_by_name(&output, Duration::from_secs(5)).unwrap();

    let results = [
        (
            "numpad A (own block)",
            tap(&mut numpad, &mut capture, KeyCode::A),
            tapped(KeyCode::B),
        ),
        (
            "main A (* block)",
            tap(&mut main, &mut capture, KeyCode::A),
            tapped(KeyCode::C),
        ),
        // numpad's first matching block has no Q mapping: passed through.
        (
            "numpad Q",
            tap(&mut numpad, &mut capture, KeyCode::Q),
            tapped(KeyCode::Q),
        ),
        (
            "main Q (no when_device match)",
            tap(&mut main, &mut capture, KeyCode::Q),
            tapped(KeyCode::Q),
        ),
    ];

    running.store(false, Ordering::SeqCst);
    thread.join().expect("daemon thread").expect("daemon run");
    for (what, got, want) in results {
        assert_eq!(got, want, "{what}");
    }
}

/// when_device inside the block a device routes to: a numpad-named device
/// without its own block falls to "*" and its when_device condition matches
/// its NAME (case-insensitive), not just the opaque device id.
#[test]
fn when_device_condition_matches_device_name() {
    keyrx_daemon::skip_if_no_uinput!();
    let config = r#"
device_start("*");
  map("VK_A", "VK_C");
  when_device_start("*NUMPAD*");
    map("VK_Q", "VK_W");
  when_device_end();
device_end();
"#;
    let dir = tempfile::tempdir().unwrap();
    let manager = ProfileManager::new(dir.path().to_path_buf()).unwrap();
    manager.create("wd", ProfileTemplate::Blank).unwrap();
    manager.set_config("wd", config).unwrap();
    assert!(manager.activate("wd").unwrap().success);

    let tag = std::process::id();
    let mut numpad = VirtualKeyboard::create(&format!("keyrx-wd-numpad-{tag}")).unwrap();
    let mut main = VirtualKeyboard::create(&format!("keyrx-wd-main-{tag}")).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    let output = format!("keyrx-wd-out-{tag}");
    let platform = LinuxPlatform::scoped(&format!("keyrx-wd-*-{tag}*"), &output);
    let mut daemon = Daemon::new(
        Box::new(platform),
        ConfigSource::ActiveProfile,
        dir.path().to_path_buf(),
    )
    .expect("daemon starts");
    let running = daemon.running_flag();
    let thread = std::thread::spawn(move || daemon.run());
    let mut capture = OutputCapture::find_by_name(&output, Duration::from_secs(5)).unwrap();

    let numpad_q = tap(&mut numpad, &mut capture, KeyCode::Q);
    let main_q = tap(&mut main, &mut capture, KeyCode::Q);
    let main_a = tap(&mut main, &mut capture, KeyCode::A);

    running.store(false, Ordering::SeqCst);
    thread.join().expect("daemon thread").expect("daemon run");
    assert_eq!(
        numpad_q,
        tapped(KeyCode::W),
        "when_device(*NUMPAD*) on the numpad"
    );
    assert_eq!(
        main_q,
        tapped(KeyCode::Q),
        "when_device must not match the main keyboard"
    );
    assert_eq!(main_a, tapped(KeyCode::C));
}
