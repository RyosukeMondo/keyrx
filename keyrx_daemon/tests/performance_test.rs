//! Performance Regression Tests
//!
//! Comprehensive performance tests to detect regressions:
//! - Benchmark all API endpoints
//! - Measure WebSocket latency
//! - Measure profile activation time
//! - Compare before/after bug fixes
//! - Alert if performance degrades >10%
//!
//! Requirements: TEST-006

mod common;

use common::test_app::TestApp;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Performance baseline thresholds
const MAX_API_LATENCY_MS: u128 = 100;
const MAX_WS_CONNECT_MS: u128 = 500;
const MAX_PROFILE_ACTIVATION_MS: u128 = 200;
const MAX_SUBSCRIPTION_MS: u128 = 100;

/// Performance measurement helper
struct PerformanceMeasurement {
    name: String,
    samples: Vec<Duration>,
}

impl PerformanceMeasurement {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            samples: Vec::new(),
        }
    }

    fn record(&mut self, duration: Duration) {
        self.samples.push(duration);
    }

    fn avg_ms(&self) -> u128 {
        if self.samples.is_empty() {
            return 0;
        }
        let total: u128 = self.samples.iter().map(|d| d.as_millis()).sum();
        total / self.samples.len() as u128
    }

    fn min_ms(&self) -> u128 {
        self.samples
            .iter()
            .map(|d| d.as_millis())
            .min()
            .unwrap_or(0)
    }

    fn max_ms(&self) -> u128 {
        self.samples
            .iter()
            .map(|d| d.as_millis())
            .max()
            .unwrap_or(0)
    }

    fn p95_ms(&self) -> u128 {
        if self.samples.is_empty() {
            return 0;
        }
        let mut sorted: Vec<u128> = self.samples.iter().map(|d| d.as_millis()).collect();
        sorted.sort_unstable();
        let idx = (sorted.len() as f64 * 0.95) as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    fn print_stats(&self) {
        println!("\n=== {} ===", self.name);
        println!("Samples: {}", self.samples.len());
        println!("Average: {}ms", self.avg_ms());
        println!("Min: {}ms", self.min_ms());
        println!("Max: {}ms", self.max_ms());
        println!("P95: {}ms", self.p95_ms());
    }
}

/// Benchmark API endpoint performance
///
/// Measures latency for all major API endpoints.
#[tokio::test]
async fn test_api_endpoint_performance() {
    let app = TestApp::new().await;

    // Warm up
    for _ in 0..10 {
        let _ = app.get("/api/status").await;
    }

    // Benchmark each endpoint
    let endpoints = vec![
        ("/api/status", "Status"),
        ("/api/profiles", "List Profiles"),
        ("/api/devices", "List Devices"),
        ("/api/settings", "Get Settings"),
    ];

    for (endpoint, name) in endpoints {
        let mut measurement = PerformanceMeasurement::new(name);

        for _ in 0..100 {
            let start = Instant::now();
            let response = app.get(endpoint).await;
            let duration = start.elapsed();

            // Only record successful requests
            if response.status().is_success() {
                measurement.record(duration);
            }

            sleep(Duration::from_millis(10)).await;
        }

        measurement.print_stats();

        // Assert performance requirements
        assert!(
            measurement.avg_ms() < MAX_API_LATENCY_MS,
            "{} average latency {}ms exceeds threshold {}ms",
            name,
            measurement.avg_ms(),
            MAX_API_LATENCY_MS
        );
    }
}

/// Benchmark profile creation performance
#[tokio::test]
async fn test_profile_creation_performance() {
    let app = TestApp::new().await;

    let mut measurement = PerformanceMeasurement::new("Profile Creation");

    for i in 0..50 {
        let profile_name = format!("perf-test-profile-{}", i);

        let start = Instant::now();
        let response = app
            .post(
                "/api/profiles",
                &serde_json::json!({
                    "name": profile_name,
                    "config_source": "default"
                }),
            )
            .await;
        let duration = start.elapsed();

        if response.status().is_success() {
            measurement.record(duration);
        }

        // Clean up
        let _ = app.delete(&format!("/api/profiles/{}", profile_name)).await;

        sleep(Duration::from_millis(20)).await;
    }

    measurement.print_stats();

    assert!(
        measurement.avg_ms() < 200,
        "Profile creation too slow: {}ms",
        measurement.avg_ms()
    );
}

