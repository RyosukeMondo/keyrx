//! WebSocket endpoint for real-time event streaming.
//!
//! This module provides a WebSocket endpoint at /ws/events that streams
//! real-time events from the daemon to connected web clients.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde_json::json;
use std::collections::VecDeque;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};

use crate::web::events::DaemonEvent;

/// WS-004: Buffer for out-of-order messages
const MESSAGE_BUFFER_SIZE: usize = 10;

/// WS-004: Message buffer for handling out-of-order delivery
struct MessageBuffer {
    buffer: VecDeque<(u64, DaemonEvent)>,
    next_expected: u64,
}

impl MessageBuffer {
    fn new() -> Self {
        Self {
            buffer: VecDeque::with_capacity(MESSAGE_BUFFER_SIZE),
            next_expected: 0, // 0 = uninitialized, syncs on first message
        }
    }

    /// Add a message and return all messages that are now in order
    fn add_message(&mut self, seq: u64, event: DaemonEvent) -> Vec<DaemonEvent> {
        let mut result = Vec::new();

        // Auto-sync on first message received (client may connect late)
        if self.next_expected == 0 {
            self.next_expected = seq;
        }

        // If this is the next expected message, deliver it immediately
        if seq == self.next_expected {
            result.push(event);
            self.next_expected += 1;

            // Check buffer for any subsequent messages
            while let Some(idx) = self
                .buffer
                .iter()
                .position(|(s, _)| *s == self.next_expected)
            {
                if let Some((_, buffered_event)) = self.buffer.remove(idx) {
                    result.push(buffered_event);
                    self.next_expected += 1;
                }
            }
        } else if seq > self.next_expected {
            // Future message - buffer it if there's space
            if self.buffer.len() < MESSAGE_BUFFER_SIZE {
                self.buffer.push_back((seq, event));
            } else {
                log::warn!("Message buffer full, dropping out-of-order message {}", seq);
            }
        }
        // seq < next_expected is a duplicate or very old message - ignore

        result
    }
}

pub fn create_router(event_tx: broadcast::Sender<DaemonEvent>) -> Router {
    Router::new()
        .route("/", get(websocket_handler))
        .route("/health", get(health_handler))
        .with_state(event_tx)
}

