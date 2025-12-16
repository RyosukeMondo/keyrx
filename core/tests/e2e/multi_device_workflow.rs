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
//! End-to-end tests for multi-device workflows.
//!
//! These tests simulate real-world multi-device scenarios:
//! - Cross-keyboard combinations (Shift on KB1 + char on KB2)
//! - Event ordering across devices
//! - Device disconnection/reconnection
//! - Simultaneous input from multiple devices

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// E2E Test: Cross-keyboard modifier combinations
///
/// CRITICAL BUG TO FIND: Do modifiers work across keyboards?
/// - Press Shift on keyboard 1
/// - Press 'a' on keyboard 2
/// - Should produce uppercase 'A'
#[tokio::test]
async fn cross_keyboard_shift_combination() {
    println!("\n=== CROSS-KEYBOARD SHIFT TEST ===");
    println!("Scenario: Shift on KB1, 'a' on KB2 -> should produce 'A'");

    // This test requires mock multi-device input
    // For now, we document the expected behavior

    println!("\nExpected behavior:");
    println!("1. KB1: Shift key DOWN (modifier state updated)");
    println!("2. KB2: 'a' key DOWN (should see Shift is active)");
    println!("3. Output: 'A' (uppercase)");
    println!("4. KB2: 'a' key UP");
    println!("5. KB1: Shift key UP");

    println!("\nCRITICAL BUG RISKS:");
    println!("- Modifier state not shared between devices");
    println!("- Events processed in wrong order due to timestamp issues");
    println!("- Modifier released before character processed");

    // TODO: Implement with mock multi-device input
    println!("\n[TEST INCOMPLETE - NEEDS MOCK IMPLEMENTATION]");
}

/// E2E Test: Cross-keyboard Ctrl+C combination
///
/// CRITICAL BUG: Control sequences across keyboards
#[tokio::test]
async fn cross_keyboard_ctrl_c() {
    println!("\n=== CROSS-KEYBOARD CTRL+C TEST ===");
    println!("Scenario: Ctrl on KB1, 'c' on KB2 -> should produce Ctrl+C");

    println!("\nExpected behavior:");
    println!("1. KB1: Ctrl DOWN");
    println!("2. KB2: 'c' DOWN -> sees Ctrl is active");
    println!("3. Output: Ctrl+C signal");
    println!("4. KB2: 'c' UP");
    println!("5. KB1: Ctrl UP");

    println!("\nBUG RISKS:");
    println!("- Ctrl state not visible to KB2");
    println!("- Timing issue: 'c' processed before Ctrl");
    println!("- Ctrl+C might not trigger if events out of order");

    println!("\n[TEST INCOMPLETE - NEEDS MOCK IMPLEMENTATION]");
}

/// E2E Test: Triple keyboard combination (Ctrl+Shift+Key)
///
/// SEVERE BUG: Complex combinations across 3 devices
#[tokio::test]
async fn triple_keyboard_combination() {
    println!("\n=== TRIPLE KEYBOARD COMBINATION TEST ===");
    println!("Scenario: Ctrl on KB1, Shift on KB2, 'A' on KB3");

    println!("\nExpected behavior:");
    println!("1. KB1: Ctrl DOWN");
    println!("2. KB2: Shift DOWN");
    println!("3. KB3: 'A' DOWN -> sees BOTH Ctrl and Shift active");
    println!("4. Output: Ctrl+Shift+A");
    println!("5. KB3: 'A' UP");
    println!("6. KB2: Shift UP");
    println!("7. KB1: Ctrl UP");

    println!("\nBUG RISKS:");
    println!("- Only one modifier state tracked");
    println!("- Modifier precedence issues");
    println!("- Event ordering with 3 timestamp sources");

    println!("\n[TEST INCOMPLETE - NEEDS MOCK IMPLEMENTATION]");
}

/// E2E Test: Simultaneous key press on different keyboards
///
/// TIMING BUG: Events with identical timestamps from different devices
#[tokio::test]
async fn simultaneous_keypresses() {
    println!("\n=== SIMULTANEOUS KEYPRESS TEST ===");
    println!("Scenario: Both KB1 and KB2 press 'a' at exactly the same time");

    println!("\nExpected behavior:");
    println!("1. KB1 and KB2: Both press 'a' at timestamp T");
    println!("2. Events merged by timestamp");
    println!("3. Output: Two 'a' keypresses (order may vary)");

    println!("\nCRITICAL BUG RISKS:");
    println!("- BinaryHeap is NOT stable - identical timestamps may reorder");
    println!("- Events might be lost if not handled properly");
    println!("- Race condition in event buffer");

    println!("\nThis is a KNOWN ISSUE in Rust's BinaryHeap:");
    println!("  Equal elements may be returned in any order");
    println!("  Solution: Include sequence number in ordering");

    println!("\n[TEST INCOMPLETE - EXPOSES HEAP INSTABILITY]");
}

