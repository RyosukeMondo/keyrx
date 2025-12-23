//! Clock abstraction for time handling.
//!
//! This module provides a time abstraction for tap-hold and other timing-sensitive
//! features. The `Clock` trait enables both real-time operation with system clocks
//! and deterministic testing with virtual clocks.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐
//! │ Clock trait │  <- Abstract time source
//! └──────┬──────┘
//!        │
//!   ┌────┴────┐
//!   │         │
//!   ▼         ▼
//! ┌─────────────┐  ┌─────────────────┐
//! │ SystemClock │  │  VirtualClock   │
//! │ (production)│  │    (testing)    │
//! └─────────────┘  └─────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust
//! use keyrx_core::runtime::clock::{Clock, VirtualClock};
//!
//! // Create virtual clock for testing
//! let clock = VirtualClock::new();
//!
//! // Initially at time 0
//! assert_eq!(clock.now(), 0);
//!
//! // Advance time manually
//! clock.advance(1000);
//! assert_eq!(clock.now(), 1000);
//!
//! // Set to specific time
//! clock.set(5000);
//! assert_eq!(clock.now(), 5000);
//! ```

use core::sync::atomic::{AtomicU64, Ordering};

/// Trait for abstracting time sources.
///
/// All times are in microseconds (μs) for consistency with evdev timestamps
/// and sufficient precision for tap-hold thresholds (typically 150-300ms).
///
/// # Implementations
///
/// - `SystemClock`: Uses the last event timestamp for production use
/// - `VirtualClock`: Manually controllable for deterministic testing
pub trait Clock {
    /// Returns the current time in microseconds.
    ///
    /// For `SystemClock`, this returns the timestamp of the last event.
    /// For `VirtualClock`, this returns the manually set/advanced time.
    fn now(&self) -> u64;
}

/// System clock that tracks the most recent event timestamp.
///
/// In production, keyboard events carry timestamps from the kernel.
/// This clock stores the most recent event timestamp and returns it
/// as the current time. This avoids wall-clock dependencies and ensures
/// timing is based on actual event timing.
///
/// # Example
///
/// ```rust
/// use keyrx_core::runtime::clock::{Clock, SystemClock};
///
/// let clock = SystemClock::new();
///
/// // Initially at time 0
/// assert_eq!(clock.now(), 0);
///
/// // Update with event timestamp
/// clock.update(12345);
/// assert_eq!(clock.now(), 12345);
/// ```
#[derive(Debug, Default)]
pub struct SystemClock {
    /// Last event timestamp in microseconds
    last_timestamp: AtomicU64,
}

impl SystemClock {
    /// Creates a new system clock initialized to time 0.
    pub const fn new() -> Self {
        Self {
            last_timestamp: AtomicU64::new(0),
        }
    }

    /// Updates the clock with a new event timestamp.
    ///
    /// Call this when processing each keyboard event to keep
    /// the clock synchronized with event timing.
    ///
    /// # Arguments
    ///
    /// * `timestamp_us` - Event timestamp in microseconds
    pub fn update(&self, timestamp_us: u64) {
        self.last_timestamp.store(timestamp_us, Ordering::Release);
    }
}

impl Clock for SystemClock {
    fn now(&self) -> u64 {
        self.last_timestamp.load(Ordering::Acquire)
    }
}

/// Virtual clock for deterministic testing.
///
/// This clock allows manual control over time, enabling reproducible
/// tests for timing-sensitive behavior like tap-hold thresholds.
///
/// # Thread Safety
///
/// Uses atomic operations for thread-safe time manipulation,
/// allowing parallel test execution without data races.
///
/// # Example
///
/// ```rust
/// use keyrx_core::runtime::clock::{Clock, VirtualClock};
///
/// let clock = VirtualClock::new();
///
/// // Test tap behavior (quick release)
/// clock.set(0);
/// // ... simulate key press ...
/// clock.advance(100_000); // 100ms
/// // ... simulate key release ...
/// // Should be a tap (< 200ms threshold)
///
/// // Test hold behavior (slow release)
/// clock.set(0);
/// // ... simulate key press ...
/// clock.advance(300_000); // 300ms
/// // ... simulate key release ...
/// // Should be a hold (> 200ms threshold)
/// ```
#[derive(Debug, Default)]
pub struct VirtualClock {
    /// Current virtual time in microseconds
    time: AtomicU64,
}

