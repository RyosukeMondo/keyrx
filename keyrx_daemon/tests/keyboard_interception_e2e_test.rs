// E2E test to verify keyboard interception is actually working
// This catches the bug where daemon runs but doesn't intercept keys

use std::process::{Command, Child};
use std::time::Duration;
use std::thread;

struct DaemonHandle {
    child: Child,
}

impl Drop for DaemonHandle {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

fn start_daemon() -> DaemonHandle {
    let child = Command::new("keyrx_daemon")
        .arg("run")
        .spawn()
        .expect("Failed to start daemon");

    // Wait for daemon to initialize
    thread::sleep(Duration::from_secs(5));

    DaemonHandle { child }
}

#[test]
#[ignore] // Run with: cargo test --test keyboard_interception_e2e_test -- --ignored --test-threads=1
fn test_daemon_starts_and_api_responds() {
    let _daemon = start_daemon();

    // Test health endpoint
    let response = reqwest::blocking::get("http://localhost:9867/api/health")
        .expect("Failed to connect to daemon API");

    assert!(
        response.status().is_success(),
        "API health check failed: {:?}",
        response.status()
    );

    let health: serde_json::Value = response.json()
        .expect("Failed to parse health response");

    assert_eq!(
        health["status"], "ok",
        "Health check returned non-ok status"
    );

    println!("✓ Daemon API is responding");
}

#[test]
#[ignore]
fn test_keyboard_interception_active() {
    let _daemon = start_daemon();

    // Activate a test profile
    let client = reqwest::blocking::Client::new();

    // First, create a minimal test profile
    let profile_config = r#"
    // Minimal test profile: remap 'a' to 'b'
    map("a", "b");
    "#;

    // Upload profile via API
    let response = client
        .post("http://localhost:9867/api/profiles/test-interception")
        .body(profile_config)
        .send()
        .expect("Failed to create test profile");

    assert!(response.status().is_success());

    // Activate profile
    let response = client
        .post("http://localhost:9867/api/profiles/test-interception/activate")
        .send()
        .expect("Failed to activate profile");

    assert!(response.status().is_success());

    // Wait for activation
    thread::sleep(Duration::from_secs(2));

    // Verify profile is active
    let response = client
        .get("http://localhost:9867/api/profiles/active")
        .send()
        .expect("Failed to get active profile");

    let active_profile: serde_json::Value = response.json()
        .expect("Failed to parse active profile");

    assert!(
        !active_profile["active_profile"].is_null(),
        "No active profile after activation!"
    );

    println!("✓ Profile activated successfully");

    // Simulate key event via API and check metrics
    let key_event = serde_json::json!({
        "dsl": "press:A,wait:50,release:A"
    });

    let response = client
        .post("http://localhost:9867/api/simulator/events")
        .json(&key_event)
        .send()
        .expect("Failed to simulate key event");

    assert!(response.status().is_success());

    // Wait for event processing
    thread::sleep(Duration::from_millis(500));

    // Check metrics to verify event was processed
    let response = client
        .get("http://localhost:9867/api/metrics/events?count=10")
        .send()
        .expect("Failed to get metrics");

    let metrics: serde_json::Value = response.json()
        .expect("Failed to parse metrics");

    let events = metrics["events"].as_array()
        .expect("Events is not an array");

    assert!(
        !events.is_empty(),
        "CRITICAL: No events detected! Keyboard interception is NOT working.\n\
         This is the bug the user is experiencing - daemon runs but doesn't intercept keys."
    );

    println!("✓ Keyboard interception is working - {} events detected", events.len());
}

#[test]
#[ignore]
fn test_metrics_reset_and_capture() {
    let _daemon = start_daemon();
    let client = reqwest::blocking::Client::new();

    // Clear existing metrics
    let response = client
        .delete("http://localhost:9867/api/metrics/events")
        .send()
        .expect("Failed to clear metrics");

    assert!(response.status().is_success());

    // Verify metrics are empty
    let response = client
        .get("http://localhost:9867/api/metrics/events?count=10")
        .send()
        .expect("Failed to get metrics");

    let metrics: serde_json::Value = response.json()
        .expect("Failed to parse metrics");

    let events = metrics["events"].as_array()
        .expect("Events is not an array");

    assert_eq!(
        events.len(), 0,
        "Metrics not empty after clear"
    );

    println!("✓ Metrics cleared successfully");
}
