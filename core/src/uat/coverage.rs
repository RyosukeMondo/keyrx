//! Requirements coverage mapping and reporting.

use std::collections::HashMap;

/// Coverage status for a requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageStatus {
    /// All linked tests pass.
    Verified,
    /// Some linked tests fail.
    AtRisk,
    /// No linked tests.
    Uncovered,
}

/// Coverage information for a single requirement.
#[derive(Debug, Clone)]
pub struct RequirementCoverage {
    /// Requirement ID.
    pub id: String,
    /// Linked test names.
    pub linked_tests: Vec<String>,
    /// Coverage status.
    pub status: CoverageStatus,
    /// Last verification timestamp.
    pub last_verified: Option<String>,
}

/// Map of requirement IDs to their coverage.
#[derive(Debug, Clone, Default)]
pub struct CoverageMap {
    /// Mapping of requirement ID to coverage info.
    pub requirements: HashMap<String, RequirementCoverage>,
}

/// Coverage report with summary statistics.
#[derive(Debug, Clone)]
pub struct CoverageReport {
    /// Coverage map.
    pub coverage: CoverageMap,
    /// Total requirements.
    pub total: usize,
    /// Verified requirements.
    pub verified: usize,
    /// At-risk requirements.
    pub at_risk: usize,
    /// Uncovered requirements.
    pub uncovered: usize,
}

/// Mapper for building coverage information.
#[derive(Debug)]
pub struct CoverageMapper;