/// WS-001: WebSocket health check endpoint
async fn health_handler(
    State(event_tx): State<broadcast::Sender<DaemonEvent>>,
) -> impl IntoResponse {
    let active_connections = event_tx.receiver_count();

    Json(json!({
        "status": "healthy",
        "websocket": {
            "active_connections": active_connections,
        },
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    }))
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
    use uuid::Uuid;

    let client_id = Uuid::new_v4().to_string();
    log::info!(
        "WebSocket client {} connected (active senders: {})",
        client_id,
        event_tx.receiver_count()
    );

    // Subscribe to daemon events
    let mut event_rx = event_tx.subscribe();
    log::info!(
        "WebSocket {} subscribed to daemon events (total receivers: {})",
        client_id,
        event_tx.receiver_count()
    );

    // WS-004: Message ordering buffer
    let mut message_buffer = MessageBuffer::new();

    // WS-001: Track last pong time for timeout detection
    let mut last_pong_time = std::time::Instant::now();

    // FIX MEM-003: Track lag events for backpressure and slow client disconnection
    let mut lag_count = 0u32;
    const MAX_LAG_EVENTS: u32 = 3; // Disconnect after 3 consecutive lag events

    // Send welcome message
    let welcome = json!({
        "type": "connected",
        "payload": {
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "clientId": client_id,
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

    // Send periodic heartbeat messages (WS-001: using ping frames)
    let mut heartbeat_interval = interval(Duration::from_secs(15));
    // WS-001: Timeout check interval
    let mut timeout_check_interval = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            // Forward daemon events to client (WS-004: with ordering, MEM-003: with lag handling)
            event_result = event_rx.recv() => {
                match event_result {
                    Ok(event) => {
                        // FIX MEM-003: Reset lag count on successful receive
                        lag_count = 0;

                        // Extract sequence number
                        let seq = match &event {
                            DaemonEvent::KeyEvent { sequence, .. } => *sequence,
                            DaemonEvent::State { sequence, .. } => *sequence,
                            DaemonEvent::Latency { sequence, .. } => *sequence,
                            DaemonEvent::Error { sequence, .. } => *sequence,
                        };

                        log::debug!("WebSocket {} received daemon event seq={}: {:?}",
                            client_id,
                            seq,
                            match &event {
                                DaemonEvent::KeyEvent { data, .. } => format!("KeyEvent({:?})", data.key_code),
                                DaemonEvent::State { .. } => "State".to_string(),
                                DaemonEvent::Latency { .. } => "Latency".to_string(),
                                DaemonEvent::Error { data, .. } => format!("Error({})", data.code),
                            }
                        );

                        // WS-004: Add to buffer and get ordered messages
                        let ordered_events = message_buffer.add_message(seq, event);

                        // Send all ordered messages
                        for ordered_event in ordered_events {
                            let json_msg = match serde_json::to_string(&ordered_event) {
                                Ok(json) => json,
                                Err(e) => {
                                    log::warn!("Failed to serialize event: {}", e);
                                    continue;
                                }
                            };

                            if socket.send(Message::Text(json_msg)).await.is_err() {
                                log::info!("WebSocket client {} disconnected (send failed)", client_id);
                                return;
                            }
                        }
                        log::debug!("WebSocket {} successfully sent ordered events to client", client_id);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        // FIX MEM-003: Handle slow client backpressure
                        lag_count += 1;
                        log::warn!(
                            "WebSocket client {} lagged (skipped {} messages, lag count {}/{})",
                            client_id, skipped, lag_count, MAX_LAG_EVENTS
                        );

                        // FIX MEM-003: Disconnect if client is consistently slow
                        if lag_count >= MAX_LAG_EVENTS {
                            log::error!(
                                "WebSocket client {} disconnected due to excessive lag ({} events skipped total)",
                                client_id, skipped
                            );
                            return;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        log::info!("WebSocket client {} event channel closed", client_id);
                        break;
                    }
                }
            }

            // WS-001: Send ping frames as heartbeat
            _ = heartbeat_interval.tick() => {
                if socket.send(Message::Ping(vec![])).await.is_err() {
                    log::info!("WebSocket client {} disconnected (ping failed)", client_id);
                    break;
                }
                log::trace!("Sent ping to client {}", client_id);
            }

            // WS-001: Check for timeout (30 seconds since last pong)
            _ = timeout_check_interval.tick() => {
                let elapsed = last_pong_time.elapsed();
                if elapsed > Duration::from_secs(30) {
                    log::warn!("WebSocket client {} timeout (no pong for {:?})", client_id, elapsed);
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
                        log::info!("WebSocket client {} closed connection", client_id);
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        // WS-001: Respond to ping frames with pong
                        if socket.send(Message::Pong(data)).await.is_err() {
                            log::info!("WebSocket client {} disconnected (pong failed)", client_id);
                            break;
                        }
                        last_pong_time = std::time::Instant::now();
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // WS-001: Client responded to our heartbeat
                        log::trace!("Received pong from client {}", client_id);
                        last_pong_time = std::time::Instant::now();
                    }
                    Some(Ok(_)) => {
                        // Ignore other message types
                    }
                    Some(Err(e)) => {
                        log::warn!("WebSocket {} error: {}", client_id, e);
                        break;
                    }
                    None => {
                        log::info!("WebSocket client {} disconnected", client_id);
                        break;
                    }
                }
            }
        }
    }

    // FIX MEM-002: When function exits, event_rx is dropped automatically
    // This unsubscribes from the broadcast channel via Rust's Drop trait
    // No manual cleanup needed - Rust's RAII handles it
    log::info!(
        "WebSocket connection {} closed (subscription auto-dropped)",
        client_id
    );
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
