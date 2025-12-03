//! Automatic markdown documentation generator for error codes.
//!
//! This module provides tools to generate comprehensive markdown documentation
//! from the error registry. It can generate documentation for all errors or
//! filter by category.
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::errors::{ErrorDocGenerator, ErrorCategory};
//!
//! // Generate documentation for all errors
//! let docs = ErrorDocGenerator::generate_all();
//! std::fs::write("docs/errors.md", docs).unwrap();
//!
//! // Generate documentation for a specific category
//! let config_docs = ErrorDocGenerator::generate_category(ErrorCategory::Config);
//! std::fs::write("docs/config-errors.md", config_docs).unwrap();
//! ```

// Writing to String never fails, so unwrap is safe in this module
#![allow(clippy::unwrap_used)]

use super::{ErrorCategory, ErrorRegistry, ErrorSeverity};
use std::fmt::Write;

/// Generator for error documentation in markdown format.
///
/// This struct provides static methods to generate markdown documentation
/// from the error registry. Documentation includes error codes, messages,
/// hints, severity levels, and documentation links.
pub struct ErrorDocGenerator;

impl ErrorDocGenerator {
    /// Generate documentation for all errors in the registry.
    ///
    /// Creates a comprehensive markdown document with a table of contents
    /// and sections for each error category.
    ///
    /// # Returns
    ///
    /// A string containing the full markdown documentation.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let docs = ErrorDocGenerator::generate_all();
    /// std::fs::write("docs/errors.md", docs).unwrap();
    /// ```
    pub fn generate_all() -> String {
        let mut doc = String::new();

        // Header
        writeln!(doc, "# KeyRx Error Codes Reference").unwrap();
        writeln!(doc).unwrap();
        writeln!(
            doc,
            "This document lists all error codes used by KeyRx with their meanings and hints."
        )
        .unwrap();
        writeln!(doc).unwrap();

        // Statistics
        let total_errors = ErrorRegistry::count();
        let categories = ErrorRegistry::categories();
        writeln!(doc, "**Total Errors:** {}", total_errors).unwrap();
        writeln!(doc, "**Categories:** {}", categories.len()).unwrap();
        writeln!(doc).unwrap();

        // Table of Contents
        writeln!(doc, "## Table of Contents").unwrap();
        writeln!(doc).unwrap();
        for category in &categories {
            let count = ErrorRegistry::get_by_category(*category).len();
            writeln!(
                doc,
                "- [{}](#{}): {} errors",
                Self::category_name(*category),
                Self::category_anchor(*category),
                count
            )
            .unwrap();
        }
        writeln!(doc).unwrap();

        // Generate section for each category
        for category in &categories {
            Self::append_category_section(&mut doc, *category);
            writeln!(doc).unwrap();
        }

        doc
    }

    /// Generate documentation for a specific error category.
    ///
    /// Creates a markdown document focused on a single error category,
    /// useful for per-category documentation files.
    ///
    /// # Arguments
    ///
    /// * `category` - The error category to document
    ///
    /// # Returns
    ///
    /// A string containing the markdown documentation for the category.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config_docs = ErrorDocGenerator::generate_category(ErrorCategory::Config);
    /// std::fs::write("docs/config-errors.md", config_docs).unwrap();
    /// ```
    pub fn generate_category(category: ErrorCategory) -> String {
        let mut doc = String::new();

        // Header
        writeln!(doc, "# {} Errors", Self::category_name(category)).unwrap();
        writeln!(doc).unwrap();
        writeln!(doc, "{}", Self::category_description(category)).unwrap();
        writeln!(doc).unwrap();

        let errors = ErrorRegistry::get_by_category(category);
        writeln!(doc, "**Total Errors:** {}", errors.len()).unwrap();
        writeln!(
            doc,
            "**Code Range:** KRX-{}{:03}-KRX-{}{:03}",
            category.prefix(),
            category.base_number(),
            category.prefix(),
            category.base_number() + 999
        )
        .unwrap();
        writeln!(doc).unwrap();

        // Error list
        for def in &errors {
            Self::append_error_entry(&mut doc, def);
        }

        doc
    }

