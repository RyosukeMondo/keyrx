//! Cycle detection for circular remap dependencies.
//!
//! Detects circular remap dependencies (A→B→C→A) which can cause unpredictable
//! behavior where keys effectively swap or create feedback loops.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::drivers::keycodes::KeyCode;
use crate::scripting::PendingOp;
use crate::validation::types::SourceLocation;

use super::{Detector, DetectorContext, DetectorResult, DetectorStats, ValidationIssue};

/// Index of an operation in the pending ops list (for location tracking).
type OpIndex = usize;

/// Information about a remap for cycle detection.
#[derive(Debug, Clone)]
struct RemapEdge {
    /// Index in the original operations list.
    index: OpIndex,
    /// Target key of the remap.
    to: KeyCode,
}

/// Detects circular remap dependencies using DFS-based cycle detection.
///
/// # Examples
///
/// ```ignore
/// use keyrx_core::validation::detectors::cycles::CycleDetector;
/// use keyrx_core::validation::detectors::{Detector, DetectorContext};
/// use keyrx_core::scripting::PendingOp;
///
/// let detector = CycleDetector;
/// let ctx = DetectorContext::new(Default::default());
/// let ops = vec![/* operations */];
/// let result = detector.detect(&ops, &ctx);
/// ```
pub struct CycleDetector;

impl CycleDetector {
    /// Build a directed graph of remap operations.
    ///
    /// Returns a map from source key to all edges (target keys and operation indices).
    fn build_remap_graph(ops: &[PendingOp]) -> HashMap<KeyCode, Vec<RemapEdge>> {
        let mut graph: HashMap<KeyCode, Vec<RemapEdge>> = HashMap::new();

        for (index, op) in ops.iter().enumerate() {
            if let PendingOp::Remap { from, to } = op {
                graph
                    .entry(*from)
                    .or_default()
                    .push(RemapEdge { index, to: *to });
            }
        }

        graph
    }

    /// Find all cycles in the remap graph using DFS.
    ///
    /// Uses depth-first search with path tracking to detect cycles.
    /// Respects the max_cycle_depth configuration to avoid infinite recursion.
    fn find_cycles(
        graph: &HashMap<KeyCode, Vec<RemapEdge>>,
        max_depth: usize,
    ) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let mut reported_cycles: HashSet<Vec<KeyCode>> = HashSet::new();

        // DFS from each remap source to find cycles
        for start_key in graph.keys() {
            let mut path: Vec<(KeyCode, OpIndex)> = vec![(*start_key, 0)];
            let mut visited: HashSet<KeyCode> = HashSet::new();
            visited.insert(*start_key);

            Self::find_cycles_dfs(
                *start_key,
                graph,
                &mut path,
                &mut visited,
                max_depth,
                &mut reported_cycles,
                &mut issues,
            );
        }

        issues
    }

    /// DFS helper to find cycles in the remap graph.
    #[allow(clippy::too_many_arguments)]
    fn find_cycles_dfs(
        current: KeyCode,
        graph: &HashMap<KeyCode, Vec<RemapEdge>>,
        path: &mut Vec<(KeyCode, OpIndex)>,
        visited: &mut HashSet<KeyCode>,
        max_depth: usize,
        reported: &mut HashSet<Vec<KeyCode>>,
        issues: &mut Vec<ValidationIssue>,
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
                    let canonical = Self::canonicalize_cycle(&cycle_keys);

                    if !reported.contains(&canonical) {
                        reported.insert(canonical);
                        issues.push(Self::create_cycle_issue(path, edge.index));
                    }
                    continue;
                }

                // Continue DFS if not visited
                if !visited.contains(&next) {
                    visited.insert(next);
                    path.push((next, edge.index));
                    Self::find_cycles_dfs(next, graph, path, visited, max_depth, reported, issues);
                    path.pop();
                    visited.remove(&next);
                }
            }
        }
    }

    /// Canonicalize a cycle so [A,B,C] and [B,C,A] are treated as the same cycle.
    ///
    /// Rotates the cycle to start at the lexicographically smallest key name.
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

    /// Create a validation issue for a circular remap.
    fn create_cycle_issue(path: &[(KeyCode, OpIndex)], last_index: OpIndex) -> ValidationIssue {
        let cycle_str: Vec<String> = path.iter().map(|(k, _)| k.name().to_string()).collect();
        let first_key = &cycle_str[0];

        // Collect all operation indices involved
        let indices: Vec<String> = path
            .iter()
            .skip(1)
            .map(|(_, idx)| (idx + 1).to_string())
            .chain(std::iter::once((last_index + 1).to_string()))
            .collect();

        ValidationIssue::warning(
            "cycle",
            format!(
                "Circular remap detected: {} → {} (operations {})",
                cycle_str.join(" → "),
                first_key,
                indices.join(", "),
            ),
        )
        .with_location(SourceLocation::new(last_index + 1))
    }
}

