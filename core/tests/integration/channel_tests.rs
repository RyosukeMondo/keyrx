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
//! Cross-platform channel communication tests.
//!
//! These tests verify the channel communication patterns used by drivers
//! to communicate with the engine.

use super::{InputEvent, KeyCode};

#[tokio::test]
async fn channel_communication_basic_flow() {
    // This test verifies the basic event flow pattern used by drivers
    use crossbeam_channel::{unbounded, TryRecvError};

    let (tx, rx) = unbounded::<InputEvent>();

    // Simulate driver sending events
    tx.send(InputEvent::key_down(KeyCode::CapsLock, 0)).unwrap();
    tx.send(InputEvent::key_up(KeyCode::CapsLock, 100)).unwrap();

    // Simulate engine receiving events
    let mut events = Vec::new();
    loop {
        match rx.try_recv() {
            Ok(event) => events.push(event),
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                unreachable!("channel should not disconnect while sender is in scope")
            }
        }
    }

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].key, KeyCode::CapsLock);
    assert!(events[0].pressed);
    assert_eq!(events[1].key, KeyCode::CapsLock);
    assert!(!events[1].pressed);
}

#[tokio::test]
async fn channel_communication_bounded_backpressure() {
    // Verify bounded channel behavior for potential backpressure scenarios
    use crossbeam_channel::bounded;

    let (tx, rx) = bounded::<InputEvent>(2);

    // Fill the channel
    tx.send(InputEvent::key_down(KeyCode::A, 0)).unwrap();
    tx.send(InputEvent::key_down(KeyCode::B, 0)).unwrap();

    // Channel is now full, try_send should fail
    let result = tx.try_send(InputEvent::key_down(KeyCode::C, 0));
    assert!(result.is_err());

    // Drain one event
    let _ = rx.recv().unwrap();

    // Now we should be able to send again
    tx.send(InputEvent::key_down(KeyCode::C, 0)).unwrap();
}

#[test]
fn channel_disconnect_detection() {
    use crossbeam_channel::{unbounded, TryRecvError};

    let (tx, rx) = unbounded::<InputEvent>();

    // Drop the sender
    drop(tx);

    // Receiver should detect disconnect
    assert!(
        matches!(rx.try_recv(), Err(TryRecvError::Disconnected)),
        "expected Disconnected error after sender is dropped"
    );
}
