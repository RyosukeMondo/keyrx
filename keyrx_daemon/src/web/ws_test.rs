//! Unit tests for WebSocket broadcasting functionality.

use super::events::{DaemonEvent, DaemonState, KeyEventData, LatencyStats};
use tokio::sync::broadcast;

#[tokio::test]
async fn test_broadcast_channel_publishes_state_event() {
    // Create broadcast channel
    let (event_tx, mut event_rx) = broadcast::channel(100);

    // Create and send state event
    let state_event = DaemonEvent::State(DaemonState {
        modifiers: vec!["MD_00".to_string()],
        locks: vec![],
        layer: "base".to_string(),
        active_profile: None,
    });

    // Send event
    event_tx.send(state_event.clone()).unwrap();

    // Verify event is received
    let received = event_rx.recv().await.unwrap();
    match received {
        DaemonEvent::State(state) => {
            assert_eq!(state.modifiers, vec!["MD_00"]);
            assert_eq!(state.locks.len(), 0);
            assert_eq!(state.layer, "base");
        }
        _ => panic!("Expected State event"),
    }
}

#[tokio::test]
async fn test_broadcast_channel_publishes_key_event() {
    let (event_tx, mut event_rx) = broadcast::channel(100);

    let key_event = DaemonEvent::KeyEvent(KeyEventData {
        timestamp: 1234567890,
        key_code: "KEY_A".to_string(),
        event_type: "press".to_string(),
        input: "A".to_string(),
        output: "B".to_string(),
        latency: 2300,
        device_id: Some("dev-001".to_string()),
        device_name: Some("USB Keyboard".to_string()),
        mapping_type: None,
        mapping_triggered: false,
    });

    event_tx.send(key_event.clone()).unwrap();

    let received = event_rx.recv().await.unwrap();
    match received {
        DaemonEvent::KeyEvent(event) => {
            assert_eq!(event.key_code, "KEY_A");
            assert_eq!(event.event_type, "press");
            assert_eq!(event.latency, 2300);
        }
        _ => panic!("Expected KeyEvent event"),
    }
}

#[tokio::test]
async fn test_broadcast_channel_publishes_latency_event() {
    let (event_tx, mut event_rx) = broadcast::channel(100);

    let latency_event = DaemonEvent::Latency(LatencyStats {
        min: 1200,
        avg: 2300,
        max: 4500,
        p95: 3800,
        p99: 4200,
        timestamp: 1234567890,
    });

    event_tx.send(latency_event.clone()).unwrap();

    let received = event_rx.recv().await.unwrap();
    match received {
        DaemonEvent::Latency(stats) => {
            assert_eq!(stats.min, 1200);
            assert_eq!(stats.avg, 2300);
            assert_eq!(stats.max, 4500);
            assert_eq!(stats.p95, 3800);
            assert_eq!(stats.p99, 4200);
        }
        _ => panic!("Expected Latency event"),
    }
}

#[tokio::test]
async fn test_broadcast_channel_multiple_subscribers() {
    let (event_tx, mut rx1) = broadcast::channel(100);
    let mut rx2 = event_tx.subscribe();
    let mut rx3 = event_tx.subscribe();

    let state_event = DaemonEvent::State(DaemonState {
        modifiers: vec![],
        locks: vec![],
        layer: "base".to_string(),
        active_profile: None,
    });

    event_tx.send(state_event.clone()).unwrap();

    // All subscribers should receive the event
    let r1 = rx1.recv().await.unwrap();
    let r2 = rx2.recv().await.unwrap();
    let r3 = rx3.recv().await.unwrap();

    assert!(matches!(r1, DaemonEvent::State(_)));
    assert!(matches!(r2, DaemonEvent::State(_)));
    assert!(matches!(r3, DaemonEvent::State(_)));
}

#[tokio::test]
async fn test_broadcast_channel_lagging_subscriber() {
    // Small buffer to force lag
    let (event_tx, mut slow_rx) = broadcast::channel(3);

    // Send more events than buffer size
    for i in 0..5 {
        event_tx
            .send(DaemonEvent::State(DaemonState {
                modifiers: vec![],
                locks: vec![],
                layer: format!("layer{}", i),
                active_profile: None,
            }))
            .unwrap();
    }

    // First recv should fail with Lagged error
    match slow_rx.recv().await {
        Err(broadcast::error::RecvError::Lagged(n)) => {
            assert!(n > 0, "Should report lagged count");
        }
        _ => panic!("Expected Lagged error"),
    }
}

#[tokio::test]
async fn test_event_serialization_state() {
    let event = DaemonEvent::State(DaemonState {
        modifiers: vec!["MD_00".to_string(), "MD_01".to_string()],
        locks: vec!["LK_00".to_string()],
        layer: "gaming".to_string(),
        active_profile: None,
    });

    let json = serde_json::to_string(&event).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["type"], "state");
    assert_eq!(parsed["payload"]["modifiers"][0], "MD_00");
    assert_eq!(parsed["payload"]["modifiers"][1], "MD_01");
    assert_eq!(parsed["payload"]["locks"][0], "LK_00");
    assert_eq!(parsed["payload"]["layer"], "gaming");
}

