//! Comprehensive tests for the error code system.
//!
//! This test suite verifies:
//! - Error code formatting (KRX-CXXX)
//! - JSON serialization/deserialization
//! - No duplicate error codes across all categories
//! - Error creation macros (keyrx_err!, bail_keyrx!)
//! - Error context chaining
//! - Registry functionality

use keyrx_core::errors::*;
use std::collections::HashSet;

// ============================================================================
// Error Code Formatting Tests
// ============================================================================

#[test]
fn test_error_code_format_all_categories() {
    let codes = vec![
        (ErrorCode::new(ErrorCategory::Config, 1001), "KRX-C1001"),
        (ErrorCode::new(ErrorCategory::Runtime, 2001), "KRX-R2001"),
        (ErrorCode::new(ErrorCategory::Driver, 3001), "KRX-D3001"),
        (ErrorCode::new(ErrorCategory::Validation, 4001), "KRX-V4001"),
        (ErrorCode::new(ErrorCategory::Ffi, 5001), "KRX-F5001"),
        (ErrorCode::new(ErrorCategory::Internal, 9001), "KRX-I9001"),
    ];

    for (code, expected) in codes {
        assert_eq!(code.to_string(), expected);
        assert_eq!(code.as_string(), expected);
    }
}

#[test]
fn test_error_code_padding() {
    // Test proper zero-padding of error codes
    let test_cases = vec![
        (ErrorCode::new(ErrorCategory::Config, 1001), "KRX-C1001"),
        (ErrorCode::new(ErrorCategory::Config, 1099), "KRX-C1099"),
        (ErrorCode::new(ErrorCategory::Config, 1999), "KRX-C1999"),
    ];

    for (code, expected) in test_cases {
        assert_eq!(code.to_string(), expected);
    }
}

#[test]
fn test_error_code_accessors() {
    let code = ErrorCode::new(ErrorCategory::Runtime, 2042);

    assert_eq!(code.category(), ErrorCategory::Runtime);
    assert_eq!(code.number(), 2042);
    assert_eq!(code.category().prefix(), 'R');
}

#[test]
fn test_error_category_ranges() {
    // Config: 1000-1999
    assert!(ErrorCategory::Config.contains(1000));
    assert!(ErrorCategory::Config.contains(1500));
    assert!(ErrorCategory::Config.contains(1999));
    assert!(!ErrorCategory::Config.contains(2000));
    assert!(!ErrorCategory::Config.contains(999));

    // Runtime: 2000-2999
    assert!(ErrorCategory::Runtime.contains(2000));
    assert!(ErrorCategory::Runtime.contains(2500));
    assert!(ErrorCategory::Runtime.contains(2999));
    assert!(!ErrorCategory::Runtime.contains(3000));

    // Driver: 3000-3999
    assert!(ErrorCategory::Driver.contains(3000));
    assert!(ErrorCategory::Driver.contains(3999));
    assert!(!ErrorCategory::Driver.contains(4000));

    // Validation: 4000-4999
    assert!(ErrorCategory::Validation.contains(4000));
    assert!(ErrorCategory::Validation.contains(4999));
    assert!(!ErrorCategory::Validation.contains(5000));

    // FFI: 5000-5999
    assert!(ErrorCategory::Ffi.contains(5000));
    assert!(ErrorCategory::Ffi.contains(5999));
    assert!(!ErrorCategory::Ffi.contains(6000));

    // Internal: 9000-9999
    assert!(ErrorCategory::Internal.contains(9000));
    assert!(ErrorCategory::Internal.contains(9999));
    assert!(!ErrorCategory::Internal.contains(10000));
}

#[test]
fn test_error_category_base_numbers() {
    assert_eq!(ErrorCategory::Config.base_number(), 1000);
    assert_eq!(ErrorCategory::Runtime.base_number(), 2000);
    assert_eq!(ErrorCategory::Driver.base_number(), 3000);
    assert_eq!(ErrorCategory::Validation.base_number(), 4000);
    assert_eq!(ErrorCategory::Ffi.base_number(), 5000);
    assert_eq!(ErrorCategory::Internal.base_number(), 9000);
}

