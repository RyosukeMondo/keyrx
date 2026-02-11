//! Layer Contamination Diagnosis Test
//!
//! This test diagnoses the issue where input 'a' produces 'wa' (and possibly arrow).
//! It checks for:
//! 1. Multiple remappings being applied to the same input
//! 2. Layer state not being properly reset
//! 3. Tap-hold causing duplicate outputs
//! 4. Base mappings conflicting with tap-hold mappings

use keyrx_daemon::services::SimulationService;
use serde_json::json;
use serial_test::serial;
use std::path::PathBuf;
use std::sync::Arc;

mod common;
use common::test_app::TestApp;

#[tokio::test]
#[serial]
async fn test_simple_a_key_remapping() {
    let app = TestApp::new().await;

    // Create a simple profile: A -> B (no layers, no tap-hold)
    let simple_profile_rhai = r#"
device_start("*");
  map("VK_A", "VK_B");  // Simple A->B mapping
device_end();
"#;

    // Write profile file
    let profile_path = app.config_path().join("profiles").join("simple-test.rhai");
    std::fs::write(&profile_path, simple_profile_rhai).unwrap();

    // Activate profile
    let activate_response = app
        .post("/api/profiles/simple-test/activate", &json!({}))
        .await;

    assert_eq!(
        activate_response.status(),
        200,
        "Failed to activate simple profile: {}",
        activate_response.text().await.unwrap()
    );

    // Wait for activation
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Simulate key press A
    let sim_response = app
        .post(
            "/api/simulator/events",
            &json!({
                "dsl": "press:A,wait:50,release:A"
            }),
        )
        .await;

    assert_eq!(
        sim_response.status(),
        200,
        "Failed to simulate events: {}",
        sim_response.text().await.unwrap()
    );

    // Wait for events to be processed
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Get metrics
    let metrics_response = app.get("/api/metrics/events?count=100").await;
    let metrics: serde_json::Value = metrics_response.json().await.unwrap();
    let events = metrics["events"].as_array().unwrap();

    println!("Simple A->B mapping events:");
    for event in events {
        println!("  {:?}", event);
    }

    // Check that we get EXACTLY 2 events: press B, release B
    // NOT: press A, press B, release B (which would indicate double processing)
    let press_count = events.iter().filter(|e| e["event_type"] == "press").count();
    let release_count = events
        .iter()
        .filter(|e| e["event_type"] == "release")
        .count();

    assert_eq!(
        press_count, 1,
        "Expected 1 press event, got {}. Events: {:?}",
        press_count, events
    );
    assert_eq!(
        release_count, 1,
        "Expected 1 release event, got {}. Events: {:?}",
        release_count, events
    );
}

#[tokio::test]
#[serial]
async fn test_a_key_with_tap_hold() {
    let app = TestApp::new().await;

    // Create a tap-hold profile: A tap=Tab, hold=MD_09
    let tap_hold_profile_rhai = r#"
device_start("*");
  tap_hold("VK_A", "VK_Tab", "MD_09", 200);
device_end();
"#;

    // Write profile file
    let profile_path = app
        .config_path()
        .join("profiles")
        .join("tap-hold-test.rhai");
    std::fs::write(&profile_path, tap_hold_profile_rhai).unwrap();

    // Activate profile
    let activate_response = app
        .post("/api/profiles/tap-hold-test/activate", &json!({}))
        .await;

    assert_eq!(
        activate_response.status(),
        200,
        "Failed to activate tap-hold profile: {}",
        activate_response.text().await.unwrap()
    );

    // Wait for activation
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Simulate QUICK press A (tap, not hold)
    let sim_response = app
        .post(
            "/api/simulator/events",
            &json!({
                "dsl": "press:A,wait:50,release:A"
            }),
        )
        .await;

    assert_eq!(
        sim_response.status(),
        200,
        "Failed to simulate events: {}",
        sim_response.text().await.unwrap()
    );

    // Wait for tap-hold resolution
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // Get metrics
    let metrics_response = app.get("/api/metrics/events?count=100").await;
    let metrics: serde_json::Value = metrics_response.json().await.unwrap();
    let events = metrics["events"].as_array().unwrap();

    println!("Tap-hold A events:");
    for event in events {
        println!("  {:?}", event);
    }

    // Check that we get EXACTLY 2 events: press Tab, release Tab
    // NOT: press A, press Tab, release Tab (which would indicate both original and remapped)
    let press_count = events.iter().filter(|e| e["event_type"] == "press").count();
    let release_count = events
        .iter()
        .filter(|e| e["event_type"] == "release")
        .count();

    assert_eq!(
        press_count, 1,
        "Expected 1 press event for tap-hold, got {}. Events: {:?}",
        press_count, events
    );
    assert_eq!(
        release_count, 1,
        "Expected 1 release event for tap-hold, got {}. Events: {:?}",
        release_count, events
    );
}

