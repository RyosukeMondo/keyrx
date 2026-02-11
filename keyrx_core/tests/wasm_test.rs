//! Comprehensive WASM module tests using wasm-bindgen-test.
//!
//! These tests run in a browser environment (headless or visible) via wasm-pack.
//!
//! # Running Tests
//!
//! ```bash
//! # Firefox (headless)
//! wasm-pack test --headless --firefox
//!
//! # Chrome (headless)
//! wasm-pack test --headless --chrome
//!
//! # With visible browser for debugging
//! wasm-pack test --firefox
//! ```
//!
//! # Prerequisites
//!
//! Install wasm-pack:
//! ```bash
//! cargo install wasm-pack
//! ```
//!
//! # Test Coverage
//!
//! Tests cover:
//! - Configuration loading (Rhai and .krx)
//! - Event simulation
//! - State querying
//! - Error handling
//! - Performance requirements
//!
//! Note: Some tests that require internal ConfigHandle construction
//! are covered by unit tests in src/wasm/mod.rs instead.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

use keyrx_core::wasm::{
    get_state, load_config, load_krx, simulate, wasm_init, EventSequence, SimKeyEvent,
};

// Configure wasm-bindgen-test to run in browser
wasm_bindgen_test_configure!(run_in_browser);

// ============================================================================
// Module Initialization Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_wasm_init() {
    // Should not panic
    wasm_init();
    // Call again to test idempotency
    wasm_init();
}

// ============================================================================
// Configuration Loading Tests - load_config (Rhai)
// ============================================================================

#[wasm_bindgen_test]
fn test_load_config_simple_mapping() {
    wasm_init();

    let rhai_source = r#"
        device("*") {
            map("A", "B");
        }
    "#;

    let result = load_config(rhai_source);
    assert!(
        result.is_ok(),
        "Simple Rhai config should load successfully"
    );
}

#[wasm_bindgen_test]
fn test_load_config_empty() {
    wasm_init();

    let rhai_source = "";
    let result = load_config(rhai_source);

    // Empty config should be valid (no devices is allowed)
    assert!(result.is_ok(), "Empty config should be valid");
}

#[wasm_bindgen_test]
fn test_load_config_invalid_syntax() {
    wasm_init();

    let rhai_source = r#"
        device("*" {
            map("A", "B")
        }
    "#; // Missing closing parenthesis

    let result = load_config(rhai_source);
    assert!(result.is_err(), "Invalid Rhai syntax should fail");

    // Error message should contain useful information
    if let Err(e) = result {
        let error_str = format!("{:?}", e);
        assert!(
            error_str.contains("error") || error_str.contains("Parse"),
            "Error message should indicate parse failure"
        );
    }
}

#[wasm_bindgen_test]
fn test_load_config_too_large() {
    wasm_init();

    // Create a config larger than 1MB
    let mut large_config = "device(\"*\") { ".to_string();
    large_config.push_str(&"map(\"A\", \"B\"); ".repeat(50000));
    large_config.push_str(" }");

    let result = load_config(&large_config);
    assert!(result.is_err(), "Config larger than 1MB should fail");

    if let Err(e) = result {
        let error_str = format!("{:?}", e);
        assert!(
            error_str.contains("too large") || error_str.contains("max"),
            "Error should mention size limit"
        );
    }
}

#[wasm_bindgen_test]
fn test_load_config_unsupported_key() {
    wasm_init();

    let rhai_source = r#"
        device("*") {
            map("INVALID_KEY", "B");
        }
    "#;

    let result = load_config(rhai_source);
    assert!(result.is_err(), "Unsupported key name should fail to parse");

    if let Err(e) = result {
        let error_str = format!("{:?}", e);
        assert!(
            error_str.contains("Unsupported") || error_str.contains("key"),
            "Error should mention unsupported key"
        );
    }
}

#[wasm_bindgen_test]
fn test_load_config_multiple_devices() {
    wasm_init();

    let rhai_source = r#"
        device("*") {
            map("A", "B");
        }
    "#;

    // Load multiple configs to test CONFIG_STORE management
    let handle1 = load_config(rhai_source).expect("First config should load");
    let handle2 = load_config(rhai_source).expect("Second config should load");

    // Handles should be different
    assert_ne!(
        format!("{:?}", handle1),
        format!("{:?}", handle2),
        "Different configs should have different handles"
    );
}

// ============================================================================
// Configuration Loading Tests - load_krx (Binary)
// ============================================================================

#[wasm_bindgen_test]
fn test_load_krx_valid() {
    use keyrx_core::config::{
        ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping, Metadata, Version,
    };

    wasm_init();

    // Create a valid ConfigRoot
    let config = ConfigRoot {
        version: Version::current(),
        devices: vec![DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*".into(),
            },
            mappings: vec![KeyMapping::simple(KeyCode::A, KeyCode::B)],
        }],
        metadata: Metadata {
            compilation_timestamp: 1234567890,
            compiler_version: "wasm-test-0.1.0".into(),
            source_hash: "test_hash".into(),
        },
    };

    // Serialize to .krx format
    let bytes = rkyv::to_bytes::<_, 1024>(&config).expect("Serialization should succeed");

    // Load the binary
    let result = load_krx(&bytes);
    assert!(result.is_ok(), "Valid .krx binary should load successfully");
}