// ============================================================================
// Error Definition and Template Tests
// ============================================================================

#[test]
fn test_error_def_message_formatting() {
    use keyrx_core::errors::config::CONFIG_NOT_FOUND;

    let msg = CONFIG_NOT_FOUND.format(&[("path", "/etc/keyrx.toml")]);
    assert_eq!(msg, "Configuration file not found: /etc/keyrx.toml");
}

#[test]
fn test_error_def_multiple_placeholders() {
    use keyrx_core::errors::validation::DUPLICATE_REMAP;

    let msg = DUPLICATE_REMAP.format(&[("key", "Ctrl+C")]);

    assert!(msg.contains("Ctrl+C"));
}

#[test]
fn test_error_def_missing_placeholder() {
    use keyrx_core::errors::config::CONFIG_NOT_FOUND;

    // If placeholder is not provided, it should remain in the message
    let msg = CONFIG_NOT_FOUND.format(&[]);
    assert!(msg.contains("{path}"));
}

#[test]
fn test_error_severity_levels() {
    use keyrx_core::errors::config::CONFIG_NOT_FOUND;
    use keyrx_core::errors::runtime::STATE_CORRUPTION_DETECTED;

    // CONFIG_NOT_FOUND should be Error severity
    assert_eq!(CONFIG_NOT_FOUND.severity(), ErrorSeverity::Error);

    // STATE_CORRUPTION_DETECTED should be Fatal severity
    assert_eq!(STATE_CORRUPTION_DETECTED.severity(), ErrorSeverity::Fatal);
}

// ============================================================================
// KeyrxError Tests
// ============================================================================

#[test]
fn test_keyrx_error_creation() {
    use keyrx_core::errors::config::CONFIG_NOT_FOUND;

    let err = KeyrxError::new(
        &CONFIG_NOT_FOUND,
        vec![("path".to_string(), "/etc/keyrx.toml".to_string())],
        None,
    );

    assert_eq!(err.code(), "KRX-C1001");
    assert_eq!(
        err.message(),
        "Configuration file not found: /etc/keyrx.toml"
    );
    assert!(err.hint().is_some());
}

#[test]
fn test_keyrx_error_simple() {
    use keyrx_core::errors::config::CONFIG_NOT_FOUND;

    let err = KeyrxError::simple(&CONFIG_NOT_FOUND);

    assert_eq!(err.code(), "KRX-C1001");
    // Message will contain placeholder since no context provided
    assert!(err.message().contains("{path}"));
}

#[test]
fn test_keyrx_error_with_context() {
    use keyrx_core::errors::config::CONFIG_NOT_FOUND;

    let err = KeyrxError::simple(&CONFIG_NOT_FOUND)
        .with_context("user", "test_user")
        .with_context("timestamp", "2024-01-01T00:00:00Z");

    let context = err.context();
    assert_eq!(context.get("user").map(|s| s.as_str()), Some("test_user"));
    assert_eq!(
        context.get("timestamp").map(|s| s.as_str()),
        Some("2024-01-01T00:00:00Z")
    );
}

#[test]
fn test_keyrx_error_display() {
    use keyrx_core::errors::config::CONFIG_NOT_FOUND;

    let err = KeyrxError::new(
        &CONFIG_NOT_FOUND,
        vec![("path".to_string(), "config.toml".to_string())],
        None,
    );

    let display = format!("{}", err);

    // Should include error code
    assert!(display.contains("KRX-C1001"));
    // Should include formatted message
    assert!(display.contains("Configuration file not found: config.toml"));
    // Should include hint
    assert!(display.contains("Hint:"));
}

#[test]
fn test_keyrx_error_std_error_trait() {
    use keyrx_core::errors::config::CONFIG_NOT_FOUND;

    let err = KeyrxError::simple(&CONFIG_NOT_FOUND);

    // Should implement std::error::Error
    fn assert_is_std_error<T: std::error::Error>(_: &T) {}
    assert_is_std_error(&err);
}

// ============================================================================
// JSON Serialization Tests
// ============================================================================

