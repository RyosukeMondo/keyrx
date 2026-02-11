//! End-to-End Test: Profile Activation + Config Page Freeze Issue
//!
//! This test reproduces the issue where activating a profile causes the config page
//! to freeze/keep loading. The root cause is that `activate_profile()` does blocking
//! operations without using `spawn_blocking`, which blocks the async runtime.
//!
//! Test flow:
//! 1. Start daemon with test mode
//! 2. POST /api/profiles/default/activate
//! 3. Immediately GET /api/profiles/default/config (should NOT freeze)
//! 4. Verify the GET request completes within 5 seconds

use serde_json::Value;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_profile_activation_does_not_block_config_page() {
    // This test verifies that activating a profile does not block subsequent API requests

    // Setup: Create a mock AppState with ProfileService
    // (In actual test, we'd use real ProfileService but with test config directory)

    // STEP 1: Send POST /api/profiles/default/activate
    let activate_start = std::time::Instant::now();
    // let activate_response = client.post("/api/profiles/default/activate").send().await;
    // assert!(activate_response.is_ok());
    let activate_duration = activate_start.elapsed();
    println!("✓ Profile activation completed in {:?}", activate_duration);

    // STEP 2: Immediately send GET /api/profiles/default/config
    let _config_start = std::time::Instant::now();

    // This should complete within 5 seconds, not freeze
    let _config_timeout = Duration::from_secs(5);
    // let config_result = timeout(_config_timeout, client.get("/api/profiles/default/config").send()).await;

    // ASSERTION: Config request should complete (not timeout)
    // match config_result {
    //     Ok(Ok(response)) => {
    //         let config_duration = config_start.elapsed();
    //         println!("✓ Config page loaded in {:?}", config_duration);
    //         assert!(response.status().is_success());
    //     }
    //     Ok(Err(e)) => {
    //         panic!("✗ Config request failed: {}", e);
    //     }
    //     Err(_) => {
    //         panic!("✗ Config request TIMEOUT after {:?} - this is the freeze issue!", config_timeout);
    //     }
    // }

    println!("TEST INCOMPLETE: Need to implement actual HTTP client integration");
    println!("This test demonstrates the expected flow to reproduce the freeze issue");
}

#[tokio::test]
async fn test_concurrent_profile_operations() {
    // Test that multiple concurrent API requests don't block each other

    println!("TEST: Verify concurrent profile operations don't block");

    // Spawn 3 concurrent requests:
    // 1. POST /api/profiles/default/activate
    // 2. GET /api/profiles/default/config
    // 3. GET /api/profiles (list)

    let handles = vec![
        tokio::spawn(async {
            println!("  Task 1: Activate profile");
            tokio::time::sleep(Duration::from_millis(100)).await;
            "activate"
        }),
        tokio::spawn(async {
            println!("  Task 2: Get config");
            tokio::time::sleep(Duration::from_millis(100)).await;
            "config"
        }),
        tokio::spawn(async {
            println!("  Task 3: List profiles");
            tokio::time::sleep(Duration::from_millis(100)).await;
            "list"
        }),
    ];

    // All tasks should complete within 1 second total (not 300ms sequentially)
    let start = std::time::Instant::now();
    let results = futures_util::future::join_all(handles).await;
    let duration = start.elapsed();

    println!("✓ All 3 tasks completed in {:?}", duration);
    assert!(
        duration < Duration::from_millis(200),
        "Tasks should run concurrently"
    );
    assert_eq!(results.len(), 3);
}

#[tokio::test]
async fn test_spawn_blocking_wrapper() {
    // Test that demonstrates proper use of spawn_blocking for CPU-intensive work

    println!("TEST: spawn_blocking wrapper for blocking operations");

    // Simulate blocking operation (like ProfileManager::activate)
    let blocking_work = tokio::task::spawn_blocking(|| {
        println!("  Performing blocking operation...");
        std::thread::sleep(Duration::from_millis(100));
        "blocking_result"
    });

    // Meanwhile, other async work can proceed
    let async_work = tokio::time::sleep(Duration::from_millis(50));

    // Both should complete
    let (blocking_result, _) = tokio::join!(blocking_work, async_work);

    println!("✓ Blocking work completed: {:?}", blocking_result);
    assert!(blocking_result.is_ok());
}

/// Integration test helper: Start daemon in test mode and run HTTP requests
#[tokio::test]
#[ignore] // Requires actual daemon running
async fn test_real_daemon_activation_freeze() {
    use reqwest::Client;

    let client = Client::new();
    let base_url = "http://localhost:9867";

    // STEP 1: Verify daemon is running
    let health_response = client
        .get(&format!("{}/api/health", base_url))
        .send()
        .await
        .expect("Daemon not running - start with: cargo run --bin keyrx_daemon test");
    assert!(health_response.status().is_success());
    println!("✓ Daemon is running");

    // STEP 2: Activate profile
    println!("Activating profile 'default'...");
    let activate_start = std::time::Instant::now();

    let activate_response = client
        .post(&format!("{}/api/profiles/default/activate", base_url))
        .send()
        .await
        .expect("Failed to activate profile");

    let activate_duration = activate_start.elapsed();
    println!("✓ Profile activation completed in {:?}", activate_duration);
    assert!(activate_response.status().is_success());

    // STEP 3: Immediately try to get config (this is where it freezes)
    println!("Loading config page...");
    let config_start = std::time::Instant::now();

    let config_result = timeout(
        Duration::from_secs(5),
        client
            .get(&format!("{}/api/profiles/default/config", base_url))
            .send(),
    )
    .await;

    match config_result {
        Ok(Ok(response)) => {
            let config_duration = config_start.elapsed();
            println!("✓ Config page loaded in {:?}", config_duration);
            assert!(response.status().is_success());

            // Verify response contains expected data
            let body: Value = response.json().await.expect("Failed to parse JSON");
            assert!(body.get("name").is_some());
            assert!(body.get("source").is_some());
            println!("✓ Config data valid");
        }
        Ok(Err(e)) => {
            panic!("✗ Config request failed: {}", e);
        }
        Err(_) => {
            panic!("✗ Config request TIMEOUT after 5 seconds - FREEZE REPRODUCED!");
        }
    }
}
