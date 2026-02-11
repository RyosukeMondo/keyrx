//! End-to-end workflow integration tests
//!
//! These tests verify the complete workflow from writing Rhai scripts
//! through compilation, verification, and parsing:
//! - write .rhai → compile to .krx → verify .krx → parse back to ConfigRoot
//! - All mapping types in one configuration
//! - Import resolution in complete workflow
//! - Deterministic compilation (same input → same output)
//! - Error scenarios (syntax errors, circular imports, etc.)

use keyrx_compiler::cli::{compile, verify};
use keyrx_compiler::parser::Parser;
use keyrx_compiler::serialize::{deserialize, serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a temporary Rhai file with content.
fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let file_path = dir.path().join(name);
    let mut file = fs::File::create(&file_path).expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write temp file");
    file_path
}

// ============================================================================
// Complete Workflow Tests
// ============================================================================

#[test]
fn test_complete_workflow_simple_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Step 1: Write .rhai configuration
    let config_content = r#"
device_start("Test Keyboard");
map("VK_A", "VK_B");
map("VK_C", "VK_D");
device_end();
"#;
    let rhai_path = create_temp_file(&temp_dir, "config.rhai", config_content);
    let krx_path = temp_dir.path().join("config.krx");

    // Step 2: Compile to .krx
    compile::handle_compile(&rhai_path, &krx_path).expect("Compilation should succeed");

    // Step 3: Verify .krx file exists and has content
    assert!(krx_path.exists(), ".krx file should exist");
    let krx_size = fs::metadata(&krx_path).unwrap().len();
    assert!(krx_size > 0, ".krx file should not be empty");

    // Step 4: Verify .krx integrity
    verify::handle_verify(&krx_path).expect("Verification should succeed");

    // Step 5: Deserialize and validate structure
    let krx_bytes = fs::read(&krx_path).expect("Failed to read .krx file");
    let config = deserialize(&krx_bytes).expect("Deserialization should succeed");

    assert_eq!(config.devices.len(), 1, "Should have 1 device");
    assert_eq!(
        config.devices[0].mappings.len(),
        2,
        "Should have 2 mappings"
    );
}

#[test]
fn test_complete_workflow_all_mapping_types() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Configuration with all mapping types
    let config_content = r#"
device_start("Comprehensive Test");

// Simple mapping (VK_ → VK_)
map("VK_A", "VK_B");

// Modifier mapping (VK_ → MD_)
map("VK_CapsLock", "MD_00");

// Lock mapping (VK_ → LK_)
map("VK_ScrollLock", "LK_01");

// TapHold mapping
tap_hold("VK_Space", "VK_Space", "MD_01", 200);

// ModifiedOutput mapping
map("VK_1", with_shift("VK_2"));

// Conditional mapping
when_start("MD_00");
map("VK_H", "VK_Left");
map("VK_J", "VK_Down");
map("VK_K", "VK_Up");
map("VK_L", "VK_Right");
when_end();

device_end();
"#;
    let rhai_path = create_temp_file(&temp_dir, "all_types.rhai", config_content);
    let krx_path = temp_dir.path().join("all_types.krx");

    // Compile
    compile::handle_compile(&rhai_path, &krx_path).expect("Compilation should succeed");

    // Verify
    verify::handle_verify(&krx_path).expect("Verification should succeed");

    // Deserialize and validate
    let krx_bytes = fs::read(&krx_path).expect("Failed to read .krx file");
    let config = deserialize(&krx_bytes).expect("Deserialization should succeed");

    assert_eq!(config.devices.len(), 1, "Should have 1 device");
    assert_eq!(
        config.devices[0].mappings.len(),
        6,
        "Should have 6 mappings (Simple, Modifier, Lock, TapHold, ModifiedOutput, Conditional)"
    );
}

// NOTE: Import functionality is not yet integrated into the CLI compile workflow.
// Import resolution is tested separately in import_tests.rs.
// This test is commented out until imports are integrated into the parser.
//
// #[test]
// fn test_complete_workflow_with_imports() { ... }

#[test]
fn test_deterministic_compilation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create configuration
    let config_content = r#"
device_start("Test Device");
map("VK_A", "VK_B");
map("VK_C", "VK_D");
map("VK_E", "VK_F");
device_end();
"#;
    let rhai_path = create_temp_file(&temp_dir, "config.rhai", config_content);

    // Compile first time
    let krx_path_1 = temp_dir.path().join("config1.krx");
    compile::handle_compile(&rhai_path, &krx_path_1).expect("First compilation should succeed");
    let bytes_1 = fs::read(&krx_path_1).expect("Failed to read first .krx");

    // Compile second time
    let krx_path_2 = temp_dir.path().join("config2.krx");
    compile::handle_compile(&rhai_path, &krx_path_2).expect("Second compilation should succeed");
    let bytes_2 = fs::read(&krx_path_2).expect("Failed to read second .krx");

    // Compare bytes (should be identical)
    assert_eq!(
        bytes_1, bytes_2,
        "Same input should produce identical output (deterministic compilation)"
    );
}

