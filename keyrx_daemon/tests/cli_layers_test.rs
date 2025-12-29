//! Integration tests for `keyrx layers` CLI command.

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test config directory with a test profile.
fn setup_test_env() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().to_path_buf();

    // Create profiles directory
    let profiles_dir = config_dir.join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    // Create a test profile
    let test_profile = profiles_dir.join("test.rhai");
    fs::write(
        &test_profile,
        r#"// Test profile
device_start("*");

map("VK_A", "VK_B");

when_start("MD_00");
  map("VK_C", "VK_D");
when_end();

device_end();
"#,
    )
    .unwrap();

    (temp_dir, config_dir)
}

#[test]
fn test_list_layers() {
    let (_temp_dir, config_dir) = setup_test_env();
    std::env::set_var("HOME", config_dir.parent().unwrap());

    // Parse test profile
    use keyrx_daemon::config::rhai_generator::RhaiGenerator;
    let profile_path = config_dir.join("profiles/test.rhai");
    let gen = RhaiGenerator::load(&profile_path).unwrap();

    let layers = gen.list_layers();
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].0, "MD_00");
    assert_eq!(layers[0].1, 1); // 1 mapping
}

#[test]
fn test_create_layer() {
    let (_temp_dir, config_dir) = setup_test_env();
    std::env::set_var("HOME", config_dir.parent().unwrap());

    use keyrx_daemon::config::rhai_generator::{LayerMode, RhaiGenerator};
    let profile_path = config_dir.join("profiles/test.rhai");
    let mut gen = RhaiGenerator::load(&profile_path).unwrap();

    // Add a new layer
    gen.add_layer("MD_01", "Navigation", LayerMode::Single)
        .unwrap();
    gen.save(&profile_path).unwrap();

    // Reload and verify
    let gen = RhaiGenerator::load(&profile_path).unwrap();
    let layers = gen.list_layers();
    assert_eq!(layers.len(), 2);
    assert!(layers.iter().any(|(id, _)| id == "MD_01"));
}

#[test]
fn test_rename_layer() {
    let (_temp_dir, config_dir) = setup_test_env();
    std::env::set_var("HOME", config_dir.parent().unwrap());

    use keyrx_daemon::config::rhai_generator::RhaiGenerator;
    let profile_path = config_dir.join("profiles/test.rhai");
    let mut gen = RhaiGenerator::load(&profile_path).unwrap();

    // Rename layer
    gen.rename_layer("MD_00", "MD_NAV").unwrap();
    gen.save(&profile_path).unwrap();

    // Reload and verify
    let gen = RhaiGenerator::load(&profile_path).unwrap();
    let layers = gen.list_layers();
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].0, "MD_NAV");
}

#[test]
fn test_delete_layer() {
    let (_temp_dir, config_dir) = setup_test_env();
    std::env::set_var("HOME", config_dir.parent().unwrap());

    use keyrx_daemon::config::rhai_generator::RhaiGenerator;
    let profile_path = config_dir.join("profiles/test.rhai");
    let mut gen = RhaiGenerator::load(&profile_path).unwrap();

    // Delete layer
    gen.delete_layer("MD_00").unwrap();
    gen.save(&profile_path).unwrap();

    // Reload and verify
    let gen = RhaiGenerator::load(&profile_path).unwrap();
    let layers = gen.list_layers();
    assert_eq!(layers.len(), 0);
}

#[test]
fn test_show_layer() {
    let (_temp_dir, config_dir) = setup_test_env();
    std::env::set_var("HOME", config_dir.parent().unwrap());

    use keyrx_daemon::config::rhai_generator::RhaiGenerator;
    let profile_path = config_dir.join("profiles/test.rhai");
    let gen = RhaiGenerator::load(&profile_path).unwrap();

    // Get layer mappings
    let mappings = gen.get_layer_mappings("MD_00").unwrap();
    assert_eq!(mappings.len(), 1);
    assert!(mappings[0].contains("VK_C"));
    assert!(mappings[0].contains("VK_D"));
}

#[test]
fn test_show_base_layer() {
    let (_temp_dir, config_dir) = setup_test_env();
    std::env::set_var("HOME", config_dir.parent().unwrap());

    use keyrx_daemon::config::rhai_generator::RhaiGenerator;
    let profile_path = config_dir.join("profiles/test.rhai");
    let gen = RhaiGenerator::load(&profile_path).unwrap();

    // Get base layer mappings
    let mappings = gen.get_layer_mappings("base").unwrap();
    assert_eq!(mappings.len(), 1);
    assert!(mappings[0].contains("VK_A"));
    assert!(mappings[0].contains("VK_B"));
}

#[test]
fn test_layer_not_found() {
    let (_temp_dir, config_dir) = setup_test_env();
    std::env::set_var("HOME", config_dir.parent().unwrap());

    use keyrx_daemon::config::rhai_generator::RhaiGenerator;
    let profile_path = config_dir.join("profiles/test.rhai");
    let gen = RhaiGenerator::load(&profile_path).unwrap();

    // Try to get non-existent layer
    let result = gen.get_layer_mappings("MD_99");
    assert!(result.is_err());
}

#[test]
fn test_duplicate_layer_creation() {
    let (_temp_dir, config_dir) = setup_test_env();
    std::env::set_var("HOME", config_dir.parent().unwrap());

    use keyrx_daemon::config::rhai_generator::{LayerMode, RhaiGenerator};
    let profile_path = config_dir.join("profiles/test.rhai");
    let mut gen = RhaiGenerator::load(&profile_path).unwrap();

    // Try to add duplicate layer
    let result = gen.add_layer("MD_00", "Duplicate", LayerMode::Single);
    assert!(result.is_err());
}

#[test]
fn test_invalid_layer_id() {
    let (_temp_dir, config_dir) = setup_test_env();
    std::env::set_var("HOME", config_dir.parent().unwrap());

    use keyrx_daemon::config::rhai_generator::{LayerMode, RhaiGenerator};
    let profile_path = config_dir.join("profiles/test.rhai");
    let mut gen = RhaiGenerator::load(&profile_path).unwrap();

    // Try to add layer with invalid ID (must start with MD_)
    let result = gen.add_layer("INVALID", "Test", LayerMode::Single);
    assert!(result.is_err());
}