/// E2E Test: Rapid typing across two keyboards
///
/// TIMING BUG: High-frequency events with minimal buffer delay
#[tokio::test]
async fn rapid_alternating_typing() {
    println!("\n=== RAPID ALTERNATING TYPING TEST ===");
    println!("Scenario: Fast typing alternating between KB1 and KB2");

    println!("\nScenario:");
    println!("KB1: 'h' (t=0ms)");
    println!("KB2: 'e' (t=1ms)");
    println!("KB1: 'l' (t=2ms)");
    println!("KB2: 'l' (t=3ms)");
    println!("KB1: 'o' (t=4ms)");
    println!("Expected output: 'hello'");

    println!("\nBUG RISKS:");
    println!("- Event ordering violated (should be h-e-l-l-o)");
    println!("- Buffer delay (10ms) is much longer than event interval (1ms)");
    println!("- All events buffered until 10ms passes");
    println!("- User experiences input lag");

    println!("\nTIMING ANALYSIS:");
    println!("  Buffer delay: 10ms");
    println!("  Event interval: 1ms");
    println!("  Result: ~10ms lag before any output");

    println!("\n[TEST INCOMPLETE - REVEALS BUFFER DELAY ISSUE]");
}

/// E2E Test: Device with slow clock
///
/// SEVERE TIMING BUG: One device clock is behind
#[tokio::test]
async fn device_with_slow_clock() {
    println!("\n=== SLOW CLOCK DEVICE TEST ===");
    println!("Scenario: KB1 clock is 5ms behind KB2 clock");

    println!("\nScenario:");
    println!("Real time: t=0ms");
    println!("  KB1 presses 'a' (KB1 timestamp: t=0ms)");
    println!("Real time: t=1ms");
    println!("  KB2 presses 'b' (KB2 timestamp: t=6ms due to clock skew)");

    println!("\nExpected: 'ab' (chronological order)");
    println!("Actual with naive sorting: 'ba' (timestamp order)");

    println!("\nCRITICAL BUG:");
    println!("- Events ordered by timestamp, not real time");
    println!("- KB2's 'b' has later timestamp than KB1's 'a'");
    println!("- Output is 'ba' instead of 'ab'!");

    println!("\nSOLUTION:");
    println!("- Timestamp synchronization between devices");
    println!("- Calibration phase to measure clock offsets");
    println!("- Or accept that timestamps may be slightly wrong");

    println!("\n[TEST INCOMPLETE - KNOWN CLOCK SKEW ISSUE]");
}

/// E2E Test: Device with fast clock
///
/// SEVERE TIMING BUG: One device clock is ahead
#[tokio::test]
async fn device_with_fast_clock() {
    println!("\n=== FAST CLOCK DEVICE TEST ===");
    println!("Scenario: KB1 clock is 20ms ahead of real time");

    println!("\nScenario:");
    println!("Real time: t=0ms");
    println!("  KB1 presses 'a' (KB1 timestamp: t=20ms)");
    println!("Real time: t=5ms");
    println!("  KB2 presses 'b' (KB2 timestamp: t=5ms)");

    println!("\nExpected: 'ab' (chronological order)");
    println!("Actual with 10ms buffer: Events stuck!");

    println!("\nCRITICAL BUG:");
    println!("- KB1's event timestamp (20ms) is in the future");
    println!("- Buffer delay is 10ms");
    println!("- KB1's event NEVER released (always too recent)");
    println!("- 'a' is stuck in buffer forever!");

    println!("\nBUFFER ANALYSIS:");
    println!("  At t=10ms: cutoff = 0ms");
    println!("    KB2's 'b' (t=5ms) > cutoff: HELD");
    println!("  At t=15ms: cutoff = 5ms");
    println!("    KB2's 'b' (t=5ms) = cutoff: RELEASED");
    println!("    KB1's 'a' (t=20ms) > cutoff: HELD");
    println!("  At t=30ms: cutoff = 20ms");
    println!("    KB1's 'a' (t=20ms) = cutoff: RELEASED");

    println!("\nRESULT: 'b' appears, then 25ms later 'a' appears");
    println!("User sees: 'b' ... (long pause) ... 'a'");

    println!("\n[TEST INCOMPLETE - EXPOSES FAST CLOCK BUG]");
}

