//! User Acceptance Testing (UAT) framework for KeyRx.
//!
//! This module provides comprehensive UAT capabilities including:
//! - Test discovery and execution
//! - Golden session recording and verification
//! - Quality gate enforcement
//! - Coverage mapping and reporting
//! - Performance testing
//! - Fuzz testing

mod coverage;
mod fuzz;
mod gates;
mod golden;
mod golden_comparison;
mod golden_types;
mod perf;
mod perf_analysis;
mod perf_runner;
mod perf_types;
mod report;
mod report_data;
mod report_html;
mod report_html_sections;
mod report_html_styles;
mod report_markdown;
mod runner;

pub use coverage::{CoverageMap, CoverageMapper, CoverageReport, RequirementCoverage};
pub use fuzz::{CrashSequence, FuzzConfig, FuzzEngine, FuzzEvent, FuzzResult, FuzzSequence};
pub use gates::{
    EvaluationContext, GateLoadError, GateResult, GateViolation, QualityGate, QualityGateEnforcer,
};
pub use golden::{
    DifferenceType, ExpectedOutput, GoldenDifference, GoldenEvent, GoldenEventType, GoldenSession,
    GoldenSessionError, GoldenSessionManager, GoldenSessionMetadata, GoldenVerifyResult,
    RecordResult, UpdateResult, GOLDEN_SESSION_VERSION,
};
pub use perf::{
    BaselineData, BaselineError, BaselineRegression, BaselineTestData, LatencyPercentiles,
    LatencyViolation, PerfComparison, PerfResults, PerformanceResult, PerformanceUat,
};
pub use report::{CategoryStats, ReportData, ReportGenerator};
pub use runner::{Priority, UatFilter, UatResult, UatResults, UatRunner, UatTest};
