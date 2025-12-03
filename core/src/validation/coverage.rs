//! Coverage analysis for script validation.
//!
//! Analyzes which keys are affected by a script and categorizes them by
//! behavior type (remapped, blocked, tap-hold, combo, unaffected).

use std::collections::{HashMap, HashSet};

use crate::drivers::keycodes::{all_keycodes, KeyCode};
use crate::scripting::PendingOp;

use super::types::{CoverageReport, LayerCoverage};

/// Analyzes key coverage from pending operations.
pub struct CoverageAnalyzer;

impl CoverageAnalyzer {
    /// Analyze coverage from a list of pending operations.
    ///
    /// Categorizes all affected keys by their behavior type and tracks
    /// per-layer coverage for layer-specific mappings.
    pub fn analyze(ops: &[PendingOp]) -> CoverageReport {
        let mut remapped: HashSet<KeyCode> = HashSet::new();
        let mut blocked: HashSet<KeyCode> = HashSet::new();
        let mut tap_hold: HashSet<KeyCode> = HashSet::new();
        let mut combo_triggers: HashSet<KeyCode> = HashSet::new();
        let mut layers: HashMap<String, LayerCoverageBuilder> = HashMap::new();

        for op in ops {
            match op {
                PendingOp::Remap { from, .. } => {
                    remapped.insert(*from);
                }
                PendingOp::Block { key } => {
                    blocked.insert(*key);
                }
                PendingOp::Pass { key } => {
                    // Pass removes any effect, so we track it but don't add to affected
                    // It cancels previous remaps/blocks for this key
                    remapped.remove(key);
                    blocked.remove(key);
                }
                PendingOp::TapHold { key, .. } => {
                    tap_hold.insert(*key);
                }
                PendingOp::Combo { keys, .. } => {
                    for key in keys {
                        combo_triggers.insert(*key);
                    }
                }
                PendingOp::LayerMap { layer, key, action } => {
                    let layer_cov = layers.entry(layer.clone()).or_default();
                    match action {
                        crate::scripting::LayerMapAction::Remap(_) => {
                            layer_cov.remapped.insert(*key);
                        }
                        crate::scripting::LayerMapAction::Block => {
                            layer_cov.blocked.insert(*key);
                        }
                        crate::scripting::LayerMapAction::Pass => {
                            // Pass in layer removes layer-specific effect
                            layer_cov.remapped.remove(key);
                            layer_cov.blocked.remove(key);
                        }
                        crate::scripting::LayerMapAction::TapHold { .. } => {
                            layer_cov.remapped.insert(*key);
                        }
                        crate::scripting::LayerMapAction::LayerPush(_)
                        | crate::scripting::LayerMapAction::LayerToggle(_)
                        | crate::scripting::LayerMapAction::LayerPop => {
                            // Layer actions don't affect key coverage directly
                            layer_cov.remapped.insert(*key);
                        }
                    }
                }
                // Layer/modifier definitions don't affect key coverage
                PendingOp::LayerDefine { .. }
                | PendingOp::LayerPush { .. }
                | PendingOp::LayerToggle { .. }
                | PendingOp::LayerPop
                | PendingOp::DefineModifier { .. }
                | PendingOp::ModifierActivate { .. }
                | PendingOp::ModifierDeactivate { .. }
                | PendingOp::ModifierOneShot { .. }
                | PendingOp::SetTiming(_) => {}
            }
        }

        // Calculate unaffected keys (keys not in any category)
        let all_keys: HashSet<KeyCode> = all_keycodes().into_iter().collect();
        let affected: HashSet<KeyCode> = remapped
            .iter()
            .chain(blocked.iter())
            .chain(tap_hold.iter())
            .chain(combo_triggers.iter())
            .copied()
            .collect();
        let unaffected: Vec<KeyCode> = all_keys.difference(&affected).copied().collect();

        // Convert layer builders to final coverage
        let layer_coverage: HashMap<String, LayerCoverage> = layers
            .into_iter()
            .map(|(name, builder)| (name, builder.into()))
            .collect();

        CoverageReport {
            remapped: remapped.into_iter().collect(),
            blocked: blocked.into_iter().collect(),
            tap_hold: tap_hold.into_iter().collect(),
            combo_triggers: combo_triggers.into_iter().collect(),
            unaffected,
            layers: layer_coverage,
        }
    }
}

/// Builder for layer coverage during analysis.
#[derive(Default)]
struct LayerCoverageBuilder {
    remapped: HashSet<KeyCode>,
    blocked: HashSet<KeyCode>,
}

