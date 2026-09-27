//! CLI error formatting utilities.
//!
//! This module provides functions for formatting DaemonError instances for
//! CLI output, supporting both human-readable colored output and structured
//! JSON output for machine parsing.

mod codes;
mod details;

#[cfg(test)]
mod tests;

use colored::Colorize;
use serde::Serialize;

use crate::error::DaemonError;

/// JSON error response structure for CLI output.
///
/// This structure provides a consistent JSON format for error responses,
/// including error details, error codes, and optional suggestions.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct JsonErrorResponse {
    /// Whether the operation succeeded (always false for errors).
    pub success: bool,
    /// Error type classification.
    #[serde(rename = "type")]
    pub error_type: String,
    /// Human-readable error message.
    pub message: String,
    /// Numeric error code for programmatic handling.
    pub code: u32,
    /// Optional suggestion for resolving the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    /// Optional additional context about the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Format a DaemonError for CLI output.
///
/// This is the main entry point for error formatting. It dispatches to either
/// JSON or human-readable formatting based on the `json` parameter.
///
/// # Arguments
///
/// * `error` - The DaemonError to format
/// * `json` - Whether to output JSON format (true) or human-readable format (false)
///
/// # Returns
///
/// Formatted error string ready for output
///
/// # Examples
///
/// ```
/// use keyrx_daemon::cli::error::format_cli_error;
/// use keyrx_daemon::error::{DaemonError, ConfigError};
/// use std::path::PathBuf;
///
/// let error = DaemonError::Config(ConfigError::FileNotFound {
///     path: PathBuf::from("/test/config.toml"),
/// });
///
/// // Human-readable format with colors
/// let human_output = format_cli_error(&error, false);
/// assert!(human_output.contains("Configuration file not found"));
///
/// // JSON format for machine parsing
/// let json_output = format_cli_error(&error, true);
/// assert!(json_output.contains("\"success\":false"));
/// ```
pub fn format_cli_error(error: &DaemonError, json: bool) -> String {
    if json {
        format_json_error(error)
    } else {
        format_human_error(error)
    }
}

/// Format error as structured JSON.
///
/// Converts a DaemonError into a JSON string that can be parsed by external
/// tools or scripts. Includes error type, message, code, and optional
/// suggestions for resolution.
///
/// # Arguments
///
/// * `error` - The DaemonError to format
///
/// # Returns
///
/// JSON-formatted error string
///
/// # Examples
///
/// ```
/// use keyrx_daemon::cli::error::format_json_error;
/// use keyrx_daemon::error::{DaemonError, CliError};
///
/// let error = DaemonError::Cli(CliError::InvalidArguments {
///     reason: "missing required field".to_string(),
/// });
///
/// let json = format_json_error(&error);
/// assert!(json.contains("\"type\":\"cli\""));
/// assert!(json.contains("\"code\":1000"));
/// ```
pub fn format_json_error(error: &DaemonError) -> String {
    let response = error_to_json_response(error);
    serde_json::to_string(&response).unwrap_or_else(|_| {
        // Fallback if JSON serialization somehow fails
        r#"{"success":false,"type":"internal","message":"Failed to serialize error","code":9999}"#
            .to_string()
    })
}

/// Format error as human-readable text with colors and suggestions.
///
/// Produces colored, multi-line output optimized for terminal display.
/// Includes error type, message, and context-specific suggestions for
/// resolving the issue.
///
/// # Arguments
///
/// * `error` - The DaemonError to format
///
/// # Returns
///
/// Human-readable error string with ANSI color codes
///
/// # Examples
///
/// ```
/// use keyrx_daemon::cli::error::format_human_error;
/// use keyrx_daemon::error::{DaemonError, PlatformError};
///
/// let error = DaemonError::Platform(PlatformError::DeviceAccess {
///     device: "/dev/input/event0".to_string(),
///     reason: "permission denied".to_string(),
///     suggestion: "Run with sudo or add user to input group".to_string(),
/// });
///
/// let output = format_human_error(&error);
/// // Output will include colors, error message, and suggestion
/// assert!(output.contains("Failed to access device"));
/// assert!(output.contains("Run with sudo"));
/// ```
pub fn format_human_error(error: &DaemonError) -> String {
    let (error_type, message, suggestion, context) = extract_error_details(error);

    let mut output = String::new();

    // Error header with type
    output.push_str(&format!(
        "{} [{}]\n",
        "Error:".red().bold(),
        error_type.yellow()
    ));

    // Main error message
    output.push_str(&format!("  {}\n", message));

    // Context if available
    if let Some(ctx) = context {
        output.push_str(&format!("\n{}\n", "Context:".cyan().bold()));
        output.push_str(&format!("  {}\n", ctx));
    }

    // Suggestion if available
    if let Some(sug) = suggestion {
        output.push_str(&format!("\n{}\n", "Suggestion:".green().bold()));
        output.push_str(&format!("  {}\n", sug));
    }

    output
}

/// Convert DaemonError to JsonErrorResponse.
///
/// Internal helper that extracts error details and constructs a JsonErrorResponse
/// with appropriate error codes and categorization.
fn error_to_json_response(error: &DaemonError) -> JsonErrorResponse {
    let (error_type, message, suggestion, context) = extract_error_details(error);
    let code = codes::error_code(error);

    JsonErrorResponse {
        success: false,
        error_type: error_type.to_lowercase(),
        message,
        code,
        suggestion,
        context,
    }
}

/// Extract detailed information from a DaemonError.
///
/// Returns a tuple of (error_type, message, suggestion, context) for use in
/// formatting functions.
fn extract_error_details(error: &DaemonError) -> (String, String, Option<String>, Option<String>) {
    match error {
        DaemonError::Platform(e) => details::extract_platform_error_details(e),
        DaemonError::Config(e) => details::extract_config_error_details(e),
        DaemonError::Web(e) => details::extract_web_error_details(e),
        DaemonError::Cli(e) => details::extract_cli_error_details(e),
        DaemonError::Serialization(e) => details::extract_serialization_error_details(e),
        DaemonError::Socket(e) => details::extract_socket_error_details(e),
        DaemonError::Registry(e) => details::extract_registry_error_details(e),
        DaemonError::Recorder(e) => details::extract_recorder_error_details(e),
        DaemonError::Core(e) => (
            "Core".to_string(),
            e.to_string(),
            Some("Check core library documentation for details".to_string()),
            None,
        ),
        DaemonError::Init(e) => (
            "Init".to_string(),
            e.to_string(),
            Some("Check initialization process and dependencies".to_string()),
            None,
        ),
    }
}