#[wasm_bindgen_test]
fn test_load_krx_too_large() {
    wasm_init();

    // Create a binary larger than 10MB
    let large_binary = vec![0u8; 11 * 1024 * 1024];

    let result = load_krx(&large_binary);
    assert!(result.is_err(), "Binary larger than 10MB should fail");

    if let Err(e) = result {
        let error_str = format!("{:?}", e);
        assert!(
            error_str.contains("too large") || error_str.contains("max"),
            "Error should mention size limit"
        );
    }
}

#[wasm_bindgen_test]
fn test_load_krx_too_small() {
    wasm_init();

    let small_binary = vec![0u8; 4];

    let result = load_krx(&small_binary);
    assert!(result.is_err(), "Binary smaller than 8 bytes should fail");

    if let Err(e) = result {
        let error_str = format!("{:?}", e);
        assert!(
            error_str.contains("too small") || error_str.contains("valid"),
            "Error should mention invalid size"
        );
    }
}

#[wasm_bindgen_test]
fn test_load_krx_invalid_data() {
    wasm_init();

    // Create invalid binary data
    let invalid_binary = vec![0xFF; 256];

    let result = load_krx(&invalid_binary);
    assert!(result.is_err(), "Invalid binary data should fail");

    if let Err(e) = result {
        let error_str = format!("{:?}", e);
        assert!(
            error_str.contains("version") || error_str.contains("corrupted"),
            "Error should mention corruption or version"
        );
    }
}

// ============================================================================
// Event Simulation Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_simulate_simple_sequence() {
    wasm_init();

    // Load a simple config
    let rhai_source = r#"
        device("*") {
            map("A", "B");
        }
    "#;

    let config_handle = load_config(rhai_source).expect("Config should load");

    // Create a simple event sequence (press A, release A)
    let events = EventSequence {
        events: vec![
            SimKeyEvent {
                keycode: "A".to_string(),
                event_type: "press".to_string(),
                timestamp_us: 0,
            },
            SimKeyEvent {
                keycode: "A".to_string(),
                event_type: "release".to_string(),
                timestamp_us: 100_000,
            },
        ],
    };

    let events_json = serde_json::to_string(&events).expect("Serialization should succeed");

    let result = simulate(config_handle, &events_json);
    assert!(result.is_ok(), "Simple simulation should succeed");
}

// Note: test_simulate_invalid_handle is covered by unit tests in mod.rs
// because ConfigHandle is opaque and cannot be constructed in external tests

#[wasm_bindgen_test]
fn test_simulate_invalid_json() {
    wasm_init();

    let rhai_source = r#"
        device("*") {
            map("A", "B");
        }
    "#;

    let config_handle = load_config(rhai_source).expect("Config should load");

    let invalid_json = "{ invalid json syntax }";

    let result = simulate(config_handle, invalid_json);
    assert!(result.is_err(), "Invalid JSON should fail");

    if let Err(e) = result {
        let error_str = format!("{:?}", e);
        assert!(
            error_str.contains("Invalid") || error_str.contains("JSON"),
            "Error should mention JSON parsing failure"
        );
    }
}

#[wasm_bindgen_test]
fn test_simulate_too_many_events() {
    wasm_init();

    let rhai_source = r#"
        device("*") {
            map("A", "B");
        }
    "#;

    let config_handle = load_config(rhai_source).expect("Config should load");

    // Create more than 1000 events
    let events = EventSequence {
        events: (0..1001)
            .map(|i| SimKeyEvent {
                keycode: "A".to_string(),
                event_type: if i % 2 == 0 { "press" } else { "release" }.to_string(),
                timestamp_us: i as u64 * 1000,
            })
            .collect(),
    };

    let events_json = serde_json::to_string(&events).expect("Serialization should succeed");

    let result = simulate(config_handle, &events_json);
    assert!(
        result.is_err(),
        "More than 1000 events should fail with error"
    );

    if let Err(e) = result {
        let error_str = format!("{:?}", e);
        assert!(
            error_str.contains("Too many") || error_str.contains("max"),
            "Error should mention event limit"
        );
    }
}