    /// Generate an index page linking to all category documentation.
    ///
    /// Creates a markdown index page that serves as the entry point
    /// to the error documentation, with links to individual category pages.
    ///
    /// # Returns
    ///
    /// A string containing the markdown index page.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let index = ErrorDocGenerator::generate_index();
    /// std::fs::write("docs/errors/index.md", index).unwrap();
    /// ```
    pub fn generate_index() -> String {
        let mut doc = String::new();

        writeln!(doc, "# KeyRx Error Codes").unwrap();
        writeln!(doc).unwrap();
        writeln!(
            doc,
            "KeyRx uses structured error codes in the format `KRX-CXXX` where:"
        )
        .unwrap();
        writeln!(doc).unwrap();
        writeln!(doc, "- `C` is the category prefix (single letter)").unwrap();
        writeln!(doc, "- `XXX` is a three-digit error number").unwrap();
        writeln!(doc).unwrap();

        writeln!(doc, "Each category has a dedicated range of error codes:").unwrap();
        writeln!(doc).unwrap();

        let categories = ErrorRegistry::categories();
        for category in &categories {
            let count = ErrorRegistry::get_by_category(*category).len();
            let range_start = category.base_number();
            let range_end = category.base_number() + 999;

            writeln!(
                doc,
                "## [{}](./{})",
                Self::category_name(*category),
                Self::category_filename(*category)
            )
            .unwrap();
            writeln!(doc).unwrap();
            writeln!(doc, "{}", Self::category_description(*category)).unwrap();
            writeln!(doc).unwrap();
            writeln!(
                doc,
                "- **Code Range:** `KRX-{}{:03}` - `KRX-{}{:03}`",
                category.prefix(),
                range_start,
                category.prefix(),
                range_end
            )
            .unwrap();
            writeln!(doc, "- **Total Errors:** {}", count).unwrap();
            writeln!(doc).unwrap();
        }

        doc
    }

    // Private helper methods

    /// Append a category section to the documentation.
    fn append_category_section(doc: &mut String, category: ErrorCategory) {
        writeln!(doc, "## {}", Self::category_name(category)).unwrap();
        writeln!(doc).unwrap();
        writeln!(doc, "{}", Self::category_description(category)).unwrap();
        writeln!(doc).unwrap();

        let errors = ErrorRegistry::get_by_category(category);
        writeln!(doc, "**Total:** {} errors", errors.len()).unwrap();
        writeln!(doc).unwrap();

        for def in &errors {
            Self::append_error_entry(doc, def);
        }
    }

    /// Append a single error entry to the documentation.
    fn append_error_entry(doc: &mut String, def: &super::ErrorDef) {
        // Error code heading
        writeln!(doc, "### {}", def.code()).unwrap();
        writeln!(doc).unwrap();

        // Severity badge
        let severity_badge = match def.severity() {
            ErrorSeverity::Fatal => "![Fatal](https://img.shields.io/badge/severity-fatal-red)",
            ErrorSeverity::Error => "![Error](https://img.shields.io/badge/severity-error-orange)",
            ErrorSeverity::Warning => {
                "![Warning](https://img.shields.io/badge/severity-warning-yellow)"
            }
            ErrorSeverity::Info => "![Info](https://img.shields.io/badge/severity-info-blue)",
        };
        writeln!(doc, "{}", severity_badge).unwrap();
        writeln!(doc).unwrap();

        // Message template
        writeln!(doc, "**Message:**").unwrap();
        writeln!(doc).unwrap();
        writeln!(doc, "```").unwrap();
        writeln!(doc, "{}", def.message_template()).unwrap();
        writeln!(doc, "```").unwrap();
        writeln!(doc).unwrap();

        // Hint (if present)
        if let Some(hint) = def.hint() {
            writeln!(doc, "**Resolution:**").unwrap();
            writeln!(doc).unwrap();
            writeln!(doc, "{}", hint).unwrap();
            writeln!(doc).unwrap();
        }

        // Documentation link (if present)
        if let Some(link) = def.doc_link() {
            writeln!(doc, "**See also:** [Documentation]({})", link).unwrap();
            writeln!(doc).unwrap();
        }

        writeln!(doc, "---").unwrap();
        writeln!(doc).unwrap();
    }

