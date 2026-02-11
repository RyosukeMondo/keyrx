//! WebSocket Infrastructure Tests
//!
//! Tests for all 5 WebSocket infrastructure bugs:
//! - WS-001: Missing Health Check Responses
//! - WS-002: Incorrect Reconnection Logic (frontend - tested via integration)
//! - WS-003: Race Conditions in Event Broadcasting
//! - WS-004: Message Ordering Issues
//! - WS-005: Duplicate Message Delivery

use axum::Router;
use futures_util::{SinkExt, StreamExt};
use keyrx_daemon::daemon::event_broadcaster::EventBroadcaster;
use keyrx_daemon::web::events::{DaemonEvent, DaemonState, KeyEventData, LatencyStats};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};

// ============================================================================
// WS-001: Health Check Responses (Ping/Pong)
// ============================================================================

#[tokio::test]
async fn test_ws001_ping_pong_handling() {
    // Create WebSocket server
    let (event_tx, _) = broadcast::channel::<DaemonEvent>(100);
    let ws_router = keyrx_daemon::web::ws::create_router(event_tx.clone());

    let app = Router::new().nest("/ws", ws_router);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give server time to start
    sleep(Duration::from_millis(100)).await;

    // Connect WebSocket client
    let url = format!("ws://{}/ws", addr);
    let (ws_stream, _) = connect_async(&url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    // Skip welcome message
    let _ = read.next().await;

    // Send ping
    write.send(WsMessage::Ping(vec![1, 2, 3])).await.unwrap();

    // Should receive pong (may need to skip server heartbeat pings)
    let result = timeout(Duration::from_secs(2), async {
        loop {
            match read.next().await {
                Some(Ok(WsMessage::Pong(data))) if data == vec![1, 2, 3] => {
                    return true; // Got our pong
                }
                Some(Ok(WsMessage::Ping(_))) => {
                    // Server heartbeat ping - respond and continue waiting
                    write.send(WsMessage::Pong(vec![])).await.unwrap();
                }
                Some(Ok(msg)) => {
                    panic!("Expected Pong, got {:?}", msg);
                }
                _ => {
                    panic!("Connection closed before pong received");
                }
            }
        }
    })
    .await;

    assert!(
        result.is_ok() && result.unwrap(),
        "Should receive pong response"
    );
}

#[tokio::test]
async fn test_ws001_server_heartbeat_ping() {
    // Create WebSocket server
    let (event_tx, _) = broadcast::channel::<DaemonEvent>(100);
    let ws_router = keyrx_daemon::web::ws::create_router(event_tx.clone());

    let app = Router::new().nest("/ws", ws_router);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    // Connect WebSocket client
    let url = format!("ws://{}/ws", addr);
    let (ws_stream, _) = connect_async(&url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    // Skip welcome message
    let _ = read.next().await;

    // Wait for server ping (sent every 15 seconds, but we'll wait up to 20)
    let ping_received = timeout(Duration::from_secs(20), async {
        loop {
            if let Some(Ok(msg)) = read.next().await {
                if matches!(msg, WsMessage::Ping(_)) {
                    return true;
                }
            }
        }
    })
    .await;

    assert!(
        ping_received.is_ok(),
        "Should receive ping from server within 20 seconds"
    );

    // Respond with pong
    write.send(WsMessage::Pong(vec![])).await.unwrap();
}

#[tokio::test]
#[ignore] // This test takes 45+ seconds to run - run manually with --ignored
async fn test_ws001_timeout_detection() {
    // This test verifies that the server disconnects clients that don't respond to pings
    // Note: This is a longer test due to the 30-second timeout + 5s check interval
    // The timeout mechanism works as follows:
    // 1. Client connects, last_pong_time = T0
    // 2. Server sends ping every 15s
    // 3. Server checks timeout every 5s
    // 4. If no pong for 30s → disconnect
    // Expected: disconnect at T=30-35s

    let (event_tx, _) = broadcast::channel::<DaemonEvent>(100);
    let ws_router = keyrx_daemon::web::ws::create_router(event_tx.clone());

    let app = Router::new().nest("/ws", ws_router);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    // Connect WebSocket client but don't respond to pings
    let url = format!("ws://{}/ws", addr);
    let (ws_stream, _) = connect_async(&url).await.unwrap();
    let (_, mut read) = ws_stream.split();

    // Skip welcome message
    let _ = read.next().await;

    // Wait for connection to be closed due to timeout (30s + 5s check interval = 35s max)
    // Add extra buffer for test reliability
    let result = timeout(Duration::from_secs(45), async {
        let mut ping_count = 0;
        loop {
            match read.next().await {
                Some(Ok(WsMessage::Ping(_))) => {
                    ping_count += 1;
                    log::debug!("Received ping #{}, not responding", ping_count);
                    // Ignore pings (don't respond)
                }
                Some(Ok(WsMessage::Close(_))) => {
                    log::debug!("Received close frame after {} pings", ping_count);
                    return true; // Connection closed
                }
                None => {
                    log::debug!("Connection closed (None) after {} pings", ping_count);
                    return true; // Connection closed
                }
                _ => {}
            }
        }
    })
    .await;

    assert!(
        result.is_ok() && result.unwrap(),
        "Server should close connection after timeout (waited 45s)"
    );
}

// ============================================================================
// WS-003: Race Conditions in Event Broadcasting
// ============================================================================

#[tokio::test]
async fn test_ws003_concurrent_subscribe_unsubscribe() {
    let (event_tx, _) = broadcast::channel::<DaemonEvent>(1000);
    let broadcaster = EventBroadcaster::new(event_tx.clone());

    // Spawn multiple tasks that subscribe/unsubscribe concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let broadcaster_clone = broadcaster.clone();
        let handle = tokio::spawn(async move {
            let client_id = format!("client-{}", i);
            broadcaster_clone.subscribe_client(&client_id);
            sleep(Duration::from_millis(10)).await;
            broadcaster_clone.unsubscribe_client(&client_id);
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete without panicking
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_ws003_concurrent_broadcasting() {
    let (event_tx, _) = broadcast::channel::<DaemonEvent>(1000);
    let broadcaster = EventBroadcaster::new(event_tx.clone());

    // Subscribe some clients
    for i in 0..5 {
        broadcaster.subscribe_client(&format!("client-{}", i));
    }

    // Broadcast events concurrently from multiple tasks
    let mut handles = vec![];

    for i in 0..20 {
        let broadcaster_clone = broadcaster.clone();
        let handle = tokio::spawn(async move {
            let event = KeyEventData {
                timestamp: i as u64,
                key_code: format!("KEY_{}", i),
                event_type: "press".to_string(),
                input: "A".to_string(),
                output: "B".to_string(),
                latency: 1000,
                device_id: None,
                device_name: None,
                mapping_type: None,
                mapping_triggered: false,
            };
            broadcaster_clone.broadcast_key_event(event);
        });
        handles.push(handle);
    }

    // Wait for all broadcasts to complete without panicking
    for handle in handles {
        handle.await.unwrap();
    }
}

// ============================================================================
// WS-004: Message Ordering Issues
// ============================================================================

#[tokio::test]
async fn test_ws004_message_sequence_numbers() {
    let (event_tx, mut event_rx) = broadcast::channel::<DaemonEvent>(100);
    let broadcaster = EventBroadcaster::new(event_tx.clone());

    // Broadcast multiple events
    for i in 0..10 {
        let event = KeyEventData {
            timestamp: i as u64,
            key_code: format!("KEY_{}", i),
            event_type: "press".to_string(),
            input: "A".to_string(),
            output: "B".to_string(),
            latency: 1000,
            device_id: None,
            device_name: None,
            mapping_type: None,
            mapping_triggered: false,
        };
        broadcaster.broadcast_key_event(event);
    }

    // Verify sequence numbers are monotonically increasing
    let mut last_seq = 0;
    for _ in 0..10 {
        let event = event_rx.recv().await.unwrap();
        let seq = match event {
            DaemonEvent::KeyEvent { sequence, .. } => sequence,
            DaemonEvent::Error { .. } => panic!("Expected KeyEvent, got Error"),
            _ => panic!("Expected KeyEvent"),
        };

        assert!(
            seq > last_seq,
            "Sequence numbers should be monotonically increasing"
        );
        last_seq = seq;
    }
}

#[tokio::test]
async fn test_ws004_different_event_types_share_sequence() {
    let (event_tx, mut event_rx) = broadcast::channel::<DaemonEvent>(100);
    let broadcaster = EventBroadcaster::new(event_tx.clone());

    // Broadcast different event types
    broadcaster.broadcast_key_event(KeyEventData {
        timestamp: 1,
        key_code: "KEY_A".to_string(),
        event_type: "press".to_string(),
        input: "A".to_string(),
        output: "B".to_string(),
        latency: 1000,
        device_id: None,
        device_name: None,
        mapping_type: None,
        mapping_triggered: false,
    });

    broadcaster.broadcast_state(DaemonState {
        modifiers: vec![],
        locks: vec![],
        layer: "base".to_string(),
        active_profile: None,
    });

    broadcaster.broadcast_latency(LatencyStats {
        min: 100,
        avg: 200,
        max: 300,
        p95: 250,
        p99: 280,
        timestamp: 1234567890,
    });

    // Verify all events have increasing sequence numbers
    let mut sequences = vec![];
    for _ in 0..3 {
        let event = event_rx.recv().await.unwrap();
        let seq = match event {
            DaemonEvent::KeyEvent { sequence, .. } => sequence,
            DaemonEvent::State { sequence, .. } => sequence,
            DaemonEvent::Latency { sequence, .. } => sequence,
            DaemonEvent::Error { .. } => panic!("Unexpected error event"),
        };
        sequences.push(seq);
    }

    // All sequences should be unique and increasing
    for i in 1..sequences.len() {
        assert!(sequences[i] > sequences[i - 1]);
    }
}

// ============================================================================
// WS-005: Duplicate Message Delivery
// ============================================================================

#[tokio::test]
async fn test_ws005_deduplication_tracking() {
    let (event_tx, _) = broadcast::channel::<DaemonEvent>(100);
    let broadcaster = EventBroadcaster::new(event_tx.clone());

    let client_id = "test-client";
    broadcaster.subscribe_client(client_id);

    // Mark some messages as delivered
    broadcaster.mark_delivered(client_id, 1);
    broadcaster.mark_delivered(client_id, 2);
    broadcaster.mark_delivered(client_id, 3);

    // Check if messages were delivered
    assert!(broadcaster.was_delivered(client_id, 1));
    assert!(broadcaster.was_delivered(client_id, 2));
    assert!(broadcaster.was_delivered(client_id, 3));
    assert!(!broadcaster.was_delivered(client_id, 4));
}

#[tokio::test]
async fn test_ws005_deduplication_ring_buffer() {
    let (event_tx, _) = broadcast::channel::<DaemonEvent>(100);
    let broadcaster = EventBroadcaster::new(event_tx.clone());

    let client_id = "test-client";
    broadcaster.subscribe_client(client_id);

    // Fill the ring buffer (size 1000)
    for i in 0..1100 {
        broadcaster.mark_delivered(client_id, i);
    }

    // Old messages (< 100) should be evicted
    assert!(!broadcaster.was_delivered(client_id, 50));

    // Recent messages should still be there
    assert!(broadcaster.was_delivered(client_id, 1050));
    assert!(broadcaster.was_delivered(client_id, 1099));
}

#[tokio::test]
async fn test_ws005_per_subscriber_tracking() {
    let (event_tx, _) = broadcast::channel::<DaemonEvent>(100);
    let broadcaster = EventBroadcaster::new(event_tx.clone());

    // Subscribe multiple clients
    broadcaster.subscribe_client("client-1");
    broadcaster.subscribe_client("client-2");

    // Mark messages as delivered to different clients
    broadcaster.mark_delivered("client-1", 1);
    broadcaster.mark_delivered("client-1", 2);
    broadcaster.mark_delivered("client-2", 1);
    broadcaster.mark_delivered("client-2", 3);

    // Verify per-client tracking
    assert!(broadcaster.was_delivered("client-1", 1));
    assert!(broadcaster.was_delivered("client-1", 2));
    assert!(!broadcaster.was_delivered("client-1", 3));

    assert!(broadcaster.was_delivered("client-2", 1));
    assert!(!broadcaster.was_delivered("client-2", 2));
    assert!(broadcaster.was_delivered("client-2", 3));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_integration_full_websocket_flow() {
    // Initialize logging for test
    let _ = env_logger::builder().is_test(true).try_init();

    // Create WebSocket server
    let (event_tx, _) = broadcast::channel::<DaemonEvent>(100);
    let broadcaster = EventBroadcaster::new(event_tx.clone());
    let ws_router = keyrx_daemon::web::ws::create_router(event_tx.clone());

    let app = Router::new().nest("/ws", ws_router);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    sleep(Duration::from_millis(200)).await;

    // Connect WebSocket client
    let url = format!("ws://{}/ws", addr);
    let (ws_stream, _) = connect_async(&url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    // Read welcome message
    let welcome = read.next().await.unwrap().unwrap();
    assert!(matches!(welcome, WsMessage::Text(_)));

    // Wait longer to ensure WebSocket is fully subscribed
    sleep(Duration::from_millis(1000)).await;

    eprintln!(
        "Broadcaster has {} subscribers before sending",
        broadcaster.has_subscribers()
    );

    // Broadcast events (not in separate task to avoid timing issues)
    for i in 0..5 {
        broadcaster.broadcast_key_event(KeyEventData {
            timestamp: i as u64,
            key_code: format!("KEY_{}", i),
            event_type: "press".to_string(),
            input: "A".to_string(),
            output: "B".to_string(),
            latency: 1000,
            device_id: None,
            device_name: None,
            mapping_type: None,
            mapping_triggered: false,
        });
        eprintln!("Broadcast event {}", i);
        sleep(Duration::from_millis(200)).await; // More delay between events
    }

    // Wait a bit for all messages to be delivered
    sleep(Duration::from_millis(500)).await;

    // Receive events (longer timeout for CI/parallel test execution)
    let result = timeout(Duration::from_secs(10), async {
        let mut received_count = 0;
        loop {
            match read.next().await {
                Some(Ok(WsMessage::Text(text))) => {
                    eprintln!("Received text: {}", text);
                    let event: serde_json::Value = serde_json::from_str(&text).unwrap();
                    if event["type"] == "event" {
                        received_count += 1;
                        eprintln!("Received event {}/{}", received_count, 5);
                        if received_count >= 5 {
                            return Ok(received_count);
                        }
                    }
                }
                Some(Ok(WsMessage::Ping(_))) => {
                    eprintln!("Received ping, sending pong");
                    write.send(WsMessage::Pong(vec![])).await.unwrap();
                }
                Some(Ok(msg)) => {
                    eprintln!("Received other: {:?}", msg);
                }
                None => {
                    eprintln!("Connection closed after {} events", received_count);
                    return Err(format!("Connection closed after {} events", received_count));
                }
                Some(Err(e)) => {
                    eprintln!("WebSocket error: {}", e);
                    return Err(format!("WebSocket error: {}", e));
                }
            }
        }
    })
    .await;

    match result {
        Ok(Ok(count)) => {
            assert_eq!(count, 5, "Should receive all 5 events");
        }
        Ok(Err(e)) => {
            panic!("Failed to receive events: {}", e);
        }
        Err(_) => {
            panic!("Timeout waiting for events");
        }
    }
}

// ============================================================================
// WS-001: Health Check Endpoint
// ============================================================================

#[tokio::test]
async fn test_ws001_health_endpoint() {
    let (event_tx, _) = broadcast::channel::<DaemonEvent>(100);
    let ws_router = keyrx_daemon::web::ws::create_router(event_tx.clone());

    let app = Router::new().nest("/ws", ws_router);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    // Make HTTP request to /ws/health
    let url = format!("http://{}/ws/health", addr);
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await.unwrap();

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "healthy");
    assert!(body["websocket"]["active_connections"].is_number());
    assert!(body["timestamp"].is_number());
}

// ============================================================================
// WS-005: Error Propagation
// ============================================================================

#[tokio::test]
async fn test_ws005_error_broadcasting() {
    let (event_tx, mut event_rx) = broadcast::channel::<DaemonEvent>(100);
    let broadcaster = EventBroadcaster::new(event_tx.clone());

    // Broadcast error event
    broadcaster.broadcast_error(
        "TEST_ERROR".to_string(),
        "Test error message".to_string(),
        Some("test context".to_string()),
    );

    // Receive error event
    let event = event_rx.recv().await.unwrap();
    match event {
        DaemonEvent::Error { data, sequence } => {
            assert_eq!(data.code, "TEST_ERROR");
            assert_eq!(data.message, "Test error message");
            assert_eq!(data.context, Some("test context".to_string()));
            assert!(data.timestamp > 0);
            assert!(sequence > 0);
        }
        _ => panic!("Expected Error event"),
    }
}

#[tokio::test]
async fn test_ws005_error_includes_context() {
    let (event_tx, mut event_rx) = broadcast::channel::<DaemonEvent>(100);
    let broadcaster = EventBroadcaster::new(event_tx.clone());

    // Broadcast error with context
    broadcaster.broadcast_error(
        "CONFIG_LOAD_FAILED".to_string(),
        "Failed to load configuration".to_string(),
        Some("/path/to/config.krx".to_string()),
    );

    let event = event_rx.recv().await.unwrap();
    match event {
        DaemonEvent::Error { data, .. } => {
            assert_eq!(data.code, "CONFIG_LOAD_FAILED");
            assert_eq!(data.context, Some("/path/to/config.krx".to_string()));
        }
        _ => panic!("Expected Error event"),
    }
}

#[tokio::test]
async fn test_ws005_error_without_context() {
    let (event_tx, mut event_rx) = broadcast::channel::<DaemonEvent>(100);
    let broadcaster = EventBroadcaster::new(event_tx.clone());

    // Broadcast error without context
    broadcaster.broadcast_error(
        "GENERIC_ERROR".to_string(),
        "Something went wrong".to_string(),
        None,
    );

    let event = event_rx.recv().await.unwrap();
    match event {
        DaemonEvent::Error { data, .. } => {
            assert_eq!(data.code, "GENERIC_ERROR");
            assert!(data.context.is_none());
        }
        _ => panic!("Expected Error event"),
    }
}
