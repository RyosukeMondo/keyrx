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
