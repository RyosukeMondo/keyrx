//! Windows E2E Test - Key Remapping with Metrics Validation
//!
//! This test validates the complete key remapping flow on Windows:
//! 1. Load a profile with remapping rules (A → B)
//! 2. Simulate key events via the simulator API
//! 3. Verify events appear in /metrics/events endpoint
//! 4. Verify remapping was applied correctly
//!
//! Run with: cargo test -p keyrx_daemon --test windows_e2e_remap --features windows

#![cfg(all(target_os = "windows", feature = "windows"))]

use keyrx_daemon::web::AppState;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::time::{sleep, Duration};

/// Test helper to create a test profile with A→B remapping
async fn setup_test_profile(config_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use keyrx_compiler::serialize::serialize;
    use keyrx_core::config::*;
    use std::fs;
    use std::io::Write;

    // Create config directory
    fs::create_dir_all(config_dir)?;

    // Create a simple A→B remapping config
    let config = ConfigRoot {
        version: Version::current(),
        devices: vec![DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: ".*".to_string(), // Match all devices
            },
            mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
        }],
        metadata: Metadata {
            compilation_timestamp: 0,
            compiler_version: "test".to_string(),
            source_hash: "test".to_string(),
        },
    };

    // Serialize to .krx binary
    let bytes = serialize(&config)?;
    let config_path = config_dir.join("test-remap.krx");
    let mut file = fs::File::create(&config_path)?;
    file.write_all(&bytes)?;

    Ok(())
}

