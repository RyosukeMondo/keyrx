//! Migration module for converting between profile system versions.
//!
//! This module provides utilities for migrating user data from older
//! profile systems to the revolutionary mapping system.

pub mod v1_to_v2;

pub use v1_to_v2::MigrationV1ToV2;

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during migration
#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("IO error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse old profile at {path}: {source}")]
    ParseError {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("Failed to save new profile: {0}")]
    SaveError(String),

    #[error("Backup failed: {0}")]
    BackupError(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),
}

/// Result of a migration operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MigrationReport {
    /// Number of profiles successfully migrated
    pub migrated_count: usize,

    /// Number of profiles that failed to migrate
    pub failed_count: usize,

    /// Total number of profiles found
    pub total_count: usize,

    /// Details of failed migrations
    pub failures: Vec<MigrationFailure>,

    /// Path to backup directory
    pub backup_path: Option<PathBuf>,
}

/// Details of a failed migration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MigrationFailure {
    /// Path to the profile that failed
    pub path: PathBuf,

    /// Error message
    pub error: String,
}

impl MigrationReport {
    /// Create a new empty migration report
    pub fn new() -> Self {
        Self {
            migrated_count: 0,
            failed_count: 0,
            total_count: 0,
            failures: Vec::new(),
            backup_path: None,
        }
    }

    /// Check if migration was fully successful
    pub fn is_success(&self) -> bool {
        self.failed_count == 0 && self.total_count > 0
    }

    /// Check if migration was partial (some successes, some failures)
    pub fn is_partial(&self) -> bool {
        self.migrated_count > 0 && self.failed_count > 0
    }

    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }
        (self.migrated_count as f64 / self.total_count as f64) * 100.0
    }

    /// Generate a summary report as a string
    pub fn summary(&self) -> String {
        let mut summary = format!(
            "Migration Summary:\n\
             Total profiles: {}\n\
             Migrated: {}\n\
             Failed: {}\n\
             Success rate: {:.1}%\n",
            self.total_count,
            self.migrated_count,
            self.failed_count,
            self.success_rate()
        );

        if let Some(backup) = &self.backup_path {
            summary.push_str(&format!("Backup created at: {}\n", backup.display()));
        }

        if !self.failures.is_empty() {
            summary.push_str("\nFailures:\n");
            for failure in &self.failures {
                summary.push_str(&format!(
                    "  - {}: {}\n",
                    failure.path.display(),
                    failure.error
                ));
            }
        }

        summary
    }
}

impl Default for MigrationReport {
    fn default() -> Self {
        Self::new()
    }
}
