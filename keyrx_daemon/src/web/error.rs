//! Web error handling and HTTP response conversion.
//!
//! This module implements the `IntoResponse` trait for `DaemonError`,
//! enabling automatic conversion of daemon errors into HTTP responses
//! with appropriate status codes and JSON error bodies.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::error::{
    CliError, ConfigError, DaemonError, PlatformError, RecorderError, RegistryError,
    SerializationError, SocketError, WebError,
};

/// Implements HTTP response conversion for DaemonError.
///
/// This implementation maps daemon error types to appropriate HTTP status codes
/// and returns structured JSON error responses. Error messages are sanitized
/// to avoid exposing internal implementation details to API clients.
///
/// # Response Format
///
/// ```json
/// {
///   "success": false,
///   "error": {
///     "code": "ERROR_CODE",
///     "message": "User-friendly error message"
///   }
/// }
/// ```
///
/// # Status Code Mapping
///
/// - `ConfigError` → 400 Bad Request (client-side configuration issues)
/// - `WebError::InvalidRequest` → 400 Bad Request
/// - `WebError::BindFailed` → 500 Internal Server Error
/// - `WebError::WebSocketError` → 500 Internal Server Error
/// - `WebError::StaticFileError` → 404 Not Found
/// - `CliError` → 400 Bad Request (invalid arguments or commands)
/// - `PlatformError` → 500 Internal Server Error (OS-level failures)
/// - `SerializationError` → 500 Internal Server Error (data corruption)
/// - `SocketError` → 500 Internal Server Error (IPC failures)
/// - `RegistryError` → 500 Internal Server Error (registry failures)
/// - `RecorderError` → 400 Bad Request (invalid state for operation)
/// - `CoreError` → 500 Internal Server Error (core library failures)
///
/// # Examples
///
/// ```
/// use keyrx_daemon::error::{DaemonError, ConfigError};
/// use std::path::PathBuf;
/// use axum::response::IntoResponse;
///
/// async fn handler() -> Result<String, DaemonError> {
///     Err(ConfigError::FileNotFound {
///         path: PathBuf::from("/etc/keyrx/config.toml"),
///     }.into())
/// }
///
/// // Returns 400 Bad Request with JSON error body
/// ```
impl IntoResponse for DaemonError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            // Initialization errors - server issues (500 Internal Server Error)
            DaemonError::Init(init_err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INIT_ERROR",
                format!("Initialization failed: {}", init_err),
            ),

            // Configuration errors - client issues (400 Bad Request)
            DaemonError::Config(config_err) => match config_err {
                ConfigError::FileNotFound { path } => (
                    StatusCode::BAD_REQUEST,
                    "CONFIG_FILE_NOT_FOUND",
                    format!("Configuration file not found: {}", path.display()),
                ),
                ConfigError::ParseError { path, reason } => (
                    StatusCode::BAD_REQUEST,
                    "CONFIG_PARSE_ERROR",
                    format!(
                        "Failed to parse configuration at {}: {}",
                        path.display(),
                        reason
                    ),
                ),
                ConfigError::InvalidProfile { name, reason } => (
                    StatusCode::BAD_REQUEST,
                    "INVALID_PROFILE",
                    format!("Invalid profile '{}': {}", name, reason),
                ),
                ConfigError::CompilationFailed { reason } => (
                    StatusCode::BAD_REQUEST,
                    "CONFIG_COMPILATION_FAILED",
                    format!("Configuration compilation failed: {}", reason),
                ),
                ConfigError::Core(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "CORE_ERROR",
                    "Internal core library error occurred".to_string(),
                ),
                ConfigError::Io(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "IO_ERROR",
                    "File system error occurred while accessing configuration".to_string(),
                ),
                ConfigError::Profile(msg) => (
                    StatusCode::BAD_REQUEST,
                    "PROFILE_ERROR",
                    format!("Profile error: {}", msg),
                ),
                ConfigError::Generator(msg) => (
                    StatusCode::BAD_REQUEST,
                    "GENERATOR_ERROR",
                    format!("Generator error: {}", msg),
                ),
            },

            // Web errors - mixed status codes
            DaemonError::Web(web_err) => match web_err {
                WebError::InvalidRequest { reason } => (
                    StatusCode::BAD_REQUEST,
                    "INVALID_REQUEST",
                    format!("Invalid request: {}", reason),
                ),
                WebError::BindFailed { address, reason } => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "BIND_FAILED",
                    format!("Failed to bind server to {}: {}", address, reason),
                ),
                WebError::WebSocketError { reason } => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "WEBSOCKET_ERROR",
                    format!("WebSocket error: {}", reason),
                ),
                WebError::StaticFileError { path, .. } => (
                    StatusCode::NOT_FOUND,
                    "STATIC_FILE_NOT_FOUND",
                    format!("Static file not found: {}", path.display()),
                ),
                WebError::Io(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "IO_ERROR",
                    "File system error occurred".to_string(),
                ),
            },

            // CLI errors - invalid arguments (400 Bad Request)
            DaemonError::Cli(cli_err) => match cli_err {
                CliError::InvalidArguments { reason } => (
                    StatusCode::BAD_REQUEST,
                    "INVALID_ARGUMENTS",
                    format!("Invalid arguments: {}", reason),
                ),
                CliError::CommandFailed { command, reason } => (
                    StatusCode::BAD_REQUEST,
                    "COMMAND_FAILED",
                    format!("Command '{}' failed: {}", command, reason),
                ),
                CliError::OutputError { reason } => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "OUTPUT_ERROR",
                    format!("Failed to format output: {}", reason),
                ),
            },

            // Platform errors - OS-level failures (500 Internal Server Error)
            DaemonError::Platform(platform_err) => match platform_err {
                PlatformError::DeviceAccess { device, reason, .. } => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DEVICE_ACCESS_FAILED",
                    format!("Failed to access device '{}': {}", device, reason),
                ),
                PlatformError::InjectionFailed { reason, .. } => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INJECTION_FAILED",
                    format!("Failed to inject keyboard event: {}", reason),
                ),
                PlatformError::Unsupported { operation } => (
                    StatusCode::NOT_IMPLEMENTED,
                    "OPERATION_NOT_SUPPORTED",
                    format!("Operation not supported: {}", operation),
                ),
                PlatformError::InitializationFailed { reason } => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "PLATFORM_INIT_FAILED",
                    format!("Platform initialization failed: {}", reason),
                ),
                PlatformError::Poisoned(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "MUTEX_POISONED",
                    "Internal synchronization error occurred".to_string(),
                ),
                PlatformError::DeviceError(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DEVICE_ERROR",
                    format!("Device operation failed: {}", msg),
                ),
                PlatformError::Io(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "IO_ERROR",
                    "Device I/O error occurred".to_string(),
                ),
            },

            // Serialization errors - data corruption (500 Internal Server Error)
            DaemonError::Serialization(ser_err) => match ser_err {
                SerializationError::InvalidMagic { expected, found } => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INVALID_MAGIC",
                    format!(
                        "Invalid binary format: expected magic {:#010x}, found {:#010x}",
                        expected, found
                    ),
                ),
                SerializationError::InvalidVersion { expected, found } => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INVALID_VERSION",
                    format!(
                        "Unsupported binary version: expected {}, found {}",
                        expected, found
                    ),
                ),
                SerializationError::InvalidSize { expected, found } => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INVALID_SIZE",
                    format!(
                        "Invalid buffer size: expected {} bytes, found {} bytes",
                        expected, found
                    ),
                ),
                SerializationError::CorruptedData(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "CORRUPTED_DATA",
                    format!("Data corrupted: {}", msg),
                ),
                SerializationError::Io(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "IO_ERROR",
                    "Serialization I/O error occurred".to_string(),
                ),
            },

            // Socket errors - IPC failures (500 Internal Server Error)
            DaemonError::Socket(socket_err) => match socket_err {
                SocketError::BindFailed { path, error } => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "SOCKET_BIND_FAILED",
                    format!("Failed to bind socket at {}: {}", path.display(), error),
                ),
                SocketError::ListenFailed { error } => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "SOCKET_LISTEN_FAILED",
                    format!("Failed to listen on socket: {}", error),
                ),
                SocketError::NotConnected => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "SOCKET_NOT_CONNECTED",
                    "Socket not connected".to_string(),
                ),
                SocketError::AlreadyConnected => (
                    StatusCode::CONFLICT,
                    "SOCKET_ALREADY_CONNECTED",
                    "Socket already connected".to_string(),
                ),
                SocketError::Io(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "IO_ERROR",
                    "Socket I/O error occurred".to_string(),
                ),
            },

            // Registry errors - registry failures (500 Internal Server Error)
            DaemonError::Registry(registry_err) => match registry_err {
                RegistryError::IOError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "REGISTRY_IO_ERROR",
                    "Registry I/O error occurred".to_string(),
                ),
                RegistryError::CorruptedRegistry(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "REGISTRY_CORRUPTED",
                    format!("Registry corrupted: {}", msg),
                ),
                RegistryError::FailedToLoad(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "REGISTRY_LOAD_FAILED",
                    "Failed to load device registry".to_string(),
                ),
            },

            // Recorder errors - invalid state (400 Bad Request)
            DaemonError::Recorder(recorder_err) => match recorder_err {
                RecorderError::NotRecording => (
                    StatusCode::BAD_REQUEST,
                    "NOT_RECORDING",
                    "Cannot stop recording: not currently recording".to_string(),
                ),
                RecorderError::AlreadyRecording => (
                    StatusCode::BAD_REQUEST,
                    "ALREADY_RECORDING",
                    "Cannot start recording: already recording".to_string(),
                ),
                RecorderError::PlaybackFailed(frame) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "PLAYBACK_FAILED",
                    format!("Macro playback failed at frame {}", frame),
                ),
                RecorderError::BufferFull(max) => (
                    StatusCode::BAD_REQUEST,
                    "BUFFER_FULL",
                    format!("Recording buffer full (max {} events)", max),
                ),
                RecorderError::MutexPoisoned(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "MUTEX_POISONED",
                    "Internal synchronization error occurred".to_string(),
                ),
            },

            // Core errors - core library failures (500 Internal Server Error)
            DaemonError::Core(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "CORE_ERROR",
                "Internal core library error occurred".to_string(),
            ),
        };

        let body = Json(json!({
            "success": false,
            "error": {
                "code": code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{
        CliError, ConfigError, DaemonError, PlatformError, RecorderError, RegistryError,
        SerializationError, SocketError, WebError,
    };
    use axum::http::StatusCode;
    use std::io;
    use std::path::PathBuf;

    #[test]
    fn test_config_file_not_found_returns_400() {
        let err: DaemonError = ConfigError::FileNotFound {
            path: PathBuf::from("/test/config.toml"),
        }
        .into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_config_parse_error_returns_400() {
        let err: DaemonError = ConfigError::ParseError {
            path: PathBuf::from("/test/config.toml"),
            reason: "invalid syntax".to_string(),
        }
        .into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_config_invalid_profile_returns_400() {
        let err: DaemonError = ConfigError::InvalidProfile {
            name: "gaming".to_string(),
            reason: "missing layers".to_string(),
        }
        .into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_web_invalid_request_returns_400() {
        let err: DaemonError = WebError::InvalidRequest {
            reason: "missing parameter".to_string(),
        }
        .into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_web_bind_failed_returns_500() {
        let err: DaemonError = WebError::BindFailed {
            address: "127.0.0.1:3030".to_string(),
            reason: "address in use".to_string(),
        }
        .into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_web_static_file_error_returns_404() {
        let err: DaemonError = WebError::StaticFileError {
            path: PathBuf::from("/static/index.html"),
            reason: "file not found".to_string(),
        }
        .into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_cli_invalid_arguments_returns_400() {
        let err: DaemonError = CliError::InvalidArguments {
            reason: "missing required argument".to_string(),
        }
        .into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_cli_command_failed_returns_400() {
        let err: DaemonError = CliError::CommandFailed {
            command: "activate".to_string(),
            reason: "profile not found".to_string(),
        }
        .into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_platform_device_access_returns_500() {
        let err: DaemonError = PlatformError::DeviceAccess {
            device: "/dev/input/event0".to_string(),
            reason: "permission denied".to_string(),
            suggestion: "Run with sudo".to_string(),
        }
        .into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_platform_injection_failed_returns_500() {
        let err: DaemonError = PlatformError::InjectionFailed {
            reason: "uinput unavailable".to_string(),
            suggestion: "Load uinput module".to_string(),
        }
        .into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_platform_unsupported_returns_501() {
        let err: DaemonError = PlatformError::Unsupported {
            operation: "hotkey capture".to_string(),
        }
        .into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[test]
    fn test_serialization_invalid_magic_returns_500() {
        let err: DaemonError = SerializationError::InvalidMagic {
            expected: 0x4B525800,
            found: 0xFFFFFFFF,
        }
        .into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_socket_not_connected_returns_503() {
        let err: DaemonError = SocketError::NotConnected.into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_socket_already_connected_returns_409() {
        let err: DaemonError = SocketError::AlreadyConnected.into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_recorder_not_recording_returns_400() {
        let err: DaemonError = RecorderError::NotRecording.into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_recorder_already_recording_returns_400() {
        let err: DaemonError = RecorderError::AlreadyRecording.into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_recorder_buffer_full_returns_400() {
        let err: DaemonError = RecorderError::BufferFull(10000).into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_registry_io_error_returns_500() {
        let err: DaemonError = RegistryError::IOError(io::ErrorKind::NotFound).into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_registry_corrupted_returns_500() {
        let err: DaemonError = RegistryError::CorruptedRegistry("invalid json".to_string()).into();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
