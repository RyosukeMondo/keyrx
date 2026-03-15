//! Integration tests for compile_file() public API.
//!
//! Verifies that the main public API produces valid .krx binaries,
//! handles errors correctly, and outputs loadable results.

use keyrx_compiler::compile_file;
use keyrx_compiler::serialize::deserialize;
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

#[test]
fn test_compile_file_simple() {
    let temp_dir = TempDir::new().unwrap();
    let rhai_path = create_temp_file(
        &temp_dir,
        "simple.rhai",
        r#"
device_start("Test");
map("VK_A", "VK_B");
map("CapsLock", "VK_Escape");
device_end();
"#,
    );
    let krx_path = temp_dir.path().join("simple.krx");

    compile_file(&rhai_path, &krx_path).expect("compile_file should succeed");

    assert!(krx_path.exists(), ".krx file should be created");
    let bytes = fs::read(&krx_path).unwrap();
    assert!(bytes.len() > 48, ".krx should be larger than header");
}

#[test]
fn test_compile_file_all_mapping_types() {
    let temp_dir = TempDir::new().unwrap();
    let rhai_path = create_temp_file(
        &temp_dir,
        "all_types.rhai",
        r#"
device_start("All Types");
// Simple remap
map("VK_A", "VK_B");
// Modifier activation
map("CapsLock", "MD_00");
// Lock toggle
map("ScrollLock", "LK_00");
// Tap-hold
tap_hold("Space", "VK_Space", "MD_01", 200);
// Modified output
map("VK_Z", with_ctrl("VK_Z"));
device_end();
"#,
    );
    let krx_path = temp_dir.path().join("all_types.krx");

    compile_file(&rhai_path, &krx_path).expect("compile_file should succeed");

    // Verify all 5 mapping types survive compilation
    let bytes = fs::read(&krx_path).unwrap();
    let config = deserialize(&bytes).expect("deserialize should succeed");
    assert_eq!(config.devices.len(), 1);
    assert_eq!(
        config.devices[0].mappings.len(),
        5,
        "All 5 mapping types should be present"
    );
}

#[test]
fn test_compile_file_syntax_error() {
    let temp_dir = TempDir::new().unwrap();
    let rhai_path = create_temp_file(
        &temp_dir,
        "bad_syntax.rhai",
        r#"
device_start("Test");
map("VK_A", "INVALID_KEY");
device_end();
"#,
    );
    let krx_path = temp_dir.path().join("bad_syntax.krx");

    let result = compile_file(&rhai_path, &krx_path);
    assert!(result.is_err(), "Should fail on invalid key name");
}

#[test]
fn test_compile_file_missing_input() {
    let temp_dir = TempDir::new().unwrap();
    let rhai_path = temp_dir.path().join("nonexistent.rhai");
    let krx_path = temp_dir.path().join("output.krx");

    let result = compile_file(&rhai_path, &krx_path);
    assert!(result.is_err(), "Should fail on missing input file");
}

#[test]
fn test_compile_file_output_is_loadable() {
    let temp_dir = TempDir::new().unwrap();
    let rhai_path = create_temp_file(
        &temp_dir,
        "loadable.rhai",
        r#"
device_start("Loadable Test");
map("VK_A", "VK_B");
map("CapsLock", "MD_00");
when_start("MD_00");
map("VK_H", "VK_Left");
map("VK_J", "VK_Down");
when_end();
device_end();
"#,
    );
    let krx_path = temp_dir.path().join("loadable.krx");

    // Compile
    compile_file(&rhai_path, &krx_path).expect("compile_file should succeed");

    // Read and deserialize
    let bytes = fs::read(&krx_path).unwrap();
    let config = deserialize(&bytes).expect("Output .krx must be deserializable");

    // Verify structure
    assert_eq!(config.devices.len(), 1);
    let device = &config.devices[0];
    // 2 base mappings (A→B, CapsLock→MD_00) + 1 conditional (with 2 inner)
    assert_eq!(device.mappings.len(), 3);
}
