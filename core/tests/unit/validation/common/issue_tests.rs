#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Unit tests for ValidationIssue and Severity types.

use keyrx_core::validation::common::issue::{Severity, ValidationIssue};
use keyrx_core::validation::types::SourceLocation;

#[test]
fn validation_issue_builder() {
    let issue = ValidationIssue::warning("conflict", "duplicate remap")
        .with_location(SourceLocation::new(10))
        .with_suggestion("Remove one of the remaps");

    assert_eq!(issue.severity, Severity::Warning);
    assert_eq!(issue.detector, "conflict");
    assert_eq!(issue.message, "duplicate remap");
    assert_eq!(issue.locations.len(), 1);
    assert!(issue.has_locations());
    assert_eq!(issue.locations[0].line, 10);
    assert!(issue.has_suggestion());
    assert_eq!(
        issue.suggestion.as_ref().unwrap(),
        "Remove one of the remaps"
    );
}

#[test]
fn validation_issue_constructors() {
    let error = ValidationIssue::error("test", "error message");
    let warning = ValidationIssue::warning("test", "warning message");
    let info = ValidationIssue::info("test", "info message");

    assert_eq!(error.severity, Severity::Error);
    assert_eq!(warning.severity, Severity::Warning);
    assert_eq!(info.severity, Severity::Info);
}

#[test]
fn validation_issue_multiple_locations() {
    let issue = ValidationIssue::error("conflict", "conflicting operations")
        .with_locations(vec![SourceLocation::new(5), SourceLocation::new(10)]);

    assert_eq!(issue.locations.len(), 2);
    assert!(issue.has_locations());
}

#[test]
fn validation_issue_display() {
    let issue = ValidationIssue::warning("conflict", "duplicate remap")
        .with_location(SourceLocation::new(42).with_column(10))
        .with_suggestion("Remove duplicate");

    let display = format!("{}", issue);
    assert!(display.contains("[warning]"));
    assert!(display.contains("conflict"));
    assert!(display.contains("duplicate remap"));
    assert!(display.contains("line 42:10"));
    assert!(display.contains("Suggestion: Remove duplicate"));
}

#[test]
fn validation_issue_display_no_locations() {
    let issue = ValidationIssue::error("test", "test error");
    let display = format!("{}", issue);
    assert!(display.contains("[error]"));
    assert!(!display.contains("line"));
}

#[test]
fn severity_ordering() {
    assert!(Severity::Info < Severity::Warning);
    assert!(Severity::Warning < Severity::Error);
    assert!(Severity::Info < Severity::Error);
}

#[test]
fn severity_display() {
    assert_eq!(Severity::Info.to_string(), "info");
    assert_eq!(Severity::Warning.to_string(), "warning");
    assert_eq!(Severity::Error.to_string(), "error");
}

#[test]
fn severity_predicates() {
    assert!(Severity::Error.is_error());
    assert!(!Severity::Error.is_warning());
    assert!(!Severity::Error.is_info());

    assert!(!Severity::Warning.is_error());
    assert!(Severity::Warning.is_warning());
    assert!(!Severity::Warning.is_info());

    assert!(!Severity::Info.is_error());
    assert!(!Severity::Info.is_warning());
    assert!(Severity::Info.is_info());
}

#[test]
fn severity_color_codes() {
    assert_eq!(Severity::Error.color_code(), 31);
    assert_eq!(Severity::Warning.color_code(), 33);
    assert_eq!(Severity::Info.color_code(), 34);
}

#[test]
fn serde_roundtrip() {
    let issue = ValidationIssue::warning("conflict", "test message")
        .with_location(SourceLocation::new(5))
        .with_suggestion("fix it");

    let json = serde_json::to_string(&issue).unwrap();
    let parsed: ValidationIssue = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.severity, Severity::Warning);
    assert_eq!(parsed.detector, "conflict");
    assert_eq!(parsed.message, "test message");
    assert_eq!(parsed.locations.len(), 1);
    assert!(parsed.has_suggestion());
}

#[test]
fn serde_skip_empty_fields() {
    let issue = ValidationIssue::error("test", "minimal issue");
    let json = serde_json::to_string(&issue).unwrap();

    // Should not serialize None suggestion
    assert!(!json.contains("suggestion"));
}
