#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::field_reassign_with_default,
    clippy::useless_conversion,
    clippy::assertions_on_constants,
    clippy::manual_div_ceil,
    clippy::manual_strip,
    clippy::len_zero,
    clippy::redundant_closure,
    clippy::manual_range_contains,
    clippy::default_constructed_unit_structs,
    clippy::clone_on_copy,
    clippy::io_other_error,
    clippy::bool_assert_comparison,
    clippy::approx_constant,
    clippy::let_unit_value,
    clippy::while_let_on_iterator,
    clippy::await_holding_lock,
    clippy::unnecessary_cast,
    clippy::drop_non_drop,
    clippy::needless_range_loop,
    unused_imports,
    unused_variables,
    dead_code,
    unsafe_code,
    clippy::collapsible_if,
    clippy::bool_comparison,
    unexpected_cfgs
)]
//! Comprehensive tests for LogEntry and FFI conversion.

use keyrx_core::observability::entry::{LogEntry, LogLevel};
use std::ffi::{CStr, CString};
use std::time::SystemTime;

#[test]
fn test_log_level_conversions() {
    assert_eq!(LogLevel::from(&tracing::Level::TRACE), LogLevel::Trace);
    assert_eq!(LogLevel::from(&tracing::Level::DEBUG), LogLevel::Debug);
    assert_eq!(LogLevel::from(&tracing::Level::INFO), LogLevel::Info);
    assert_eq!(LogLevel::from(&tracing::Level::WARN), LogLevel::Warn);
    assert_eq!(LogLevel::from(&tracing::Level::ERROR), LogLevel::Error);
}

#[test]
fn test_log_level_display_formatting() {
    assert_eq!(format!("{}", LogLevel::Trace), "TRACE");
    assert_eq!(format!("{}", LogLevel::Debug), "DEBUG");
    assert_eq!(format!("{}", LogLevel::Info), "INFO");
    assert_eq!(format!("{}", LogLevel::Warn), "WARN");
    assert_eq!(format!("{}", LogLevel::Error), "ERROR");
}

#[test]
fn test_log_level_equality() {
    assert_eq!(LogLevel::Info, LogLevel::Info);
    assert_ne!(LogLevel::Info, LogLevel::Debug);
}

#[test]
fn test_log_level_clone() {
    let level = LogLevel::Debug;
    let cloned = level;
    assert_eq!(level, cloned);
}

#[test]
fn test_log_entry_basic_creation() {
    let entry = LogEntry::new(LogLevel::Info, "test::module", "Test message");

    assert_eq!(entry.level, LogLevel::Info);
    assert_eq!(entry.target, "test::module");
    assert_eq!(entry.message, "Test message");
    assert!(entry.fields.is_empty());
    assert!(entry.span.is_none());
    assert!(entry.timestamp > 0);
}

#[test]
fn test_log_entry_timestamp_is_recent() {
    let before = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let entry = LogEntry::new(LogLevel::Info, "test", "Message");

    let after = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    assert!(entry.timestamp >= before);
    assert!(entry.timestamp <= after);
}

#[test]
fn test_log_entry_with_single_field() {
    let entry = LogEntry::new(LogLevel::Debug, "test", "Message").with_field("key", "value");

    assert_eq!(entry.fields.len(), 1);
    assert_eq!(entry.fields.get("key"), Some(&"value".to_string()));
}

#[test]
fn test_log_entry_with_multiple_fields() {
    let entry = LogEntry::new(LogLevel::Warn, "test", "Message")
        .with_field("user_id", "123")
        .with_field("action", "login")
        .with_field("ip", "192.168.1.1");

    assert_eq!(entry.fields.len(), 3);
    assert_eq!(entry.fields.get("user_id"), Some(&"123".to_string()));
    assert_eq!(entry.fields.get("action"), Some(&"login".to_string()));
    assert_eq!(entry.fields.get("ip"), Some(&"192.168.1.1".to_string()));
}

#[test]
fn test_log_entry_with_span() {
    let entry = LogEntry::new(LogLevel::Error, "test", "Message").with_span("request_handler");

    assert_eq!(entry.span, Some("request_handler".to_string()));
}

#[test]
fn test_log_entry_builder_chaining() {
    let entry = LogEntry::new(LogLevel::Info, "test", "Message")
        .with_field("key1", "value1")
        .with_span("my_span")
        .with_field("key2", "value2");

    assert_eq!(entry.fields.len(), 2);
    assert_eq!(entry.span, Some("my_span".to_string()));
}

#[test]
fn test_log_entry_json_serialization() {
    let entry =
        LogEntry::new(LogLevel::Info, "test::module", "Test message").with_field("user_id", "42");

    let json = entry.to_json().expect("Serialization failed");

    assert!(json.contains("\"level\":\"Info\""));
    assert!(json.contains("\"target\":\"test::module\""));
    assert!(json.contains("\"message\":\"Test message\""));
    assert!(json.contains("\"user_id\":\"42\""));
    assert!(json.contains("\"timestamp\""));
}

