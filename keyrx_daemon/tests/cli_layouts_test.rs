//! Integration tests for `keyrx layouts` CLI command.

use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test layouts directory.
fn setup_test_env() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let layouts_dir = temp_dir.path().join("layouts");
    fs::create_dir_all(&layouts_dir).unwrap();

    (temp_dir, layouts_dir)
}

/// Create a valid KLE JSON layout.
fn create_test_kle() -> serde_json::Value {
    json!([
        [{"w": 1}, "Esc", "1", "2", "3", "4", "5"],
        ["Tab", "Q", "W", "E", "R", "T"],
        ["Caps", "A", "S", "D", "F", "G"],
        ["Shift", "Z", "X", "C", "V", "B"],
        ["Ctrl", "Win", "Alt", {"w": 2}, "Space"]
    ])
}

#[test]
fn test_layout_manager_list() {
    let (_temp_dir, layouts_dir) = setup_test_env();

    use keyrx_daemon::config::layout_manager::LayoutManager;
    let manager = LayoutManager::new(layouts_dir).unwrap();

    let layouts = manager.list();
    // Should have 5 builtin layouts
    assert_eq!(manager.builtin_count(), 5);
    assert_eq!(manager.custom_count(), 0);
    assert!(layouts.len() >= 5);

    // Check that builtin layouts are present
    let names: Vec<&str> = layouts.iter().map(|l| l.name.as_str()).collect();
    assert!(names.contains(&"ansi_104"));
    assert!(names.contains(&"iso_105"));
    assert!(names.contains(&"jis_109"));
    assert!(names.contains(&"hhkb"));
    assert!(names.contains(&"numpad"));
}

#[test]
fn test_import_custom_layout() {
    let (temp_dir, layouts_dir) = setup_test_env();

    use keyrx_daemon::config::layout_manager::LayoutManager;
    let mut manager = LayoutManager::new(layouts_dir.clone()).unwrap();

    // Create a test layout file
    let test_layout_path = temp_dir.path().join("my_layout.json");
    let kle = create_test_kle();
    fs::write(
        &test_layout_path,
        serde_json::to_string_pretty(&kle).unwrap(),
    )
    .unwrap();

    // Import the layout
    let result = manager.import(&test_layout_path, "my_custom_layout");
    assert!(result.is_ok());

    // Verify it was imported
    assert_eq!(manager.custom_count(), 1);
    let layout = manager.get("my_custom_layout");
    assert!(layout.is_some());
    assert_eq!(layout.unwrap().name, "my_custom_layout");

    // Verify the file was created in layouts_dir
    let saved_path = layouts_dir.join("my_custom_layout.json");
    assert!(saved_path.exists());
}

#[test]
fn test_import_invalid_kle() {
    let (temp_dir, layouts_dir) = setup_test_env();

    use keyrx_daemon::config::layout_manager::LayoutManager;
    let mut manager = LayoutManager::new(layouts_dir).unwrap();

    // Create an invalid layout file (not an array)
    let test_layout_path = temp_dir.path().join("invalid.json");
    let invalid_kle = json!({"foo": "bar"});
    fs::write(
        &test_layout_path,
        serde_json::to_string_pretty(&invalid_kle).unwrap(),
    )
    .unwrap();

    // Import should fail
    let result = manager.import(&test_layout_path, "invalid_layout");
    assert!(result.is_err());
    assert_eq!(manager.custom_count(), 0);
}

#[test]
fn test_delete_custom_layout() {
    let (temp_dir, layouts_dir) = setup_test_env();

    use keyrx_daemon::config::layout_manager::LayoutManager;
    let mut manager = LayoutManager::new(layouts_dir.clone()).unwrap();

    // Import a layout first
    let test_layout_path = temp_dir.path().join("my_layout.json");
    let kle = create_test_kle();
    fs::write(
        &test_layout_path,
        serde_json::to_string_pretty(&kle).unwrap(),
    )
    .unwrap();
    manager.import(&test_layout_path, "test_layout").unwrap();
    assert_eq!(manager.custom_count(), 1);

    // Delete it
    let result = manager.delete("test_layout");
    assert!(result.is_ok());
    assert_eq!(manager.custom_count(), 0);

    // Verify the file was removed
    let saved_path = layouts_dir.join("test_layout.json");
    assert!(!saved_path.exists());
}

