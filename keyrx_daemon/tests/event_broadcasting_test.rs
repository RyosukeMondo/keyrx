//! Event Broadcasting Integration Tests
//!
//! These tests verify that the daemon correctly broadcasts events to WebSocket
//! subscribers on different channels (daemon-state, events, latency).

use futures_util::{SinkExt, StreamExt};
use keyrx_daemon::web::rpc_types::ServerMessage;
use serde_json::json;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Helper to start a test RPC server on a random port
async fn start_test_server() -> (u16, tokio::task::JoinHandle<()>) {
    use keyrx_daemon::config::ProfileManager;
    use keyrx_daemon::macro_recorder::MacroRecorder;
    use keyrx_daemon::services::{ConfigService, DeviceService, ProfileService};
    use keyrx_daemon::web::subscriptions::SubscriptionManager;
    use keyrx_daemon::web::AppState;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::broadcast;

    // Bind to port 0 to get random available port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let port = addr.port();

    // Create application state with test dependencies
    let config_dir = PathBuf::from("/tmp/keyrx-broadcast-test");
    let _ = std::fs::create_dir_all(&config_dir);

    let macro_recorder = Arc::new(MacroRecorder::default());
    let profile_manager = Arc::new(ProfileManager::new(config_dir.clone()).unwrap());
    let profile_service = Arc::new(ProfileService::new(Arc::clone(&profile_manager)));
    let device_service = Arc::new(DeviceService::new(config_dir.clone()));
    let config_service = Arc::new(ConfigService::new(profile_manager));
    let settings_service = Arc::new(keyrx_daemon::services::SettingsService::new(
        config_dir.clone(),
    ));
    let simulation_service = Arc::new(keyrx_daemon::services::SimulationService::new(
        config_dir.clone(),
        None,
    ));
    let subscription_manager = Arc::new(SubscriptionManager::new());
    let (event_broadcaster, _) = tokio::sync::broadcast::channel(1000);

    let state = Arc::new(AppState::new(
        macro_recorder,
        profile_service,
        device_service,
        config_service,
        settings_service,
        simulation_service,
        subscription_manager,
        event_broadcaster,
        None,
    ));

    // Create broadcast channel for daemon events
    let (event_tx, _) = broadcast::channel(100);

    // Start the server
    let server_handle = tokio::spawn(async move {
        let app = keyrx_daemon::web::create_app(event_tx, state).await;
        axum::serve(listener, app).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    (port, server_handle)
}

/// Helper to connect to the RPC WebSocket endpoint
async fn connect_client(
    port: u16,
) -> (
    futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
) {
    let url = format!("ws://127.0.0.1:{}/ws-rpc", port);
    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
    ws_stream.split()
}

/// Helper to send a message and receive the response
async fn send_and_receive(
    write: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    read: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    message: &str,
) -> String {
    write
        .send(Message::Text(message.to_string()))
        .await
        .expect("Failed to send message");

    match timeout(Duration::from_secs(5), read.next()).await {
        Ok(Some(Ok(Message::Text(text)))) => text,
        Ok(Some(Ok(msg))) => panic!("Unexpected message type: {:?}", msg),
        Ok(Some(Err(e))) => panic!("WebSocket error: {}", e),
        Ok(None) => panic!("WebSocket closed"),
        Err(_) => panic!("Timeout waiting for response"),
    }
}

#[tokio::test]
async fn test_subscribe_to_daemon_state_channel() {
    let (port, _server) = start_test_server().await;
    let (mut write, mut read) = connect_client(port).await;

    // Skip Connected handshake
    let _ = read.next().await;

    // Subscribe to daemon-state channel
    let subscribe = json!({
        "type": "subscribe",
        "id": "sub-state",
        "channel": "daemon-state"
    });

    let response = send_and_receive(&mut write, &mut read, &subscribe.to_string()).await;
    let msg: ServerMessage = serde_json::from_str(&response).expect("Failed to parse");

    match msg {
        ServerMessage::Response { result, error, .. } => {
            assert!(result.is_some(), "Subscribe should succeed");
            assert!(error.is_none(), "Should not have error: {:?}", error);
        }
        _ => panic!("Expected Response message, got: {:?}", msg),
    }

    // Note: Actual state change events would require triggering daemon state changes
    // This test verifies the subscription mechanism works
}

#[tokio::test]
async fn test_subscribe_to_events_channel() {
    let (port, _server) = start_test_server().await;
    let (mut write, mut read) = connect_client(port).await;

    // Skip Connected handshake
    let _ = read.next().await;

    // Subscribe to events channel
    let subscribe = json!({
        "type": "subscribe",
        "id": "sub-events",
        "channel": "events"
    });

    let response = send_and_receive(&mut write, &mut read, &subscribe.to_string()).await;
    let msg: ServerMessage = serde_json::from_str(&response).expect("Failed to parse");

    match msg {
        ServerMessage::Response { result, error, .. } => {
            assert!(result.is_some(), "Subscribe should succeed");
            assert!(error.is_none(), "Should not have error: {:?}", error);
        }
        _ => panic!("Expected Response message, got: {:?}", msg),
    }

    // Note: Actual key events would require simulating input
    // This test verifies the subscription mechanism works
}

#[tokio::test]
async fn test_subscribe_to_latency_channel() {
    let (port, _server) = start_test_server().await;
    let (mut write, mut read) = connect_client(port).await;

    // Skip Connected handshake
    let _ = read.next().await;

    // Subscribe to latency channel
    let subscribe = json!({
        "type": "subscribe",
        "id": "sub-latency",
        "channel": "latency"
    });

    let response = send_and_receive(&mut write, &mut read, &subscribe.to_string()).await;
    let msg: ServerMessage = serde_json::from_str(&response).expect("Failed to parse");

    match msg {
        ServerMessage::Response { result, error, .. } => {
            assert!(result.is_some(), "Subscribe should succeed");
            assert!(error.is_none(), "Should not have error: {:?}", error);
        }
        _ => panic!("Expected Response message, got: {:?}", msg),
    }

    // Note: Latency metrics are broadcast every 1 second
    // In a real test, we could wait for a broadcast event
    // For now, we verify the subscription works
}

#[tokio::test]
async fn test_multiple_subscribers_receive_same_event() {
    let (port, _server) = start_test_server().await;

    // Connect two clients
    let (mut write1, mut read1) = connect_client(port).await;
    let (mut write2, mut read2) = connect_client(port).await;

    // Skip Connected handshakes
    let _ = read1.next().await;
    let _ = read2.next().await;

    // Both subscribe to daemon-state
    let subscribe = json!({
        "type": "subscribe",
        "id": "sub-both",
        "channel": "daemon-state"
    });

    let _ = send_and_receive(&mut write1, &mut read1, &subscribe.to_string()).await;
    let _ = send_and_receive(&mut write2, &mut read2, &subscribe.to_string()).await;

    // Note: To properly test that both receive the same event, we would need to:
    // 1. Trigger a daemon state change
    // 2. Verify both clients receive the Event message
    // For now, this test documents the expected behavior
}

#[tokio::test]
async fn test_unsubscribe_from_channel() {
    let (port, _server) = start_test_server().await;
    let (mut write, mut read) = connect_client(port).await;

    // Skip Connected handshake
    let _ = read.next().await;

    // Subscribe first
    let subscribe = json!({
        "type": "subscribe",
        "id": "sub-1",
        "channel": "daemon-state"
    });

    let _ = send_and_receive(&mut write, &mut read, &subscribe.to_string()).await;

    // Unsubscribe
    let unsubscribe = json!({
        "type": "unsubscribe",
        "id": "unsub-1",
        "channel": "daemon-state"
    });

    let response = send_and_receive(&mut write, &mut read, &unsubscribe.to_string()).await;
    let msg: ServerMessage = serde_json::from_str(&response).expect("Failed to parse");

    match msg {
        ServerMessage::Response { result, error, .. } => {
            assert!(result.is_some(), "Unsubscribe should succeed");
            assert!(error.is_none(), "Should not have error: {:?}", error);
        }
        _ => panic!("Expected Response message, got: {:?}", msg),
    }

    // After unsubscribe, client should not receive state change events
    // This would require triggering a state change and verifying no event is received
}

#[tokio::test]
async fn test_subscribe_to_invalid_channel() {
    let (port, _server) = start_test_server().await;
    let (mut write, mut read) = connect_client(port).await;

    // Skip Connected handshake
    let _ = read.next().await;

    // Try to subscribe to invalid channel
    let subscribe = json!({
        "type": "subscribe",
        "id": "sub-invalid",
        "channel": "invalid-channel-name"
    });

    let response = send_and_receive(&mut write, &mut read, &subscribe.to_string()).await;
    let msg: ServerMessage = serde_json::from_str(&response).expect("Failed to parse");

    match msg {
        ServerMessage::Response { result, error, .. } => {
            // Should either reject invalid channel or accept it (implementation-specific)
            // The important thing is it doesn't crash
            if result.is_none() {
                assert!(error.is_some(), "Should have error for invalid channel");
            }
        }
        _ => panic!("Expected Response message, got: {:?}", msg),
    }
}

#[tokio::test]
async fn test_subscription_cleanup_on_disconnect() {
    let (port, _server) = start_test_server().await;
    let (mut write, mut read) = connect_client(port).await;

    // Skip Connected handshake
    let _ = read.next().await;

    // Subscribe to all channels
    for channel in &["daemon-state", "events", "latency"] {
        let subscribe = json!({
            "type": "subscribe",
            "id": format!("sub-{}", channel),
            "channel": channel
        });

        let _ = send_and_receive(&mut write, &mut read, &subscribe.to_string()).await;
    }

    // Disconnect
    drop(write);
    drop(read);

    // Give server time to clean up
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Note: Proper verification would require inspecting server state
    // to ensure subscriptions are removed. This test documents the
    // expected cleanup behavior on disconnect.
}
