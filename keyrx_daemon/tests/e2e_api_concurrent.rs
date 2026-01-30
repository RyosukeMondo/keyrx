//! End-to-End Concurrent API Tests
//!
//! Comprehensive concurrent testing for all fixed API endpoints in keyrx_daemon.
//! Tests verify that API endpoints handle concurrent requests correctly without
//! blocking, deadlocking, or corrupting data.
//!
//! Test Categories:
//! 1. Concurrent same endpoint - Multiple requests to same endpoint
//! 2. Concurrent mixed endpoints - Requests to different endpoints
//! 3. Regression tests - Verify specific bugs are fixed
//! 4. Race condition tests - Device registry concurrent modifications
//! 5. Stress tests - High-volume concurrent requests
//!
//! Run with: `cargo test -p keyrx_daemon --test e2e_api_concurrent`

mod common;

use common::test_app::TestApp;
use serde_json::json;
use std::time::Duration;

// ============================================================================
// 1. CONCURRENT SAME ENDPOINT TESTS
// ============================================================================

#[tokio::test]
async fn test_concurrent_get_profiles() {
    let app = TestApp::new().await;

    // Spawn 50 concurrent GET requests to /api/profiles
    let handles: Vec<_> = (0..50)
        .map(|i| {
            let client = app.clone_client();
            tokio::spawn(async move {
                let response = client.get("/api/profiles").await;
                (i, response.status().as_u16())
            })
        })
        .collect();

    let start = std::time::Instant::now();
    let results = futures_util::future::join_all(handles).await;
    let duration = start.elapsed();

    // All requests should succeed
    assert_eq!(results.len(), 50, "Should have 50 results");
    for result in &results {
        let (i, status) = result.as_ref().unwrap();
        assert_eq!(
            *status, 200,
            "Request {} should return 200 OK, got {}",
            i, status
        );
    }

    // Should complete concurrently (not 50 * individual_time)
    assert!(
        duration < Duration::from_millis(2000),
        "50 concurrent requests should complete in <2s, took {:?}",
        duration
    );

    println!(
        "✓ 50 concurrent GET /api/profiles completed in {:?}",
        duration
    );
}

#[tokio::test]
async fn test_concurrent_get_devices() {
    let app = TestApp::new().await;

    // Spawn 50 concurrent GET requests to /api/devices
    let handles: Vec<_> = (0..50)
        .map(|i| {
            let client = app.clone_client();
            tokio::spawn(async move {
                let response = client.get("/api/devices").await;
                (i, response.status().as_u16())
            })
        })
        .collect();

    let start = std::time::Instant::now();
    let results = futures_util::future::join_all(handles).await;
    let duration = start.elapsed();

    // All requests should succeed
    assert_eq!(results.len(), 50);
    for result in &results {
        let (i, status) = result.as_ref().unwrap();
        assert_eq!(
            *status, 200,
            "Request {} should return 200 OK, got {}",
            i, status
        );
    }

    assert!(
        duration < Duration::from_millis(2000),
        "50 concurrent requests should complete in <2s, took {:?}",
        duration
    );

    println!(
        "✓ 50 concurrent GET /api/devices completed in {:?}",
        duration
    );
}

#[tokio::test]
async fn test_concurrent_get_status() {
    let app = TestApp::new().await;

    // Spawn 100 concurrent GET requests to /api/status
    let handles: Vec<_> = (0..100)
        .map(|i| {
            let client = app.clone_client();
            tokio::spawn(async move {
                let response = client.get("/api/status").await;
                (i, response.status().as_u16())
            })
        })
        .collect();

    let start = std::time::Instant::now();
    let results = futures_util::future::join_all(handles).await;
    let duration = start.elapsed();

    // All requests should succeed
    assert_eq!(results.len(), 100);
    for result in &results {
        let (i, status) = result.as_ref().unwrap();
        assert!(
            *status == 200 || *status == 404,
            "Request {} should return 200/404, got {}",
            i,
            status
        );
    }

    assert!(
        duration < Duration::from_millis(2000),
        "100 concurrent requests should complete in <2s, took {:?}",
        duration
    );

    println!(
        "✓ 100 concurrent GET /api/status completed in {:?}",
        duration
    );
}

