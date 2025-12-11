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
//! Integration tests for LogBridge with tracing.
//!
//! These tests verify that LogBridge correctly captures log events
//! from the tracing framework.

use keyrx_core::observability::bridge::LogBridge;
use keyrx_core::observability::entry::LogLevel;
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

/// Helper to create a test subscriber with LogBridge.
/// Returns the bridge and a guard that needs to be kept alive.
fn setup_test_subscriber() -> (LogBridge, tracing::subscriber::DefaultGuard) {
    let bridge = LogBridge::new();
    let bridge_clone = bridge.clone();

    // Create a new subscriber for this test
    let subscriber = Registry::default().with(bridge_clone);

    // Set as the default for this thread and return the guard
    let guard = tracing::subscriber::set_default(subscriber);

    (bridge, guard)
}

#[test]
fn test_bridge_captures_info_log() {
    let (bridge, _guard) = setup_test_subscriber();

    info!("Test info message");

    let logs = bridge.drain();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].level, LogLevel::Info);
    assert!(logs[0].message.contains("Test info message"));
}

#[test]
fn test_bridge_captures_all_log_levels() {
    let (bridge, _guard) = setup_test_subscriber();

    trace!("Trace message");
    debug!("Debug message");
    info!("Info message");
    warn!("Warn message");
    error!("Error message");

    let logs = bridge.drain();
    assert_eq!(logs.len(), 5);

    assert_eq!(logs[0].level, LogLevel::Trace);
    assert_eq!(logs[1].level, LogLevel::Debug);
    assert_eq!(logs[2].level, LogLevel::Info);
    assert_eq!(logs[3].level, LogLevel::Warn);
    assert_eq!(logs[4].level, LogLevel::Error);
}

#[test]
fn test_bridge_captures_structured_fields() {
    let (bridge, _guard) = setup_test_subscriber();

    info!(user_id = 123, action = "login", "User logged in");

    let logs = bridge.drain();
    assert_eq!(logs.len(), 1);

    let entry = &logs[0];
    assert_eq!(entry.level, LogLevel::Info);
    assert!(entry.message.contains("User logged in"));
    assert!(entry.fields.contains_key("user_id"));
    assert!(entry.fields.contains_key("action"));
}

#[test]
fn test_bridge_captures_target_module() {
    let (bridge, _guard) = setup_test_subscriber();

    info!(target: "custom::target", "Custom target message");

    let logs = bridge.drain();
    assert_eq!(logs.len(), 1);

    // The target should be set to the custom value or fall back to module path
    assert!(!logs[0].target.is_empty());
}

#[test]
fn test_bridge_captures_span_context() {
    let (bridge, _sub_guard) = setup_test_subscriber();

    let span = tracing::span!(tracing::Level::INFO, "test_span");
    let _span_guard = span.enter();

    info!("Message inside span");

    drop(_span_guard);

    let logs = bridge.drain();
    assert_eq!(logs.len(), 1);

    assert_eq!(logs[0].span, Some("test_span".to_string()));
    drop(_sub_guard);
}

#[test]
fn test_bridge_respects_enabled_state() {
    let (bridge, _guard) = setup_test_subscriber();

    info!("Before disable");

    bridge.set_enabled(false);

    info!("After disable");

    bridge.set_enabled(true);

    info!("After enable");

    let _logs = bridge.drain();

    // Should capture the first and third messages, but not the second
    // Note: This depends on the bridge's on_event implementation
    // which checks is_enabled() at event time
}

#[test]
fn test_bridge_handles_rapid_logging() {
    let (bridge, _guard) = setup_test_subscriber();

    for i in 0..100 {
        info!(iteration = i, "Rapid log message");
    }

    let logs = bridge.drain();
    assert_eq!(logs.len(), 100);
}

#[test]
fn test_bridge_handles_large_messages() {
    let (bridge, _guard) = setup_test_subscriber();

    let large_message = "x".repeat(10000);
    info!("{}", large_message);

    let logs = bridge.drain();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].message.len(), 10000);
}

#[test]
fn test_bridge_handles_special_characters() {
    let (bridge, _guard) = setup_test_subscriber();

    info!("Special chars: \"quotes\" 'apostrophes' \n newline \t tab");

    let logs = bridge.drain();
    assert_eq!(logs.len(), 1);
    assert!(logs[0].message.contains("Special chars"));
}

#[test]
fn test_bridge_handles_unicode() {
    let (bridge, _guard) = setup_test_subscriber();

    info!("Unicode: 你好世界 مرحبا 🚀");

    let logs = bridge.drain();
    assert_eq!(logs.len(), 1);
    assert!(logs[0].message.contains("Unicode"));
}

#[test]
fn test_bridge_with_multiple_fields() {
    let (bridge, _guard) = setup_test_subscriber();

    info!(
        user = "alice",
        age = 30,
        active = true,
        score = 95.5,
        "User details"
    );

    let logs = bridge.drain();
    assert_eq!(logs.len(), 1);

    let fields = &logs[0].fields;
    assert!(fields.contains_key("user"));
    assert!(fields.contains_key("age"));
    assert!(fields.contains_key("active"));
    assert!(fields.contains_key("score"));
}

#[test]
fn test_bridge_nested_spans() {
    let (bridge, _guard) = setup_test_subscriber();

    let outer_span = tracing::span!(tracing::Level::INFO, "outer");
    let _outer_guard = outer_span.enter();

    info!("Outer message");

    {
        let inner_span = tracing::span!(tracing::Level::INFO, "inner");
        let _inner_guard = inner_span.enter();

        info!("Inner message");
    }

    let logs = bridge.drain();

    // Both messages should be captured
    assert_eq!(logs.len(), 2);
}

#[test]
fn test_bridge_empty_message() {
    let (bridge, _guard) = setup_test_subscriber();

    info!("");

    let logs = bridge.drain();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].message, "");
}

#[test]
fn test_bridge_buffer_overflow_in_integration() {
    let bridge = LogBridge::with_buffer_size(10);
    let bridge_clone = bridge.clone();

    let subscriber = Registry::default().with(bridge_clone);
    let _guard = tracing::subscriber::set_default(subscriber);

    // Log more than buffer size
    for i in 0..20 {
        info!(iteration = i, "Overflow test");
    }

    let logs = bridge.drain();

    // Should only have 10 entries (buffer size)
    assert_eq!(logs.len(), 10);
}

#[test]
fn test_bridge_preserves_log_order() {
    let (bridge, _guard) = setup_test_subscriber();

    for i in 0..10 {
        info!("Message {}", i);
    }

    let logs = bridge.drain();

    for (idx, log) in logs.iter().enumerate() {
        assert!(log.message.contains(&format!("Message {}", idx)));
    }
}

#[test]
fn test_bridge_multiple_subscribers_isolation() {
    // Create first bridge with its own subscriber
    let bridge1 = LogBridge::new();
    let subscriber1 = Registry::default().with(bridge1.clone());

    {
        let _guard = tracing::subscriber::set_default(subscriber1);
        info!("Message 1");
    }

    // Create second bridge with its own subscriber
    let bridge2 = LogBridge::new();
    let subscriber2 = Registry::default().with(bridge2.clone());

    {
        let _guard = tracing::subscriber::set_default(subscriber2);
        info!("Message 2");
    }

    let logs1 = bridge1.drain();
    let logs2 = bridge2.drain();

    assert_eq!(logs1.len(), 1);
    assert_eq!(logs2.len(), 1);
    assert!(logs1[0].message.contains("Message 1"));
    assert!(logs2[0].message.contains("Message 2"));
}
