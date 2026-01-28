//! Data validation utilities for secure profile and configuration operations.
//!
//! This module provides comprehensive validation for:
//! - Profile names (alphanumeric with specific rules)
//! - File paths (preventing traversal attacks)
//! - File sizes (enforcing limits)
//! - Configuration content (syntax and security checks)
//! - Input sanitization (HTML entities, control characters)

pub mod content;
pub mod path;
pub mod profile_name;
pub mod sanitization;

use thiserror::Error;

/// Maximum file size for profile configurations (100KB)
pub const MAX_PROFILE_SIZE: u64 = 100 * 1024;

/// Maximum number of profiles allowed
pub const MAX_PROFILE_COUNT: usize = 10;

/// Validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid profile name: {0}")]
    InvalidProfileName(String),

    #[error("Path traversal attempt detected: {0}")]
    PathTraversal(String),

    #[error("File size exceeds limit: {actual} bytes (max {limit})")]
    FileSizeTooLarge { actual: u64, limit: u64 },

    #[error("Too many profiles: {actual} (max {limit})")]
    TooManyProfiles { actual: usize, limit: usize },

    #[error("Invalid configuration content: {0}")]
    InvalidContent(String),

    #[error("Malicious code pattern detected: {0}")]
    MaliciousPattern(String),

    #[error("Invalid binary format: {0}")]
    InvalidBinaryFormat(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type ValidationResult<T> = Result<T, ValidationError>;