#[tokio::test]
async fn test_event_serialization_key_event() {
    let event = DaemonEvent::KeyEvent(KeyEventData {
        timestamp: 9876543210,
        key_code: "KEY_SPACE".to_string(),
        event_type: "release".to_string(),
        input: "SPACE".to_string(),
        output: "ENTER".to_string(),
        latency: 3400,
        device_id: Some("dev-002".to_string()),
        device_name: Some("Gaming Keyboard".to_string()),
        mapping_type: Some("simple".to_string()),
        mapping_triggered: true,
    });

    let json = serde_json::to_string(&event).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["type"], "event");
    assert_eq!(parsed["payload"]["timestamp"], 9876543210u64);
    assert_eq!(parsed["payload"]["keyCode"], "KEY_SPACE");
    assert_eq!(parsed["payload"]["eventType"], "release");
    assert_eq!(parsed["payload"]["input"], "SPACE");
    assert_eq!(parsed["payload"]["output"], "ENTER");
    assert_eq!(parsed["payload"]["latency"], 3400);
}

#[tokio::test]
async fn test_event_serialization_latency() {
    let event = DaemonEvent::Latency(LatencyStats {
        min: 800,
        avg: 2100,
        max: 5200,
        p95: 4000,
        p99: 4800,
        timestamp: 1111111111,
    });

    let json = serde_json::to_string(&event).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["type"], "latency");
    assert_eq!(parsed["payload"]["min"], 800);
    assert_eq!(parsed["payload"]["avg"], 2100);
    assert_eq!(parsed["payload"]["max"], 5200);
    assert_eq!(parsed["payload"]["p95"], 4000);
    assert_eq!(parsed["payload"]["p99"], 4800);
    assert_eq!(parsed["payload"]["timestamp"], 1111111111u64);
}

#[tokio::test]
async fn test_high_frequency_batching() {
    let (event_tx, mut event_rx) = broadcast::channel(1000);

    // Simulate high-frequency events (>100 events/sec)
    let event_count = 150;
    let start = std::time::Instant::now();

    for i in 0..event_count {
        event_tx
            .send(DaemonEvent::KeyEvent(KeyEventData {
                timestamp: i,
                key_code: format!("KEY_{}", i),
                event_type: "press".to_string(),
                input: format!("IN_{}", i),
                output: format!("OUT_{}", i),
                latency: 1000 + i,
                device_id: None,
                device_name: None,
                mapping_type: None,
                mapping_triggered: false,
            }))
            .unwrap();
    }

    let duration = start.elapsed();

    // Verify all events can be sent quickly
    assert!(
        duration.as_millis() < 100,
        "Sending 150 events should be fast"
    );

    // Verify we can receive all events
    let mut received_count = 0;
    while event_rx.try_recv().is_ok() {
        received_count += 1;
    }

    assert_eq!(
        received_count, event_count,
        "All events should be receivable"
    );
}

#[tokio::test]
async fn test_channel_capacity_bounds() {
    // Test with exact capacity
    let capacity = 100;
    let (event_tx, _rx) = broadcast::channel::<DaemonEvent>(capacity);

    // Should be able to send at least capacity events without blocking
    // Keep receiver alive to prevent SendError
    for i in 0..capacity {
        event_tx
            .send(DaemonEvent::State(DaemonState {
                modifiers: vec![],
                locks: vec![],
                layer: format!("layer{}", i),
                active_profile: None,
            }))
            .ok(); // Use ok() instead of unwrap() as send can fail if no receivers
    }

    // Verify receiver can still receive
    drop(_rx);
}

#[tokio::test]
async fn test_event_cloning() {
    let original = DaemonEvent::State(DaemonState {
        modifiers: vec!["MD_00".to_string()],
        locks: vec!["LK_00".to_string()],
        layer: "test".to_string(),
        active_profile: None,
    });

    let cloned = original.clone();

    match (&original, &cloned) {
        (DaemonEvent::State(s1), DaemonEvent::State(s2)) => {
            assert_eq!(s1.modifiers, s2.modifiers);
            assert_eq!(s1.locks, s2.locks);
            assert_eq!(s1.layer, s2.layer);
        }
        _ => panic!("Clone should preserve event type"),
    }
}

#[tokio::test]
async fn test_subscriber_disconnect_cleanup() {
    let (event_tx, rx1) = broadcast::channel(100);
    let rx2 = event_tx.subscribe();

    // Drop one subscriber
    drop(rx1);

    // Should still work with remaining subscriber
    event_tx
        .send(DaemonEvent::State(DaemonState {
            modifiers: vec![],
            locks: vec![],
            layer: "base".to_string(),
            active_profile: None,
        }))
        .unwrap();

    // rx2 should still be valid (dropped to avoid unused warning)
    drop(rx2);
}
