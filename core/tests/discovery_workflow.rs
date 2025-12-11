#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::field_reassign_with_default,
    clippy::useless_conversion,
    clippy::assertions_on_constants,
    clippy::manual_div_ceil,
    clippy::manual_strip,
    clippy::len_zero,
    clippy::redundant_closure,
    clippy::manual_range_contains,
    clippy::default_constructed_unit_structs,
    clippy::clone_on_copy,
    clippy::io_other_error,
    clippy::bool_assert_comparison,
    clippy::approx_constant,
    clippy::let_unit_value,
    clippy::while_let_on_iterator,
    clippy::await_holding_lock,
    clippy::unnecessary_cast,
    clippy::drop_non_drop,
    clippy::needless_range_loop,
    unused_imports,
    unused_variables,
    dead_code,
    unsafe_code,
    clippy::collapsible_if,
    clippy::bool_comparison,
    unexpected_cfgs
)]
use chrono::Utc;
use keyrx_core::discovery::{
    default_schema_version, profile_path, DeviceId, DeviceProfile, DeviceRegistry, ProfileSource,
    RegistryStatus, SessionStatus, SessionUpdate,
};
use keyrx_core::engine::{InputEvent, KeyCode};
use std::env;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};
use tempfile::tempdir;

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn lock_env() -> MutexGuard<'static, ()> {
    env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn with_temp_config<F: FnOnce(&Path)>(f: F) {
    let _guard = lock_env();
    let temp = tempdir().unwrap();
    let prev_xdg = env::var("XDG_CONFIG_HOME").ok();
    let prev_home = env::var("HOME").ok();

    env::set_var("XDG_CONFIG_HOME", temp.path());
    env::remove_var("HOME");

    f(temp.path());

    match prev_xdg {
        Some(val) => env::set_var("XDG_CONFIG_HOME", val),
        None => env::remove_var("XDG_CONFIG_HOME"),
    }
    match prev_home {
        Some(val) => env::set_var("HOME", val),
        None => env::remove_var("HOME"),
    }
}

fn press(scan_code: u16, device: &str) -> InputEvent {
    InputEvent::with_metadata(
        KeyCode::Unknown(scan_code),
        true,
        scan_code as u64,
        Some(device.to_string()),
        false,
        false,
        scan_code,
        None,
    )
}

fn press_key(key: KeyCode, scan_code: u16, device: &str) -> InputEvent {
    InputEvent::with_metadata(
        key,
        true,
        scan_code as u64,
        Some(device.to_string()),
        false,
        false,
        scan_code,
        None,
    )
}

fn build_profile(summary: &keyrx_core::discovery::DiscoverySummary) -> DeviceProfile {
    DeviceProfile {
        schema_version: default_schema_version(),
        vendor_id: summary.device_id.vendor_id,
        product_id: summary.device_id.product_id,
        name: Some("Mock Keyboard".to_string()),
        discovered_at: Utc::now(),
        rows: summary.rows,
        cols_per_row: summary.cols_per_row.clone(),
        keymap: summary.keymap.clone(),
        aliases: summary.aliases.clone(),
        source: ProfileSource::Discovered,
    }
}

#[test]
fn discovery_flow_persists_and_reloads_profile() {
    with_temp_config(|_| {
        let device_id = DeviceId::new(0x1, 0x2);
        let device_path = "/dev/input/mock0";
        let mut session = keyrx_core::discovery::DiscoverySession::new(device_id, 1, vec![2])
            .unwrap()
            .with_target_device_id(device_path);

        let mut summary = None;
        for event in [press(30, device_path), press(31, device_path)] {
            if let SessionUpdate::Finished(done) = session.handle_event(&event) {
                summary = Some(done);
                break;
            }
        }

        let summary = summary.unwrap();
        assert_eq!(summary.status, SessionStatus::Completed);
        assert_eq!(summary.captured, 2);
        assert!(summary.unmapped.is_empty());

        let mut registry = DeviceRegistry::new();
        let path = registry
            .save_profile(build_profile(&summary))
            .expect("profile should save");
        assert!(path.exists());

        let entry = registry.load_or_default(device_id);
        assert_eq!(entry.status, RegistryStatus::Ready);
        assert_eq!(entry.profile.keymap.len(), 2);
        assert_eq!(entry.profile.aliases.len(), 2);
    });
}

#[test]
fn duplicate_key_is_reported_and_profile_saved() {
    with_temp_config(|_| {
        let device_id = DeviceId::new(0x10, 0x20);
        let device_path = "/dev/input/mock1";
        let mut session = keyrx_core::discovery::DiscoverySession::new(device_id, 1, vec![3])
            .unwrap()
            .with_target_device_id(device_path);

        let events = [
            press(10, device_path),
            press(11, device_path),
            // Re-press the first key to trigger duplicate detection on a later position
            press(10, device_path),
            press(12, device_path),
        ];

        let mut summary = None;
        for event in events {
            if let SessionUpdate::Finished(done) = session.handle_event(&event) {
                summary = Some(done);
                break;
            }
        }

        let summary = summary.expect("session should complete");
        assert_eq!(summary.status, SessionStatus::Completed);
        assert_eq!(summary.duplicates.len(), 1);
        assert_eq!(summary.captured, 3);

        let mut registry = DeviceRegistry::new();
        registry
            .save_profile(build_profile(&summary))
            .expect("profile should persist after duplicate handling");

        let entry = registry.load_or_default(device_id);
        assert_eq!(entry.status, RegistryStatus::Ready);
        assert_eq!(entry.profile.keymap.len(), 3);
        assert_eq!(entry.profile.aliases.len(), 3);
    });
}

