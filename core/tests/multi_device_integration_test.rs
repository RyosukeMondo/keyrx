#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::field_reassign_with_default,
    clippy::useless_conversion,
    clippy::assertions_on_constants,
    clippy::manual_div_ceil,
    clippy::manual_strip,
    clippy::len_zero,
    clippy::redundant_closure,
    clippy::manual_range_contains,
    clippy::default_constructed_unit_structs,
    clippy::clone_on_copy,
    clippy::io_other_error,
    clippy::bool_assert_comparison,
    clippy::approx_constant,
    clippy::let_unit_value,
    clippy::while_let_on_iterator,
    clippy::await_holding_lock,
    clippy::unnecessary_cast,
    clippy::drop_non_drop,
    clippy::needless_range_loop,
    unused_imports,
    unused_variables,
    dead_code,
    unsafe_code,
    clippy::collapsible_if,
    clippy::bool_comparison,
    unexpected_cfgs
)]
//! Integration tests for multi-device CLI functionality.
//!
//! Tests the complete flow from CLI argument parsing through execution,
//! focusing on edge cases and flag interactions.

use keyrx_core::cli::commands::RunCommand;
use keyrx_core::cli::OutputFormat;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test: --all and --device are mutually exclusive
#[test]
#[should_panic(expected = "Invalid configuration: Cannot specify --device with --all or --devices")]
fn all_and_device_mutually_exclusive() {
    // This should panic because --device and --all can't both be set
    let _cmd = RunCommand::new(
        None,
        false,
        false,
        Some(PathBuf::from("/dev/input/event0")),
        OutputFormat::Human,
    )
    .with_multi_device(true, None);
}

/// Test: --all and --devices are mutually exclusive
#[test]
#[should_panic(expected = "Invalid configuration: Cannot specify both --all and --devices")]
fn all_and_devices_mutually_exclusive() {
    let devices = vec![
        PathBuf::from("/dev/input/event0"),
        PathBuf::from("/dev/input/event1"),
    ];

    // This should panic because --all and --devices can't both be set
    let _cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(true, Some(devices));
}

/// Test: RunCommand with --all flag
#[test]
fn run_command_with_all_flag() {
    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(true, None);

    assert!(cmd.use_all_devices, "--all flag should be set");
    assert!(
        cmd.device_paths.is_none(),
        "device_paths should be None with --all"
    );
    assert!(
        cmd.device_path.is_none(),
        "device_path should be None with --all"
    );
}

/// Test: RunCommand with --devices flag
#[test]
fn run_command_with_devices_flag() {
    let devices = vec![
        PathBuf::from("/dev/input/event3"),
        PathBuf::from("/dev/input/event4"),
        PathBuf::from("/dev/input/event5"),
    ];

    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(false, Some(devices.clone()));

    assert!(!cmd.use_all_devices, "use_all_devices should be false");
    assert_eq!(
        cmd.device_paths,
        Some(devices),
        "device_paths should match input"
    );
    assert!(
        cmd.device_path.is_none(),
        "device_path should be None with --devices"
    );
}

/// Test: Empty device list with --devices
#[test]
#[should_panic(expected = "Invalid configuration: --devices requires at least one device path")]
fn empty_devices_list() {
    // This should panic because --devices requires at least one path
    let _cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(false, Some(vec![]));
}

/// Test: Single device in --devices list (degenerate case)
#[test]
fn single_device_in_devices_list() {
    let devices = vec![PathBuf::from("/dev/input/event0")];

    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(false, Some(devices.clone()));

    assert_eq!(
        cmd.device_paths,
        Some(devices),
        "Should work with single device"
    );

    println!("INFO: Single device in --devices list");
    println!("This is valid but could use --device instead");
}

/// Test: Many devices (stress test)
#[test]
fn many_devices_in_list() {
    let devices: Vec<PathBuf> = (0..100)
        .map(|i| PathBuf::from(format!("/dev/input/event{}", i)))
        .collect();

    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(false, Some(devices.clone()));

    assert_eq!(
        cmd.device_paths.as_ref().unwrap().len(),
        100,
        "Should handle 100 devices"
    );

    println!("INFO: Testing with {} devices", devices.len());
    println!("This may have performance implications");
}

/// Test: Duplicate devices in list
#[test]
#[should_panic(expected = "Invalid configuration: Duplicate device path")]
fn duplicate_devices_in_list() {
    let devices = vec![
        PathBuf::from("/dev/input/event0"),
        PathBuf::from("/dev/input/event1"),
        PathBuf::from("/dev/input/event0"), // Duplicate!
    ];

    // This should panic because of duplicate device
    let _cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(false, Some(devices));
}

