//! Metrics snapshot for export and serialization.
//!
//! This module provides serializable snapshots of metrics data, enabling
//! JSON export for debugging, monitoring, and integration with external tools.
//!
//! # Design
//!
//! - All types are `Serialize + Deserialize` for flexible export formats
//! - Snapshots are immutable point-in-time views (no interior mutability)
//! - FFI-compatible: simple types that can be marshaled across FFI boundary
//! - Bounded size: snapshots don't grow unbounded over time
//!
//! # Example
//!
//! ```ignore
//! use keyrx_core::metrics::{MetricsCollector, MetricsSnapshot};
//!
//! // Collect some metrics
//! collector.record_latency(Operation::EventProcess, 150);
//! collector.record_memory(1024 * 1024);
//!
//! // Take a snapshot
//! let snapshot = collector.snapshot();
//!
//! // Export to JSON
//! let json = serde_json::to_string_pretty(&snapshot).unwrap();
//! println!("{}", json);
//! ```

use super::errors::ErrorSnapshot;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Snapshot of all metrics at a point in time.
///
/// This type captures a complete view of all collected metrics including
/// latency histograms, memory usage, and profile points. It's designed to
/// be serialized to JSON for export or sent across FFI boundaries.
///
/// # Thread Safety
///
/// Snapshots are immutable and can be safely shared across threads.
///
/// # Serialization
///
/// All fields use simple types that serialize cleanly to JSON:
/// - Numbers as numbers
/// - Maps as objects
/// - Arrays as arrays
///
/// # Example JSON Output
///
/// ```json
/// {
///   "timestamp": 1701234567890,
///   "latencies": {
///     "event_process": {
///       "count": 1000,
///       "p50": 100,
///       "p95": 250,
///       "p99": 500,
///       "mean": 120.5,
///       "min": 50,
///       "max": 1000
///     }
///   },
///   "memory": {
///     "current": 10485760,
///     "peak": 15728640,
///     "baseline": 8388608,
///     "growth": 2097152,
///     "has_potential_leak": false
///   },
///   "profiles": {
///     "process_event": {
///       "count": 1000,
///       "total_micros": 120000,
///       "avg_micros": 120,
///       "min_micros": 50,
///       "max_micros": 500
///     }
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Unix timestamp in milliseconds when snapshot was taken.
    #[serde(rename = "timestamp")]
    pub timestamp: u64,

    /// Latency statistics for each operation type.
    ///
    /// Maps operation name to histogram statistics.
    #[serde(rename = "latencies")]
    pub latencies: HashMap<String, LatencyStats>,

    /// Memory usage statistics.
    #[serde(rename = "memory")]
    pub memory: MemorySnapshot,

    /// Error statistics including counts and rolling rate.
    #[serde(rename = "errors")]
    pub errors: ErrorSnapshot,

    /// Profile point statistics.
    ///
    /// Maps profile point name to timing statistics.
    #[serde(rename = "profiles")]
    pub profiles: HashMap<String, ProfileSnapshot>,
}

impl MetricsSnapshot {
    /// Create an empty snapshot with current timestamp.
    ///
    /// This is used by NoOpCollector and as a starting point for
    /// building snapshots.
    pub fn empty() -> Self {
        Self {
            timestamp: Self::current_timestamp(),
            latencies: HashMap::new(),
            memory: MemorySnapshot::empty(),
            errors: ErrorSnapshot::empty(),
            profiles: HashMap::new(),
        }
    }

    /// Create a new snapshot with all metrics.
    ///
    /// # Arguments
    ///
    /// * `latencies` - Latency statistics per operation
    /// * `memory` - Memory usage snapshot
    /// * `errors` - Error metrics snapshot
    /// * `profiles` - Profile point statistics
    ///
    /// # Example
    ///
    /// ```ignore
    /// let snapshot = MetricsSnapshot::new(latencies_map, memory_snap, errors, profiles_map);
    /// ```
    pub fn new(
        latencies: HashMap<String, LatencyStats>,
        memory: MemorySnapshot,
        errors: ErrorSnapshot,
        profiles: HashMap<String, ProfileSnapshot>,
    ) -> Self {
        Self {
            timestamp: Self::current_timestamp(),
            latencies,
            memory,
            errors,
            profiles,
        }
    }

