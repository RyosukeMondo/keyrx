//! Golden session recording and verification.

use serde::{Deserialize, Serialize};

/// A golden session recording.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenSession {
    /// Session name.
    pub name: String,
    /// Schema version.
    pub version: String,
    /// Creation timestamp.
    pub created: String,
    /// Session metadata.
    pub metadata: serde_json::Value,
    /// Recorded events.
    pub events: Vec<serde_json::Value>,
    /// Expected outputs.
    pub expected_outputs: Vec<serde_json::Value>,
}

/// Result of golden session verification.
#[derive(Debug, Clone)]
pub struct GoldenVerifyResult {
    /// Whether verification passed.
    pub passed: bool,
    /// List of differences found.
    pub differences: Vec<GoldenDifference>,
}

/// A difference found during golden verification.
#[derive(Debug, Clone)]
pub struct GoldenDifference {
    /// Event index where difference occurred.
    pub event_index: usize,
    /// Expected value.
    pub expected: String,
    /// Actual value.
    pub actual: String,
}

/// Manager for golden session operations.
#[derive(Debug)]
pub struct GoldenSessionManager;
