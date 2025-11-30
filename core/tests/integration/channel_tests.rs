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
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
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
    match rx.try_recv() {
        Err(TryRecvError::Disconnected) => {
            // Expected
        }
        other => panic!("Expected Disconnected, got {:?}", other),
    }
}