    /// Get the human-readable name for a category.
    fn category_name(category: ErrorCategory) -> &'static str {
        match category {
            ErrorCategory::Config => "Configuration",
            ErrorCategory::Runtime => "Runtime",
            ErrorCategory::Driver => "Driver",
            ErrorCategory::Validation => "Validation",
            ErrorCategory::Ffi => "FFI",
            ErrorCategory::Internal => "Internal",
        }
    }

    /// Get the description for a category.
    fn category_description(category: ErrorCategory) -> &'static str {
        match category {
            ErrorCategory::Config => {
                "Errors related to configuration file loading, parsing, and validation."
            }
            ErrorCategory::Runtime => {
                "Errors that occur during engine runtime, including processing and state errors."
            }
            ErrorCategory::Driver => {
                "Platform-specific driver errors for Windows and Linux keyboard/mouse drivers."
            }
            ErrorCategory::Validation => {
                "Configuration validation errors including conflict detection and rule validation."
            }
            ErrorCategory::Ffi => {
                "Errors that occur at the FFI boundary between Rust core and Flutter UI."
            }
            ErrorCategory::Internal => {
                "Internal errors that indicate bugs or unexpected states in the application."
            }
        }
    }

    /// Get the anchor name for a category (for TOC links).
    fn category_anchor(category: ErrorCategory) -> &'static str {
        match category {
            ErrorCategory::Config => "configuration",
            ErrorCategory::Runtime => "runtime",
            ErrorCategory::Driver => "driver",
            ErrorCategory::Validation => "validation",
            ErrorCategory::Ffi => "ffi",
            ErrorCategory::Internal => "internal",
        }
    }

    /// Get the filename for a category documentation file.
    fn category_filename(category: ErrorCategory) -> &'static str {
        match category {
            ErrorCategory::Config => "config.md",
            ErrorCategory::Runtime => "runtime.md",
            ErrorCategory::Driver => "driver.md",
            ErrorCategory::Validation => "validation.md",
            ErrorCategory::Ffi => "ffi.md",
            ErrorCategory::Internal => "internal.md",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_name_mapping() {
        assert_eq!(
            ErrorDocGenerator::category_name(ErrorCategory::Config),
            "Configuration"
        );
        assert_eq!(
            ErrorDocGenerator::category_name(ErrorCategory::Runtime),
            "Runtime"
        );
        assert_eq!(
            ErrorDocGenerator::category_name(ErrorCategory::Driver),
            "Driver"
        );
    }

    #[test]
    fn category_anchor_mapping() {
        assert_eq!(
            ErrorDocGenerator::category_anchor(ErrorCategory::Config),
            "configuration"
        );
        assert_eq!(
            ErrorDocGenerator::category_anchor(ErrorCategory::Runtime),
            "runtime"
        );
    }

    #[test]
    fn category_filename_mapping() {
        assert_eq!(
            ErrorDocGenerator::category_filename(ErrorCategory::Config),
            "config.md"
        );
        assert_eq!(
            ErrorDocGenerator::category_filename(ErrorCategory::Runtime),
            "runtime.md"
        );
    }

    #[test]
    fn generate_all_structure() {
        // Should not panic and should contain basic structure
        let docs = ErrorDocGenerator::generate_all();
        assert!(docs.contains("# KeyRx Error Codes Reference"));
        assert!(docs.contains("## Table of Contents"));
    }

    #[test]
    fn generate_index_structure() {
        let index = ErrorDocGenerator::generate_index();
        assert!(index.contains("# KeyRx Error Codes"));
        assert!(index.contains("KRX-CXXX"));
    }

    #[test]
    fn generate_category_structure() {
        let docs = ErrorDocGenerator::generate_category(ErrorCategory::Config);
        assert!(docs.contains("# Configuration Errors"));
        assert!(docs.contains("**Code Range:**"));
    }
}
