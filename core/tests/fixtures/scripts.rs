#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Common Rhai script snippets for testing.
//!
//! This module provides reusable script fixtures that cover:
//! - Basic remapping operations
//! - Layer management
//! - Modifier usage
//! - Combo key definitions
//! - Timing configurations
//! - Error scenarios (syntax errors, runtime errors)
//!
//! These fixtures reduce boilerplate in test files and ensure consistency
//! across different test categories.

/// Returns a minimal valid script with a single remap operation.
///
/// This is the simplest possible valid script for basic tests.
pub fn minimal_script() -> &'static str {
    r#"remap("CapsLock", "Escape");"#
}

/// Returns a basic remap script with multiple simple remappings.
///
/// Useful for testing basic validation and operation building.
pub fn basic_remap_script() -> &'static str {
    r#"
        remap("CapsLock", "Escape");
        remap("A", "B");
    "#
}

/// Returns a script demonstrating tap-hold functionality.
pub fn tap_hold_script() -> &'static str {
    r#"
        tap_hold("CapsLock", "Escape", "LeftCtrl");
        tap_hold("Space", "Space", "LeftShift");
    "#
}

/// Returns a script demonstrating layer functionality.
///
/// Includes layer definition, activation, and layer mappings.
pub fn layer_script() -> &'static str {
    r#"
        define_layer("navigation");
        define_layer("symbols", true);
        layer_push("navigation");
        layer_toggle("symbols");
        layer_pop();

        layer_map("navigation", "H", "Left");
        layer_map("navigation", "J", "Down");
        layer_map("navigation", "K", "Up");
        layer_map("navigation", "L", "Right");
    "#
}

/// Returns a comprehensive layer script with navigation mappings.
pub fn layer_navigation_script() -> &'static str {
    r#"
        define_layer("nav");
        layer_map("nav", "H", "Left");
        layer_map("nav", "J", "Down");
        layer_map("nav", "K", "Up");
        layer_map("nav", "L", "Right");
    "#
}

/// Returns a script demonstrating modifier functionality.
pub fn modifier_script() -> &'static str {
    r#"
        define_modifier("hyper");
        modifier_activate("hyper");
        modifier_deactivate("hyper");
        modifier_one_shot("hyper");
    "#
}

/// Returns a script demonstrating combo key functionality.
pub fn combo_script() -> &'static str {
    r#"
        combo(["A", "S"], "Escape");
        combo(["D", "F"], "block");
        combo(["J", "K"], "Tab");
    "#
}

/// Returns a script configuring timing parameters.
pub fn timing_script() -> &'static str {
    r#"
        tap_timeout(200);
        combo_timeout(50);
        hold_delay(150);
    "#
}

/// Returns a comprehensive script using multiple features.
///
/// This is useful for integration tests that need a realistic,
/// complex configuration.
pub fn complex_valid_script() -> &'static str {
    r#"
        // Basic remappings
        remap("CapsLock", "Escape");
        tap_hold("Space", "Space", "LeftShift");

        // Layer configuration
        define_layer("nav");
        layer_map("nav", "H", "Left");
        layer_map("nav", "J", "Down");
        layer_map("nav", "K", "Up");
        layer_map("nav", "L", "Right");

        // Modifier configuration
        define_modifier("hyper");

        // Timing configuration
        tap_timeout(200);
        combo_timeout(50);

        // Combo keys
        combo(["A", "S"], "Tab");
    "#
}

/// Returns a script with blocking operations.
pub fn block_script() -> &'static str {
    r#"
        block("CapsLock");
        remap("A", "B");
    "#
}

// =============================================================================
// Error Scenarios
// =============================================================================

/// Returns a script with a syntax error (unclosed string).
///
/// This causes a Rhai parsing error before execution.
pub fn syntax_error_script() -> &'static str {
    r#"remap("CapsLock, "Escape");"#
}

/// Returns a script with an invalid key name.
///
/// This causes a runtime error (E000) when the script executes.
pub fn invalid_key_error_script() -> &'static str {
    r#"remap("InvalidKeyName123", "Escape");"#
}

/// Returns a script attempting to use an undefined layer.
///
/// This causes a validation error (E002).
pub fn undefined_layer_error_script() -> &'static str {
    r#"layer_push("nonexistent_layer");"#
}

/// Returns a script attempting to use an undefined modifier.
///
/// This causes a validation error (E003).
pub fn undefined_modifier_error_script() -> &'static str {
    r#"modifier_activate("undefined_mod");"#
}

/// Returns a script with unclosed function parenthesis.
///
/// This causes a syntax error during parsing.
pub fn unclosed_paren_error_script() -> &'static str {
    r#"remap("A", "B";"#
}

/// Returns a script with invalid Rhai syntax (missing semicolon is OK in Rhai,
/// but using an invalid operator is not).
pub fn invalid_syntax_error_script() -> &'static str {
    r#"remap("A" @@ "B");"#
}

// =============================================================================
// Warning Scenarios
// =============================================================================

/// Returns a script that triggers a duplicate remap warning.
///
/// Maps the same source key twice, which is valid but suspicious.
pub fn duplicate_remap_warning_script() -> &'static str {
    r#"
        remap("A", "B");
        remap("A", "C");
    "#
}

/// Returns a script that triggers a safety warning.
///
/// Remapping Escape is discouraged as it's the emergency exit key.
pub fn escape_remap_warning_script() -> &'static str {
    r#"remap("Escape", "A");"#
}

/// Returns a script with a shadowing warning scenario.
///
/// Block followed by remap of the same key.
pub fn shadowing_warning_script() -> &'static str {
    r#"
        block("A");
        remap("A", "B");
    "#
}

// =============================================================================
// Empty and Edge Cases
// =============================================================================

/// Returns an empty script (valid but does nothing).
pub fn empty_script() -> &'static str {
    ""
}

/// Returns a script with only comments.
pub fn comment_only_script() -> &'static str {
    r#"
        // This is a comment
        // Another comment
    "#
}

/// Returns a script with whitespace only.
pub fn whitespace_only_script() -> &'static str {
    "   \n\t\n  "
}

// =============================================================================
// Test Discovery Helpers
// =============================================================================

/// Returns a Rhai test script with multiple test functions.
///
/// Useful for testing the test discovery and runner infrastructure.
pub fn test_functions_script() -> &'static str {
    r#"
        // Helper function (not a test)
        fn helper_add(a, b) {
            a + b
        }

        // Test: Simple arithmetic
        fn test_arithmetic() {
            let result = helper_add(2, 3);
            if result != 5 {
                throw "Expected 5, got " + result;
            }
        }

        // Test: String operations
        fn test_string_concat() {
            let s = "hello" + " world";
            if s != "hello world" {
                throw "String concat failed";
            }
        }

        // Test: Array operations
        fn test_array_push() {
            let arr = [];
            arr.push(1);
            arr.push(2);
            if arr.len() != 2 {
                throw "Array length should be 2";
            }
        }
    "#
}

/// Returns a script with a failing test.
pub fn failing_test_script() -> &'static str {
    r#"
        fn test_failure() {
            throw "This test always fails";
        }
    "#
}