#[wasm_bindgen_test]
fn test_simulate_empty_sequence() {
    wasm_init();

    let rhai_source = r#"
        device("*") {
            map("A", "B");
        }
    "#;

    let config_handle = load_config(rhai_source).expect("Config should load");

    let events = EventSequence { events: vec![] };

    let events_json = serde_json::to_string(&events).expect("Serialization should succeed");

    let result = simulate(config_handle, &events_json);
    // Empty sequence should be valid
    assert!(result.is_ok(), "Empty event sequence should be valid");
}

// ============================================================================
// State Query Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_get_state_before_simulation() {
    wasm_init();

    let rhai_source = r#"
        device("*") {
            map("A", "B");
        }
    "#;

    let config_handle = load_config(rhai_source).expect("Config should load");

    // Try to get state before running any simulation
    let result = get_state(config_handle);
    assert!(
        result.is_err(),
        "Getting state before simulation should fail"
    );

    if let Err(e) = result {
        let error_str = format!("{:?}", e);
        assert!(
            error_str.contains("simulation") || error_str.contains("state"),
            "Error should mention missing simulation"
        );
    }
}

#[wasm_bindgen_test]
fn test_get_state_after_simulation() {
    wasm_init();

    let rhai_source = r#"
        device("*") {
            map("A", "B");
        }
    "#;

    let config_handle = load_config(rhai_source).expect("Config should load");

    // Run a simulation first
    let events = EventSequence {
        events: vec![
            SimKeyEvent {
                keycode: "A".to_string(),
                event_type: "press".to_string(),
                timestamp_us: 0,
            },
            SimKeyEvent {
                keycode: "A".to_string(),
                event_type: "release".to_string(),
                timestamp_us: 100_000,
            },
        ],
    };

    let events_json = serde_json::to_string(&events).expect("Serialization should succeed");
    simulate(config_handle, &events_json).expect("Simulation should succeed");

    // Now get state
    let result = get_state(config_handle);
    assert!(result.is_ok(), "Getting state after simulation should work");
}

// Note: test_get_state_invalid_handle is covered by unit tests in mod.rs
// because ConfigHandle is opaque and cannot be constructed in external tests

// ============================================================================
// Performance Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_simulation_performance_100_events() {
    use web_sys::window;

    wasm_init();

    let rhai_source = r#"
        device("*") {
            map("A", "B");
        }
    "#;

    let config_handle = load_config(rhai_source).expect("Config should load");

    // Create 100-event sequence
    let events = EventSequence {
        events: (0..100)
            .map(|i| SimKeyEvent {
                keycode: "A".to_string(),
                event_type: if i % 2 == 0 { "press" } else { "release" }.to_string(),
                timestamp_us: i as u64 * 1000,
            })
            .collect(),
    };

    let events_json = serde_json::to_string(&events).expect("Serialization should succeed");

    // Measure execution time
    let start = window().unwrap().performance().unwrap().now();
    let result = simulate(config_handle, &events_json);
    let end = window().unwrap().performance().unwrap().now();

    assert!(result.is_ok(), "100-event simulation should succeed");

    let duration_ms = end - start;
    // Should complete in reasonable time (< 100ms for 100 events)
    assert!(
        duration_ms < 100.0,
        "100-event simulation should complete in <100ms, took: {}ms",
        duration_ms
    );
}

// ============================================================================
// Integration Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_full_workflow() {
    wasm_init();

    // 1. Load config
    let rhai_source = r#"
        device("*") {
            map("A", "B");
        }
    "#;

    let config_handle = load_config(rhai_source).expect("Config should load");

    // 2. Run simulation
    let events = EventSequence {
        events: vec![
            SimKeyEvent {
                keycode: "A".to_string(),
                event_type: "press".to_string(),
                timestamp_us: 0,
            },
            SimKeyEvent {
                keycode: "A".to_string(),
                event_type: "release".to_string(),
                timestamp_us: 100_000,
            },
        ],
    };

    let events_json = serde_json::to_string(&events).expect("Serialization should succeed");
    let sim_result = simulate(config_handle, &events_json).expect("Simulation should succeed");

    // Verify simulation result is valid JSON
    assert!(
        !sim_result.is_undefined() && !sim_result.is_null(),
        "Simulation result should be valid"
    );

    // 3. Get state
    let state_result = get_state(config_handle);
    assert!(state_result.is_ok(), "Getting state should work");

    // Verify state is valid JSON
    if let Ok(state) = state_result {
        assert!(
            !state.is_undefined() && !state.is_null(),
            "State should be valid"
        );
    }
}
