//! Script validation and safety analysis.
//!
//! This module provides comprehensive validation for KeyRx scripts:
//! - Semantic validation (key names, layer/modifier references)
//! - Conflict detection (duplicate remaps, circular dependencies)
//! - Safety analysis (dangerous patterns, emergency exit protection)
//! - Coverage reporting (which keys are affected)

pub mod common;
pub mod config;
pub mod conflicts;
pub mod coverage;
pub mod detectors;
pub mod engine;
pub mod orchestrator;
pub mod safety;
pub mod schema;
pub mod semantic;
pub mod suggestions;
#[cfg(test)]
mod tests;
pub mod types;

pub use config::ValidationConfig;
pub use conflicts::{
    detect_circular_remaps, detect_combo_shadowing, detect_remap_conflicts, ConflictDetector,
};
pub use coverage::{analyze_coverage, render_ascii_keyboard, CoverageAnalyzer};
pub use engine::{
    collect_definitions, find_operation_line, LocatedOp, ParsedScript, ScriptContext,
    ValidationEngine,
};
pub use orchestrator::{DetectorOrchestrator, NamedDetectorStats, ValidationReport};
pub use safety::{analyze_safety, SafetyAnalyzer};
pub use schema::{
    SchemaError, SchemaRegistry, ValidationFailure, ValidationIssue, CONFIG_SCHEMA_NAME,
    DEVICE_PROFILE_SCHEMA_NAME,
};
pub use semantic::{validate_operations, validate_timing, SemanticValidator};
pub use suggestions::suggest_similar_keys;
pub use types::{
    CoverageReport, LayerCoverage, SourceLocation, ValidationError, ValidationOptions,
    ValidationResult, ValidationWarning, WarningCategory,
};
