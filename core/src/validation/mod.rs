//! Script validation and safety analysis.
//!
//! This module provides comprehensive validation for KeyRx scripts:
//! - Semantic validation (key names, layer/modifier references)
//! - Conflict detection (duplicate remaps, circular dependencies)
//! - Safety analysis (dangerous patterns, emergency exit protection)
//! - Coverage reporting (which keys are affected)

pub mod config;
pub mod conflicts;
pub mod coverage;
pub mod detectors;
pub mod engine;
pub mod safety;
pub mod semantic;
pub mod suggestions;
pub mod types;

pub use config::ValidationConfig;
pub use conflicts::{detect_remap_conflicts, ConflictDetector};
pub use coverage::{analyze_coverage, render_ascii_keyboard, CoverageAnalyzer};
pub use engine::{
    collect_definitions, find_operation_line, LocatedOp, ParsedScript, ScriptContext,
    ValidationEngine,
};
pub use safety::{analyze_safety, SafetyAnalyzer};
pub use semantic::{validate_operations, validate_timing, SemanticValidator};
pub use suggestions::suggest_similar_keys;
pub use types::{
    CoverageReport, LayerCoverage, SourceLocation, ValidationError, ValidationOptions,
    ValidationResult, ValidationWarning, WarningCategory,
};
