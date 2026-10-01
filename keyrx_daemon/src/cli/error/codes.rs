//! Numeric error code assignment for JSON CLI output.

use crate::error::{
    CliError, ConfigError, DaemonError, PlatformError, RecorderError, RegistryError,
    SerializationError, SocketError, WebError,
};

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
pub(super) fn error_code(error: &DaemonError) -> u32 {
    match error {
        // CLI errors: 1000-1999
        DaemonError::Cli(CliError::InvalidArguments { .. }) => 1000,
        DaemonError::Cli(CliError::CommandFailed { .. }) => 1001,
        DaemonError::Cli(CliError::OutputError { .. }) => 1002,
        DaemonError::Cli(CliError::Reported) => 1003,

        // Configuration errors: 2000-2999
        DaemonError::Config(ConfigError::FileNotFound { .. }) => 2000,
        DaemonError::Config(ConfigError::ParseError { .. }) => 2001,
        DaemonError::Config(ConfigError::InvalidProfile { .. }) => 2002,
        DaemonError::Config(ConfigError::CompilationFailed { .. }) => 2003,
        DaemonError::Config(ConfigError::Core(_)) => 2004,
        DaemonError::Config(ConfigError::Io(_)) => 2005,
        DaemonError::Config(ConfigError::Profile(_)) => 2006,
        DaemonError::Config(ConfigError::Generator(_)) => 2007,

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
        DaemonError::Init(_) => 9001,
    }
}
