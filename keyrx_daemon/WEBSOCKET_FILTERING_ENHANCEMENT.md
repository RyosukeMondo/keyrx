# WebSocket Client-Side Event Filtering Enhancement

**Status:** Tracked for future implementation
**Priority:** Enhancement (not blocking)
**Related Files:** `keyrx_daemon/src/web/ws.rs`, `keyrx_daemon/src/web/events.rs`

## Overview

Enhance the WebSocket event streaming to support client-side event filtering through subscribe/unsubscribe commands.

## Current State

The WebSocket endpoint at `/ws/events` is **fully functional** and streams all daemon events to connected clients:

- ✅ Event types: State changes, KeyEvents, Latency metrics
- ✅ Broadcasting: Uses `tokio::sync::broadcast::channel` for multi-subscriber support
- ✅ Heartbeat: Sends periodic heartbeat messages every 30 seconds
- ✅ Connection management: Handles client connect/disconnect gracefully
- ✅ Error handling: Client errors don't affect other clients
- ✅ Message handling: Receives text messages (ready for command processing)

**The system is production-ready** with all events broadcast to all clients (standard WebSocket pattern).

## Problem Statement

All connected clients receive all events, which:
1. Wastes bandwidth for clients that only need specific event types
2. Increases client-side filtering overhead
3. Doesn't scale optimally with multiple clients having different needs

## Proposed Solution

Implement **per-client event filtering** with subscribe/unsubscribe commands.

### Command Format

Clients send JSON commands to control their event subscriptions:

```json
// Subscribe to specific event types
{
  "command": "subscribe",
  "event_types": ["state", "event", "latency"]
}

// Unsubscribe from specific event types
{
  "command": "unsubscribe",
  "event_types": ["event"]
}

// Subscribe to all events (default behavior)
{
  "command": "subscribe",
  "event_types": ["all"]
}
```

### Command Response Format

```json
// Success
{
  "type": "command_response",
  "payload": {
    "success": true,
    "command": "subscribe",
    "active_subscriptions": ["state", "latency"]
  }
}

// Error
{
  "type": "command_response",
  "payload": {
    "success": false,
    "command": "subscribe",
    "error": "Invalid event type: 'invalid_type'"
  }
}
```

## Implementation Details

### File: `keyrx_daemon/src/web/ws.rs`

#### 1. Per-Client Subscription State

```rust
use std::collections::HashSet;

struct ClientSubscriptions {
    subscribed_types: HashSet<String>,
}

impl ClientSubscriptions {
    fn new() -> Self {
        let mut subscribed_types = HashSet::new();
        // Default: subscribe to all events (backward compatible)
        subscribed_types.insert("all".to_string());
        Self { subscribed_types }
    }

    fn is_subscribed(&self, event_type: &str) -> bool {
        self.subscribed_types.contains("all") ||
        self.subscribed_types.contains(event_type)
    }

    fn subscribe(&mut self, types: Vec<String>) -> Result<(), String> {
        // Validate event types
        for t in &types {
            if !matches!(t.as_str(), "all" | "state" | "event" | "latency") {
                return Err(format!("Invalid event type: '{}'", t));
            }
        }

        // Add subscriptions
        for t in types {
            self.subscribed_types.insert(t);
        }

        // Remove "all" if specific types are subscribed
        if self.subscribed_types.len() > 1 {
            self.subscribed_types.remove("all");
        }

        Ok(())
    }

    fn unsubscribe(&mut self, types: Vec<String>) {
        for t in types {
            self.subscribed_types.remove(&t);
        }

        // If no subscriptions remain, default to "all"
        if self.subscribed_types.is_empty() {
            self.subscribed_types.insert("all".to_string());
        }
    }

    fn get_active_subscriptions(&self) -> Vec<String> {
        self.subscribed_types.iter().cloned().collect()
    }
}
```

#### 2. Command Handler (line ~107)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct ClientCommand {
    command: String,
    event_types: Vec<String>,
}

