use super::*;
use std::fs;
use tempfile::TempDir;

fn create_test_device(id: &str, name: &str) -> DeviceEntry {
    DeviceEntry::new(
        id.to_string(),
        name.to_string(),
        None,
        None,
        current_timestamp(),
    )
}

#[test]
fn test_new_registry() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("registry.json");
    let registry = DeviceRegistry::new(path.clone());

    assert_eq!(registry.list().len(), 0);
    assert_eq!(registry.path, path);
}

#[test]
fn test_register_device() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("registry.json");
    let mut registry = DeviceRegistry::new(path);

    let device = create_test_device("dev1", "Test Device");
    registry.register(device).unwrap();

    assert_eq!(registry.list().len(), 1);
    assert!(registry.get("dev1").is_some());
}

#[test]
fn test_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("registry.json");

    let mut registry = DeviceRegistry::new(path.clone());
    let device = create_test_device("dev1", "Test Device");
    registry.register(device).unwrap();
    registry.save().unwrap();

    let loaded = DeviceRegistry::load(&path).unwrap();
    assert_eq!(loaded.list().len(), 1);

    let loaded_device = loaded.get("dev1").unwrap();
    assert_eq!(loaded_device.name, "Test Device");
}

#[test]
fn test_atomic_write_no_corruption() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("registry.json");

    let mut registry = DeviceRegistry::new(path.clone());
    let device = create_test_device("dev1", "Device1");
    registry.register(device).unwrap();
    registry.save().unwrap();

    let tmp_path = path.with_extension("tmp");
    assert!(
        !tmp_path.exists(),
        "Temp file should be removed after atomic rename"
    );
    assert!(path.exists(), "Registry file should exist");
}

#[test]
fn test_rename_device() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("registry.json");
    let mut registry = DeviceRegistry::new(path);

    let device = create_test_device("dev1", "Old Name");
    registry.register(device).unwrap();

    registry.rename("dev1", "New Name").unwrap();
    assert_eq!(registry.get("dev1").unwrap().name, "New Name");
}

#[test]
fn test_rename_nonexistent_device() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("registry.json");
    let mut registry = DeviceRegistry::new(path);

    let result = registry.rename("nonexistent", "New Name");
    assert!(matches!(
        result,
        Err(DeviceValidationError::DeviceNotFound(_))
    ));
}

#[test]
fn test_set_layout() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("registry.json");
    let mut registry = DeviceRegistry::new(path);

    let device = create_test_device("dev1", "Device");
    registry.register(device).unwrap();

    registry.set_layout("dev1", "ansi_104").unwrap();
    assert_eq!(
        registry.get("dev1").unwrap().layout,
        Some("ansi_104".to_string())
    );
}

#[test]
fn test_forget_device() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("registry.json");
    let mut registry = DeviceRegistry::new(path);

    let device = create_test_device("dev1", "Device");
    registry.register(device).unwrap();

    let removed = registry.forget("dev1").unwrap();
    assert_eq!(removed.id, "dev1");
    assert!(registry.get("dev1").is_none());
}

#[test]
fn test_update_last_seen() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("registry.json");
    let mut registry = DeviceRegistry::new(path);

    let mut device = create_test_device("dev1", "Device");
    device.last_seen = 1000;
    registry.register(device).unwrap();

    registry.update_last_seen("dev1").unwrap();

    let updated = registry.get("dev1").unwrap();
    assert!(updated.last_seen > 1000);
}

#[test]
fn test_validate_device_name_too_long() {
    let long_name = "a".repeat(65);
    let result = validate_device_name(&long_name);
    assert!(matches!(result, Err(DeviceValidationError::InvalidName(_))));
}

#[test]
fn test_validate_device_name_invalid_chars() {
    let result = validate_device_name("Device@#$");
    assert!(matches!(result, Err(DeviceValidationError::InvalidName(_))));
}

#[test]
fn test_validate_device_name_valid() {
    assert!(validate_device_name("My Device-123_test").is_ok());
}

#[test]
fn test_validate_device_id_too_long() {
    let long_id = "a".repeat(257);
    let result = validate_device_id(&long_id);
    assert!(matches!(
        result,
        Err(DeviceValidationError::InvalidDeviceId(_))
    ));
}

