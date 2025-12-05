//! Conflict detection tests for remaps and blocks.
//!
//! Tests for detecting duplicate remaps, remap-block conflicts,
//! tap-hold conflicts, and related warning generation.

use crate::drivers::keycodes::KeyCode;
use crate::engine::HoldAction;
use crate::validation::common::test_helpers::*;
use crate::validation::conflicts::detect_remap_conflicts;
use crate::validation::types::WarningCategory;

// ==================== Basic Conflict Tests ====================

#[test]
fn no_conflicts_in_empty_ops() {
    let warnings = detect_remap_conflicts(&[]);
    assert_no_warnings(&warnings);
}

#[test]
fn no_conflicts_for_different_keys() {
    let ops = vec![remap(KeyCode::A, KeyCode::B), remap(KeyCode::C, KeyCode::D)];
    assert_no_warnings(&detect_remap_conflicts(&ops));
}

// ==================== Duplicate Remap Tests ====================

#[test]
fn detects_duplicate_remap() {
    let ops = vec![remap(KeyCode::A, KeyCode::B), remap(KeyCode::A, KeyCode::C)];
    let warnings = detect_remap_conflicts(&ops);
    assert_warning_code(&warnings, "W-conflict:duplicate-remap");
    assert_first_warning_contains(&warnings, "remapped multiple times");
}

#[test]
fn same_key_to_different_targets_creates_conflict() {
    // A→B, A→C - same source, different targets
    let ops = vec![remap(KeyCode::A, KeyCode::B), remap(KeyCode::A, KeyCode::C)];
    let warnings = detect_remap_conflicts(&ops);
    assert_warning_code(&warnings, "W-conflict:duplicate-remap");
}

// ==================== Remap-Block Conflict Tests ====================

#[test]
fn detects_remap_block_conflict() {
    let ops = vec![remap(KeyCode::A, KeyCode::B), block(KeyCode::A)];
    let warnings = detect_remap_conflicts(&ops);
    assert_warning_code(&warnings, "W-conflict:remap-block");
    assert_first_warning_contains(&warnings, "both remapped");
    assert_first_warning_contains(&warnings, "blocked");
}

#[test]
fn detects_block_remap_conflict_reversed() {
    let ops = vec![block(KeyCode::A), remap(KeyCode::A, KeyCode::B)];
    let warnings = detect_remap_conflicts(&ops);
    assert_warning_code(&warnings, "W-conflict:remap-block");
}

// ==================== Duplicate Block Tests ====================

#[test]
fn duplicate_block_is_detected() {
    let ops = vec![block(KeyCode::A), block(KeyCode::A)];
    let warnings = detect_remap_conflicts(&ops);
    assert_warning_code(&warnings, "W-conflict:duplicate-block");
}

// ==================== Tap-Hold Conflict Tests ====================

#[test]
fn detects_tap_hold_remap_conflict() {
    let ops = vec![
        tap_hold(
            KeyCode::CapsLock,
            KeyCode::Escape,
            HoldAction::Key(KeyCode::LeftCtrl),
        ),
        remap(KeyCode::CapsLock, KeyCode::LeftCtrl),
    ];
    let warnings = detect_remap_conflicts(&ops);
    assert_warning_code(&warnings, "W-conflict:taphold-remap");
    assert_first_warning_contains(&warnings, "tap-hold");
    assert_first_warning_contains(&warnings, "remap");
}

#[test]
fn detects_tap_hold_block_conflict() {
    let ops = vec![
        tap_hold(
            KeyCode::CapsLock,
            KeyCode::Escape,
            HoldAction::Key(KeyCode::LeftCtrl),
        ),
        block(KeyCode::CapsLock),
    ];
    let warnings = detect_remap_conflicts(&ops);
    assert_warning_code(&warnings, "W-conflict:taphold-block");
}

