//! Safety analysis for dangerous patterns.
//!
//! Warns about patterns that could lock users out of their keyboard,
//! such as remapping Escape or blocking emergency exit combos.

use std::collections::HashSet;

use crate::drivers::keycodes::KeyCode;
use crate::scripting::PendingOp;

use super::config::ValidationConfig;
use super::types::{SourceLocation, ValidationWarning, WarningCategory};

/// Index of an operation in the pending ops list.
type OpIndex = usize;

/// Left/Right modifier pairs that must not both be blocked.
const MODIFIER_PAIRS: &[(KeyCode, KeyCode, &str)] = &[
    (KeyCode::LeftCtrl, KeyCode::RightCtrl, "Ctrl"),
    (KeyCode::LeftAlt, KeyCode::RightAlt, "Alt"),
    (KeyCode::LeftShift, KeyCode::RightShift, "Shift"),
    (KeyCode::LeftMeta, KeyCode::RightMeta, "Meta/Super"),
];

/// Analyzes script operations for dangerous patterns.
pub struct SafetyAnalyzer;

impl SafetyAnalyzer {
    /// Analyze all safety concerns in the operations list.
    ///
    /// Returns warnings for:
    /// - Escape key remapping or blocking
    /// - Emergency exit combo interference
    /// - Blocking both left and right variants of same modifier
    /// - Too many blocked keys (configurable threshold)
    pub fn analyze_safety(ops: &[PendingOp], config: &ValidationConfig) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        warnings.extend(Self::check_escape_key(ops));
        warnings.extend(Self::check_emergency_combo_keys(ops));
        warnings.extend(Self::check_modifier_pairs(ops));
        warnings.extend(Self::check_blocked_keys_count(ops, config));