    /// Get current Unix timestamp in milliseconds.
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Export snapshot to JSON string.
    ///
    /// # Returns
    ///
    /// Pretty-printed JSON string, or error if serialization fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let json = snapshot.to_json()?;
    /// println!("{}", json);
    /// ```
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Export snapshot to compact JSON string.
    ///
    /// # Returns
    ///
    /// Compact JSON string without formatting, or error if serialization fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let json = snapshot.to_json_compact()?;
    /// // Send over network or FFI
    /// ```
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Parse a snapshot from JSON string.
    ///
    /// # Arguments
    ///
    /// * `json` - JSON string to parse
    ///
    /// # Returns
    ///
    /// Parsed snapshot, or error if JSON is invalid.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let snapshot = MetricsSnapshot::from_json(json_string)?;
    /// ```
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Latency statistics for a single operation.
///
/// Contains histogram percentiles and basic statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    /// Total number of samples recorded.
    #[serde(rename = "count")]
    pub count: u64,

    /// 50th percentile (median) in microseconds.
    #[serde(rename = "p50")]
    pub p50: u64,

    /// 95th percentile in microseconds.
    #[serde(rename = "p95")]
    pub p95: u64,

    /// 99th percentile in microseconds.
    #[serde(rename = "p99")]
    pub p99: u64,

    /// Mean (average) latency in microseconds.
    #[serde(rename = "mean")]
    pub mean: f64,

    /// Minimum recorded latency in microseconds.
    #[serde(rename = "min")]
    pub min: u64,

    /// Maximum recorded latency in microseconds.
    #[serde(rename = "max")]
    pub max: u64,
}

impl LatencyStats {
    /// Create empty latency statistics.
    pub fn empty() -> Self {
        Self {
            count: 0,
            p50: 0,
            p95: 0,
            p99: 0,
            mean: 0.0,
            min: 0,
            max: 0,
        }
    }
}

/// Memory usage snapshot.
///
/// Captures current, peak, and baseline memory along with leak detection status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// Current memory usage in bytes.
    #[serde(rename = "current")]
    pub current: usize,

    /// Peak memory usage in bytes.
    #[serde(rename = "peak")]
    pub peak: usize,

    /// Baseline memory usage (first sample) in bytes.
    #[serde(rename = "baseline")]
    pub baseline: usize,

    /// Growth from baseline in bytes (current - baseline).
    #[serde(rename = "growth")]
    pub growth: usize,

    /// Whether a potential memory leak has been detected.
    #[serde(rename = "has_potential_leak")]
    pub has_potential_leak: bool,
}

impl MemorySnapshot {
    /// Create empty memory snapshot.
    pub fn empty() -> Self {
        Self {
            current: 0,
            peak: 0,
            baseline: 0,
            growth: 0,
            has_potential_leak: false,
        }
    }

    /// Create a new memory snapshot.
    pub fn new(
        current: usize,
        peak: usize,
        baseline: usize,
        growth: usize,
        has_potential_leak: bool,
    ) -> Self {
        Self {
            current,
            peak,
            baseline,
            growth,
            has_potential_leak,
        }
    }
}

/// Profile point statistics snapshot.
///
/// Contains aggregated timing statistics for a single profile point.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProfileSnapshot {
    /// Total number of times this profile point was recorded.
    #[serde(rename = "count")]
    pub count: u64,

    /// Total accumulated time in microseconds across all calls.
    #[serde(rename = "total_micros")]
    pub total_micros: u64,

    /// Average time per call in microseconds.
    #[serde(rename = "avg_micros")]
    pub avg_micros: u64,

    /// Minimum recorded time in microseconds.
    #[serde(rename = "min_micros")]
    pub min_micros: u64,

    /// Maximum recorded time in microseconds.
    #[serde(rename = "max_micros")]
    pub max_micros: u64,
}

impl ProfileSnapshot {
    /// Create empty profile snapshot.
    pub fn empty() -> Self {
        Self {
            count: 0,
            total_micros: 0,
            avg_micros: 0,
            min_micros: 0,
            max_micros: 0,
        }
    }

    /// Create a new profile snapshot.
    pub fn new(
        count: u64,
        total_micros: u64,
        avg_micros: u64,
        min_micros: u64,
        max_micros: u64,
    ) -> Self {
        Self {
            count,
            total_micros,
            avg_micros,
            min_micros,
            max_micros,
        }
    }
}

