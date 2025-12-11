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
//! Comprehensive tests for LogBridge FFI functionality (public API).
//!
//! Note: Many LogBridge tests are in tracing_integration_tests.rs since
//! push_entry() is a private method and is tested via tracing integration.

use keyrx_core::observability::bridge::LogBridge;

#[test]
fn test_bridge_creation_defaults() {
    let bridge = LogBridge::new();
    assert!(bridge.is_enabled());
    assert_eq!(bridge.len(), 0);
    assert!(bridge.is_empty());
}

#[test]
fn test_bridge_with_custom_buffer_size() {
    let bridge = LogBridge::with_buffer_size(100);
    assert!(bridge.is_enabled());
    // Cannot access private fields, but we can verify behavior
    assert_eq!(bridge.len(), 0);
}

#[test]
fn test_bridge_enable_disable() {
    let bridge = LogBridge::new();
    assert!(bridge.is_enabled());

    bridge.set_enabled(false);
    assert!(!bridge.is_enabled());

    bridge.set_enabled(true);
    assert!(bridge.is_enabled());
}

#[test]
fn test_bridge_drain_empty_buffer() {
    let bridge = LogBridge::new();

    let drained = bridge.drain();
    assert_eq!(drained.len(), 0);
}

#[test]
fn test_bridge_clear_empty_buffer() {
    let bridge = LogBridge::new();

    bridge.clear();
    assert_eq!(bridge.len(), 0);
    assert!(bridge.is_empty());
}

#[test]
fn test_bridge_callback_management() {
    let bridge = LogBridge::new();

    extern "C" fn test_callback(_entry: *const keyrx_core::observability::entry::CLogEntry) {
        // Test callback
    }

    bridge.set_callback(test_callback);
    bridge.clear_callback();

    // Callback methods should not panic
}

#[test]
fn test_bridge_default_trait() {
    let bridge = LogBridge::default();
    assert!(bridge.is_enabled());
    assert_eq!(bridge.len(), 0);
}

#[test]
fn test_bridge_is_empty_after_creation() {
    let bridge = LogBridge::new();
    assert!(bridge.is_empty());
}

#[test]
fn test_bridge_len_is_zero_initially() {
    let bridge = LogBridge::new();
    assert_eq!(bridge.len(), 0);
}

#[test]
fn test_bridge_enabled_by_default() {
    let bridge = LogBridge::new();
    assert!(bridge.is_enabled());
}

#[test]
fn test_bridge_clone_creates_independent_instance() {
    let bridge = LogBridge::new();
    let bridge_clone = bridge.clone();

    // Both should have same initial state
    assert_eq!(bridge.is_enabled(), bridge_clone.is_enabled());
    assert_eq!(bridge.len(), bridge_clone.len());
}

#[test]
fn test_bridge_multiple_enable_disable_cycles() {
    let bridge = LogBridge::new();

    for _ in 0..10 {
        bridge.set_enabled(false);
        assert!(!bridge.is_enabled());

        bridge.set_enabled(true);
        assert!(bridge.is_enabled());
    }
}

#[test]
fn test_bridge_drain_multiple_times() {
    let bridge = LogBridge::new();

    for _ in 0..5 {
        let drained = bridge.drain();
        assert_eq!(drained.len(), 0);
    }
}

#[test]
fn test_bridge_clear_multiple_times() {
    let bridge = LogBridge::new();

    for _ in 0..5 {
        bridge.clear();
        assert_eq!(bridge.len(), 0);
    }
}

// Additional tests that exercise the LogBridge via tracing integration
// are in tracing_integration_tests.rs, including:
// - Buffer overflow behavior
// - Log level preservation
// - Field preservation
// - Span preservation
// - FIFO ordering
// - Concurrent access
