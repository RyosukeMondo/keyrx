use super::codes::error_code;
use super::details::{extract_config_error_details, extract_platform_error_details};
use super::*;
use crate::error::{CliError, ConfigError, PlatformError};
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
