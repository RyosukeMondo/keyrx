#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Edge case and real-world validation tests.
//!
//! Tests edge cases and realistic configurations including:
//! - Parse errors and malformed scripts
//! - Real-world configuration examples
//! - Complex multi-feature scripts

use keyrx_core::validation::engine::ValidationEngine;
use keyrx_core::validation::types::ValidationOptions;

// =============================================================================
// Parse Error Tests
// =============================================================================

#[test]
fn detects_syntax_error() {
    let script = "this is not valid rhai {{{{";

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(!result.is_valid);
    assert!(!result.errors.is_empty());
    assert_eq!(result.errors[0].code, "E000");
}

#[test]
fn detects_unclosed_string() {
    let script = r#"remap("CapsLock, "Escape");"#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(!result.is_valid);
    assert!(!result.errors.is_empty());
}

#[test]
fn detects_undefined_function() {
    let script = "nonexistent_function();";

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(!result.is_valid);
    assert!(!result.errors.is_empty());
}

// =============================================================================
// Real-World Script Tests
// =============================================================================

#[test]
fn capslock_to_escape_config() {
    let script = r#"
        // Classic CapsLock to Escape remap
        remap("CapsLock", "Escape");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn vim_style_navigation_layer() {
    let script = r#"
        // Vim-style navigation on a layer
        define_layer("nav");

        // HJKL navigation
        layer_map("nav", "H", "Left");
        layer_map("nav", "J", "Down");
        layer_map("nav", "K", "Up");
        layer_map("nav", "L", "Right");

        // Tap-hold for layer activation
        tap_hold("CapsLock", "Escape", "LeftCtrl");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn home_row_mods_config() {
    let script = r#"
        // Home row mods
        tap_hold("A", "A", "LeftAlt");
        tap_hold("S", "S", "LeftCtrl");
        tap_hold("D", "D", "LeftShift");
        tap_hold("F", "F", "LeftMeta");

        tap_hold("J", "J", "RightMeta");
        tap_hold("K", "K", "RightShift");
        tap_hold("L", "L", "RightCtrl");
        tap_hold("Semicolon", "Semicolon", "RightAlt");

        // Timing adjustments
        tap_timeout(200);
        hold_delay(150);
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn combo_based_shortcuts() {
    let script = r#"
        // Combo-based shortcuts
        combo(["A", "S"], "Tab");
        combo(["S", "D"], "Escape");
        combo(["D", "F"], "Enter");

        // Timing
        combo_timeout(50);
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn multi_layer_config() {
    let script = r#"
        // Multiple layers
        define_layer("nav");
        define_layer("symbols");
        define_layer("numbers");

        // Navigation layer
        layer_map("nav", "H", "Left");
        layer_map("nav", "J", "Down");
        layer_map("nav", "K", "Up");
        layer_map("nav", "L", "Right");

        // Custom modifier for layer switching
        define_modifier("layer_switch");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}
