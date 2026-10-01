//! Per-domain error detail extraction (message, suggestion, context).

use crate::error::{
    CliError, ConfigError, PlatformError, RecorderError, RegistryError, SerializationError,
    SocketError, WebError,
};

/// Extract details from PlatformError.
pub(super) fn extract_platform_error_details(
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
pub(super) fn extract_config_error_details(
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
        ConfigError::Profile(msg) => (
            "Config".to_string(),
            format!("Profile error: {}", msg),
            Some("Check profile configuration and ensure profile exists".to_string()),
            None,
        ),
        ConfigError::Generator(msg) => (
            "Config".to_string(),
            format!("Generator error: {}", msg),
            Some("Check Rhai configuration syntax and structure".to_string()),
            None,
        ),
    }
}

/// Extract details from WebError.
pub(super) fn extract_web_error_details(
    error: &WebError,
) -> (String, String, Option<String>, Option<String>) {
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
pub(super) fn extract_cli_error_details(
    error: &CliError,
) -> (String, String, Option<String>, Option<String>) {
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
        CliError::Reported => (
            "CLI".to_string(),
            "Command failed (details were already printed)".to_string(),
            None,
            None,
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
pub(super) fn extract_serialization_error_details(
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
pub(super) fn extract_socket_error_details(
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
pub(super) fn extract_registry_error_details(
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
pub(super) fn extract_recorder_error_details(
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
