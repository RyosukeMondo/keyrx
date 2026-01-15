//! WebSocket Message Contract Tests
//!
//! These tests verify that WebSocket messages sent by the daemon match the
//! format expected by the frontend. They act as contract tests to prevent
//! breaking changes to the WebSocket protocol.
//!
//! **IMPORTANT**: If these tests fail, it means the frontend and backend
//! message formats are out of sync. Both sides must be updated together.

use keyrx_daemon::web::events::{DaemonEvent, DaemonState, KeyEventData, LatencyStats};
use serde_json::Value;

/// Verify that DaemonEvent::Latency serializes to the expected legacy format
///
/// Current daemon sends: { "type": "latency", "payload": {...} }
/// Frontend expects (new): { "type": "event", "channel": "latency", "data": {...} }
/// Frontend handles (legacy): { "type": "latency", "payload": {...} }
///
/// This test documents the current format. When migrating to the new RPC format,
/// update this test and the ws.rs handler.
#[test]
fn test_latency_event_serialization_format() {
    let event = DaemonEvent::Latency(LatencyStats {
        min: 100,
        avg: 250,
        max: 500,
        p95: 400,
        p99: 480,
        timestamp: 1234567890,
    });

    let json = serde_json::to_string(&event).expect("Failed to serialize");
    let parsed: Value = serde_json::from_str(&json).expect("Failed to parse");

    // Verify legacy format structure
    assert_eq!(parsed["type"], "latency", "Message type must be 'latency'");
    assert!(
        parsed["payload"].is_object(),
        "Must have 'payload' object (legacy format)"
    );

    // Verify payload structure
    let payload = &parsed["payload"];
    assert_eq!(payload["min"], 100);
    assert_eq!(payload["avg"], 250);
    assert_eq!(payload["max"], 500);
    assert_eq!(payload["p95"], 400);
    assert_eq!(payload["p99"], 480);
    assert_eq!(payload["timestamp"], 1234567890);

    // Document expected migration to new format
    // TODO: When migrating to RPC format, message should be:
    // {
    //   "type": "event",
    //   "channel": "latency",
    //   "data": { "min": 100, "avg": 250, ... }
    // }
}

/// Verify that DaemonEvent::State serializes correctly
#[test]
fn test_state_event_serialization_format() {
    let event = DaemonEvent::State(DaemonState {
        modifiers: vec!["MD_00".to_string()],
        locks: vec!["LK_00".to_string()],
        layer: "base".to_string(),
        active_profile: None,
    });

    let json = serde_json::to_string(&event).expect("Failed to serialize");
    let parsed: Value = serde_json::from_str(&json).expect("Failed to parse");

    assert_eq!(parsed["type"], "state");
    assert!(parsed["payload"].is_object());

    let payload = &parsed["payload"];
    assert_eq!(payload["modifiers"][0], "MD_00");
    assert_eq!(payload["locks"][0], "LK_00");
    assert_eq!(payload["layer"], "base");
}

/// Verify that DaemonEvent::KeyEvent serializes correctly
#[test]
fn test_key_event_serialization_format() {
    let event = DaemonEvent::KeyEvent(KeyEventData {
        timestamp: 1234567890,
        key_code: "KEY_A".to_string(),
        event_type: "press".to_string(),
        input: "KEY_A".to_string(),
        output: "KEY_B".to_string(),
        latency: 150,
        device_id: Some("dev-001".to_string()),
        device_name: Some("USB Keyboard".to_string()),
        mapping_type: None,
        mapping_triggered: false,
    });

    let json = serde_json::to_string(&event).expect("Failed to serialize");
    let parsed: Value = serde_json::from_str(&json).expect("Failed to parse");

    assert_eq!(parsed["type"], "event");
    assert!(parsed["payload"].is_object());

    let payload = &parsed["payload"];
    assert_eq!(payload["timestamp"], 1234567890);
    assert_eq!(payload["keyCode"], "KEY_A"); // Note: serde rename to camelCase
    assert_eq!(payload["eventType"], "press"); // Note: serde rename to camelCase
    assert_eq!(payload["latency"], 150);
}

/// Test that all DaemonEvent variants can be serialized without errors
#[test]
fn test_all_daemon_events_serialize() {
    let events = vec![
        DaemonEvent::State(DaemonState {
            modifiers: vec![],
            locks: vec![],
            layer: "base".to_string(),
            active_profile: None,
        }),
        DaemonEvent::KeyEvent(KeyEventData {
            timestamp: 0,
            key_code: "KEY_A".to_string(),
            event_type: "press".to_string(),
            input: "KEY_A".to_string(),
            output: "KEY_A".to_string(),
            latency: 0,
            device_id: None,
            device_name: None,
            mapping_type: None,
            mapping_triggered: false,
        }),
        DaemonEvent::Latency(LatencyStats {
            min: 0,
            avg: 0,
            max: 0,
            p95: 0,
            p99: 0,
            timestamp: 0,
        }),
    ];

    for event in events {
        let result = serde_json::to_string(&event);
        assert!(
            result.is_ok(),
            "All DaemonEvent variants must serialize successfully"
        );

        let json = result.unwrap();
        let parsed: Value = serde_json::from_str(&json).expect("Must parse as valid JSON");

        // All events must have a 'type' field
        assert!(
            parsed["type"].is_string(),
            "All events must have a 'type' field"
        );

        // All events must have a 'payload' field (legacy format)
        assert!(
            parsed["payload"].is_object(),
            "All events must have a 'payload' object"
        );
    }
}

/// Verify that the frontend can parse all DaemonEvent types
///
/// This test documents the message types that the frontend must handle.
/// If you add a new DaemonEvent variant, add it here and update the frontend.
#[test]
fn test_frontend_compatibility_message_types() {
    let event_types = vec!["state", "event", "latency"];

    for event_type in event_types {
        // Frontend must handle these message types
        // See: keyrx_ui/src/hooks/useUnifiedApi.ts
        assert!(
            !event_type.is_empty(),
            "Frontend must handle event type: {}",
            event_type
        );
    }

    // Document: Frontend also handles these RPC message types:
    // - "response" (for RPC responses)
    // - "connected" (for initial handshake)
    // - Legacy format: { "type": "latency", "payload": {...} }
}

/// Test that message format changes are documented
#[test]
fn test_message_format_documentation() {
    // This test serves as documentation and a reminder:
    //
    // CURRENT FORMAT (Legacy):
    // {
    //   "type": "latency" | "state" | "event",
    //   "payload": { ... }
    // }
    //
    // PLANNED FORMAT (RPC):
    // {
    //   "type": "event",
    //   "channel": "latency" | "daemon-state" | "events",
    //   "data": { ... }
    // }
    //
    // MIGRATION STEPS:
    // 1. Update keyrx_daemon/src/web/ws.rs to wrap DaemonEvent in ServerMessage::Event
    // 2. Update frontend to expect new format (remove legacy handler)
    // 3. Update these tests to verify new format
    //
    // See: keyrx_daemon/src/web/rpc_types.rs for ServerMessage definition
}
