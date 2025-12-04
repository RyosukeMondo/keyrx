//! Combo shadowing detection.
//!
//! Detects when one combo's keys are a subset of another combo's keys,
//! causing the shorter combo to trigger before the longer one can complete.

use std::collections::HashSet;

use crate::drivers::keycodes::KeyCode;
use crate::scripting::PendingOp;

use super::{Detector, DetectorContext, DetectorResult, ValidationIssue};

/// Index of an operation in the pending ops list (for location tracking).
type OpIndex = usize;

/// Information about a combo for shadowing detection.
#[derive(Debug, Clone)]
struct ComboInfo {
    /// Index in the original operations list.
    index: OpIndex,
    /// Keys in the combo as a set for subset comparison.
    keys: HashSet<KeyCode>,
    /// Keys as vector for display (preserves original order).
    keys_display: Vec<KeyCode>,
}

/// Detector for combo shadowing issues.
///
/// When combo A's keys are a subset of combo B's keys, combo A will always
/// trigger before combo B can be completed, effectively shadowing it.
///
/// Example: [A, S] shadows [A, S, D] because pressing A+S+D will trigger
/// the A+S combo before D is pressed.
///
/// This detector is marked as skippable because shadowing detection can be
/// expensive for configurations with many combos, and is often less critical
/// than conflict detection.
pub struct ShadowingDetector;

impl ShadowingDetector {
    /// Creates a new shadowing detector.
    pub fn new() -> Self {
        Self
    }

    /// Detect combo shadowing where one combo's keys are a subset of another.
    fn detect_shadowing(ops: &[PendingOp]) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Collect all combo operations
        let combos: Vec<ComboInfo> = ops
            .iter()
            .enumerate()
            .filter_map(|(index, op)| {
                if let PendingOp::Combo { keys, .. } = op {
                    Some(ComboInfo {
                        index,
                        keys: keys.iter().copied().collect(),
                        keys_display: keys.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        // Check each pair of combos for subset relationships
        for i in 0..combos.len() {
            for j in (i + 1)..combos.len() {
                let combo_a = &combos[i];
                let combo_b = &combos[j];

                // Check if one is a proper subset of the other
                if Self::is_proper_subset(&combo_a.keys, &combo_b.keys) {
                    // combo_a shadows combo_b
                    issues.push(Self::create_shadowing_issue(combo_a, combo_b));
                } else if Self::is_proper_subset(&combo_b.keys, &combo_a.keys) {
                    // combo_b shadows combo_a
                    issues.push(Self::create_shadowing_issue(combo_b, combo_a));
                }
            }
        }

        issues
    }

    /// Check if `smaller` is a proper subset of `larger`.
    fn is_proper_subset(smaller: &HashSet<KeyCode>, larger: &HashSet<KeyCode>) -> bool {
        smaller.len() < larger.len() && smaller.is_subset(larger)
    }

    /// Create an issue for combo shadowing.
    fn create_shadowing_issue(shorter: &ComboInfo, longer: &ComboInfo) -> ValidationIssue {
        let shorter_keys: Vec<String> = shorter
            .keys_display
            .iter()
            .map(|k| k.name().to_string())
            .collect();
        let longer_keys: Vec<String> = longer
            .keys_display
            .iter()
            .map(|k| k.name().to_string())
            .collect();

        ValidationIssue::warning(
            "shadowing",
            format!(
                "Combo [{}] (at {}) shadows combo [{}] (at {}): shorter combo triggers first",
                shorter_keys.join("+"),
                shorter.index + 1,
                longer_keys.join("+"),
                longer.index + 1
            ),
        )
        .with_location(crate::validation::types::SourceLocation::new(
            longer.index + 1,
        ))
    }
}

impl Default for ShadowingDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for ShadowingDetector {
    fn name(&self) -> &'static str {
        "shadowing"
    }

    fn detect(&self, ops: &[PendingOp], _ctx: &DetectorContext) -> DetectorResult {
        let start = std::time::Instant::now();
        let issues = Self::detect_shadowing(ops);
        let duration = start.elapsed();

        DetectorResult::with_stats(
            issues.clone(),
            super::DetectorStats::new(ops.len(), issues.len(), duration),
        )
    }

    fn is_skippable(&self) -> bool {
        true
    }
}