#[test]
fn test_error_json_serialization() {
    use keyrx_core::errors::config::CONFIG_NOT_FOUND;

    let err = KeyrxError::new(
        &CONFIG_NOT_FOUND,
        vec![("path".to_string(), "/etc/keyrx.toml".to_string())],
        None,
    );

    let json = serde_json::to_string(&err).expect("Failed to serialize");

    assert!(json.contains("KRX-C1001"));
    assert!(json.contains("Configuration file not found"));
    assert!(json.contains("/etc/keyrx.toml"));
}

#[test]
fn test_error_json_deserialization() {
    use keyrx_core::errors::config::CONFIG_NOT_FOUND;

    let err = KeyrxError::new(
        &CONFIG_NOT_FOUND,
        vec![("path".to_string(), "/etc/keyrx.toml".to_string())],
        None,
    );

    let json = serde_json::to_string(&err).expect("Failed to serialize");
    let deserialized: KeyrxError = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.code(), err.code());
    assert_eq!(deserialized.message(), err.message());
    assert_eq!(deserialized.severity(), err.severity());
}

#[test]
fn test_error_json_roundtrip() {
    use keyrx_core::errors::runtime::ENGINE_START_FAILED;

    let err = KeyrxError::new(
        &ENGINE_START_FAILED,
        vec![(
            "reason".to_string(),
            "Failed to initialize driver".to_string(),
        )],
        None,
    );

    let json = serde_json::to_string(&err).expect("Serialization failed");
    let restored: KeyrxError = serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(restored.code(), err.code());
    assert_eq!(restored.message(), err.message());
}

// ============================================================================
// Error Macro Tests
// ============================================================================

#[test]
fn test_keyrx_err_macro() {
    use keyrx_core::errors::config::CONFIG_NOT_FOUND;
    use keyrx_core::keyrx_err;

    let err = keyrx_err!(CONFIG_NOT_FOUND, path = "/test/path".to_string());

    assert_eq!(err.code(), "KRX-C1001");
    assert!(err.message().contains("/test/path"));
}

#[test]
fn test_keyrx_err_macro_multiple_args() {
    use keyrx_core::errors::config::CONFIG_INVALID_TYPE;
    use keyrx_core::keyrx_err;

    let err = keyrx_err!(
        CONFIG_INVALID_TYPE,
        field = "timeout".to_string(),
        expected = "integer".to_string(),
        actual = "string".to_string()
    );

    assert_eq!(err.code(), "KRX-C1007");
    assert!(err.message().contains("timeout"));
    assert!(err.message().contains("integer"));
    assert!(err.message().contains("string"));
}

#[test]
fn test_bail_keyrx_macro() {
    use keyrx_core::bail_keyrx;
    use keyrx_core::errors::config::CONFIG_NOT_FOUND;

    fn failing_function() -> Result<(), KeyrxError> {
        bail_keyrx!(CONFIG_NOT_FOUND, path = "/nonexistent".to_string());
    }

    let result = failing_function();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.code(), "KRX-C1001");
    assert!(err.message().contains("/nonexistent"));
}

// ============================================================================
// Duplicate Code Detection Tests
// ============================================================================

#[test]
fn test_no_duplicate_error_codes_config() {
    use keyrx_core::errors::config::*;

    let codes = vec![
        CONFIG_NOT_FOUND.code(),
        CONFIG_READ_ERROR.code(),
        CONFIG_PARSE_ERROR.code(),
        CONFIG_INVALID_VALUE.code(),
        CONFIG_MISSING_FIELD.code(),
        CONFIG_INVALID_PATH.code(),
        CONFIG_INVALID_TYPE.code(),
    ];

    let mut seen = HashSet::new();
    for code in codes {
        assert!(
            seen.insert(code.to_string()),
            "Duplicate error code found: {}",
            code
        );
    }
}

#[test]
fn test_no_duplicate_error_codes_runtime() {
    use keyrx_core::errors::runtime::*;

    let codes = vec![
        ENGINE_NOT_INITIALIZED.code(),
        ENGINE_ALREADY_RUNNING.code(),
        ENGINE_START_FAILED.code(),
        EVENT_PROCESSING_FAILED.code(),
        INVALID_EVENT_DATA.code(),
        OUTPUT_INJECTION_FAILED.code(),
        STATE_CORRUPTION_DETECTED.code(),
    ];

    let mut seen = HashSet::new();
    for code in codes {
        assert!(
            seen.insert(code.to_string()),
            "Duplicate error code found: {}",
            code
        );
    }
}

