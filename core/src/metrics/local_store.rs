//! In-memory time-series store for metrics snapshots.
//!
//! This module keeps a bounded, queryable history of `MetricsSnapshot` values
//! so the UI can render recent metrics without hitting external backends.
//! Entries are pruned by retention window and max sample count to avoid
//! unbounded memory growth.

use super::snapshot::MetricsSnapshot;
use std::collections::VecDeque;
use std::sync::RwLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Default maximum number of snapshots to retain in memory.
pub const DEFAULT_MAX_SAMPLES: usize = 1024;

/// Default retention window (1 hour) used to prune stale snapshots.
pub const DEFAULT_RETENTION: Duration = Duration::from_secs(60 * 60);

/// Thread-safe local store for metrics snapshots.
///
/// The store keeps snapshots ordered by their timestamp, enforcing both a
/// maximum sample count and a retention duration. All operations clone
/// snapshots when returning them to avoid exposing internal mutability.
#[derive(Debug)]
pub struct LocalMetricsStore {
    max_samples: usize,
    retention: Duration,
    samples: RwLock<VecDeque<MetricsSnapshot>>,
}

impl LocalMetricsStore {
    /// Create a new store with the provided bounds.
    ///
    /// The store will retain up to `max_samples` snapshots and drop entries
    /// older than `retention` whenever a new snapshot is inserted. A minimum
    /// capacity of 1 sample is always enforced.
    pub fn new(max_samples: usize, retention: Duration) -> Self {
        let bounded_samples = max_samples.max(1);
        Self {
            max_samples: bounded_samples,
            retention,
            samples: RwLock::new(VecDeque::with_capacity(bounded_samples)),
        }
    }

    /// Store a snapshot, pruning stale and excess entries.
    pub fn push(&self, snapshot: MetricsSnapshot) {
        let cutoff = snapshot
            .timestamp
            .saturating_sub(self.retention.as_millis() as u64);

        let mut guard = match self.samples.write() {
            Ok(lock) => lock,
            Err(poisoned) => poisoned.into_inner(),
        };
        while guard.front().is_some_and(|s| s.timestamp < cutoff) {
            guard.pop_front();
        }

        if guard.len() >= self.max_samples {
            guard.pop_front();
        }

        guard.push_back(snapshot);
    }

    /// Return the most recent snapshot, if any.
    pub fn latest(&self) -> Option<MetricsSnapshot> {
        self.samples
            .read()
            .ok()
            .and_then(|samples| samples.back().cloned())
    }

    /// Query snapshots within an optional inclusive timestamp range.
    ///
    /// When `start` or `end` are `None` the range is treated as unbounded on
    /// that side. Returns snapshots ordered from oldest to newest.
    pub fn query(&self, start: Option<u64>, end: Option<u64>) -> Vec<MetricsSnapshot> {
        let start_ts = start.unwrap_or(0);
        let end_ts = end.unwrap_or(u64::MAX);

        if start_ts > end_ts {
            return Vec::new();
        }

        match self.samples.read() {
            Ok(samples) => samples
                .iter()
                .filter(|snapshot| snapshot.timestamp >= start_ts && snapshot.timestamp <= end_ts)
                .cloned()
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Query snapshots captured within the provided trailing window.
    pub fn query_window(&self, window: Duration) -> Vec<MetricsSnapshot> {
        let now = current_timestamp();
        let start = now.saturating_sub(window.as_millis() as u64);
        self.query(Some(start), Some(now))
    }

    /// Number of snapshots currently stored.
    pub fn len(&self) -> usize {
        self.samples
            .read()
            .map(|samples| samples.len())
            .unwrap_or(0)
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove all stored snapshots.
    pub fn clear(&self) {
        if let Ok(mut samples) = self.samples.write() {
            samples.clear();
        }
    }
}

impl Default for LocalMetricsStore {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_SAMPLES, DEFAULT_RETENTION)
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{ErrorSnapshot, MemorySnapshot};
    use std::collections::HashMap;

    fn snapshot_at(timestamp: u64) -> MetricsSnapshot {
        MetricsSnapshot {
            timestamp,
            latencies: HashMap::new(),
            memory: MemorySnapshot::empty(),
            errors: ErrorSnapshot::empty(),
            profiles: HashMap::new(),
        }
    }

    #[test]
    fn enforces_max_samples() {
        let store = LocalMetricsStore::new(3, Duration::from_secs(60));

        store.push(snapshot_at(1));
        store.push(snapshot_at(2));
        store.push(snapshot_at(3));
        store.push(snapshot_at(4));

        assert_eq!(store.len(), 3);
        let snapshots = store.query(None, None);
        let timestamps: Vec<u64> = snapshots.iter().map(|s| s.timestamp).collect();
        assert_eq!(timestamps, vec![2, 3, 4]);
    }

    #[test]
    fn queries_by_range() {
        let store = LocalMetricsStore::new(10, Duration::from_secs(60));
        store.push(snapshot_at(1_000));
        store.push(snapshot_at(2_000));
        store.push(snapshot_at(3_000));

        let results = store.query(Some(1_500), Some(2_500));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].timestamp, 2_000);
    }

    #[test]
    fn prunes_by_retention_on_push() {
        let store = LocalMetricsStore::new(5, Duration::from_millis(1_500));
        store.push(snapshot_at(1_000));
        store.push(snapshot_at(1_800));

        // cutoff should drop the first snapshot (1_800 - 1_500 = 300)
        store.push(snapshot_at(2_800));

        let timestamps: Vec<u64> = store
            .query(None, None)
            .iter()
            .map(|s| s.timestamp)
            .collect();
        assert_eq!(timestamps, vec![1_800, 2_800]);
    }

    #[test]
    fn query_window_returns_recent_snapshots() {
        let store = LocalMetricsStore::new(10, Duration::from_secs(60 * 60));
        let now = current_timestamp();
        store.push(snapshot_at(now.saturating_sub(5_000)));
        store.push(snapshot_at(now.saturating_sub(1_000)));

        let recent = store.query_window(Duration::from_millis(2_000));
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].timestamp, now.saturating_sub(1_000));
    }

    #[test]
    fn latest_returns_most_recent_snapshot() {
        let store = LocalMetricsStore::new(3, Duration::from_secs(60));
        assert!(store.latest().is_none());

        store.push(snapshot_at(10));
        store.push(snapshot_at(20));

        let latest = store.latest().expect("should have snapshot");
        assert_eq!(latest.timestamp, 20);
    }
}