#[tokio::test]
#[serial]
async fn test_complex_profile_with_conflicting_mappings() {
    let app = TestApp::new().await;

    // Create a profile with BOTH base mapping W->A AND tap-hold A->Tab
    // This tests the user's exact scenario
    let complex_profile_rhai = r#"
device_start("*");
  map("VK_W", "VK_A");                          // W -> A
  tap_hold("VK_A", "VK_Tab", "MD_09", 200);    // A tap=Tab, hold=MD_09
device_end();
"#;

    // Write profile file
    let profile_path = app.config_path().join("profiles").join("complex-test.rhai");
    std::fs::write(&profile_path, complex_profile_rhai).unwrap();

    // Activate profile
    let activate_response = app
        .post("/api/profiles/complex-test/activate", &json!({}))
        .await;

    assert_eq!(
        activate_response.status(),
        200,
        "Failed to activate complex profile: {}",
        activate_response.text().await.unwrap()
    );

    // Wait for activation
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    println!("\n=== Test 1: Press A directly ===");
    // Clear metrics
    app.delete("/api/metrics/events").await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Simulate press A
    let sim_response = app
        .post(
            "/api/simulator/events",
            &json!({
                "dsl": "press:A,wait:50,release:A"
            }),
        )
        .await;

    assert_eq!(sim_response.status(), 200);
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // Get metrics for A press
    let metrics_response = app.get("/api/metrics/events?count=100").await;
    let metrics: serde_json::Value = metrics_response.json().await.unwrap();
    let events_a = metrics["events"].as_array().unwrap();

    println!("Events when pressing A:");
    for event in events_a {
        println!("  {:?}", event);
    }

    // Expected: Press Tab, Release Tab (from tap-hold)
    // NOT: Press A, Press Tab, Release Tab (double output)
    let press_count_a = events_a
        .iter()
        .filter(|e| e["event_type"] == "press")
        .count();
    let release_count_a = events_a
        .iter()
        .filter(|e| e["event_type"] == "release")
        .count();

    assert_eq!(
        press_count_a, 1,
        "Expected 1 press event when pressing A, got {}. This indicates LAYER CONTAMINATION or DOUBLE PROCESSING. Events: {:?}",
        press_count_a, events_a
    );

    println!("\n=== Test 2: Press W (which maps to A) ===");
    // Clear metrics
    app.delete("/api/metrics/events").await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Simulate press W
    let sim_response = app
        .post(
            "/api/simulator/events",
            &json!({
                "dsl": "press:W,wait:50,release:W"
            }),
        )
        .await;

    assert_eq!(sim_response.status(), 200);
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // Get metrics for W press
    let metrics_response = app.get("/api/metrics/events?count=100").await;
    let metrics: serde_json::Value = metrics_response.json().await.unwrap();
    let events_w = metrics["events"].as_array().unwrap();

    println!("Events when pressing W:");
    for event in events_w {
        println!("  {:?}", event);
    }

    // Expected: W -> A (base mapping), then A -> Tab (tap-hold)
    // This is CORRECT chaining behavior
    // We should get: Press Tab, Release Tab
    // NOT: Press W, Press A, Press Tab, Release Tab (multi-level contamination)
    let press_count_w = events_w
        .iter()
        .filter(|e| e["event_type"] == "press")
        .count();
    let release_count_w = events_w
        .iter()
        .filter(|e| e["event_type"] == "release")
        .count();

    assert_eq!(
        press_count_w, 1,
        "Expected 1 press event when pressing W->A->Tab, got {}. User reported 'wa' output (W + remapped A) which indicates the ORIGINAL KEY IS LEAKING THROUGH. Events: {:?}",
        press_count_w, events_w
    );

    println!("\n=== DIAGNOSIS ===");
    println!("If press_count_a or press_count_w > 1:");
    println!("  - Layer contamination: Multiple layers are active simultaneously");
    println!("  - Double processing: Input is processed multiple times");
    println!("  - Original key leakage: Hardware key code is not being suppressed");
    println!("\nUser reported: input 'a' -> output 'wa'");
    println!("This suggests the hardware 'W' is NOT being suppressed before remapping to 'A'");
}