impl From<super::profile::ProfileStats> for ProfileSnapshot {
    fn from(stats: super::profile::ProfileStats) -> Self {
        Self {
            count: stats.count,
            total_micros: stats.total_micros,
            avg_micros: stats.avg_micros,
            min_micros: stats.min_micros,
            max_micros: stats.max_micros,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_snapshot() {
        let snapshot = MetricsSnapshot::empty();
        assert!(snapshot.latencies.is_empty());
        assert_eq!(snapshot.memory.current, 0);
        assert_eq!(snapshot.errors.total, 0);
        assert!(snapshot.errors.by_type.is_empty());
        assert!(snapshot.profiles.is_empty());
        assert!(snapshot.timestamp > 0);
    }

    #[test]
    fn test_snapshot_serialization() {
        let mut latencies = HashMap::new();
        latencies.insert(
            "event_process".to_string(),
            LatencyStats {
                count: 100,
                p50: 100,
                p95: 250,
                p99: 500,
                mean: 120.5,
                min: 50,
                max: 1000,
            },
        );

        let memory = MemorySnapshot::new(1024 * 1024, 2048 * 1024, 512 * 1024, 512 * 1024, false);

        let mut profiles = HashMap::new();
        profiles.insert(
            "test_function".to_string(),
            ProfileSnapshot::new(50, 5000, 100, 50, 200),
        );

        let mut errors = ErrorSnapshot::empty();
        errors.total = 5;
        errors.rate_per_minute = 12.5;
        errors.by_type.insert("io".to_string(), 3);
        errors.by_type.insert("parse".to_string(), 2);

        let snapshot = MetricsSnapshot::new(latencies, memory, errors, profiles);

        // Test JSON serialization
        let json = snapshot.to_json().unwrap();
        assert!(json.contains("timestamp"));
        assert!(json.contains("latencies"));
        assert!(json.contains("memory"));
        assert!(json.contains("errors"));
        assert!(json.contains("profiles"));
        assert!(json.contains("event_process"));
        assert!(json.contains("test_function"));

        // Test deserialization
        let parsed = MetricsSnapshot::from_json(&json).unwrap();
        assert_eq!(parsed.latencies.len(), 1);
        assert_eq!(parsed.profiles.len(), 1);
        assert_eq!(parsed.errors.total, 5);
        assert_eq!(parsed.errors.by_type.len(), 2);
    }

    #[test]
    fn test_compact_serialization() {
        let snapshot = MetricsSnapshot::empty();
        let compact = snapshot.to_json_compact().unwrap();
        let pretty = snapshot.to_json().unwrap();

        // Compact should be shorter (no whitespace)
        assert!(compact.len() < pretty.len());

        // Both should deserialize to same data
        let from_compact = MetricsSnapshot::from_json(&compact).unwrap();
        let from_pretty = MetricsSnapshot::from_json(&pretty).unwrap();
        assert_eq!(from_compact.timestamp, from_pretty.timestamp);
    }

    #[test]
    fn test_latency_stats_serialization() {
        let stats = LatencyStats {
            count: 1000,
            p50: 100,
            p95: 250,
            p99: 500,
            mean: 120.5,
            min: 50,
            max: 1000,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let parsed: LatencyStats = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.count, 1000);
        assert_eq!(parsed.p50, 100);
        assert_eq!(parsed.p95, 250);
        assert_eq!(parsed.p99, 500);
        assert_eq!(parsed.mean, 120.5);
        assert_eq!(parsed.min, 50);
        assert_eq!(parsed.max, 1000);
    }

    #[test]
    fn test_memory_snapshot_serialization() {
        let snapshot = MemorySnapshot::new(1024, 2048, 512, 512, true);

        let json = serde_json::to_string(&snapshot).unwrap();
        let parsed: MemorySnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.current, 1024);
        assert_eq!(parsed.peak, 2048);
        assert_eq!(parsed.baseline, 512);
        assert_eq!(parsed.growth, 512);
        assert!(parsed.has_potential_leak);
    }

    #[test]
    fn test_profile_snapshot_serialization() {
        let snapshot = ProfileSnapshot::new(100, 10000, 100, 50, 200);

        let json = serde_json::to_string(&snapshot).unwrap();
        let parsed: ProfileSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.count, 100);
        assert_eq!(parsed.total_micros, 10000);
        assert_eq!(parsed.avg_micros, 100);
        assert_eq!(parsed.min_micros, 50);
        assert_eq!(parsed.max_micros, 200);
    }

