//! Quality gate configuration and enforcement.

use serde::{Deserialize, Serialize};

/// Quality gate configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    /// Minimum pass rate (0.0-1.0).
    pub pass_rate: f64,
    /// Maximum open P0 issues.
    pub p0_open: usize,
    /// Maximum open P1 issues.
    pub p1_open: usize,
    /// Maximum latency in microseconds.
    pub max_latency_us: u64,
    /// Minimum coverage percentage (0.0-1.0).
    pub coverage_min: f64,
}

/// Result of quality gate evaluation.
#[derive(Debug, Clone)]
pub struct GateResult {
    /// Whether the gate passed.
    pub passed: bool,
    /// List of violations.
    pub violations: Vec<GateViolation>,
}

/// A quality gate violation.
#[derive(Debug, Clone)]
pub struct GateViolation {
    /// Name of the violated criterion.
    pub criterion: String,
    /// Expected value.
    pub expected: String,
    /// Actual value.
    pub actual: String,
}

/// Enforcer for quality gate evaluation.
#[derive(Debug)]
pub struct QualityGateEnforcer;
