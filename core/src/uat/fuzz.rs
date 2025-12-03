//! Fuzz testing with random key sequence generation.

/// Result of a fuzz testing run.
#[derive(Debug, Clone)]
pub struct FuzzResult {
    /// Number of sequences tested.
    pub sequences_tested: usize,
    /// Duration in seconds.
    pub duration_secs: f64,
    /// Number of unique execution paths discovered.
    pub unique_paths: usize,
    /// Crash sequences found.
    pub crashes: Vec<CrashSequence>,
}

/// A crash-inducing sequence.
#[derive(Debug, Clone)]
pub struct CrashSequence {
    /// Timestamp when crash occurred.
    pub timestamp: String,
    /// Path to saved sequence file.
    pub file_path: String,
    /// Error message.
    pub error: String,
}

/// Fuzz testing engine.
#[derive(Debug)]
pub struct FuzzEngine;
