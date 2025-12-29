//! Integration tests for the keyrx_compiler CLI
//!
//! These tests verify the end-to-end functionality of all CLI commands:
//! - compile: Rhai → .krx file
//! - verify: Validate .krx files
//! - hash: Extract SHA256 hash
//! - parse: Parse and output JSON

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a temporary directory with test files
fn setup_test_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Helper to get a command for the keyrx_compiler binary
#[allow(deprecated)]
fn get_binary() -> Command {
    Command::cargo_bin("keyrx_compiler").expect("Failed to find keyrx_compiler binary")
}

/// Helper to create a simple valid Rhai config
fn create_simple_rhai_config(dir: &TempDir, filename: &str) -> PathBuf {
    let config_path = dir.path().join(filename);
    let config_content = r#"
device_start("Test Keyboard");
map("A", "VK_B");
map("B", "VK_C");
device_end();
"#;
    fs::write(&config_path, config_content).expect("Failed to write test config");
    config_path
}

/// Helper to create an advanced Rhai config with various features
fn create_advanced_rhai_config(dir: &TempDir, filename: &str) -> PathBuf {
    let config_path = dir.path().join(filename);
    let config_content = r#"
device_start("Advanced Keyboard");
// Simple remapping
map("A", "VK_B");

// Custom modifier
map("CapsLock", "MD_00");

// Custom lock
map("ScrollLock", "LK_00");

// Tap/hold behavior
tap_hold("Space", "VK_Space", "MD_01", 200);
device_end();
"#;
    fs::write(&config_path, config_content).expect("Failed to write test config");
    config_path
}

/// Helper to create an invalid Rhai config (missing prefix)
fn create_invalid_rhai_config(dir: &TempDir, filename: &str) -> PathBuf {
    let config_path = dir.path().join(filename);
    let config_content = r#"
device_start("Test Keyboard");
map("A", "B");  // Missing VK_ prefix
device_end();
"#;
    fs::write(&config_path, config_content).expect("Failed to write test config");
    config_path
}

