//! UAT test runner with discovery, filtering, and execution.

/// Test priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Priority {
    /// Critical path tests that must always pass.
    P0,
    /// High priority tests.
    P1,
    /// Medium priority tests.
    P2,
}

/// A discovered UAT test.
#[derive(Debug, Clone)]
pub struct UatTest {
    /// Test name.
    pub name: String,
    /// Source file path.
    pub file: String,
    /// Test category.
    pub category: String,
    /// Test priority.
    pub priority: Priority,
    /// Linked requirement IDs.
    pub requirements: Vec<String>,
    /// Latency threshold in microseconds.
    pub latency_threshold: Option<u64>,
}

/// Filter for selecting UAT tests.
#[derive(Debug, Default, Clone)]
pub struct UatFilter {
    /// Filter by categories.
    pub categories: Vec<String>,
    /// Filter by priorities.
    pub priorities: Vec<Priority>,
    /// Filter by name pattern.
    pub pattern: Option<String>,
}

/// Result of a single UAT test.
#[derive(Debug, Clone)]
pub struct UatResult {
    /// Test that was run.
    pub test: UatTest,
    /// Whether the test passed.
    pub passed: bool,
    /// Duration in microseconds.
    pub duration_us: u64,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Aggregated results from a UAT run.
#[derive(Debug, Clone)]
pub struct UatResults {
    /// Total tests run.
    pub total: usize,
    /// Tests that passed.
    pub passed: usize,
    /// Tests that failed.
    pub failed: usize,
    /// Tests that were skipped.
    pub skipped: usize,
    /// Total duration in microseconds.
    pub duration_us: u64,
    /// Individual test results.
    pub results: Vec<UatResult>,
}

/// UAT test runner.
#[derive(Debug)]
pub struct UatRunner;