impl VirtualClock {
    /// Creates a new virtual clock initialized to time 0.
    pub const fn new() -> Self {
        Self {
            time: AtomicU64::new(0),
        }
    }

    /// Sets the virtual clock to a specific time.
    ///
    /// # Arguments
    ///
    /// * `time_us` - New time value in microseconds
    pub fn set(&self, time_us: u64) {
        self.time.store(time_us, Ordering::Release);
    }

    /// Advances the virtual clock by a delta.
    ///
    /// # Arguments
    ///
    /// * `delta_us` - Time to advance in microseconds
    pub fn advance(&self, delta_us: u64) {
        self.time.fetch_add(delta_us, Ordering::AcqRel);
    }

    /// Resets the virtual clock to time 0.
    pub fn reset(&self) {
        self.set(0);
    }
}

impl Clock for VirtualClock {
    fn now(&self) -> u64 {
        self.time.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_clock_initial_value() {
        let clock = SystemClock::new();
        assert_eq!(clock.now(), 0);
    }

    #[test]
    fn test_system_clock_update() {
        let clock = SystemClock::new();

        clock.update(12345);
        assert_eq!(clock.now(), 12345);

        clock.update(67890);
        assert_eq!(clock.now(), 67890);
    }

    #[test]
    fn test_system_clock_default() {
        let clock = SystemClock::default();
        assert_eq!(clock.now(), 0);
    }

    #[test]
    fn test_virtual_clock_initial_value() {
        let clock = VirtualClock::new();
        assert_eq!(clock.now(), 0);
    }

    #[test]
    fn test_virtual_clock_set() {
        let clock = VirtualClock::new();

        clock.set(1000);
        assert_eq!(clock.now(), 1000);

        clock.set(5000);
        assert_eq!(clock.now(), 5000);
    }

    #[test]
    fn test_virtual_clock_advance() {
        let clock = VirtualClock::new();

        clock.advance(100);
        assert_eq!(clock.now(), 100);

        clock.advance(200);
        assert_eq!(clock.now(), 300);

        clock.advance(50);
        assert_eq!(clock.now(), 350);
    }

    #[test]
    fn test_virtual_clock_reset() {
        let clock = VirtualClock::new();

        clock.set(12345);
        assert_eq!(clock.now(), 12345);

        clock.reset();
        assert_eq!(clock.now(), 0);
    }

    #[test]
    fn test_virtual_clock_default() {
        let clock = VirtualClock::default();
        assert_eq!(clock.now(), 0);
    }

    #[test]
    fn test_tap_hold_threshold_simulation() {
        let clock = VirtualClock::new();
        let threshold_us = 200_000; // 200ms

        // Simulate tap (quick release)
        clock.set(0);
        let press_time = clock.now();
        clock.advance(100_000); // 100ms later
        let release_time = clock.now();
        let duration = release_time - press_time;
        assert!(duration < threshold_us, "Should be detected as tap");

        // Simulate hold (slow release)
        clock.reset();
        let press_time = clock.now();
        clock.advance(300_000); // 300ms later
        let release_time = clock.now();
        let duration = release_time - press_time;
        assert!(duration >= threshold_us, "Should be detected as hold");
    }

    #[test]
    fn test_clock_trait_polymorphism() {
        fn measure_duration<C: Clock>(clock: &C, start: u64) -> u64 {
            clock.now() - start
        }

        // Works with SystemClock
        let sys_clock = SystemClock::new();
        sys_clock.update(1000);
        assert_eq!(measure_duration(&sys_clock, 500), 500);

        // Works with VirtualClock
        let virt_clock = VirtualClock::new();
        virt_clock.set(2000);
        assert_eq!(measure_duration(&virt_clock, 1500), 500);
    }

    #[test]
    fn test_edge_case_zero_advance() {
        let clock = VirtualClock::new();
        clock.set(100);
        clock.advance(0);
        assert_eq!(clock.now(), 100);
    }

    #[test]
    fn test_edge_case_large_timestamp() {
        let clock = VirtualClock::new();
        let large_time = u64::MAX - 1;
        clock.set(large_time);
        assert_eq!(clock.now(), large_time);
    }
}
