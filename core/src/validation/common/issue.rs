//! Validation issue types.
//!
//! Defines the types for representing validation issues with severity levels,
//! source locations, and suggestions for fixing problems.

use crate::validation::types::SourceLocation;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A validation issue found by a detector.
///
/// Represents a problem found during validation, with severity, location,
/// and optional suggestions for fixing the issue.
///
/// # Example
///
/// ```ignore
/// use keyrx_core::validation::common::issue::{ValidationIssue, Severity};
/// use keyrx_core::validation::types::SourceLocation;
///
/// let issue = ValidationIssue::warning("conflict", "Duplicate remap detected")
///     .with_location(SourceLocation::new(42))
///     .with_suggestion("Remove one of the conflicting remaps");
/// ```
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
    ///
    /// # Arguments
    ///
    /// * `severity` - The severity level of the issue
    /// * `detector` - Name of the detector that found this issue
    /// * `message` - Human-readable description of the problem
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
    ///
    /// Errors represent problems that must be fixed before the configuration can be used.
    pub fn error(detector: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Error, detector, message)
    }

    /// Creates a warning-level issue.
    ///
    /// Warnings represent potential problems that may cause unexpected behavior.
    pub fn warning(detector: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, detector, message)
    }

    /// Creates an info-level issue.
    ///
    /// Info-level issues are informational messages that don't require action.
    pub fn info(detector: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Info, detector, message)
    }

    /// Adds a source location to this issue.
    ///
    /// Issues can have multiple locations (e.g., for conflicts involving multiple keys).
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
    ///
    /// Suggestions provide actionable guidance to help resolve the problem.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Returns whether this issue has any source locations.
    pub fn has_locations(&self) -> bool {
        !self.locations.is_empty()
    }

    /// Returns whether this issue has a suggestion.
    pub fn has_suggestion(&self) -> bool {
        self.suggestion.is_some()
    }
}

impl fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format: [severity] detector: message
        write!(f, "[{}] {}: {}", self.severity, self.detector, self.message)?;

        // Add locations if present
        if !self.locations.is_empty() {
            write!(f, " (")?;
            for (i, loc) in self.locations.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "line {}", loc.line)?;
                if let Some(col) = loc.column {
                    write!(f, ":{}", col)?;
                }
            }
            write!(f, ")")?;
        }

        // Add suggestion if present
        if let Some(suggestion) = &self.suggestion {
            write!(f, " - Suggestion: {}", suggestion)?;
        }

        Ok(())
    }
}

/// Severity level of a validation issue.
///
/// Defines the importance and urgency of a validation issue.
/// Severity levels are ordered: Info < Warning < Error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational message, no action required.
    ///
    /// Used for helpful tips or context about the configuration.
    Info,

    /// Warning about a potential issue that may cause problems.
    ///
    /// The configuration is valid, but may exhibit unexpected behavior.
    Warning,

    /// Error that must be fixed before the configuration can be used.
    ///
    /// The configuration is invalid and will not work correctly.
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

impl Severity {
    /// Returns whether this is an error-level severity.
    pub fn is_error(&self) -> bool {
        matches!(self, Severity::Error)
    }

    /// Returns whether this is a warning-level severity.
    pub fn is_warning(&self) -> bool {
        matches!(self, Severity::Warning)
    }

    /// Returns whether this is an info-level severity.
    pub fn is_info(&self) -> bool {
        matches!(self, Severity::Info)
    }

    /// Returns the ANSI color code for this severity level.
    ///
    /// Useful for terminal output:
    /// - Error: red (31)
    /// - Warning: yellow (33)
    /// - Info: blue (34)
    pub fn color_code(&self) -> u8 {
        match self {
            Severity::Error => 31,   // red
            Severity::Warning => 33, // yellow
            Severity::Info => 34,    // blue
        }
    }
}
