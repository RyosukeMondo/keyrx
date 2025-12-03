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
pub mod engine;
pub mod safety;
pub mod semantic;
pub mod suggestions;
pub mod types;

pub use config::ValidationConfig;
pub use types::{
    CoverageReport, LayerCoverage, SourceLocation, ValidationError, ValidationOptions,
    ValidationResult, ValidationWarning, WarningCategory,
};
