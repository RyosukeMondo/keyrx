//! Tests for combo shadowing detection.
//!
//! These tests verify that the ShadowingDetector correctly identifies when
//! one combo's keys are a subset of another combo's keys, causing the shorter
//! combo to trigger before the longer one can complete.

use crate::drivers::keycodes::KeyCode;
use crate::engine::LayerAction;
use crate::scripting::PendingOp;
use crate::validation::config::ValidationConfig;
use crate::validation::detectors::shadowing::ShadowingDetector;
use crate::validation::detectors::{Detector, DetectorContext, Severity};

#[test]
fn no_shadowing_for_empty_ops() {
    let detector = ShadowingDetector::new();
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(&[], &ctx);
    assert!(result.issues.is_empty());
}

#[test]
fn no_shadowing_for_disjoint_combos() {
    let ops = vec![
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::S],
            action: LayerAction::Block,
        },
        PendingOp::Combo {
            keys: vec![KeyCode::D, KeyCode::F],
            action: LayerAction::Block,
        },
    ];
    let detector = ShadowingDetector::new();
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(&ops, &ctx);
    assert!(result.issues.is_empty());
}

#[test]
fn detects_simple_subset_shadowing() {
    // [A, S] shadows [A, S, D]
    let ops = vec![
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::S],
            action: LayerAction::Block,
        },
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::S, KeyCode::D],
            action: LayerAction::Block,
        },
    ];
    let detector = ShadowingDetector::new();
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
    assert_eq!(result.issues[0].severity, Severity::Warning);
    assert_eq!(result.issues[0].detector, "shadowing");
    assert!(result.issues[0].message.contains("shadows"));
    assert!(result.issues[0].message.contains("A+S"));
}

#[test]
fn detects_reversed_order_shadowing() {
    // [A, S, D] is defined first, but [A, S] still shadows it
    let ops = vec![
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::S, KeyCode::D],
            action: LayerAction::Block,
        },
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::S],
            action: LayerAction::Block,
        },
    ];
    let detector = ShadowingDetector::new();
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
}

#[test]
fn no_shadowing_for_same_size_different_keys() {
    // [A, S] and [A, D] - same size, different keys
    let ops = vec![
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::S],
            action: LayerAction::Block,
        },
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::D],
            action: LayerAction::Block,
        },
    ];
    let detector = ShadowingDetector::new();
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(&ops, &ctx);
    assert!(result.issues.is_empty());
}

#[test]
fn no_shadowing_for_identical_combos() {
    // Same combo twice is not shadowing (it's a duplicate, handled elsewhere)
    let ops = vec![
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::S],
            action: LayerAction::Block,
        },
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::S],
            action: LayerAction::Remap(KeyCode::B),
        },
    ];
    let detector = ShadowingDetector::new();
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(&ops, &ctx);
    assert!(result.issues.is_empty());
}

#[test]
fn handles_unsorted_combo_keys() {
    // Keys in different order should still be detected
    let ops = vec![
        PendingOp::Combo {
            keys: vec![KeyCode::S, KeyCode::A], // reversed
            action: LayerAction::Block,
        },
        PendingOp::Combo {
            keys: vec![KeyCode::D, KeyCode::A, KeyCode::S], // scrambled
            action: LayerAction::Block,
        },
    ];
    let detector = ShadowingDetector::new();
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
}

#[test]
fn detects_multiple_shadowing_relationships() {
    // [A] shadows [A, S] and [A, S, D]
    // [A, S] also shadows [A, S, D]
    let ops = vec![
        PendingOp::Combo {
            keys: vec![KeyCode::A],
            action: LayerAction::Block,
        },
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::S],
            action: LayerAction::Block,
        },
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::S, KeyCode::D],
            action: LayerAction::Block,
        },
    ];
    let detector = ShadowingDetector::new();
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(&ops, &ctx);
    // [A] shadows [A,S], [A] shadows [A,S,D], [A,S] shadows [A,S,D]
    assert_eq!(result.issues.len(), 3);
}

#[test]
fn shadowing_issue_has_correct_location() {
    let ops = vec![
        PendingOp::Remap {
            from: KeyCode::X,
            to: KeyCode::Y,
        }, // index 0
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::S],
            action: LayerAction::Block,
        }, // index 1
        PendingOp::Block { key: KeyCode::Z }, // index 2
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::S, KeyCode::D],
            action: LayerAction::Block,
        }, // index 3
    ];
    let detector = ShadowingDetector::new();
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
    // Location should point to the shadowed (longer) combo at index 3 -> line 4
    assert_eq!(result.issues[0].locations[0].line, 4);
}

#[test]
fn ignores_non_combo_ops() {
    // Only Combo ops should be considered
    let ops = vec![
        PendingOp::Remap {
            from: KeyCode::A,
            to: KeyCode::B,
        },
        PendingOp::Block { key: KeyCode::S },
        PendingOp::Combo {
            keys: vec![KeyCode::D, KeyCode::F],
            action: LayerAction::Block,
        },
    ];
    let detector = ShadowingDetector::new();
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(&ops, &ctx);
    assert!(result.issues.is_empty());
}

#[test]
fn single_key_combo_can_shadow_larger_combo() {
    // Single-key "combos" are unusual but the detector should handle them
    let ops = vec![
        PendingOp::Combo {
            keys: vec![KeyCode::A],
            action: LayerAction::Block,
        },
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::B],
            action: LayerAction::Block,
        },
    ];
    let detector = ShadowingDetector::new();
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
    assert!(result.issues[0].message.contains("shadows"));
}

#[test]
fn partial_overlap_combos_no_shadowing() {
    // [A, B] and [B, C] overlap but neither is a subset
    let ops = vec![
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::B],
            action: LayerAction::Block,
        },
        PendingOp::Combo {
            keys: vec![KeyCode::B, KeyCode::C],
            action: LayerAction::Block,
        },
    ];
    let detector = ShadowingDetector::new();
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(&ops, &ctx);
    assert!(result.issues.is_empty());
}

#[test]
fn combo_with_all_common_keys_but_different_order_shadows() {
    // Same keys in different order should still be detected
    let ops = vec![
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::B],
            action: LayerAction::Block,
        },
        PendingOp::Combo {
            keys: vec![KeyCode::C, KeyCode::B, KeyCode::A], // A,B is subset
            action: LayerAction::Block,
        },
    ];
    let detector = ShadowingDetector::new();
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
}

#[test]
fn combo_with_remap_action_also_detected_for_shadowing() {
    let ops = vec![
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::B],
            action: LayerAction::Remap(KeyCode::X),
        },
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::B, KeyCode::C],
            action: LayerAction::Remap(KeyCode::Y),
        },
    ];
    let detector = ShadowingDetector::new();
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
}

#[test]
fn detector_is_skippable() {
    let detector = ShadowingDetector::new();
    assert!(detector.is_skippable());
}

#[test]
fn detector_name_is_shadowing() {
    let detector = ShadowingDetector::new();
    assert_eq!(detector.name(), "shadowing");
}

#[test]
fn detector_stats_are_populated() {
    let ops = vec![
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::S],
            action: LayerAction::Block,
        },
        PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::S, KeyCode::D],
            action: LayerAction::Block,
        },
    ];
    let detector = ShadowingDetector::new();
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(&ops, &ctx);

    assert_eq!(result.stats.operations_checked, 2);
    assert_eq!(result.stats.issues_found, 1);
    assert!(result.stats.duration.as_nanos() > 0);
}