#[test]
fn test_validate_layout_name_too_long() {
    let long_layout = "a".repeat(33);
    let result = validate_layout_name(&long_layout);
    assert!(matches!(result, Err(DeviceValidationError::InvalidName(_))));
}

#[test]
fn test_corrupted_registry_file_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("registry.json");

    fs::write(&path, "{ invalid json ").unwrap();

    // Load should recover by creating empty registry
    let registry = DeviceRegistry::load(&path).unwrap();
    assert_eq!(registry.list().len(), 0);

    // Verify the file was fixed
    let contents = fs::read_to_string(&path).unwrap();
    assert!(contents.contains("{}") || contents.contains("{ }"));
}

#[test]
fn test_load_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("nonexistent.json");

    let registry = DeviceRegistry::load(&path).unwrap();
    assert_eq!(registry.list().len(), 0);
}

#[test]
fn test_list_devices() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("registry.json");
    let mut registry = DeviceRegistry::new(path);

    registry
        .register(create_test_device("dev1", "Device 1"))
        .unwrap();
    registry
        .register(create_test_device("dev2", "Device 2"))
        .unwrap();

    let list = registry.list();
    assert_eq!(list.len(), 2);
}

#[test]
fn test_empty_device_name() {
    let result = validate_device_name("");
    assert!(matches!(result, Err(DeviceValidationError::InvalidName(_))));
}

#[test]
fn test_empty_device_id() {
    let result = validate_device_id("");
    assert!(matches!(
        result,
        Err(DeviceValidationError::InvalidDeviceId(_))
    ));
}

#[test]
#[cfg(unix)] // Permission-based tests only work on Unix
fn test_write_protected_directory() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    // Test that save returns proper error when directory is not writable
    let temp_dir = TempDir::new().unwrap();
    let protected_dir = temp_dir.path().join("protected");
    fs::create_dir(&protected_dir).unwrap();

    // Make directory read-only (remove write permission)
    let mut perms = fs::metadata(&protected_dir).unwrap().permissions();
    perms.set_mode(0o444); // r--r--r--
    fs::set_permissions(&protected_dir, perms).unwrap();

    let path = protected_dir.join("registry.json");
    let registry = DeviceRegistry::new(path);
    let result = registry.save();

    // Cleanup: restore permissions before TempDir cleanup
    let mut perms = fs::metadata(&protected_dir).unwrap().permissions();
    perms.set_mode(0o755);
    let _ = fs::set_permissions(&protected_dir, perms);

    assert!(matches!(result, Err(RegistryError::IOError(_))));
}

#[test]
fn test_creates_parent_directory() {
    // Test that save creates parent directory if it doesn't exist
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir
        .path()
        .join("nonexistent_dir")
        .join("registry.json");

    let registry = DeviceRegistry::new(path.clone());
    let result = registry.save();

    assert!(result.is_ok());
    assert!(path.exists());
    assert!(path.parent().unwrap().exists());
}

#[test]
fn test_recovery_creates_valid_empty_registry() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("registry.json");

    // Write corrupted data
    fs::write(&path, "[this is not a hashmap]").unwrap();

    // Load should recover and create empty registry
    let registry = DeviceRegistry::load(&path).unwrap();
    assert_eq!(registry.list().len(), 0);

    // Verify we can add devices to the recovered registry
    let mut registry = registry;
    let device = create_test_device("dev1", "Test Device");
    registry.register(device).unwrap();
    assert_eq!(registry.list().len(), 1);
}

#[test]
fn test_ensure_registered_sanitizes_and_keeps_existing() {
    let dir = tempfile::tempdir().unwrap();
    let mut registry = DeviceRegistry::load(&dir.path().join("devices.json")).unwrap();
    let long = format!("Sunshine (libvirtualhid) X-Box {}", "x".repeat(80));
    registry
        .ensure_registered("path-/dev/input/event26", &long)
        .unwrap();
    let entry = registry.get("path-/dev/input/event26").unwrap();
    assert!(entry.name.starts_with("Sunshine -libvirtualhid- X-Box"));
    assert_eq!(entry.name.chars().count(), 64);

    registry.rename("path-/dev/input/event26", "Pad").unwrap();
    registry
        .ensure_registered("path-/dev/input/event26", "Other")
        .unwrap();
    assert_eq!(registry.get("path-/dev/input/event26").unwrap().name, "Pad");
}