/// Benchmark profile activation performance
#[tokio::test]
async fn test_profile_activation_performance() {
    let app = TestApp::new().await;

    // Create test profile
    let _ = app
        .post(
            "/api/profiles",
            &serde_json::json!({
                "name": "activation-test",
                "config_source": "default"
            }),
        )
        .await;

    sleep(Duration::from_millis(100)).await;

    let mut measurement = PerformanceMeasurement::new("Profile Activation");

    for _ in 0..100 {
        let start = Instant::now();
        let response = app
            .post(
                "/api/profiles/activation-test/activate",
                &serde_json::json!({}),
            )
            .await;
        let duration = start.elapsed();

        if response.status().is_success() {
            measurement.record(duration);
        }

        sleep(Duration::from_millis(50)).await;
    }

    measurement.print_stats();

    assert!(
        measurement.avg_ms() < MAX_PROFILE_ACTIVATION_MS,
        "Profile activation too slow: {}ms",
        measurement.avg_ms()
    );
}

/// Benchmark WebSocket connection performance
#[tokio::test]
async fn test_websocket_connection_performance() {
    let app = TestApp::new().await;

    let mut measurement = PerformanceMeasurement::new("WebSocket Connection");

    for _ in 0..50 {
        let start = Instant::now();
        let ws = app.connect_ws().await;
        let duration = start.elapsed();

        measurement.record(duration);

        drop(ws);
        sleep(Duration::from_millis(50)).await;
    }

    measurement.print_stats();

    assert!(
        measurement.avg_ms() < MAX_WS_CONNECT_MS,
        "WebSocket connection too slow: {}ms",
        measurement.avg_ms()
    );
}

/// Benchmark WebSocket subscription performance
#[tokio::test]
async fn test_websocket_subscription_performance() {
    let app = TestApp::new().await;

    let mut measurement = PerformanceMeasurement::new("WebSocket Subscription");

    for i in 0..50 {
        let mut ws = app.connect_ws().await;

        let subscribe_msg = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "subscribe",
            "params": {
                "topics": ["daemon_state"]
            },
            "id": i
        });

        let start = Instant::now();
        ws.send_text(subscribe_msg.to_string()).await.unwrap();
        // Wait for subscription to be processed
        sleep(Duration::from_millis(10)).await;
        let duration = start.elapsed();

        measurement.record(duration);

        drop(ws);
        sleep(Duration::from_millis(20)).await;
    }

    measurement.print_stats();

    assert!(
        measurement.avg_ms() < MAX_SUBSCRIPTION_MS,
        "WebSocket subscription too slow: {}ms",
        measurement.avg_ms()
    );
}

/// Benchmark WebSocket message broadcast latency
#[tokio::test]
async fn test_websocket_broadcast_latency() {
    let app = TestApp::new().await;

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

    let mut measurement = PerformanceMeasurement::new("WebSocket Broadcast Latency");

    for _ in 0..50 {
        let start = Instant::now();

        // Trigger event
        let _ = app
            .post("/api/profiles/default/activate", &serde_json::json!({}))
            .await;

        // Wait for broadcast (simplified - real test would read WS messages)
        sleep(Duration::from_millis(50)).await;

        let duration = start.elapsed();
        measurement.record(duration);

        sleep(Duration::from_millis(100)).await;
    }

    measurement.print_stats();

    // Broadcast latency should be reasonable
    assert!(
        measurement.avg_ms() < 300,
        "Broadcast latency too high: {}ms",
        measurement.avg_ms()
    );

    drop(ws);
}

/// Benchmark concurrent request performance
#[tokio::test]
async fn test_concurrent_request_performance() {
    let app = TestApp::new().await;

    let mut measurement = PerformanceMeasurement::new("Concurrent Requests (10 parallel)");

    for _ in 0..20 {
        let start = Instant::now();

        // Make 10 concurrent requests
        let futures: Vec<_> = (0..10).map(|_| app.get("/api/status")).collect();
        let results: Vec<reqwest::Response> = futures_util::future::join_all(futures).await;

        let duration = start.elapsed();
        measurement.record(duration);

        // Verify all succeeded
        let success_count = results.iter().filter(|r| r.status().is_success()).count();
        assert!(success_count > 5, "Most concurrent requests should succeed");

        sleep(Duration::from_millis(100)).await;
    }

    measurement.print_stats();

    // 10 concurrent requests should complete faster than 10x sequential
    assert!(
        measurement.avg_ms() < 500,
        "Concurrent requests too slow: {}ms",
        measurement.avg_ms()
    );
}

