//! Conflict detection for remaps and combos.
//!
//! Detects duplicate remaps, remap-block conflicts, and tap-hold conflicts.

use std::collections::HashMap;
use std::time::Instant;

use crate::drivers::keycodes::KeyCode;
use crate::scripting::PendingOp;
use crate::validation::types::SourceLocation;

use super::{Detector, DetectorContext, DetectorResult, DetectorStats, ValidationIssue};

/// Index of an operation in the pending ops list (for location tracking).
type OpIndex = usize;

/// Information about a key operation for conflict detection.
#[derive(Debug, Clone)]
struct KeyOp {
    /// Index in the original operations list.
    index: OpIndex,
    /// Type of operation on this key.
    op_type: KeyOpType,
}

/// Types of operations that can be performed on a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyOpType {
    Remap,
    Block,
    Pass,
    TapHold,
}

/// Detects conflicts between key operations.
///
/// This detector finds issues where the same key has multiple conflicting operations
/// defined, such as:
/// - Duplicate remaps (same key remapped multiple times)
/// - Remap + block conflicts (key is both remapped and blocked)
/// - Tap-hold conflicts with simple remaps
///
/// The detector uses an O(n) algorithm by building a map of keys to their operations,
/// then checking each key with multiple operations for conflicts.
pub struct ConflictDetector;

impl ConflictDetector {
    /// Creates a new ConflictDetector.
    pub fn new() -> Self {
        Self
    }

    /// Detect all remap-related conflicts in the operations list.
    ///
    /// Returns issues for:
    /// - Duplicate remaps (same key remapped multiple times)
    /// - Remap + block conflicts (key is both remapped and blocked)
    /// - Tap-hold conflicts with simple remaps
    fn detect_conflicts(&self, ops: &[PendingOp]) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let mut key_ops: HashMap<KeyCode, Vec<KeyOp>> = HashMap::new();

        // Collect all key operations - O(n)
        for (index, op) in ops.iter().enumerate() {
            match op {
                PendingOp::Remap { from, .. } => {
                    key_ops.entry(*from).or_default().push(KeyOp {
                        index,
                        op_type: KeyOpType::Remap,
                    });
                }
                PendingOp::Block { key } => {
                    key_ops.entry(*key).or_default().push(KeyOp {
                        index,
                        op_type: KeyOpType::Block,
                    });
                }
                PendingOp::Pass { key } => {
                    key_ops.entry(*key).or_default().push(KeyOp {
                        index,
                        op_type: KeyOpType::Pass,
                    });
                }
                PendingOp::TapHold { key, .. } => {
                    key_ops.entry(*key).or_default().push(KeyOp {
                        index,
                        op_type: KeyOpType::TapHold,
                    });
                }
                _ => {}
            }
        }

        // Check for conflicts on each key - O(k*m^2) where k is number of keys with conflicts
        // and m is operations per key (typically small)
        for (key, operations) in key_ops {
            if operations.len() > 1 {
                issues.extend(Self::check_key_conflicts(key, &operations));
            }
        }

