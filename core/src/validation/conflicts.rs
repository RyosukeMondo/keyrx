//! Conflict detection for remaps and combos.
//!
//! This module provides backward-compatible exports for conflict detection functions.
//! The actual implementation has been moved to specialized detector modules.
//!
//! # Migration
//!
//! This module now re-exports from:
//! - `detectors::conflicts` - Remap/block conflict detection
//! - `detectors::shadowing` - Combo shadowing detection
//! - `detectors::cycles` - Circular remap detection
//!
//! New code should use the `DetectorOrchestrator` for running all detectors,
//! or import individual detectors directly from `validation::detectors::*`.

use crate::scripting::PendingOp;

use super::config::ValidationConfig;
use super::detectors::conflicts::ConflictDetector as NewConflictDetector;
use super::detectors::cycles::CycleDetector;
use super::detectors::shadowing::ShadowingDetector;
use super::detectors::{Detector, DetectorContext};
use super::types::ValidationWarning;

/// Detects conflicts between key operations.
///
/// This is a re-export of the new detector for backward compatibility.
/// Prefer using the `DetectorOrchestrator` or importing from `detectors::conflicts`.
pub struct ConflictDetector;

impl ConflictDetector {
    /// Detect all remap-related conflicts in the operations list.
    ///
    /// Returns warnings for:
    /// - Duplicate remaps (same key remapped multiple times)
    /// - Remap + block conflicts (key is both remapped and blocked)
    /// - Tap-hold conflicts with simple remaps
    pub fn detect_remap_conflicts(ops: &[PendingOp]) -> Vec<ValidationWarning> {
        let detector = NewConflictDetector;
        let ctx = DetectorContext::new(ValidationConfig::default());
        let result = detector.detect(ops, &ctx);

        // Convert new ValidationIssue format to old ValidationWarning format
        result
            .issues
            .iter()
            .map(|issue| issue.to_warning())
            .collect()
    }
}

/// Convenience function for detecting remap conflicts.
///
/// This function provides backward compatibility with the old API.
/// Consider using `DetectorOrchestrator` for new code.
pub fn detect_remap_conflicts(ops: &[PendingOp]) -> Vec<ValidationWarning> {
    ConflictDetector::detect_remap_conflicts(ops)
}

/// Detect combo shadowing where one combo's keys are a subset of another.
///
/// When combo A's keys are a subset of combo B's keys, combo A will always
/// trigger before combo B can be completed, effectively shadowing it.
///
/// Example: [A, S] shadows [A, S, D] because pressing A+S+D will trigger
/// the A+S combo before D is pressed.
///
/// This function provides backward compatibility with the old API.
/// Consider using `DetectorOrchestrator` for new code.
pub fn detect_combo_shadowing(ops: &[PendingOp]) -> Vec<ValidationWarning> {
    let detector = ShadowingDetector;
    let ctx = DetectorContext::new(ValidationConfig::default());
    let result = detector.detect(ops, &ctx);

    // Convert new ValidationIssue format to old ValidationWarning format
    result
        .issues
        .iter()
        .map(|issue| issue.to_warning())
        .collect()
}

/// Detect circular remap dependencies (A→B→C→A).
///
/// Circular remaps can cause unpredictable behavior where keys effectively
/// swap or create feedback loops. Uses DFS to find cycles up to config.max_cycle_depth.
///
/// This function provides backward compatibility with the old API.
/// Consider using `DetectorOrchestrator` for new code.
pub fn detect_circular_remaps(
    ops: &[PendingOp],
    config: &ValidationConfig,
) -> Vec<ValidationWarning> {
    let detector = CycleDetector;
    let ctx = DetectorContext::new(config.clone());
    let result = detector.detect(ops, &ctx);

    // Convert new ValidationIssue format to old ValidationWarning format
    result
        .issues
        .iter()
        .map(|issue| issue.to_warning())
        .collect()
}
