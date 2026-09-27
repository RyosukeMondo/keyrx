//! Tests for the newer error types (ConfigError, WebError, CliError) and
//! their composition into DaemonError.

use super::*;
use std::path::PathBuf;

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
