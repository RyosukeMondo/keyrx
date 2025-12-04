//! Comprehensive tests for StructuredLogger.

use keyrx_core::observability::logger::{LogError, OutputFormat, StructuredLogger};
use tempfile::TempDir;
use tracing::Level;

#[test]
fn test_logger_builder_defaults() {
    let logger = StructuredLogger::new();
    // Can only test that the builder was created successfully
    // Fields are private, so we can't inspect them directly
    drop(logger);
}

#[test]
fn test_logger_with_custom_settings() {
    let logger = StructuredLogger::new()
        .with_format(OutputFormat::Json)
        .with_level(Level::DEBUG)
        .with_span_events(false);

    // Fields are private, verify builder pattern works
    drop(logger);
}

#[test]
fn test_logger_with_file_path() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test.log");

    let logger = StructuredLogger::new().with_file(&log_path);

    // Fields are private, verify builder pattern works
    drop(logger);
}

#[test]
fn test_logger_with_env_filter() {
    let logger = StructuredLogger::new().with_env_filter("keyrx=debug,tokio=info");

    // Fields are private, verify builder pattern works
    drop(logger);
}

#[test]
fn test_logger_output_format_variants() {
    assert_eq!(OutputFormat::Pretty, OutputFormat::Pretty);
    assert_eq!(OutputFormat::Json, OutputFormat::Json);
    assert_eq!(OutputFormat::Compact, OutputFormat::Compact);
    assert_ne!(OutputFormat::Pretty, OutputFormat::Json);
}

#[test]
fn test_logger_default_trait() {
    let logger = StructuredLogger::default();
    // Verify the default trait works
    drop(logger);
}

#[test]
fn test_logger_multiple_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let log_path1 = temp_dir.path().join("test1.log");
    let log_path2 = temp_dir.path().join("test2.log");

    let logger = StructuredLogger::new()
        .with_file(&log_path1)
        .with_file(&log_path2); // Should overwrite

    // Verify builder pattern works with multiple file calls
    drop(logger);
}

#[test]
fn test_logger_chain_all_methods() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("chain.log");

    let logger = StructuredLogger::new()
        .with_format(OutputFormat::Compact)
        .with_level(Level::TRACE)
        .with_file(&log_path)
        .with_span_events(true)
        .with_env_filter("test=trace");

    // Verify all methods can be chained
    drop(logger);
}

#[test]
fn test_log_error_display() {
    let err = LogError::AlreadyInitialized;
    assert_eq!(err.to_string(), "Logger already initialized");

    let err = LogError::InitError("test error".to_string());
    assert_eq!(err.to_string(), "Failed to initialize logger: test error");
}

#[test]
fn test_logger_with_span_events_toggle() {
    let logger1 = StructuredLogger::new().with_span_events(true);
    drop(logger1);

    let logger2 = StructuredLogger::new().with_span_events(false);
    drop(logger2);
}

// Note: We cannot test actual initialization multiple times in the same process
// because tracing global subscriber can only be set once. These tests verify
// the builder pattern configuration instead.
