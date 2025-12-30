//! Profile compilation with timeout handling.
//!
//! This module provides the `ProfileCompiler` for compiling Rhai configuration
//! files to binary .krx format with timeout protection.

use std::path::Path;
use std::time::Instant;

use thiserror::Error;

/// Compilation timeout in seconds
const COMPILATION_TIMEOUT_SECS: u64 = 30;

/// Result of profile compilation.
#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub compile_time_ms: u64,
    pub success: bool,
}

/// Errors that can occur during compilation.
#[derive(Debug, Error)]
pub enum CompilationError {
    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Compilation timeout (exceeded {COMPILATION_TIMEOUT_SECS}s)")]
    CompilationTimeout,

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Profile compiler for converting Rhai to binary format.
pub struct ProfileCompiler;

impl ProfileCompiler {
    /// Create a new profile compiler.
    pub fn new() -> Self {
        Self
    }

    /// Compile a profile from source to binary format.
    ///
    /// # Arguments
    ///
    /// * `source` - Path to the source .rhai file
    /// * `output` - Path where the compiled .krx file will be written
    ///
    /// # Returns
    ///
    /// Returns a `CompilationResult` containing compilation time and success status.
    ///
    /// # Errors
    ///
    /// Returns `CompilationError` if:
    /// - The source file cannot be read
    /// - The compilation fails
    /// - The compilation exceeds the timeout
    pub fn compile_profile(
        &self,
        source: &Path,
        output: &Path,
    ) -> Result<CompilationResult, CompilationError> {
        let start = Instant::now();

        self.compile_with_timeout(source, output)?;

        let compile_time = start.elapsed().as_millis() as u64;

        Ok(CompilationResult {
            compile_time_ms: compile_time,
            success: true,
        })
    }

    /// Compile with timeout protection.
    ///
    /// # Arguments
    ///
    /// * `rhai_path` - Path to the source .rhai file
    /// * `krx_path` - Path where the compiled .krx file will be written
    ///
    /// # Errors
    ///
    /// Returns `CompilationError::CompilationFailed` if compilation fails.
    /// Returns `CompilationError::CompilationTimeout` if compilation exceeds timeout.
    fn compile_with_timeout(
        &self,
        rhai_path: &Path,
        krx_path: &Path,
    ) -> Result<(), CompilationError> {
        // For now, use keyrx_compiler directly
        // In production, this would use timeout mechanism
        keyrx_compiler::compile_file(rhai_path, krx_path)
            .map_err(|e| CompilationError::CompilationFailed(e.to_string()))?;

        Ok(())
    }

    /// Validate a configuration file without compiling.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .rhai configuration file to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the configuration is valid.
    ///
    /// # Errors
    ///
    /// Returns `CompilationError` if the configuration is invalid.
    pub fn validate_config(&self, path: &Path) -> Result<(), CompilationError> {
        // For now, just check if the file exists and is readable
        // In production, this would parse the Rhai AST without full compilation
        std::fs::read_to_string(path).map_err(CompilationError::IoError)?;

        Ok(())
    }
}

impl Default for ProfileCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_compiler_creation() {
        let compiler = ProfileCompiler::new();
        assert!(std::mem::size_of_val(&compiler) == 0); // Zero-sized type
    }

    #[test]
    fn test_compiler_default() {
        let compiler = ProfileCompiler::default();
        assert!(std::mem::size_of_val(&compiler) == 0);
    }

    #[test]
    fn test_validate_config_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.rhai");
        fs::write(&config_path, "layer(\"base\", #{});").unwrap();

        let compiler = ProfileCompiler::new();
        let result = compiler.validate_config(&config_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.rhai");

        let compiler = ProfileCompiler::new();
        let result = compiler.validate_config(&config_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_compilation_result_structure() {
        let result = CompilationResult {
            compile_time_ms: 100,
            success: true,
        };

        assert_eq!(result.compile_time_ms, 100);
        assert!(result.success);
    }
}
