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
//! Comprehensive integration tests for the device definition system.
//!
//! This test suite covers:
//! - TOML parsing with various definition formats
//! - Validation of device definitions
//! - VID:PID lookup operations
//! - Recursive directory loading
//! - Error handling and edge cases
//! - Integration with real device definition files

use keyrx_core::definitions::DeviceDefinitionLibrary;
use keyrx_core::registry::PhysicalPosition;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// TOML Parsing Tests
// ============================================================================

#[test]
fn test_parse_minimal_device_definition() {
    let toml = r#"
name = "Minimal Device"
vendor_id = 0x1234
product_id = 0x5678

[layout]
layout_type = "matrix"
rows = 2
cols = 2

[matrix_map]
"1" = { row = 0, col = 0 }
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("minimal.toml");
    std::fs::write(&file_path, toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    let result = library.load_from_directory(temp_dir.path());

    assert!(result.is_ok(), "Failed to parse minimal definition");
    assert_eq!(result.unwrap(), 1);

    let def = library.find_definition(0x1234, 0x5678).unwrap();
    assert_eq!(def.name, "Minimal Device");
    assert_eq!(def.manufacturer, None);
    assert_eq!(def.visual, None);
}

#[test]
fn test_parse_full_device_definition_with_all_fields() {
    let toml = r#"
name = "Complete Device"
vendor_id = 0xABCD
product_id = 0xEF12
manufacturer = "Test Corp"

[layout]
layout_type = "matrix"
rows = 3
cols = 5

[matrix_map]
"1" = { row = 0, col = 0 }
"2" = { row = 0, col = 1 }
"3" = { row = 0, col = 2 }
"4" = { row = 0, col = 3 }
"5" = { row = 0, col = 4 }
"6" = { row = 1, col = 0 }
"7" = { row = 1, col = 1 }
"8" = { row = 1, col = 2 }
"9" = { row = 1, col = 3 }
"10" = { row = 1, col = 4 }
"11" = { row = 2, col = 0 }

[visual]
key_width = 72
key_height = 72
key_spacing = 8
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("complete.toml");
    std::fs::write(&file_path, toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(temp_dir.path()).unwrap();

    let def = library.find_definition(0xABCD, 0xEF12).unwrap();
    assert_eq!(def.name, "Complete Device");
    assert_eq!(def.manufacturer, Some("Test Corp".to_string()));
    assert_eq!(def.layout.layout_type, "matrix");
    assert_eq!(def.layout.rows, 3);
    assert_eq!(def.layout.cols, Some(5));
    assert_eq!(def.matrix_map.len(), 11);

    let visual = def.visual.unwrap();
    assert_eq!(visual.key_width, 72);
    assert_eq!(visual.key_height, 72);
    assert_eq!(visual.key_spacing, 8);
}

#[test]
fn test_parse_standard_keyboard_layout() {
    let toml = r#"
name = "Standard Keyboard"
vendor_id = 0x1111
product_id = 0x2222

[layout]
layout_type = "standard"
rows = 6
cols_per_row = [15, 15, 15, 13, 12, 8]

[matrix_map]
"30" = { row = 0, col = 0 }
"31" = { row = 0, col = 1 }
"32" = { row = 0, col = 2 }
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("keyboard.toml");
    std::fs::write(&file_path, toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(temp_dir.path()).unwrap();

    let def = library.find_definition(0x1111, 0x2222).unwrap();
    assert_eq!(def.layout.layout_type, "standard");
    assert_eq!(def.layout.cols_per_row, Some(vec![15, 15, 15, 13, 12, 8]));
}

#[test]
fn test_parse_split_keyboard_layout() {
    let toml = r#"
name = "Split Keyboard"
vendor_id = 0x3333
product_id = 0x4444

[layout]
layout_type = "split"
rows = 4
cols = 6

[matrix_map]
"1" = { row = 0, col = 0 }
"2" = { row = 0, col = 1 }
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("split.toml");
    std::fs::write(&file_path, toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(temp_dir.path()).unwrap();

    let def = library.find_definition(0x3333, 0x4444).unwrap();
    assert_eq!(def.layout.layout_type, "split");
}

#[test]
fn test_parse_large_scancode_values() {
    let toml = r#"
name = "HID Device"
vendor_id = 0x5555
product_id = 0x6666

[layout]
layout_type = "matrix"
rows = 2
cols = 2

[matrix_map]
"65535" = { row = 0, col = 0 }
"65534" = { row = 0, col = 1 }
"32768" = { row = 1, col = 0 }
"1" = { row = 1, col = 1 }
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("hid.toml");
    std::fs::write(&file_path, toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(temp_dir.path()).unwrap();

    let def = library.find_definition(0x5555, 0x6666).unwrap();
    assert_eq!(
        def.scancode_to_position(65535),
        Some(PhysicalPosition::new(0, 0))
    );
    assert_eq!(
        def.scancode_to_position(65534),
        Some(PhysicalPosition::new(0, 1))
    );
    assert_eq!(
        def.scancode_to_position(32768),
        Some(PhysicalPosition::new(1, 0))
    );
}

// ============================================================================
// Validation Tests
// ============================================================================

#[test]
fn test_validation_rejects_zero_vendor_id() {
    let toml = r#"
name = "Invalid Device"
vendor_id = 0
product_id = 0x1234

[layout]
layout_type = "matrix"
rows = 1
cols = 1

[matrix_map]
"1" = { row = 0, col = 0 }
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid.toml");
    std::fs::write(&file_path, toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    // Should be skipped due to validation error
    assert_eq!(count, 0);
    assert!(library.is_empty());
}

#[test]
fn test_validation_rejects_zero_product_id() {
    let toml = r#"
name = "Invalid Device"
vendor_id = 0x1234
product_id = 0

[layout]
layout_type = "matrix"
rows = 1
cols = 1

[matrix_map]
"1" = { row = 0, col = 0 }
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid.toml");
    std::fs::write(&file_path, toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(count, 0);
}

#[test]
fn test_validation_rejects_invalid_layout_type() {
    let toml = r#"
name = "Invalid Device"
vendor_id = 0x1234
product_id = 0x5678

[layout]
layout_type = "foobar"
rows = 1
cols = 1

[matrix_map]
"1" = { row = 0, col = 0 }
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid.toml");
    std::fs::write(&file_path, toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(count, 0);
}

#[test]
fn test_validation_rejects_zero_rows() {
    let toml = r#"
name = "Invalid Device"
vendor_id = 0x1234
product_id = 0x5678

[layout]
layout_type = "matrix"
rows = 0
cols = 1

[matrix_map]
"1" = { row = 0, col = 0 }
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid.toml");
    std::fs::write(&file_path, toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(count, 0);
}

#[test]
fn test_validation_rejects_zero_cols() {
    let toml = r#"
name = "Invalid Device"
vendor_id = 0x1234
product_id = 0x5678

[layout]
layout_type = "matrix"
rows = 1
cols = 0

[matrix_map]
"1" = { row = 0, col = 0 }
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid.toml");
    std::fs::write(&file_path, toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(count, 0);
}

#[test]
fn test_validation_rejects_empty_matrix_map() {
    let toml = r#"
name = "Invalid Device"
vendor_id = 0x1234
product_id = 0x5678

[layout]
layout_type = "matrix"
rows = 2
cols = 2

[matrix_map]
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid.toml");
    std::fs::write(&file_path, toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(count, 0);
}

#[test]
fn test_validation_rejects_position_out_of_bounds_row() {
    let toml = r#"
name = "Invalid Device"
vendor_id = 0x1234
product_id = 0x5678

[layout]
layout_type = "matrix"
rows = 2
cols = 2

[matrix_map]
"1" = { row = 5, col = 0 }
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid.toml");
    std::fs::write(&file_path, toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(count, 0);
}

#[test]
fn test_validation_rejects_position_out_of_bounds_col() {
    let toml = r#"
name = "Invalid Device"
vendor_id = 0x1234
product_id = 0x5678

[layout]
layout_type = "matrix"
rows = 2
cols = 2

[matrix_map]
"1" = { row = 0, col = 5 }
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid.toml");
    std::fs::write(&file_path, toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(count, 0);
}

#[test]
fn test_validation_rejects_cols_per_row_mismatch() {
    let toml = r#"
name = "Invalid Device"
vendor_id = 0x1234
product_id = 0x5678

[layout]
layout_type = "standard"
rows = 3
cols_per_row = [10, 12]

[matrix_map]
"1" = { row = 0, col = 0 }
"#;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("invalid.toml");
    std::fs::write(&file_path, toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(count, 0);
}

// ============================================================================
// VID:PID Lookup Tests
// ============================================================================

#[test]
fn test_lookup_existing_device() {
    let temp_dir = TempDir::new().unwrap();
    create_test_device(&temp_dir, "device1.toml", 0x1111, 0x2222, "Device 1");

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(temp_dir.path()).unwrap();

    let result = library.find_definition(0x1111, 0x2222);
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "Device 1");
}

#[test]
fn test_lookup_nonexistent_device() {
    let temp_dir = TempDir::new().unwrap();
    create_test_device(&temp_dir, "device1.toml", 0x1111, 0x2222, "Device 1");

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(temp_dir.path()).unwrap();

    let result = library.find_definition(0x9999, 0x8888);
    assert!(result.is_none());
}

#[test]
fn test_lookup_multiple_devices() {
    let temp_dir = TempDir::new().unwrap();
    create_test_device(&temp_dir, "device1.toml", 0x1111, 0x2222, "Device 1");
    create_test_device(&temp_dir, "device2.toml", 0x3333, 0x4444, "Device 2");
    create_test_device(&temp_dir, "device3.toml", 0x5555, 0x6666, "Device 3");

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(library.count(), 3);
    assert!(library.find_definition(0x1111, 0x2222).is_some());
    assert!(library.find_definition(0x3333, 0x4444).is_some());
    assert!(library.find_definition(0x5555, 0x6666).is_some());
}

#[test]
fn test_lookup_device_key_format() {
    let temp_dir = TempDir::new().unwrap();
    create_test_device(&temp_dir, "device1.toml", 0xABCD, 0xEF12, "Test");

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(temp_dir.path()).unwrap();

    let def = library.find_definition(0xABCD, 0xEF12).unwrap();
    assert_eq!(def.device_key(), "abcd:ef12");
}

#[test]
fn test_list_all_definitions() {
    let temp_dir = TempDir::new().unwrap();
    create_test_device(&temp_dir, "device1.toml", 0x1111, 0x2222, "Device 1");
    create_test_device(&temp_dir, "device2.toml", 0x3333, 0x4444, "Device 2");

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(temp_dir.path()).unwrap();

    let definitions: Vec<_> = library.list_definitions().collect();
    assert_eq!(definitions.len(), 2);

    let names: Vec<&str> = definitions.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"Device 1"));
    assert!(names.contains(&"Device 2"));
}

// ============================================================================
// Recursive Directory Loading Tests
// ============================================================================

#[test]
fn test_load_from_flat_directory() {
    let temp_dir = TempDir::new().unwrap();
    create_test_device(&temp_dir, "device1.toml", 0x1111, 0x2222, "Device 1");
    create_test_device(&temp_dir, "device2.toml", 0x3333, 0x4444, "Device 2");

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(count, 2);
    assert_eq!(library.count(), 2);
}

#[test]
fn test_load_from_nested_directories() {
    let temp_dir = TempDir::new().unwrap();

    // Create root level file
    create_test_device(&temp_dir, "root.toml", 0x1111, 0x2222, "Root Device");

    // Create subdirectory with files
    let subdir = temp_dir.path().join("vendor1");
    std::fs::create_dir(&subdir).unwrap();
    create_test_device_at(&subdir, "device1.toml", 0x3333, 0x4444, "Vendor1 Device");

    // Create nested subdirectory
    let nested = subdir.join("models");
    std::fs::create_dir(&nested).unwrap();
    create_test_device_at(
        &nested,
        "device2.toml",
        0x5555,
        0x6666,
        "Vendor1 Model Device",
    );

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(count, 3);
    assert_eq!(library.count(), 3);
    assert!(library.find_definition(0x1111, 0x2222).is_some());
    assert!(library.find_definition(0x3333, 0x4444).is_some());
    assert!(library.find_definition(0x5555, 0x6666).is_some());
}

#[test]
fn test_load_deeply_nested_directories() {
    let temp_dir = TempDir::new().unwrap();

    // Create deep nesting: level1/level2/level3/level4/device.toml
    let level1 = temp_dir.path().join("level1");
    std::fs::create_dir(&level1).unwrap();
    let level2 = level1.join("level2");
    std::fs::create_dir(&level2).unwrap();
    let level3 = level2.join("level3");
    std::fs::create_dir(&level3).unwrap();
    let level4 = level3.join("level4");
    std::fs::create_dir(&level4).unwrap();

    create_test_device_at(&level4, "deep.toml", 0x1234, 0x5678, "Deep Device");

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(count, 1);
    assert!(library.find_definition(0x1234, 0x5678).is_some());
}

#[test]
fn test_ignore_non_toml_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create .toml file
    create_test_device(&temp_dir, "device.toml", 0x1234, 0x5678, "Device");

    // Create non-.toml files
    std::fs::write(temp_dir.path().join("README.md"), "Documentation").unwrap();
    std::fs::write(temp_dir.path().join("config.json"), "{}").unwrap();
    std::fs::write(temp_dir.path().join("script.sh"), "#!/bin/bash").unwrap();
    std::fs::write(temp_dir.path().join(".gitignore"), "*.bak").unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    // Should only load the .toml file
    assert_eq!(count, 1);
    assert_eq!(library.count(), 1);
}

#[test]
fn test_load_from_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(count, 0);
    assert!(library.is_empty());
}

#[test]
fn test_skip_invalid_files_continue_loading() {
    let temp_dir = TempDir::new().unwrap();

    // Create valid file
    create_test_device(&temp_dir, "valid.toml", 0x1111, 0x2222, "Valid Device");

    // Create invalid files (should be skipped)
    let invalid_toml = r#"
name = "Invalid"
vendor_id = 0
product_id = 0x1234
"#;
    std::fs::write(temp_dir.path().join("invalid.toml"), invalid_toml).unwrap();

    let corrupt_toml = "this is not valid TOML }{][";
    std::fs::write(temp_dir.path().join("corrupt.toml"), corrupt_toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    // Should load only the valid file
    assert_eq!(count, 1);
    assert_eq!(library.count(), 1);
    assert!(library.find_definition(0x1111, 0x2222).is_some());
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_duplicate_definition_same_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Create two files with same VID:PID
    create_test_device(&temp_dir, "device1.toml", 0x1234, 0x5678, "Device 1");
    create_test_device(&temp_dir, "device2.toml", 0x1234, 0x5678, "Device 2");

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    // First file loads, second is skipped as duplicate
    assert_eq!(count, 1);
    assert_eq!(library.count(), 1);
}

#[test]
fn test_malformed_toml_syntax() {
    let temp_dir = TempDir::new().unwrap();
    let malformed = r#"
name = "Broken Device
vendor_id = 0x1234
[layout
"#;
    std::fs::write(temp_dir.path().join("broken.toml"), malformed).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(count, 0);
}

#[test]
fn test_invalid_scancode_in_matrix_map() {
    let toml = r#"
name = "Invalid Scancode"
vendor_id = 0x1234
product_id = 0x5678

[layout]
layout_type = "matrix"
rows = 2
cols = 2

[matrix_map]
"not_a_number" = { row = 0, col = 0 }
"#;

    let temp_dir = TempDir::new().unwrap();
    std::fs::write(temp_dir.path().join("invalid.toml"), toml).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    let count = library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(count, 0);
}

#[test]
fn test_get_source_path() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("device.toml");
    create_test_device(&temp_dir, "device.toml", 0x1234, 0x5678, "Device");

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(temp_dir.path()).unwrap();

    let source = library.get_source_path(0x1234, 0x5678);
    assert!(source.is_some());
    assert_eq!(source.unwrap(), file_path.as_path());
}

#[test]
fn test_clear_library() {
    let temp_dir = TempDir::new().unwrap();
    create_test_device(&temp_dir, "device1.toml", 0x1111, 0x2222, "Device 1");
    create_test_device(&temp_dir, "device2.toml", 0x3333, 0x4444, "Device 2");

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(library.count(), 2);

    library.clear();

    assert_eq!(library.count(), 0);
    assert!(library.is_empty());
    assert!(library.find_definition(0x1111, 0x2222).is_none());
}

// ============================================================================
// Real Device Definition Tests
// ============================================================================

#[test]
fn test_load_real_device_definitions() {
    let device_defs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("device_definitions");

    // Skip test if directory doesn't exist (CI environment)
    if !device_defs_path.exists() {
        println!("Skipping: device_definitions directory not found");
        return;
    }

    let mut library = DeviceDefinitionLibrary::new();
    let result = library.load_from_directory(&device_defs_path);

    assert!(
        result.is_ok(),
        "Failed to load real definitions: {:?}",
        result.err()
    );

    let count = result.unwrap();
    assert!(
        count >= 5,
        "Expected at least 5 definitions (ANSI, ISO, 3x Stream Deck), got {}",
        count
    );

    println!("Loaded {} device definitions", count);
    for def in library.list_definitions() {
        println!(
            "  - {} ({:04x}:{:04x}) - {} layout with {} mappings",
            def.name,
            def.vendor_id,
            def.product_id,
            def.layout_type_str(),
            def.matrix_map.len()
        );
    }
}

#[test]
fn test_stream_deck_mk2_definition() {
    let device_defs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("device_definitions");

    if !device_defs_path.exists() {
        return;
    }

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(&device_defs_path).unwrap();

    let mk2 = library.find_definition(0x0fd9, 0x0080);
    assert!(mk2.is_some(), "Stream Deck MK.2 definition not found");

    let mk2 = mk2.unwrap();
    assert_eq!(mk2.name, "Elgato Stream Deck MK.2");
    assert_eq!(mk2.layout.layout_type, "matrix");
    assert_eq!(mk2.layout.rows, 3);
    assert_eq!(mk2.layout.cols, Some(5));
    assert_eq!(mk2.matrix_map.len(), 15); // 3x5 = 15 buttons
}

#[test]
fn test_stream_deck_xl_definition() {
    let device_defs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("device_definitions");

    if !device_defs_path.exists() {
        return;
    }

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(&device_defs_path).unwrap();

    let xl = library.find_definition(0x0fd9, 0x006c);
    assert!(xl.is_some(), "Stream Deck XL definition not found");

    let xl = xl.unwrap();
    assert_eq!(xl.name, "Elgato Stream Deck XL");
    assert_eq!(xl.layout.layout_type, "matrix");
    assert_eq!(xl.layout.rows, 4);
    assert_eq!(xl.layout.cols, Some(8));
    assert_eq!(xl.matrix_map.len(), 32); // 4x8 = 32 buttons
}

#[test]
fn test_stream_deck_mini_definition() {
    let device_defs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("device_definitions");

    if !device_defs_path.exists() {
        return;
    }

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(&device_defs_path).unwrap();

    let mini = library.find_definition(0x0fd9, 0x0063);
    assert!(mini.is_some(), "Stream Deck Mini definition not found");

    let mini = mini.unwrap();
    assert_eq!(mini.name, "Elgato Stream Deck Mini");
    assert_eq!(mini.layout.layout_type, "matrix");
    assert_eq!(mini.layout.rows, 2);
    assert_eq!(mini.layout.cols, Some(3));
    assert_eq!(mini.matrix_map.len(), 6); // 2x3 = 6 buttons
}

#[test]
fn test_ansi_keyboard_definition() {
    let device_defs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("device_definitions");

    if !device_defs_path.exists() {
        return;
    }

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(&device_defs_path).unwrap();

    // ANSI keyboard should have VID:PID 0x0000:0x0001 (generic)
    let ansi = library.find_definition(0x0000, 0x0001);
    if let Some(ansi) = ansi {
        assert!(ansi.name.contains("ANSI"));
        assert_eq!(ansi.layout.layout_type, "standard");
        assert!(ansi.matrix_map.len() >= 100); // At least 100 keys
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a test device definition in the given directory
fn create_test_device(dir: &TempDir, filename: &str, vid: u16, pid: u16, name: &str) {
    create_test_device_at(dir.path(), filename, vid, pid, name);
}

/// Create a test device definition at the given path
fn create_test_device_at(path: &std::path::Path, filename: &str, vid: u16, pid: u16, name: &str) {
    let toml = format!(
        r#"
name = "{}"
vendor_id = 0x{:04X}
product_id = 0x{:04X}

[layout]
layout_type = "matrix"
rows = 2
cols = 2

[matrix_map]
"1" = {{ row = 0, col = 0 }}
"2" = {{ row = 0, col = 1 }}
"3" = {{ row = 1, col = 0 }}
"4" = {{ row = 1, col = 1 }}
"#,
        name, vid, pid
    );

    std::fs::write(path.join(filename), toml).unwrap();
}