/// Helper to create a Rhai config with syntax error
fn create_syntax_error_rhai_config(dir: &TempDir, filename: &str) -> PathBuf {
    let config_path = dir.path().join(filename);
    let config_content = r#"
device_start("Test Keyboard");
map("A", "VK_B"  // Missing closing parenthesis
device_end();
"#;
    fs::write(&config_path, config_content).expect("Failed to write test config");
    config_path
}

/// Helper to create a Rhai config with physical modifier in MD_ prefix
fn create_physical_modifier_error_config(dir: &TempDir, filename: &str) -> PathBuf {
    let config_path = dir.path().join(filename);
    let config_content = r#"
device_start("Test Keyboard");
map("CapsLock", "MD_LShift");  // Physical modifier in MD_ prefix
device_end();
"#;
    fs::write(&config_path, config_content).expect("Failed to write test config");
    config_path
}

// ============================================================================
// Compile Command Tests
// ============================================================================

#[test]
fn test_compile_simple_config() {
    let temp_dir = setup_test_dir();
    let input = create_simple_rhai_config(&temp_dir, "config.rhai");
    let output = temp_dir.path().join("output.krx");

    get_binary()
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully compiled"));

    // Verify output file exists
    assert!(output.exists(), "Output .krx file should exist");

    // Verify output file is not empty
    let metadata = fs::metadata(&output).expect("Failed to read output file metadata");
    assert!(
        metadata.len() > 48,
        "Output file should be larger than header size"
    );
}

#[test]
fn test_compile_default_output() {
    let temp_dir = setup_test_dir();
    let input = create_simple_rhai_config(&temp_dir, "config.rhai");
    let default_output = temp_dir.path().join("config.krx");

    get_binary()
        .arg("compile")
        .arg(&input)
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully compiled"));

    // Verify default output file exists
    assert!(
        default_output.exists(),
        "Default output .krx file should exist"
    );
}

#[test]
fn test_compile_advanced_config() {
    let temp_dir = setup_test_dir();
    let input = create_advanced_rhai_config(&temp_dir, "advanced.rhai");
    let output = temp_dir.path().join("advanced.krx");

    get_binary()
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully compiled"));

    assert!(output.exists(), "Output .krx file should exist");
}

#[test]
fn test_compile_missing_input_file() {
    let temp_dir = setup_test_dir();
    let input = temp_dir.path().join("nonexistent.rhai");
    let output = temp_dir.path().join("output.krx");

    get_binary()
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_compile_invalid_prefix() {
    let temp_dir = setup_test_dir();
    let input = create_invalid_rhai_config(&temp_dir, "invalid.rhai");
    let output = temp_dir.path().join("output.krx");

    get_binary()
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_compile_syntax_error() {
    let temp_dir = setup_test_dir();
    let input = create_syntax_error_rhai_config(&temp_dir, "syntax_error.rhai");
    let output = temp_dir.path().join("output.krx");

    get_binary()
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_compile_physical_modifier_error() {
    let temp_dir = setup_test_dir();
    let input = create_physical_modifier_error_config(&temp_dir, "physical_mod.rhai");
    let output = temp_dir.path().join("output.krx");

    get_binary()
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Error"));
}

// ============================================================================
// Verify Command Tests
// ============================================================================

#[test]
fn test_verify_valid_file() {
    let temp_dir = setup_test_dir();
    let input = create_simple_rhai_config(&temp_dir, "config.rhai");
    let krx_file = temp_dir.path().join("config.krx");

    // First compile
    get_binary()
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&krx_file)
        .assert()
        .success();

    // Then verify
    get_binary()
        .arg("verify")
        .arg(&krx_file)
        .assert()
        .success()
        .stderr(predicate::str::contains("✓ Verification passed"))
        .stderr(predicate::str::contains("✓ Magic bytes valid"))
        .stderr(predicate::str::contains("✓ Version:"))
        .stderr(predicate::str::contains("✓ SHA256 hash matches"))
        .stderr(predicate::str::contains(
            "✓ rkyv deserialization successful",
        ));
}

#[test]
fn test_verify_missing_file() {
    let temp_dir = setup_test_dir();
    let krx_file = temp_dir.path().join("nonexistent.krx");

    get_binary()
        .arg("verify")
        .arg(&krx_file)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_verify_corrupted_magic() {
    let temp_dir = setup_test_dir();
    let input = create_simple_rhai_config(&temp_dir, "config.rhai");
    let krx_file = temp_dir.path().join("config.krx");

    // Compile valid file
    get_binary()
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&krx_file)
        .assert()
        .success();

    // Corrupt the magic bytes
    let mut bytes = fs::read(&krx_file).expect("Failed to read file");
    bytes[0] = 0xFF; // Corrupt first byte
    fs::write(&krx_file, bytes).expect("Failed to write corrupted file");

    // Verify should fail
    get_binary()
        .arg("verify")
        .arg(&krx_file)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("✗ Magic bytes invalid"))
        .stderr(predicate::str::contains("✗ Verification failed"));
}

#[test]
fn test_verify_corrupted_hash() {
    let temp_dir = setup_test_dir();
    let input = create_simple_rhai_config(&temp_dir, "config.rhai");
    let krx_file = temp_dir.path().join("config.krx");

    // Compile valid file
    get_binary()
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&krx_file)
        .assert()
        .success();

    // Corrupt the hash (bytes 8-40)
    let mut bytes = fs::read(&krx_file).expect("Failed to read file");
    bytes[10] = 0xFF; // Corrupt a hash byte
    fs::write(&krx_file, bytes).expect("Failed to write corrupted file");

    // Verify should fail
    get_binary()
        .arg("verify")
        .arg(&krx_file)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("✗ SHA256 hash mismatch"))
        .stderr(predicate::str::contains("✗ Verification failed"));
}

#[test]
fn test_verify_truncated_file() {
    let temp_dir = setup_test_dir();
    let krx_file = temp_dir.path().join("truncated.krx");

    // Create a file that's too small
    fs::write(&krx_file, vec![0u8; 10]).expect("Failed to write truncated file");

    get_binary()
        .arg("verify")
        .arg(&krx_file)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Error"));
}

// ============================================================================
// Hash Command Tests
// ============================================================================

#[test]
fn test_hash_extract() {
    let temp_dir = setup_test_dir();
    let input = create_simple_rhai_config(&temp_dir, "config.rhai");
    let krx_file = temp_dir.path().join("config.krx");

    // Compile
    get_binary()
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&krx_file)
        .assert()
        .success();

    // Extract hash
    let output = get_binary()
        .arg("hash")
        .arg(&krx_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let hash_str = String::from_utf8_lossy(&output);
    let hash_str = hash_str.trim();

    // Verify hash is 64 hex characters (32 bytes * 2)
    assert_eq!(hash_str.len(), 64, "Hash should be 64 hex characters");
    assert!(
        hash_str.chars().all(|c| c.is_ascii_hexdigit()),
        "Hash should only contain hex characters"
    );
}