// NOTE: Import functionality is not yet integrated into the CLI compile workflow.
// Import resolution is tested separately in import_tests.rs.
// This test is commented out until imports are integrated into the parser.
//
// #[test]
// fn test_workflow_with_multilevel_imports() { ... }

#[test]
fn test_workflow_with_multiple_devices() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Configuration with multiple devices
    let config_content = r#"
device_start("Laptop Keyboard");
map("VK_A", "VK_B");
device_end();

device_start("External Keyboard");
map("VK_C", "VK_D");
map("VK_E", "VK_F");
device_end();

device_start("Numpad");
map("VK_Num1", "VK_Num2");
device_end();
"#;
    let rhai_path = create_temp_file(&temp_dir, "multi_device.rhai", config_content);
    let krx_path = temp_dir.path().join("multi_device.krx");

    // Compile
    compile::handle_compile(&rhai_path, &krx_path).expect("Compilation should succeed");

    // Verify
    verify::handle_verify(&krx_path).expect("Verification should succeed");

    // Deserialize and validate
    let krx_bytes = fs::read(&krx_path).expect("Failed to read .krx file");
    let config = deserialize(&krx_bytes).expect("Deserialization should succeed");

    assert_eq!(config.devices.len(), 3, "Should have 3 devices");

    let laptop = config
        .devices
        .iter()
        .find(|d| d.identifier.pattern == "Laptop Keyboard")
        .expect("Should have Laptop Keyboard");
    assert_eq!(laptop.mappings.len(), 1);

    let external = config
        .devices
        .iter()
        .find(|d| d.identifier.pattern == "External Keyboard")
        .expect("Should have External Keyboard");
    assert_eq!(external.mappings.len(), 2);

    let numpad = config
        .devices
        .iter()
        .find(|d| d.identifier.pattern == "Numpad")
        .expect("Should have Numpad");
    assert_eq!(numpad.mappings.len(), 1);
}

// ============================================================================
// Error Scenario Tests
// ============================================================================

#[test]
fn test_workflow_syntax_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Invalid syntax (missing semicolon)
    let config_content = r#"
device_start("Test")
map("VK_A", "VK_B")
device_end();
"#;
    let rhai_path = create_temp_file(&temp_dir, "invalid.rhai", config_content);
    let krx_path = temp_dir.path().join("invalid.krx");

    // Compilation should fail
    let result = compile::handle_compile(&rhai_path, &krx_path);
    assert!(result.is_err(), "Compilation with syntax error should fail");
}

#[test]
fn test_workflow_invalid_prefix() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Missing prefix on output key (this is invalid)
    let config_content = r#"
device_start("Test");
map("VK_A", "B");
device_end();
"#;
    let rhai_path = create_temp_file(&temp_dir, "invalid_prefix.rhai", config_content);
    let krx_path = temp_dir.path().join("invalid_prefix.krx");

    // Compilation should fail (output must have VK_/MD_/LK_ prefix)
    let result = compile::handle_compile(&rhai_path, &krx_path);
    assert!(
        result.is_err(),
        "Compilation with invalid output prefix should fail"
    );
}

// NOTE: Import functionality is not yet integrated into the CLI compile workflow.
// Circular import detection is tested in import_tests.rs with the ImportResolver.
// This test is commented out until imports are integrated into the parser.
//
// #[test]
// fn test_workflow_circular_import_error() { ... }

// NOTE: Import functionality is not yet integrated into the CLI compile workflow.
// Import not found errors are tested in import_tests.rs with the ImportResolver.
// This test is commented out until imports are integrated into the parser.
//
// #[test]
// fn test_workflow_import_not_found() { ... }

#[test]
fn test_workflow_verify_corrupted_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create valid .krx file first
    let config_content = r#"
device_start("Test");
map("VK_A", "VK_B");
device_end();
"#;
    let rhai_path = create_temp_file(&temp_dir, "config.rhai", config_content);
    let krx_path = temp_dir.path().join("config.krx");

    compile::handle_compile(&rhai_path, &krx_path).expect("Compilation should succeed");

    // Corrupt the .krx file by modifying bytes
    let mut krx_bytes = fs::read(&krx_path).expect("Failed to read .krx");
    if krx_bytes.len() > 50 {
        // Corrupt data section (skip magic and hash)
        krx_bytes[50] = krx_bytes[50].wrapping_add(1);
        fs::write(&krx_path, &krx_bytes).expect("Failed to write corrupted file");
    }

    // Verification should fail
    let result = verify::handle_verify(&krx_path);
    assert!(
        result.is_err(),
        "Verification of corrupted file should fail"
    );
}

#[test]
fn test_workflow_parse_round_trip() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create configuration
    let config_content = r#"
