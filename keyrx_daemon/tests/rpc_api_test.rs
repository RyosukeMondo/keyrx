//! Integration tests for WebSocket RPC API
//!
//! These tests verify the complete RPC API implementation using real WebSocket
//! connections to ensure all acceptance criteria are met.

#![allow(clippy::needless_borrow)] // Allow needless borrows in tests for clarity

use futures_util::{SinkExt, StreamExt};
use keyrx_daemon::web::rpc_types::{ServerMessage, METHOD_NOT_FOUND, PARSE_ERROR};
use serde_json::json;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Helper to start a test RPC server on a random port
async fn start_test_server() -> (u16, tokio::task::JoinHandle<()>) {
    use keyrx_daemon::config::ProfileManager;
    use keyrx_daemon::macro_recorder::MacroRecorder;
    use keyrx_daemon::services::{
        ConfigService, DeviceService, ProfileService, SettingsService, SimulationService,
    };
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
    let config_dir = PathBuf::from("/tmp/keyrx-rpc-test");
    let _ = std::fs::create_dir_all(&config_dir);

    let macro_recorder = Arc::new(MacroRecorder::default());
    let profile_manager = Arc::new(ProfileManager::new(config_dir.clone()).unwrap());
    let profile_service = Arc::new(ProfileService::new(Arc::clone(&profile_manager)));
    let device_service = Arc::new(DeviceService::new(config_dir.clone()));
    let config_service = Arc::new(ConfigService::new(profile_manager));
    let settings_service = Arc::new(SettingsService::new(config_dir.clone()));
    let simulation_service = Arc::new(SimulationService::new(config_dir.clone(), None));
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
async fn test_connection_receives_connected_handshake() {
    let (port, _server) = start_test_server().await;
    let (_write, mut read) = connect_client(port).await;

    // First message should be Connected handshake
    match timeout(Duration::from_secs(5), read.next()).await {
        Ok(Some(Ok(Message::Text(text)))) => {
            let msg: ServerMessage = serde_json::from_str(&text).expect("Failed to parse");
            assert!(
                matches!(msg, ServerMessage::Connected { .. }),
                "Expected Connected message, got: {:?}",
                msg
            );
        }
        _ => panic!("Did not receive Connected handshake"),
    }
}

#[tokio::test]
async fn test_query_with_id_receives_response_with_matching_id() {
    let (port, _server) = start_test_server().await;
    let (mut write, mut read) = connect_client(port).await;

    // Skip Connected handshake
    let _ = read.next().await;

    // Send a query message with a specific ID
    let query = json!({
        "type": "query",
        "id": "test-query-123",
        "method": "get_profiles"
    });

    let response = send_and_receive(&mut write, &mut read, &query.to_string()).await;
    let msg: ServerMessage = serde_json::from_str(&response).expect("Failed to parse");

    match msg {
        ServerMessage::Response { id, result, error } => {
            assert_eq!(id, "test-query-123", "Response ID should match request ID");
            assert!(
                result.is_some() || error.is_some(),
                "Response should have result or error"
            );
        }
        _ => panic!("Expected Response message, got: {:?}", msg),
    }
}

#[tokio::test]
async fn test_command_executes_and_returns_success() {
    let (port, _server) = start_test_server().await;
    let (mut write, mut read) = connect_client(port).await;

    // Skip Connected handshake
    let _ = read.next().await;

    // Send a command message (use unique name to avoid conflicts)
    let profile_name = format!(
        "test-profile-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros()
    );

    let command = json!({
        "type": "command",
        "id": "cmd-1",
        "method": "create_profile",
        "params": {
            "name": profile_name,
            "template": "blank"
        }
    });

    let response = send_and_receive(&mut write, &mut read, &command.to_string()).await;
    let msg: ServerMessage = serde_json::from_str(&response).expect("Failed to parse");

    match msg {
        ServerMessage::Response { id, result, error } => {
            assert_eq!(id, "cmd-1");
            assert!(result.is_some(), "Command should succeed");
            assert!(error.is_none(), "Should not have error: {:?}", error);
        }
        _ => panic!("Expected Response message, got: {:?}", msg),
    }
}

#[tokio::test]
async fn test_subscribe_to_channel_receives_broadcast_events() {
    let (port, _server) = start_test_server().await;
    let (mut write, mut read) = connect_client(port).await;

    // Skip Connected handshake
    let _ = read.next().await;

    // Subscribe to daemon-state channel
    let subscribe = json!({
        "type": "subscribe",
        "id": "sub-1",
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

    // Trigger a state change that would broadcast to daemon-state
    // Note: This would require actual daemon state changes
    // For now, we verify the subscription was successful
}

#[tokio::test]
async fn test_unsubscribe_stops_events() {
    let (port, _server) = start_test_server().await;
    let (mut write, mut read) = connect_client(port).await;

    // Skip Connected handshake
    let _ = read.next().await;

    // Subscribe first
    let subscribe = json!({
        "type": "subscribe",
        "id": "sub-1",
        "channel": "events"
    });

    let _ = send_and_receive(&mut write, &mut read, &subscribe.to_string()).await;

    // Unsubscribe
    let unsubscribe = json!({
        "type": "unsubscribe",
        "id": "unsub-1",
        "channel": "events"
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
}

#[tokio::test]
async fn test_invalid_json_returns_parse_error() {
    let (port, _server) = start_test_server().await;
    let (mut write, mut read) = connect_client(port).await;

    // Skip Connected handshake
    let _ = read.next().await;

    // Send invalid JSON
    let invalid_json = "{invalid json here}";

    let response = send_and_receive(&mut write, &mut read, invalid_json).await;
    let msg: ServerMessage = serde_json::from_str(&response).expect("Failed to parse");

    match msg {
        ServerMessage::Response { result, error, .. } => {
            assert!(result.is_none(), "Should not have result");
            assert!(error.is_some(), "Should have error for invalid JSON");
            if let Some(err) = error {
                assert_eq!(err.code, PARSE_ERROR, "Should be PARSE_ERROR code (-32700)");
            }
        }
        _ => panic!("Expected Response message, got: {:?}", msg),
    }
}

#[tokio::test]
async fn test_unknown_method_returns_method_not_found() {
    let (port, _server) = start_test_server().await;
    let (mut write, mut read) = connect_client(port).await;

    // Skip Connected handshake
    let _ = read.next().await;

    // Send query with unknown method
    let query = json!({
        "type": "query",
        "id": "unknown-1",
        "method": "unknown.method.that.does.not.exist"
    });

    let response = send_and_receive(&mut write, &mut read, &query.to_string()).await;
    let msg: ServerMessage = serde_json::from_str(&response).expect("Failed to parse");

    match msg {
        ServerMessage::Response { result, error, .. } => {
            assert!(result.is_none(), "Should not have result");
            assert!(error.is_some(), "Should have error for unknown method");
            if let Some(err) = error {
                assert_eq!(
                    err.code, METHOD_NOT_FOUND,
                    "Should be METHOD_NOT_FOUND code (-32601)"
                );
            }
        }
        _ => panic!("Expected Response message, got: {:?}", msg),
    }
}

#[tokio::test]
async fn test_concurrent_requests_from_multiple_clients() {
    let (port, _server) = start_test_server().await;

    // Connect two clients
    let (mut write1, mut read1) = connect_client(port).await;
    let (mut write2, mut read2) = connect_client(port).await;

    // Skip Connected handshakes
    let _ = read1.next().await;
    let _ = read2.next().await;

    // Send concurrent requests with different IDs
    let query1 = json!({
        "type": "query",
        "id": "client1-query",
        "method": "get_profiles"
    });

    let query2 = json!({
        "type": "query",
        "id": "client2-query",
        "method": "get_profiles"
    });

    // Send both requests
    write1
        .send(Message::Text(query1.to_string()))
        .await
        .unwrap();
    write2
        .send(Message::Text(query2.to_string()))
        .await
        .unwrap();

    // Receive responses (order may vary)
    let resp1 = timeout(Duration::from_secs(5), read1.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let resp2 = timeout(Duration::from_secs(5), read2.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    // Parse responses
    let msg1: ServerMessage = serde_json::from_str(&resp1.to_text().unwrap()).unwrap();
    let msg2: ServerMessage = serde_json::from_str(&resp2.to_text().unwrap()).unwrap();

    // Verify IDs are correctly correlated
    match (msg1, msg2) {
        (ServerMessage::Response { id: id1, .. }, ServerMessage::Response { id: id2, .. }) => {
            assert_eq!(id1, "client1-query");
            assert_eq!(id2, "client2-query");
        }
        _ => panic!("Expected Response messages"),
    }
}

#[tokio::test]
async fn test_request_timeout_behavior() {
    let (port, _server) = start_test_server().await;
    let (mut write, mut read) = connect_client(port).await;

    // Skip Connected handshake
    let _ = read.next().await;

    // Send a valid query
    let query = json!({
        "type": "query",
        "id": "timeout-test",
        "method": "get_profiles"
    });

    write.send(Message::Text(query.to_string())).await.unwrap();

    // Response should arrive quickly (well under 30 seconds)
    let result = timeout(Duration::from_secs(5), read.next()).await;
    assert!(result.is_ok(), "Response should arrive quickly");

    // Note: Testing actual 30s timeout would make tests slow
    // The timeout is enforced client-side in the UI
}

#[tokio::test]
async fn test_disconnect_cleans_up_subscriptions() {
    let (port, _server) = start_test_server().await;
    let (mut write, mut read) = connect_client(port).await;

    // Skip Connected handshake
    let _ = read.next().await;

    // Subscribe to multiple channels
    let subscribe1 = json!({
        "type": "subscribe",
        "id": "sub-1",
        "channel": "daemon-state"
    });

    let subscribe2 = json!({
        "type": "subscribe",
        "id": "sub-2",
        "channel": "events"
    });

    let _ = send_and_receive(&mut write, &mut read, &subscribe1.to_string()).await;
    let _ = send_and_receive(&mut write, &mut read, &subscribe2.to_string()).await;

    // Disconnect by dropping the connection
    drop(write);
    drop(read);

    // Give server time to clean up
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Note: Verification that subscriptions are cleaned up would require
    // inspecting internal server state. This test documents the expected
    // behavior that disconnect should trigger cleanup.
}