#[test]
fn test_hash_missing_file() {
    let temp_dir = setup_test_dir();
    let krx_file = temp_dir.path().join("nonexistent.krx");

    get_binary()
        .arg("hash")
        .arg(&krx_file)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_hash_truncated_file() {
    let temp_dir = setup_test_dir();
    let krx_file = temp_dir.path().join("truncated.krx");

    // Create a file that's too small
    fs::write(&krx_file, vec![0u8; 10]).expect("Failed to write truncated file");

    get_binary()
        .arg("hash")
        .arg(&krx_file)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("File too small"));
}

#[test]
fn test_hash_determinism() {
    let temp_dir = setup_test_dir();
    let input = create_simple_rhai_config(&temp_dir, "config.rhai");
    let krx_file1 = temp_dir.path().join("config1.krx");
    let krx_file2 = temp_dir.path().join("config2.krx");

    // Compile twice with deterministic build mode
    get_binary()
        .env("KEYRX_DETERMINISTIC_BUILD", "1")
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&krx_file1)
        .assert()
        .success();

    get_binary()
        .env("KEYRX_DETERMINISTIC_BUILD", "1")
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&krx_file2)
        .assert()
        .success();

    // Extract both hashes
    let hash1_output = get_binary()
        .arg("hash")
        .arg(&krx_file1)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let hash2_output = get_binary()
        .arg("hash")
        .arg(&krx_file2)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Hashes should be identical (deterministic serialization)
    assert_eq!(
        hash1_output, hash2_output,
        "Hashes should be identical for deterministic serialization"
    );
}

#[test]
fn test_hash_verify_valid() {
    let temp_dir = setup_test_dir();
    let input = create_simple_rhai_config(&temp_dir, "config.rhai");
    let krx_file = temp_dir.path().join("config.krx");

    // Compile
    get_binary()
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&krx_file)
        .assert()
        .success();

    // Verify hash with --verify flag
    get_binary()
        .arg("hash")
        .arg(&krx_file)
        .arg("--verify")
        .assert()
        .success()
        .stderr(predicate::str::contains("✓ Hash matches"));
}

#[test]
fn test_hash_verify_corrupted() {
    let temp_dir = setup_test_dir();
    let input = create_simple_rhai_config(&temp_dir, "config.rhai");
    let krx_file = temp_dir.path().join("config.krx");

    // Compile
    get_binary()
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&krx_file)
        .assert()
        .success();

    // Corrupt the data section (not the hash)
    let mut bytes = fs::read(&krx_file).expect("Failed to read file");
    if bytes.len() > 48 {
        bytes[50] = !bytes[50]; // Flip a bit in the data section
        fs::write(&krx_file, bytes).expect("Failed to write corrupted file");
    }

    // Verify should fail
    get_binary()
        .arg("hash")
        .arg(&krx_file)
        .arg("--verify")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("✗ Hash mismatch"))
        .stderr(predicate::str::contains("Embedded:"))
        .stderr(predicate::str::contains("Computed:"));
}

#[test]
fn test_hash_verify_displays_both_hashes_on_mismatch() {
    let temp_dir = setup_test_dir();
    let input = create_simple_rhai_config(&temp_dir, "config.rhai");
    let krx_file = temp_dir.path().join("config.krx");

    // Compile
    get_binary()
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&krx_file)
        .assert()
        .success();

    // Corrupt the data section
    let mut bytes = fs::read(&krx_file).expect("Failed to read file");
    bytes[60] = !bytes[60]; // Flip a bit
    fs::write(&krx_file, bytes).expect("Failed to write corrupted file");

    // Verify should show both hashes
    let output = get_binary()
        .arg("hash")
        .arg(&krx_file)
        .arg("--verify")
        .assert()
        .failure()
        .code(1)
        .get_output()
        .stderr
        .clone();

    let stderr_str = String::from_utf8_lossy(&output);

    // Check that both embedded and computed hashes are displayed
    assert!(stderr_str.contains("Embedded:"));
    assert!(stderr_str.contains("Computed:"));

    // Both should be 64 hex characters
    let lines: Vec<&str> = stderr_str.lines().collect();
    for line in lines {
        if line.contains("Embedded:") || line.contains("Computed:") {
            // Extract the hash part (everything after the colon and whitespace)
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() > 1 {
                let hash_part = parts[1].trim();
                assert_eq!(
                    hash_part.len(),
                    64,
                    "Hash should be 64 hex characters: {}",
                    line
                );
            }
        }
    }
}