#[test]
fn recovers_from_corrupt_profile_with_rediscovery() {
    with_temp_config(|config_root| {
        let device_id = DeviceId::new(0xAA, 0xBB);
        let path = profile_path(device_id);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).unwrap();
        }
        fs::write(&path, "not-json").unwrap();
        assert!(path.starts_with(config_root));

        let mut registry = DeviceRegistry::new();
        let entry = registry.load_or_default(device_id);
        assert!(
            entry.discover_needed(),
            "corrupt profiles should request discovery"
        );
        assert_eq!(entry.profile.rows, 0);

        let device_path = "/dev/input/mock2";
        let mut session = keyrx_core::discovery::DiscoverySession::new(device_id, 1, vec![2])
            .unwrap()
            .with_target_device_id(device_path);

        let mut summary = None;
        for event in [press(40, device_path), press(41, device_path)] {
            if let SessionUpdate::Finished(done) = session.handle_event(&event) {
                summary = Some(done);
                break;
            }
        }
        let summary = summary.unwrap();
        assert_eq!(summary.status, SessionStatus::Completed);

        registry
            .save_profile(build_profile(&summary))
            .expect("recovered profile should save");

        let refreshed = registry.load_or_default(device_id);
        assert!(!refreshed.discover_needed());
        assert_eq!(refreshed.status, RegistryStatus::Ready);
        assert_eq!(refreshed.profile.keymap.len(), 2);
    });
}

#[test]
fn force_re_discovers_and_replaces_profile() {
    with_temp_config(|_| {
        let device_id = DeviceId::new(0x33, 0x44);
        let mut registry = DeviceRegistry::new();

        let original_profile = DeviceProfile {
            schema_version: default_schema_version(),
            vendor_id: device_id.vendor_id,
            product_id: device_id.product_id,
            name: Some("First pass".to_string()),
            discovered_at: Utc::now(),
            rows: 1,
            cols_per_row: vec![1],
            keymap: [(10u16, keyrx_core::discovery::PhysicalKey::new(10, 0, 0))]
                .into_iter()
                .collect(),
            aliases: [("r0_c0".to_string(), 10)].into_iter().collect(),
            source: ProfileSource::Discovered,
        };
        registry
            .save_profile(original_profile)
            .expect("initial profile should save");

        // Simulate a forced re-discovery with a different layout and scan codes.
        let device_path = "/dev/input/mock3";
        let mut session = keyrx_core::discovery::DiscoverySession::new(device_id, 1, vec![2])
            .unwrap()
            .with_target_device_id(device_path);

        let mut summary = None;
        for event in [press(20, device_path), press(21, device_path)] {
            if let SessionUpdate::Finished(done) = session.handle_event(&event) {
                summary = Some(done);
                break;
            }
        }
        let summary = summary.unwrap();
        let path = registry
            .save_profile(build_profile(&summary))
            .expect("forced re-discovery should overwrite");
        assert!(path.exists());

        // Reload using a new registry to ensure disk state was updated.
        let mut fresh_registry = DeviceRegistry::new();
        let entry = fresh_registry.load_or_default(device_id);
        assert_eq!(entry.status, RegistryStatus::Ready);
        assert_eq!(entry.profile.rows, 1);
        assert_eq!(entry.profile.keymap.len(), 2);
        assert!(!entry.profile.keymap.contains_key(&10));
        assert!(entry.profile.keymap.contains_key(&20));
    });
}

#[test]
fn emergency_exit_bypasses_and_skips_write() {
    with_temp_config(|_| {
        let device_id = DeviceId::new(0xDE, 0xAD);
        let device_path = "/dev/input/mock4";

        // Detector mirrors the CLI emergency-exit hook (Ctrl+Alt+Shift+Esc).
        use std::sync::Arc;
        let state = Arc::new(Mutex::new((false, false, false)));
        let state_clone = state.clone();

        let detector = move |event: &InputEvent| {
            let mut guard = state_clone.lock().unwrap();
            match event.key {
                KeyCode::LeftCtrl | KeyCode::RightCtrl => guard.0 = event.pressed,
                KeyCode::LeftAlt | KeyCode::RightAlt => guard.1 = event.pressed,
                KeyCode::LeftShift | KeyCode::RightShift => guard.2 = event.pressed,
                KeyCode::Escape => {
                    if event.pressed && guard.0 && guard.1 && guard.2 {
                        return true;
                    }
                }
                _ => {}
            }
            false
        };

        let mut session = keyrx_core::discovery::DiscoverySession::new(device_id, 1, vec![4])
            .unwrap()
            .with_target_device_id(device_path)
            .with_emergency_exit_detector(detector);

        // Press modifiers then Esc to trigger the bypass before any layout capture completes.
        let events = [
            press_key(KeyCode::LeftCtrl, 60, device_path),
            press_key(KeyCode::LeftAlt, 61, device_path),
            press_key(KeyCode::LeftShift, 62, device_path),
            press_key(KeyCode::Escape, 63, device_path),
        ];

        let mut summary = None;
        for event in events {
            if let SessionUpdate::Finished(done) = session.handle_event(&event) {
                summary = Some(done);
                break;
            }
        }

        let summary = summary.expect("bypass should finish session");
        assert_eq!(summary.status, SessionStatus::Bypassed);
        assert_eq!(summary.message.as_deref(), Some("emergency-exit triggered"));

        let profile_path = profile_path(device_id);
        assert!(
            !profile_path.exists(),
            "bypass should not write discovery profiles"
        );
    });
}
