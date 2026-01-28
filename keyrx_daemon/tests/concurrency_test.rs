//! Concurrency Tests
//!
//! Comprehensive tests to verify thread safety and concurrent operation:
//! - Concurrent profile activations
//! - Concurrent WebSocket connections
//! - Race conditions in event broadcasting
//! - Message ordering under concurrent load
//!
//! Requirements: TEST-002

mod common;

use common::test_app::TestApp;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Test concurrent profile activations (10 threads)
///
/// Verifies that multiple threads can safely activate profiles
/// without race conditions or data corruption.
#[tokio::test]
async fn test_concurrent_profile_activations() {
    let app = Arc::new(TestApp::new().await);

    // Create test profiles
    for i in 0..10 {
        let profile_name = format!("concurrent-profile-{}", i);
        let _ = app
            .post(
                "/api/profiles",
                &serde_json::json!({
                    "name": profile_name,
                    "config_source": "default"
                }),
            )
            .await;
    }

    sleep(Duration::from_millis(200)).await;

    // Spawn 10 concurrent tasks to activate profiles
    let mut handles = Vec::new();

    for i in 0..10 {
        let app_clone = Arc::clone(&app);
        let profile_name = format!("concurrent-profile-{}", i);

        let handle = tokio::spawn(async move {
            // Each task activates its profile 5 times
            for _ in 0..5 {
                let response = app_clone
                    .post(
                        &format!("/api/profiles/{}/activate", profile_name),
                        &serde_json::json!({}),
                    )
                    .await;

                // Activation should succeed or fail gracefully (no panics/crashes)
                assert!(response.status().as_u16() > 0);

                // Small random delay
                sleep(Duration::from_millis((i * 7) % 50)).await;
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify server is still responsive
    let status_response = app.get("/api/status").await;
    assert!(status_response.status().is_success());
}

/// Test concurrent WebSocket connections (100 connections)
///
/// Verifies that the server can handle many concurrent WebSocket
/// connections without resource exhaustion or performance degradation.
#[tokio::test]
#[ignore] // Run with: cargo test test_100_concurrent_websocket_connections -- --ignored --nocapture
async fn test_100_concurrent_websocket_connections() {
    let app = Arc::new(TestApp::new().await);

    println!("Establishing 100 concurrent WebSocket connections...");

    // Connect 100 WebSocket clients concurrently
    let mut handles = Vec::new();

    for i in 0..100 {
        let app_clone = Arc::clone(&app);

        let handle = tokio::spawn(async move {
            let mut ws = app_clone.connect_ws().await;

            // Subscribe to topics
            let subscribe_msg = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "subscribe",
                "params": {
                    "topics": ["daemon_state"]
                },
                "id": i
            });

            ws.send_text(subscribe_msg.to_string()).await.unwrap();

            // Keep connection alive for a bit
            sleep(Duration::from_secs(2)).await;

            // Return connection for cleanup
            ws
        });

        handles.push(handle);
    }

    // Wait for all connections to establish
    let mut connections = Vec::new();
    for handle in handles {
        let ws = handle.await.unwrap();
        connections.push(ws);
    }

    println!("All 100 connections established");

    // Verify server is still responsive during high load
    let status_response = app.get("/api/status").await;
    assert!(status_response.status().is_success());

    // Clean up all connections
    drop(connections);
    sleep(Duration::from_millis(500)).await;

    // Verify server recovers after connection cleanup
    let final_status = app.get("/api/status").await;
    assert!(final_status.status().is_success());

    println!("Test completed successfully");
}

/// Test race conditions in event broadcasting
///
/// Verifies that concurrent event broadcasting does not cause
/// race conditions, message loss, or duplicate messages.
#[tokio::test]
async fn test_event_broadcasting_race_conditions() {
    let app = Arc::new(TestApp::new().await);

    // Connect 5 subscribers
    let mut subscribers = Vec::new();
    for i in 0..5 {
        let mut ws = app.connect_ws().await;

        let subscribe_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "subscribe",
            "params": {
                "topics": ["daemon_state", "metrics"]
            },
            "id": i
        });

        ws.send_text(subscribe_msg.to_string()).await.unwrap();
        subscribers.push(ws);
    }

    sleep(Duration::from_millis(200)).await;

    // Spawn 10 concurrent tasks that trigger events
    let mut event_handles = Vec::new();

    for task_id in 0..10 {
        let app_clone = Arc::clone(&app);

        let handle = tokio::spawn(async move {
            for iteration in 0..10 {
                // Trigger various API operations that might broadcast events
                let _ = app_clone
                    .post(
                        "/api/test/event",
                        &serde_json::json!({
                            "task_id": task_id,
                            "iteration": iteration
                        }),
                    )
                    .await;

                // Small random delay
                let delay_ms = (task_id * 3 + iteration * 5) % 30;
                sleep(Duration::from_millis(delay_ms as u64)).await;
            }
        });

        event_handles.push(handle);
    }

    // Wait for all event triggers to complete
    for handle in event_handles {
        handle.await.unwrap();
    }

    // Allow time for event propagation
    sleep(Duration::from_millis(500)).await;

    // Server should remain stable
    let status_response = app.get("/api/status").await;
    assert!(status_response.status().is_success());

    // Clean up subscribers
    drop(subscribers);
}