#[test]
fn test_no_duplicate_error_codes_driver() {
    use keyrx_core::errors::driver::*;

    let codes = vec![
        DRIVER_NOT_FOUND.code(),
        DRIVER_LOAD_FAILED.code(),
        DRIVER_INIT_FAILED.code(),
        DRIVER_PERMISSION_DENIED.code(),
        DRIVER_DEVICE_NOT_FOUND.code(),
        DRIVER_DEVICE_DISCONNECTED.code(),
        DRIVER_NOT_SUPPORTED.code(),
        EVDEV_DEVICE_GRAB_FAILED.code(),
        EVDEV_UINPUT_CREATE_FAILED.code(),
    ];

    let mut seen = HashSet::new();
    for code in codes {
        assert!(
            seen.insert(code.to_string()),
            "Duplicate error code found: {}",
            code
        );
    }
}

#[test]
fn test_no_duplicate_error_codes_validation() {
    use keyrx_core::errors::validation::*;

    let codes = vec![
        UNKNOWN_KEY.code(),
        UNDEFINED_LAYER.code(),
        DUPLICATE_REMAP.code(),
        REMAP_BLOCK_CONFLICT.code(),
        CIRCULAR_REMAP.code(),
        COMBO_SHADOWING.code(),
        SCRIPT_SYNTAX_ERROR.code(),
    ];

    let mut seen = HashSet::new();
    for code in codes {
        assert!(
            seen.insert(code.to_string()),
            "Duplicate error code found: {}",
            code
        );
    }
}

#[test]
fn test_no_duplicate_codes_across_all_categories() {
    use keyrx_core::errors::{config::*, driver::*, runtime::*, validation::*};

    let all_codes = vec![
        // Config
        CONFIG_NOT_FOUND.code(),
        CONFIG_READ_ERROR.code(),
        CONFIG_PARSE_ERROR.code(),
        CONFIG_INVALID_VALUE.code(),
        CONFIG_MISSING_FIELD.code(),
        CONFIG_INVALID_PATH.code(),
        CONFIG_INVALID_TYPE.code(),
        // Runtime
        ENGINE_NOT_INITIALIZED.code(),
        ENGINE_ALREADY_RUNNING.code(),
        ENGINE_START_FAILED.code(),
        EVENT_PROCESSING_FAILED.code(),
        INVALID_EVENT_DATA.code(),
        OUTPUT_INJECTION_FAILED.code(),
        // Driver
        DRIVER_NOT_FOUND.code(),
        DRIVER_LOAD_FAILED.code(),
        DRIVER_INIT_FAILED.code(),
        DRIVER_PERMISSION_DENIED.code(),
        DRIVER_DEVICE_NOT_FOUND.code(),
        DRIVER_DEVICE_DISCONNECTED.code(),
        DRIVER_NOT_SUPPORTED.code(),
        EVDEV_DEVICE_GRAB_FAILED.code(),
        EVDEV_UINPUT_CREATE_FAILED.code(),
        // Validation
        UNKNOWN_KEY.code(),
        UNDEFINED_LAYER.code(),
        DUPLICATE_REMAP.code(),
        REMAP_BLOCK_CONFLICT.code(),
        CIRCULAR_REMAP.code(),
        COMBO_SHADOWING.code(),
        SCRIPT_SYNTAX_ERROR.code(),
    ];

    let mut seen = HashSet::new();
    for code in all_codes {
        let code_str = code.to_string();
        assert!(
            seen.insert(code_str.clone()),
            "Duplicate error code found across categories: {}",
            code_str
        );
    }
}

// ============================================================================
// Error Code Range Tests
// ============================================================================