// ============================================================================
// 2. CONCURRENT MIXED ENDPOINTS TESTS
// ============================================================================

#[tokio::test]
async fn test_concurrent_mixed_endpoints() {
    let app = TestApp::new().await;

    // Spawn 10 concurrent requests to different endpoints
    let mut handles = Vec::new();

    for i in 0..10 {
        let client = app.clone_client();
        let endpoint = match i % 5 {
            0 => "/api/profiles",
            1 => "/api/devices",
            2 => "/api/status",
            3 => "/api/settings",
            _ => "/api/health",
        };

        let handle = tokio::spawn(async move {
            let response = client.get(endpoint).await;
            (endpoint.to_string(), response.status().as_u16())
        });
        handles.push(handle);
    }

    let start = std::time::Instant::now();
    let results = futures_util::future::join_all(handles).await;
    let duration = start.elapsed();

    // All requests should succeed
    assert_eq!(results.len(), 10);
    for result in &results {
        let (endpoint, status) = result.as_ref().unwrap();
        assert!(
            *status >= 200 && *status < 500,
            "Request to {} should return valid status, got {}",
            endpoint,
            status
        );
    }

    assert!(
        duration < Duration::from_millis(1000),
        "10 concurrent mixed requests should complete in <1s, took {:?}",
        duration
    );

    println!(
        "✓ 10 concurrent mixed endpoint requests completed in {:?}",
        duration
    );
}

#[tokio::test]
async fn test_concurrent_read_write_mix() {
    let app = TestApp::new().await;

    // Create a test profile first
    let create_response = app
        .post(
            "/api/profiles",
            &json!({
                "name": "concurrent-test",
                "template": "blank"
            }),
        )
        .await;
    assert!(create_response.status().is_success());

    // Spawn concurrent mix of read and write operations
    let mut handles = Vec::new();

    // 5 readers
    for i in 0..5 {
        let client = app.clone_client();
        handles.push(tokio::spawn(async move {
            let response = client.get("/api/profiles").await;
            (format!("read-{}", i), response.status().as_u16())
        }));
    }

    // 3 writers (profile activation)
    for i in 0..3 {
        let client = app.clone_client();
        handles.push(tokio::spawn(async move {
            let response = client
                .post("/api/profiles/concurrent-test/activate", &json!({}))
                .await;
            (format!("write-{}", i), response.status().as_u16())
        }));
    }

    // 2 more readers
    for i in 0..2 {
        let client = app.clone_client();
        handles.push(tokio::spawn(async move {
            let response = client.get("/api/profiles/concurrent-test/config").await;
            (format!("config-read-{}", i), response.status().as_u16())
        }));
    }

    let start = std::time::Instant::now();
    let results = futures_util::future::join_all(handles).await;
    let duration = start.elapsed();

    // All requests should succeed
    assert_eq!(results.len(), 10);
    for result in &results {
        let (op, status) = result.as_ref().unwrap();
        assert!(
            *status >= 200 && *status < 500,
            "Operation {} should return valid status, got {}",
            op,
            status
        );
    }

    assert!(
        duration < Duration::from_millis(1500),
        "10 concurrent read/write requests should complete in <1.5s, took {:?}",
        duration
    );

    println!(
        "✓ 10 concurrent read/write mix completed in {:?}",
        duration
    );
}

// ============================================================================
// 3. REGRESSION TESTS
// ============================================================================

