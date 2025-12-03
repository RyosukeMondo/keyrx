//! Validation types and result structures.
//!
//! Defines the data structures for validation results, errors, warnings,
//! and coverage reports used by the script validation system.

use crate::drivers::keycodes::KeyCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of script validation containing errors, warnings, and optional coverage.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Script is valid (no errors, warnings allowed).
    pub is_valid: bool,
    /// Semantic and structural errors.
    pub errors: Vec<ValidationError>,
    /// Conflict and safety warnings.
    pub warnings: Vec<ValidationWarning>,
    /// Coverage analysis (if requested).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<CoverageReport>,
}

impl ValidationResult {
    /// Create a new valid result with no errors or warnings.
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            coverage: None,
        }
    }

    /// Create a result from a list of errors.
    pub fn with_errors(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: errors.is_empty(),
            errors,
            warnings: Vec::new(),
            coverage: None,
        }
    }

    /// Add an error to this result.
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.is_valid = false;
    }

    /// Add a warning to this result.
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Add coverage report to this result.
    pub fn with_coverage(mut self, coverage: CoverageReport) -> Self {
        self.coverage = Some(coverage);
        self
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// A validation error indicating an invalid script construct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error code for categorization (e.g., "E001").
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Source location if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
    /// Suggested fixes (e.g., similar key names).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<String>,
}

impl ValidationError {
    /// Create a new validation error.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            location: None,
            suggestions: Vec::new(),
        }
    }

    /// Add source location to this error.
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add suggestions to this error.
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    /// Create an unknown key error.
    pub fn unknown_key(key: &str, suggestions: Vec<String>) -> Self {
        Self::new("E001", format!("Unknown key: '{key}'")).with_suggestions(suggestions)
    }

    /// Create an undefined layer error.
    pub fn undefined_layer(layer: &str, defined: &[String]) -> Self {
        let suggestion = if defined.is_empty() {
            String::new()
        } else {
            format!(". Defined layers: {}", defined.join(", "))
        };
        Self::new("E002", format!("Undefined layer: '{layer}'{suggestion}"))
    }

    /// Create an undefined modifier error.
    pub fn undefined_modifier(modifier: &str, defined: &[String]) -> Self {
        let suggestion = if defined.is_empty() {
            String::new()
        } else {
            format!(". Defined modifiers: {}", defined.join(", "))
        };
        Self::new(
            "E003",
            format!("Undefined modifier: '{modifier}'{suggestion}"),
        )
    }
}

/// A validation warning indicating a potential issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning code for categorization (e.g., "W001").
    pub code: String,
    /// Warning category.
    pub category: WarningCategory,
    /// Human-readable message.
    pub message: String,
    /// Source location if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
}

impl ValidationWarning {
    /// Create a new validation warning.
    pub fn new(
        code: impl Into<String>,
        category: WarningCategory,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            category,
            message: message.into(),
            location: None,
        }
    }

    /// Add source location to this warning.
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Create a conflict warning.
    pub fn conflict(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(code, WarningCategory::Conflict, message)
    }

    /// Create a safety warning.
    pub fn safety(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(code, WarningCategory::Safety, message)
    }

    /// Create a performance warning.
    pub fn performance(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(code, WarningCategory::Performance, message)
    }
}

/// Categories of validation warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WarningCategory {
    /// Conflicting key mappings.
    Conflict,
    /// Potentially dangerous patterns (lockout risk).
    Safety,
    /// Performance-related concerns.
    Performance,
}

/// Source location for an error or warning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Line number (1-indexed).
    pub line: usize,
    /// Column number (1-indexed, optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    /// The problematic line of code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl SourceLocation {
    /// Create a new source location.
    pub fn new(line: usize) -> Self {
        Self {
            line,
            column: None,
            context: None,
        }
    }

    /// Add column to this location.
    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// Add context (the line content) to this location.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Coverage report showing which keys are affected by the script.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoverageReport {
    /// Keys that are remapped.
    pub remapped: Vec<KeyCode>,
    /// Keys that are blocked.
    pub blocked: Vec<KeyCode>,
    /// Keys with tap-hold behavior.
    pub tap_hold: Vec<KeyCode>,
    /// Keys involved in combos (trigger keys).
    pub combo_triggers: Vec<KeyCode>,
    /// Keys unaffected by script.
    pub unaffected: Vec<KeyCode>,
    /// Per-layer coverage.
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub layers: HashMap<String, LayerCoverage>,
}

