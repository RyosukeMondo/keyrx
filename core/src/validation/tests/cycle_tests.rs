//! Cycle detection tests for circular remap dependencies.
//!
//! Tests for detecting circular remap chains (A→B→C→A) which can cause
//! unpredictable behavior where keys effectively swap or create feedback loops.

use crate::drivers::keycodes::KeyCode;
use crate::validation::common::test_helpers::*;
use crate::validation::config::ValidationConfig;
use crate::validation::detectors::cycles::CycleDetector;
use crate::validation::detectors::{Detector, DetectorContext, Severity};

// Helper function to create default config
fn default_config() -> ValidationConfig {
    ValidationConfig::default()
}

// Helper function to create detector context
fn make_ctx(config: ValidationConfig) -> DetectorContext {
    DetectorContext::new(config)
}

// ==================== Basic Cycle Tests ====================

#[test]
fn no_cycles_for_empty_ops() {
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&[], &ctx);
    assert!(result.issues.is_empty());
}

#[test]
fn no_cycles_for_linear_chain() {
    // A→B, B→C, C→D is not a cycle
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        remap(KeyCode::B, KeyCode::C),
        remap(KeyCode::C, KeyCode::D),
    ];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);
    assert!(result.issues.is_empty());
}

// ==================== Two-Key Cycle Tests ====================

#[test]
fn detects_simple_two_key_cycle() {
    // A→B, B→A is a cycle
    let ops = vec![remap(KeyCode::A, KeyCode::B), remap(KeyCode::B, KeyCode::A)];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
    assert!(result.issues[0].message.contains("Circular remap"));
}

#[test]
fn does_not_report_duplicate_cycles() {
    // A→B, B→A forms one cycle, should only be reported once
    let ops = vec![remap(KeyCode::A, KeyCode::B), remap(KeyCode::B, KeyCode::A)];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);
    // Should only report once, not from A and from B
    assert_eq!(result.issues.len(), 1);
}

// ==================== Three-Key Cycle Tests ====================

#[test]
fn detects_three_key_cycle() {
    // A→B, B→C, C→A is a cycle
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        remap(KeyCode::B, KeyCode::C),
        remap(KeyCode::C, KeyCode::A),
    ];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
    let msg = &result.issues[0].message;
    assert!(msg.contains("Circular remap"));
    assert!(msg.contains("A"));
    assert!(msg.contains("B"));
    assert!(msg.contains("C"));
}

// ==================== Four-Key Cycle Tests ====================

#[test]
fn four_key_cycle_detected_within_depth() {
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        remap(KeyCode::B, KeyCode::C),
        remap(KeyCode::C, KeyCode::D),
        remap(KeyCode::D, KeyCode::A),
    ];

    let detector = CycleDetector;
    let mut config = ValidationConfig::default();
    config.max_cycle_depth = 5;
    let ctx = make_ctx(config);
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
}

// ==================== Max Cycle Depth Tests ====================

#[test]
fn respects_max_cycle_depth() {
    // Create a long chain that exceeds depth 3
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        remap(KeyCode::B, KeyCode::C),
        remap(KeyCode::C, KeyCode::D),
        remap(KeyCode::D, KeyCode::E),
        remap(KeyCode::E, KeyCode::A),
    ];

    let detector = CycleDetector;

    // With max_cycle_depth=3, this 5-key cycle should not be detected
    let mut config = ValidationConfig::default();
    config.max_cycle_depth = 3;
    let ctx = make_ctx(config.clone());
    let result = detector.detect(&ops, &ctx);
    assert!(result.issues.is_empty());

    // With max_cycle_depth=10, it should be detected
    config.max_cycle_depth = 10;
    let ctx = make_ctx(config);
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
}

