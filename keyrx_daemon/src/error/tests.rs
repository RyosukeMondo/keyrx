use super::*;
use std::io;

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
