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
    loaded.devices[0].mappings[0].clone()
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

    // A profile-based daemon follows .active on a plain reload.
    write_profile(dir.path(), "b", KeyCode::A, KeyCode::C);
    let loaded = live.load(&ConfigSource::Profile("b".to_string())).unwrap();
    live.set_loaded(loaded);
    assert_eq!(live.reload_source(None), ConfigSource::ActiveProfile);
}

// ---- Daemon-side reload: what status reports vs. what is loaded ----------

use crate::daemon::{apply_loaded, reload_remapping, DaemonSharedState};
use crate::platform::{DeviceInfo, Platform, PlatformResult};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn shared_state() -> DaemonSharedState {
    DaemonSharedState::new(Arc::new(AtomicBool::new(true)), None, PathBuf::new(), 0)
}

/// A `Platform` double that does nothing - these tests only care about
/// [`LiveConfig`]/[`DaemonSharedState`] bookkeeping, not device I/O.
struct NoopPlatform;

impl Platform for NoopPlatform {
    fn initialize(&mut self) -> PlatformResult<()> {
        Ok(())
    }
    fn capture_input(&mut self) -> PlatformResult<keyrx_core::runtime::event::KeyEvent> {
        Err(crate::platform::PlatformError::NoInput)
    }
    fn inject_output(
        &mut self,
        _event: keyrx_core::runtime::event::KeyEvent,
    ) -> PlatformResult<()> {
        Ok(())
    }
    fn list_devices(&self) -> PlatformResult<Vec<DeviceInfo>> {
        Ok(Vec::new())
    }
    fn shutdown(&mut self) -> PlatformResult<()> {
        Ok(())
    }
}

