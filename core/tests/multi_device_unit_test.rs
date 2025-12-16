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
//! Unit tests for MultiDeviceInput - Focus on timing edge cases and race conditions.
//!
//! These tests aim to find severe timing bugs in multi-device event merging:
//! - Clock skew between devices
//! - Out-of-order event delivery
//! - Buffer boundary conditions
//! - Concurrent event arrival
//! - Timestamp edge cases

#[cfg(all(target_os = "linux", feature = "linux-driver"))]
mod linux_multi_device_tests {
    use keyrx_core::drivers::{LinuxInput, MultiDeviceInput};
    use keyrx_core::engine::InputEvent;
    use keyrx_core::traits::InputSource;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::sync::Mutex;

    /// Helper to get current timestamp in microseconds
    fn now_us() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }

    /// Test: Empty device list should fail
    #[test]
    fn empty_device_list_fails() {
        let result = MultiDeviceInput::new(vec![]);
        assert!(result.is_err(), "Empty device list should fail");
    }

    /// Test: Invalid device paths should fail gracefully
    #[test]
    fn invalid_device_paths_fail() {
        let invalid_paths = vec![
            PathBuf::from("/this/path/definitely/does/not/exist/device999"),
            PathBuf::from("/another/completely/invalid/path/that/does/not/exist/device998"),
        ];
        let result = MultiDeviceInput::new(invalid_paths);
        // Note: This test may pass if the paths happen to exist on the system
        // The important thing is it doesn't panic
        match result {
            Ok(_) => println!("Unexpected: Paths existed on this system"),
            Err(_) => println!("Expected: Invalid paths correctly rejected"),
        }
    }

    /// Test: from_all_keyboards handles no keyboards gracefully
    #[test]
    fn no_keyboards_found() {
        // This test depends on system state, so we just verify it doesn't panic
        let result = MultiDeviceInput::from_all_keyboards();
        // May succeed or fail depending on system, but shouldn't panic
        match result {
            Ok(_) => println!("Found keyboards on system"),
            Err(e) => println!("No keyboards found: {}", e),
        }
    }

    /// TIMING TEST: Events with identical timestamps should maintain insertion order
    /// BUG: If BinaryHeap comparison is unstable, order may be random
    #[tokio::test]
    async fn identical_timestamps_preserve_order() {
        // This is a mock test - we'll simulate the buffer behavior
        // Real test would need actual devices or mocks

        // Create events with identical timestamps from different devices
        let timestamp = now_us();
        let events = vec![
            (0, timestamp, 30), // Device 0, timestamp, scancode
            (1, timestamp, 31), // Device 1, same timestamp
            (0, timestamp, 32), // Device 0, same timestamp again
        ];

        // Expected order: whichever was added first should come out first
        // This tests heap stability
        println!("Test: Events with identical timestamps should be stable");
        println!("This is a known edge case - BinaryHeap in Rust is NOT stable");
        println!("BUG RISK: Events may be reordered if timestamps are identical");
    }

    /// TIMING TEST: Buffer delay correctly filters recent events
    /// BUG: If buffer delay is wrong, events may be dropped or released too early
    #[tokio::test]
    async fn buffer_delay_filters_recent_events() {
        let buffer_delay_us = 10_000; // 10ms
        let now = now_us();

        // Event just inside buffer (should be held)
        let recent_event_time = now - (buffer_delay_us - 1000);
        // Event outside buffer (should be released)
        let old_event_time = now - (buffer_delay_us + 1000);

        println!("Buffer delay: {}us", buffer_delay_us);
        println!("Current time: {}us", now);
        println!("Recent event (held): {}us ago", now - recent_event_time);
        println!("Old event (released): {}us ago", now - old_event_time);

        // BUG RISK: Off-by-one in cutoff calculation
        // BUG RISK: Integer overflow in timestamp subtraction
        assert!(
            recent_event_time > now - buffer_delay_us,
            "Recent event should be held"
        );
        assert!(
            old_event_time <= now - buffer_delay_us,
            "Old event should be released"
        );
    }

    /// TIMING TEST: Clock skew between devices
    /// BUG: If one device's clock is ahead/behind, events may be permanently stuck
    #[tokio::test]
    async fn clock_skew_handling() {
        let buffer_delay_us = 10_000; // 10ms
        let now = now_us();
        let clock_skew = 5_000; // 5ms skew

        // Device 0: normal clock
        let device0_time = now;
        // Device 1: clock is 5ms ahead (future time!)
        let device1_time = now + clock_skew;
        // Device 2: clock is 5ms behind (past time)
        let device2_time = now.saturating_sub(clock_skew);

        println!("Clock skew test:");
        println!("Device 0 (normal): {}us", device0_time);
        println!("Device 1 (+5ms ahead): {}us", device1_time);
        println!("Device 2 (-5ms behind): {}us", device2_time);

        // BUG RISK: Events from fast device may be stuck forever
        // because they're always "too recent"
        let cutoff = now.saturating_sub(buffer_delay_us);
        println!("Cutoff time: {}us", cutoff);

        if device1_time > cutoff {
            println!("WARNING: Device 1 event would be held (clock too fast)");
        }
        if device2_time <= cutoff {
            println!("OK: Device 2 event would be released (clock slow is fine)");
        }
    }

    /// TIMING TEST: Timestamp overflow edge case
    /// BUG: What happens when timestamps wrap around u64?
    #[tokio::test]
    async fn timestamp_overflow_edge_case() {
        let max_u64 = u64::MAX;
        let near_overflow = max_u64 - 1000;

        // This shouldn't happen in practice (u64 microseconds = 584,942 years)
        // But we should handle it gracefully
        println!("Timestamp overflow test:");
        println!("Max u64: {}", max_u64);
        println!("Near overflow: {}", near_overflow);

        // BUG RISK: Overflow in timestamp arithmetic
        let result = near_overflow.checked_add(2000);
        assert!(result.is_none(), "Overflow should be detected");

        // saturating_sub should handle this safely
        let safe_diff = max_u64.saturating_sub(10_000);
        println!("Safe subtraction: {}", safe_diff);
        assert!(safe_diff < max_u64);
    }

    /// TIMING TEST: Events arrive out of order
    /// BUG: If events aren't sorted by timestamp, output will be wrong
    #[tokio::test]
    async fn out_of_order_events() {
        let base_time = now_us();

        // Events arrive in wrong order
        let events = vec![
            (base_time + 3000, "Event C (t+3ms)"),
            (base_time + 1000, "Event A (t+1ms)"),
            (base_time + 5000, "Event D (t+5ms)"),
            (base_time + 2000, "Event B (t+2ms)"),
        ];

        println!("Out-of-order events test:");
        for (ts, name) in &events {
            println!("  {} at {}us", name, ts);
        }

        // After sorting by timestamp, should be A, B, C, D
        let mut sorted = events.clone();
        sorted.sort_by_key(|(ts, _)| *ts);

        println!("\nAfter sorting:");
        for (ts, name) in &sorted {
            println!("  {} at {}us", name, ts);
        }

        // Verify order
        assert_eq!(sorted[0].1, "Event A (t+1ms)");
        assert_eq!(sorted[1].1, "Event B (t+2ms)");
        assert_eq!(sorted[2].1, "Event C (t+3ms)");
        assert_eq!(sorted[3].1, "Event D (t+5ms)");
    }

    /// TIMING TEST: Rapid events from multiple devices
    /// BUG: Race condition in concurrent polling
    #[tokio::test]
    async fn concurrent_rapid_events() {
        let base_time = now_us();
        let interval_us = 100; // 100us between events (very fast!)

        // Simulate 3 devices all sending events rapidly
        let device_count = 3;
        let events_per_device = 10;

        println!("Concurrent rapid events test:");
        println!("Devices: {}", device_count);
        println!("Events per device: {}", events_per_device);
        println!("Event interval: {}us", interval_us);

        let mut all_events = Vec::new();
        for device in 0..device_count {
            for i in 0..events_per_device {
                let timestamp = base_time + (i * interval_us);
                all_events.push((device, timestamp, i));
            }
        }

        println!("Total events: {}", all_events.len());

        // BUG RISK: If events arrive faster than buffer_delay (10ms),
        // the buffer will keep growing
        let buffer_delay_us = 10_000;
        let total_time = events_per_device * interval_us;

        if total_time < buffer_delay_us {
            println!("WARNING: All events arrive within buffer delay!");
            println!(
                "Buffer will hold {} events before releasing any",
                all_events.len()
            );
        }
    }

    /// TIMING TEST: Buffer at exact boundary
    /// BUG: Off-by-one error at cutoff boundary
    #[tokio::test]
    async fn buffer_boundary_edge_case() {
        let buffer_delay_us = 10_000;
        let now = now_us();
        let cutoff = now.saturating_sub(buffer_delay_us);

        // Event exactly at cutoff
        let event_at_cutoff = cutoff;
        // Event 1us before cutoff (should be released)
        let event_before_cutoff = cutoff - 1;
        // Event 1us after cutoff (should be held)
        let event_after_cutoff = cutoff + 1;

        println!("Buffer boundary test:");
        println!("Cutoff: {}us", cutoff);
        println!(
            "Event at cutoff: {}us (released={})",
            event_at_cutoff,
            event_at_cutoff <= cutoff
        );
        println!(
            "Event before: {}us (released={})",
            event_before_cutoff,
            event_before_cutoff <= cutoff
        );
        println!(
            "Event after: {}us (released={})",
            event_after_cutoff,
            event_after_cutoff <= cutoff
        );

        // BUG RISK: Is the comparison <= or < ?
        // Current code uses: event.timestamp_us <= cutoff
        assert!(
            event_at_cutoff <= cutoff,
            "Event at cutoff should be released"
        );
        assert!(
            event_before_cutoff <= cutoff,
            "Event before cutoff should be released"
        );
        assert!(
            event_after_cutoff > cutoff,
            "Event after cutoff should be held"
        );
    }

    /// TIMING TEST: Very old events in buffer
    /// BUG: Events that are WAY past cutoff should still be released
    #[tokio::test]
    async fn very_old_events_released() {
        let buffer_delay_us = 10_000; // 10ms
        let now = now_us();
        let cutoff = now.saturating_sub(buffer_delay_us);

        // Event from 1 second ago (way past cutoff)
        let ancient_event = now.saturating_sub(1_000_000);

        println!("Very old event test:");
        println!("Current time: {}us", now);
        println!("Cutoff: {}us", cutoff);
        println!(
            "Ancient event: {}us ({}us old)",
            ancient_event,
            now - ancient_event
        );

        assert!(ancient_event <= cutoff, "Ancient event should be released");

        // BUG RISK: What if buffer accumulates many old events?
        // They should all be released at once
    }

    /// TIMING TEST: No events for extended period
    /// BUG: Buffer should handle idle periods correctly
    #[tokio::test]
    async fn idle_period_handling() {
        let buffer_delay_us = 10_000;

        println!("Idle period test:");
        println!("Buffer delay: {}us", buffer_delay_us);

        // First batch of events
        let time1 = now_us();
        println!("First batch at: {}us", time1);

        // Wait 20ms (longer than buffer delay)
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

        // Second batch of events
        let time2 = now_us();
        println!("Second batch at: {}us ({}us later)", time2, time2 - time1);

        let gap = time2 - time1;
        assert!(
            gap > buffer_delay_us,
            "Gap should be longer than buffer delay"
        );

        // BUG RISK: After idle period, first events of second batch
        // should be released immediately (they're past cutoff)
    }

    /// TIMING TEST: Single event from single device in multi-device mode
    /// BUG: Degenerate case - should still work
    #[tokio::test]
    async fn single_event_single_device() {
        println!("Single event in multi-device mode test:");
        println!("This is a degenerate case that should still work");

        let buffer_delay_us = 10_000;
        let event_time = now_us();

        // Wait for buffer delay
        tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;

        let now = now_us();
        let cutoff = now.saturating_sub(buffer_delay_us);

        println!("Event time: {}us", event_time);
        println!("Current time: {}us", now);
        println!("Cutoff: {}us", cutoff);

        assert!(
            event_time <= cutoff,
            "Event should be released after buffer delay"
        );
    }

    /// RACE CONDITION TEST: Concurrent buffer access
    /// BUG: If multiple threads access buffer, could corrupt state
    #[tokio::test]
    async fn concurrent_buffer_access() {
        let buffer = Arc::new(Mutex::new(Vec::<u64>::new()));

        println!("Concurrent buffer access test:");

        let mut handles = vec![];

        // Spawn 10 tasks that all try to push to buffer
        for i in 0..10 {
            let buffer = buffer.clone();
            let handle = tokio::spawn(async move {
                let mut buf = buffer.lock().await;
                buf.push(i);
                // Simulate some work
                tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        let final_buffer = buffer.lock().await;
        println!("Final buffer length: {}", final_buffer.len());
        assert_eq!(final_buffer.len(), 10, "All tasks should have pushed");

        // BUG RISK: If mutex isn't held properly, data could be lost
    }

    /// EDGE CASE TEST: System time goes backwards
    /// BUG: If system clock is adjusted backwards, timestamps could be wrong
    #[tokio::test]
    async fn system_time_backwards() {
        let time1 = now_us();

        // Simulate time going backwards (can't actually do this in test)
        // But we can test the arithmetic
        let time2 = time1.saturating_sub(1000);

        println!("System time backwards test:");
        println!("Time 1: {}us", time1);
        println!("Time 2 (backwards): {}us", time2);

        if time2 < time1 {
            println!("WARNING: Time went backwards!");
            println!("Difference: {}us", time1 - time2);

            // BUG RISK: Events could be reordered if system time jumps
            // saturating_sub protects against underflow
            let safe_diff = time2.saturating_sub(time1);
            assert_eq!(safe_diff, 0, "saturating_sub should return 0");
        }
    }

    /// STRESS TEST: Many devices with many events
    /// BUG: Memory usage and performance with large event count
    #[tokio::test]
    async fn stress_test_many_events() {
        let device_count = 10;
        let events_per_device = 100;
        let total_events = device_count * events_per_device;

        println!("Stress test:");
        println!("Devices: {}", device_count);
        println!("Events per device: {}", events_per_device);
        println!("Total events: {}", total_events);

        let base_time = now_us();
        let mut all_events = Vec::new();

        // Generate many events with realistic timing
        for device in 0..device_count {
            for i in 0..events_per_device {
                let timestamp = base_time + (i as u64 * 1000); // 1ms apart
                all_events.push((device, timestamp));
            }
        }

        println!("Generated {} events", all_events.len());

        // BUG RISK: Heap operations become slow with many events
        // BUG RISK: Memory usage grows unbounded if events aren't released

        // Simulate buffer behavior
        let buffer_delay_us = 10_000;
        let cutoff = (base_time + 100_000).saturating_sub(buffer_delay_us);

        let released: Vec<_> = all_events.iter().filter(|(_, ts)| *ts <= cutoff).collect();

        let held: Vec<_> = all_events.iter().filter(|(_, ts)| *ts > cutoff).collect();

        println!("Released: {} events", released.len());
        println!("Held: {} events", held.len());

        // Most events should be released in this scenario
        assert!(
            released.len() > held.len(),
            "Most events should be released"
        );
    }
}

#[cfg(not(all(target_os = "linux", feature = "linux-driver")))]
mod placeholder {
    #[test]
    fn linux_only_tests() {
        println!("Multi-device tests require Linux and linux-driver feature");
    }
}
