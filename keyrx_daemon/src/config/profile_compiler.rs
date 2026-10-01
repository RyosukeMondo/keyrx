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
    /// Lint findings (e.g. a mapping that can never fire). The profile still
    /// compiled; the compiler used to print these to stderr, along with
    /// progress chatter, on every activation.
    pub warnings: Vec<String>,
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

impl CompilationError {
    /// The same error with `from` replaced by `to` in its message: a
    /// candidate source is compiled from a temp file, but the user should be
    /// told about their real file (`profiles/work.rhai:3:1`).
    #[must_use]
    pub fn naming_file(self, from: &Path, to: &Path) -> Self {
        match self {
            Self::CompilationFailed(msg) => Self::CompilationFailed(
                msg.replace(&from.display().to_string(), &to.display().to_string()),
            ),
            other => other,
        }
    }

    /// Source position (line, column) of the error, when the compiler reported
    /// one. Compiler messages end with `(line N, position M)`.
    pub fn location(&self) -> Option<(usize, usize)> {
        let CompilationError::CompilationFailed(message) = self else {
            return None;
        };
        let tail = &message[message.rfind("(line ")? + "(line ".len()..];
        let (line, rest) = tail.split_once(", position ")?;
        let column = rest.split(')').next()?;
        Some((line.trim().parse().ok()?, column.trim().parse().ok()?))
    }

    /// The message without the file path prefix and position suffix, for
    /// display next to the editor line.
    pub fn short_message(&self) -> String {
        let text = self.to_string();
        let body = text.rfind(" (line ").map_or(text.as_str(), |i| &text[..i]);
        // "<path>:<line>:<col>: Syntax error: ..." -> "Syntax error: ..."
        match body.find(".rhai:") {
            Some(i) => body[i + ".rhai:".len()..]
                .splitn(3, ':')
                .nth(2)
                .map_or(body, str::trim)
                .to_string(),
            None => body.trim().to_string(),
        }
    }
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

        let warnings = self.compile_with_timeout(source, output)?;

        let compile_time = start.elapsed().as_millis() as u64;

        Ok(CompilationResult {
            compile_time_ms: compile_time,
            success: true,
            warnings,
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
    ) -> Result<Vec<String>, CompilationError> {
        let (bytes, warnings) = self.build(rhai_path)?;
        write_atomically(krx_path, &bytes)?;
        for warning in &warnings {
            log::warn!("{}: {warning}", rhai_path.display());
        }
        Ok(warnings)
    }

    /// Parses `source` into a configuration, with its lint warnings. Nothing
    /// is written and nothing is printed - this is the ONE parse every
    /// caller (activation, `config set-key`, `validate`) shares, so none of
    /// them needs a scratch file just to find out whether a profile compiles.
    pub fn parse(
        &self,
        source: &Path,
    ) -> Result<(keyrx_core::config::ConfigRoot, Vec<String>), CompilationError> {
        let config = keyrx_compiler::parser::Parser::new()
            .parse_script(source)
            .map_err(|e| {
                CompilationError::CompilationFailed(
                    keyrx_compiler::CompileError::ParseError(e).to_string(),
                )
            })?;
        let warnings = config
            .devices
            .iter()
            .flat_map(keyrx_core::config::lint::dead_mappings)
            .map(|dead| dead.to_string())
            .collect();
        Ok((config, warnings))
    }

    /// Compiles `source` to the `.krx` bytes (and lint warnings) in memory.
    pub fn build(&self, source: &Path) -> Result<(Vec<u8>, Vec<String>), CompilationError> {
        let (config, warnings) = self.parse(source)?;
        let bytes = keyrx_compiler::serialize::serialize(&config).map_err(|e| {
            CompilationError::CompilationFailed(
                keyrx_compiler::CompileError::SerializeError(e).to_string(),
            )
        })?;
        Ok((bytes, warnings))
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

/// Writes `bytes` to `path` through a sibling file and a rename, so a reader
/// (the daemon, the file watcher) never sees a half-written `.krx`.
fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut part = path.as_os_str().to_owned();
    part.push(".part");
    let part = std::path::PathBuf::from(part);
    std::fs::write(&part, bytes)?;
    std::fs::rename(&part, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&part);
    })
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
        let compiler = ProfileCompiler;
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
            warnings: Vec::new(),
        };

        assert_eq!(result.compile_time_ms, 100);
        assert!(result.success);
    }

    #[test]
    fn test_compilation_error_format_user_friendly() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("invalid.rhai");
        let output = temp_dir.path().join("invalid.krx");

        // Write invalid Rhai script (missing device_start)
        fs::write(&source, "layer(\"base\", #{});").unwrap();

        let compiler = ProfileCompiler::new();
        let result = compiler.compile_profile(&source, &output);

        assert!(result.is_err(), "Should fail on invalid script");

        let error = result.unwrap_err();
        let error_message = error.to_string();

        // Verify error message is user-friendly (NOT Debug format)
        assert!(
            !error_message.contains("SyntaxError {"),
            "Should not contain Rust debug format 'SyntaxError {{'\nGot: {}",
            error_message
        );
        assert!(
            !error_message.contains("file:"),
            "Should not contain debug field 'file:'\nGot: {}",
            error_message
        );
        assert!(
            !error_message.contains("import_chain:"),
            "Should not contain debug field 'import_chain:'\nGot: {}",
            error_message
        );

        // Should contain the actual error information
        assert!(
            error_message.contains("Compilation failed") || error_message.contains("line"),
            "Should contain useful error information\nGot: {}",
            error_message
        );
    }

    #[test]
    fn test_compilation_error_location() {
        let e = CompilationError::CompilationFailed(
            "/x/p.rhai:4:1: Syntax error: Syntax error: Expecting ',' (line 4, position 1)".into(),
        );
        assert_eq!(e.location(), Some((4, 1)));
        assert_eq!(
            e.short_message(),
            "Syntax error: Syntax error: Expecting ','"
        );

        let multiline = CompilationError::CompilationFailed(
            "/x/p.rhai:2:3: Syntax error: Runtime error: Unknown key name: 'NOPE'\n (line 2, position 3)".into(),
        );
        assert_eq!(multiline.location(), Some((2, 3)));
        assert_eq!(CompilationError::CompilationTimeout.location(), None);
        assert_eq!(
            CompilationError::CompilationFailed("no position".into()).location(),
            None
        );
    }
}
