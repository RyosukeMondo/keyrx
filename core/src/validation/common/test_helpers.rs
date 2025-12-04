//! Test helpers for validation testing.
//!
//! Provides builder functions for creating PendingOp instances and
//! assertion helpers for validating warnings and errors in tests.

#![cfg(test)]

use crate::drivers::keycodes::KeyCode;
use crate::engine::{HoldAction, LayerAction};
use crate::scripting::{LayerMapAction, PendingOp};
use crate::validation::types::{ValidationWarning, WarningCategory};

/// Builder for creating remap operations in tests.
pub fn remap(from: KeyCode, to: KeyCode) -> PendingOp {
    PendingOp::Remap { from, to }
}

/// Builder for creating block operations in tests.
pub fn block(key: KeyCode) -> PendingOp {
    PendingOp::Block { key }
}

/// Builder for creating pass operations in tests.
pub fn pass(key: KeyCode) -> PendingOp {
    PendingOp::Pass { key }
}

/// Builder for creating tap-hold operations in tests.
pub fn tap_hold(key: KeyCode, tap: KeyCode, hold: HoldAction) -> PendingOp {
    PendingOp::TapHold { key, tap, hold }
}

/// Builder for creating combo operations in tests.
pub fn combo(keys: Vec<KeyCode>, action: LayerAction) -> PendingOp {
    PendingOp::Combo { keys, action }
}

/// Builder for creating combo block operations in tests.
pub fn combo_block(keys: Vec<KeyCode>) -> PendingOp {
    PendingOp::Combo {
        keys,
        action: LayerAction::Block,
    }
}

/// Builder for creating combo remap operations in tests.
pub fn combo_remap(keys: Vec<KeyCode>, to: KeyCode) -> PendingOp {
    PendingOp::Combo {
        keys,
        action: LayerAction::Remap(to),
    }
}

/// Builder for creating layer definition operations in tests.
pub fn layer_define(name: impl Into<String>, transparent: bool) -> PendingOp {
    PendingOp::LayerDefine {
        name: name.into(),
        transparent,
    }
}

/// Builder for creating layer map operations in tests.
pub fn layer_map(layer: impl Into<String>, key: KeyCode, action: LayerMapAction) -> PendingOp {
    PendingOp::LayerMap {
        layer: layer.into(),
        key,
        action,
    }
}

/// Builder for creating layer push operations in tests.
pub fn layer_push(name: impl Into<String>) -> PendingOp {
    PendingOp::LayerPush { name: name.into() }
}

/// Builder for creating layer toggle operations in tests.
pub fn layer_toggle(name: impl Into<String>) -> PendingOp {
    PendingOp::LayerToggle { name: name.into() }
}

/// Builder for creating layer pop operations in tests.
pub fn layer_pop() -> PendingOp {
    PendingOp::LayerPop
}

// Assertion helpers for warnings

/// Assert that a list of warnings is empty.
pub fn assert_no_warnings(warnings: &[ValidationWarning]) {
    assert!(
        warnings.is_empty(),
        "Expected no warnings but got: {:?}",
        warnings
    );
}

/// Assert that warnings contain exactly one warning with the given code.
pub fn assert_warning_code(warnings: &[ValidationWarning], expected_code: &str) {
    assert_eq!(
        warnings.len(),
        1,
        "Expected 1 warning but got {}: {:?}",
        warnings.len(),
        warnings
    );
    assert_eq!(
        warnings[0].code, expected_code,
        "Expected code '{}' but got '{}': {}",
        expected_code, warnings[0].code, warnings[0].message
    );
}

/// Assert that warnings contain a specific number of warnings.
pub fn assert_warning_count(warnings: &[ValidationWarning], expected_count: usize) {
    assert_eq!(
        warnings.len(),
        expected_count,
        "Expected {} warnings but got {}: {:?}",
        expected_count,
        warnings.len(),
        warnings
    );
}

/// Assert that warnings contain at least one warning with the given code.
pub fn assert_has_warning_code(warnings: &[ValidationWarning], expected_code: &str) {
    let has_code = warnings.iter().any(|w| w.code == expected_code);
    assert!(
        has_code,
        "Expected warning with code '{}' but not found in: {:?}",
        expected_code, warnings
    );
}

/// Assert that a warning has the expected category.
pub fn assert_warning_category(warning: &ValidationWarning, expected_category: WarningCategory) {
    assert_eq!(
        warning.category, expected_category,
        "Expected category {:?} but got {:?}",
        expected_category, warning.category
    );
}

/// Assert that all warnings have the expected category.
pub fn assert_all_warnings_category(
    warnings: &[ValidationWarning],
    expected_category: WarningCategory,
) {
    for (i, warning) in warnings.iter().enumerate() {
        assert_eq!(
            warning.category, expected_category,
            "Warning {} has wrong category: expected {:?}, got {:?}",
            i, expected_category, warning.category
        );
    }
}

/// Assert that a warning message contains a specific substring.
pub fn assert_warning_contains(warning: &ValidationWarning, substring: &str) {
    assert!(
        warning.message.contains(substring),
        "Expected warning message to contain '{}' but got: {}",
        substring,
        warning.message
    );
}

/// Assert that the first warning contains a specific substring.
pub fn assert_first_warning_contains(warnings: &[ValidationWarning], substring: &str) {
    assert!(
        !warnings.is_empty(),
        "Expected at least one warning but got none"
    );
    assert_warning_contains(&warnings[0], substring);
}