#[test]
fn test_cannot_delete_builtin() {
    let (_temp_dir, layouts_dir) = setup_test_env();

    use keyrx_daemon::config::layout_manager::LayoutManager;
    let mut manager = LayoutManager::new(layouts_dir).unwrap();

    // Try to delete a builtin layout
    let result = manager.delete("ansi_104");
    assert!(result.is_err());
}

#[test]
fn test_cannot_overwrite_builtin() {
    let (temp_dir, layouts_dir) = setup_test_env();

    use keyrx_daemon::config::layout_manager::LayoutManager;
    let mut manager = LayoutManager::new(layouts_dir).unwrap();

    // Create a test layout file
    let test_layout_path = temp_dir.path().join("custom.json");
    let kle = create_test_kle();
    fs::write(
        &test_layout_path,
        serde_json::to_string_pretty(&kle).unwrap(),
    )
    .unwrap();

    // Try to import with a builtin name
    let result = manager.import(&test_layout_path, "ansi_104");
    assert!(result.is_err());
}

#[test]
fn test_show_layout() {
    let (_temp_dir, layouts_dir) = setup_test_env();

    use keyrx_daemon::config::layout_manager::LayoutManager;
    let manager = LayoutManager::new(layouts_dir).unwrap();

    // Get a builtin layout
    let layout = manager.get("ansi_104");
    assert!(layout.is_some());

    let layout = layout.unwrap();
    assert_eq!(layout.name, "ansi_104");

    // Verify it has valid KLE JSON
    assert!(layout.kle_json.is_array());
}

#[test]
fn test_layout_name_validation() {
    use keyrx_daemon::config::layout_manager::LayoutManager;

    // Valid names
    let valid_names = vec!["my_layout", "layout-1", "LAYOUT_123", "abc", "my-custom"];
    for name in valid_names {
        let result = LayoutManager::validate_kle(&create_test_kle());
        assert!(result.is_ok(), "Name '{}' should be valid", name);
    }

    // Test too long name
    let (temp_dir, layouts_dir) = setup_test_env();
    let mut manager = LayoutManager::new(layouts_dir).unwrap();

    let test_layout_path = temp_dir.path().join("test.json");
    let kle = create_test_kle();
    fs::write(
        &test_layout_path,
        serde_json::to_string_pretty(&kle).unwrap(),
    )
    .unwrap();

    // Try to import with a too-long name (>32 chars)
    let long_name = "a".repeat(33);
    let result = manager.import(&test_layout_path, &long_name);
    assert!(result.is_err());
}

#[test]
fn test_max_custom_layouts() {
    use keyrx_daemon::config::layout_manager::LayoutManager;

    let (temp_dir, layouts_dir) = setup_test_env();
    let mut manager = LayoutManager::new(layouts_dir).unwrap();

    let kle = create_test_kle();

    // Import 50 layouts (the maximum)
    for i in 0..50 {
        let test_layout_path = temp_dir.path().join(format!("layout{}.json", i));
        fs::write(
            &test_layout_path,
            serde_json::to_string_pretty(&kle).unwrap(),
        )
        .unwrap();

        let result = manager.import(&test_layout_path, &format!("layout{}", i));
        assert!(result.is_ok(), "Should be able to import layout {}", i);
    }

    assert_eq!(manager.custom_count(), 50);

    // Try to import one more (should fail)
    let test_layout_path = temp_dir.path().join("layout51.json");
    fs::write(
        &test_layout_path,
        serde_json::to_string_pretty(&kle).unwrap(),
    )
    .unwrap();

    let result = manager.import(&test_layout_path, "layout51");
    assert!(result.is_err());
}
