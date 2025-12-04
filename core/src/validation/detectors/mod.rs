//! Detector trait and common types for validation detectors.
//!
//! This module defines the common interface that all validation detectors must implement,
//! along with supporting types for context, results, and statistics.

pub mod conflicts;
pub mod shadowing;

use crate::scripting::PendingOp;
use crate::validation::config::ValidationConfig;
use crate::validation::types::{SourceLocation, ValidationWarning};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Common interface for all validation detectors.
///
/// Detectors analyze a list of pending operations and produce validation issues.
/// Each detector focuses on a specific concern (conflicts, shadowing, cycles, etc.).
///
/// # Object Safety
///
/// This trait is object-safe and supports `Send + Sync` for use in concurrent contexts.
///
/// # Example
///
/// ```ignore
/// struct MyDetector;
///
/// impl Detector for MyDetector {
///     fn name(&self) -> &'static str {
///         "my-detector"
///     }
///
///     fn detect(&self, ops: &[PendingOp], ctx: &DetectorContext) -> DetectorResult {
///         // Analysis logic here
///         DetectorResult::default()
///     }
/// }
/// ```
pub trait Detector: Send + Sync {
    /// Returns the unique name of this detector for reporting purposes.
    ///
    /// The name should be lowercase and hyphen-separated (e.g., "conflict", "cycle").
    fn name(&self) -> &'static str;

    /// Runs detection on the given operations and returns issues found.
    ///
    /// # Arguments
    ///
    /// * `ops` - The list of pending operations to analyze
    /// * `ctx` - Contextual information for the detection pass
    ///
    /// # Returns
    ///
    /// A `DetectorResult` containing any issues found and statistics about the detection pass.
    fn detect(&self, ops: &[PendingOp], ctx: &DetectorContext) -> DetectorResult;

    /// Indicates whether this detector can be skipped for performance reasons.
    ///
    /// Some detectors (like shadowing analysis) may be expensive and can be optionally
    /// skipped when quick validation is needed.
    ///
    /// # Returns
    ///
    /// `true` if this detector can be skipped, `false` otherwise. Defaults to `false`.
    fn is_skippable(&self) -> bool {
        false
    }
}

/// Context information provided to detectors during analysis.
///
/// Contains configuration and metadata needed by detectors to perform their analysis.
#[derive(Debug, Clone)]
pub struct DetectorContext {
    /// Path to the script being validated (if available).
    pub script_path: Option<PathBuf>,

    /// Validation configuration with thresholds and limits.
    pub config: ValidationConfig,

    /// Whether to skip optional/expensive detectors.
    pub skip_optional: bool,
}

impl DetectorContext {
    /// Creates a new detector context with the given configuration.
    pub fn new(config: ValidationConfig) -> Self {
        Self {
            script_path: None,
            config,
            skip_optional: false,
        }
    }

    /// Sets the script path for this context.
    pub fn with_script_path(mut self, path: PathBuf) -> Self {
        self.script_path = Some(path);
        self
    }

    /// Enables skipping of optional detectors.
    pub fn with_skip_optional(mut self, skip: bool) -> Self {
        self.skip_optional = skip;
        self
    }
}

/// Result of a detector's analysis.
///
/// Contains the issues found and statistics about the detection pass.
#[derive(Debug, Clone, Default)]
pub struct DetectorResult {
    /// Validation issues found by the detector.
    pub issues: Vec<ValidationIssue>,

    /// Statistics about the detection pass.
    pub stats: DetectorStats,
}

impl DetectorResult {
    /// Creates a new empty result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a result with the given issues.
    pub fn with_issues(issues: Vec<ValidationIssue>) -> Self {
        Self {
            issues,
            stats: DetectorStats::default(),
        }
    }

    /// Creates a result with the given issues and statistics.
    pub fn with_stats(issues: Vec<ValidationIssue>, stats: DetectorStats) -> Self {
        Self { issues, stats }
    }

    /// Adds an issue to this result.
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    /// Returns whether any issues were found.
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    /// Returns the number of issues found.
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }
}

/// Statistics about a detector's execution.
///
/// Tracks metrics like number of operations analyzed, issues found, and execution time.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DetectorStats {
    /// Number of operations analyzed by the detector.
    pub operations_checked: usize,

    /// Number of issues found by the detector.
    pub issues_found: usize,

    /// Time taken to run the detector.
    #[serde(skip)]
    pub duration: Duration,
}

impl DetectorStats {
    /// Creates a new statistics object.
    pub fn new(operations_checked: usize, issues_found: usize, duration: Duration) -> Self {
        Self {
            operations_checked,
            issues_found,
            duration,
        }
    }

    /// Returns the duration in microseconds for legacy compatibility.
    pub fn duration_us(&self) -> u64 {
        self.duration.as_micros() as u64
    }
}

