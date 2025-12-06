//! Error metrics tracking.
//!
//! This module provides lightweight error counting by type along with a
//! rolling error rate calculation. Counts are tracked per error type and
//! aggregated totals are exposed for snapshots and exporters.
//!
//! # Design
//!
//! - Per-type counts stored in a concurrent map (DashMap) with atomic values
//! - Rolling error rate over a fixed window (60s) using a bounded deque
//! - Thread-safe: atomic counters + mutex for the rate window
//!
//! # Performance
//!
//! - `record`: O(1) with relaxed atomics; mutex is uncontended for low error rates
//! - Rate calculation prunes stale buckets opportunistically on reads/writes

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Rolling window used for rate calculations (60 seconds).
const RATE_WINDOW: Duration = Duration::from_secs(60);

/// Snapshot of error metrics for serialization and export.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorSnapshot {
    /// Total errors recorded across all types.
    #[serde(rename = "total")]
    pub total: u64,

    /// Rolling errors-per-minute rate over the configured window.
    #[serde(rename = "rate_per_minute")]
    pub rate_per_minute: f64,

    /// Counts per error type.
    #[serde(rename = "by_type")]
    pub by_type: HashMap<String, u64>,
}

impl ErrorSnapshot {
    /// Create an empty snapshot.
    pub fn empty() -> Self {
        Self {
            total: 0,
            rate_per_minute: 0.0,
            by_type: HashMap::new(),
        }
    }
}

/// Thread-safe error metrics tracker with per-type counts and rolling rate.
#[derive(Debug)]
pub struct ErrorMetrics {
    total: AtomicU64,
    counts: DashMap<String, AtomicU64>,
    rate_window: Mutex<VecDeque<(Instant, u64)>>,
}

impl ErrorMetrics {
    /// Create a new error metrics tracker.
    pub fn new() -> Self {
        Self {
            total: AtomicU64::new(0),
            counts: DashMap::new(),
            rate_window: Mutex::new(VecDeque::new()),
        }
    }

    /// Record an error occurrence with the provided type label.
    pub fn record(&self, error_type: &str) {
        self.record_at(error_type, Instant::now());
    }

    /// Reset all tracked error metrics.
    pub fn reset(&self) {
        self.total.store(0, Ordering::Relaxed);
        self.counts.clear();
        if let Ok(mut window) = self.rate_window.lock() {
            window.clear();
        }
    }

    /// Create a snapshot of current error metrics.
    pub fn snapshot(&self) -> ErrorSnapshot {
        let now = Instant::now();
        let rate_per_minute = self.rate_at(now);
        let by_type = self
            .counts
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().load(Ordering::Relaxed)))
            .collect();

        ErrorSnapshot {
            total: self.total.load(Ordering::Relaxed),
            rate_per_minute,
            by_type,
        }
    }

    /// Record an error occurrence at a specific timestamp (primarily for testing).
    pub(crate) fn record_at(&self, error_type: &str, now: Instant) {
        self.total.fetch_add(1, Ordering::Relaxed);
        let entry = self
            .counts
            .entry(error_type.to_string())
            .or_insert_with(|| AtomicU64::new(0));
        entry.fetch_add(1, Ordering::Relaxed);
        self.push_window_event(now);
    }

    /// Calculate rate per minute at the provided time (used for snapshots/tests).
    fn rate_at(&self, now: Instant) -> f64 {
        if let Ok(mut window) = self.rate_window.lock() {
            Self::prune(&mut window, now);

            if window.is_empty() {
                return 0.0;
            }

            let total_events: u64 = window.iter().map(|(_, count)| *count).sum();
            let span_start = match window.front() {
                Some((ts, _)) => *ts,
                None => return 0.0,
            };
            let elapsed = now
                .checked_duration_since(span_start)
                .unwrap_or_default()
                .max(Duration::from_secs(1));

            return total_events as f64 / (elapsed.as_secs_f64() / 60.0);
        }

        0.0
    }

    /// Push an event timestamp into the rolling window.
    fn push_window_event(&self, now: Instant) {
        if let Ok(mut window) = self.rate_window.lock() {
            match window.back_mut() {
                Some((last_ts, count))
                    if now
                        .checked_duration_since(*last_ts)
                        .map(|delta| delta < Duration::from_secs(1))
                        .unwrap_or(false) =>
                {
                    *count += 1;
                }
                _ => window.push_back((now, 1)),
            }

            Self::prune(&mut window, now);
        }
    }

    /// Remove window entries older than the configured rate window.
    fn prune(window: &mut VecDeque<(Instant, u64)>, now: Instant) {
        window.retain(|(ts, _)| match now.checked_duration_since(*ts) {
            Some(delta) => delta <= RATE_WINDOW,
            None => true,
        });
    }
}

impl Default for ErrorMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_counts_by_type() {
        let metrics = ErrorMetrics::new();

        metrics.record_at("io", Instant::now());
        metrics.record_at("io", Instant::now());
        metrics.record_at("parser", Instant::now());

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total, 3);
        assert_eq!(snapshot.by_type.get("io"), Some(&2));
        assert_eq!(snapshot.by_type.get("parser"), Some(&1));
    }

    #[test]
    fn calculates_rate_over_window() {
        let metrics = ErrorMetrics::new();
        let now = Instant::now();

        // Two errors 10 seconds ago
        let ten_secs_ago = now.checked_sub(Duration::from_secs(10)).unwrap();
        metrics.record_at("io", ten_secs_ago);
        metrics.record_at("io", ten_secs_ago);

        // One error 70 seconds ago should be pruned
        let too_old = now.checked_sub(Duration::from_secs(70)).unwrap();
        metrics.record_at("stale", too_old);

        let rate = metrics.rate_at(now);
        // 2 events over ~10 seconds => ~12 per minute
        assert!(rate > 10.0 && rate < 15.0, "rate was {}", rate);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total, 3);
        assert!(snapshot.rate_per_minute > 10.0);
    }

    #[test]
    fn reset_clears_state() {
        let metrics = ErrorMetrics::new();
        metrics.record_at("io", Instant::now());

        metrics.reset();
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total, 0);
        assert!(snapshot.by_type.is_empty());
        assert_eq!(snapshot.rate_per_minute, 0.0);
    }
}
