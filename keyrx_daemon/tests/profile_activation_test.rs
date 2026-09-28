//! Profile Activation Tests
//! These tests catch the bug where large/complex profiles fail to activate
//! while simple profiles succeed.

use keyrx_compiler::compile_file;
use keyrx_core::config::ConfigRoot;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

/// Test that simple profiles activate successfully
#[test]
fn test_simple_profile_activation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("simple.rhai");

    // Write a simple profile
    let simple_config = r#"
// Simple profile
fn main() {
    remap_key(KEY_A, KEY_B);
}
"#;
    fs::write(&config_path, simple_config).unwrap();

    // Compile it
    let krx_path = temp_dir.path().join("simple.krx");
    let result = compile_file(&config_path, &krx_path);

    assert!(
        result.is_ok(),
        "Simple profile should compile successfully: {:?}",
        result.err()
    );

    // Verify output file exists and is not empty
    assert!(krx_path.exists(), "Compiled .krx file should exist");

    let krx_data = fs::read(&krx_path).unwrap();
    assert!(!krx_data.is_empty(), "Compiled .krx should not be empty");

    // Verify it can be deserialized using rkyv
    let result = rkyv::check_archived_root::<ConfigRoot>(&krx_data);
    assert!(
        result.is_ok(),
        "Simple profile should deserialize: {:?}",
        result.err()
    );
}

/// Test that complex profiles (like default.rhai) activate successfully
#[test]
fn test_complex_profile_activation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("complex.rhai");

    // Write a complex profile with multiple layers and many remappings
    let complex_config = r#"
// Complex profile with many remappings
fn main() {
    // Base layer
    remap_key(KEY_A, KEY_Z);
    remap_key(KEY_B, KEY_Y);
    remap_key(KEY_C, KEY_X);
    remap_key(KEY_D, KEY_W);
    remap_key(KEY_E, KEY_V);
    remap_key(KEY_F, KEY_U);
    remap_key(KEY_G, KEY_T);
    remap_key(KEY_H, KEY_S);
    remap_key(KEY_I, KEY_R);
    remap_key(KEY_J, KEY_Q);

    // Layer 1 with modifiers
    add_layer(1, [KEY_LEFTSHIFT]);
    remap_key_layer(1, KEY_A, KEY_1);
    remap_key_layer(1, KEY_B, KEY_2);
    remap_key_layer(1, KEY_C, KEY_3);
    remap_key_layer(1, KEY_D, KEY_4);
    remap_key_layer(1, KEY_E, KEY_5);
    remap_key_layer(1, KEY_F, KEY_6);
    remap_key_layer(1, KEY_G, KEY_7);
    remap_key_layer(1, KEY_H, KEY_8);
    remap_key_layer(1, KEY_I, KEY_9);
    remap_key_layer(1, KEY_J, KEY_0);

    // Layer 2
    add_layer(2, [KEY_LEFTCTRL]);
    remap_key_layer(2, KEY_A, KEY_F1);
    remap_key_layer(2, KEY_B, KEY_F2);
    remap_key_layer(2, KEY_C, KEY_F3);
    remap_key_layer(2, KEY_D, KEY_F4);
    remap_key_layer(2, KEY_E, KEY_F5);
    remap_key_layer(2, KEY_F, KEY_F6);
    remap_key_layer(2, KEY_G, KEY_F7);
    remap_key_layer(2, KEY_H, KEY_F8);
    remap_key_layer(2, KEY_I, KEY_F9);
    remap_key_layer(2, KEY_J, KEY_F10);

    // Macros
    define_macro("hello", [KEY_H, KEY_E, KEY_L, KEY_L, KEY_O]);
    bind_macro(KEY_F1, "hello");
}
"#;
    fs::write(&config_path, complex_config).unwrap();

    // Compile it with timeout (complex profiles might take longer)
    let krx_path = temp_dir.path().join("complex.krx");
    let result = compile_file(&config_path, &krx_path);

    assert!(
        result.is_ok(),
        "Complex profile should compile successfully: {:?}",
        result.err()
    );

    // Verify output file exists and is not empty
    assert!(krx_path.exists(), "Compiled .krx file should exist");

    let krx_data = fs::read(&krx_path).unwrap();
    assert!(!krx_data.is_empty(), "Compiled .krx should not be empty");

    // Verify it can be deserialized using rkyv
    let result = rkyv::check_archived_root::<ConfigRoot>(&krx_data);
    assert!(
        result.is_ok(),
        "Complex profile should deserialize: {:?}",
        result.err()
    );
}

