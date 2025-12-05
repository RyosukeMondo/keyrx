//! Shared output formatting contracts for CLI commands.

use serde::Serialize;

/// Supported output formats for CLI output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable text output.
    #[default]
    Human,
    /// JSON output for programmatic parsing.
    Json,
    /// Aligned table output for human readability.
    Table,
}

/// Common interface for rendering CLI output across formats.
pub trait OutputFormatter {
    /// Format a success message.
    fn format_success(&self, message: &str) -> String;

    /// Format an error message.
    fn format_error(&self, message: &str) -> String;

    /// Format a warning message.
    fn format_warning(&self, message: &str) -> String;

    /// Format structured data payloads.
    fn format_data<T: Serialize + ?Sized>(&self, data: &T) -> serde_json::Result<String>;
}
