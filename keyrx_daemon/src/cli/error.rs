//! CLI error formatting utilities.
//!
//! This module provides functions for formatting DaemonError instances for
//! CLI output, supporting both human-readable colored output and structured
//! JSON output for machine parsing.

use colored::Colorize;
use serde::Serialize;

use crate::error::{
    CliError, ConfigError, DaemonError, PlatformError, RecorderError, RegistryError,
    SerializationError, SocketError, WebError,
};

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
    let code = error_code(error);

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
        DaemonError::Platform(e) => extract_platform_error_details(e),
        DaemonError::Config(e) => extract_config_error_details(e),
        DaemonError::Web(e) => extract_web_error_details(e),
        DaemonError::Cli(e) => extract_cli_error_details(e),
        DaemonError::Serialization(e) => extract_serialization_error_details(e),
        DaemonError::Socket(e) => extract_socket_error_details(e),
        DaemonError::Registry(e) => extract_registry_error_details(e),
        DaemonError::Recorder(e) => extract_recorder_error_details(e),
        DaemonError::Core(e) => (
            "Core".to_string(),
            e.to_string(),
            Some("Check core library documentation for details".to_string()),
            None,
        ),
    }
}

/// Extract details from PlatformError.
fn extract_platform_error_details(
    error: &PlatformError,
) -> (String, String, Option<String>, Option<String>) {
    match error {
        PlatformError::DeviceAccess {
            device,
            reason,
            suggestion,
        } => (
            "Platform".to_string(),
            format!("Failed to access device '{}': {}", device, reason),
            Some(suggestion.clone()),
            Some(format!("Device: {}", device)),
        ),
        PlatformError::InjectionFailed { reason, suggestion } => (
            "Platform".to_string(),
            format!("Failed to inject keyboard event: {}", reason),
            Some(suggestion.clone()),
            None,
        ),
        PlatformError::Unsupported { operation } => (
            "Platform".to_string(),
            format!("Operation not supported: {}", operation),
            Some("This operation may not be available on your platform".to_string()),
            None,
        ),
        PlatformError::InitializationFailed { reason } => (
            "Platform".to_string(),
            format!("Platform initialization failed: {}", reason),
            Some("Try restarting the daemon or checking system permissions".to_string()),
            None,
        ),
        PlatformError::Poisoned(msg) => (
            "Platform".to_string(),
            format!("Internal error (mutex poisoned): {}", msg),
            Some("This is likely a bug. Please report it".to_string()),
            None,
        ),
        PlatformError::DeviceError(msg) => (
            "Platform".to_string(),
            format!("Device operation failed: {}", msg),
            Some("Check device permissions and availability".to_string()),
            None,
        ),
        PlatformError::Io(e) => (
            "Platform".to_string(),
            format!("IO error: {}", e),
            Some("Check file permissions and disk space".to_string()),
            None,
        ),
    }
}

/// Extract details from ConfigError.
fn extract_config_error_details(
    error: &ConfigError,
) -> (String, String, Option<String>, Option<String>) {
    match error {
        ConfigError::FileNotFound { path } => (
            "Config".to_string(),
            format!("Configuration file not found: {:?}", path),
            Some(format!(
                "Create a configuration file at {:?} or use --config to specify a different location",
                path
            )),
            None,
        ),
        ConfigError::ParseError { path, reason } => (
            "Config".to_string(),
            format!("Failed to parse configuration at {:?}: {}", path, reason),
            Some("Check the configuration file syntax and format".to_string()),
            Some(format!("File: {:?}", path)),
        ),
        ConfigError::InvalidProfile { name, reason } => (
            "Config".to_string(),
            format!("Invalid profile '{}': {}", name, reason),
            Some("Review the profile configuration and ensure all required fields are present".to_string()),
            Some(format!("Profile: {}", name)),
        ),
        ConfigError::CompilationFailed { reason } => (
            "Config".to_string(),
            format!("Failed to compile configuration: {}", reason),
            Some("Check the Rhai script syntax and available functions".to_string()),
            None,
        ),
        ConfigError::Core(e) => (
            "Config".to_string(),
            format!("Core library error: {}", e),
            Some("This may indicate invalid key codes or state definitions".to_string()),
            None,
        ),
        ConfigError::Io(e) => (
            "Config".to_string(),
            format!("IO error while accessing configuration: {}", e),
            Some("Check file permissions and disk space".to_string()),
            None,
        ),
    }
}