#[test]
fn test_log_entry_json_without_optional_fields() {
    let entry = LogEntry::new(LogLevel::Debug, "test", "Simple message");

    let json = entry.to_json().expect("Serialization failed");

    // Empty fields should be omitted
    assert!(!json.contains("\"fields\""));
    // Null span should be omitted
    assert!(!json.contains("\"span\""));
}

#[test]
fn test_log_entry_json_with_all_fields() {
    let entry = LogEntry::new(LogLevel::Warn, "test", "Full message")
        .with_field("field1", "value1")
        .with_span("test_span");

    let json = entry.to_json().expect("Serialization failed");

    assert!(json.contains("\"fields\""));
    assert!(json.contains("\"span\":\"test_span\""));
}

#[test]
#[allow(unsafe_code)]
fn test_log_entry_to_ffi_conversion() {
    let entry = LogEntry::new(LogLevel::Error, "test::ffi", "FFI test message");

    let c_entry = entry.to_ffi().expect("FFI conversion failed");

    assert_eq!(c_entry.level, LogLevel::Error);
    assert_eq!(c_entry.timestamp, entry.timestamp);

    unsafe {
        assert!(!c_entry.target.is_null());
        assert!(!c_entry.message.is_null());
        assert!(!c_entry.json.is_null());

        let target_str = CStr::from_ptr(c_entry.target).to_str().unwrap();
        assert_eq!(target_str, "test::ffi");

        let message_str = CStr::from_ptr(c_entry.message).to_str().unwrap();
        assert_eq!(message_str, "FFI test message");

        let json_str = CStr::from_ptr(c_entry.json).to_str().unwrap();
        assert!(json_str.contains("\"level\":\"Error\""));

        // Clean up
        let _ = CString::from_raw(c_entry.target);
        let _ = CString::from_raw(c_entry.message);
        let _ = CString::from_raw(c_entry.json);
    }
}

#[test]
#[allow(unsafe_code)]
fn test_c_log_entry_create_valid_input() {
    use keyrx_core::observability::entry::{c_log_entry_create, c_log_entry_free};

    unsafe {
        let target = CString::new("test::target").unwrap();
        let message = CString::new("Test message").unwrap();

        let entry = c_log_entry_create(
            1234567890,
            LogLevel::Info,
            target.as_ptr(),
            message.as_ptr(),
        );

        assert!(!entry.is_null());

        let entry_ref = &*entry;
        assert_eq!(entry_ref.timestamp, 1234567890);
        assert_eq!(entry_ref.level, LogLevel::Info);

        let target_str = CStr::from_ptr(entry_ref.target).to_str().unwrap();
        assert_eq!(target_str, "test::target");

        c_log_entry_free(entry);
    }
}

#[test]
#[allow(unsafe_code)]
fn test_c_log_entry_create_null_target() {
    use keyrx_core::observability::entry::c_log_entry_create;

    unsafe {
        let message = CString::new("message").unwrap();

        let entry = c_log_entry_create(
            1234567890,
            LogLevel::Info,
            std::ptr::null(),
            message.as_ptr(),
        );

        assert!(entry.is_null());
    }
}

#[test]
#[allow(unsafe_code)]
fn test_c_log_entry_create_null_message() {
    use keyrx_core::observability::entry::c_log_entry_create;

    unsafe {
        let target = CString::new("target").unwrap();

        let entry = c_log_entry_create(
            1234567890,
            LogLevel::Info,
            target.as_ptr(),
            std::ptr::null(),
        );

        assert!(entry.is_null());
    }
}

#[test]
#[allow(unsafe_code)]
fn test_c_log_entry_free_null_is_safe() {
    use keyrx_core::observability::entry::c_log_entry_free;

    unsafe {
        c_log_entry_free(std::ptr::null_mut());
        // Should not crash
    }
}

#[test]
fn test_log_entry_clone() {
    let entry = LogEntry::new(LogLevel::Debug, "test", "Message")
        .with_field("key", "value")
        .with_span("span");

    let cloned = entry.clone();

    assert_eq!(entry.level, cloned.level);
    assert_eq!(entry.target, cloned.target);
    assert_eq!(entry.message, cloned.message);
    assert_eq!(entry.fields, cloned.fields);
    assert_eq!(entry.span, cloned.span);
    assert_eq!(entry.timestamp, cloned.timestamp);
}

#[test]
fn test_log_entry_debug_formatting() {
    let entry = LogEntry::new(LogLevel::Info, "test", "Debug test");
    let debug_str = format!("{:?}", entry);

    assert!(debug_str.contains("LogEntry"));
    assert!(debug_str.contains("Info"));
}

#[test]
fn test_log_level_repr_c() {
    // Verify the repr(C) values are as expected
    assert_eq!(LogLevel::Trace as u8, 0);
    assert_eq!(LogLevel::Debug as u8, 1);
    assert_eq!(LogLevel::Info as u8, 2);
    assert_eq!(LogLevel::Warn as u8, 3);
    assert_eq!(LogLevel::Error as u8, 4);
}

#[test]
fn test_log_entry_field_overwrite() {
    let entry = LogEntry::new(LogLevel::Info, "test", "Message")
        .with_field("key", "value1")
        .with_field("key", "value2");

    // Second value should overwrite
    assert_eq!(entry.fields.get("key"), Some(&"value2".to_string()));
}
