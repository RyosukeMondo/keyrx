//! Quality gate type definitions.
//!
//! This module contains all type definitions for quality gates including
//! gate configurations, results, violations, and errors.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Quality gate configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    /// Minimum pass rate (0.0-1.0).
    #[serde(default = "default_pass_rate")]
    pub pass_rate: f64,
    /// Maximum open P0 issues.
    #[serde(default)]
    pub p0_open: usize,
    /// Maximum open P1 issues.
    #[serde(default = "default_p1_open")]
    pub p1_open: usize,
    /// Maximum latency in microseconds.
    #[serde(default = "default_max_latency")]
    pub max_latency_us: u64,
    /// Minimum coverage percentage (0.0-1.0).
    #[serde(default = "default_coverage")]
    pub coverage_min: f64,
}

pub(crate) fn default_pass_rate() -> f64 {
    0.95
}

pub(crate) fn default_p1_open() -> usize {
    2
}

pub(crate) fn default_max_latency() -> u64 {
    1000
}

pub(crate) fn default_coverage() -> f64 {
    0.80
}

impl Default for QualityGate {
    fn default() -> Self {
        Self {
            pass_rate: default_pass_rate(),
            p0_open: 0,
            p1_open: default_p1_open(),
            max_latency_us: default_max_latency(),
            coverage_min: default_coverage(),
        }
    }
}

impl QualityGate {
    /// Create a new quality gate with all thresholds specified.
    pub fn new(
        pass_rate: f64,
        p0_open: usize,
        p1_open: usize,
        max_latency_us: u64,
        coverage_min: f64,
    ) -> Self {
        Self {
            pass_rate,
            p0_open,
            p1_open,
            max_latency_us,
            coverage_min,
        }
    }

    /// Create an alpha (relaxed) quality gate.
    pub fn alpha() -> Self {
        Self {
            pass_rate: 0.80,
            p0_open: 0,
            p1_open: 5,
            max_latency_us: 2000,
            coverage_min: 0.60,
        }
    }

    /// Create a beta quality gate.
    pub fn beta() -> Self {
        Self {
            pass_rate: 0.90,
            p0_open: 0,
            p1_open: 2,
            max_latency_us: 1000,
            coverage_min: 0.75,
        }
    }

    /// Create an RC (release candidate) quality gate.
    pub fn rc() -> Self {
        Self {
            pass_rate: 0.98,
            p0_open: 0,
            p1_open: 0,
            max_latency_us: 500,
            coverage_min: 0.85,
        }
    }

    /// Create a GA (general availability) quality gate.
    pub fn ga() -> Self {
        Self {
            pass_rate: 1.0,
            p0_open: 0,
            p1_open: 0,
            max_latency_us: 500,
            coverage_min: 0.90,
        }
    }
}

/// Result of quality gate evaluation.
#[derive(Debug, Clone, Serialize)]
pub struct GateResult {
    /// Whether the gate passed.
    pub passed: bool,
    /// List of violations.
    pub violations: Vec<GateViolation>,
}

impl GateResult {
    /// Create a passing gate result.
    pub fn pass() -> Self {
        Self {
            passed: true,
            violations: Vec::new(),
        }
    }

    /// Create a failing gate result with the given violations.
    pub fn fail(violations: Vec<GateViolation>) -> Self {
        Self {
            passed: false,
            violations,
        }
    }
}

/// A quality gate violation.
#[derive(Debug, Clone, Serialize)]
pub struct GateViolation {
    /// Name of the violated criterion.
    pub criterion: String,
    /// Expected value.
    pub expected: String,
    /// Actual value.
    pub actual: String,
}