#[tokio::test]
async fn test_config_freeze_regression() {
    // Verifies that profile activation doesn't block config page
    // This was the original bug reported by users
    let app = TestApp::new().await;

    // Create a test profile
    let create_response = app
        .post(
            "/api/profiles",
            &json!({
                "name": "freeze-test",
                "template": "blank"
            }),
        )
        .await;
    assert!(create_response.status().is_success());

    // STEP 1: Activate profile (potentially blocking operation)
    let activate_start = std::time::Instant::now();
    let activate_response = app
        .post("/api/profiles/freeze-test/activate", &json!({}))
        .await;
    let activate_duration = activate_start.elapsed();

    println!("✓ Profile activation completed in {:?}", activate_duration);
    assert!(activate_response.status().is_success());

    // STEP 2: Immediately request config page (should NOT freeze)
    let config_start = std::time::Instant::now();
    let config_timeout = Duration::from_secs(5);

    let config_result = tokio::time::timeout(
        config_timeout,
        app.get("/api/profiles/freeze-test/config"),
    )
    .await;

    match config_result {
        Ok(response) => {
            let config_duration = config_start.elapsed();
            println!("✓ Config page loaded in {:?}", config_duration);
            assert!(
                response.status().is_success(),
                "Config request should succeed"
            );

            // Verify response contains valid data
            let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
            assert!(
                body.get("name").is_some() || body.get("source").is_some(),
                "Config should have name or source field"
            );
            println!("✓ Config data valid");
        }
        Err(_) => {
            panic!(
                "✗ Config request TIMEOUT after {:?} - FREEZE REGRESSION DETECTED!",
                config_timeout
            );
        }
    }

    println!("✓ Config freeze regression test PASSED");
}

#[tokio::test]
async fn test_concurrent_profile_activation_no_deadlock() {
    // Verify that multiple concurrent profile activations don't deadlock
    let app = TestApp::new().await;

    // Create multiple test profiles
    for i in 0..5 {
        let response = app
            .post(
                "/api/profiles",
                &json!({
                    "name": format!("profile-{}", i),
                    "template": "blank"
                }),
            )
            .await;
        assert!(response.status().is_success());
    }

    // Activate all profiles concurrently
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let client = app.clone_client();
            tokio::spawn(async move {
                let response = client
                    .post(
                        &format!("/api/profiles/profile-{}/activate", i),
                        &json!({}),
                    )
                    .await;
                (i, response.status().as_u16())
            })
        })
        .collect();

    let start = std::time::Instant::now();
    let results = futures_util::future::join_all(handles).await;
    let duration = start.elapsed();

    // All activations should complete (no deadlock)
    assert_eq!(results.len(), 5);
    for result in &results {
        let (i, status) = result.as_ref().unwrap();
        assert!(
            *status >= 200 && *status < 500,
            "Activation {} should complete with valid status, got {}",
            i,
            status
        );
    }

    assert!(
        duration < Duration::from_secs(10),
        "5 concurrent activations should complete in <10s, took {:?}",
        duration
    );

    println!(
        "✓ 5 concurrent profile activations completed without deadlock in {:?}",
        duration
    );
}

// ============================================================================
// 4. RACE CONDITION TESTS
// ============================================================================

#[tokio::test]
async fn test_device_registry_concurrent_modifications() {
    // Tests concurrent modifications to device registry for race conditions
    let app = TestApp::new().await;

    // Get current devices
    let initial_response = app.get("/api/devices").await;
    assert_eq!(initial_response.status(), 200);
    let initial_body: serde_json::Value = initial_response.json().await.unwrap();
    let initial_count = initial_body["devices"].as_array().unwrap().len();

    // Spawn 20 concurrent requests that might modify device state
    let handles: Vec<_> = (0..20)
        .map(|i| {
            let client = app.clone_client();
            tokio::spawn(async move {
                // Alternate between reads and potential writes
                let response = if i % 2 == 0 {
                    client.get("/api/devices").await
                } else {
                    // Try to toggle a device (may fail if device doesn't exist)
                    client
                        .post("/api/devices/test-device/toggle", &json!({}))
                        .await
                };
                (i, response.status().as_u16())
            })
        })
        .collect();

    let start = std::time::Instant::now();
    let results = futures_util::future::join_all(handles).await;
    let duration = start.elapsed();

    // All requests should complete without errors
    assert_eq!(results.len(), 20);
    for result in &results {
        let (i, status) = result.as_ref().unwrap();
        assert!(
            *status >= 200 && *status < 500,
            "Request {} should return valid status, got {}",
            i,
            status
        );
    }

    // Verify device list is still valid after concurrent modifications
    let final_response = app.get("/api/devices").await;
    assert_eq!(final_response.status(), 200);
    let final_body: serde_json::Value = final_response.json().await.unwrap();
    let final_count = final_body["devices"].as_array().unwrap().len();

    // Device count should be consistent
    assert_eq!(
        initial_count, final_count,
        "Device count should remain consistent after concurrent operations"
    );

    println!(
        "✓ 20 concurrent device registry operations completed in {:?} without data corruption",
        duration
    );
}