#[tokio::test]
#[serial]
async fn test_layer_state_isolation() {
    let app = TestApp::new().await;

    // Create profile with layer activation
    let layer_profile_rhai = r#"
device_start("*");
  tap_hold("VK_A", "VK_Tab", "MD_00", 200);

  layer_start("MD_00");
    map("VK_J", "VK_Left");   // In MD_00 layer: J -> Left Arrow
  layer_end();
device_end();
"#;

    // Write profile file
    let profile_path = app.config_path().join("profiles").join("layer-test.rhai");
    std::fs::write(&profile_path, layer_profile_rhai).unwrap();

    // Activate profile
    let activate_response = app
        .post("/api/profiles/layer-test/activate", &json!({}))
        .await;

    assert_eq!(activate_response.status(), 200);
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Test 1: J without layer active -> should be J
    app.delete("/api/metrics/events").await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let sim_response = app
        .post(
            "/api/simulator/events",
            &json!({
                "dsl": "press:J,wait:50,release:J"
            }),
        )
        .await;
    assert_eq!(sim_response.status(), 200);
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let metrics_response = app.get("/api/metrics/events?count=100").await;
    let metrics: serde_json::Value = metrics_response.json().await.unwrap();
    let events = metrics["events"].as_array().unwrap();

    println!("J without layer:");
    for event in events {
        println!("  {:?}", event);
    }

    // Should output J (no remapping)
    let has_j = events.iter().any(|e| {
        e["output"].as_str().unwrap_or("").contains("J")
            || e["key_code"].as_u64() == Some(keyrx_core::config::KeyCode::J as u64)
    });
    assert!(has_j, "J key not found in output without layer active");

    // Test 2: Hold A (activate MD_00), press J -> should be Left Arrow
    app.delete("/api/metrics/events").await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let sim_response = app
        .post(
            "/api/simulator/events",
            &json!({
                "dsl": "press:A,wait:250,press:J,wait:50,release:J,release:A"
            }),
        )
        .await;
    assert_eq!(sim_response.status(), 200);
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let metrics_response = app.get("/api/metrics/events?count=100").await;
    let metrics: serde_json::Value = metrics_response.json().await.unwrap();
    let events = metrics["events"].as_array().unwrap();

    println!("J with layer MD_00 active:");
    for event in events {
        println!("  {:?}", event);
    }

    // Should output Left Arrow (layer remapping)
    let has_left = events.iter().any(|e| {
        e["output"].as_str().unwrap_or("").contains("Left")
            || e["key_code"].as_u64() == Some(keyrx_core::config::KeyCode::Left as u64)
    });
    assert!(
        has_left,
        "Left Arrow not found when MD_00 layer should be active"
    );

    // Test 3: After releasing A, press J again -> should be J (layer deactivated)
    app.delete("/api/metrics/events").await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let sim_response = app
        .post(
            "/api/simulator/events",
            &json!({
                "dsl": "press:J,wait:50,release:J"
            }),
        )
        .await;
    assert_eq!(sim_response.status(), 200);
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let metrics_response = app.get("/api/metrics/events?count=100").await;
    let metrics: serde_json::Value = metrics_response.json().await.unwrap();
    let events = metrics["events"].as_array().unwrap();

    println!("J after layer deactivation:");
    for event in events {
        println!("  {:?}", event);
    }

    // Should output J again (layer deactivated)
    let has_j_again = events.iter().any(|e| {
        e["output"].as_str().unwrap_or("").contains("J")
            || e["key_code"].as_u64() == Some(keyrx_core::config::KeyCode::J as u64)
    });
    assert!(
        has_j_again,
        "LAYER CONTAMINATION: Layer MD_00 is still active after releasing A"
    );
}