#[test]
fn test_config_errors_in_valid_range() {
    use keyrx_core::errors::config::*;

    let codes = vec![
        CONFIG_NOT_FOUND.code(),
        CONFIG_READ_ERROR.code(),
        CONFIG_PARSE_ERROR.code(),
        CONFIG_INVALID_VALUE.code(),
        CONFIG_MISSING_FIELD.code(),
        CONFIG_INVALID_PATH.code(),
        CONFIG_INVALID_TYPE.code(),
    ];

    for code in codes {
        assert_eq!(code.category(), ErrorCategory::Config);
        assert!(
            code.number() >= 1000 && code.number() < 2000,
            "Config error code {} outside valid range",
            code
        );
    }
}

#[test]
fn test_runtime_errors_in_valid_range() {
    use keyrx_core::errors::runtime::*;

    let codes = vec![
        ENGINE_NOT_INITIALIZED.code(),
        ENGINE_ALREADY_RUNNING.code(),
        ENGINE_START_FAILED.code(),
        EVENT_PROCESSING_FAILED.code(),
        INVALID_EVENT_DATA.code(),
        OUTPUT_INJECTION_FAILED.code(),
    ];

    for code in codes {
        assert_eq!(code.category(), ErrorCategory::Runtime);
        assert!(
            code.number() >= 2000 && code.number() < 3000,
            "Runtime error code {} outside valid range",
            code
        );
    }
}

#[test]
fn test_driver_errors_in_valid_range() {
    use keyrx_core::errors::driver::*;

    let codes = vec![
        DRIVER_NOT_FOUND.code(),
        DRIVER_LOAD_FAILED.code(),
        DRIVER_INIT_FAILED.code(),
        DRIVER_PERMISSION_DENIED.code(),
        DRIVER_DEVICE_NOT_FOUND.code(),
        DRIVER_DEVICE_DISCONNECTED.code(),
        DRIVER_NOT_SUPPORTED.code(),
        EVDEV_DEVICE_GRAB_FAILED.code(),
        EVDEV_UINPUT_CREATE_FAILED.code(),
    ];

    for code in codes {
        assert_eq!(code.category(), ErrorCategory::Driver);
        assert!(
            code.number() >= 3000 && code.number() < 4000,
            "Driver error code {} outside valid range",
            code
        );
    }
}

#[test]
fn test_validation_errors_in_valid_range() {
    use keyrx_core::errors::validation::*;

    let codes = vec![
        UNKNOWN_KEY.code(),
        UNDEFINED_LAYER.code(),
        DUPLICATE_REMAP.code(),
        REMAP_BLOCK_CONFLICT.code(),
        CIRCULAR_REMAP.code(),
        COMBO_SHADOWING.code(),
        SCRIPT_SYNTAX_ERROR.code(),
    ];

    for code in codes {
        assert_eq!(code.category(), ErrorCategory::Validation);
        assert!(
            code.number() >= 4000 && code.number() < 5000,
            "Validation error code {} outside valid range",
            code
        );
    }
}

// ============================================================================
// Error Context and Chaining Tests
// ============================================================================

#[test]
fn test_error_context_map() {
    use keyrx_core::errors::config::CONFIG_NOT_FOUND;

    let err = KeyrxError::new(
        &CONFIG_NOT_FOUND,
        vec![
            ("path".to_string(), "/etc/keyrx.toml".to_string()),
            ("attempted_at".to_string(), "startup".to_string()),
        ],
        None,
    );

    let context = err.context();
    assert_eq!(context.len(), 2);
    assert_eq!(
        context.get("path").map(|s| s.as_str()),
        Some("/etc/keyrx.toml")
    );
    assert_eq!(
        context.get("attempted_at").map(|s| s.as_str()),
        Some("startup")
    );
}

#[test]
fn test_error_with_source_string() {
    use keyrx_core::errors::config::CONFIG_READ_ERROR;

    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let err = KeyrxError::new(
        &CONFIG_READ_ERROR,
        vec![("path".to_string(), "/etc/keyrx.toml".to_string())],
        Some(Box::new(io_error)),
    );

    assert!(err.source_str().is_some());
    let source_str = err.source_str().unwrap();
    assert!(source_str.contains("access denied"));
}

