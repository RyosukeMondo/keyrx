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
use std::sync::Arc;
use std::time::Duration;

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEventType;
use keyrx_daemon::config::{ProfileManager, ProfileTemplate};
use keyrx_daemon::daemon::ConfigSource;
use keyrx_daemon::services::ProfileService;
use tempfile::TempDir;

#[path = "common/live_daemon.rs"]
mod live_daemon;
use live_daemon::{tapped, Harness};

const CAPS_TO_LCTRL: &str = r#"
device_start("*");
  map("VK_CapsLock", "VK_LCtrl");
device_end();
"#;

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

/// `activate` means "live": a tap typed the instant it returns must already
/// hit the new mapping (it used to race the daemon's swap and be handled by,
/// or lost between, the old and new config).
#[test]
fn activate_returns_only_once_the_new_mapping_is_live() {
    keyrx_daemon::skip_if_no_uinput!();
    let dir = TempDir::new().unwrap();
    let manager = profiles(dir.path());
    assert!(manager.activate("a").expect("activate a").success);
    let mut h = Harness::start("immediate", ConfigSource::ActiveProfile, dir.path());

    let service = ProfileService::new(manager);
    service.attach_daemon_state(Arc::clone(&h.shared));
    for _ in 0..5 {
        activate(&service, "b");
        // No wait_for_profile: the result itself must imply the swap.
        assert_eq!(h.shared.get_active_profile().as_deref(), Some("b"));
        assert_eq!(h.tap(KeyCode::CapsLock), tapped(KeyCode::LCtrl));
        activate(&service, "a");
        assert_eq!(h.tap(KeyCode::CapsLock), tapped(KeyCode::Escape));
    }
}

/// Saving the loaded profile's `.rhai` recompiles and applies it by itself;
/// a source that does not compile leaves the running config alone.
#[test]
fn editing_the_loaded_profile_source_hot_reloads_it() {
    keyrx_daemon::skip_if_no_uinput!();
    let dir = TempDir::new().unwrap();
    let manager = profiles(dir.path());
    assert!(manager.activate("a").expect("activate a").success);
    let mut h = Harness::start("watch", ConfigSource::ActiveProfile, dir.path());
    let _watcher = keyrx_daemon::daemon::config_watch::spawn(
        Arc::clone(&manager),
        Arc::clone(&h.shared),
        Arc::clone(&h.running),
        Duration::from_millis(50),
    );
    assert_eq!(h.tap(KeyCode::CapsLock), tapped(KeyCode::Escape));

    let source = dir.path().join("profiles/a.rhai");
    std::fs::write(&source, CAPS_TO_LCTRL).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while h.tap(KeyCode::CapsLock) != tapped(KeyCode::LCtrl) {
        assert!(
            std::time::Instant::now() < deadline,
            "the edited profile never went live"
        );
    }

    std::fs::write(&source, "device_start(\"*\"\n  map(").unwrap();
    std::thread::sleep(Duration::from_millis(800));
    assert_eq!(
        h.tap(KeyCode::CapsLock),
        tapped(KeyCode::LCtrl),
        "a source that does not compile must not replace the running config"
    );
}