fn noop_platform() -> Box<dyn Platform> {
    Box::new(NoopPlatform)
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
    let mut platform = noop_platform();
    assert!(apply_loaded(&mut platform, &mut live, &shared, start).is_some());
    assert_eq!(shared.get_active_profile().as_deref(), Some("a"));

    shared.request_activation("b");
    let mut state = reload_remapping(&mut platform, &mut live, &shared)
        .unwrap()
        .unwrap();
    let routed = state.route(Some("kbd"), |id| vec![id.to_string()]).unwrap();
    assert!(routed
        .lookup
        .find_mapping(KeyCode::CapsLock, routed.state)
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
    let mut platform = noop_platform();
    apply_loaded(&mut platform, &mut live, &shared, start);

    shared.request_activation("missing");
    assert!(reload_remapping(&mut platform, &mut live, &shared).is_err());
    assert_eq!(shared.get_active_profile().as_deref(), Some("a"));
    assert_eq!(live.loaded().and_then(|l| l.profile.as_deref()), Some("a"));
    // The request was consumed: the next plain reload re-reads .active ("a").
    assert!(reload_remapping(&mut platform, &mut live, &shared)
        .unwrap()
        .is_some());
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
    let mut platform = noop_platform();
    apply_loaded(&mut platform, &mut live, &shared, start);
    assert_eq!(shared.get_active_profile(), None);
    assert_eq!(shared.get_config_path(), file);
}

#[test]
fn plain_reload_follows_active_file_and_clears_on_delete() {
    let dir = TempDir::new().unwrap();
    write_profile(dir.path(), "a", KeyCode::CapsLock, KeyCode::Escape);
    write_profile(dir.path(), "b", KeyCode::CapsLock, KeyCode::LCtrl);
    set_active(dir.path(), "a");
    let mut live = LiveConfig::new(dir.path().to_path_buf());
    let shared = shared_state();
    let start = live.load(&ConfigSource::ActiveProfile).unwrap();
    let mut platform = noop_platform();
    apply_loaded(&mut platform, &mut live, &shared, start);

    // Out-of-band activation (CLI without a daemon connection) + SIGHUP.
    set_active(dir.path(), "b");
    assert!(reload_remapping(&mut platform, &mut live, &shared)
        .unwrap()
        .is_some());
    assert_eq!(shared.get_active_profile().as_deref(), Some("b"));

    // Active profile deleted (ProfileManager removes .active) + reload.
    fs::remove_file(dir.path().join(".active")).unwrap();
    assert!(reload_remapping(&mut platform, &mut live, &shared)
        .unwrap()
        .is_none());
    assert_eq!(shared.get_active_profile(), None);
}

/// Records how many `device_start` blocks each `reconfigure_devices` call got:
/// an empty slice is "no config live", which must capture nothing.
struct RecordingPlatform(Arc<std::sync::Mutex<Vec<usize>>>);

impl Platform for RecordingPlatform {
    fn initialize(&mut self) -> PlatformResult<()> {
        Ok(())
    }
    fn capture_input(&mut self) -> PlatformResult<keyrx_core::runtime::event::KeyEvent> {
        Err(crate::platform::PlatformError::NoInput)
    }
    fn inject_output(
        &mut self,
        _event: keyrx_core::runtime::event::KeyEvent,
    ) -> PlatformResult<()> {
        Ok(())
    }
    fn list_devices(&self) -> PlatformResult<Vec<DeviceInfo>> {
        Ok(Vec::new())
    }
    fn reconfigure_devices(&mut self, configs: &[DeviceConfig]) -> PlatformResult<()> {
        self.0.lock().unwrap().push(configs.len());
        Ok(())
    }
    fn shutdown(&mut self) -> PlatformResult<()> {
        Ok(())
    }
}

#[test]
fn broken_active_profile_at_startup_leaves_no_config_live_and_reports_why() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("profiles")).unwrap();
    fs::write(dir.path().join("profiles/bad.krx"), b"not a krx").unwrap();
    set_active(dir.path(), "bad");
    let calls = Arc::new(std::sync::Mutex::new(Vec::new()));

    let mut daemon = crate::daemon::Daemon::new(
        Box::new(RecordingPlatform(Arc::clone(&calls))),
        ConfigSource::ActiveProfile,
        dir.path().to_path_buf(),
    )
    .expect("a broken active profile is not fatal");

    // The platform was told "nothing is live" (zero blocks => grab nothing).
    assert_eq!(*calls.lock().unwrap(), vec![0]);
    let shared = daemon.shared_state();
    assert_eq!(shared.get_active_profile(), None);
    let error = shared.get_config_error().expect("config_error is reported");
    assert!(error.contains("active profile"), "{error}");
    assert!(daemon.config_path().is_none());

    // Fixing the profile and reloading brings it live and clears the error.
    write_profile(dir.path(), "bad", KeyCode::CapsLock, KeyCode::Escape);
    daemon.reload().expect("reload");
    assert_eq!(*calls.lock().unwrap(), vec![0, 1]);
    assert_eq!(shared.get_active_profile().as_deref(), Some("bad"));
    assert_eq!(shared.get_config_error(), None);
}

#[test]
fn no_active_profile_is_no_config_and_no_error() {
    let dir = TempDir::new().unwrap();
    let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
    let daemon = crate::daemon::Daemon::new(
        Box::new(RecordingPlatform(Arc::clone(&calls))),
        ConfigSource::ActiveProfile,
        dir.path().to_path_buf(),
    )
    .unwrap();
    assert_eq!(*calls.lock().unwrap(), vec![0]);
    assert_eq!(daemon.shared_state().get_config_error(), None);
}

#[test]
fn failed_reload_keeps_the_previous_config_and_reports_the_error() {
    let dir = TempDir::new().unwrap();
    write_profile(dir.path(), "a", KeyCode::CapsLock, KeyCode::Escape);
    set_active(dir.path(), "a");
    let mut live = LiveConfig::new(dir.path().to_path_buf());
    let shared = shared_state();
    let start = live.load(&ConfigSource::ActiveProfile).unwrap();
    let mut platform = noop_platform();
    apply_loaded(&mut platform, &mut live, &shared, start);
    assert_eq!(shared.get_config_error(), None);

    shared.request_activation("missing");
    assert!(reload_remapping(&mut platform, &mut live, &shared).is_err());
    assert_eq!(shared.get_active_profile().as_deref(), Some("a"));
    assert!(shared.get_config_error().is_some());

    assert!(reload_remapping(&mut platform, &mut live, &shared).is_ok());
    assert_eq!(shared.get_config_error(), None);
}

