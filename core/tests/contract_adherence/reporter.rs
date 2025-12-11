//! Error Reporter for FFI Contract Validation
//!
//! This module generates human-readable error reports from validation errors,
//! including file locations, expected vs found values, and fix suggestions.

use super::validator::{ValidationError, ValidationReport};

/// Generates a comprehensive error report from validation errors.
///
/// The report includes:
/// - Summary of validation results
/// - Detailed error messages with file locations
/// - Expected vs found values for type mismatches
/// - Suggested fixes for each error
pub fn generate_full_report(report: &ValidationReport) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&generate_header(report));

    if report.is_success() {
        output.push_str("\n✓ All FFI contracts validated successfully.\n");
        return output;
    }

    // Group errors by type for better readability
    let (missing, uncontracted, signature_errors) = group_errors(&report.errors);

    // Missing functions section
    if !missing.is_empty() {
        output.push_str(&format_missing_functions(&missing));
    }

    // Uncontracted functions section
    if !uncontracted.is_empty() {
        output.push_str(&format_uncontracted_functions(&uncontracted));
    }

    // Signature mismatch section
    if !signature_errors.is_empty() {
        output.push_str(&format_signature_errors(&signature_errors));
    }

    // Footer with summary
    output.push_str(&generate_footer(report));

    output
}

/// Generates a summary string for a single validation error.
pub fn format_error(error: &ValidationError) -> String {
    let mut output = String::new();

    // Location prefix if available
    if let Some(loc) = error.location() {
        output.push_str(&format!("  → {}\n", loc));
    }

    // Error description
    output.push_str(&format!("    {}\n", error));

    // Fix suggestion
    output.push_str(&format!("    Fix: {}\n", error.fix_suggestion()));

    output
}

fn generate_header(report: &ValidationReport) -> String {
    let status = if report.is_success() {
        "PASSED"
    } else {
        "FAILED"
    };
    format!(
        "═══════════════════════════════════════════════════════════════\n\
         FFI Contract Validation Report - {}\n\
         ═══════════════════════════════════════════════════════════════\n\
         Contracts: {}  |  Implementations: {}  |  Passed: {}  |  Failed: {}\n",
        status,
        report.total_contracts,
        report.total_parsed,
        report.passed,
        report.failed_count()
    )
}

fn generate_footer(report: &ValidationReport) -> String {
    format!(
        "\n───────────────────────────────────────────────────────────────\n\
         Total errors: {}\n\
         Fix all errors above to ensure FFI contract compliance.\n\
         ───────────────────────────────────────────────────────────────\n",
        report.failed_count()
    )
}

fn group_errors(
    errors: &[ValidationError],
) -> (
    Vec<&ValidationError>,
    Vec<&ValidationError>,
    Vec<&ValidationError>,
) {
    let mut missing = Vec::new();
    let mut uncontracted = Vec::new();
    let mut signature_errors = Vec::new();

    for error in errors {
        match error {
            ValidationError::MissingFunction { .. } => missing.push(error),
            ValidationError::UncontractedFunction { .. } => uncontracted.push(error),
            _ => signature_errors.push(error),
        }
    }

    (missing, uncontracted, signature_errors)
}

fn format_missing_functions(errors: &[&ValidationError]) -> String {
    let mut output = String::new();
    output.push_str("\n┌─ MISSING IMPLEMENTATIONS ─────────────────────────────────────\n");
    output.push_str("│ Functions defined in contracts but not found in source:\n");

    for error in errors {
        if let ValidationError::MissingFunction {
            name,
            contract_file,
        } = error
        {
            output.push_str(&format!("│\n│  ✗ {}\n", name));
            output.push_str(&format!("│    Contract: {}\n", contract_file));
            output.push_str(&format!("│    Fix: {}\n", error.fix_suggestion()));
        }
    }

    output.push_str("└───────────────────────────────────────────────────────────────\n");
    output
}

fn format_uncontracted_functions(errors: &[&ValidationError]) -> String {
    let mut output = String::new();
    output.push_str("\n┌─ UNCONTRACTED FUNCTIONS ──────────────────────────────────────\n");
    output.push_str("│ FFI functions without contract definitions:\n");

    for error in errors {
        if let ValidationError::UncontractedFunction { name, location } = error {
            output.push_str(&format!("│\n│  ⚠ {}\n", name));
            output.push_str(&format!("│    Location: {}\n", location));
            output.push_str(&format!("│    Fix: {}\n", error.fix_suggestion()));
        }
    }

    output.push_str("└───────────────────────────────────────────────────────────────\n");
    output
}