/// A validation issue found by a detector.
///
/// Represents a problem found during validation, with severity, location,
/// and optional suggestions for fixing the issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Severity level of the issue.
    pub severity: Severity,

    /// Name of the detector that found this issue.
    pub detector: String,

    /// Human-readable description of the issue.
    pub message: String,

    /// Source locations related to this issue.
    pub locations: Vec<SourceLocation>,

    /// Optional suggestion for how to fix the issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

impl ValidationIssue {
    /// Creates a new validation issue.
    pub fn new(
        severity: Severity,
        detector: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            detector: detector.into(),
            message: message.into(),
            locations: Vec::new(),
            suggestion: None,
        }
    }

    /// Creates an error-level issue.
    pub fn error(detector: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Error, detector, message)
    }

    /// Creates a warning-level issue.
    pub fn warning(detector: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, detector, message)
    }

    /// Creates an info-level issue.
    pub fn info(detector: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Info, detector, message)
    }

    /// Adds a source location to this issue.
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.locations.push(location);
        self
    }

    /// Adds multiple source locations to this issue.
    pub fn with_locations(mut self, locations: Vec<SourceLocation>) -> Self {
        self.locations.extend(locations);
        self
    }

    /// Adds a suggestion for fixing this issue.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Converts this issue to a ValidationWarning for backward compatibility.
    pub fn to_warning(&self) -> ValidationWarning {
        let warning = match self.severity {
            Severity::Error | Severity::Warning => {
                ValidationWarning::conflict(format!("W-{}", self.detector), &self.message)
            }
            Severity::Info => {
                ValidationWarning::performance(format!("I-{}", self.detector), &self.message)
            }
        };

        // Add first location if available
        if let Some(loc) = self.locations.first() {
            warning.with_location(loc.clone())
        } else {
            warning
        }
    }
}

/// Severity level of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational message, no action required.
    Info,
    /// Warning about a potential issue that may cause problems.
    Warning,
    /// Error that must be fixed before the configuration can be used.
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDetector {
        name: &'static str,
        skippable: bool,
    }

    impl Detector for TestDetector {
        fn name(&self) -> &'static str {
            self.name
        }

        fn detect(&self, _ops: &[PendingOp], _ctx: &DetectorContext) -> DetectorResult {
            DetectorResult::default()
        }

        fn is_skippable(&self) -> bool {
            self.skippable
        }
    }

    #[test]
    fn detector_trait_is_object_safe() {
        let detector: Box<dyn Detector> = Box::new(TestDetector {
            name: "test",
            skippable: false,
        });
        assert_eq!(detector.name(), "test");
        assert!(!detector.is_skippable());
    }

    #[test]
    fn detector_context_builder() {
        let config = ValidationConfig::default();
        let ctx = DetectorContext::new(config.clone())
            .with_script_path(PathBuf::from("/test/script.rhai"))
            .with_skip_optional(true);

        assert_eq!(ctx.script_path, Some(PathBuf::from("/test/script.rhai")));
        assert!(ctx.skip_optional);
    }

    #[test]
    fn detector_result_operations() {
        let mut result = DetectorResult::new();
        assert!(!result.has_issues());
        assert_eq!(result.issue_count(), 0);

        result.add_issue(ValidationIssue::error("test", "test error"));
        assert!(result.has_issues());
        assert_eq!(result.issue_count(), 1);
    }

    #[test]
    fn validation_issue_builder() {
        let issue = ValidationIssue::warning("conflict", "duplicate remap")
            .with_location(SourceLocation::new(10))
            .with_suggestion("Remove one of the remaps");

        assert_eq!(issue.severity, Severity::Warning);
        assert_eq!(issue.detector, "conflict");
        assert_eq!(issue.locations.len(), 1);
        assert!(issue.suggestion.is_some());
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn severity_display() {
        assert_eq!(Severity::Info.to_string(), "info");
        assert_eq!(Severity::Warning.to_string(), "warning");
        assert_eq!(Severity::Error.to_string(), "error");
    }

    #[test]
    fn detector_stats_duration_us() {
        let stats = DetectorStats::new(100, 5, Duration::from_micros(1234));
        assert_eq!(stats.operations_checked, 100);
        assert_eq!(stats.issues_found, 5);
        assert_eq!(stats.duration_us(), 1234);
    }

    #[test]
    fn validation_issue_to_warning() {
        let issue = ValidationIssue::error("conflict", "test message")
            .with_location(SourceLocation::new(5));

        let warning = issue.to_warning();
        assert_eq!(warning.code, "W-conflict");
        assert!(warning.message.contains("test message"));
        assert!(warning.location.is_some());
    }

    #[test]
    fn detector_result_with_stats() {
        let issues = vec![ValidationIssue::warning("test", "test issue")];
        let stats = DetectorStats::new(50, 1, Duration::from_millis(10));
        let result = DetectorResult::with_stats(issues, stats);

        assert_eq!(result.issue_count(), 1);
        assert_eq!(result.stats.operations_checked, 50);
        assert_eq!(result.stats.issues_found, 1);
    }
}