#[test]
fn test_error_definition_accessor() {
    use keyrx_core::errors::config::CONFIG_NOT_FOUND;

    let err = KeyrxError::simple(&CONFIG_NOT_FOUND);

    let def = err.definition();
    assert!(def.is_some());
    assert_eq!(def.unwrap().code(), CONFIG_NOT_FOUND.code());
}

// ============================================================================
// Error Code Equality Tests
// ============================================================================

#[test]
fn test_error_code_equality() {
    let code1 = ErrorCode::new(ErrorCategory::Config, 1001);
    let code2 = ErrorCode::new(ErrorCategory::Config, 1001);
    let code3 = ErrorCode::new(ErrorCategory::Config, 1002);
    let code4 = ErrorCode::new(ErrorCategory::Runtime, 2001);

    assert_eq!(code1, code2);
    assert_ne!(code1, code3);
    assert_ne!(code1, code4);
}

#[test]
fn test_error_code_hash() {
    use std::collections::HashMap;

    let code1 = ErrorCode::new(ErrorCategory::Config, 1001);
    let code2 = ErrorCode::new(ErrorCategory::Config, 1001);

    let mut map = HashMap::new();
    map.insert(code1, "error1");

    // Same code should retrieve the same value
    assert_eq!(map.get(&code2), Some(&"error1"));
}

// ============================================================================
// Error Conversion Tests
// ============================================================================

#[test]
fn test_io_error_conversion() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let keyrx_error = KeyrxError::from(io_error);

    assert_eq!(keyrx_error.code(), "KRX-I9001");
    assert!(keyrx_error.message().contains("I/O error"));
}

// ============================================================================
// All Errors Have Required Fields Tests
// ============================================================================

#[test]
fn test_all_config_errors_have_hints() {
    use keyrx_core::errors::config::*;

    let errors = vec![
        &CONFIG_NOT_FOUND,
        &CONFIG_READ_ERROR,
        &CONFIG_PARSE_ERROR,
        &CONFIG_INVALID_VALUE,
        &CONFIG_MISSING_FIELD,
        &CONFIG_INVALID_PATH,
        &CONFIG_INVALID_TYPE,
    ];

    for error in errors {
        assert!(
            error.hint().is_some(),
            "Error {} missing hint",
            error.code()
        );
    }
}

#[test]
fn test_all_runtime_errors_have_hints() {
    use keyrx_core::errors::runtime::*;

    let errors = vec![
        &ENGINE_NOT_INITIALIZED,
        &ENGINE_ALREADY_RUNNING,
        &ENGINE_START_FAILED,
        &EVENT_PROCESSING_FAILED,
        &INVALID_EVENT_DATA,
        &OUTPUT_INJECTION_FAILED,
    ];

    for error in errors {
        assert!(
            error.hint().is_some(),
            "Error {} missing hint",
            error.code()
        );
    }
}

#[test]
fn test_all_driver_errors_have_hints() {
    use keyrx_core::errors::driver::*;

    let errors = vec![
        &DRIVER_NOT_FOUND,
        &DRIVER_LOAD_FAILED,
        &DRIVER_INIT_FAILED,
        &DRIVER_PERMISSION_DENIED,
        &DRIVER_DEVICE_NOT_FOUND,
        &DRIVER_DEVICE_DISCONNECTED,
        &DRIVER_NOT_SUPPORTED,
        &EVDEV_DEVICE_GRAB_FAILED,
        &EVDEV_UINPUT_CREATE_FAILED,
    ];

    for error in errors {
        assert!(
            error.hint().is_some(),
            "Error {} missing hint",
            error.code()
        );
    }
}

#[test]
fn test_all_validation_errors_have_hints() {
    use keyrx_core::errors::validation::*;

    let errors = vec![
        &UNKNOWN_KEY,
        &UNDEFINED_LAYER,
        &DUPLICATE_REMAP,
        &REMAP_BLOCK_CONFLICT,
        &CIRCULAR_REMAP,
        &COMBO_SHADOWING,
        &SCRIPT_SYNTAX_ERROR,
    ];

    for error in errors {
        assert!(
            error.hint().is_some(),
            "Error {} missing hint",
            error.code()
        );
    }
}