        warnings
    }

    /// Check if Escape key is remapped or blocked.
    fn check_escape_key(ops: &[PendingOp]) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        for (index, op) in ops.iter().enumerate() {
            match op {
                PendingOp::Remap { from, .. } if *from == KeyCode::Escape => {
                    warnings.push(Self::escape_warning(
                        "W101",
                        "Escape key is remapped",
                        "Remapping Escape may make it difficult to exit applications or cancel operations",
                        index,
                    ));
                }
                PendingOp::Block { key } if *key == KeyCode::Escape => {
                    warnings.push(Self::escape_warning(
                        "W102",
                        "Escape key is blocked",
                        "Blocking Escape may make it difficult to exit applications or cancel operations",
                        index,
                    ));
                }
                PendingOp::TapHold { key, .. } if *key == KeyCode::Escape => {
                    warnings.push(Self::escape_warning(
                        "W103",
                        "Escape key has tap-hold behavior",
                        "Adding tap-hold to Escape may affect its responsiveness in time-critical situations",
                        index,
                    ));
                }
                _ => {}
            }
        }

        warnings
    }

    /// Create an escape-related safety warning.
    fn escape_warning(
        code: &str,
        summary: &str,
        detail: &str,
        index: OpIndex,
    ) -> ValidationWarning {
        ValidationWarning::new(
            code,
            WarningCategory::Safety,
            format!("{}: {}", summary, detail),
        )
        .with_location(SourceLocation::new(index + 1))
    }

    /// Check if emergency exit combo keys (Ctrl, Alt, Shift, Escape) are affected.
    fn check_emergency_combo_keys(ops: &[PendingOp]) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();
        let mut blocked_modifiers: HashSet<&str> = HashSet::new();
        let mut blocked_escape = false;
        let mut escape_block_index = 0;

        // Collect all blocked emergency combo keys
        for (index, op) in ops.iter().enumerate() {
            if let PendingOp::Block { key } = op {
                match *key {
                    KeyCode::Escape => {
                        blocked_escape = true;
                        escape_block_index = index;
                    }
                    KeyCode::LeftCtrl | KeyCode::RightCtrl => {
                        blocked_modifiers.insert("Ctrl");
                    }
                    KeyCode::LeftAlt | KeyCode::RightAlt => {
                        blocked_modifiers.insert("Alt");
                    }
                    KeyCode::LeftShift | KeyCode::RightShift => {
                        blocked_modifiers.insert("Shift");
                    }
                    _ => {}
                }
            }
        }

        // Check if both variants of any modifier are blocked
        let ctrl_both_blocked = Self::both_blocked(ops, KeyCode::LeftCtrl, KeyCode::RightCtrl);
        let alt_both_blocked = Self::both_blocked(ops, KeyCode::LeftAlt, KeyCode::RightAlt);
        let shift_both_blocked = Self::both_blocked(ops, KeyCode::LeftShift, KeyCode::RightShift);

        // If Escape is blocked and any required modifier pair is fully blocked
        if blocked_escape && (ctrl_both_blocked || alt_both_blocked || shift_both_blocked) {
            let mut blocked_mods = Vec::new();
            if ctrl_both_blocked {
                blocked_mods.push("Ctrl");
            }
            if alt_both_blocked {
                blocked_mods.push("Alt");
            }
            if shift_both_blocked {
                blocked_mods.push("Shift");
            }

            warnings.push(
                ValidationWarning::new(
                    "W104",
                    WarningCategory::Safety,
                    format!(
                        "Emergency exit combo (Ctrl+Alt+Shift+Escape) is blocked: \
                         Escape and {} modifier(s) are all blocked. \
                         This may prevent recovery from broken configurations",
                        blocked_mods.join(", ")
                    ),
                )
                .with_location(SourceLocation::new(escape_block_index + 1)),
            );
        }

        warnings
    }

    /// Check if both left and right variants of a key are blocked.
    fn both_blocked(ops: &[PendingOp], left: KeyCode, right: KeyCode) -> bool {
        let mut left_blocked = false;
        let mut right_blocked = false;

        for op in ops {
            if let PendingOp::Block { key } = op {
                if *key == left {
                    left_blocked = true;
                }
                if *key == right {
                    right_blocked = true;
                }
            }
        }

        left_blocked && right_blocked
    }

    /// Check if both left and right variants of modifier keys are blocked.
    fn check_modifier_pairs(ops: &[PendingOp]) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        for (left, right, name) in MODIFIER_PAIRS {
            if let Some((left_idx, right_idx)) = Self::find_both_blocked(ops, *left, *right) {
                warnings.push(
                    ValidationWarning::new(
                        "W105",
                        WarningCategory::Safety,
                        format!(
                            "Both Left{} and Right{} are blocked (at operations {} and {}): \
                             this disables all {} key functionality",
                            name,
                            name,
                            left_idx + 1,
                            right_idx + 1,
                            name
                        ),
                    )
                    .with_location(SourceLocation::new(right_idx.max(left_idx) + 1)),
                );
            }
        }

        warnings
    }

    /// Find indices where both left and right variants are blocked.
    fn find_both_blocked(
        ops: &[PendingOp],
        left: KeyCode,
        right: KeyCode,
    ) -> Option<(OpIndex, OpIndex)> {
        let mut left_idx = None;
        let mut right_idx = None;

        for (index, op) in ops.iter().enumerate() {
            if let PendingOp::Block { key } = op {
                if *key == left {
                    left_idx = Some(index);
                }
                if *key == right {
                    right_idx = Some(index);
                }
            }
        }

        match (left_idx, right_idx) {
            (Some(l), Some(r)) => Some((l, r)),
            _ => None,
        }
    }

    /// Check if too many keys are blocked.
    fn check_blocked_keys_count(
        ops: &[PendingOp],
        config: &ValidationConfig,
    ) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();
        let blocked_keys: HashSet<KeyCode> = ops
            .iter()
            .filter_map(|op| {
                if let PendingOp::Block { key } = op {
                    Some(*key)
                } else {
                    None
                }
            })
            .collect();

        let count = blocked_keys.len();
        if count >= config.blocked_keys_warning_threshold {
            warnings.push(ValidationWarning::new(
                "W106",
                WarningCategory::Safety,
                format!(
                    "Many keys are blocked ({} keys): \
                     blocking many keys may lead to unexpected behavior. \
                     Consider using remaps instead of blocks where possible",
                    count
                ),
            ));
        }

        warnings
    }
}