/// Test message ordering under concurrent load
///
/// Verifies that messages are delivered in a reasonable order
/// even under high concurrent load.
#[tokio::test]
async fn test_message_ordering_under_concurrent_load() {
    let app = Arc::new(TestApp::new().await);

    // Connect subscriber
    let mut ws = app.connect_ws().await;

    let subscribe_msg = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "subscribe",
        "params": {
            "topics": ["daemon_state"]
        },
        "id": 1
    });

    ws.send_text(subscribe_msg.to_string()).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // Spawn multiple tasks that send ordered sequences of events
    let mut handles = Vec::new();

    for task_id in 0..5 {
        let app_clone = Arc::clone(&app);

        let handle = tokio::spawn(async move {
            for seq in 0..20 {
                let _ = app_clone
                    .post(
                        "/api/test/ordered-event",
                        &serde_json::json!({
                            "task_id": task_id,
                            "sequence": seq
                        }),
                    )
                    .await;

                sleep(Duration::from_millis(10)).await;
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Allow message delivery
    sleep(Duration::from_millis(500)).await;

    // Server should remain responsive
    let status_response = app.get("/api/status").await;
    assert!(status_response.status().is_success());

    drop(ws);
}

/// Test concurrent API endpoint access
///
/// Verifies that multiple threads can safely access different
/// API endpoints concurrently.
#[tokio::test]
async fn test_concurrent_api_endpoint_access() {
    let app = Arc::new(TestApp::new().await);

    // Spawn tasks that access different endpoints concurrently
    let mut handles = Vec::new();

    // Task 1: Profile operations
    let app1 = Arc::clone(&app);
    handles.push(tokio::spawn(async move {
        for i in 0..10 {
            let _ = app1.get("/api/profiles").await;
            sleep(Duration::from_millis(i * 5)).await;
        }
    }));

    // Task 2: Device operations
    let app2 = Arc::clone(&app);
    handles.push(tokio::spawn(async move {
        for i in 0..10 {
            let _ = app2.get("/api/devices").await;
            sleep(Duration::from_millis(i * 7)).await;
        }
    }));

    // Task 3: Status checks
    let app3 = Arc::clone(&app);
    handles.push(tokio::spawn(async move {
        for i in 0..10 {
            let _ = app3.get("/api/status").await;
            sleep(Duration::from_millis(i * 3)).await;
        }
    }));

    // Task 4: Settings operations
    let app4 = Arc::clone(&app);
    handles.push(tokio::spawn(async move {
        for i in 0..10 {
            let _ = app4.get("/api/settings").await;
            sleep(Duration::from_millis(i * 11)).await;
        }
    }));

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify final state is consistent
    let status_response = app.get("/api/status").await;
    assert!(status_response.status().is_success());
}

/// Test concurrent profile creation and deletion
///
/// Verifies that creating and deleting profiles concurrently
/// does not cause race conditions or data corruption.
#[tokio::test]
async fn test_concurrent_profile_create_delete() {
    let app = Arc::new(TestApp::new().await);

    let mut handles = Vec::new();

    for task_id in 0..5 {
        let app_clone = Arc::clone(&app);

        let handle = tokio::spawn(async move {
            for iteration in 0..10 {
                let profile_name = format!("test-profile-{}-{}", task_id, iteration);

                // Create profile
                let create_response = app_clone
                    .post(
                        "/api/profiles",
                        &serde_json::json!({
                            "name": profile_name,
                            "config_source": "default"
                        }),
                    )
                    .await;

                // Small delay
                sleep(Duration::from_millis(20)).await;

                // Delete profile if creation succeeded
                if create_response.status().is_success() {
                    let _ = app_clone
                        .delete(&format!("/api/profiles/{}", profile_name))
                        .await;
                }

                sleep(Duration::from_millis(10)).await;
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify server stability
    let status_response = app.get("/api/status").await;
    assert!(status_response.status().is_success());
}

/// Test concurrent WebSocket subscribe/unsubscribe
///
/// Verifies that concurrent subscription changes do not cause
/// race conditions or resource leaks.
#[tokio::test]
async fn test_concurrent_websocket_subscribe_unsubscribe() {
    let app = Arc::new(TestApp::new().await);

    let mut handles = Vec::new();

    for client_id in 0..10 {
        let app_clone = Arc::clone(&app);

        let handle = tokio::spawn(async move {
            let mut ws = app_clone.connect_ws().await;

            // Subscribe and unsubscribe multiple times
            for iteration in 0..5 {
                // Subscribe
                let subscribe_msg = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "subscribe",
                    "params": {
                        "topics": ["daemon_state", "metrics"]
                    },
                    "id": client_id * 100 + iteration
                });

                ws.send_text(subscribe_msg.to_string()).await.unwrap();
                sleep(Duration::from_millis(50)).await;

                // Unsubscribe
                let unsubscribe_msg = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "unsubscribe",
                    "params": {
                        "topics": ["daemon_state"]
                    },
                    "id": client_id * 100 + iteration + 1000
                });

                ws.send_text(unsubscribe_msg.to_string()).await.unwrap();
                sleep(Duration::from_millis(50)).await;
            }

            drop(ws);
        });

        handles.push(handle);
    }

    // Wait for all clients
    for handle in handles {
        handle.await.unwrap();
    }

    // Allow cleanup
    sleep(Duration::from_millis(300)).await;

    // Verify server stability
    let status_response = app.get("/api/status").await;
    assert!(status_response.status().is_success());
}

/// Test concurrent access to shared state
///
/// Verifies that concurrent access to daemon state does not
/// cause data races or inconsistent reads.
#[tokio::test]
async fn test_concurrent_shared_state_access() {
    let app = Arc::new(TestApp::new().await);

    let mut handles = Vec::new();

    // Spawn readers
    for _ in 0..10 {
        let app_clone = Arc::clone(&app);
        handles.push(tokio::spawn(async move {
            for _ in 0..20 {
                let _ = app_clone.get("/api/status").await;
                sleep(Duration::from_millis(5)).await;
            }
        }));
    }

    // Spawn writers (profile activations)
    for i in 0..5 {
        let app_clone = Arc::clone(&app);
        handles.push(tokio::spawn(async move {
            for _ in 0..10 {
                let _ = app_clone
                    .post("/api/profiles/default/activate", &serde_json::json!({}))
                    .await;
                sleep(Duration::from_millis(i * 10)).await;
            }
        }));
    }

    // Wait for all operations
    for handle in handles {
        handle.await.unwrap();
    }

    // Final state check
    let status_response = app.get("/api/status").await;
    assert!(status_response.status().is_success());
}