const RHAI: &str = "device_start(\"*\");\nmap(\"VK_A\", \"VK_B\");\ndevice_end();\n";

fn write_source(dir: &Path, name: &str, source: &str) -> PathBuf {
    let profiles = dir.join("profiles");
    fs::create_dir_all(&profiles).unwrap();
    let path = profiles.join(format!("{name}.rhai"));
    fs::write(&path, source).unwrap();
    path
}

fn live_for(dir: &Path, name: &str) -> LiveConfig {
    set_active(dir, name);
    LiveConfig::new(dir.to_path_buf())
}

#[test]
fn active_profile_without_krx_is_compiled_from_its_source() {
    let dir = TempDir::new().unwrap();
    write_source(dir.path(), "p", RHAI);
    let live = live_for(dir.path(), "p");

    let loaded = live.load(&ConfigSource::ActiveProfile).unwrap().unwrap();
    assert_eq!(
        first_mapping_output(&loaded),
        KeyMapping::simple(KeyCode::A, KeyCode::B)
    );
}

/// The upgrade path: a `.krx` from a release with another format version
/// must be rebuilt, not leave the daemon with no config live.
#[test]
fn active_profile_with_unreadable_krx_is_rebuilt_from_its_source() {
    let dir = TempDir::new().unwrap();
    write_source(dir.path(), "p", RHAI);
    let krx = write_profile(dir.path(), "p", KeyCode::C, KeyCode::D);
    let mut bytes = fs::read(&krx).unwrap();
    bytes[4..8].copy_from_slice(&99u32.to_le_bytes()); // future/past format version
    fs::write(&krx, bytes).unwrap();
    let live = live_for(dir.path(), "p");

    let loaded = live.load(&ConfigSource::ActiveProfile).unwrap().unwrap();
    assert_eq!(
        first_mapping_output(&loaded),
        KeyMapping::simple(KeyCode::A, KeyCode::B)
    );
}

#[test]
fn source_edited_after_compiling_wins_over_the_stale_krx() {
    let dir = TempDir::new().unwrap();
    let krx = write_profile(dir.path(), "p", KeyCode::C, KeyCode::D);
    let rhai = write_source(dir.path(), "p", RHAI);
    let later = fs::metadata(&krx).unwrap().modified().unwrap() + std::time::Duration::from_secs(5);
    fs::File::options()
        .write(true)
        .open(&rhai)
        .unwrap()
        .set_modified(later)
        .unwrap();
    let live = live_for(dir.path(), "p");

    let loaded = live.load(&ConfigSource::ActiveProfile).unwrap().unwrap();
    assert_eq!(
        first_mapping_output(&loaded),
        KeyMapping::simple(KeyCode::A, KeyCode::B)
    );
}

#[test]
fn broken_new_source_keeps_a_good_krx_but_not_a_bad_one() {
    let dir = TempDir::new().unwrap();
    let krx = write_profile(dir.path(), "p", KeyCode::C, KeyCode::D);
    let rhai = write_source(dir.path(), "p", "device_start(");
    let later = fs::metadata(&krx).unwrap().modified().unwrap() + std::time::Duration::from_secs(5);
    fs::File::options()
        .write(true)
        .open(&rhai)
        .unwrap()
        .set_modified(later)
        .unwrap();
    let live = live_for(dir.path(), "p");

    // The previous compile still works: keep remapping with it.
    let loaded = live.load(&ConfigSource::ActiveProfile).unwrap().unwrap();
    assert_eq!(
        first_mapping_output(&loaded),
        KeyMapping::simple(KeyCode::C, KeyCode::D)
    );

    // No usable krx and a source that does not compile: say why.
    fs::write(&krx, b"garbage").unwrap();
    let err = live
        .load(&ConfigSource::ActiveProfile)
        .unwrap_err()
        .to_string();
    assert!(err.contains("cannot be compiled"), "{err}");
}