/// Test that large files (>10KB) compile within reasonable time
#[test]
fn test_large_profile_timeout() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("large.rhai");

    // Generate a large profile (similar to user's 24KB default.rhai)
    let mut large_config = String::from("fn main() {\n");

    // Add 500+ remappings to make file >10KB
    for i in 0..500 {
        large_config.push_str(&format!(
            "    remap_key(KEY_{}, KEY_{});\n",
            i % 26,
            (i + 1) % 26
        ));
    }

    large_config.push_str("}\n");

    assert!(
        large_config.len() > 10_000,
        "Test profile should be >10KB (got {} bytes)",
        large_config.len()
    );

    fs::write(&config_path, &large_config).unwrap();

    // Compile with timeout check
    let krx_path = temp_dir.path().join("large.krx");
    let start = std::time::Instant::now();

    let result = compile_file(&config_path, &krx_path);
    let elapsed = start.elapsed();

    // Should complete within 30 seconds
    assert!(
        elapsed < Duration::from_secs(30),
        "Large profile compilation took too long: {:?}",
        elapsed
    );

    assert!(
        result.is_ok(),
        "Large profile should compile successfully: {:?}",
        result.err()
    );

    println!(
        "Large profile ({} bytes) compiled in {:?}",
        large_config.len(),
        elapsed
    );
}

/// Test activation via HTTP API (integration test)
#[cfg(feature = "integration_tests")]
#[tokio::test]
async fn test_profile_activation_via_api() {
    use reqwest::Client;
    use serde_json::json;

    // Assumes daemon is running on localhost:9867
    let client = Client::new();
    let base_url = "http://localhost:9867/api";

    // Test profile-a (known working)
    let response = client
        .post(&format!("{}/profiles/profile-a/activate", base_url))
        .timeout(Duration::from_secs(30))
        .send()
        .await;

    assert!(
        response.is_ok(),
        "profile-a activation should succeed: {:?}",
        response.err()
    );

    let status = response.unwrap().status();
    assert!(
        status.is_success(),
        "profile-a should return success status: {}",
        status
    );

    // Test default (was failing)
    let response = client
        .post(&format!("{}/profiles/default/activate", base_url))
        .timeout(Duration::from_secs(60))
        .send()
        .await;

    assert!(
        response.is_ok(),
        "default activation should not timeout: {:?}",
        response.err()
    );

    let response = response.unwrap();
    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        panic!("default profile activation failed: {} - {}", status, body);
    }
}

/// Test that activation failures are properly reported
#[test]
fn test_invalid_profile_error_reporting() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.rhai");

    // Write invalid Rhai syntax (unclosed brace)
    let invalid_config = r#"
device_start("*");
map("VK_A", "VK_B"  // Missing closing parenthesis
device_end();
"#;
    fs::write(&config_path, invalid_config).unwrap();

    // Should return error with clear message
    let krx_path = temp_dir.path().join("invalid.krx");
    let result = compile_file(&config_path, &krx_path);

    assert!(result.is_err(), "Invalid profile should fail compilation");

    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("syntax") || error.contains("parse") || error.contains("invalid"),
        "Error should mention syntax/parse issue: {}",
        error
    );

    println!("Error message: {}", error);
}

/// Compiling a large profile stays fast, and every mapping survives.
///
/// Uses real mappings over distinct keys (a Rhai `fn` that is never called
/// would compile to an empty config and time nothing). The bound is loose
/// on purpose - it catches an order-of-magnitude regression without flaking
/// when the machine is busy running other test binaries in parallel.
#[test]
fn test_profile_compilation_performance() {
    const KEYS: [&str; 26] = [
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R",
        "S", "T", "U", "V", "W", "X", "Y", "Z",
    ];
    let temp_dir = TempDir::new().unwrap();
    // 26 base mappings + 26 in each of 38 layers = 1014 mappings.
    let mut config = String::from("device_start(\"*\");\n");
    for (i, key) in KEYS.iter().enumerate() {
        let to = KEYS[(i + 1) % KEYS.len()];
        config.push_str(&format!("map(\"VK_{key}\", \"VK_{to}\");\n"));
    }
    for layer in 0..38 {
        config.push_str(&format!("when_start(\"MD_{layer:02X}\");\n"));
        for (i, key) in KEYS.iter().enumerate() {
            let to = KEYS[(i + 3) % KEYS.len()];
            config.push_str(&format!("map(\"VK_{key}\", \"VK_{to}\");\n"));
        }
        config.push_str("when_end();\n");
    }
    config.push_str("device_end();\n");
    let rhai = temp_dir.path().join("huge.rhai");
    let krx = temp_dir.path().join("huge.krx");
    fs::write(&rhai, &config).unwrap();

    let start = std::time::Instant::now();
    compile_file(&rhai, &krx).expect("compiles");
    let elapsed = start.elapsed();

    let root = keyrx_daemon::config_loader::load_config(&krx).expect("loads");
    let device = &root.devices[0];
    let layered: usize = device
        .mappings
        .iter()
        .map(|m| match m {
            keyrx_core::config::KeyMapping::Conditional { mappings, .. } => mappings.len(),
            keyrx_core::config::KeyMapping::Base(_) => 1,
        })
        .sum();
    assert_eq!(layered, 26 + 26 * 38, "every mapping compiled");
    assert!(
        elapsed < Duration::from_secs(2),
        "1014 mappings took {elapsed:?} to compile"
    );
}