#[tokio::test]
async fn test_concurrent_profile_config_reads() {
    // Tests that concurrent config reads don't corrupt data
    let app = TestApp::new().await;

    // Create a test profile
    let create_response = app
        .post(
            "/api/profiles",
            &json!({
                "name": "race-test",
                "template": "blank"
            }),
        )
        .await;
    assert!(create_response.status().is_success());

    // Read config once to get expected data
    let expected_response = app.get("/api/profiles/race-test/config").await;
    let expected_body: serde_json::Value = expected_response.json().await.unwrap();

    // Spawn 50 concurrent config reads
    let handles: Vec<_> = (0..50)
        .map(|i| {
            let client = app.clone_client();
            tokio::spawn(async move {
                let response = client.get("/api/profiles/race-test/config").await;
                let status = response.status().as_u16();
                let body: serde_json::Value = response.json().await.unwrap();
                (i, status, body)
            })
        })
        .collect();

    let start = std::time::Instant::now();
    let results = futures_util::future::join_all(handles).await;
    let duration = start.elapsed();

    // All reads should return consistent data
    assert_eq!(results.len(), 50);
    for result in &results {
        let (i, status, body) = result.as_ref().unwrap();
        assert_eq!(*status, 200, "Read {} should return 200 OK", i);

        // Verify data consistency
        assert_eq!(
            body.get("name"),
            expected_body.get("name"),
            "Config name should be consistent across concurrent reads"
        );
    }

    println!(
        "✓ 50 concurrent config reads completed with consistent data in {:?}",
        duration
    );
}

// ============================================================================
// 5. STRESS TESTS
// ============================================================================

#[tokio::test]
async fn test_api_stress_1000_requests() {
    // High-volume stress test with 1000 concurrent requests
    let app = TestApp::new().await;

    println!("Starting 1000 request stress test...");

    // Spawn 1000 concurrent requests across different endpoints
    let handles: Vec<_> = (0..1000)
        .map(|i| {
            let client = app.clone_client();
            tokio::spawn(async move {
                let endpoint = match i % 4 {
                    0 => "/api/profiles",
                    1 => "/api/devices",
                    2 => "/api/status",
                    _ => "/api/health",
                };

                let response = client.get(endpoint).await;
                (i, response.status().as_u16())
            })
        })
        .collect();

    let start = std::time::Instant::now();
    let results = futures_util::future::join_all(handles).await;
    let duration = start.elapsed();

    // All requests should complete
    assert_eq!(results.len(), 1000, "Should have 1000 results");

    let mut success_count = 0;
    let mut error_count = 0;

    for result in &results {
        let (_, status) = result.as_ref().unwrap();
        if *status >= 200 && *status < 400 {
            success_count += 1;
        } else {
            error_count += 1;
        }
    }

    // At least 95% should succeed
    let success_rate = (success_count as f64 / 1000.0) * 100.0;
    assert!(
        success_rate >= 95.0,
        "At least 95% of requests should succeed, got {:.1}%",
        success_rate
    );

    println!("✓ 1000 concurrent requests completed in {:?}", duration);
    println!("  Success: {} ({:.1}%)", success_count, success_rate);
    println!("  Errors: {} ({:.1}%)", error_count, 100.0 - success_rate);
    println!(
        "  Throughput: {:.0} req/s",
        1000.0 / duration.as_secs_f64()
    );
}

