#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Safety and strict mode validation tests.
//!
//! Tests safety warnings and strict validation including:
//! - Escape key remap safety warnings
//! - Strict mode behavior
//! - Warning suppression
//! - Config-driven validation rules

use keyrx_core::validation::config::ValidationConfig;
use keyrx_core::validation::engine::ValidationEngine;
use keyrx_core::validation::types::{ValidationOptions, WarningCategory};
use std::io::Write;

#[test]
fn detects_escape_remap_safety_warning() {
    let script = r#"remap("Escape", "A");"#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.has_warnings());
    assert!(result
        .warnings
        .iter()
        .any(|w| w.category == WarningCategory::Safety));
}

#[test]
fn strict_mode_fails_on_any_warning() {
    let script = r#"
        remap("A", "B");
        remap("A", "C");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new().strict());

    assert!(!result.is_valid);
    assert!(result.has_warnings());
}

#[test]
fn strict_mode_passes_without_warnings() {
    let script = r#"remap("A", "B");"#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new().strict());

    assert!(result.is_valid);
    assert!(!result.has_warnings());
}

#[test]
fn no_warnings_option_suppresses_warnings() {
    let script = r#"
        remap("A", "B");
        remap("A", "C");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new().no_warnings());

    assert!(result.is_valid);
    assert!(!result.has_warnings());
}

#[test]
fn respects_max_errors_limit() {
    let mut config = ValidationConfig::default();
    config.max_errors = 2;

    let script = r#"
        layer_push("layer1");
        layer_push("layer2");
        layer_push("layer3");
        layer_push("layer4");
    "#;

    let engine = ValidationEngine::with_config(config);
    let result = engine.validate(script, ValidationOptions::new());

    // Should stop at max_errors
    assert_eq!(result.errors.len(), 2);
}

#[test]
fn respects_custom_tap_timeout_range() {
    let mut config = ValidationConfig::default();
    config.tap_timeout_warn_range = (100, 300);

    let engine = ValidationEngine::with_config(config);

    // 200ms is within range - no warning
    let script1 = "tap_timeout(200);";
    let result1 = engine.validate(script1, ValidationOptions::new());
    assert!(!result1.has_warnings());

    // 50ms is below min - warning
    let script2 = "tap_timeout(50);";
    let result2 = engine.validate(script2, ValidationOptions::new());
    assert!(result2.has_warnings());

    // 500ms is above max - warning
    let script3 = "tap_timeout(500);";
    let result3 = engine.validate(script3, ValidationOptions::new());
    assert!(result3.has_warnings());
}

#[test]
fn respects_custom_combo_timeout_range() {
    let mut config = ValidationConfig::default();
    config.combo_timeout_warn_range = (20, 80);

    let engine = ValidationEngine::with_config(config);

    // 50ms is within range - no warning
    let script1 = "combo_timeout(50);";
    let result1 = engine.validate(script1, ValidationOptions::new());
    assert!(!result1.has_warnings());

    // 5ms is below min - warning
    let script2 = "combo_timeout(5);";
    let result2 = engine.validate(script2, ValidationOptions::new());
    assert!(result2.has_warnings());
}

#[test]
fn loads_config_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("validation.toml");

    let mut file = std::fs::File::create(&config_path).unwrap();
    writeln!(file, "max_errors = 5").unwrap();
    writeln!(file, "max_suggestions = 3").unwrap();

    let config = ValidationConfig::load_from_path(&config_path).unwrap();
    assert_eq!(config.max_errors, 5);
    assert_eq!(config.max_suggestions, 3);
}
