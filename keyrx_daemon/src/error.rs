//! Error types for the KeyRx daemon.
//!
//! This module defines a comprehensive error type hierarchy for the daemon,
//! enabling proper error propagation and recovery instead of panics.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

use keyrx_core::error::CoreError;

/// Platform-specific operation errors.
///
/// This error type covers failures in platform-specific operations such as
/// device access, mutex operations, and platform initialization.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PlatformError {
    /// Mutex was poisoned (another thread panicked while holding the lock).
    #[error("Mutex poisoned: {0}")]
    Poisoned(String),

    /// Platform initialization failed.
    #[error("Platform initialization failed: {reason}")]
    InitializationFailed {
        /// Reason for initialization failure.
        reason: String,
    },

    /// Device access failed (e.g., permission denied, device not found).
    #[error("Failed to access device '{device}': {reason}. {suggestion}")]
    DeviceAccess {
        /// Name or path of the device that failed to be accessed.
        device: String,
        /// Reason for the access failure.
        reason: String,
        /// Suggestion for how to resolve the issue.
        suggestion: String,
    },

    /// Keyboard event injection failed.
    #[error("Failed to inject keyboard event: {reason}. {suggestion}")]
    InjectionFailed {
        /// Reason for injection failure.
        reason: String,
        /// Suggestion for recovery or resolution.
        suggestion: String,
    },

    /// Requested platform operation is not supported.
    #[error("Operation not supported on this platform: {operation}")]
    Unsupported {
        /// Description of the unsupported operation.
        operation: String,
    },

    /// Device operation failed (legacy variant, prefer more specific variants).
    #[error("Device operation failed: {0}")]
    DeviceError(String),

    /// IO error occurred during platform operation.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Binary format parsing and serialization errors.
///
/// This error type covers failures when parsing or validating .krx binary files.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SerializationError {
    /// Magic number in binary file doesn't match expected value.
    #[error("Invalid magic number: expected {expected:#010x}, found {found:#010x}")]
    InvalidMagic {
        /// Expected magic number.
        expected: u32,
        /// Actual magic number found in file.
        found: u32,
    },

    /// Version number is not supported.
    #[error("Unsupported version: expected {expected}, found {found}")]
    InvalidVersion {
        /// Expected version number.
        expected: u32,
        /// Actual version number found in file.
        found: u32,
    },

    /// Buffer size doesn't match expected size.
    #[error("Invalid size: expected {expected} bytes, found {found} bytes")]
    InvalidSize {
        /// Expected buffer size in bytes.
        expected: usize,
        /// Actual buffer size in bytes.
        found: usize,
    },

    /// Data is corrupted or malformed.
    #[error("Corrupted data: {0}")]
    CorruptedData(String),

    /// IO error occurred during serialization.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// IPC socket operation errors.
///
/// This error type covers failures in Unix socket or named pipe operations
/// used for inter-process communication.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SocketError {
    /// Failed to bind socket at the specified path.
    #[error("Failed to bind socket at {path:?}: {error}")]
    BindFailed {
        /// Path where socket binding was attempted.
        path: PathBuf,
        /// Underlying IO error.
        error: io::Error,
    },

    /// Failed to listen on the socket.
    #[error("Failed to listen on socket: {error}")]
    ListenFailed {
        /// Underlying IO error.
        error: io::Error,
    },

    /// Attempted operation on disconnected socket.
    #[error("Socket not connected")]
    NotConnected,

    /// Attempted to connect an already connected socket.
    #[error("Socket already connected")]
    AlreadyConnected,

    /// IO error occurred during socket operation.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Device registry operation errors.
///
/// This error type covers failures when loading or saving the device registry.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RegistryError {
    /// IO error occurred during registry operation.
    #[error("IO error: {0:?}")]
    IOError(io::ErrorKind),

    /// Registry file is corrupted.
    #[error("Corrupted registry: {0}")]
    CorruptedRegistry(String),

    /// Failed to load registry file.
    #[error("Failed to load registry: {0:?}")]
    FailedToLoad(io::ErrorKind),
}

