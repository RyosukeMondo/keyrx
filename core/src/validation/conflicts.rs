//! Conflict detection for remaps and combos.
//!
//! Detects duplicate remaps, remap-block conflicts, tap-hold conflicts,
//! combo shadowing, and circular remap dependencies.

use std::collections::{HashMap, HashSet};

use crate::drivers::keycodes::KeyCode;
use crate::scripting::PendingOp;

use super::config::ValidationConfig;
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

/// Detect combo shadowing where one combo's keys are a subset of another.
///
/// When combo A's keys are a subset of combo B's keys, combo A will always
/// trigger before combo B can be completed, effectively shadowing it.
///
/// Example: [A, S] shadows [A, S, D] because pressing A+S+D will trigger
/// the A+S combo before D is pressed.
pub fn detect_combo_shadowing(ops: &[PendingOp]) -> Vec<ValidationWarning> {
    let mut warnings = Vec::new();

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
            if is_proper_subset(&combo_a.keys, &combo_b.keys) {
                // combo_a shadows combo_b
                warnings.push(create_shadowing_warning(combo_a, combo_b));
            } else if is_proper_subset(&combo_b.keys, &combo_a.keys) {
                // combo_b shadows combo_a
                warnings.push(create_shadowing_warning(combo_b, combo_a));
            }
        }
    }

    warnings
}

/// Check if `smaller` is a proper subset of `larger`.
fn is_proper_subset(smaller: &HashSet<KeyCode>, larger: &HashSet<KeyCode>) -> bool {
    smaller.len() < larger.len() && smaller.is_subset(larger)
}

/// Create a warning for combo shadowing.
fn create_shadowing_warning(shorter: &ComboInfo, longer: &ComboInfo) -> ValidationWarning {
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

    ValidationWarning::new(
        "W008",
        WarningCategory::Conflict,
        format!(
            "Combo [{}] (at {}) shadows combo [{}] (at {}): shorter combo triggers first",
            shorter_keys.join("+"),
            shorter.index + 1,
            longer_keys.join("+"),
            longer.index + 1
        ),
    )
    .with_location(SourceLocation::new(longer.index + 1))
}

/// Information about a remap for cycle detection.
#[derive(Debug, Clone)]
struct RemapEdge {
    /// Index in the original operations list.
    index: OpIndex,
    /// Target key of the remap.
    to: KeyCode,
}

/// Detect circular remap dependencies (A→B→C→A).
///
/// Circular remaps can cause unpredictable behavior where keys effectively
/// swap or create feedback loops. Uses DFS to find cycles up to config.max_cycle_depth.
pub fn detect_circular_remaps(
    ops: &[PendingOp],
    config: &ValidationConfig,
) -> Vec<ValidationWarning> {
    let mut warnings = Vec::new();

    // Build directed graph: from_key -> [(to_key, index)]
    let mut remap_graph: HashMap<KeyCode, Vec<RemapEdge>> = HashMap::new();

    for (index, op) in ops.iter().enumerate() {
        if let PendingOp::Remap { from, to } = op {
            remap_graph
                .entry(*from)
                .or_default()
                .push(RemapEdge { index, to: *to });
        }
    }

    // Track cycles we've already reported (by their canonical form)
    let mut reported_cycles: HashSet<Vec<KeyCode>> = HashSet::new();

    // DFS from each remap source to find cycles
    for start_key in remap_graph.keys() {
        let mut path: Vec<(KeyCode, OpIndex)> = vec![(*start_key, 0)];
        let mut visited: HashSet<KeyCode> = HashSet::new();
        visited.insert(*start_key);

        find_cycles_dfs(
            *start_key,
            &remap_graph,
            &mut path,
            &mut visited,
            config.max_cycle_depth,
            &mut reported_cycles,
            &mut warnings,
        );
    }

    warnings
}

/// DFS helper to find cycles in the remap graph.
fn find_cycles_dfs(
    current: KeyCode,
    graph: &HashMap<KeyCode, Vec<RemapEdge>>,
    path: &mut Vec<(KeyCode, OpIndex)>,
    visited: &mut HashSet<KeyCode>,
    max_depth: usize,
    reported: &mut HashSet<Vec<KeyCode>>,
    warnings: &mut Vec<ValidationWarning>,
) {
    if path.len() > max_depth {
        return;
    }

    if let Some(edges) = graph.get(&current) {
        for edge in edges {
            let next = edge.to;

            // Check if we found a cycle back to start
            if next == path[0].0 && path.len() >= 2 {
                // Extract cycle keys for canonical form
                let cycle_keys: Vec<KeyCode> = path.iter().map(|(k, _)| *k).collect();
                let canonical = canonicalize_cycle(&cycle_keys);

                if !reported.contains(&canonical) {
                    reported.insert(canonical);
                    warnings.push(create_cycle_warning(path, edge.index));
                }
                continue;
            }

            // Continue DFS if not visited
            if !visited.contains(&next) {
                visited.insert(next);
                path.push((next, edge.index));
                find_cycles_dfs(next, graph, path, visited, max_depth, reported, warnings);
                path.pop();
                visited.remove(&next);
            }
        }
    }
}

/// Canonicalize a cycle so [A,B,C] and [B,C,A] are treated as the same cycle.
fn canonicalize_cycle(cycle: &[KeyCode]) -> Vec<KeyCode> {
    if cycle.is_empty() {
        return vec![];
    }
    // Find minimum element's position by key name and rotate to start there
    let min_pos = cycle
        .iter()
        .enumerate()
        .min_by_key(|(_, k)| k.name())
        .map(|(i, _)| i)
        .unwrap_or(0);
    let mut canonical: Vec<KeyCode> = cycle[min_pos..].to_vec();
    canonical.extend_from_slice(&cycle[..min_pos]);
    canonical
}

/// Create a warning for a circular remap.
fn create_cycle_warning(path: &[(KeyCode, OpIndex)], last_index: OpIndex) -> ValidationWarning {
    let cycle_str: Vec<String> = path.iter().map(|(k, _)| k.name().to_string()).collect();
    let first_key = &cycle_str[0];

    // Collect all operation indices involved
    let indices: Vec<String> = path
        .iter()
        .skip(1)
        .map(|(_, idx)| (idx + 1).to_string())
        .chain(std::iter::once((last_index + 1).to_string()))
        .collect();

    ValidationWarning::new(
        "W009",
        WarningCategory::Conflict,
        format!(
            "Circular remap detected: {} → {} (operations {})",
            cycle_str.join(" → "),
            first_key,
            indices.join(", "),
        ),
    )
    .with_location(SourceLocation::new(last_index + 1))
}