impl From<LayerCoverageBuilder> for LayerCoverage {
    fn from(builder: LayerCoverageBuilder) -> Self {
        LayerCoverage {
            remapped: builder.remapped.into_iter().collect(),
            blocked: builder.blocked.into_iter().collect(),
        }
    }
}

/// Convenience function for analyzing coverage.
pub fn analyze_coverage(ops: &[PendingOp]) -> CoverageReport {
    CoverageAnalyzer::analyze(ops)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{HoldAction, LayerAction};
    use crate::scripting::LayerMapAction;

    #[test]
    fn empty_ops_returns_all_unaffected() {
        let report = analyze_coverage(&[]);
        assert!(report.remapped.is_empty());
        assert!(report.blocked.is_empty());
        assert!(report.tap_hold.is_empty());
        assert!(report.combo_triggers.is_empty());
        assert!(!report.unaffected.is_empty());
    }

    #[test]
    fn categorizes_remapped_keys() {
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::CapsLock,
                to: KeyCode::Escape,
            },
        ];
        let report = analyze_coverage(&ops);
        assert!(report.remapped.contains(&KeyCode::A));
        assert!(report.remapped.contains(&KeyCode::CapsLock));
        assert!(!report.remapped.contains(&KeyCode::B)); // target not affected
    }

    #[test]
    fn categorizes_blocked_keys() {
        let ops = vec![
            PendingOp::Block {
                key: KeyCode::CapsLock,
            },
            PendingOp::Block {
                key: KeyCode::Insert,
            },
        ];
        let report = analyze_coverage(&ops);
        assert!(report.blocked.contains(&KeyCode::CapsLock));
        assert!(report.blocked.contains(&KeyCode::Insert));
    }

    #[test]
    fn categorizes_tap_hold_keys() {
        let ops = vec![PendingOp::TapHold {
            key: KeyCode::CapsLock,
            tap: KeyCode::Escape,
            hold: HoldAction::Key(KeyCode::LeftCtrl),
        }];
        let report = analyze_coverage(&ops);
        assert!(report.tap_hold.contains(&KeyCode::CapsLock));
    }

    #[test]
    fn categorizes_combo_trigger_keys() {
        let ops = vec![PendingOp::Combo {
            keys: vec![KeyCode::A, KeyCode::S, KeyCode::D],
            action: LayerAction::Block,
        }];
        let report = analyze_coverage(&ops);
        assert!(report.combo_triggers.contains(&KeyCode::A));
        assert!(report.combo_triggers.contains(&KeyCode::S));
        assert!(report.combo_triggers.contains(&KeyCode::D));
    }

    #[test]
    fn deduplicates_keys() {
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
        let report = analyze_coverage(&ops);
        // A should appear only once in remapped
        assert_eq!(
            report.remapped.iter().filter(|k| **k == KeyCode::A).count(),
            1
        );
    }

    #[test]
    fn pass_removes_key_from_affected() {
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Pass { key: KeyCode::A },
        ];
        let report = analyze_coverage(&ops);
        assert!(!report.remapped.contains(&KeyCode::A));
        assert!(report.unaffected.contains(&KeyCode::A));
    }

    #[test]
    fn tracks_per_layer_coverage() {
        let ops = vec![
            PendingOp::LayerMap {
                layer: "nav".to_string(),
                key: KeyCode::H,
                action: LayerMapAction::Remap(KeyCode::Left),
            },
            PendingOp::LayerMap {
                layer: "nav".to_string(),
                key: KeyCode::CapsLock,
                action: LayerMapAction::Block,
            },
        ];
        let report = analyze_coverage(&ops);
        assert!(report.layers.contains_key("nav"));
        let nav = &report.layers["nav"];
        assert!(nav.remapped.contains(&KeyCode::H));
        assert!(nav.blocked.contains(&KeyCode::CapsLock));
    }

    #[test]
    fn affected_count_correct() {
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Block { key: KeyCode::C },
            PendingOp::TapHold {
                key: KeyCode::CapsLock,
                tap: KeyCode::Escape,
                hold: HoldAction::Key(KeyCode::LeftCtrl),
            },
            PendingOp::Combo {
                keys: vec![KeyCode::D, KeyCode::F],
                action: LayerAction::Block,
            },
        ];
        let report = analyze_coverage(&ops);
        // 1 remapped + 1 blocked + 1 tap-hold + 2 combo triggers = 5
        assert_eq!(report.affected_count(), 5);
    }
}
