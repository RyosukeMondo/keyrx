//! Common validation types and utilities.
//!
//! This module contains shared types and utilities used by all validation detectors,
//! including issue reporting types and visitor patterns.

pub mod issue;

pub use issue::{Severity, ValidationIssue};