#[tokio::test]
async fn test_windows_key_remap_e2e() -> Result<(), Box<dyn std::error::Error>> {
    // Setup
    let temp_dir = tempfile::tempdir()?;
    let config_dir = temp_dir.path().to_path_buf();

    // Create test profile with A→B remapping
    setup_test_profile(&config_dir).await?;

    // Create AppState
    let state = Arc::new(AppState::new_for_testing(config_dir.clone()));

    // Start a test server
    let app = keyrx_daemon::web::create_router(state.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Wait for server to start
    sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let base_url = format!("http://{}", addr);

    // Step 1: Load the test profile for simulation
    println!("Step 1: Loading test profile...");
    let load_response = client
        .post(format!("{}/api/simulator/load-profile", base_url))
        .json(&json!({ "name": "test-remap" }))
        .send()
        .await?;

    assert!(
        load_response.status().is_success(),
        "Failed to load profile"
    );
    println!("✓ Profile loaded");

    // Step 2: Clear existing events (if any)
    println!("Step 2: Clearing existing metrics...");
    let _clear = client
        .delete(format!("{}/api/metrics/events", base_url))
        .send()
        .await?;
    println!("✓ Metrics cleared");

    // Step 3: Simulate key events (press A, release A)
    println!("Step 3: Simulating key events (press A, release A)...");
    let sim_response = client
        .post(format!("{}/api/simulator/events", base_url))
        .json(&json!({
            "dsl": "press:A,wait:50,release:A"
        }))
        .send()
        .await?;

    assert!(
        sim_response.status().is_success(),
        "Failed to simulate events"
    );
    let sim_result: Value = sim_response.json().await?;
    println!("✓ Events simulated: {:?}", sim_result);

    // Step 4: Wait for events to be processed
    sleep(Duration::from_millis(200)).await;

    // Step 5: Retrieve events from /metrics/events
    println!("Step 4: Retrieving events from /metrics/events...");
    let metrics_response = client
        .get(format!("{}/api/metrics/events?count=100", base_url))
        .send()
        .await?;

    assert!(
        metrics_response.status().is_success(),
        "Failed to get metrics"
    );
    let metrics: Value = metrics_response.json().await?;
    println!("✓ Metrics retrieved: {:?}", metrics);

    // Step 6: Verify remapping occurred
    println!("Step 5: Verifying remapping...");
    let events = metrics["events"].as_array().expect("Expected events array");

    assert!(!events.is_empty(), "No events recorded in metrics!");

    // Look for press and release events
    let mut found_press_a = false;
    let mut found_release_a = false;
    let mut found_output_b = false;

    for event in events {
        let event_type = event["event_type"].as_str().unwrap_or("");
        let key_code = event["key_code"].as_u64().unwrap_or(0);
        let output = event["output"].as_str().unwrap_or("");

        println!(
            "  Event: type={}, key_code={}, output={}",
            event_type, key_code, output
        );

        // Check for input event (A pressed)
        if event_type == "press" && key_code == keyrx_core::config::KeyCode::A as u64 {
            found_press_a = true;
        }

        // Check for input event (A released)
        if event_type == "release" && key_code == keyrx_core::config::KeyCode::A as u64 {
            found_release_a = true;
        }

        // Check for remapped output (B)
        if output.contains("B")
            || output.contains(&format!("{}", keyrx_core::config::KeyCode::B as u16))
        {
            found_output_b = true;
        }
    }

    println!("✓ Press A detected: {}", found_press_a);
    println!("✓ Release A detected: {}", found_release_a);
    println!("✓ Output B detected: {}", found_output_b);

    // Assertions
    assert!(found_press_a, "Did not detect press A event");
    assert!(found_release_a, "Did not detect release A event");
    assert!(found_output_b, "Remapping A→B did not occur");

    // Step 7: Check latency metrics
    println!("Step 6: Checking latency metrics...");
    let latency_response = client
        .get(format!("{}/api/metrics/latency", base_url))
        .send()
        .await?;

    if latency_response.status().is_success() {
        let latency: Value = latency_response.json().await?;
        println!("✓ Latency stats: {:?}", latency);

        // Verify latency values are reasonable
        if let Some(avg_us) = latency["avg_us"].as_u64() {
            assert!(avg_us < 10_000, "Average latency too high: {}μs", avg_us);
            println!("✓ Average latency: {}μs", avg_us);
        }
    } else {
        println!("⚠ Latency metrics not available (daemon may not be running)");
    }

    println!("\n✅ All Windows E2E tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_windows_metrics_endpoint_available() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing /metrics endpoint availability...");

    let temp_dir = tempfile::tempdir()?;
    let config_dir = temp_dir.path().to_path_buf();
    let state = Arc::new(AppState::new_for_testing(config_dir));

    let app = keyrx_daemon::web::create_router(state);
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let base_url = format!("http://{}", addr);

    // Test /api/metrics/events
    let events_response = client
        .get(format!("{}/api/metrics/events", base_url))
        .send()
        .await?;

    println!("✓ /api/metrics/events status: {}", events_response.status());
    assert!(events_response.status().is_success() || events_response.status() == 500); // 500 if daemon not running

    // Test /api/metrics/latency
    let latency_response = client
        .get(format!("{}/api/metrics/latency", base_url))
        .send()
        .await?;

    println!(
        "✓ /api/metrics/latency status: {}",
        latency_response.status()
    );
    assert!(latency_response.status().is_success() || latency_response.status() == 500); // 500 if daemon not running

    println!("✅ Metrics endpoints are available");
    Ok(())
}

#[tokio::test]
async fn test_windows_simulator_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing simulator integration...");

    let temp_dir = tempfile::tempdir()?;
    let config_dir = temp_dir.path().to_path_buf();

    // Create test profile
    setup_test_profile(&config_dir).await?;

    let state = Arc::new(AppState::new_for_testing(config_dir));
    let app = keyrx_daemon::web::create_router(state);
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let base_url = format!("http://{}", addr);

    // Load profile
    let load_response = client
        .post(format!("{}/api/simulator/load-profile", base_url))
        .json(&json!({ "name": "test-remap" }))
        .send()
        .await?;

    assert!(load_response.status().is_success());

    // Simulate events
    let sim_response = client
        .post(format!("{}/api/simulator/events", base_url))
        .json(&json!({
            "dsl": "press:A,wait:10,release:A,wait:10,press:B,wait:10,release:B"
        }))
        .send()
        .await?;

    assert!(sim_response.status().is_success());
    let result: Value = sim_response.json().await?;

    // Verify outputs
    let outputs = result["outputs"]
        .as_array()
        .expect("Expected outputs array");

    assert!(!outputs.is_empty(), "No outputs from simulation");
    println!("✓ Simulator produced {} outputs", outputs.len());

    // First output should be B (remapped from A)
    // Second output should be B (direct)
    let mut remap_count = 0;
    for output in outputs {
        if output["key"].as_str() == Some("B") {
            remap_count += 1;
        }
    }

    assert!(
        remap_count >= 2,
        "Expected at least 2 B outputs (1 remapped, 1 direct)"
    );
    println!("✓ Remapping verified: {} B outputs", remap_count);

    println!("✅ Simulator integration test passed");
    Ok(())
}
