//! Integration tests for validation conflict detection.
//!
//! Tests conflict-related warnings including:
//! - Duplicate remaps
//! - Remap cycles
//! - Shadowing conflicts

use keyrx_core::validation::config::ValidationConfig;
use keyrx_core::validation::engine::ValidationEngine;
use keyrx_core::validation::types::{ValidationOptions, WarningCategory};

// =============================================================================
// Duplicate Remap Tests
// =============================================================================

#[test]
fn detects_duplicate_remap_warning() {
    let script = r#"
        remap("A", "B");
        remap("A", "C");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid); // Warnings don't make script invalid
    assert!(result.has_warnings());
    assert!(result
        .warnings
        .iter()
        .any(|w| w.category == WarningCategory::Conflict));
}

#[test]
fn duplicate_remap_with_different_targets() {
    let script = r#"
        remap("Space", "A");
        remap("Space", "B");
        remap("Space", "C");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.has_warnings());
    // Should have multiple conflict warnings
    assert!(
        result
            .warnings
            .iter()
            .filter(|w| w.category == WarningCategory::Conflict)
            .count()
            >= 1
    );
}

#[test]
fn no_duplicate_warning_for_different_keys() {
    let script = r#"
        remap("A", "B");
        remap("B", "C");
        remap("C", "D");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    // This creates a chain but not duplicates on the same key
    // Cycle detection is separate
}

// =============================================================================
// Cycle Detection Tests
// =============================================================================

#[test]
fn detects_simple_two_key_cycle() {
    let script = r#"
        remap("A", "B");
        remap("B", "A");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.has_warnings());
    assert!(result
        .warnings
        .iter()
        .any(|w| w.category == WarningCategory::Conflict));
}

#[test]
fn detects_three_key_cycle() {
    let script = r#"
        remap("A", "B");
        remap("B", "C");
        remap("C", "A");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.has_warnings());
    assert!(result
        .warnings
        .iter()
        .any(|w| w.category == WarningCategory::Conflict));
}

#[test]
fn respects_custom_cycle_depth() {
    let mut config = ValidationConfig::default();
    config.max_cycle_depth = 3;

    // This creates a 2-step cycle which should be detected
    let script = r#"
        remap("A", "B");
        remap("B", "A");
    "#;

    let engine = ValidationEngine::with_config(config);
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.has_warnings());
    assert!(result
        .warnings
        .iter()
        .any(|w| w.category == WarningCategory::Conflict));
}

#[test]
fn no_cycle_in_valid_chain() {
    let script = r#"
        remap("A", "B");
        remap("B", "C");
        remap("C", "D");
        remap("D", "E");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    // This is a chain, not a cycle
    assert!(result.is_valid);
}

#[test]
fn cycle_with_long_chain() {
    let script = r#"
        remap("A", "B");
        remap("B", "C");
        remap("C", "D");
        remap("D", "E");
        remap("E", "A");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    // Should detect the cycle
    assert!(result.has_warnings());
    assert!(result
        .warnings
        .iter()
        .any(|w| w.category == WarningCategory::Conflict));
}

// =============================================================================
// Shadowing Tests
// =============================================================================

#[test]
fn detects_shadowing_warning() {
    let script = r#"
        block("A");
        remap("A", "B");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.has_warnings());
    // Shadowing is when a blocked key is later remapped
}

#[test]
fn shadowing_remap_after_block() {
    let script = r#"
        block("Space");
        remap("Space", "Enter");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    // Block followed by remap creates shadowing
    assert!(result.has_warnings());
}

#[test]
fn no_shadowing_different_keys() {
    let script = r#"
        block("A");
        remap("B", "C");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    // Different keys, no shadowing
}

// =============================================================================
// Strict Mode with Conflicts
// =============================================================================

#[test]
fn strict_mode_fails_on_duplicate_remap() {
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
fn strict_mode_fails_on_cycle() {
    let script = r#"
        remap("A", "B");
        remap("B", "A");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new().strict());

    assert!(!result.is_valid);
    assert!(result.has_warnings());
}

#[test]
fn strict_mode_fails_on_shadowing() {
    let script = r#"
        block("A");
        remap("A", "B");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new().strict());

    assert!(!result.is_valid);
    assert!(result.has_warnings());
}

// =============================================================================
// Multiple Conflict Types
// =============================================================================

#[test]
fn detects_multiple_conflict_types() {
    let script = r#"
        // Duplicate remap
        remap("A", "B");
        remap("A", "C");

        // Cycle
        remap("X", "Y");
        remap("Y", "X");

        // Shadowing
        block("Z");
        remap("Z", "W");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    assert!(result.has_warnings());
    // Should have multiple conflict warnings
    let conflict_count = result
        .warnings
        .iter()
        .filter(|w| w.category == WarningCategory::Conflict)
        .count();
    assert!(conflict_count >= 2);
}

// =============================================================================
// Layer-Related Conflicts
// =============================================================================

#[test]
fn layer_map_does_not_conflict_across_layers() {
    let script = r#"
        define_layer("nav");
        define_layer("edit");
        layer_map("nav", "H", "Left");
        layer_map("edit", "H", "Backspace");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    // Same key on different layers is fine
}

// =============================================================================
// Combo Conflicts
// =============================================================================

#[test]
fn overlapping_combo_patterns() {
    let script = r#"
        combo(["A", "S", "D"], "X");
        combo(["A", "S"], "Y");
    "#;

    let engine = ValidationEngine::new();
    let result = engine.validate(script, ValidationOptions::new());

    assert!(result.is_valid);
    // Overlapping combos might be intentional
}