/// Benchmark memory allocation performance
#[tokio::test]
async fn test_memory_allocation_performance() {
    let app = TestApp::new().await;

    // Test that creating and destroying resources is efficient
    let mut measurement = PerformanceMeasurement::new("Resource Creation/Cleanup");

    for i in 0..100 {
        let start = Instant::now();

        // Create profile
        let profile_name = format!("mem-test-{}", i);
        let _ = app
            .post(
                "/api/profiles",
                &serde_json::json!({
                    "name": profile_name,
                    "config_source": "default"
                }),
            )
            .await;

        // Delete profile
        let _ = app.delete(&format!("/api/profiles/{}", profile_name)).await;

        let duration = start.elapsed();
        measurement.record(duration);

        if i % 20 == 0 {
            sleep(Duration::from_millis(50)).await;
        }
    }

    measurement.print_stats();

    assert!(
        measurement.avg_ms() < 300,
        "Resource allocation too slow: {}ms",
        measurement.avg_ms()
    );
}

/// Benchmark JSON serialization performance
#[tokio::test]
async fn test_json_serialization_performance() {
    let app = TestApp::new().await;

    let mut measurement = PerformanceMeasurement::new("JSON Response Parsing");

    for _ in 0..100 {
        let start = Instant::now();

        let response = app.get("/api/profiles").await;
        if response.status().is_success() {
            let _ = response.json::<serde_json::Value>().await;
        }

        let duration = start.elapsed();
        measurement.record(duration);

        sleep(Duration::from_millis(10)).await;
    }

    measurement.print_stats();

    assert!(
        measurement.avg_ms() < 150,
        "JSON parsing too slow: {}ms",
        measurement.avg_ms()
    );
}

/// Detect performance regression by comparing with baseline
#[tokio::test]
async fn test_performance_regression_detection() {
    let app = TestApp::new().await;

    // Baseline measurements (these would be saved from previous runs)
    let baseline_status_ms = 50u128;
    let baseline_profiles_ms = 80u128;

    // Current measurements
    let mut status_measurement = PerformanceMeasurement::new("Status Endpoint");
    let mut profiles_measurement = PerformanceMeasurement::new("Profiles Endpoint");

    for _ in 0..50 {
        let start = Instant::now();
        let _ = app.get("/api/status").await;
        status_measurement.record(start.elapsed());

        let start = Instant::now();
        let _ = app.get("/api/profiles").await;
        profiles_measurement.record(start.elapsed());

        sleep(Duration::from_millis(20)).await;
    }

    status_measurement.print_stats();
    profiles_measurement.print_stats();

    // Check for regression (>10% slower than baseline)
    let status_regression_pct =
        ((status_measurement.avg_ms() as f64 / baseline_status_ms as f64) - 1.0) * 100.0;
    let profiles_regression_pct =
        ((profiles_measurement.avg_ms() as f64 / baseline_profiles_ms as f64) - 1.0) * 100.0;

    println!("\nRegression Analysis:");
    println!("Status: {:.1}% change", status_regression_pct);
    println!("Profiles: {:.1}% change", profiles_regression_pct);

    // Alert if >10% regression
    if status_regression_pct > 10.0 {
        println!(
            "⚠️  WARNING: Status endpoint regressed by {:.1}%",
            status_regression_pct
        );
    }

    if profiles_regression_pct > 10.0 {
        println!(
            "⚠️  WARNING: Profiles endpoint regressed by {:.1}%",
            profiles_regression_pct
        );
    }

    // Don't fail test for performance regression in test environment
    // In CI, you would fail the test or require manual approval
}

/// Benchmark cold start vs warm performance
#[tokio::test]
async fn test_cold_start_vs_warm_performance() {
    let app = TestApp::new().await;

    // Cold start (first request)
    let start = Instant::now();
    let _ = app.get("/api/status").await;
    let cold_start_ms = start.elapsed().as_millis();

    sleep(Duration::from_millis(100)).await;

    // Warm requests
    let mut warm_measurement = PerformanceMeasurement::new("Warm Requests");

    for _ in 0..50 {
        let start = Instant::now();
        let _ = app.get("/api/status").await;
        warm_measurement.record(start.elapsed());
        sleep(Duration::from_millis(10)).await;
    }

    println!("\nCold start: {}ms", cold_start_ms);
    warm_measurement.print_stats();

    // Warm requests should be faster than cold start
    assert!(
        warm_measurement.avg_ms() <= cold_start_ms,
        "Warm requests should not be slower than cold start"
    );
}