device_start("Test Device");
map("VK_A", "VK_B");
map("VK_CapsLock", "MD_00");
tap_hold("VK_Space", "VK_Space", "MD_01", 200);
device_end();
"#;
    let rhai_path = create_temp_file(&temp_dir, "config.rhai", config_content);

    // Parse to ConfigRoot
    let mut parser = Parser::new();
    let config1 = parser
        .parse_script(&rhai_path)
        .expect("First parse should succeed");

    // Serialize to .krx
    let krx_bytes = serialize(&config1).expect("Serialization should succeed");

    // Deserialize back to ConfigRoot
    let config2 = deserialize(&krx_bytes).expect("Deserialization should succeed");

    // Compare structures
    assert_eq!(config1.devices.len(), config2.devices.len());
    // Note: Can't directly compare config1.version (Version) with config2.version (ArchivedVersion)
    // This is expected with rkyv serialization

    for (dev1, dev2) in config1.devices.iter().zip(config2.devices.iter()) {
        assert_eq!(dev1.identifier.pattern, dev2.identifier.pattern);
        assert_eq!(dev1.mappings.len(), dev2.mappings.len());
    }
}

// ============================================================================
// Complex Scenario Tests
// ============================================================================

#[test]
fn test_workflow_complex_realistic_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Realistic configuration combining multiple features
    let config_content = r#"
// Main keyboard configuration
device_start("Default Keyboard");

// Escape on tap, Ctrl on hold
tap_hold("VK_CapsLock", "VK_Escape", "MD_00", 200);

// Space on tap, navigation layer on hold
tap_hold("VK_Space", "VK_Space", "MD_01", 200);

// Navigation layer (MD_01 + HJKL)
when_start("MD_01");
map("VK_H", "VK_Left");
map("VK_J", "VK_Down");
map("VK_K", "VK_Up");
map("VK_L", "VK_Right");
map("VK_U", "VK_PageUp");
map("VK_D", "VK_PageDown");
when_end();

// Shift layer (MD_00 + numbers)
when_start("MD_00");
map("VK_Num1", with_shift("VK_Num1"));
map("VK_Num2", with_shift("VK_Num2"));
when_end();

device_end();
"#;
    let rhai_path = create_temp_file(&temp_dir, "realistic.rhai", config_content);
    let krx_path = temp_dir.path().join("realistic.krx");

    // Compile
    compile::handle_compile(&rhai_path, &krx_path)
        .expect("Complex config compilation should succeed");

    // Verify
    verify::handle_verify(&krx_path).expect("Verification should succeed");

    // Deserialize and validate
    let krx_bytes = fs::read(&krx_path).expect("Failed to read .krx file");
    let config = deserialize(&krx_bytes).expect("Deserialization should succeed");

    assert_eq!(config.devices.len(), 1);
    assert_eq!(config.devices[0].identifier.pattern, "Default Keyboard");

    // Should have: 2 tap_hold + 2 conditional blocks (6 mappings in first, 2 in second)
    assert_eq!(
        config.devices[0].mappings.len(),
        4,
        "Should have 2 TapHold + 2 Conditional mappings"
    );
}

#[test]
fn test_workflow_empty_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Empty configuration (no devices)
    let config_content = r#"
// Empty configuration file
// This is valid - no devices defined
"#;
    let rhai_path = create_temp_file(&temp_dir, "empty.rhai", config_content);
    let krx_path = temp_dir.path().join("empty.krx");

    // Compile (should succeed with empty config)
    compile::handle_compile(&rhai_path, &krx_path)
        .expect("Empty config compilation should succeed");

    // Verify
    verify::handle_verify(&krx_path).expect("Verification should succeed");

    // Deserialize
    let krx_bytes = fs::read(&krx_path).expect("Failed to read .krx file");
    let config = deserialize(&krx_bytes).expect("Deserialization should succeed");

    assert_eq!(
        config.devices.len(),
        0,
        "Empty config should have 0 devices"
    );
}

#[test]
fn test_workflow_large_config_many_mappings() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Generate large configuration with many mappings
    let mut config_content = String::from("device_start(\"Test Device\");\n");

    // Add 100 simple mappings
    for i in 0..100 {
        config_content.push_str(&format!(
            "map(\"VK_F{}\", \"VK_F{}\");\n",
            (i % 24) + 1,
            ((i + 1) % 24) + 1
        ));
    }

    config_content.push_str("device_end();\n");

    let rhai_path = create_temp_file(&temp_dir, "large.rhai", &config_content);
    let krx_path = temp_dir.path().join("large.krx");

    // Compile
    compile::handle_compile(&rhai_path, &krx_path)
        .expect("Large config compilation should succeed");

    // Verify
    verify::handle_verify(&krx_path).expect("Verification should succeed");

    // Deserialize
    let krx_bytes = fs::read(&krx_path).expect("Failed to read .krx file");
    let config = deserialize(&krx_bytes).expect("Deserialization should succeed");

    assert_eq!(config.devices.len(), 1);
    assert_eq!(
        config.devices[0].mappings.len(),
        100,
        "Should have 100 mappings"
    );
}