fn format_signature_errors(errors: &[&ValidationError]) -> String {
    let mut output = String::new();
    output.push_str("\n┌─ SIGNATURE MISMATCHES ────────────────────────────────────────\n");
    output.push_str("│ Contract vs implementation signature differences:\n");

    for error in errors {
        output.push_str("│\n");
        match error {
            ValidationError::ParameterCountMismatch {
                function,
                expected,
                found,
                location,
            } => {
                output.push_str(&format!("│  ✗ {} (parameter count)\n", function));
                output.push_str(&format!("│    Location: {}\n", location));
                output.push_str(&format!("│    Expected: {} parameters\n", expected));
                output.push_str(&format!("│    Found:    {} parameters\n", found));
                output.push_str(&format!("│    Fix: {}\n", error.fix_suggestion()));
            }

            ValidationError::ParameterTypeMismatch {
                function,
                param_name,
                param_index,
                expected_type,
                found_type,
                location,
            } => {
                output.push_str(&format!("│  ✗ {} (parameter type)\n", function));
                output.push_str(&format!("│    Location: {}\n", location));
                output.push_str(&format!(
                    "│    Parameter: '{}' (index {})\n",
                    param_name, param_index
                ));
                output.push_str(&format!("│    Expected: {}\n", expected_type));
                output.push_str(&format!("│    Found:    {}\n", found_type));
                output.push_str(&format!("│    Fix: {}\n", error.fix_suggestion()));
            }

            ValidationError::ReturnTypeMismatch {
                function,
                expected_type,
                found_type,
                location,
            } => {
                output.push_str(&format!("│  ✗ {} (return type)\n", function));
                output.push_str(&format!("│    Location: {}\n", location));
                output.push_str(&format!("│    Expected: {}\n", expected_type));
                output.push_str(&format!("│    Found:    {}\n", found_type));
                output.push_str(&format!("│    Fix: {}\n", error.fix_suggestion()));
            }

            ValidationError::MissingErrorPointer { function, location } => {
                output.push_str(&format!("│  ✗ {} (missing error pointer)\n", function));
                output.push_str(&format!("│    Location: {}\n", location));
                output.push_str(&format!("│    Fix: {}\n", error.fix_suggestion()));
            }

            ValidationError::InvalidErrorPointer {
                function,
                found_type,
                location,
            } => {
                output.push_str(&format!("│  ✗ {} (invalid error pointer)\n", function));
                output.push_str(&format!("│    Location: {}\n", location));
                output.push_str("│    Expected: *mut *mut c_char\n");
                output.push_str(&format!("│    Found:    {}\n", found_type));
                output.push_str(&format!("│    Fix: {}\n", error.fix_suggestion()));
            }

            // These are handled in other sections
            ValidationError::MissingFunction { .. }
            | ValidationError::UncontractedFunction { .. } => {}
        }
    }

    output.push_str("└───────────────────────────────────────────────────────────────\n");
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract_adherence::validator::FileLocation;
    use std::path::PathBuf;

    fn make_report(
        errors: Vec<ValidationError>,
        passed: usize,
        contracts: usize,
    ) -> ValidationReport {
        let total_parsed = passed + errors.len();
        ValidationReport {
            errors,
            passed,
            total_contracts: contracts,
            total_parsed,
        }
    }

    #[test]
    fn test_generate_report_success() {
        let report = make_report(vec![], 5, 5);
        let output = generate_full_report(&report);

        assert!(output.contains("PASSED"));
        assert!(output.contains("All FFI contracts validated successfully"));
        assert!(output.contains("Passed: 5"));
    }

    #[test]
    fn test_generate_report_with_missing_function() {
        let errors = vec![ValidationError::MissingFunction {
            name: "keyrx_init".to_string(),
            contract_file: "engine.ffi-contract.json".to_string(),
        }];
        let report = make_report(errors, 2, 3);
        let output = generate_full_report(&report);

        assert!(output.contains("FAILED"));
        assert!(output.contains("MISSING IMPLEMENTATIONS"));
        assert!(output.contains("keyrx_init"));
        assert!(output.contains("engine.ffi-contract.json"));
    }

    #[test]
    fn test_generate_report_with_uncontracted_function() {
        let loc = FileLocation::new(PathBuf::from("src/exports.rs"), 42);
        let errors = vec![ValidationError::UncontractedFunction {
            name: "keyrx_orphan".to_string(),
            location: loc,
        }];
        let report = make_report(errors, 3, 3);
        let output = generate_full_report(&report);

        assert!(output.contains("UNCONTRACTED FUNCTIONS"));
        assert!(output.contains("keyrx_orphan"));
        assert!(output.contains("src/exports.rs:42"));
    }

    #[test]
    fn test_generate_report_with_parameter_count_mismatch() {
        let loc = FileLocation::new(PathBuf::from("src/exports.rs"), 100);
        let errors = vec![ValidationError::ParameterCountMismatch {
            function: "keyrx_test".to_string(),
            expected: 3,
            found: 1,
            location: loc,
        }];
        let report = make_report(errors, 4, 5);
        let output = generate_full_report(&report);

        assert!(output.contains("SIGNATURE MISMATCHES"));
        assert!(output.contains("parameter count"));
        assert!(output.contains("Expected: 3 parameters"));
        assert!(output.contains("Found:    1 parameters"));
    }

    #[test]
    fn test_generate_report_with_parameter_type_mismatch() {
        let loc = FileLocation::new(PathBuf::from("src/exports.rs"), 50);
        let errors = vec![ValidationError::ParameterTypeMismatch {
            function: "keyrx_process".to_string(),
            param_name: "input".to_string(),
            param_index: 0,
            expected_type: "*const c_char".to_string(),
            found_type: "i32".to_string(),
            location: loc,
        }];
        let report = make_report(errors, 5, 6);
        let output = generate_full_report(&report);

        assert!(output.contains("parameter type"));
        assert!(output.contains("Parameter: 'input' (index 0)"));
        assert!(output.contains("Expected: *const c_char"));
        assert!(output.contains("Found:    i32"));
    }

    #[test]
    fn test_generate_report_with_return_type_mismatch() {
        let loc = FileLocation::new(PathBuf::from("src/exports.rs"), 75);
        let errors = vec![ValidationError::ReturnTypeMismatch {
            function: "keyrx_get".to_string(),
            expected_type: "*const c_char".to_string(),
            found_type: "()".to_string(),
            location: loc,
        }];
        let report = make_report(errors, 2, 3);
        let output = generate_full_report(&report);

        assert!(output.contains("return type"));
        assert!(output.contains("Expected: *const c_char"));
        assert!(output.contains("Found:    ()"));
    }

    #[test]
    fn test_generate_report_with_error_pointer_issues() {
        let loc = FileLocation::new(PathBuf::from("src/exports.rs"), 30);
        let errors = vec![
            ValidationError::MissingErrorPointer {
                function: "keyrx_no_err".to_string(),
                location: loc.clone(),
            },
            ValidationError::InvalidErrorPointer {
                function: "keyrx_bad_err".to_string(),
                found_type: "*mut c_char".to_string(),
                location: loc,
            },
        ];
        let report = make_report(errors, 1, 3);
        let output = generate_full_report(&report);

        assert!(output.contains("missing error pointer"));
        assert!(output.contains("invalid error pointer"));
        assert!(output.contains("keyrx_no_err"));
        assert!(output.contains("keyrx_bad_err"));
    }

    #[test]
    fn test_generate_report_multiple_error_types() {
        let loc = FileLocation::new(PathBuf::from("src/exports.rs"), 10);
        let errors = vec![
            ValidationError::MissingFunction {
                name: "keyrx_missing".to_string(),
                contract_file: "test.json".to_string(),
            },
            ValidationError::UncontractedFunction {
                name: "keyrx_extra".to_string(),
                location: loc.clone(),
            },
            ValidationError::ParameterTypeMismatch {
                function: "keyrx_wrong".to_string(),
                param_name: "x".to_string(),
                param_index: 0,
                expected_type: "bool".to_string(),
                found_type: "i32".to_string(),
                location: loc,
            },
        ];
        let report = make_report(errors, 2, 5);
        let output = generate_full_report(&report);

        // All three sections should appear
        assert!(output.contains("MISSING IMPLEMENTATIONS"));
        assert!(output.contains("UNCONTRACTED FUNCTIONS"));
        assert!(output.contains("SIGNATURE MISMATCHES"));
        assert!(output.contains("Total errors: 3"));
    }

    #[test]
    fn test_format_single_error() {
        let loc = FileLocation::new(PathBuf::from("test.rs"), 42);
        let error = ValidationError::ParameterCountMismatch {
            function: "test_fn".to_string(),
            expected: 2,
            found: 1,
            location: loc,
        };

        let output = format_error(&error);

        assert!(output.contains("test.rs:42"));
        assert!(output.contains("expected 2"));
        assert!(output.contains("Fix:"));
    }

    #[test]
    fn test_format_error_without_location() {
        let error = ValidationError::MissingFunction {
            name: "keyrx_test".to_string(),
            contract_file: "test.json".to_string(),
        };

        let output = format_error(&error);

        // Should not have location line since MissingFunction has no location
        assert!(!output.contains("→"));
        assert!(output.contains("keyrx_test"));
        assert!(output.contains("Fix:"));
    }

    #[test]
    fn test_report_footer_shows_fix_instruction() {
        let loc = FileLocation::new(PathBuf::from("test.rs"), 1);
        let errors = vec![ValidationError::MissingErrorPointer {
            function: "test".to_string(),
            location: loc,
        }];
        let report = make_report(errors, 0, 1);
        let output = generate_full_report(&report);

        assert!(output.contains("Fix all errors above"));
    }
}