impl Detector for CycleDetector {
    fn name(&self) -> &'static str {
        "cycle"
    }

    fn detect(&self, ops: &[PendingOp], ctx: &DetectorContext) -> DetectorResult {
        let start_time = Instant::now();

        // Build directed graph: from_key -> [(to_key, index)]
        let graph = Self::build_remap_graph(ops);

        // Find all cycles using DFS
        let issues = Self::find_cycles(&graph, ctx.config.max_cycle_depth);

        let duration = start_time.elapsed();
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
    use crate::validation::config::ValidationConfig;

    fn default_config() -> ValidationConfig {
        ValidationConfig::default()
    }

    fn make_ctx(config: ValidationConfig) -> DetectorContext {
        DetectorContext::new(config)
    }

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
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::B,
                to: KeyCode::C,
            },
            PendingOp::Remap {
                from: KeyCode::C,
                to: KeyCode::D,
            },
        ];
        let detector = CycleDetector;
        let ctx = make_ctx(default_config());
        let result = detector.detect(&ops, &ctx);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn detects_simple_two_key_cycle() {
        // A→B, B→A is a cycle
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::B,
                to: KeyCode::A,
            },
        ];
        let detector = CycleDetector;
        let ctx = make_ctx(default_config());
        let result = detector.detect(&ops, &ctx);
        assert_eq!(result.issues.len(), 1);
        assert!(result.issues[0].message.contains("Circular remap"));
    }

    #[test]
    fn detects_three_key_cycle() {
        // A→B, B→C, C→A is a cycle
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::B,
                to: KeyCode::C,
            },
            PendingOp::Remap {
                from: KeyCode::C,
                to: KeyCode::A,
            },
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

    #[test]
    fn respects_max_cycle_depth() {
        // Create a long chain that exceeds depth 3
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::B,
                to: KeyCode::C,
            },
            PendingOp::Remap {
                from: KeyCode::C,
                to: KeyCode::D,
            },
            PendingOp::Remap {
                from: KeyCode::D,
                to: KeyCode::E,
            },
            PendingOp::Remap {
                from: KeyCode::E,
                to: KeyCode::A,
            },
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
    fn does_not_report_duplicate_cycles() {
        // A→B, B→A forms one cycle, should only be reported once
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::B,
                to: KeyCode::A,
            },
        ];
        let detector = CycleDetector;
        let ctx = make_ctx(default_config());
        let result = detector.detect(&ops, &ctx);
        // Should only report once, not from A and from B
        assert_eq!(result.issues.len(), 1);
    }

    #[test]
    fn detects_multiple_independent_cycles() {
        // A→B→A and C→D→C are two separate cycles
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::B,
                to: KeyCode::A,
            },
            PendingOp::Remap {
                from: KeyCode::C,
                to: KeyCode::D,
            },
            PendingOp::Remap {
                from: KeyCode::D,
                to: KeyCode::C,
            },
        ];
        let detector = CycleDetector;
        let ctx = make_ctx(default_config());
        let result = detector.detect(&ops, &ctx);
        assert_eq!(result.issues.len(), 2);
    }

    #[test]
    fn ignores_non_remap_ops_for_cycles() {
        // Block and other ops don't form remap cycles
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Block { key: KeyCode::B },
        ];
        let detector = CycleDetector;
        let ctx = make_ctx(default_config());
        let result = detector.detect(&ops, &ctx);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn cycle_issue_has_correct_location() {
        let ops = vec![
            PendingOp::Block { key: KeyCode::X }, // index 0
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            }, // index 1
            PendingOp::Remap {
                from: KeyCode::B,
                to: KeyCode::A,
            }, // index 2
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
        use crate::validation::detectors::Severity;
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::B,
                to: KeyCode::A,
            },
        ];
        let detector = CycleDetector;
        let ctx = make_ctx(default_config());
        let result = detector.detect(&ops, &ctx);
        assert_eq!(result.issues[0].severity, Severity::Warning);
    }

    #[test]
    fn multiple_remaps_to_same_target_no_false_positive_cycle() {
        // A→C, B→C is not a cycle (both point to C but don't loop back)
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::C,
            },
            PendingOp::Remap {
                from: KeyCode::B,
                to: KeyCode::C,
            },
        ];
        let detector = CycleDetector;
        let ctx = make_ctx(default_config());
        let result = detector.detect(&ops, &ctx);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn self_remap_is_not_a_cycle() {
        // A→A is technically a remap but treated as a no-op, not a cycle
        let ops = vec![PendingOp::Remap {
            from: KeyCode::A,
            to: KeyCode::A,
        }];
        let detector = CycleDetector;
        let ctx = make_ctx(default_config());
        let result = detector.detect(&ops, &ctx);
        // This creates a self-loop in the graph, but path length is 1, requiring 2 edges for cycle detection
        assert!(result.issues.is_empty());
    }

    #[test]
    fn four_key_cycle_detected_within_depth() {
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::B,
                to: KeyCode::C,
            },
            PendingOp::Remap {
                from: KeyCode::C,
                to: KeyCode::D,
            },
            PendingOp::Remap {
                from: KeyCode::D,
                to: KeyCode::A,
            },
        ];

        let detector = CycleDetector;
        let mut config = ValidationConfig::default();
        config.max_cycle_depth = 5;
        let ctx = make_ctx(config);
        let result = detector.detect(&ops, &ctx);
        assert_eq!(result.issues.len(), 1);
    }

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
        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::B,
                to: KeyCode::A,
            },
        ];
        let detector = CycleDetector;
        let ctx = make_ctx(default_config());
        let result = detector.detect(&ops, &ctx);

        assert_eq!(result.stats.operations_checked, 2);
        assert_eq!(result.stats.issues_found, 1);
        assert!(result.stats.duration.as_micros() > 0);
    }

    #[test]
    fn config_max_cycle_depth_of_one_only_finds_self_loops() {
        // With max_depth=1, we should only find direct self-loops (A→A)
        // A→B→A requires depth 2
        let mut config = ValidationConfig::default();
        config.max_cycle_depth = 1;

        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::B,
                to: KeyCode::A,
            },
        ];
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

        let ops = vec![
            PendingOp::Remap {
                from: KeyCode::A,
                to: KeyCode::B,
            },
            PendingOp::Remap {
                from: KeyCode::B,
                to: KeyCode::A,
            },
        ];
        let detector = CycleDetector;
        let ctx = make_ctx(config);
        let result = detector.detect(&ops, &ctx);
        assert_eq!(result.issues.len(), 1);
    }
}