async fn handle_client_command(
    command_text: &str,
    subscriptions: &mut ClientSubscriptions,
) -> Result<String, String> {
    let command: ClientCommand = serde_json::from_str(command_text)
        .map_err(|e| format!("Invalid command format: {}", e))?;

    match command.command.as_str() {
        "subscribe" => {
            subscriptions.subscribe(command.event_types)?;
            Ok(serde_json::to_string(&json!({
                "type": "command_response",
                "payload": {
                    "success": true,
                    "command": "subscribe",
                    "active_subscriptions": subscriptions.get_active_subscriptions(),
                }
            })).unwrap())
        }
        "unsubscribe" => {
            subscriptions.unsubscribe(command.event_types);
            Ok(serde_json::to_string(&json!({
                "type": "command_response",
                "payload": {
                    "success": true,
                    "command": "unsubscribe",
                    "active_subscriptions": subscriptions.get_active_subscriptions(),
                }
            })).unwrap())
        }
        _ => Err(format!("Unknown command: {}", command.command)),
    }
}
```

#### 3. Event Filtering in Main Loop (lines ~68-81)

```rust
async fn handle_websocket(mut socket: WebSocket, event_tx: broadcast::Sender<DaemonEvent>) {
    log::info!("WebSocket client connected");

    let mut event_rx = event_tx.subscribe();
    let mut heartbeat_interval = interval(Duration::from_secs(30));

    // Initialize client subscriptions (default: all events)
    let mut subscriptions = ClientSubscriptions::new();

    // ... send welcome message ...

    loop {
        tokio::select! {
            // Forward daemon events to client (with filtering)
            Ok(event) = event_rx.recv() => {
                // Check if client is subscribed to this event type
                let event_type = match &event {
                    DaemonEvent::State(_) => "state",
                    DaemonEvent::KeyEvent(_) => "event",
                    DaemonEvent::Latency(_) => "latency",
                };

                if !subscriptions.is_subscribed(event_type) {
                    continue; // Skip events client isn't subscribed to
                }

                let json_msg = match serde_json::to_string(&event) {
                    Ok(json) => json,
                    Err(e) => {
                        log::warn!("Failed to serialize event: {}", e);
                        continue;
                    }
                };

                if socket.send(Message::Text(json_msg)).await.is_err() {
                    log::info!("WebSocket client disconnected (send failed)");
                    break;
                }
            }

            // ... heartbeat handling ...

            // Handle incoming messages (commands)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        log::debug!("Received WebSocket message: {}", text);

                        match handle_client_command(&text, &mut subscriptions).await {
                            Ok(response) => {
                                if socket.send(Message::Text(response)).await.is_err() {
                                    log::warn!("Failed to send command response");
                                }
                            }
                            Err(error) => {
                                let error_response = json!({
                                    "type": "command_response",
                                    "payload": {
                                        "success": false,
                                        "error": error,
                                    }
                                });
                                socket.send(Message::Text(error_response.to_string())).await.ok();
                            }
                        }
                    }
                    // ... other message types ...
                }
            }
        }
    }
}
```

## Performance Considerations

### Requirements
- **Must not impact** core input processing (<1ms latency requirement)
- **Lock-free** patterns for event distribution
- **Client isolation**: Errors in one client don't affect others
- **O(1) filtering** per event per client

### Implementation Notes
- Subscription checking is O(1) with `HashSet`
- Event serialization happens once per event (not per client)
- Client errors are isolated (existing behavior maintained)
- No locks in hot path (existing broadcast channel is lock-free)

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_subscription_all() {
        let subs = ClientSubscriptions::new();
        assert!(subs.is_subscribed("state"));
        assert!(subs.is_subscribed("event"));
        assert!(subs.is_subscribed("latency"));
    }

    #[test]
    fn test_subscribe_specific_types() {
        let mut subs = ClientSubscriptions::new();
        subs.subscribe(vec!["state".to_string(), "latency".to_string()]).unwrap();

        assert!(subs.is_subscribed("state"));
        assert!(!subs.is_subscribed("event"));
        assert!(subs.is_subscribed("latency"));
    }

    #[test]
    fn test_unsubscribe() {
        let mut subs = ClientSubscriptions::new();
        subs.subscribe(vec!["state".to_string()]).unwrap();
        subs.unsubscribe(vec!["state".to_string()]);

        // Should default back to "all"
        assert!(subs.is_subscribed("event"));
    }

    #[test]
    fn test_invalid_event_type() {
        let mut subs = ClientSubscriptions::new();
        let result = subs.subscribe(vec!["invalid".to_string()]);
        assert!(result.is_err());
    }
}
```

### Integration Tests

1. **Multi-client filtering**: Connect multiple clients with different subscriptions, verify each receives only subscribed events
2. **Dynamic subscription changes**: Client changes subscriptions mid-session
3. **Backward compatibility**: Clients not sending commands receive all events (default)
4. **Error handling**: Invalid commands return error responses without disconnecting client
5. **Performance**: Measure overhead with 10+ clients with different subscriptions

## Acceptance Criteria

- [ ] Clients can send subscribe/unsubscribe commands
- [ ] Invalid commands receive error responses (JSON format)
- [ ] Event filtering works correctly per client (unit tested)
- [ ] Default behavior: all events streamed (backward compatible)
- [ ] Unit tests verify command parsing and filtering (≥80% coverage)
- [ ] Integration tests verify multi-client scenarios
- [ ] Documentation updated in ws.rs
- [ ] No performance regression in event processing (<1ms maintained)
- [ ] Client errors don't affect other clients (isolation verified)

## Backward Compatibility

**Important:** This enhancement is fully backward compatible:
- Existing clients continue to receive all events (default behavior)
- No breaking changes to message format
- Optional feature that clients can opt into

## Related Documentation

- **Current Implementation:** `keyrx_daemon/src/web/ws.rs` (lines 36-132)
- **Event Types:** `keyrx_daemon/src/web/events.rs`
- **Technical Debt Spec:** `.spec-workflow/specs/technical-debt-remediation/tasks.md` (Task 28)

## Decision

**Status:** Deferred to future enhancement

**Rationale:**
- Current implementation is fully functional for all use cases
- Broadcasting all events is standard WebSocket pattern
- Client-side filtering works well for current scale
- Enhancement adds complexity without immediate need
- Implementation can be added when scale demands it

**Next Steps:**
1. Create GitHub issue with this design document
2. Implement when one of these conditions is met:
   - 10+ concurrent clients reporting performance issues
   - Client bandwidth becomes a bottleneck
   - Client-side filtering overhead becomes measurable
   - User requests the feature

## References

- [WebSocket RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- [tokio::sync::broadcast](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html)
- [Axum WebSocket example](https://github.com/tokio-rs/axum/blob/main/examples/websockets/src/main.rs)