/// Convenience function for safety analysis.
pub fn analyze_safety(ops: &[PendingOp], config: &ValidationConfig) -> Vec<ValidationWarning> {
    SafetyAnalyzer::analyze_safety(ops, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{HoldAction, LayerAction};

    fn default_config() -> ValidationConfig {
        ValidationConfig::default()
    }

    #[test]
    fn no_warnings_for_empty_ops() {
        let warnings = analyze_safety(&[], &default_config());
        assert!(warnings.is_empty());
    }

    #[test]
    fn no_warnings_for_safe_remaps() {
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::CapsLock,
                to: KeyCode::LeftCtrl,
            },
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
        ];
        let warnings = analyze_safety(&ops, &default_config());
        assert!(warnings.is_empty());
    }

    #[test]
    fn warns_on_escape_remap() {
        let ops = vec![PendingOp::Remap {
            from: KeyCode::Escape,
            to: KeyCode::CapsLock,
        }];
        let warnings = analyze_safety(&ops, &default_config());
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W101");
        assert!(warnings[0].message.contains("Escape key is remapped"));
        assert_eq!(warnings[0].category, WarningCategory::Safety);
    }

    #[test]
    fn warns_on_escape_block() {
        let ops = vec![PendingOp::Block {
            key: KeyCode::Escape,
        }];
        let warnings = analyze_safety(&ops, &default_config());
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W102");
        assert!(warnings[0].message.contains("Escape key is blocked"));
    }

    #[test]
    fn warns_on_escape_tap_hold() {
        let ops = vec![PendingOp::TapHold {
            key: KeyCode::Escape,
            tap: KeyCode::Escape,
            hold: HoldAction::Key(KeyCode::LeftCtrl),
        }];
        let warnings = analyze_safety(&ops, &default_config());
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, "W103");
        assert!(warnings[0].message.contains("tap-hold"));
    }

    #[test]
    fn warns_on_emergency_combo_blocked() {
        // Block Escape and both Ctrl keys
        let ops = vec![
            PendingOp::Block {
                key: KeyCode::Escape,
            },
            PendingOp::Block {
                key: KeyCode::LeftCtrl,
            },
            PendingOp::Block {
                key: KeyCode::RightCtrl,
            },
        ];
        let warnings = analyze_safety(&ops, &default_config());

        // Should have W102 (Escape blocked) + W104 (emergency combo) + W105 (both Ctrl blocked)
        let emergency_warning = warnings.iter().find(|w| w.code == "W104");
        assert!(emergency_warning.is_some());
        assert!(emergency_warning
            .unwrap()
            .message
            .to_lowercase()
            .contains("emergency exit"));
    }

    #[test]
    fn no_emergency_warning_with_one_ctrl_available() {
        // Block Escape and only left Ctrl
        let ops = vec![
            PendingOp::Block {
                key: KeyCode::Escape,
            },
            PendingOp::Block {
                key: KeyCode::LeftCtrl,
            },
        ];
        let warnings = analyze_safety(&ops, &default_config());

        // Should have W102 (Escape blocked) but NOT W104 (right Ctrl still available)
        let emergency_warning = warnings.iter().find(|w| w.code == "W104");
        assert!(emergency_warning.is_none());
    }

    #[test]
    fn warns_on_both_ctrl_blocked() {
        let ops = vec![
            PendingOp::Block {
                key: KeyCode::LeftCtrl,
            },
            PendingOp::Block {
                key: KeyCode::RightCtrl,
            },
        ];
        let warnings = analyze_safety(&ops, &default_config());
        let modifier_warning = warnings.iter().find(|w| w.code == "W105");
        assert!(modifier_warning.is_some());
        assert!(modifier_warning.unwrap().message.contains("Ctrl"));
    }

    #[test]
    fn warns_on_both_alt_blocked() {
        let ops = vec![
            PendingOp::Block {
                key: KeyCode::LeftAlt,
            },
            PendingOp::Block {
                key: KeyCode::RightAlt,
            },
        ];
        let warnings = analyze_safety(&ops, &default_config());
        let modifier_warning = warnings.iter().find(|w| w.code == "W105");
        assert!(modifier_warning.is_some());
        assert!(modifier_warning.unwrap().message.contains("Alt"));
    }

    #[test]
    fn warns_on_both_shift_blocked() {
        let ops = vec![
            PendingOp::Block {
                key: KeyCode::LeftShift,
            },
            PendingOp::Block {
                key: KeyCode::RightShift,
            },
        ];
        let warnings = analyze_safety(&ops, &default_config());
        let modifier_warning = warnings.iter().find(|w| w.code == "W105");
        assert!(modifier_warning.is_some());
        assert!(modifier_warning.unwrap().message.contains("Shift"));
    }

    #[test]
    fn warns_on_both_meta_blocked() {
        let ops = vec![
            PendingOp::Block {
                key: KeyCode::LeftMeta,
            },
            PendingOp::Block {
                key: KeyCode::RightMeta,
            },
        ];
        let warnings = analyze_safety(&ops, &default_config());
        let modifier_warning = warnings.iter().find(|w| w.code == "W105");
        assert!(modifier_warning.is_some());
        assert!(modifier_warning.unwrap().message.contains("Meta"));
    }

    #[test]
    fn no_warning_with_one_modifier_available() {
        let ops = vec![PendingOp::Block {
            key: KeyCode::LeftCtrl,
        }];
        let warnings = analyze_safety(&ops, &default_config());
        let modifier_warning = warnings.iter().find(|w| w.code == "W105");
        assert!(modifier_warning.is_none());
    }

    #[test]
    fn warns_on_many_blocked_keys() {
        let ops: Vec<PendingOp> = (0..10)
            .map(|i| {
                let key = match i {
                    0 => KeyCode::A,
                    1 => KeyCode::B,
                    2 => KeyCode::C,
                    3 => KeyCode::D,
                    4 => KeyCode::E,
                    5 => KeyCode::F,
                    6 => KeyCode::G,
                    7 => KeyCode::H,
                    8 => KeyCode::I,
                    _ => KeyCode::J,
                };
                PendingOp::Block { key }
            })
            .collect();

        let warnings = analyze_safety(&ops, &default_config());
        let many_blocked = warnings.iter().find(|w| w.code == "W106");
        assert!(many_blocked.is_some());
        assert!(many_blocked.unwrap().message.contains("10 keys"));
    }

    #[test]
    fn no_warning_below_blocked_threshold() {
        let ops: Vec<PendingOp> = (0..5)
            .map(|i| {
                let key = match i {
                    0 => KeyCode::A,
                    1 => KeyCode::B,
                    2 => KeyCode::C,
                    3 => KeyCode::D,
                    _ => KeyCode::E,
                };
                PendingOp::Block { key }
            })
            .collect();

        let warnings = analyze_safety(&ops, &default_config());
        let many_blocked = warnings.iter().find(|w| w.code == "W106");
        assert!(many_blocked.is_none());
    }

    #[test]
    fn respects_config_blocked_threshold() {
        let ops = vec![
            PendingOp::Block { key: KeyCode::A },
            PendingOp::Block { key: KeyCode::B },
            PendingOp::Block { key: KeyCode::C },
        ];

        // With threshold=3, should warn
        let mut config = ValidationConfig::default();
        config.blocked_keys_warning_threshold = 3;
        let warnings = analyze_safety(&ops, &config);
        assert!(warnings.iter().any(|w| w.code == "W106"));

        // With threshold=5, should not warn
        config.blocked_keys_warning_threshold = 5;
        let warnings = analyze_safety(&ops, &config);
        assert!(!warnings.iter().any(|w| w.code == "W106"));
    }

    #[test]
    fn duplicate_blocks_count_once() {
        // Same key blocked twice should only count as 1 blocked key
        let ops = vec![
            PendingOp::Block { key: KeyCode::A },
            PendingOp::Block { key: KeyCode::A },
            PendingOp::Block { key: KeyCode::A },
        ];

        let mut config = ValidationConfig::default();
        config.blocked_keys_warning_threshold = 2;
        let warnings = analyze_safety(&ops, &config);

        // Should not trigger W106 since only 1 unique key is blocked
        let many_blocked = warnings.iter().find(|w| w.code == "W106");
        assert!(many_blocked.is_none());
    }

    #[test]
    fn warning_has_correct_location() {
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Block { key: KeyCode::C },
            PendingOp::Block {
                key: KeyCode::Escape,
            },
        ];
        let warnings = analyze_safety(&ops, &default_config());
        let escape_warning = warnings.iter().find(|w| w.code == "W102").unwrap();
        assert_eq!(escape_warning.location.as_ref().unwrap().line, 3);
    }

    #[test]
    fn all_warnings_have_safety_category() {
        let ops = vec![
            PendingOp::Block {
                key: KeyCode::Escape,
            },
            PendingOp::Block {
                key: KeyCode::LeftCtrl,
            },
            PendingOp::Block {
                key: KeyCode::RightCtrl,
            },
        ];
        let warnings = analyze_safety(&ops, &default_config());
        for warning in &warnings {
            assert_eq!(warning.category, WarningCategory::Safety);
        }
    }

    #[test]
    fn ignores_non_blocking_ops() {
        // Combos that use escape as trigger should not trigger warnings
        let ops = vec![PendingOp::Combo {
            keys: vec![KeyCode::Escape, KeyCode::A],
            action: LayerAction::Block,
        }];
        let warnings = analyze_safety(&ops, &default_config());
        // No escape-specific warnings
        let escape_warnings: Vec<_> = warnings
            .iter()
            .filter(|w| w.code.starts_with("W10"))
            .collect();
        assert!(escape_warnings.is_empty());
    }

    // Additional edge case tests for config-driven behavior and false positive prevention

    #[test]
    fn escape_as_remap_target_no_warning() {
        // Remapping TO Escape should not trigger warnings (only FROM Escape)
        let ops = vec![PendingOp::Remap {
            from: KeyCode::CapsLock,
            to: KeyCode::Escape,
        }];
        let warnings = analyze_safety(&ops, &default_config());
        let escape_warnings: Vec<_> = warnings
            .iter()
            .filter(|w| w.code.starts_with("W10"))
            .collect();
        assert!(escape_warnings.is_empty());
    }

    #[test]
    fn pass_on_escape_no_warning() {
        // Pass on Escape is explicitly allowing it, not a safety concern
        let ops = vec![PendingOp::Pass {
            key: KeyCode::Escape,
        }];
        let warnings = analyze_safety(&ops, &default_config());
        let escape_warnings: Vec<_> = warnings
            .iter()
            .filter(|w| w.code.starts_with("W10"))
            .collect();
        assert!(escape_warnings.is_empty());
    }

    #[test]
    fn layer_map_on_escape_no_warning() {
        // Layer-specific mapping on Escape is not detected by base safety analyzer
        use crate::scripting::LayerMapAction;
        let ops = vec![PendingOp::LayerMap {
            layer: "nav".to_string(),
            key: KeyCode::Escape,
            action: LayerMapAction::Block,
        }];
        let warnings = analyze_safety(&ops, &default_config());
        // LayerMap is not checked for escape warnings (only base operations)
        let escape_warnings: Vec<_> = warnings
            .iter()
            .filter(|w| w.code.starts_with("W10"))
            .collect();
        assert!(escape_warnings.is_empty());
    }

    #[test]
    fn blocked_keys_warning_threshold_exact_boundary() {
        let mut config = ValidationConfig::default();
        config.blocked_keys_warning_threshold = 3;

        // Exactly at threshold should warn
        let ops = vec![
            PendingOp::Block { key: KeyCode::A },
            PendingOp::Block { key: KeyCode::B },
            PendingOp::Block { key: KeyCode::C },
        ];
        let warnings = analyze_safety(&ops, &config);
        assert!(warnings.iter().any(|w| w.code == "W106"));

        // One below threshold should not warn
        let ops = vec![
            PendingOp::Block { key: KeyCode::A },
            PendingOp::Block { key: KeyCode::B },
        ];
        let warnings = analyze_safety(&ops, &config);
        assert!(!warnings.iter().any(|w| w.code == "W106"));
    }

    #[test]
    fn emergency_combo_with_alt_blocked() {
        // Block Escape and both Alt keys - should trigger emergency warning
        let ops = vec![
            PendingOp::Block {
                key: KeyCode::Escape,
            },
            PendingOp::Block {
                key: KeyCode::LeftAlt,
            },
            PendingOp::Block {
                key: KeyCode::RightAlt,
            },
        ];
        let warnings = analyze_safety(&ops, &default_config());
        let emergency_warning = warnings.iter().find(|w| w.code == "W104");
        assert!(emergency_warning.is_some());
        assert!(emergency_warning.unwrap().message.contains("Alt"));
    }

    #[test]
    fn emergency_combo_with_shift_blocked() {
        // Block Escape and both Shift keys - should trigger emergency warning
        let ops = vec![
            PendingOp::Block {
                key: KeyCode::Escape,
            },
            PendingOp::Block {
                key: KeyCode::LeftShift,
            },
            PendingOp::Block {
                key: KeyCode::RightShift,
            },
        ];
        let warnings = analyze_safety(&ops, &default_config());
        let emergency_warning = warnings.iter().find(|w| w.code == "W104");
        assert!(emergency_warning.is_some());
        assert!(emergency_warning.unwrap().message.contains("Shift"));
    }

    #[test]
    fn no_emergency_warning_without_escape_blocked() {
        // Both Ctrl keys blocked but Escape is not - no emergency warning
        let ops = vec![
            PendingOp::Block {
                key: KeyCode::LeftCtrl,
            },
            PendingOp::Block {
                key: KeyCode::RightCtrl,
            },
        ];
        let warnings = analyze_safety(&ops, &default_config());
        let emergency_warning = warnings.iter().find(|w| w.code == "W104");
        assert!(emergency_warning.is_none());
    }

    #[test]
    fn multiple_modifier_pair_warnings() {
        // Block multiple modifier pairs - should get multiple W105 warnings
        let ops = vec![
            PendingOp::Block {
                key: KeyCode::LeftCtrl,
            },
            PendingOp::Block {
                key: KeyCode::RightCtrl,
            },
            PendingOp::Block {
                key: KeyCode::LeftAlt,
            },
            PendingOp::Block {
                key: KeyCode::RightAlt,
            },
        ];
        let warnings = analyze_safety(&ops, &default_config());
        let modifier_warnings: Vec<_> = warnings.iter().filter(|w| w.code == "W105").collect();
        assert_eq!(modifier_warnings.len(), 2);
    }

    #[test]
    fn remap_escape_has_different_code_than_block_escape() {
        let remap_ops = vec![PendingOp::Remap {
            from: KeyCode::Escape,
            to: KeyCode::CapsLock,
        }];
        let block_ops = vec![PendingOp::Block {
            key: KeyCode::Escape,
        }];

        let remap_warnings = analyze_safety(&remap_ops, &default_config());
        let block_warnings = analyze_safety(&block_ops, &default_config());

        assert_eq!(remap_warnings[0].code, "W101"); // Remap
        assert_eq!(block_warnings[0].code, "W102"); // Block
    }

    #[test]
    fn escape_tap_hold_warning_mentions_responsiveness() {
        let ops = vec![PendingOp::TapHold {
            key: KeyCode::Escape,
            tap: KeyCode::Escape,
            hold: HoldAction::Key(KeyCode::LeftCtrl),
        }];
        let warnings = analyze_safety(&ops, &default_config());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("responsiveness"));
    }

    #[test]
    fn config_with_high_threshold_suppresses_many_blocked_warning() {
        let mut config = ValidationConfig::default();
        config.blocked_keys_warning_threshold = 100;

        let ops: Vec<PendingOp> = (0..20)
            .map(|i| {
                let key = match i % 10 {
                    0 => KeyCode::A,
                    1 => KeyCode::B,
                    2 => KeyCode::C,
                    3 => KeyCode::D,
                    4 => KeyCode::E,
                    5 => KeyCode::F,
                    6 => KeyCode::G,
                    7 => KeyCode::H,
                    8 => KeyCode::I,
                    _ => KeyCode::J,
                };
                PendingOp::Block { key }
            })
            .collect();

        let warnings = analyze_safety(&ops, &config);
        // Should not have W106 because threshold is 100
        let many_blocked = warnings.iter().find(|w| w.code == "W106");
        assert!(many_blocked.is_none());
    }

    #[test]
    fn escape_remap_from_multiple_locations() {
        // Multiple escape remaps should produce multiple warnings
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::Escape,
                to: KeyCode::A,
            },
            PendingOp::Block { key: KeyCode::X },
            PendingOp::Remap {
                from: KeyCode::Escape,
                to: KeyCode::B,
            },
        ];
        let warnings = analyze_safety(&ops, &default_config());
        let escape_warnings: Vec<_> = warnings.iter().filter(|w| w.code == "W101").collect();
        assert_eq!(escape_warnings.len(), 2);
    }

    #[test]
    fn modifier_pair_warning_shows_operation_indices() {
        let ops = vec![
            PendingOp::Block { key: KeyCode::X }, // index 0
            PendingOp::Block {
                key: KeyCode::LeftCtrl,
            }, // index 1
            PendingOp::Block { key: KeyCode::Y }, // index 2
            PendingOp::Block {
                key: KeyCode::RightCtrl,
            }, // index 3
        ];
        let warnings = analyze_safety(&ops, &default_config());
        let modifier_warning = warnings.iter().find(|w| w.code == "W105").unwrap();
        // Message should contain 1-indexed positions
        assert!(modifier_warning.message.contains("2") && modifier_warning.message.contains("4"));
    }

    #[test]
    fn combo_ops_not_counted_as_blocked_keys() {
        // Combo operations should not affect the blocked key count
        let mut config = ValidationConfig::default();
        config.blocked_keys_warning_threshold = 2;

        let ops = vec![
            PendingOp::Block { key: KeyCode::A },
            PendingOp::Combo {
                keys: vec![KeyCode::B, KeyCode::C],
                action: LayerAction::Block,
            },
            PendingOp::Combo {
                keys: vec![KeyCode::D, KeyCode::E],
                action: LayerAction::Block,
            },
        ];
        let warnings = analyze_safety(&ops, &config);
        // Only 1 actual Block operation, so W106 should not trigger
        let many_blocked = warnings.iter().find(|w| w.code == "W106");
        assert!(many_blocked.is_none());
    }

    #[test]
    fn remap_ops_not_counted_as_blocked_keys() {
        let mut config = ValidationConfig::default();
        config.blocked_keys_warning_threshold = 2;

        let ops = vec![
            PendingOp::Block { key: KeyCode::A },
            PendingOp::Remap {
                from: KeyCode::B,
                to: KeyCode::C,
            },
            PendingOp::Remap {
                from: KeyCode::D,
                to: KeyCode::E,
            },
        ];
        let warnings = analyze_safety(&ops, &config);
        // Only 1 actual Block operation, so W106 should not trigger
        let many_blocked = warnings.iter().find(|w| w.code == "W106");
        assert!(many_blocked.is_none());
    }
}