/// Extract details from WebError.
fn extract_web_error_details(error: &WebError) -> (String, String, Option<String>, Option<String>) {
    match error {
        WebError::BindFailed { address, reason } => (
            "Web".to_string(),
            format!("Failed to bind web server to {}: {}", address, reason),
            Some(
                "Check if the port is already in use or if you have permission to bind".to_string(),
            ),
            Some(format!("Address: {}", address)),
        ),
        WebError::InvalidRequest { reason } => (
            "Web".to_string(),
            format!("Invalid API request: {}", reason),
            Some("Check the request format and required parameters".to_string()),
            None,
        ),
        WebError::WebSocketError { reason } => (
            "Web".to_string(),
            format!("WebSocket error: {}", reason),
            Some("Check the WebSocket connection and network configuration".to_string()),
            None,
        ),
        WebError::StaticFileError { path, reason } => (
            "Web".to_string(),
            format!("Failed to serve static file {:?}: {}", path, reason),
            Some("Ensure the UI files are properly embedded in the binary".to_string()),
            Some(format!("File: {:?}", path)),
        ),
        WebError::Io(e) => (
            "Web".to_string(),
            format!("IO error in web server: {}", e),
            Some("Check network configuration and file permissions".to_string()),
            None,
        ),
    }
}

/// Extract details from CliError.
fn extract_cli_error_details(error: &CliError) -> (String, String, Option<String>, Option<String>) {
    match error {
        CliError::InvalidArguments { reason } => (
            "CLI".to_string(),
            format!("Invalid arguments: {}", reason),
            Some("Run with --help to see available options".to_string()),
            None,
        ),
        CliError::CommandFailed { command, reason } => (
            "CLI".to_string(),
            format!("Command '{}' failed: {}", command, reason),
            Some(format!("Check the '{}' command usage with --help", command)),
            Some(format!("Command: {}", command)),
        ),
        CliError::OutputError { reason } => (
            "CLI".to_string(),
            format!("Failed to format output: {}", reason),
            Some("This is likely a bug. Please report it".to_string()),
            None,
        ),
    }
}

/// Extract details from SerializationError.
fn extract_serialization_error_details(
    error: &SerializationError,
) -> (String, String, Option<String>, Option<String>) {
    match error {
        SerializationError::InvalidMagic { expected, found } => (
            "Serialization".to_string(),
            format!(
                "Invalid binary format: expected magic {:#010x}, found {:#010x}",
                expected, found
            ),
            Some("The file may be corrupted or not a valid .krx file".to_string()),
            None,
        ),
        SerializationError::InvalidVersion { expected, found } => (
            "Serialization".to_string(),
            format!(
                "Unsupported binary version: expected {}, found {}",
                expected, found
            ),
            Some("Recompile the configuration with the current compiler version".to_string()),
            None,
        ),
        SerializationError::InvalidSize { expected, found } => (
            "Serialization".to_string(),
            format!(
                "Invalid binary size: expected {} bytes, found {} bytes",
                expected, found
            ),
            Some("The file may be corrupted or truncated".to_string()),
            None,
        ),
        SerializationError::CorruptedData(msg) => (
            "Serialization".to_string(),
            format!("Corrupted data: {}", msg),
            Some("Try recompiling the configuration".to_string()),
            None,
        ),
        SerializationError::Io(e) => (
            "Serialization".to_string(),
            format!("IO error: {}", e),
            Some("Check file permissions and disk space".to_string()),
            None,
        ),
    }
}

/// Extract details from SocketError.
fn extract_socket_error_details(
    error: &SocketError,
) -> (String, String, Option<String>, Option<String>) {
    match error {
        SocketError::BindFailed { path, error } => (
            "Socket".to_string(),
            format!("Failed to bind socket at {:?}: {}", path, error),
            Some("Check if another instance is running or if you have permission".to_string()),
            Some(format!("Path: {:?}", path)),
        ),
        SocketError::ListenFailed { error } => (
            "Socket".to_string(),
            format!("Failed to listen on socket: {}", error),
            Some("Check system resources and permissions".to_string()),
            None,
        ),
        SocketError::NotConnected => (
            "Socket".to_string(),
            "Socket not connected".to_string(),
            Some("Ensure the daemon is running before attempting this operation".to_string()),
            None,
        ),
        SocketError::AlreadyConnected => (
            "Socket".to_string(),
            "Socket already connected".to_string(),
            Some("Close the existing connection before reconnecting".to_string()),
            None,
        ),
        SocketError::Io(e) => (
            "Socket".to_string(),
            format!("IO error: {}", e),
            Some("Check network configuration and permissions".to_string()),
            None,
        ),
    }
}

