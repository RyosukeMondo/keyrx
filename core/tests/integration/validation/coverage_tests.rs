#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Coverage tracking validation tests.
//!
//! Tests coverage reporting functionality including:
//! - Tracking affected keys
//! - Coverage report generation
//! - JSON coverage output

use keyrx_core::validation::engine::ValidationEngine;
use keyrx_core::validation::types::ValidationOptions;

#[test]
fn coverage_report_tracks_affected_keys() {
    let script = r#"
        remap("A", "B");
        block("C");
        tap_hold("D", "E", "F");
        combo(["G", "H"], "I");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new().with_coverage());

    assert!(result.is_valid);
    assert!(result.coverage.is_some());

    let coverage = result.coverage.unwrap();
    assert!(coverage.remapped.iter().any(|k| k.name() == "A"));
    assert!(coverage.blocked.iter().any(|k| k.name() == "C"));
    assert!(coverage.tap_hold.iter().any(|k| k.name() == "D"));
    assert!(
        coverage.combo_triggers.iter().any(|k| k.name() == "G")
            || coverage.combo_triggers.iter().any(|k| k.name() == "H")
    );
}

#[test]
fn json_coverage_when_requested() {
    let script = r#"remap("A", "B");"#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new().with_coverage());

    let json = serde_json::to_string(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(parsed["coverage"].is_object());
    assert!(parsed["coverage"]["remapped"].is_array());
    assert!(parsed["coverage"]["blocked"].is_array());
}
