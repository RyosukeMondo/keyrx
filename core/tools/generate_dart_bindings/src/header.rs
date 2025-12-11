//! File header generator for generated Dart files
//!
//! This module provides functionality to generate file headers with warnings,
//! timestamps, and source contract information for generated Dart FFI bindings.

use crate::templates::{context, render, BINDINGS_FILE_HEADER, MODELS_FILE_HEADER};
use chrono::{DateTime, Utc};

/// Generate a header for bindings files with the current timestamp
pub fn generate_bindings_header() -> String {
    generate_bindings_header_with_time(Utc::now())
}

/// Generate a header for bindings files with a specific timestamp (for testing)
pub fn generate_bindings_header_with_time(timestamp: DateTime<Utc>) -> String {
    let mut ctx = context();
    ctx.insert("timestamp".to_string(), format_timestamp(timestamp));
    render(BINDINGS_FILE_HEADER, &ctx)
}

/// Generate a header for models files with the current timestamp
pub fn generate_models_header() -> String {
    generate_models_header_with_time(Utc::now())
}

/// Generate a header for models files with a specific timestamp (for testing)
pub fn generate_models_header_with_time(timestamp: DateTime<Utc>) -> String {
    let mut ctx = context();
    ctx.insert("timestamp".to_string(), format_timestamp(timestamp));
    render(MODELS_FILE_HEADER, &ctx)
}

/// Format timestamp as ISO 8601 string
fn format_timestamp(timestamp: DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn test_timestamp() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2025, 1, 15, 10, 30, 0).unwrap()
    }

    #[test]
    fn test_bindings_header_contains_warning() {
        let header = generate_bindings_header_with_time(test_timestamp());
        assert!(header.contains("GENERATED CODE - DO NOT EDIT"));
    }

    #[test]
    fn test_bindings_header_contains_timestamp() {
        let header = generate_bindings_header_with_time(test_timestamp());
        assert!(header.contains("2025-01-15T10:30:00Z"));
    }

    #[test]
    fn test_bindings_header_contains_source_info() {
        let header = generate_bindings_header_with_time(test_timestamp());
        assert!(header.contains("core/src/ffi/contracts/*.ffi-contract.json"));
    }

    #[test]
    fn test_bindings_header_contains_regenerate_instructions() {
        let header = generate_bindings_header_with_time(test_timestamp());
        assert!(header.contains("cargo run --bin generate-dart-bindings"));
        assert!(header.contains("just gen-dart-bindings"));
    }

    #[test]
    fn test_bindings_header_contains_imports() {
        let header = generate_bindings_header_with_time(test_timestamp());
        assert!(header.contains("import 'dart:ffi';"));
        assert!(header.contains("import 'dart:convert';"));
        assert!(header.contains("import 'package:ffi/ffi.dart';"));
    }

    #[test]
    fn test_bindings_header_contains_lint_ignores() {
        let header = generate_bindings_header_with_time(test_timestamp());
        assert!(header.contains("ignore_for_file:"));
    }

    #[test]
    fn test_models_header_contains_warning() {
        let header = generate_models_header_with_time(test_timestamp());
        assert!(header.contains("GENERATED CODE - DO NOT EDIT"));
    }

    #[test]
    fn test_models_header_contains_timestamp() {
        let header = generate_models_header_with_time(test_timestamp());
        assert!(header.contains("2025-01-15T10:30:00Z"));
    }

    #[test]
    fn test_models_header_contains_source_info() {
        let header = generate_models_header_with_time(test_timestamp());
        assert!(header.contains("core/src/ffi/contracts/*.ffi-contract.json"));
    }

    #[test]
    fn test_format_timestamp() {
        let ts = Utc.with_ymd_and_hms(2025, 12, 31, 23, 59, 59).unwrap();
        assert_eq!(format_timestamp(ts), "2025-12-31T23:59:59Z");
    }

    #[test]
    fn test_generate_bindings_header_not_empty() {
        let header = generate_bindings_header();
        assert!(!header.is_empty());
        assert!(header.contains("GENERATED CODE"));
    }

    #[test]
    fn test_generate_models_header_not_empty() {
        let header = generate_models_header();
        assert!(!header.is_empty());
        assert!(header.contains("GENERATED CODE"));
    }
}
