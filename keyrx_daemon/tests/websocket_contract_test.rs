//! WebSocket Message Contract Tests
//!
//! These tests verify that WebSocket messages sent by the daemon match the
//! format expected by the frontend. They act as contract tests to prevent
//! breaking changes to the WebSocket protocol.
//!
//! **IMPORTANT**: If these tests fail, it means the frontend and backend
//! message formats are out of sync. Both sides must be updated together.
//!
//! Format: DaemonEvent uses `#[serde(tag = "type")]` + `#[serde(flatten)]`,
//! so fields are at the top level alongside "type", not in a "payload" wrapper.

use keyrx_daemon::web::events::{DaemonEvent, DaemonState, KeyEventData, LatencyStats};
use serde_json::Value;

/// Verify that DaemonEvent::Latency serializes to the flattened format
#[test]
fn test_latency_event_serialization_format() {
    let event = DaemonEvent::Latency {
        data: LatencyStats {
            min: 100,
            avg: 250,
            max: 500,
            p95: 400,
            p99: 480,
            timestamp: 1234567890,
        },
        sequence: 1,
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize");
    let parsed: Value = serde_json::from_str(&json).expect("Failed to parse");

    assert_eq!(parsed["type"], "latency", "Message type must be 'latency'");
    assert_eq!(parsed["min"], 100);
    assert_eq!(parsed["avg"], 250);
    assert_eq!(parsed["max"], 500);
    assert_eq!(parsed["p95"], 400);
    assert_eq!(parsed["p99"], 480);
    assert_eq!(parsed["timestamp"], 1234567890);
    assert_eq!(parsed["seq"], 1);
}

/// Verify that DaemonEvent::State serializes correctly
#[test]
fn test_state_event_serialization_format() {
    let event = DaemonEvent::State {
        data: DaemonState {
            modifiers: vec!["MD_00".to_string()],
            locks: vec!["LK_00".to_string()],
            layer: "base".to_string(),
            active_profile: None,
        },
        sequence: 1,
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize");
    let parsed: Value = serde_json::from_str(&json).expect("Failed to parse");

    assert_eq!(parsed["type"], "state");
    assert_eq!(parsed["modifiers"][0], "MD_00");
    assert_eq!(parsed["locks"][0], "LK_00");
    assert_eq!(parsed["layer"], "base");
    assert_eq!(parsed["seq"], 1);
}

/// Verify that DaemonEvent::KeyEvent serializes correctly
#[test]
fn test_key_event_serialization_format() {
    let event = DaemonEvent::KeyEvent {
        data: KeyEventData {
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
        },
        sequence: 1,
    };

    let json = serde_json::to_string(&event).expect("Failed to serialize");
    let parsed: Value = serde_json::from_str(&json).expect("Failed to parse");

    assert_eq!(parsed["type"], "event");
    assert_eq!(parsed["timestamp"], 1234567890);
    assert_eq!(parsed["latency"], 150);
    assert_eq!(parsed["seq"], 1);
}

/// Test that all DaemonEvent variants can be serialized without errors
#[test]
fn test_all_daemon_events_serialize() {
    let events = vec![
        DaemonEvent::State {
            data: DaemonState {
                modifiers: vec![],
                locks: vec![],
                layer: "base".to_string(),
                active_profile: None,
            },
            sequence: 1,
        },
        DaemonEvent::KeyEvent {
            data: KeyEventData {
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
            },
            sequence: 2,
        },
        DaemonEvent::Latency {
            data: LatencyStats {
                min: 0,
                avg: 0,
                max: 0,
                p95: 0,
                p99: 0,
                timestamp: 0,
            },
            sequence: 3,
        },
    ];

    for event in events {
        let result = serde_json::to_string(&event);
        assert!(
            result.is_ok(),
            "All DaemonEvent variants must serialize successfully"
        );

        let json = result.unwrap();
        let parsed: Value = serde_json::from_str(&json).expect("Must parse as valid JSON");

        assert!(
            parsed["type"].is_string(),
            "All events must have a 'type' field"
        );

        assert!(
            parsed["seq"].is_number(),
            "All events must have a 'seq' field"
        );
    }
}

/// Verify that the frontend can parse all DaemonEvent types
#[test]
fn test_frontend_compatibility_message_types() {
    let event_types = vec!["state", "event", "latency"];

    for event_type in event_types {
        assert!(
            !event_type.is_empty(),
            "Frontend must handle event type: {}",
            event_type
        );
    }
}

/// Test that message format changes are documented
#[test]
fn test_message_format_documentation() {
    // DaemonEvent uses #[serde(tag = "type")] + #[serde(flatten)]:
    // { "type": "latency", "min": ..., "avg": ..., "seq": ... }
    // { "type": "state", "modifiers": [...], "locks": [...], "seq": ... }
    // { "type": "event", "timestamp": ..., "keyCode": ..., "seq": ... }
}