/// Macro recording operation errors.
///
/// This error type covers failures in macro recording and playback operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RecorderError {
    /// Attempted to stop recording when not currently recording.
    #[error("Not currently recording")]
    NotRecording,

    /// Attempted to start recording when already recording.
    #[error("Already recording")]
    AlreadyRecording,

    /// Playback failed at the specified frame.
    #[error("Playback failed at frame {0}")]
    PlaybackFailed(usize),

    /// Recording buffer is full.
    #[error("Recording buffer full (max {0} events)")]
    BufferFull(usize),

    /// Mutex poisoned during recorder operation.
    #[error("Mutex poisoned: {0}")]
    MutexPoisoned(String),
}

/// Configuration loading and validation errors.
///
/// This error type covers failures when loading, parsing, or validating
/// configuration files and profiles.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigError {
    /// Configuration file not found.
    #[error("Configuration file not found: {path:?}")]
    FileNotFound {
        /// Path to the missing configuration file.
        path: PathBuf,
    },

    /// Failed to parse configuration file.
    #[error("Failed to parse configuration at {path:?}: {reason}")]
    ParseError {
        /// Path to the configuration file.
        path: PathBuf,
        /// Reason for parse failure.
        reason: String,
    },

    /// Invalid profile configuration.
    #[error("Invalid profile '{name}': {reason}")]
    InvalidProfile {
        /// Name of the invalid profile.
        name: String,
        /// Reason why the profile is invalid.
        reason: String,
    },

    /// Configuration compilation failed.
    #[error("Failed to compile configuration: {reason}")]
    CompilationFailed {
        /// Reason for compilation failure.
        reason: String,
    },

    /// Core library error occurred during configuration processing.
    #[error("Core error: {0}")]
    Core(#[from] CoreError),

    /// IO error occurred during configuration operation.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Profile manager error occurred.
    #[error("Profile error: {0}")]
    Profile(String),

    /// Rhai generator error occurred.
    #[error("Generator error: {0}")]
    Generator(String),
}

impl From<crate::config::profile_manager::ProfileError> for ConfigError {
    fn from(err: crate::config::profile_manager::ProfileError) -> Self {
        ConfigError::Profile(err.to_string())
    }
}

impl From<crate::config::rhai_generator::GeneratorError> for ConfigError {
    fn from(err: crate::config::rhai_generator::GeneratorError) -> Self {
        ConfigError::Generator(err.to_string())
    }
}

/// Web server and API errors.
///
/// This error type covers failures in the embedded web server and REST API.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum WebError {
    /// Failed to bind web server to the specified address.
    #[error("Failed to bind web server to {address}: {reason}")]
    BindFailed {
        /// Address where binding was attempted.
        address: String,
        /// Reason for bind failure.
        reason: String,
    },

    /// Invalid API request.
    #[error("Invalid API request: {reason}")]
    InvalidRequest {
        /// Reason why the request is invalid.
        reason: String,
    },

    /// WebSocket error occurred.
    #[error("WebSocket error: {reason}")]
    WebSocketError {
        /// Reason for WebSocket failure.
        reason: String,
    },

    /// Failed to serve static files.
    #[error("Failed to serve static file {path:?}: {reason}")]
    StaticFileError {
        /// Path to the static file.
        path: PathBuf,
        /// Reason for failure.
        reason: String,
    },

    /// IO error occurred during web operation.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// CLI command errors.
///
/// This error type covers failures when executing CLI commands.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CliError {
    /// Invalid command-line arguments.
    #[error("Invalid arguments: {reason}")]
    InvalidArguments {
        /// Reason why arguments are invalid.
        reason: String,
    },

    /// Command execution failed.
    #[error("Command '{command}' failed: {reason}")]
    CommandFailed {
        /// Name of the command that failed.
        command: String,
        /// Reason for command failure.
        reason: String,
    },

    /// Output formatting error.
    #[error("Failed to format output: {reason}")]
    OutputError {
        /// Reason for output formatting failure.
        reason: String,
    },
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        CliError::OutputError {
            reason: err.to_string(),
        }
    }
}