    #[test]
    fn test_empty_latency_stats() {
        let stats = LatencyStats::empty();
        assert_eq!(stats.count, 0);
        assert_eq!(stats.p50, 0);
        assert_eq!(stats.p95, 0);
        assert_eq!(stats.p99, 0);
        assert_eq!(stats.mean, 0.0);
        assert_eq!(stats.min, 0);
        assert_eq!(stats.max, 0);
    }

    #[test]
    fn test_empty_memory_snapshot() {
        let snapshot = MemorySnapshot::empty();
        assert_eq!(snapshot.current, 0);
        assert_eq!(snapshot.peak, 0);
        assert_eq!(snapshot.baseline, 0);
        assert_eq!(snapshot.growth, 0);
        assert!(!snapshot.has_potential_leak);
    }

    #[test]
    fn test_empty_profile_snapshot() {
        let snapshot = ProfileSnapshot::empty();
        assert_eq!(snapshot.count, 0);
        assert_eq!(snapshot.total_micros, 0);
        assert_eq!(snapshot.avg_micros, 0);
        assert_eq!(snapshot.min_micros, 0);
        assert_eq!(snapshot.max_micros, 0);
    }

    #[test]
    fn test_profile_stats_conversion() {
        let profile_stats = super::super::profile::ProfileStats {
            count: 50,
            total_micros: 5000,
            avg_micros: 100,
            min_micros: 50,
            max_micros: 200,
        };

        let snapshot: ProfileSnapshot = profile_stats.into();
        assert_eq!(snapshot.count, 50);
        assert_eq!(snapshot.total_micros, 5000);
        assert_eq!(snapshot.avg_micros, 100);
        assert_eq!(snapshot.min_micros, 50);
        assert_eq!(snapshot.max_micros, 200);
    }

    #[test]
    fn test_snapshot_roundtrip() {
        // Create a complex snapshot
        let mut latencies = HashMap::new();
        latencies.insert(
            "op1".to_string(),
            LatencyStats {
                count: 100,
                p50: 100,
                p95: 200,
                p99: 300,
                mean: 110.5,
                min: 50,
                max: 500,
            },
        );
        latencies.insert(
            "op2".to_string(),
            LatencyStats {
                count: 200,
                p50: 150,
                p95: 250,
                p99: 350,
                mean: 160.5,
                min: 80,
                max: 600,
            },
        );

        let memory = MemorySnapshot::new(1024, 2048, 512, 512, false);

        let mut profiles = HashMap::new();
        profiles.insert(
            "func1".to_string(),
            ProfileSnapshot::new(10, 1000, 100, 50, 200),
        );
        profiles.insert(
            "func2".to_string(),
            ProfileSnapshot::new(20, 2000, 100, 60, 180),
        );

        let mut errors = ErrorSnapshot::empty();
        errors.total = 7;
        errors.by_type.insert("io".to_string(), 5);
        errors.by_type.insert("config".to_string(), 2);
        errors.rate_per_minute = 14.0;

        let original = MetricsSnapshot::new(latencies, memory, errors, profiles);

        // Serialize and deserialize
        let json = original.to_json().unwrap();
        let restored = MetricsSnapshot::from_json(&json).unwrap();

        // Verify data integrity
        assert_eq!(original.latencies.len(), restored.latencies.len());
        assert_eq!(original.profiles.len(), restored.profiles.len());
        assert_eq!(original.errors.total, restored.errors.total);

        let op1 = restored.latencies.get("op1").unwrap();
        assert_eq!(op1.count, 100);
        assert_eq!(op1.mean, 110.5);

        let func1 = restored.profiles.get("func1").unwrap();
        assert_eq!(func1.count, 10);
        assert_eq!(func1.total_micros, 1000);
    }

    #[test]
    fn test_timestamp_is_reasonable() {
        let snapshot = MetricsSnapshot::empty();

        // Timestamp should be > year 2020 (1577836800000 ms)
        assert!(snapshot.timestamp > 1_577_836_800_000);

        // Timestamp should be < year 2100 (4102444800000 ms)
        assert!(snapshot.timestamp < 4_102_444_800_000);
    }
}