#[test]
fn detects_duplicate_tap_hold() {
    let ops = vec![
        tap_hold(
            KeyCode::CapsLock,
            KeyCode::Escape,
            HoldAction::Key(KeyCode::LeftCtrl),
        ),
        tap_hold(
            KeyCode::CapsLock,
            KeyCode::Tab,
            HoldAction::Key(KeyCode::LeftAlt),
        ),
    ];
    let warnings = detect_remap_conflicts(&ops);
    assert_warning_code(&warnings, "W-conflict:duplicate-taphold");
}

#[test]
fn tap_hold_on_different_keys_no_conflict() {
    let ops = vec![
        tap_hold(KeyCode::A, KeyCode::B, HoldAction::Key(KeyCode::C)),
        tap_hold(KeyCode::D, KeyCode::E, HoldAction::Key(KeyCode::F)),
    ];
    assert_no_warnings(&detect_remap_conflicts(&ops));
}

#[test]
fn remap_and_tap_hold_on_different_keys_no_conflict() {
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        tap_hold(KeyCode::C, KeyCode::D, HoldAction::Key(KeyCode::E)),
    ];
    assert_no_warnings(&detect_remap_conflicts(&ops));
}

// ==================== Pass Operation Tests ====================

#[test]
fn pass_operations_generate_warnings() {
    let ops = vec![remap(KeyCode::A, KeyCode::B), pass(KeyCode::A)];
    let warnings = detect_remap_conflicts(&ops);
    assert_warning_code(&warnings, "W-conflict:pass-conflict");
}

// ==================== Multiple Conflicts Tests ====================

#[test]
fn detects_multiple_conflicts_for_same_key() {
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        remap(KeyCode::A, KeyCode::C),
        block(KeyCode::A),
    ];
    let warnings = detect_remap_conflicts(&ops);
    // 3 operations = 3 pairs: (0,1), (0,2), (1,2)
    assert_warning_count(&warnings, 3);
}

// ==================== Location Tracking Tests ====================

#[test]
fn warning_has_correct_location() {
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        block(KeyCode::C),
        remap(KeyCode::A, KeyCode::D),
    ];
    let warnings = detect_remap_conflicts(&ops);
    assert_warning_count(&warnings, 1);
    // Location should point to the second conflicting operation (index 2 -> line 3)
    assert_first_warning_location(&warnings, 3);
}

#[test]
fn warning_message_contains_operation_indices() {
    let ops = vec![
        block(KeyCode::X),             // index 0
        remap(KeyCode::A, KeyCode::B), // index 1
        block(KeyCode::Y),             // index 2
        remap(KeyCode::A, KeyCode::C), // index 3
    ];
    let warnings = detect_remap_conflicts(&ops);
    assert_warning_count(&warnings, 1);
    // Message should contain the 1-indexed operation numbers
    assert!(warnings[0].message.contains("2") && warnings[0].message.contains("4"));
}

// ==================== Warning Category Tests ====================

#[test]
fn warning_category_is_conflict() {
    let ops = vec![remap(KeyCode::A, KeyCode::B), remap(KeyCode::A, KeyCode::C)];
    let warnings = detect_remap_conflicts(&ops);
    assert_all_warnings_category(&warnings, WarningCategory::Conflict);
}

// ==================== Edge Cases ====================

#[test]
fn multiple_remaps_to_same_target_no_false_positive_cycle() {
    // A→C, B→C is not a cycle (both point to C but don't loop back)
    let ops = vec![remap(KeyCode::A, KeyCode::C), remap(KeyCode::B, KeyCode::C)];
    assert_no_warnings(&detect_remap_conflicts(&ops));
}

#[test]
fn layer_map_ops_not_tracked_in_remap_conflicts() {
    // LayerMap operations should not be considered for remap conflict detection
    use crate::scripting::LayerMapAction;
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        layer_map("nav", KeyCode::A, LayerMapAction::Remap(KeyCode::C)),
    ];
    // LayerMap on key A should not conflict with base Remap on key A
    assert_no_warnings(&detect_remap_conflicts(&ops));
}