        issues
    }

    /// Check conflicts for a single key with multiple operations.
    fn check_key_conflicts(key: KeyCode, operations: &[KeyOp]) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Compare each pair of operations
        for i in 0..operations.len() {
            for j in (i + 1)..operations.len() {
                let first = &operations[i];
                let second = &operations[j];

                let issue = Self::create_conflict_issue(key, first, second);
                issues.push(issue);
            }
        }

        issues
    }

    /// Create an issue for a specific conflict between two operations.
    fn create_conflict_issue(key: KeyCode, first: &KeyOp, second: &KeyOp) -> ValidationIssue {
        let key_name = key.name();
        let (code, message) = Self::conflict_message(&key_name, first, second);

        // All conflicts are warnings (not errors) since they're behavioral issues
        ValidationIssue::warning(format!("conflict:{}", code), message)
            .with_location(SourceLocation::new(second.index + 1))
    }

    /// Generate the appropriate warning code and message for a conflict.
    fn conflict_message(key_name: &str, first: &KeyOp, second: &KeyOp) -> (&'static str, String) {
        match (first.op_type, second.op_type) {
            // Same operation type - duplicate
            (KeyOpType::Remap, KeyOpType::Remap) => (
                "duplicate-remap",
                format!(
                    "Key '{}' remapped multiple times: first at operation {}, overridden at {}",
                    key_name,
                    first.index + 1,
                    second.index + 1
                ),
            ),
            (KeyOpType::Block, KeyOpType::Block) => (
                "duplicate-block",
                format!(
                    "Key '{}' blocked multiple times: at operations {} and {}",
                    key_name,
                    first.index + 1,
                    second.index + 1
                ),
            ),
            (KeyOpType::TapHold, KeyOpType::TapHold) => (
                "duplicate-taphold",
                format!(
                    "Key '{}' has multiple tap-hold definitions: first at {}, overridden at {}",
                    key_name,
                    first.index + 1,
                    second.index + 1
                ),
            ),

            // Remap + Block conflict
            (KeyOpType::Remap, KeyOpType::Block) | (KeyOpType::Block, KeyOpType::Remap) => {
                let (remap_idx, block_idx) = if first.op_type == KeyOpType::Remap {
                    (first.index, second.index)
                } else {
                    (second.index, first.index)
                };
                (
                    "remap-block",
                    format!(
                        "Key '{}' is both remapped (at {}) and blocked (at {}): block takes precedence",
                        key_name,
                        remap_idx + 1,
                        block_idx + 1
                    ),
                )
            }

            // Tap-hold + simple operation conflicts
            (KeyOpType::TapHold, KeyOpType::Remap) | (KeyOpType::Remap, KeyOpType::TapHold) => {
                let (th_idx, remap_idx) = if first.op_type == KeyOpType::TapHold {
                    (first.index, second.index)
                } else {
                    (second.index, first.index)
                };
                (
                    "taphold-remap",
                    format!(
                        "Key '{}' has both tap-hold (at {}) and remap (at {}): later definition wins",
                        key_name,
                        th_idx + 1,
                        remap_idx + 1
                    ),
                )
            }

            (KeyOpType::TapHold, KeyOpType::Block) | (KeyOpType::Block, KeyOpType::TapHold) => {
                let (th_idx, block_idx) = if first.op_type == KeyOpType::TapHold {
                    (first.index, second.index)
                } else {
                    (second.index, first.index)
                };
                (
                    "taphold-block",
                    format!(
                        "Key '{}' has both tap-hold (at {}) and block (at {}): block takes precedence",
                        key_name,
                        th_idx + 1,
                        block_idx + 1
                    ),
                )
            }

            // Pass conflicts (usually intentional resets)
            (KeyOpType::Pass, _) | (_, KeyOpType::Pass) => (
                "pass-conflict",
                format!(
                    "Key '{}' has multiple operations including pass: at {} and {}",
                    key_name,
                    first.index + 1,
                    second.index + 1
                ),
            ),
        }
    }
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for ConflictDetector {
    fn name(&self) -> &'static str {
        "conflict"
    }

    fn detect(&self, ops: &[PendingOp], _ctx: &DetectorContext) -> DetectorResult {
        let start = Instant::now();
        let issues = self.detect_conflicts(ops);
        let duration = start.elapsed();

        let stats = DetectorStats::new(ops.len(), issues.len(), duration);
        DetectorResult::with_stats(issues, stats)
    }

    fn is_skippable(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::HoldAction;
    use crate::validation::config::ValidationConfig;
    use crate::validation::detectors::Severity;

    fn detect(ops: &[PendingOp]) -> Vec<ValidationIssue> {
        let detector = ConflictDetector::new();
        let ctx = DetectorContext::new(ValidationConfig::default());
        detector.detect(ops, &ctx).issues
    }

    #[test]
    fn no_conflicts_in_empty_ops() {
        let issues = detect(&[]);
        assert!(issues.is_empty());
    }

    #[test]
    fn no_conflicts_for_different_keys() {
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::C,
                to: KeyCode::D,
            },
        ];
        let issues = detect(&ops);
        assert!(issues.is_empty());
    }

    #[test]
    fn detects_duplicate_remap() {
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::C,
            },
        ];
        let issues = detect(&ops);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].detector, "conflict:duplicate-remap");
        assert!(issues[0].message.contains("remapped multiple times"));
    }

    #[test]
    fn detects_remap_block_conflict() {
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Block { key: KeyCode::A },
        ];
        let issues = detect(&ops);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].detector, "conflict:remap-block");
        assert!(issues[0].message.contains("both remapped"));
        assert!(issues[0].message.contains("blocked"));
    }

    #[test]
    fn detects_block_remap_conflict_reversed() {
        let ops = vec![
            PendingOp::Block { key: KeyCode::A },
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
        ];
        let issues = detect(&ops);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].detector, "conflict:remap-block");
    }

    #[test]
    fn detects_tap_hold_remap_conflict() {
        let ops = vec![
            PendingOp::TapHold {
                key: KeyCode::CapsLock,
                tap: KeyCode::Escape,
                hold: HoldAction::Key(KeyCode::LeftCtrl),
            },
            PendingOp::Remap {
                from: KeyCode::CapsLock,
                to: KeyCode::LeftCtrl,
            },
        ];
        let issues = detect(&ops);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].detector, "conflict:taphold-remap");
        assert!(issues[0].message.contains("tap-hold"));
        assert!(issues[0].message.contains("remap"));
    }

    #[test]
    fn detects_tap_hold_block_conflict() {
        let ops = vec![
            PendingOp::TapHold {
                key: KeyCode::CapsLock,
                tap: KeyCode::Escape,
                hold: HoldAction::Key(KeyCode::LeftCtrl),
            },
            PendingOp::Block {
                key: KeyCode::CapsLock,
            },
        ];
        let issues = detect(&ops);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].detector, "conflict:taphold-block");
    }

    #[test]
    fn detects_duplicate_tap_hold() {
        let ops = vec![
            PendingOp::TapHold {
                key: KeyCode::CapsLock,
                tap: KeyCode::Escape,
                hold: HoldAction::Key(KeyCode::LeftCtrl),
            },
            PendingOp::TapHold {
                key: KeyCode::CapsLock,
                tap: KeyCode::Tab,
                hold: HoldAction::Key(KeyCode::LeftAlt),
            },
        ];
        let issues = detect(&ops);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].detector, "conflict:duplicate-taphold");
    }

    #[test]
    fn detects_multiple_conflicts_for_same_key() {
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::C,
            },
            PendingOp::Block { key: KeyCode::A },
        ];
        let issues = detect(&ops);
        // 3 operations = 3 pairs: (0,1), (0,2), (1,2)
        assert_eq!(issues.len(), 3);
    }

    #[test]
    fn issue_has_correct_location() {
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Block { key: KeyCode::C },
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::D,
            },
        ];
        let issues = detect(&ops);
        assert_eq!(issues.len(), 1);
        // Location should point to the second conflicting operation (index 2 -> line 3)
        assert_eq!(issues[0].locations[0].line, 3);
    }

    #[test]
    fn pass_operations_generate_warnings() {
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Pass { key: KeyCode::A },
        ];
        let issues = detect(&ops);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].detector, "conflict:pass-conflict");
    }

    #[test]
    fn issue_severity_is_warning() {
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::C,
            },
        ];
        let issues = detect(&ops);
        assert_eq!(issues[0].severity, Severity::Warning);
    }

    #[test]
    fn detector_name_is_conflict() {
        let detector = ConflictDetector::new();
        assert_eq!(detector.name(), "conflict");
    }

    #[test]
    fn detector_is_not_skippable() {
        let detector = ConflictDetector::new();
        assert!(!detector.is_skippable());
    }

    #[test]
    fn detector_returns_stats() {
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::C,
            },
        ];
        let detector = ConflictDetector::new();
        let ctx = DetectorContext::new(ValidationConfig::default());
        let result = detector.detect(&ops, &ctx);

        assert_eq!(result.stats.operations_checked, 2);
        assert_eq!(result.stats.issues_found, 1);
        assert!(result.stats.duration.as_micros() > 0);
    }

    #[test]
    fn duplicate_block_is_detected() {
        let ops = vec![
            PendingOp::Block { key: KeyCode::A },
            PendingOp::Block { key: KeyCode::A },
        ];
        let issues = detect(&ops);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].detector, "conflict:duplicate-block");
    }

    #[test]
    fn tap_hold_on_different_keys_no_conflict() {
        let ops = vec![
            PendingOp::TapHold {
                key: KeyCode::A,
                tap: KeyCode::B,
                hold: HoldAction::Key(KeyCode::C),
            },
            PendingOp::TapHold {
                key: KeyCode::D,
                tap: KeyCode::E,
                hold: HoldAction::Key(KeyCode::F),
            },
        ];
        let issues = detect(&ops);
        assert!(issues.is_empty());
    }

    #[test]
    fn remap_and_tap_hold_on_different_keys_no_conflict() {
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::TapHold {
                key: KeyCode::C,
                tap: KeyCode::D,
                hold: HoldAction::Key(KeyCode::E),
            },
        ];
        let issues = detect(&ops);
        assert!(issues.is_empty());
    }

    #[test]
    fn layer_map_ops_not_tracked_in_remap_conflicts() {
        // LayerMap operations should not be considered for remap conflict detection
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::LayerMap {
                layer: "nav".to_string(),
                key: KeyCode::A,
                action: crate::scripting::LayerMapAction::Remap(KeyCode::C),
            },
        ];
        let issues = detect(&ops);
        // LayerMap on key A should not conflict with base Remap on key A
        assert!(issues.is_empty());
    }

    #[test]
    fn warning_message_contains_operation_indices() {
        let ops = vec![
            PendingOp::Block { key: KeyCode::X },
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Block { key: KeyCode::Y },
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::C,
            },
        ];
        let issues = detect(&ops);
        assert_eq!(issues.len(), 1);
        // Message should contain the 1-indexed operation numbers
        assert!(issues[0].message.contains("2") && issues[0].message.contains("4"));
    }

    #[test]
    fn same_key_to_different_targets_creates_conflict() {
        // A→B, A→C - same source, different targets
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::C,
            },
        ];
        let issues = detect(&ops);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].detector, "conflict:duplicate-remap");
    }
}