/// E2E Test: Complete workflow with script
#[tokio::test]
async fn multi_device_with_remap_script() {
    use keyrx_core::cli::commands::RunCommand;
    use keyrx_core::cli::OutputFormat;

    println!("\n=== MULTI-DEVICE WITH REMAP SCRIPT TEST ===");

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("multi_device.rhai");

    // Simple remap script
    let script = r#"
        // Remap CapsLock to Escape on any keyboard
        remap("CapsLock", "Escape");

        // Map Ctrl+Shift+A to custom action
        combo(["Ctrl", "Shift", "A"], || {
            tap("B");
        });
    "#;

    fs::write(&script_path, script).unwrap();

    let devices = vec![
        PathBuf::from("/dev/input/event0"),
        PathBuf::from("/dev/input/event1"),
    ];

    let cmd = RunCommand::new(
        Some(script_path),
        true,
        true, // Use mock mode for testing
        None,
        OutputFormat::Human,
    )
    .with_multi_device(false, Some(devices))
    .with_mock_run_limit(tokio::time::Duration::from_millis(100));

    println!("Running multi-device with remap script...");

    let result = cmd.run().await;

    if result.is_ok() {
        println!("SUCCESS: Multi-device script execution completed");
    } else {
        println!("FAILED: {:?}", result.err());
    }

    assert!(result.is_ok(), "Multi-device script execution should succeed");
}

/// E2E Test: Device disconnection during operation
///
/// SEVERE BUG: What happens when a device disconnects mid-session?
#[tokio::test]
async fn device_disconnection_during_operation() {
    println!("\n=== DEVICE DISCONNECTION TEST ===");
    println!("Scenario: KB2 unplugged while KB1 still active");

    println!("\nExpected behavior:");
    println!("1. KB1 and KB2 both sending events");
    println!("2. KB2 unplugged");
    println!("3. Error on next poll from KB2");
    println!("4. Continue processing KB1 events");
    println!("5. Graceful handling, no crash");

    println!("\nBUG RISKS:");
    println!("- Entire multi-device input crashes");
    println!("- Events in KB2 buffer are lost");
    println!("- KB1 stops working due to KB2 failure");
    println!("- Deadlock waiting for KB2");

    println!("\nCurrent code has error handling:");
    println!("  On poll error: log warning, continue with other devices");
    println!("  This should prevent total failure");

    println!("\n[TEST INCOMPLETE - NEEDS DEVICE MOCK WITH DISCONNECT]");
}

/// E2E Test: Emergency exit with multi-device
#[tokio::test]
async fn emergency_exit_multi_device() {
    println!("\n=== EMERGENCY EXIT TEST ===");
    println!("Scenario: User triggers emergency exit (e.g., Ctrl+Alt+Shift+Backspace)");

    println!("\nExpected behavior:");
    println!("1. Emergency exit combo pressed across multiple keyboards");
    println!("2. All devices detected the combo");
    println!("3. Clean shutdown of all devices");
    println!("4. No hung processes");

    println!("\nBUG RISKS:");
    println!("- Emergency exit not detected when keys on different devices");
    println!("- Some devices not cleaned up on exit");
    println!("- Devices left in grabbed state");

    println!("\n[TEST INCOMPLETE - NEEDS EMERGENCY EXIT SIMULATION]");
}

/// E2E Test: Performance with many devices
#[tokio::test]
async fn performance_with_many_devices() {
    println!("\n=== PERFORMANCE TEST: 10 DEVICES ===");

    let devices: Vec<PathBuf> = (0..10)
        .map(|i| PathBuf::from(format!("/dev/input/event{}", i)))
        .collect();

    println!("Devices: {}", devices.len());

    println!("\nPerformance concerns:");
    println!("- Sequential polling of 10 devices may be slow");
    println!("- BinaryHeap operations with many events");
    println!("- Memory usage with large event buffer");

    println!("\nExpected latency:");
    println!("- Single device: ~1ms");
    println!("- 10 devices sequential: ~10ms");
    println!("- 10 devices concurrent: ~2ms (if parallelized)");

    println!("\nCurrent implementation: SEQUENTIAL polling");
    println!("  This may have noticeable latency with many devices");

    println!("\n[TEST INCOMPLETE - NEEDS PERFORMANCE MEASUREMENT]");
}

/// E2E Test: Memory leak detection
#[tokio::test]
async fn memory_leak_detection() {
    println!("\n=== MEMORY LEAK TEST ===");

    println!("Scenario: Run multi-device for extended period");
    println!("Check: Memory usage should remain constant");

    println!("\nPotential memory leaks:");
    println!("- Event buffer grows unbounded");
    println!("- Old events not released from heap");
    println!("- Device structs not properly dropped");

    println!("\nMitigation:");
    println!("- Buffer delay ensures old events are released");
    println!("- Drop implementation should clean up devices");
    println!("- No manual memory management (Rust safety)");

    println!("\n[TEST INCOMPLETE - NEEDS LONG-RUNNING TEST]");
}