#[tokio::test]
async fn test_sustained_load() {
    // Sustained load test - moderate concurrent requests over time
    let app = TestApp::new().await;

    println!("Starting sustained load test (5 waves of 50 requests)...");

    let mut total_duration = Duration::ZERO;
    let mut all_success = true;

    for wave in 0..5 {
        let handles: Vec<_> = (0..50)
            .map(|i| {
                let client = app.clone_client();
                tokio::spawn(async move {
                    let response = client.get("/api/profiles").await;
                    (i, response.status().as_u16())
                })
            })
            .collect();

        let start = std::time::Instant::now();
        let results = futures_util::future::join_all(handles).await;
        let duration = start.elapsed();
        total_duration += duration;

        // Check all succeeded
        for result in &results {
            let (_, status) = result.as_ref().unwrap();
            if *status != 200 {
                all_success = false;
            }
        }

        println!("  Wave {} completed in {:?}", wave + 1, duration);

        // Small delay between waves
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    assert!(all_success, "All requests should succeed");
    println!("✓ Sustained load test completed in {:?}", total_duration);
    println!(
        "  Average wave time: {:?}",
        total_duration / 5
    );
}

// ============================================================================
// 6. INTEGRATION TESTS (Requires Running Daemon)
// ============================================================================

#[tokio::test]
#[ignore] // Run with: cargo test --ignored
async fn test_real_daemon_concurrent_api() {
    use reqwest::Client;

    let client = Client::new();
    let base_url = "http://localhost:9867";

    // Verify daemon is running
    let health = client
        .get(&format!("{}/api/health", base_url))
        .send()
        .await;
    assert!(
        health.is_ok(),
        "Daemon not running - start with: cargo run --bin keyrx_daemon test"
    );

    println!("✓ Daemon is running");

    // Test concurrent requests to real daemon
    let handles: Vec<_> = (0..20)
        .map(|i| {
            let client = client.clone();
            let base_url = base_url.to_string();
            tokio::spawn(async move {
                let endpoint = match i % 3 {
                    0 => "/api/profiles",
                    1 => "/api/devices",
                    _ => "/api/status",
                };
                let url = format!("{}{}", base_url, endpoint);
                let response = client.get(&url).send().await;
                (endpoint.to_string(), response.is_ok())
            })
        })
        .collect();

    let start = std::time::Instant::now();
    let results = futures_util::future::join_all(handles).await;
    let duration = start.elapsed();

    // All requests should succeed
    for result in &results {
        let (endpoint, success) = result.as_ref().unwrap();
        assert!(success, "Request to {} should succeed", endpoint);
    }

    println!(
        "✓ 20 concurrent requests to real daemon completed in {:?}",
        duration
    );
}

#[tokio::test]
#[ignore] // Run with: cargo test --ignored
async fn test_real_daemon_profile_activation_concurrent() {
    use reqwest::Client;

    let client = Client::new();
    let base_url = "http://localhost:9867";

    // Verify daemon is running
    let health = client
        .get(&format!("{}/api/health", base_url))
        .send()
        .await;
    assert!(
        health.is_ok(),
        "Daemon not running - start with: cargo run --bin keyrx_daemon test"
    );

    // Create test profile
    let create_response = client
        .post(&format!("{}/api/profiles", base_url))
        .json(&json!({
            "name": "real-concurrent-test",
            "template": "blank"
        }))
        .send()
        .await
        .expect("Failed to create profile");
    assert!(create_response.status().is_success());

    // Spawn concurrent activation and config reads
    let mut handles = Vec::new();

    // 3 activations
    for i in 0..3 {
        let client = client.clone();
        let base_url = base_url.to_string();
        handles.push(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(i * 50)).await;
            let response = client
                .post(&format!(
                    "{}/api/profiles/real-concurrent-test/activate",
                    base_url
                ))
                .json(&json!({}))
                .send()
                .await;
            (format!("activate-{}", i), response.is_ok())
        }));
    }

    // 5 config reads
    for i in 0..5 {
        let client = client.clone();
        let base_url = base_url.to_string();
        handles.push(tokio::spawn(async move {
            let response = client
                .get(&format!(
                    "{}/api/profiles/real-concurrent-test/config",
                    base_url
                ))
                .send()
                .await;
            (format!("config-{}", i), response.is_ok())
        }));
    }

    let start = std::time::Instant::now();
    let results = futures_util::future::join_all(handles).await;
    let duration = start.elapsed();

    // All should succeed
    for result in &results {
        let (op, success) = result.as_ref().unwrap();
        assert!(success, "Operation {} should succeed", op);
    }

    println!(
        "✓ Real daemon concurrent activation/config test completed in {:?}",
        duration
    );
}
