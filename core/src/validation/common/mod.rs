//! Common validation types and utilities.
//!
//! This module contains shared types and utilities used by all validation detectors,
//! including issue reporting types and visitor patterns.

pub mod issue;
pub mod visitor;

#[cfg(test)]
pub mod test_helpers;

pub use issue::{Severity, ValidationIssue};
pub use visitor::{visit_all, OperationVisitor};