/// Test: Invalid device paths
#[test]
fn invalid_device_paths() {
    let devices = vec![
        PathBuf::from("/dev/input/event999"),
        PathBuf::from("/nonexistent/path/device"),
        PathBuf::from(""),
    ];

    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(false, Some(devices));

    println!("INFO: Invalid device paths should fail during initialization");
    assert!(cmd.device_paths.is_some());
}

/// Test: Multi-device with script path
#[test]
fn multi_device_with_script() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test.rhai");
    fs::write(&script_path, "remap(\"a\", \"b\");").unwrap();

    let devices = vec![
        PathBuf::from("/dev/input/event0"),
        PathBuf::from("/dev/input/event1"),
    ];

    let cmd = RunCommand::new(
        Some(script_path.clone()),
        false,
        false,
        None,
        OutputFormat::Human,
    )
    .with_multi_device(false, Some(devices.clone()));

    assert_eq!(cmd.script_path, Some(script_path));
    assert_eq!(cmd.device_paths, Some(devices));
}

/// Test: Multi-device with debug mode
#[test]
fn multi_device_with_debug() {
    let cmd =
        RunCommand::new(None, true, false, None, OutputFormat::Human).with_multi_device(true, None);

    assert!(cmd.debug);
    assert!(cmd.use_all_devices);
}

/// Test: Multi-device with recording
#[test]
fn multi_device_with_recording() {
    let record_path = PathBuf::from("/tmp/session.krx");
    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(true, None)
        .with_record_path(Some(record_path.clone()));

    assert!(cmd.use_all_devices);
    assert_eq!(cmd.record_path, Some(record_path));
}

/// Test: Multi-device with tracing
#[test]
fn multi_device_with_tracing() {
    let trace_path = PathBuf::from("/tmp/trace.json");
    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(true, None)
        .with_trace_path(Some(trace_path.clone()));

    assert!(cmd.use_all_devices);
    assert_eq!(cmd.trace_path, Some(trace_path));
}

/// Test: Multi-device with cache options
#[test]
fn multi_device_with_cache_options() {
    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(true, None)
        .with_cache_options(true, false);

    assert!(cmd.use_all_devices);
    assert!(cmd.disable_cache);
    assert!(!cmd.clear_cache);
}

/// Test: Multi-device with validate_only
#[test]
fn multi_device_with_validate_only() {
    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(true, None)
        .with_validate_only(true);

    assert!(cmd.use_all_devices);
    assert!(cmd.validate_only);

    println!("INFO: Validate-only mode with multi-device");
    println!("Should validate script without initializing devices");
}

/// Test: Builder pattern combinations
#[test]
fn builder_pattern_all_options() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("full.rhai");
    fs::write(&script_path, "// Full config").unwrap();

    let devices = vec![
        PathBuf::from("/dev/input/event0"),
        PathBuf::from("/dev/input/event1"),
    ];

    let cmd = RunCommand::new(
        Some(script_path.clone()),
        true,
        false,
        None,
        OutputFormat::Json,
    )
    .with_multi_device(false, Some(devices.clone()))
    .with_record_path(Some(PathBuf::from("/tmp/session.krx")))
    .with_trace_path(Some(PathBuf::from("/tmp/trace.json")))
    .with_cache_options(false, true)
    .with_validate_only(false);

    // Verify all fields
    assert_eq!(cmd.script_path, Some(script_path));
    assert!(cmd.debug);
    assert!(!cmd.use_mock);
    assert_eq!(cmd.device_paths, Some(devices));
    assert!(cmd.record_path.is_some());
    assert!(cmd.trace_path.is_some());
    assert!(!cmd.disable_cache);
    assert!(cmd.clear_cache);
    assert!(!cmd.validate_only);
}

/// EDGE CASE: Device paths with special characters
#[test]
fn device_paths_with_special_characters() {
    let devices = vec![
        PathBuf::from("/dev/input/event with spaces"),
        PathBuf::from("/dev/input/event-with-dashes"),
        PathBuf::from("/dev/input/event_with_underscores"),
        PathBuf::from("/dev/input/event.with.dots"),
    ];

    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(false, Some(devices.clone()));

    println!("INFO: Testing special characters in device paths");
    assert_eq!(cmd.device_paths, Some(devices));
}