impl GateViolation {
    /// Create a new gate violation.
    pub fn new(
        criterion: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self {
            criterion: criterion.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}

/// Configuration file format for quality gates.
#[derive(Debug, Deserialize)]
pub(crate) struct QualityGatesConfig {
    #[serde(flatten)]
    pub gates: HashMap<String, QualityGate>,
}

/// Evaluation context containing additional metrics for gate evaluation.
#[derive(Debug, Default, Clone)]
pub struct EvaluationContext {
    /// Coverage percentage (0.0-1.0). If None, coverage check is skipped.
    pub coverage: Option<f64>,
    /// Maximum observed latency in microseconds.
    pub max_latency_us: Option<u64>,
}

/// Error loading a quality gate.
#[derive(Debug)]
pub enum GateLoadError {
    /// Failed to read config file.
    FileRead(PathBuf, String),
    /// Failed to parse config file.
    ParseError(String),
    /// Gate not found.
    GateNotFound(String),
}

impl std::fmt::Display for GateLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileRead(path, err) => {
                write!(
                    f,
                    "Failed to read config file '{}': {}",
                    path.display(),
                    err
                )
            }
            Self::ParseError(err) => write!(f, "Failed to parse config file: {}", err),
            Self::GateNotFound(name) => write!(
                f,
                "Gate '{}' not found. Available: default, alpha, beta, rc, ga",
                name
            ),
        }
    }
}

impl std::error::Error for GateLoadError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_gate_default_values() {
        let gate = QualityGate::default();
        assert!((gate.pass_rate - 0.95).abs() < f64::EPSILON);
        assert_eq!(gate.p0_open, 0);
        assert_eq!(gate.p1_open, 2);
        assert_eq!(gate.max_latency_us, 1000);
        assert!((gate.coverage_min - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn quality_gate_alpha_preset() {
        let gate = QualityGate::alpha();
        assert!((gate.pass_rate - 0.80).abs() < f64::EPSILON);
        assert_eq!(gate.p0_open, 0);
        assert_eq!(gate.p1_open, 5);
        assert_eq!(gate.max_latency_us, 2000);
        assert!((gate.coverage_min - 0.60).abs() < f64::EPSILON);
    }

    #[test]
    fn quality_gate_beta_preset() {
        let gate = QualityGate::beta();
        assert!((gate.pass_rate - 0.90).abs() < f64::EPSILON);
        assert_eq!(gate.p0_open, 0);
        assert_eq!(gate.p1_open, 2);
        assert_eq!(gate.max_latency_us, 1000);
        assert!((gate.coverage_min - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn quality_gate_rc_preset() {
        let gate = QualityGate::rc();
        assert!((gate.pass_rate - 0.98).abs() < f64::EPSILON);
        assert_eq!(gate.p0_open, 0);
        assert_eq!(gate.p1_open, 0);
        assert_eq!(gate.max_latency_us, 500);
        assert!((gate.coverage_min - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn quality_gate_ga_preset() {
        let gate = QualityGate::ga();
        assert!((gate.pass_rate - 1.0).abs() < f64::EPSILON);
        assert_eq!(gate.p0_open, 0);
        assert_eq!(gate.p1_open, 0);
        assert_eq!(gate.max_latency_us, 500);
        assert!((gate.coverage_min - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn quality_gate_new_custom() {
        let gate = QualityGate::new(0.99, 1, 3, 750, 0.85);
        assert!((gate.pass_rate - 0.99).abs() < f64::EPSILON);
        assert_eq!(gate.p0_open, 1);
        assert_eq!(gate.p1_open, 3);
        assert_eq!(gate.max_latency_us, 750);
        assert!((gate.coverage_min - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn gate_result_pass() {
        let result = GateResult::pass();
        assert!(result.passed);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn gate_result_fail() {
        let violations = vec![
            GateViolation::new("pass_rate", "95%", "90%"),
            GateViolation::new("p0_open", "0", "2"),
        ];
        let result = GateResult::fail(violations);
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 2);
    }

    #[test]
    fn gate_violation_new() {
        let violation = GateViolation::new("pass_rate", "≥95%", "90%");
        assert_eq!(violation.criterion, "pass_rate");
        assert_eq!(violation.expected, "≥95%");
        assert_eq!(violation.actual, "90%");
    }

    #[test]
    fn gate_load_error_display() {
        let err = GateLoadError::FileRead(PathBuf::from("/test.toml"), "not found".to_string());
        assert!(err.to_string().contains("/test.toml"));
        assert!(err.to_string().contains("not found"));

        let err = GateLoadError::ParseError("invalid TOML".to_string());
        assert!(err.to_string().contains("invalid TOML"));

        let err = GateLoadError::GateNotFound("unknown".to_string());
        assert!(err.to_string().contains("unknown"));
        assert!(err.to_string().contains("not found"));
    }
}
