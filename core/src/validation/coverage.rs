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
                | PendingOp::LayoutDefine { .. }
                | PendingOp::LayoutEnable { .. }
                | PendingOp::LayoutDisable { .. }
                | PendingOp::LayoutRemove { .. }
                | PendingOp::LayoutSetPriority { .. }
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

/// Render an ASCII visualization of the ANSI keyboard layout with affected keys highlighted.
/// Keys are marked: `R`=Remap, `B`=Block, `T`=Tap-hold, `C`=Combo, ` `=Unaffected
pub fn render_ascii_keyboard(coverage: &CoverageReport) -> String {
    use std::collections::HashSet;
    use KeyCode::*;

    let remapped: HashSet<_> = coverage.remapped.iter().collect();
    let blocked: HashSet<_> = coverage.blocked.iter().collect();
    let tap_hold: HashSet<_> = coverage.tap_hold.iter().collect();
    let combo: HashSet<_> = coverage.combo_triggers.iter().collect();

    let ind = |k: &KeyCode| -> char {
        if tap_hold.contains(k) {
            'T'
        } else if combo.contains(k) {
            'C'
        } else if remapped.contains(k) {
            'R'
        } else if blocked.contains(k) {
            'B'
        } else {
            ' '
        }
    };

    let cell = |k: KeyCode, label: &str| format!("[{}{}]", ind(&k), label);

    // Keyboard layout: Each row defined as (key, label) pairs with separators
    let rows: &[&[(Option<KeyCode>, &str)]] = &[
        // F-row: Esc + F1-F12 + Print/Scroll/Pause
        &[
            (Some(Escape), "Esc"),
            (None, " "),
            (Some(F1), "F1"),
            (Some(F2), "F2"),
            (Some(F3), "F3"),
            (Some(F4), "F4"),
            (None, ""),
            (Some(F5), "F5"),
            (Some(F6), "F6"),
            (Some(F7), "F7"),
            (Some(F8), "F8"),
            (None, ""),
            (Some(F9), "F9"),
            (Some(F10), "F10"),
            (Some(F11), "F11"),
            (Some(F12), "F12"),
            (None, " "),
            (Some(PrintScreen), "Prt"),
            (Some(ScrollLock), "Scr"),
            (Some(Pause), "Pau"),
        ],
        // Number row
        &[
            (Some(Grave), "`"),
            (Some(Key1), "1"),
            (Some(Key2), "2"),
            (Some(Key3), "3"),
            (Some(Key4), "4"),
            (Some(Key5), "5"),
            (Some(Key6), "6"),
            (Some(Key7), "7"),
            (Some(Key8), "8"),
            (Some(Key9), "9"),
            (Some(Key0), "0"),
            (Some(Minus), "-"),
            (Some(Equal), "="),
            (Some(Backspace), "Bksp"),
            (None, " "),
            (Some(Insert), "Ins"),
            (Some(Home), "Hom"),
            (Some(PageUp), "PgU"),
            (None, " "),
            (Some(NumLock), "Num"),
            (Some(NumpadDivide), "/"),
            (Some(NumpadMultiply), "*"),
            (Some(NumpadSubtract), "-"),
        ],
        // QWERTY row
        &[
            (Some(Tab), "Tab"),
            (Some(Q), "Q"),
            (Some(W), "W"),
            (Some(E), "E"),
            (Some(R), "R"),
            (Some(T), "T"),
            (Some(Y), "Y"),
            (Some(U), "U"),
            (Some(I), "I"),
            (Some(O), "O"),
            (Some(P), "P"),
            (Some(LeftBracket), "["),
            (Some(RightBracket), "]"),
            (Some(Backslash), "\\"),
            (None, " "),
            (Some(Delete), "Del"),
            (Some(End), "End"),
            (Some(PageDown), "PgD"),
            (None, " "),
            (Some(Numpad7), "7"),
            (Some(Numpad8), "8"),
            (Some(Numpad9), "9"),
            (Some(NumpadAdd), "+"),
        ],
        // Home row
        &[
            (Some(CapsLock), "Caps"),
            (Some(A), "A"),
            (Some(S), "S"),
            (Some(D), "D"),
            (Some(F), "F"),
            (Some(G), "G"),
            (Some(H), "H"),
            (Some(J), "J"),
            (Some(K), "K"),
            (Some(L), "L"),
            (Some(Semicolon), ";"),
            (Some(Apostrophe), "'"),
            (Some(Enter), "Enter"),
            (None, "     "),
            (Some(Numpad4), "4"),
            (Some(Numpad5), "5"),
            (Some(Numpad6), "6"),
        ],
        // Shift row
        &[
            (Some(LeftShift), "Shift"),
            (Some(Z), "Z"),
            (Some(X), "X"),
            (Some(C), "C"),
            (Some(V), "V"),
            (Some(B), "B"),
            (Some(N), "N"),
            (Some(M), "M"),
            (Some(Comma), ","),
            (Some(Period), "."),
            (Some(Slash), "/"),
            (Some(RightShift), "Shift"),
            (None, "  "),
            (Some(Up), "^"),
            (None, "  "),
            (Some(Numpad1), "1"),
            (Some(Numpad2), "2"),
            (Some(Numpad3), "3"),
            (Some(NumpadEnter), "En"),
        ],
        // Bottom row
        &[
            (Some(LeftCtrl), "Ctrl"),
            (Some(LeftMeta), "Win"),
            (Some(LeftAlt), "Alt"),
            (Some(Space), "  Space  "),
            (Some(RightAlt), "Alt"),
            (Some(RightMeta), "Win"),
            (Some(RightCtrl), "Ctrl"),
            (None, " "),
            (Some(Left), "<"),
            (Some(Down), "v"),
            (Some(Right), ">"),
            (None, " "),
            (Some(Numpad0), " 0 "),
            (Some(NumpadDecimal), "."),
        ],
    ];

    let mut output = String::new();
    for row in rows {
        for (key, label) in *row {
            match key {
                Some(k) => output.push_str(&cell(*k, label)),
                None => output.push_str(label),
            }
        }
        output.push('\n');
    }
    output.push_str("\nLegend: [R]=Remap [B]=Block [T]=Tap-Hold [C]=Combo [ ]=Unaffected\n");
    output.push_str(&format!(
        "Coverage: {} remapped, {} blocked, {} tap-hold, {} combo, {} unaffected\n",
        coverage.remapped.len(),
        coverage.blocked.len(),
        coverage.tap_hold.len(),
        coverage.combo_triggers.len(),
        coverage.unaffected.len(),
    ));
    output
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

    #[test]
    fn ascii_keyboard_renders_without_panic() {
        let report = CoverageReport::default();
        let output = render_ascii_keyboard(&report);
        assert!(!output.is_empty());
        assert!(output.contains("Legend:"));
        assert!(output.contains("Coverage:"));
    }

    #[test]
    fn ascii_keyboard_shows_remapped_keys() {
        let mut report = CoverageReport::default();
        report.remapped.push(KeyCode::A);
        report.remapped.push(KeyCode::CapsLock);
        let output = render_ascii_keyboard(&report);
        // Check that remapped indicator R appears for affected keys
        assert!(output.contains("[R"));
        assert!(output.contains("2 remapped"));
    }

    #[test]
    fn ascii_keyboard_shows_blocked_keys() {
        let mut report = CoverageReport::default();
        report.blocked.push(KeyCode::Insert);
        let output = render_ascii_keyboard(&report);
        assert!(output.contains("[B"));
        assert!(output.contains("1 blocked"));
    }

    #[test]
    fn ascii_keyboard_shows_tap_hold_keys() {
        let mut report = CoverageReport::default();
        report.tap_hold.push(KeyCode::CapsLock);
        let output = render_ascii_keyboard(&report);
        assert!(output.contains("[T"));
        assert!(output.contains("1 tap-hold"));
    }

    #[test]
    fn ascii_keyboard_shows_combo_keys() {
        let mut report = CoverageReport::default();
        report.combo_triggers.push(KeyCode::A);
        report.combo_triggers.push(KeyCode::S);
        report.combo_triggers.push(KeyCode::D);
        let output = render_ascii_keyboard(&report);
        assert!(output.contains("[C"));
        assert!(output.contains("3 combo"));
    }

    #[test]
    fn ascii_keyboard_tap_hold_has_priority() {
        // Tap-hold should have highest priority over other states
        let mut report = CoverageReport::default();
        report.tap_hold.push(KeyCode::A);
        report.remapped.push(KeyCode::A); // Also remapped
        report.combo_triggers.push(KeyCode::A); // Also combo
        let output = render_ascii_keyboard(&report);
        // Should show T for tap-hold, not R or C
        // The A key cell should have T indicator (format: [TA] for width 4)
        assert!(
            output.contains("[TA]"),
            "Expected tap-hold indicator T for key A"
        );
        // Should report 1 tap-hold
        assert!(output.contains("1 tap-hold"));
    }

    #[test]
    fn ascii_keyboard_contains_all_rows() {
        let report = CoverageReport::default();
        let output = render_ascii_keyboard(&report);
        // Check for keys from each row
        assert!(output.contains("Esc")); // Row 1 (F-key row)
        assert!(output.contains("F1")); // Row 1
        assert!(output.contains("Bksp")); // Row 2 (number row)
        assert!(output.contains("Tab")); // Row 3 (QWERTY row)
        assert!(output.contains("Caps")); // Row 4 (home row)
        assert!(output.contains("Shift")); // Row 5
        assert!(output.contains("Ctrl")); // Row 6 (bottom row)
        assert!(output.contains("Space")); // Row 6
    }
}