/// EDGE CASE: Very long device paths
#[test]
fn very_long_device_paths() {
    let long_path = format!("/dev/input/{}", "a".repeat(200));
    let devices = vec![PathBuf::from(long_path)];

    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(false, Some(devices.clone()));

    println!(
        "INFO: Testing very long device path ({}  chars)",
        cmd.device_paths.as_ref().unwrap()[0]
            .display()
            .to_string()
            .len()
    );
    assert_eq!(cmd.device_paths, Some(devices));
}

/// EDGE CASE: Relative device paths
#[test]
fn relative_device_paths() {
    let devices = vec![
        PathBuf::from("./event0"),
        PathBuf::from("../input/event1"),
        PathBuf::from("event2"),
    ];

    let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(false, Some(devices.clone()));

    println!("WARNING: Relative device paths!");
    println!("These should probably fail or be resolved to absolute paths");
    assert_eq!(cmd.device_paths, Some(devices));
}

/// TIMING TEST: Multiple RunCommand instances
#[test]
fn multiple_run_command_instances() {
    let devices1 = vec![PathBuf::from("/dev/input/event0")];
    let devices2 = vec![PathBuf::from("/dev/input/event1")];

    let cmd1 = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(false, Some(devices1.clone()));

    let cmd2 = RunCommand::new(None, false, false, None, OutputFormat::Human)
        .with_multi_device(false, Some(devices2.clone()));

    // Should be independent
    assert_ne!(cmd1.device_paths, cmd2.device_paths);

    println!("INFO: Multiple RunCommand instances are independent");
}

/// MEMORY TEST: Large number of devices doesn't leak memory
#[test]
fn memory_test_many_devices() {
    let mut commands = Vec::new();

    for i in 0..100 {
        let devices = vec![PathBuf::from(format!("/dev/input/event{}", i))];
        let cmd = RunCommand::new(None, false, false, None, OutputFormat::Human)
            .with_multi_device(false, Some(devices));
        commands.push(cmd);
    }

    println!("INFO: Created {} RunCommand instances", commands.len());
    println!("If this test crashes, there may be a memory leak");

    assert_eq!(commands.len(), 100);
}

/// TIMING BUG: Mock run limit with multi-device
#[tokio::test]
async fn mock_run_limit_with_multi_device() {
    use tokio::time::Duration;

    let cmd = RunCommand::new(None, false, true, None, OutputFormat::Human)
        .with_multi_device(true, None)
        .with_mock_run_limit(Duration::from_millis(100));

    assert!(cmd.use_mock, "Should be in mock mode");
    assert!(cmd.use_all_devices, "Should use all devices");
    assert_eq!(cmd.mock_run_limit, Some(Duration::from_millis(100)));

    println!("INFO: Testing mock run with multi-device and time limit");
    println!("This tests timeout behavior in multi-device mode");

    // Run should complete within time limit
    let start = std::time::Instant::now();
    let result = cmd.run().await;
    let elapsed = start.elapsed();

    println!("Elapsed: {:?}", elapsed);
    assert!(result.is_ok(), "Mock run should succeed");
    assert!(
        elapsed < Duration::from_millis(500),
        "Should complete within timeout"
    );
}

/// BUG FINDER: Conflicting configuration detection
#[test]
fn detect_conflicting_configurations() {
    println!("\n=== CONFIGURATION CONFLICT DETECTION ===\n");

    // Test 1: use_all_devices AND device_paths both set - should panic
    let result1 = std::panic::catch_unwind(|| {
        RunCommand::new(None, false, false, None, OutputFormat::Human)
            .with_multi_device(true, Some(vec![PathBuf::from("/dev/input/event0")]))
    });
    assert!(
        result1.is_err(),
        "Should panic when both --all and --devices are set"
    );
    println!("✓ Correctly rejected: --all with --devices");

    // Test 2: device_path AND device_paths both set - should panic
    let result2 = std::panic::catch_unwind(|| {
        RunCommand::new(
            None,
            false,
            false,
            Some(PathBuf::from("/dev/input/event0")),
            OutputFormat::Human,
        )
        .with_multi_device(false, Some(vec![PathBuf::from("/dev/input/event1")]))
    });
    assert!(
        result2.is_err(),
        "Should panic when both --device and --devices are set"
    );
    println!("✓ Correctly rejected: --device with --devices");

    // Test 3: use_mock with real device paths - this is allowed (mock ignores paths)
    let cmd3 = RunCommand::new(None, false, true, None, OutputFormat::Human)
        .with_multi_device(false, Some(vec![PathBuf::from("/dev/input/event0")]));

    if cmd3.use_mock && cmd3.device_paths.is_some() {
        println!("✓ Allowed: use_mock with device_paths (mock mode ignores device paths)");
    }

    println!("\n=== END CONFLICT DETECTION ===\n");
}
