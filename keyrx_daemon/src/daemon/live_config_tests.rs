use super::*;
use keyrx_compiler::serialize::serialize;
use keyrx_core::config::{
    ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
};
use tempfile::TempDir;

fn krx_bytes(from: KeyCode, to: KeyCode) -> Vec<u8> {
    let root = ConfigRoot {
        version: Version::current(),
        devices: vec![DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*".to_string(),
            },
            mappings: vec![KeyMapping::simple(from, to)],
        }],
        metadata: Metadata {
            compilation_timestamp: 0,
            compiler_version: "test".to_string(),
            source_hash: "test".to_string(),
        },
    };
    serialize(&root).expect("serialize")
}

fn write_profile(dir: &Path, name: &str, from: KeyCode, to: KeyCode) -> PathBuf {
    let profiles = dir.join("profiles");
    fs::create_dir_all(&profiles).unwrap();
    let path = profiles.join(format!("{name}.krx"));
    fs::write(&path, krx_bytes(from, to)).unwrap();
    path
}

fn set_active(dir: &Path, name: &str) {
    let json = serde_json::json!({ "name": name, "activated_at": 0 });
    fs::write(dir.join(".active"), json.to_string()).unwrap();
}

fn first_mapping_output(loaded: &LoadedConfig) -> KeyMapping {
    loaded.device_config.mappings[0].clone()
}

#[test]
fn active_profile_without_active_file_is_pass_through() {
    let dir = TempDir::new().unwrap();
    let live = LiveConfig::new(dir.path().to_path_buf());
    assert!(live.load(&ConfigSource::ActiveProfile).unwrap().is_none());
}

#[test]
fn active_profile_loads_named_profile() {
    let dir = TempDir::new().unwrap();
    write_profile(dir.path(), "a", KeyCode::CapsLock, KeyCode::Escape);
    set_active(dir.path(), "a");
    let live = LiveConfig::new(dir.path().to_path_buf());

    let loaded = live.load(&ConfigSource::ActiveProfile).unwrap().unwrap();
    assert_eq!(loaded.profile.as_deref(), Some("a"));
    assert_eq!(
        first_mapping_output(&loaded),
        KeyMapping::simple(KeyCode::CapsLock, KeyCode::Escape)
    );
}

#[test]
fn legacy_plain_text_active_file_is_understood() {
    let dir = TempDir::new().unwrap();
    write_profile(dir.path(), "legacy", KeyCode::A, KeyCode::B);
    fs::write(dir.path().join(".active"), "legacy\n").unwrap();
    let live = LiveConfig::new(dir.path().to_path_buf());

    let loaded = live.load(&ConfigSource::ActiveProfile).unwrap().unwrap();
    assert_eq!(loaded.profile.as_deref(), Some("legacy"));
}

#[test]
fn active_profile_without_krx_is_an_error_not_pass_through() {
    let dir = TempDir::new().unwrap();
    set_active(dir.path(), "ghost");
    let live = LiveConfig::new(dir.path().to_path_buf());
    assert!(live.load(&ConfigSource::ActiveProfile).is_err());
}

#[test]
fn explicit_file_outside_profiles_has_no_profile_name() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.krx");
    fs::write(&path, krx_bytes(KeyCode::A, KeyCode::B)).unwrap();
    let live = LiveConfig::new(dir.path().to_path_buf());

    let loaded = live
        .load(&ConfigSource::File(path.clone()))
        .unwrap()
        .unwrap();
    assert_eq!(loaded.profile, None);
    assert_eq!(loaded.path, path);
}

#[test]
fn explicit_file_of_a_profile_reports_that_profile() {
    let dir = TempDir::new().unwrap();
    let path = write_profile(dir.path(), "gaming", KeyCode::A, KeyCode::B);
    let live = LiveConfig::new(dir.path().to_path_buf());

    let loaded = live.load(&ConfigSource::File(path)).unwrap().unwrap();
    assert_eq!(loaded.profile.as_deref(), Some("gaming"));
}