/// Initialization and startup errors.
///
/// This error type covers failures during daemon initialization and startup,
/// including component initialization, dependency checks, and lock poisoning.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum InitError {
    /// Failed to initialize a component.
    #[error("Failed to initialize {component}: {reason}")]
    ComponentInit {
        /// Name of the component that failed to initialize.
        component: String,
        /// Reason for initialization failure.
        reason: String,
    },

    /// Required dependency is missing or unavailable.
    #[error("Required dependency missing: {dependency}")]
    MissingDependency {
        /// Name of the missing dependency.
        dependency: String,
    },

    /// Configuration directory is not accessible.
    #[error("Configuration directory not accessible: {path:?}")]
    ConfigDirUnavailable {
        /// Path to the inaccessible configuration directory.
        path: PathBuf,
    },

    /// Lock was poisoned by a panicked thread.
    #[error("Lock poisoned in {component}: {details}")]
    LockPoisoned {
        /// Component where the lock was poisoned.
        component: String,
        /// Additional details about the poisoning.
        details: String,
    },

    /// Channel was closed unexpectedly.
    #[error("Channel closed unexpectedly in {context}")]
    ChannelClosed {
        /// Context where the channel closure occurred.
        context: String,
    },

    /// JSON serialization failed.
    #[error("JSON serialization failed: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    /// IO error occurred during initialization.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Top-level daemon error type.
