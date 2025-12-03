//! Performance UAT testing with latency measurement.

/// Performance test result.
#[derive(Debug, Clone)]
pub struct PerformanceResult {
    /// Test name.
    pub test_name: String,
    /// P50 latency in microseconds.
    pub p50_us: u64,
    /// P95 latency in microseconds.
    pub p95_us: u64,
    /// P99 latency in microseconds.
    pub p99_us: u64,
    /// Maximum latency in microseconds.
    pub max_us: u64,
    /// Whether the threshold was exceeded.
    pub threshold_exceeded: bool,
}

/// Performance UAT runner.
#[derive(Debug)]
pub struct PerformanceUat;
