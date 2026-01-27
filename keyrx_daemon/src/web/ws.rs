//! WebSocket endpoint for real-time event streaming.
//!
//! This module provides a WebSocket endpoint at /ws/events that streams
//! real-time events from the daemon to connected web clients.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use serde_json::json;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};

use crate::web::events::{DaemonEvent};

pub fn create_router(event_tx: broadcast::Sender<DaemonEvent>) -> Router {
    Router::new()
        .route("/", get(websocket_handler))
        .with_state(event_tx)
}

/// WebSocket upgrade handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(event_tx): State<broadcast::Sender<DaemonEvent>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, event_tx))
}

/// Handle WebSocket connection
async fn handle_websocket(mut socket: WebSocket, event_tx: broadcast::Sender<DaemonEvent>) {
    log::info!("WebSocket client connected (active senders: {})", event_tx.receiver_count());

    // Subscribe to daemon events
    let mut event_rx = event_tx.subscribe();
    log::info!("WebSocket subscribed to daemon events (total receivers: {})", event_tx.receiver_count());

    // Send welcome message
    let welcome = json!({
        "type": "connected",
        "payload": {
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    });

    if socket
        .send(Message::Text(welcome.to_string()))
        .await
        .is_err()
    {
        log::warn!("Failed to send welcome message");
        return;
    }

    // Send periodic heartbeat messages
    let mut heartbeat_interval = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            // Forward daemon events to client
            Ok(event) = event_rx.recv() => {
                log::debug!("WebSocket received daemon event: {:?}",
                    match &event {
                        DaemonEvent::KeyEvent(e) => format!("KeyEvent({:?})", e.key_code),
                        DaemonEvent::State(_) => "State".to_string(),
                        DaemonEvent::Latency(_) => "Latency".to_string(),
                    }
                );

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
                log::debug!("WebSocket successfully sent event to client");
            }

            // Send heartbeat
            _ = heartbeat_interval.tick() => {
                let heartbeat = json!({
                    "type": "heartbeat",
                    "payload": {
                        "timestamp": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    }
                });

                if socket.send(Message::Text(heartbeat.to_string())).await.is_err() {
                    log::info!("WebSocket client disconnected (heartbeat failed)");
                    break;
                }
            }

            // Handle incoming messages
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        log::debug!("Received WebSocket message: {}", text);
                        // NOTE: Client command handling for subscribe/unsubscribe is tracked in GitHub issue
                        // See GitHub issue for WebSocket client-side event filtering enhancement
                        // Currently all events are broadcast to all clients (default behavior)
                    }
                    Some(Ok(Message::Close(_))) => {
                        log::info!("WebSocket client closed connection");
                        break;
                    }
                    Some(Ok(_)) => {
                        // Ignore binary/ping/pong messages
                    }
                    Some(Err(e)) => {
                        log::warn!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        log::info!("WebSocket client disconnected");
                        break;
                    }
                }
            }
        }
    }

    log::info!("WebSocket connection closed");
}

// ============================================================================
// DESIGN NOTE: Future Enhancement - Client-Side Event Filtering
// ============================================================================
//
// The WebSocket currently broadcasts all events to all clients (working as designed).
// For enhanced efficiency, consider implementing per-client event filtering:
//
// GitHub Issue: See project issues for "WebSocket client-side event filtering"
//
// The following architecture could support client-side filtering:
//
// 1. **Event Broadcasting Mechanism in Daemon**
//    - Add a broadcast channel in the daemon's main event loop
//    - Publish events (key press/release, state changes, latency updates) to the channel
//    - Example: use tokio::sync::broadcast::channel for multi-subscriber support
//
// 2. **Event Subscription in Web Server**
//    - When the web server starts, subscribe to the daemon's broadcast channel
//    - Store the channel receiver in Arc<Mutex<>> for sharing across WebSocket handlers
//    - Forward events from the channel to all connected WebSocket clients
//
// 3. **Event Filtering and Client Subscriptions**
//    - Allow clients to subscribe to specific event types (events, state, latency, errors)
//    - Maintain per-client subscription state
//    - Only forward events matching client's active subscriptions
//
// 4. **Event Message Format**
//    - Standardize on JSON messages with type-tagged payloads
//    - Example event types:
//      * "event" - Key press/release events
//      * "state" - Daemon state changes (modifiers/locks/layers)
//      * "latency" - Performance metrics updates
//      * "error" - Error notifications
//
// 5. **Implementation Example**
//
// ```rust
// // In daemon main.rs:
// let (event_tx, _) = tokio::sync::broadcast::channel(1000);
// let event_tx_clone = event_tx.clone();
//
// // In event processing loop:
// event_tx.send(Event::KeyPress { ... }).ok();
//
// // In web server (ws.rs):
// async fn handle_websocket(mut socket: WebSocket, event_rx: Receiver<Event>) {
//     loop {
//         tokio::select! {
//             // Forward daemon events to client
//             Ok(event) = event_rx.recv() => {
//                 let msg = json!({
//                     "type": "event",
//                     "payload": event,
//                 });
//                 socket.send(Message::Text(msg.to_string())).await.ok();
//             }
//             // ... other select branches ...
//         }
//     }
// }
// ```
//
// Expected event message formats:
//
// ```json
// {
//   "type": "event",
//   "payload": {
//     "timestamp_us": 1234567890,
//     "event_type": "press",
//     "key_code": "A",
//     "device_id": "serial-ABC123",
//     "layer": "base",
//     "latency_us": 2300
//   }
// }
// ```
//
// ```json
// {
//   "type": "state",
//   "payload": {
//     "timestamp_us": 1234567890,
//     "active_modifiers": ["MD_00", "MD_01"],
//     "active_locks": ["LK_00"],
//     "active_layer": "base"
//   }
// }
// ```
//
// ```json
// {
//   "type": "latency",
//   "payload": {
//     "timestamp_us": 1234567890,
//     "min_us": 1200,
//     "avg_us": 2300,
//     "max_us": 4500,
//     "p95_us": 3800,
//     "p99_us": 4200
//   }
// }
// ```

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_router() {
        let (event_tx, _) = broadcast::channel(100);
        let router = create_router(event_tx);
        assert!(std::mem::size_of_val(&router) > 0);
    }
}