///
/// This is the main error type for the daemon, encompassing all possible
/// error conditions. Module-specific errors automatically convert into
/// this type via `From` implementations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DaemonError {
    /// Initialization or startup error occurred.
    #[error("Initialization error: {0}")]
    Init(#[from] InitError),

    /// Platform-specific error occurred.
    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),

    /// Serialization or deserialization error occurred.
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),

    /// Socket operation error occurred.
    #[error("Socket error: {0}")]
    Socket(#[from] SocketError),

    /// Registry operation error occurred.
    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),

    /// Macro recorder error occurred.
    #[error("Recorder error: {0}")]
    Recorder(#[from] RecorderError),

    /// Configuration error occurred.
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Web server or API error occurred.
    #[error("Web error: {0}")]
    Web(#[from] WebError),

    /// CLI command error occurred.
    #[error("CLI error: {0}")]
    Cli(#[from] CliError),

    /// Core library error occurred.
    #[error("Core error: {0}")]
    Core(#[from] CoreError),
}

/// Result type alias for daemon operations.
///
/// This is a convenience type alias for operations that can fail with a DaemonError.
///
/// # Examples
///
/// ```
/// use keyrx_daemon::error::{DaemonResult, DaemonError, ConfigError};
/// use std::path::PathBuf;
///
/// fn load_config(path: PathBuf) -> DaemonResult<String> {
///     if !path.exists() {
///         return Err(ConfigError::FileNotFound { path }.into());
///     }
///     Ok("config data".to_string())
/// }
/// ```
pub type DaemonResult<T> = Result<T, DaemonError>;

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Error Construction Tests
    // ============================================================================

    #[test]
    fn test_platform_error_construction() {
        let err = PlatformError::Poisoned("test mutex".into());
        assert!(matches!(err, PlatformError::Poisoned(_)));

        let err = PlatformError::InitializationFailed {
            reason: "failed to init".into(),
        };
        assert!(matches!(err, PlatformError::InitializationFailed { .. }));

        let err = PlatformError::DeviceAccess {
            device: "/dev/input/event0".into(),
            reason: "permission denied".into(),
            suggestion: "Run with sudo or add user to input group".into(),
        };
        assert!(matches!(err, PlatformError::DeviceAccess { .. }));

        let err = PlatformError::InjectionFailed {
            reason: "uinput device not available".into(),
            suggestion: "Ensure uinput kernel module is loaded".into(),
        };
        assert!(matches!(err, PlatformError::InjectionFailed { .. }));

        let err = PlatformError::Unsupported {
            operation: "hotkey capture".into(),
        };
        assert!(matches!(err, PlatformError::Unsupported { .. }));

        let err = PlatformError::DeviceError("device not found".into());
        assert!(matches!(err, PlatformError::DeviceError(_)));
    }

    #[test]
    fn test_serialization_error_construction() {
        let err = SerializationError::InvalidMagic {
            expected: 0x4B525800,
            found: 0xFFFFFFFF,
        };
        assert!(matches!(err, SerializationError::InvalidMagic { .. }));

        let err = SerializationError::InvalidVersion {
            expected: 1,
            found: 2,
        };
        assert!(matches!(err, SerializationError::InvalidVersion { .. }));

        let err = SerializationError::InvalidSize {
            expected: 100,
            found: 50,
        };
        assert!(matches!(err, SerializationError::InvalidSize { .. }));

        let err = SerializationError::CorruptedData("bad data".into());
        assert!(matches!(err, SerializationError::CorruptedData(_)));
    }

    #[test]
    fn test_socket_error_construction() {
        let err = SocketError::NotConnected;
        assert!(matches!(err, SocketError::NotConnected));

        let err = SocketError::AlreadyConnected;
        assert!(matches!(err, SocketError::AlreadyConnected));
    }

    #[test]
    fn test_registry_error_construction() {
        let err = RegistryError::IOError(io::ErrorKind::NotFound);
        assert!(matches!(err, RegistryError::IOError(_)));

        let err = RegistryError::CorruptedRegistry("invalid json".into());
        assert!(matches!(err, RegistryError::CorruptedRegistry(_)));

        let err = RegistryError::FailedToLoad(io::ErrorKind::PermissionDenied);
        assert!(matches!(err, RegistryError::FailedToLoad(_)));
    }

    #[test]
    fn test_recorder_error_construction() {
        let err = RecorderError::NotRecording;
        assert!(matches!(err, RecorderError::NotRecording));

        let err = RecorderError::AlreadyRecording;
        assert!(matches!(err, RecorderError::AlreadyRecording));

        let err = RecorderError::PlaybackFailed(42);
        assert!(matches!(err, RecorderError::PlaybackFailed(42)));

        let err = RecorderError::BufferFull(10000);
        assert!(matches!(err, RecorderError::BufferFull(10000)));

        let err = RecorderError::MutexPoisoned("state".into());
        assert!(matches!(err, RecorderError::MutexPoisoned(_)));
    }

    // ============================================================================
    // Display Implementation Tests
    // ============================================================================

    #[test]
    fn test_platform_error_display() {
        let err = PlatformError::Poisoned("test mutex".into());
        let msg = err.to_string();
        assert!(msg.contains("Mutex poisoned"));
        assert!(msg.contains("test mutex"));

        let err = PlatformError::DeviceAccess {
            device: "/dev/input/event0".into(),
            reason: "permission denied".into(),
            suggestion: "Run with sudo".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Failed to access device"));
        assert!(msg.contains("/dev/input/event0"));
        assert!(msg.contains("permission denied"));
        assert!(msg.contains("Run with sudo"));

        let err = PlatformError::InjectionFailed {
            reason: "uinput unavailable".into(),
            suggestion: "Load uinput module".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Failed to inject"));
        assert!(msg.contains("uinput unavailable"));
        assert!(msg.contains("Load uinput module"));

        let err = PlatformError::Unsupported {
            operation: "hotkey capture".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("not supported"));
        assert!(msg.contains("hotkey capture"));
    }

    #[test]
    fn test_serialization_error_display() {
        let err = SerializationError::InvalidMagic {
            expected: 0x4B525800,
            found: 0xFFFFFFFF,
        };
        let msg = err.to_string();
        assert!(msg.contains("0x4b525800"));
        assert!(msg.contains("0xffffffff"));
    }

    #[test]
    fn test_serialization_version_display() {
        let err = SerializationError::InvalidVersion {
            expected: 1,
            found: 999,
        };
        let msg = err.to_string();
        assert!(msg.contains("expected 1"));
        assert!(msg.contains("found 999"));
    }

    #[test]
    fn test_serialization_size_display() {
        let err = SerializationError::InvalidSize {
            expected: 1024,
            found: 512,
        };
        let msg = err.to_string();
        assert!(msg.contains("expected 1024 bytes"));
        assert!(msg.contains("found 512 bytes"));
    }

    #[test]
    fn test_socket_error_display() {
        let err = SocketError::NotConnected;
        let msg = err.to_string();
        assert!(msg.contains("not connected"));
    }

    #[test]
    fn test_recorder_error_display() {
        let err = RecorderError::PlaybackFailed(42);
        let msg = err.to_string();
        assert!(msg.contains("frame 42"));

        let err = RecorderError::BufferFull(10000);
        let msg = err.to_string();
        assert!(msg.contains("10000"));
        assert!(msg.contains("buffer full"));

        let err = RecorderError::MutexPoisoned("state".into());
        let msg = err.to_string();
        assert!(msg.contains("state"));
        assert!(msg.contains("poisoned"));
    }

    // ============================================================================
    // From Conversion Tests
    // ============================================================================

    #[test]
    fn test_platform_error_to_daemon_error() {
        let platform_err = PlatformError::Poisoned("mutex".into());
        let daemon_err: DaemonError = platform_err.into();
        assert!(matches!(daemon_err, DaemonError::Platform(_)));
    }

    #[test]
    fn test_serialization_error_to_daemon_error() {
        let serialization_err = SerializationError::CorruptedData("bad".into());
        let daemon_err: DaemonError = serialization_err.into();
        assert!(matches!(daemon_err, DaemonError::Serialization(_)));
    }

    #[test]
    fn test_socket_error_to_daemon_error() {
        let socket_err = SocketError::NotConnected;
        let daemon_err: DaemonError = socket_err.into();
        assert!(matches!(daemon_err, DaemonError::Socket(_)));
    }

    #[test]
    fn test_registry_error_to_daemon_error() {
        let registry_err = RegistryError::CorruptedRegistry("invalid".into());
        let daemon_err: DaemonError = registry_err.into();
        assert!(matches!(daemon_err, DaemonError::Registry(_)));
    }

    #[test]
    fn test_recorder_error_to_daemon_error() {
        let recorder_err = RecorderError::NotRecording;
        let daemon_err: DaemonError = recorder_err.into();
        assert!(matches!(daemon_err, DaemonError::Recorder(_)));
    }

    // ============================================================================
    // Error Context Preservation Tests
    // ============================================================================

    #[test]
    fn test_error_context_preserved() {
        let platform_err = PlatformError::DeviceError("device123".into());
        let daemon_err: DaemonError = platform_err.into();
        let msg = daemon_err.to_string();
        assert!(msg.contains("device123"));
    }

    #[test]
    fn test_serialization_context_preserved() {
        let serialization_err = SerializationError::InvalidMagic {
            expected: 0x1234,
            found: 0x5678,
        };
        let daemon_err: DaemonError = serialization_err.into();
        let msg = daemon_err.to_string();
        assert!(msg.contains("0x00001234"));
        assert!(msg.contains("0x00005678"));
    }

    // ============================================================================
    // Error Trait Tests
    // ============================================================================

    #[test]
    fn test_platform_error_implements_error_trait() {
        let err = PlatformError::Poisoned("test".into());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_serialization_error_implements_error_trait() {
        let err = SerializationError::CorruptedData("test".into());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_socket_error_implements_error_trait() {
        let err = SocketError::NotConnected;
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_registry_error_implements_error_trait() {
        let err = RegistryError::CorruptedRegistry("test".into());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_recorder_error_implements_error_trait() {
        let err = RecorderError::NotRecording;
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_daemon_error_implements_error_trait() {
        let err = DaemonError::Platform(PlatformError::Poisoned("test".into()));
        let _: &dyn std::error::Error = &err;
    }

    // ============================================================================
    // IO Error Conversion Tests
    // ============================================================================

    #[test]
    fn test_io_error_to_platform_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let platform_err: PlatformError = io_err.into();
        assert!(matches!(platform_err, PlatformError::Io(_)));
    }

    #[test]
    fn test_io_error_to_serialization_error() {
        let io_err = io::Error::new(io::ErrorKind::UnexpectedEof, "unexpected eof");
        let serialization_err: SerializationError = io_err.into();
        assert!(matches!(serialization_err, SerializationError::Io(_)));
    }

    #[test]
    fn test_io_error_to_socket_error() {
        let io_err = io::Error::new(io::ErrorKind::ConnectionRefused, "refused");
        let socket_err: SocketError = io_err.into();
        assert!(matches!(socket_err, SocketError::Io(_)));
    }

    // ============================================================================
    // New Error Type Tests (ConfigError, WebError, CliError)
    // ============================================================================

    #[test]
    fn test_config_error_construction() {
        let err = ConfigError::FileNotFound {
            path: PathBuf::from("/test/config.toml"),
        };
        assert!(matches!(err, ConfigError::FileNotFound { .. }));

        let err = ConfigError::ParseError {
            path: PathBuf::from("/test/config.toml"),
            reason: "invalid syntax".into(),
        };
        assert!(matches!(err, ConfigError::ParseError { .. }));

        let err = ConfigError::InvalidProfile {
            name: "default".into(),
            reason: "missing layers".into(),
        };
        assert!(matches!(err, ConfigError::InvalidProfile { .. }));

        let err = ConfigError::CompilationFailed {
            reason: "syntax error".into(),
        };
        assert!(matches!(err, ConfigError::CompilationFailed { .. }));
    }

    #[test]
    fn test_web_error_construction() {
        let err = WebError::BindFailed {
            address: "127.0.0.1:3030".into(),
            reason: "address in use".into(),
        };
        assert!(matches!(err, WebError::BindFailed { .. }));

        let err = WebError::InvalidRequest {
            reason: "missing parameter".into(),
        };
        assert!(matches!(err, WebError::InvalidRequest { .. }));

        let err = WebError::WebSocketError {
            reason: "connection closed".into(),
        };
        assert!(matches!(err, WebError::WebSocketError { .. }));

        let err = WebError::StaticFileError {
            path: PathBuf::from("/static/index.html"),
            reason: "file not found".into(),
        };
        assert!(matches!(err, WebError::StaticFileError { .. }));
    }

    #[test]
    fn test_cli_error_construction() {
        let err = CliError::InvalidArguments {
            reason: "missing required argument".into(),
        };
        assert!(matches!(err, CliError::InvalidArguments { .. }));

        let err = CliError::CommandFailed {
            command: "activate".into(),
            reason: "profile not found".into(),
        };
        assert!(matches!(err, CliError::CommandFailed { .. }));

        let err = CliError::OutputError {
            reason: "failed to serialize JSON".into(),
        };
        assert!(matches!(err, CliError::OutputError { .. }));
    }

    #[test]
    fn test_config_error_to_daemon_error() {
        let config_err = ConfigError::FileNotFound {
            path: PathBuf::from("/test"),
        };
        let daemon_err: DaemonError = config_err.into();
        assert!(matches!(daemon_err, DaemonError::Config(_)));
    }

    #[test]
    fn test_web_error_to_daemon_error() {
        let web_err = WebError::InvalidRequest {
            reason: "test".into(),
        };
        let daemon_err: DaemonError = web_err.into();
        assert!(matches!(daemon_err, DaemonError::Web(_)));
    }

    #[test]
    fn test_cli_error_to_daemon_error() {
        let cli_err = CliError::InvalidArguments {
            reason: "test".into(),
        };
        let daemon_err: DaemonError = cli_err.into();
        assert!(matches!(daemon_err, DaemonError::Cli(_)));
    }

    #[test]
    fn test_core_error_to_daemon_error() {
        use keyrx_core::error::CoreError;
        let core_err = CoreError::InvalidState {
            message: "test".into(),
        };
        let daemon_err: DaemonError = core_err.into();
        assert!(matches!(daemon_err, DaemonError::Core(_)));
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::FileNotFound {
            path: PathBuf::from("/test/config.toml"),
        };
        let msg = err.to_string();
        assert!(msg.contains("Configuration file not found"));
        assert!(msg.contains("/test/config.toml"));

        let err = ConfigError::InvalidProfile {
            name: "default".into(),
            reason: "missing layers".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid profile"));
        assert!(msg.contains("default"));
        assert!(msg.contains("missing layers"));
    }

    #[test]
    fn test_web_error_display() {
        let err = WebError::BindFailed {
            address: "127.0.0.1:3030".into(),
            reason: "address in use".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Failed to bind"));
        assert!(msg.contains("127.0.0.1:3030"));
        assert!(msg.contains("address in use"));
    }

    #[test]
    fn test_cli_error_display() {
        let err = CliError::CommandFailed {
            command: "activate".into(),
            reason: "profile not found".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Command 'activate' failed"));
        assert!(msg.contains("profile not found"));
    }

    #[test]
    fn test_daemon_result_type_alias() {
        fn returns_ok() -> DaemonResult<i32> {
            Ok(42)
        }
        assert_eq!(returns_ok().unwrap(), 42);

        fn returns_err() -> DaemonResult<i32> {
            Err(CliError::InvalidArguments {
                reason: "test".into(),
            }
            .into())
        }
        assert!(returns_err().is_err());
    }

    #[test]
    fn test_core_error_through_config_error() {
        use keyrx_core::error::CoreError;
        let core_err = CoreError::Validation {
            field: "key_code".into(),
            reason: "invalid".into(),
        };
        let config_err: ConfigError = core_err.into();
        assert!(matches!(config_err, ConfigError::Core(_)));

        let daemon_err: DaemonError = config_err.into();
        assert!(matches!(daemon_err, DaemonError::Config(_)));
    }
}
