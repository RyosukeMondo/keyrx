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
//! Basic validation engine integration tests.
//!
//! Tests core validation functionality including:
//! - Script parsing and validation
//! - Feature detection (remaps, layers, modifiers, combos)
//! - Error detection and reporting
//! - Visual output generation

use keyrx_core::validation::engine::ValidationEngine;
use keyrx_core::validation::types::ValidationOptions;

#[test]
fn validates_simple_remap_script() {
    let script = r#"
        remap("CapsLock", "Escape");
        remap("A", "B");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn validates_tap_hold_script() {
    let script = r#"
        tap_hold("CapsLock", "Escape", "LeftCtrl");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn validates_layer_script() {
    let script = r#"
        define_layer("navigation");
        define_layer("symbols", true);
        layer_push("navigation");
        layer_toggle("symbols");
        layer_pop();
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn validates_modifier_script() {
    let script = r#"
        define_modifier("hyper");
        modifier_activate("hyper");
        modifier_deactivate("hyper");
        modifier_one_shot("hyper");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn validates_combo_script() {
    let script = r#"
        combo(["A", "S"], "Escape");
        combo(["D", "F"], "block");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn validates_layer_map_script() {
    let script = r#"
        define_layer("nav");
        layer_map("nav", "H", "Left");
        layer_map("nav", "J", "Down");
        layer_map("nav", "K", "Up");
        layer_map("nav", "L", "Right");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn validates_timing_script() {
    let script = r#"
        tap_timeout(200);
        combo_timeout(50);
        hold_delay(150);
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn detects_invalid_key_name() {
    // Invalid key names cause Rhai runtime errors (E000)
    let script = r#"remap("InvalidKeyName123", "Escape");"#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(!result.is_valid);
    // Invalid keys in Rhai functions cause runtime errors
    assert!(result.errors.iter().any(|e| e.code == "E000"));
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("Unknown key")));
}

#[test]
fn detects_undefined_layer() {
    let script = r#"layer_push("nonexistent_layer");"#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|e| e.code == "E002"));
}

#[test]
fn detects_undefined_modifier() {
    let script = r#"modifier_activate("undefined_mod");"#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|e| e.code == "E003"));
}

#[test]
fn no_false_positives_on_valid_complex_script() {
    let script = r#"
        // A more complex script with various features
        remap("CapsLock", "Escape");
        tap_hold("Space", "Space", "LeftShift");

        define_layer("nav");
        layer_map("nav", "H", "Left");
        layer_map("nav", "J", "Down");
        layer_map("nav", "K", "Up");
        layer_map("nav", "L", "Right");

        define_modifier("hyper");

        tap_timeout(200);
        combo_timeout(50);

        combo(["A", "S"], "Tab");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn visual_output_contains_legend() {
    let script = r#"remap("A", "B");"#;

    let engine = ValidationEngine::new();
    let (result, visual) = engine.validate_with_visual(script);

    assert!(result.is_valid);
    assert!(visual.is_some());
    let visual_str = visual.unwrap();
    assert!(visual_str.contains("Legend:"));
}