/// Extract details from RegistryError.
fn extract_registry_error_details(
    error: &RegistryError,
) -> (String, String, Option<String>, Option<String>) {
    match error {
        RegistryError::IOError(kind) => (
            "Registry".to_string(),
            format!("IO error: {:?}", kind),
            Some("Check file permissions and disk space".to_string()),
            None,
        ),
        RegistryError::CorruptedRegistry(msg) => (
            "Registry".to_string(),
            format!("Corrupted registry: {}", msg),
            Some("Delete the registry file to rebuild it".to_string()),
            None,
        ),
        RegistryError::FailedToLoad(kind) => (
            "Registry".to_string(),
            format!("Failed to load registry: {:?}", kind),
            Some("Check if the registry file exists and is readable".to_string()),
            None,
        ),
    }
}

/// Extract details from RecorderError.
fn extract_recorder_error_details(
    error: &RecorderError,
) -> (String, String, Option<String>, Option<String>) {
    match error {
        RecorderError::NotRecording => (
            "Recorder".to_string(),
            "Not currently recording".to_string(),
            Some("Start recording before attempting to stop".to_string()),
            None,
        ),
        RecorderError::AlreadyRecording => (
            "Recorder".to_string(),
            "Already recording".to_string(),
            Some("Stop the current recording before starting a new one".to_string()),
            None,
        ),
        RecorderError::PlaybackFailed(frame) => (
            "Recorder".to_string(),
            format!("Playback failed at frame {}", frame),
            Some("Check the recorded macro for invalid events".to_string()),
            Some(format!("Frame: {}", frame)),
        ),
        RecorderError::BufferFull(max) => (
            "Recorder".to_string(),
            format!("Recording buffer full (max {} events)", max),
            Some("Stop recording or increase the buffer size".to_string()),
            None,
        ),
        RecorderError::MutexPoisoned(msg) => (
            "Recorder".to_string(),
            format!("Internal error (mutex poisoned): {}", msg),
            Some("This is likely a bug. Please report it".to_string()),
            None,
        ),
    }
}

