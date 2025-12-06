//! Detector orchestrator for coordinating validation detectors.
//!
//! The orchestrator manages multiple detectors, running them in sequence and
//! aggregating their results into a unified report.

use std::time::Instant;

use crate::scripting::PendingOp;

use super::detectors::{Detector, DetectorContext, DetectorStats, ValidationIssue};

/// Aggregated report from all detector passes.
///
/// Contains all issues found by all detectors, along with aggregate statistics.
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    /// All validation issues found, grouped by detector.
    pub issues: Vec<ValidationIssue>,

    /// Statistics from each detector pass.
    pub detector_stats: Vec<NamedDetectorStats>,

    /// Total operations analyzed.
    pub total_operations: usize,

    /// Total time spent in detection.
    pub total_duration_us: u64,

    /// Number of detectors that were skipped.
    pub skipped_detectors: usize,
}

impl ValidationReport {
    /// Creates a new empty validation report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether any issues were found.
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    /// Returns the total number of issues found.
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    /// Returns the number of error-level issues.
    pub fn error_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity.is_error()).count()
    }

    /// Returns the number of warning-level issues.
    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity.is_warning())
            .count()
    }

    /// Returns the number of info-level issues.
    pub fn info_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity.is_info()).count()
    }
}

/// Detector statistics paired with detector name.
#[derive(Debug, Clone)]
pub struct NamedDetectorStats {
    /// Name of the detector.
    pub name: String,
    /// Statistics from the detector's execution.
    pub stats: DetectorStats,
}

/// Orchestrates multiple validation detectors.
///
/// The orchestrator manages a collection of detectors, running them in sequence
/// and aggregating their results. It supports skipping optional detectors based
/// on context configuration.
///
/// # Example
///
/// ```ignore
/// use keyrx_core::validation::orchestrator::DetectorOrchestrator;
/// use keyrx_core::validation::detectors::{conflicts::ConflictDetector, DetectorContext};
/// use keyrx_core::validation::config::ValidationConfig;
///
/// let mut orchestrator = DetectorOrchestrator::new();
/// orchestrator.register(Box::new(ConflictDetector::new()));
///
/// let ops = vec![/* pending operations */];
/// let ctx = DetectorContext::new(ValidationConfig::default());
/// let report = orchestrator.run(&ops, &ctx);
///
/// println!("Found {} issues", report.issue_count());
/// ```
pub struct DetectorOrchestrator {
    detectors: Vec<Box<dyn Detector>>,
}

impl DetectorOrchestrator {
    /// Creates a new empty orchestrator.
    pub fn new() -> Self {
        Self {
            detectors: Vec::new(),
        }
    }

    /// Registers a detector with the orchestrator.
    ///
    /// Detectors are run in the order they are registered.
    pub fn register(&mut self, detector: Box<dyn Detector>) {
        self.detectors.push(detector);
    }

    /// Registers multiple detectors at once.
    pub fn register_all(&mut self, detectors: Vec<Box<dyn Detector>>) {
        self.detectors.extend(detectors);
    }

    /// Runs all registered detectors and aggregates their results.
    ///
    /// Detectors are run in registration order. If `ctx.skip_optional` is true,
    /// detectors marked as skippable will be skipped.
    ///
    /// # Arguments
    ///
    /// * `ops` - The operations to analyze
    /// * `ctx` - Context for the detection passes
    ///
    /// # Returns
    ///
    /// A `ValidationReport` containing all issues found and aggregate statistics.
    pub fn run(&self, ops: &[PendingOp], ctx: &DetectorContext) -> ValidationReport {
        let mut report = ValidationReport::new();
        report.total_operations = ops.len();

        let overall_start = Instant::now();

        for detector in &self.detectors {
            // Skip optional detectors if requested
            if ctx.skip_optional && detector.is_skippable() {
                report.skipped_detectors += 1;
                continue;
            }

            // Run the detector
            let start = Instant::now();
            let result = detector.detect(ops, ctx);
            let duration = start.elapsed();

            // Aggregate results
            report.issues.extend(result.issues);

            // Record statistics with actual duration
            let mut stats = result.stats;
            stats.duration = duration;

            report.detector_stats.push(NamedDetectorStats {
                name: detector.name().to_string(),
                stats,
            });
        }

        report.total_duration_us = overall_start.elapsed().as_micros() as u64;

        report
    }

    /// Returns the number of registered detectors.
    pub fn detector_count(&self) -> usize {
        self.detectors.len()
    }

    /// Returns the names of all registered detectors.
    pub fn detector_names(&self) -> Vec<&str> {
        self.detectors.iter().map(|d| d.name()).collect()
    }
}

