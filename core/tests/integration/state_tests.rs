#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Graceful shutdown and state management tests.
//!
//! These tests verify that drivers handle shutdown gracefully,
//! including edge cases like pending events and concurrent operations.

use super::{InputEvent, InputSource, KeyCode, MockInput};
use keyrx_core::mocks::MockCall;

#[tokio::test]
async fn graceful_shutdown_with_pending_events() {
    let mut input = MockInput::new();

    input.queue_event(InputEvent::key_down(KeyCode::A, 0));
    input.queue_event(InputEvent::key_down(KeyCode::B, 0));

    input.start().await.expect("start failed");

    // Stop without polling events - should not hang or panic
    input.stop().await.expect("stop failed");

    // Call history should show start and stop
    let history = input.call_history();
    assert!(matches!(history[0], MockCall::Start));
    assert!(matches!(history[1], MockCall::Stop));
}

#[tokio::test]
async fn graceful_shutdown_concurrent_operations() {
    let mut input = MockInput::new();

    input.start().await.expect("start failed");

    // Queue event while running
    input.queue_event(InputEvent::key_down(KeyCode::Space, 0));

    // Poll and stop in sequence
    let events = input.poll_events().await.expect("poll_events failed");
    assert_eq!(events.len(), 1);

    input.stop().await.expect("stop failed");
}