// ============================================================================
// Parse Command Tests
// ============================================================================

#[test]
fn test_parse_human_readable() {
    let temp_dir = setup_test_dir();
    let input = create_simple_rhai_config(&temp_dir, "config.rhai");

    get_binary()
        .arg("parse")
        .arg(&input)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Configuration parsed successfully",
        ))
        .stdout(predicate::str::contains("Version:"))
        .stdout(predicate::str::contains("Devices:"))
        .stdout(predicate::str::contains("Test Keyboard"));
}

#[test]
fn test_parse_json_output() {
    let temp_dir = setup_test_dir();
    let input = create_simple_rhai_config(&temp_dir, "config.rhai");

    let output = get_binary()
        .arg("parse")
        .arg(&input)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&output);

    // Verify it's valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Output should be valid JSON");

    // Verify structure
    assert!(parsed.get("version").is_some(), "JSON should have version");
    assert!(parsed.get("devices").is_some(), "JSON should have devices");
    assert!(
        parsed.get("metadata").is_some(),
        "JSON should have metadata"
    );
}

#[test]
fn test_parse_advanced_config() {
    let temp_dir = setup_test_dir();
    let input = create_advanced_rhai_config(&temp_dir, "advanced.rhai");

    get_binary()
        .arg("parse")
        .arg(&input)
        .arg("--json")
        .assert()
        .success();
}

#[test]
fn test_parse_missing_file() {
    let temp_dir = setup_test_dir();
    let input = temp_dir.path().join("nonexistent.rhai");

    get_binary()
        .arg("parse")
        .arg(&input)
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_parse_invalid_syntax() {
    let temp_dir = setup_test_dir();
    let input = create_syntax_error_rhai_config(&temp_dir, "syntax_error.rhai");

    get_binary()
        .arg("parse")
        .arg(&input)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_parse_prefix_error() {
    let temp_dir = setup_test_dir();
    let input = create_invalid_rhai_config(&temp_dir, "invalid.rhai");

    get_binary()
        .arg("parse")
        .arg(&input)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Error"));
}

// ============================================================================
// Help Command Tests
// ============================================================================

#[test]
fn test_help_main() {
    get_binary()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("keyrx_compiler"))
        .stdout(predicate::str::contains("compile"))
        .stdout(predicate::str::contains("verify"))
        .stdout(predicate::str::contains("hash"))
        .stdout(predicate::str::contains("parse"));
}

#[test]
fn test_help_compile() {
    get_binary()
        .arg("compile")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Compile a Rhai script"))
        .stdout(predicate::str::contains("--output"));
}

#[test]
fn test_help_verify() {
    get_binary()
        .arg("verify")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Verify a .krx binary file"));
}

#[test]
fn test_help_hash() {
    get_binary()
        .arg("hash")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("SHA256 hash"));
}

#[test]
fn test_help_parse() {
    get_binary()
        .arg("parse")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse a Rhai script"))
        .stdout(predicate::str::contains("--json"));
}

// ============================================================================
// Edge Cases and Error Scenarios
// ============================================================================

#[test]
fn test_compile_empty_config() {
    let temp_dir = setup_test_dir();
    let input = temp_dir.path().join("empty.rhai");
    fs::write(&input, "").expect("Failed to write empty file");
    let output = temp_dir.path().join("output.krx");

    // Empty config should compile successfully (no devices)
    get_binary()
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .success();
}

#[test]
fn test_compile_multiple_devices() {
    let temp_dir = setup_test_dir();
    let input = temp_dir.path().join("multi.rhai");
    let config_content = r#"
device_start("Keyboard 1");
map("A", "VK_B");
device_end();

device_start("Keyboard 2");
map("C", "VK_D");
device_end();
"#;
    fs::write(&input, config_content).expect("Failed to write config");
    let output = temp_dir.path().join("output.krx");

    get_binary()
        .arg("compile")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .success();

    // Verify the output contains both devices
    get_binary()
        .arg("parse")
        .arg(&input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Keyboard 1"))
        .stdout(predicate::str::contains("Keyboard 2"));
}

#[test]
fn test_no_subcommand() {
    get_binary().assert().failure().code(2); // Clap returns exit code 2 for usage errors
}

#[test]
fn test_invalid_subcommand() {
    get_binary()
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}