impl Default for DetectorOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::config::ValidationConfig;
    use crate::validation::detectors::DetectorResult;

    // Mock detector for testing
    struct MockDetector {
        name: &'static str,
        skippable: bool,
        issue_count: usize,
    }

    impl Detector for MockDetector {
        fn name(&self) -> &'static str {
            self.name
        }

        fn detect(&self, ops: &[PendingOp], _ctx: &DetectorContext) -> DetectorResult {
            let mut result = DetectorResult::new();

            // Generate mock issues
            for i in 0..self.issue_count {
                result.add_issue(ValidationIssue::warning(
                    self.name,
                    format!("Mock issue {}", i),
                ));
            }

            result.stats = DetectorStats::new(ops.len(), self.issue_count, Default::default());
            result
        }

        fn is_skippable(&self) -> bool {
            self.skippable
        }
    }

    #[test]
    fn orchestrator_runs_detectors_in_order() {
        let mut orchestrator = DetectorOrchestrator::new();
        orchestrator.register(Box::new(MockDetector {
            name: "detector-1",
            skippable: false,
            issue_count: 2,
        }));
        orchestrator.register(Box::new(MockDetector {
            name: "detector-2",
            skippable: false,
            issue_count: 3,
        }));

        let ops = vec![];
        let ctx = DetectorContext::new(ValidationConfig::default());
        let report = orchestrator.run(&ops, &ctx);

        assert_eq!(report.issue_count(), 5);
        assert_eq!(report.detector_stats.len(), 2);
        assert_eq!(report.detector_stats[0].name, "detector-1");
        assert_eq!(report.detector_stats[1].name, "detector-2");
    }

    #[test]
    fn orchestrator_skips_optional_detectors() {
        let mut orchestrator = DetectorOrchestrator::new();
        orchestrator.register(Box::new(MockDetector {
            name: "required",
            skippable: false,
            issue_count: 2,
        }));
        orchestrator.register(Box::new(MockDetector {
            name: "optional",
            skippable: true,
            issue_count: 3,
        }));

        let ops = vec![];
        let ctx = DetectorContext::new(ValidationConfig::default()).with_skip_optional(true);
        let report = orchestrator.run(&ops, &ctx);

        // Only required detector should run
        assert_eq!(report.issue_count(), 2);
        assert_eq!(report.detector_stats.len(), 1);
        assert_eq!(report.detector_stats[0].name, "required");
        assert_eq!(report.skipped_detectors, 1);
    }

    #[test]
    fn orchestrator_runs_all_when_skip_optional_false() {
        let mut orchestrator = DetectorOrchestrator::new();
        orchestrator.register(Box::new(MockDetector {
            name: "required",
            skippable: false,
            issue_count: 2,
        }));
        orchestrator.register(Box::new(MockDetector {
            name: "optional",
            skippable: true,
            issue_count: 3,
        }));

        let ops = vec![];
        let ctx = DetectorContext::new(ValidationConfig::default()).with_skip_optional(false);
        let report = orchestrator.run(&ops, &ctx);

        // Both detectors should run
        assert_eq!(report.issue_count(), 5);
        assert_eq!(report.detector_stats.len(), 2);
        assert_eq!(report.skipped_detectors, 0);
    }

    #[test]
    fn validation_report_counts() {
        let mut report = ValidationReport::new();
        assert!(!report.has_issues());
        assert_eq!(report.issue_count(), 0);

        report
            .issues
            .push(ValidationIssue::error("test", "error 1"));
        report
            .issues
            .push(ValidationIssue::warning("test", "warning 1"));
        report
            .issues
            .push(ValidationIssue::warning("test", "warning 2"));
        report.issues.push(ValidationIssue::info("test", "info 1"));

        assert!(report.has_issues());
        assert_eq!(report.issue_count(), 4);
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.warning_count(), 2);
        assert_eq!(report.info_count(), 1);
    }

    #[test]
    fn orchestrator_empty() {
        let orchestrator = DetectorOrchestrator::new();
        assert_eq!(orchestrator.detector_count(), 0);

        let ops = vec![];
        let ctx = DetectorContext::new(ValidationConfig::default());
        let report = orchestrator.run(&ops, &ctx);

        assert!(!report.has_issues());
        assert_eq!(report.detector_stats.len(), 0);
    }

    #[test]
    fn orchestrator_detector_names() {
        let mut orchestrator = DetectorOrchestrator::new();
        orchestrator.register(Box::new(MockDetector {
            name: "detector-a",
            skippable: false,
            issue_count: 0,
        }));
        orchestrator.register(Box::new(MockDetector {
            name: "detector-b",
            skippable: false,
            issue_count: 0,
        }));

        let names = orchestrator.detector_names();
        assert_eq!(names, vec!["detector-a", "detector-b"]);
    }

    #[test]
    fn orchestrator_tracks_total_operations() {
        let mut orchestrator = DetectorOrchestrator::new();
        orchestrator.register(Box::new(MockDetector {
            name: "test",
            skippable: false,
            issue_count: 0,
        }));

        let ops = vec![
            PendingOp::Remap {
                from: crate::drivers::keycodes::KeyCode::A,
                to: crate::drivers::keycodes::KeyCode::B,
            },
            PendingOp::Block {
                key: crate::drivers::keycodes::KeyCode::C,
            },
        ];

        let ctx = DetectorContext::new(ValidationConfig::default());
        let report = orchestrator.run(&ops, &ctx);

        assert_eq!(report.total_operations, 2);
    }

    #[test]
    fn orchestrator_register_all() {
        let mut orchestrator = DetectorOrchestrator::new();
        let detectors: Vec<Box<dyn Detector>> = vec![
            Box::new(MockDetector {
                name: "detector-1",
                skippable: false,
                issue_count: 1,
            }),
            Box::new(MockDetector {
                name: "detector-2",
                skippable: false,
                issue_count: 1,
            }),
        ];

        orchestrator.register_all(detectors);
        assert_eq!(orchestrator.detector_count(), 2);
    }
}