/// Get numeric error code for a DaemonError.
///
/// Error codes are used for programmatic error handling in JSON output.
/// They are organized by error category:
/// - 1000-1999: CLI errors
/// - 2000-2999: Configuration errors
/// - 3000-3999: Platform errors
/// - 4000-4999: Web/API errors
/// - 5000-5999: Serialization errors
/// - 6000-6999: Socket errors
/// - 7000-7999: Registry errors
/// - 8000-8999: Recorder errors
/// - 9000-9999: Core errors and other
fn error_code(error: &DaemonError) -> u32 {
    match error {
        // CLI errors: 1000-1999
        DaemonError::Cli(CliError::InvalidArguments { .. }) => 1000,
        DaemonError::Cli(CliError::CommandFailed { .. }) => 1001,
        DaemonError::Cli(CliError::OutputError { .. }) => 1002,

        // Configuration errors: 2000-2999
        DaemonError::Config(ConfigError::FileNotFound { .. }) => 2000,
        DaemonError::Config(ConfigError::ParseError { .. }) => 2001,
        DaemonError::Config(ConfigError::InvalidProfile { .. }) => 2002,
        DaemonError::Config(ConfigError::CompilationFailed { .. }) => 2003,
        DaemonError::Config(ConfigError::Core(_)) => 2004,
        DaemonError::Config(ConfigError::Io(_)) => 2005,

        // Platform errors: 3000-3999
        DaemonError::Platform(PlatformError::DeviceAccess { .. }) => 3000,
        DaemonError::Platform(PlatformError::InjectionFailed { .. }) => 3001,
        DaemonError::Platform(PlatformError::Unsupported { .. }) => 3002,
        DaemonError::Platform(PlatformError::InitializationFailed { .. }) => 3003,
        DaemonError::Platform(PlatformError::Poisoned(_)) => 3004,
        DaemonError::Platform(PlatformError::DeviceError(_)) => 3005,
        DaemonError::Platform(PlatformError::Io(_)) => 3006,

        // Web errors: 4000-4999
        DaemonError::Web(WebError::BindFailed { .. }) => 4000,
        DaemonError::Web(WebError::InvalidRequest { .. }) => 4001,
        DaemonError::Web(WebError::WebSocketError { .. }) => 4002,
        DaemonError::Web(WebError::StaticFileError { .. }) => 4003,
        DaemonError::Web(WebError::Io(_)) => 4004,

        // Serialization errors: 5000-5999
        DaemonError::Serialization(SerializationError::InvalidMagic { .. }) => 5000,
        DaemonError::Serialization(SerializationError::InvalidVersion { .. }) => 5001,
        DaemonError::Serialization(SerializationError::InvalidSize { .. }) => 5002,
        DaemonError::Serialization(SerializationError::CorruptedData(_)) => 5003,
        DaemonError::Serialization(SerializationError::Io(_)) => 5004,

        // Socket errors: 6000-6999
        DaemonError::Socket(SocketError::BindFailed { .. }) => 6000,
        DaemonError::Socket(SocketError::ListenFailed { .. }) => 6001,
        DaemonError::Socket(SocketError::NotConnected) => 6002,
        DaemonError::Socket(SocketError::AlreadyConnected) => 6003,
        DaemonError::Socket(SocketError::Io(_)) => 6004,

        // Registry errors: 7000-7999
        DaemonError::Registry(RegistryError::IOError(_)) => 7000,
        DaemonError::Registry(RegistryError::CorruptedRegistry(_)) => 7001,
        DaemonError::Registry(RegistryError::FailedToLoad(_)) => 7002,

        // Recorder errors: 8000-8999
        DaemonError::Recorder(RecorderError::NotRecording) => 8000,
        DaemonError::Recorder(RecorderError::AlreadyRecording) => 8001,
        DaemonError::Recorder(RecorderError::PlaybackFailed(_)) => 8002,
        DaemonError::Recorder(RecorderError::BufferFull(_)) => 8003,
        DaemonError::Recorder(RecorderError::MutexPoisoned(_)) => 8004,

        // Core and other errors: 9000-9999
        DaemonError::Core(_) => 9000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_cli_error_json() {
        let error = DaemonError::Cli(CliError::InvalidArguments {
            reason: "test reason".to_string(),
        });

        let output = format_cli_error(&error, true);
        assert!(output.contains("\"success\":false"));
        assert!(output.contains("\"type\":\"cli\""));
        assert!(output.contains("\"code\":1000"));
    }

    #[test]
    fn test_format_cli_error_human() {
        let error = DaemonError::Cli(CliError::InvalidArguments {
            reason: "test reason".to_string(),
        });

        let output = format_cli_error(&error, false);
        assert!(output.contains("Error:"));
        assert!(output.contains("Invalid arguments"));
    }

    #[test]
    fn test_format_json_error() {
        let error = DaemonError::Config(ConfigError::FileNotFound {
            path: PathBuf::from("/test/config.toml"),
        });

        let json = format_json_error(&error);
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"type\":\"config\""));
        assert!(json.contains("\"code\":2000"));
    }

    #[test]
    fn test_format_human_error_with_suggestion() {
        let error = DaemonError::Platform(PlatformError::DeviceAccess {
            device: "/dev/input/event0".to_string(),
            reason: "permission denied".to_string(),
            suggestion: "Run with sudo".to_string(),
        });

        let output = format_human_error(&error);
        assert!(output.contains("Error:"));
        assert!(output.contains("Failed to access device"));
        assert!(output.contains("Suggestion:"));
        assert!(output.contains("Run with sudo"));
    }

    #[test]
    fn test_error_code_mapping() {
        assert_eq!(
            error_code(&DaemonError::Cli(CliError::InvalidArguments {
                reason: "test".to_string()
            })),
            1000
        );
        assert_eq!(
            error_code(&DaemonError::Config(ConfigError::FileNotFound {
                path: PathBuf::from("/test")
            })),
            2000
        );
        assert_eq!(
            error_code(&DaemonError::Platform(PlatformError::DeviceAccess {
                device: "test".to_string(),
                reason: "test".to_string(),
                suggestion: "test".to_string()
            })),
            3000
        );
    }

    #[test]
    fn test_json_error_response_serialization() {
        let response = JsonErrorResponse {
            success: false,
            error_type: "test".to_string(),
            message: "test message".to_string(),
            code: 1234,
            suggestion: Some("test suggestion".to_string()),
            context: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"type\":\"test\""));
        assert!(json.contains("\"code\":1234"));
        assert!(json.contains("\"suggestion\":\"test suggestion\""));
        assert!(!json.contains("context"));
    }

    #[test]
    fn test_extract_platform_error_details() {
        let error = PlatformError::DeviceAccess {
            device: "/dev/input/event0".to_string(),
            reason: "permission denied".to_string(),
            suggestion: "Run with sudo".to_string(),
        };

        let (error_type, message, suggestion, context) = extract_platform_error_details(&error);
        assert_eq!(error_type, "Platform");
        assert!(message.contains("Failed to access device"));
        assert!(suggestion.unwrap().contains("Run with sudo"));
        assert!(context.unwrap().contains("/dev/input/event0"));
    }

    #[test]
    fn test_extract_config_error_details() {
        let error = ConfigError::ParseError {
            path: PathBuf::from("/test/config.toml"),
            reason: "invalid syntax".to_string(),
        };

        let (error_type, message, suggestion, context) = extract_config_error_details(&error);
        assert_eq!(error_type, "Config");
        assert!(message.contains("Failed to parse"));
        assert!(suggestion.is_some());
        assert!(context.unwrap().contains("/test/config.toml"));
    }
}
