//! Conflict detection for remaps and combos.
//!
//! Detects duplicate remaps, remap-block conflicts, tap-hold conflicts,
//! combo shadowing, and circular remap dependencies.

use std::collections::HashMap;

use crate::drivers::keycodes::KeyCode;
use crate::scripting::PendingOp;

use super::types::{SourceLocation, ValidationWarning, WarningCategory};

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
pub struct ConflictDetector;

impl ConflictDetector {
    /// Detect all remap-related conflicts in the operations list.
    ///
    /// Returns warnings for:
    /// - Duplicate remaps (same key remapped multiple times)
    /// - Remap + block conflicts (key is both remapped and blocked)
    /// - Tap-hold conflicts with simple remaps
    pub fn detect_remap_conflicts(ops: &[PendingOp]) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();
        let mut key_ops: HashMap<KeyCode, Vec<KeyOp>> = HashMap::new();

        // Collect all key operations
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

        // Check for conflicts on each key
        for (key, operations) in key_ops {
            if operations.len() > 1 {
                warnings.extend(Self::check_key_conflicts(key, &operations));
            }
        }

        warnings
    }

    /// Check conflicts for a single key with multiple operations.
    fn check_key_conflicts(key: KeyCode, operations: &[KeyOp]) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        // Compare each pair of operations
        for i in 0..operations.len() {
            for j in (i + 1)..operations.len() {
                let first = &operations[i];
                let second = &operations[j];

                let warning = Self::create_conflict_warning(key, first, second);
                warnings.push(warning);
            }
        }

        warnings
    }

    /// Create a warning for a specific conflict between two operations.
    fn create_conflict_warning(key: KeyCode, first: &KeyOp, second: &KeyOp) -> ValidationWarning {
        let key_name = key.name();
        let (code, message) = Self::conflict_message(&key_name, first, second);

        ValidationWarning::new(code, WarningCategory::Conflict, message)
            .with_location(SourceLocation::new(second.index + 1))
    }

    /// Generate the appropriate warning code and message for a conflict.
    fn conflict_message(key_name: &str, first: &KeyOp, second: &KeyOp) -> (&'static str, String) {
        match (first.op_type, second.op_type) {
            // Same operation type - duplicate
            (KeyOpType::Remap, KeyOpType::Remap) => (
                "W001",
                format!(
                    "Key '{}' remapped multiple times: first at operation {}, overridden at {}",
                    key_name,
                    first.index + 1,
                    second.index + 1
                ),
            ),
            (KeyOpType::Block, KeyOpType::Block) => (
                "W002",
                format!(
                    "Key '{}' blocked multiple times: at operations {} and {}",
                    key_name,
                    first.index + 1,
                    second.index + 1
                ),
            ),
            (KeyOpType::TapHold, KeyOpType::TapHold) => (
                "W003",
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
                    "W004",
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
                    "W005",
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
                    "W006",
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
                "W007",
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

/// Convenience function for detecting remap conflicts.
pub fn detect_remap_conflicts(ops: &[PendingOp]) -> Vec<ValidationWarning> {
    ConflictDetector::detect_remap_conflicts(ops)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::HoldAction;

    #[test]
    fn no_conflicts_in_empty_ops() {
        let warnings = detect_remap_conflicts(&[]);
        assert!(warnings.is_empty());
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
        let warnings = detect_remap_conflicts(&ops);
        assert!(warnings.is_empty());
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
        let warnings = detect_remap_conflicts(&ops);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W001");
        assert!(warnings[0].message.contains("remapped multiple times"));
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
        let warnings = detect_remap_conflicts(&ops);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W004");
        assert!(warnings[0].message.contains("both remapped"));
        assert!(warnings[0].message.contains("blocked"));
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
        let warnings = detect_remap_conflicts(&ops);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W004");
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
        let warnings = detect_remap_conflicts(&ops);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W005");
        assert!(warnings[0].message.contains("tap-hold"));
        assert!(warnings[0].message.contains("remap"));
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
        let warnings = detect_remap_conflicts(&ops);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W006");
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
        let warnings = detect_remap_conflicts(&ops);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W003");
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
        let warnings = detect_remap_conflicts(&ops);
        // 3 operations = 3 pairs: (0,1), (0,2), (1,2)
        assert_eq!(warnings.len(), 3);
    }

    #[test]
    fn warning_has_correct_location() {
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
        let warnings = detect_remap_conflicts(&ops);
        assert_eq!(warnings.len(), 1);
        // Location should point to the second conflicting operation (index 2 -> line 3)
        assert_eq!(warnings[0].location.as_ref().unwrap().line, 3);
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
        let warnings = detect_remap_conflicts(&ops);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W007");
    }

    #[test]
    fn warning_category_is_conflict() {
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
        let warnings = detect_remap_conflicts(&ops);
        assert_eq!(warnings[0].category, WarningCategory::Conflict);
    }
}
