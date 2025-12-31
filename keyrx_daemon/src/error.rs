//! Error types for the KeyRx daemon.
//!
//! This module defines a comprehensive error type hierarchy for the daemon,
//! enabling proper error propagation and recovery instead of panics.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

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
    #[error("Platform initialization failed: {0}")]
    InitializationFailed(String),

    /// Device operation failed.
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
}

/// Top-level daemon error type.
///
/// This is the main error type for the daemon, encompassing all possible
/// error conditions. Module-specific errors automatically convert into
/// this type via `From` implementations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DaemonError {
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
}

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

        let err = PlatformError::InitializationFailed("failed to init".into());
        assert!(matches!(err, PlatformError::InitializationFailed(_)));

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
}
