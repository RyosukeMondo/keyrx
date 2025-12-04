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