/// Assert that a warning has a location pointing to the given line.
pub fn assert_warning_location(warning: &ValidationWarning, expected_line: usize) {
    let location = warning
        .location
        .as_ref()
        .expect("Expected warning to have location");
    assert_eq!(
        location.line, expected_line,
        "Expected warning at line {} but got line {}",
        expected_line, location.line
    );
}

/// Assert that the first warning points to the given line.
pub fn assert_first_warning_location(warnings: &[ValidationWarning], expected_line: usize) {
    assert!(
        !warnings.is_empty(),
        "Expected at least one warning but got none"
    );
    assert_warning_location(&warnings[0], expected_line);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remap_builder_creates_remap_op() {
        let op = remap(KeyCode::A, KeyCode::B);
        match op {
            PendingOp::Remap { from, to } => {
                assert_eq!(from, KeyCode::A);
                assert_eq!(to, KeyCode::B);
            }
            _ => panic!("Expected Remap operation"),
        }
    }

    #[test]
    fn block_builder_creates_block_op() {
        let op = block(KeyCode::A);
        match op {
            PendingOp::Block { key } => {
                assert_eq!(key, KeyCode::A);
            }
            _ => panic!("Expected Block operation"),
        }
    }

    #[test]
    fn combo_block_builder_creates_combo_with_block_action() {
        let op = combo_block(vec![KeyCode::A, KeyCode::B]);
        match op {
            PendingOp::Combo { keys, action } => {
                assert_eq!(keys, vec![KeyCode::A, KeyCode::B]);
                assert!(matches!(action, LayerAction::Block));
            }
            _ => panic!("Expected Combo operation"),
        }
    }

    #[test]
    fn combo_remap_builder_creates_combo_with_remap_action() {
        let op = combo_remap(vec![KeyCode::A, KeyCode::B], KeyCode::C);
        match op {
            PendingOp::Combo { keys, action } => {
                assert_eq!(keys, vec![KeyCode::A, KeyCode::B]);
                match action {
                    LayerAction::Remap(to) => assert_eq!(to, KeyCode::C),
                    _ => panic!("Expected Remap action"),
                }
            }
            _ => panic!("Expected Combo operation"),
        }
    }

    #[test]
    fn tap_hold_builder_creates_tap_hold_op() {
        let op = tap_hold(
            KeyCode::CapsLock,
            KeyCode::Escape,
            HoldAction::Key(KeyCode::LeftCtrl),
        );
        match op {
            PendingOp::TapHold { key, tap, hold } => {
                assert_eq!(key, KeyCode::CapsLock);
                assert_eq!(tap, KeyCode::Escape);
                assert!(matches!(hold, HoldAction::Key(KeyCode::LeftCtrl)));
            }
            _ => panic!("Expected TapHold operation"),
        }
    }

    #[test]
    fn layer_builders_create_layer_ops() {
        let define = layer_define("nav", false);
        assert!(matches!(define, PendingOp::LayerDefine { .. }));

        let push = layer_push("nav");
        assert!(matches!(push, PendingOp::LayerPush { .. }));

        let toggle = layer_toggle("nav");
        assert!(matches!(toggle, PendingOp::LayerToggle { .. }));

        let pop = layer_pop();
        assert!(matches!(pop, PendingOp::LayerPop));
    }

    #[test]
    #[should_panic(expected = "Expected no warnings")]
    fn assert_no_warnings_panics_on_warnings() {
        let warnings = vec![ValidationWarning::new(
            "W001",
            WarningCategory::Conflict,
            "test",
        )];
        assert_no_warnings(&warnings);
    }

    #[test]
    fn assert_no_warnings_passes_on_empty() {
        assert_no_warnings(&[]);
    }

    #[test]
    fn assert_warning_code_validates_code() {
        let warnings = vec![ValidationWarning::new(
            "W001",
            WarningCategory::Conflict,
            "test",
        )];
        assert_warning_code(&warnings, "W001");
    }

    #[test]
    #[should_panic(expected = "Expected 1 warning")]
    fn assert_warning_code_panics_on_wrong_count() {
        let warnings = vec![
            ValidationWarning::new("W001", WarningCategory::Conflict, "test1"),
            ValidationWarning::new("W002", WarningCategory::Conflict, "test2"),
        ];
        assert_warning_code(&warnings, "W001");
    }

    #[test]
    fn assert_warning_count_validates_count() {
        let warnings = vec![
            ValidationWarning::new("W001", WarningCategory::Conflict, "test1"),
            ValidationWarning::new("W002", WarningCategory::Conflict, "test2"),
        ];
        assert_warning_count(&warnings, 2);
    }

    #[test]
    fn assert_has_warning_code_finds_code() {
        let warnings = vec![
            ValidationWarning::new("W001", WarningCategory::Conflict, "test1"),
            ValidationWarning::new("W002", WarningCategory::Conflict, "test2"),
        ];
        assert_has_warning_code(&warnings, "W001");
        assert_has_warning_code(&warnings, "W002");
    }

    #[test]
    fn assert_warning_contains_validates_message() {
        let warning = ValidationWarning::new("W001", WarningCategory::Conflict, "duplicate remap");
        assert_warning_contains(&warning, "duplicate");
        assert_warning_contains(&warning, "remap");
    }
}