#[test]
fn config_max_cycle_depth_of_one_only_finds_self_loops() {
    // With max_depth=1, we should only find direct self-loops (A→A)
    // A→B→A requires depth 2
    let mut config = ValidationConfig::default();
    config.max_cycle_depth = 1;

    let ops = vec![remap(KeyCode::A, KeyCode::B), remap(KeyCode::B, KeyCode::A)];
    let detector = CycleDetector;
    let ctx = make_ctx(config);
    let result = detector.detect(&ops, &ctx);
    // Path length check happens after we've traversed, so depth=1 means path.len() > 1 fails
    assert!(result.issues.is_empty());
}

#[test]
fn config_max_cycle_depth_of_two_finds_simple_cycles() {
    let mut config = ValidationConfig::default();
    config.max_cycle_depth = 2;

    let ops = vec![remap(KeyCode::A, KeyCode::B), remap(KeyCode::B, KeyCode::A)];
    let detector = CycleDetector;
    let ctx = make_ctx(config);
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
}

// ==================== Multiple Independent Cycles Tests ====================

#[test]
fn detects_multiple_independent_cycles() {
    // A→B→A and C→D→C are two separate cycles
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        remap(KeyCode::B, KeyCode::A),
        remap(KeyCode::C, KeyCode::D),
        remap(KeyCode::D, KeyCode::C),
    ];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 2);
}

// ==================== Non-Remap Operation Tests ====================

#[test]
fn ignores_non_remap_ops_for_cycles() {
    // Block and other ops don't form remap cycles
    let ops = vec![remap(KeyCode::A, KeyCode::B), block(KeyCode::B)];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);
    assert!(result.issues.is_empty());
}

// ==================== Location and Issue Tests ====================

#[test]
fn cycle_issue_has_correct_location() {
    let ops = vec![
        block(KeyCode::X),             // index 0
        remap(KeyCode::A, KeyCode::B), // index 1
        remap(KeyCode::B, KeyCode::A), // index 2
    ];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
    // Location should point to the edge that completes the cycle
    let loc = &result.issues[0].locations[0];
    assert!(loc.line == 2 || loc.line == 3); // Either index 1 or 2 (1-indexed)
}

#[test]
fn cycle_issue_severity_is_warning() {
    let ops = vec![remap(KeyCode::A, KeyCode::B), remap(KeyCode::B, KeyCode::A)];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues[0].severity, Severity::Warning);
}

// ==================== False Positive Prevention Tests ====================

#[test]
fn multiple_remaps_to_same_target_no_false_positive_cycle() {
    // A→C, B→C is not a cycle (both point to C but don't loop back)
    let ops = vec![remap(KeyCode::A, KeyCode::C), remap(KeyCode::B, KeyCode::C)];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);
    assert!(result.issues.is_empty());
}

#[test]
fn self_remap_is_not_a_cycle() {
    // A→A is technically a remap but treated as a no-op, not a cycle
    let ops = vec![remap(KeyCode::A, KeyCode::A)];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);
    // This creates a self-loop in the graph, but path length is 1, requiring 2 edges for cycle detection
    assert!(result.issues.is_empty());
}

// ==================== Detector Trait Tests ====================

#[test]
fn detector_name_is_cycle() {
    let detector = CycleDetector;
    assert_eq!(detector.name(), "cycle");
}

#[test]
fn detector_is_not_skippable() {
    let detector = CycleDetector;
    assert!(!detector.is_skippable());
}

#[test]
fn detector_stats_are_populated() {
    let ops = vec![remap(KeyCode::A, KeyCode::B), remap(KeyCode::B, KeyCode::A)];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);

    assert_eq!(result.stats.operations_checked, 2);
    assert_eq!(result.stats.issues_found, 1);
    assert!(result.stats.duration.as_micros() > 0);
}

// ==================== Complex Edge Cases ====================

#[test]
fn detects_overlapping_cycles() {
    // Two cycles that share a common edge: A→B→A and B→C→B
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        remap(KeyCode::B, KeyCode::A),
        remap(KeyCode::B, KeyCode::C),
        remap(KeyCode::C, KeyCode::B),
    ];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);
    // Should detect both cycles
    assert!(result.issues.len() >= 2);
}