impl CoverageReport {
    /// Create an empty coverage report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get total count of affected keys.
    pub fn affected_count(&self) -> usize {
        self.remapped.len() + self.blocked.len() + self.tap_hold.len() + self.combo_triggers.len()
    }
}

/// Per-layer coverage information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayerCoverage {
    /// Keys remapped in this layer.
    pub remapped: Vec<KeyCode>,
    /// Keys blocked in this layer.
    pub blocked: Vec<KeyCode>,
}

/// Options controlling validation behavior.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationOptions {
    /// Treat warnings as errors.
    pub strict: bool,
    /// Suppress warnings in output.
    pub no_warnings: bool,
    /// Include coverage analysis.
    pub include_coverage: bool,
    /// Include ASCII keyboard visualization.
    pub include_visual: bool,
}

impl ValidationOptions {
    /// Create default validation options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable strict mode (warnings as errors).
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Disable warnings in output.
    pub fn no_warnings(mut self) -> Self {
        self.no_warnings = true;
        self
    }

    /// Include coverage analysis.
    pub fn with_coverage(mut self) -> Self {
        self.include_coverage = true;
        self
    }

    /// Include ASCII keyboard visualization.
    pub fn with_visual(mut self) -> Self {
        self.include_visual = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_result_valid_by_default() {
        let result = ValidationResult::valid();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn validation_result_becomes_invalid_on_error() {
        let mut result = ValidationResult::valid();
        result.add_error(ValidationError::new("E001", "test error"));
        assert!(!result.is_valid);
        assert!(result.has_errors());
    }

    #[test]
    fn validation_error_with_suggestions() {
        let error = ValidationError::unknown_key("Escpe", vec!["Escape".into()]);
        assert_eq!(error.code, "E001");
        assert!(error.message.contains("Escpe"));
        assert_eq!(error.suggestions, vec!["Escape"]);
    }

    #[test]
    fn validation_warning_categories() {
        let conflict = ValidationWarning::conflict("W001", "duplicate remap");
        let safety = ValidationWarning::safety("W002", "escape blocked");
        let perf = ValidationWarning::performance("W003", "many combos");

        assert_eq!(conflict.category, WarningCategory::Conflict);
        assert_eq!(safety.category, WarningCategory::Safety);
        assert_eq!(perf.category, WarningCategory::Performance);
    }

    #[test]
    fn source_location_builder() {
        let loc = SourceLocation::new(10)
            .with_column(5)
            .with_context("remap(\"A\", \"B\")");

        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, Some(5));
        assert_eq!(loc.context, Some("remap(\"A\", \"B\")".into()));
    }

    #[test]
    fn validation_options_builder() {
        let opts = ValidationOptions::new()
            .strict()
            .with_coverage()
            .with_visual();

        assert!(opts.strict);
        assert!(opts.include_coverage);
        assert!(opts.include_visual);
        assert!(!opts.no_warnings);
    }

    #[test]
    fn coverage_report_affected_count() {
        let mut report = CoverageReport::new();
        report.remapped.push(KeyCode::A);
        report.blocked.push(KeyCode::B);
        report.tap_hold.push(KeyCode::C);

        assert_eq!(report.affected_count(), 3);
    }

    #[test]
    fn serde_roundtrip() {
        let mut result = ValidationResult::valid();
        result.add_error(
            ValidationError::unknown_key("BadKey", vec!["GoodKey".into()])
                .with_location(SourceLocation::new(5)),
        );
        result.add_warning(ValidationWarning::safety("W001", "escape blocked"));

        let json = serde_json::to_string(&result).unwrap();
        let parsed: ValidationResult = serde_json::from_str(&json).unwrap();

        assert!(!parsed.is_valid);
        assert_eq!(parsed.errors.len(), 1);
        assert_eq!(parsed.warnings.len(), 1);
        assert_eq!(parsed.errors[0].code, "E001");
    }
}