#[test]
fn reload_source_prefers_activation_then_loaded_source() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("config.krx");
    fs::write(&file, krx_bytes(KeyCode::A, KeyCode::B)).unwrap();
    let mut live = LiveConfig::new(dir.path().to_path_buf());

    assert_eq!(live.reload_source(None), ConfigSource::ActiveProfile);

    let loaded = live.load(&ConfigSource::File(file.clone())).unwrap();
    live.set_loaded(loaded);
    assert_eq!(live.reload_source(None), ConfigSource::File(file));
    assert_eq!(
        live.reload_source(Some("b".to_string())),
        ConfigSource::Profile("b".to_string())
    );

    write_profile(dir.path(), "b", KeyCode::A, KeyCode::C);
    let loaded = live.load(&ConfigSource::Profile("b".to_string())).unwrap();
    live.set_loaded(loaded);
    assert_eq!(
        live.reload_source(None),
        ConfigSource::Profile("b".to_string())
    );
}

// ---- Daemon-side reload: what status reports vs. what is loaded ----------

use crate::daemon::{apply_loaded, reload_remapping, DaemonSharedState};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn shared_state() -> DaemonSharedState {
    DaemonSharedState::new(Arc::new(AtomicBool::new(true)), None, PathBuf::new(), 0)
}

#[test]
fn activation_switches_mappings_and_status_together() {
    let dir = TempDir::new().unwrap();
    write_profile(dir.path(), "a", KeyCode::CapsLock, KeyCode::Escape);
    let b_path = write_profile(dir.path(), "b", KeyCode::CapsLock, KeyCode::LCtrl);
    set_active(dir.path(), "a");
    let mut live = LiveConfig::new(dir.path().to_path_buf());
    let shared = shared_state();

    let start = live.load(&ConfigSource::ActiveProfile).unwrap();
    assert!(apply_loaded(&mut live, &shared, start).is_some());
    assert_eq!(shared.get_active_profile().as_deref(), Some("a"));

    shared.request_activation("b");
    let state = reload_remapping(&mut live, &shared).unwrap().unwrap();
    assert!(state
        .lookup()
        .find_mapping(KeyCode::CapsLock, &keyrx_core::runtime::DeviceState::new())
        .is_some());
    assert_eq!(shared.get_active_profile().as_deref(), Some("b"));
    assert_eq!(shared.get_config_path(), b_path);
    assert_eq!(
        live.loaded().map(first_mapping_output),
        Some(KeyMapping::simple(KeyCode::CapsLock, KeyCode::LCtrl))
    );
}

#[test]
fn failed_activation_keeps_previous_config_and_status() {
    let dir = TempDir::new().unwrap();
    write_profile(dir.path(), "a", KeyCode::CapsLock, KeyCode::Escape);
    set_active(dir.path(), "a");
    let mut live = LiveConfig::new(dir.path().to_path_buf());
    let shared = shared_state();
    let start = live.load(&ConfigSource::ActiveProfile).unwrap();
    apply_loaded(&mut live, &shared, start);

    shared.request_activation("missing");
    assert!(reload_remapping(&mut live, &shared).is_err());
    assert_eq!(shared.get_active_profile().as_deref(), Some("a"));
    assert_eq!(live.loaded().and_then(|l| l.profile.as_deref()), Some("a"));
    // The request was consumed: the next plain reload re-reads "a".
    assert!(reload_remapping(&mut live, &shared).unwrap().is_some());
    assert_eq!(shared.get_active_profile().as_deref(), Some("a"));
}

#[test]
fn explicit_file_is_reported_without_a_profile_name() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("config.krx");
    fs::write(&file, krx_bytes(KeyCode::A, KeyCode::B)).unwrap();
    let mut live = LiveConfig::new(dir.path().to_path_buf());
    let shared = shared_state();

    let start = live.load(&ConfigSource::File(file.clone())).unwrap();
    apply_loaded(&mut live, &shared, start);
    assert_eq!(shared.get_active_profile(), None);
    assert_eq!(shared.get_config_path(), file);
}