#[test]
fn detects_nested_cycles_with_shared_node() {
    // Create a figure-8 pattern: A→B→C→B and B→D→E→B
    // B is the common node
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        remap(KeyCode::B, KeyCode::C),
        remap(KeyCode::C, KeyCode::B),
        remap(KeyCode::B, KeyCode::D),
        remap(KeyCode::D, KeyCode::E),
        remap(KeyCode::E, KeyCode::B),
    ];
    let detector = CycleDetector;
    let mut config = ValidationConfig::default();
    config.max_cycle_depth = 10;
    let ctx = make_ctx(config);
    let result = detector.detect(&ops, &ctx);
    // Should detect multiple cycles
    assert!(result.issues.len() >= 2);
}

#[test]
fn detects_cycle_with_diverging_paths() {
    // A→B→C→A (cycle) and B→D (diverging path)
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        remap(KeyCode::B, KeyCode::C),
        remap(KeyCode::C, KeyCode::A),
        remap(KeyCode::B, KeyCode::D),
    ];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);
    // Should detect the cycle but not be confused by the diverging path
    assert_eq!(result.issues.len(), 1);
    assert!(result.issues[0].message.contains("A"));
    assert!(result.issues[0].message.contains("B"));
    assert!(result.issues[0].message.contains("C"));
}

#[test]
fn detects_five_key_cycle() {
    // Test a longer cycle: A→B→C→D→E→A
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        remap(KeyCode::B, KeyCode::C),
        remap(KeyCode::C, KeyCode::D),
        remap(KeyCode::D, KeyCode::E),
        remap(KeyCode::E, KeyCode::A),
    ];
    let detector = CycleDetector;
    let mut config = ValidationConfig::default();
    config.max_cycle_depth = 10;
    let ctx = make_ctx(config);
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
}

#[test]
fn detects_six_key_cycle() {
    // Test an even longer cycle: A→B→C→D→E→F→A
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        remap(KeyCode::B, KeyCode::C),
        remap(KeyCode::C, KeyCode::D),
        remap(KeyCode::D, KeyCode::E),
        remap(KeyCode::E, KeyCode::F),
        remap(KeyCode::F, KeyCode::A),
    ];
    let detector = CycleDetector;
    let mut config = ValidationConfig::default();
    config.max_cycle_depth = 10;
    let ctx = make_ctx(config);
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
}

#[test]
fn multiple_outgoing_edges_no_false_positive() {
    // A→B, A→C, B→D, C→D (diamond pattern, no cycles)
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        remap(KeyCode::A, KeyCode::C),
        remap(KeyCode::B, KeyCode::D),
        remap(KeyCode::C, KeyCode::D),
    ];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);
    assert!(result.issues.is_empty());
}

#[test]
fn complex_graph_with_single_cycle() {
    // Complex graph with only one cycle: A→B→C→A and other non-cyclic paths
    let ops = vec![
        remap(KeyCode::A, KeyCode::B),
        remap(KeyCode::B, KeyCode::C),
        remap(KeyCode::C, KeyCode::A), // cycle
        remap(KeyCode::D, KeyCode::E),
        remap(KeyCode::E, KeyCode::F),
        remap(KeyCode::F, KeyCode::G), // no cycle
    ];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 1);
}

#[test]
fn interleaved_cycles() {
    // Operations for two cycles are interleaved
    let ops = vec![
        remap(KeyCode::A, KeyCode::B), // cycle 1
        remap(KeyCode::C, KeyCode::D), // cycle 2
        remap(KeyCode::B, KeyCode::A), // cycle 1
        remap(KeyCode::D, KeyCode::C), // cycle 2
    ];
    let detector = CycleDetector;
    let ctx = make_ctx(default_config());
    let result = detector.detect(&ops, &ctx);
    assert_eq!(result.issues.len(), 2);
}
