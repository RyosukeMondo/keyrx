//! Generation pipeline orchestration
//!
//! Connects all components: load → generate → write → format

use anyhow::{Context, Result};
use generate_dart_bindings::{
    bindings_gen::{
        generate_ffi_signatures, generate_function_pointers_block, generate_typedefs_block,
        generate_wrapper_functions, generate_wrappers_block,
    },
    cli::Cli,
    formatter::{format_file, is_dart_available, FormatResult},
    header::{generate_bindings_header, generate_models_header},
    loader::{load_all_contracts, load_contracts_for_domain},
    models_gen::{generate_all_models, generate_models_block},
    writer::{write_if_changed, WriteResult},
};
use keyrx_core::ffi::contract::FfiContract;
use std::path::PathBuf;

/// Result of the generation pipeline
#[derive(Debug, Default)]
pub struct PipelineResult {
    pub bindings_generated: usize,
    pub models_generated: usize,
    pub files_written: usize,
    pub files_skipped: usize,
    pub needs_regeneration: bool,
}

/// Generation pipeline that orchestrates all components
pub struct GenerationPipeline<'a> {
    cli: &'a Cli,
}

impl<'a> GenerationPipeline<'a> {
    pub fn new(cli: &'a Cli) -> Self {
        Self { cli }
    }

    /// Run the full generation pipeline
    pub fn run(&self) -> Result<PipelineResult> {
        let mut result = PipelineResult::default();

        // Step 1: Load contracts
        let contracts = self.load_contracts()?;
        if self.cli.verbose {
            eprintln!("Loaded {} contract(s)", contracts.len());
        }

        // Step 2: Generate code for each contract
        let (bindings_code, models_code) = self.generate_code(&contracts)?;

        // Step 3: Write files (unless in check mode)
        let output_dir = self.cli.output_dir();
        let bindings_path = output_dir.join("ffi/generated_bindings.dart");
        let models_path = output_dir.join("models/generated_models.dart");

        if self.cli.check {
            // Check mode: verify files match without writing
            result.needs_regeneration = self
                .check_needs_regeneration(&bindings_path, &bindings_code)?
                || self.check_needs_regeneration(&models_path, &models_code)?;
            result.bindings_generated = 1;
            result.models_generated = 1;
        } else {
            // Write mode: write files and format
            let (written, skipped) =
                self.write_and_format(&bindings_path, &bindings_code, &models_path, &models_code)?;
            result.files_written = written;
            result.files_skipped = skipped;
            result.bindings_generated = 1;
            result.models_generated = 1;
        }

        Ok(result)
    }

    /// Load contracts based on CLI options
    fn load_contracts(&self) -> Result<Vec<FfiContract>> {
        let contracts_dir = self.cli.contracts_dir();

        if let Some(domain) = &self.cli.domain {
            load_contracts_for_domain(&contracts_dir, domain)
                .with_context(|| format!("Failed to load contracts for domain: {domain}"))
        } else {
            load_all_contracts(&contracts_dir).with_context(|| {
                format!("Failed to load contracts from: {}", contracts_dir.display())
            })
        }
    }

    /// Generate bindings and models code from contracts
    fn generate_code(&self, contracts: &[FfiContract]) -> Result<(String, String)> {
        let mut bindings_parts = Vec::new();
        let mut models_parts = Vec::new();

        // Add file headers
        bindings_parts.push(generate_bindings_header());
        models_parts.push(generate_models_header());

        // Generate code for each contract
        for contract in contracts {
            if self.cli.verbose {
                eprintln!("Processing contract: {}", contract.domain);
            }

            // Generate FFI bindings
            let bindings = self.generate_contract_bindings(contract)?;
            bindings_parts.push(bindings);

            // Generate models
            let models = self.generate_contract_models(contract)?;
            if !models.is_empty() {
                models_parts.push(models);
            }
        }

        Ok((bindings_parts.join("\n\n"), models_parts.join("\n\n")))
    }

    /// Generate FFI bindings for a single contract
    fn generate_contract_bindings(&self, contract: &FfiContract) -> Result<String> {
        let mut output = Vec::new();

        // Generate typedefs at file level (outside classes - required by Dart)
        let signatures = generate_ffi_signatures(contract)
            .map_err(|e| anyhow::anyhow!("Failed to generate FFI signatures: {e}"))?;

        output.push(format!(
            "// ============================================================================="
        ));
        output.push(format!(
            "// {} Domain Bindings",
            to_pascal_case(&contract.domain)
        ));
        output.push(format!(
            "// ============================================================================="
        ));
        output.push(String::new());

        // Typedefs must be at top level (not inside class)
        output.push(format!("// Typedefs for {} domain", contract.domain));
        output.push(generate_typedefs_block(&signatures));

        // Generate class wrapper for the domain
        let class_name = format!("{}Bindings", to_pascal_case(&contract.domain));
        output.push(format!(
            "/// FFI bindings for the {} domain",
            contract.domain
        ));
        output.push(format!("class {class_name} {{"));
        output.push("  final DynamicLibrary _lib;".to_string());
        output.push(String::new());
        output.push(format!("  {class_name}(this._lib);"));
        output.push(String::new());
        // Add free string function pointer (needed for memory cleanup)
        output.push("  // Free string function for memory cleanup".to_string());
        output.push("  late final _keyrx_free_string = _lib".to_string());
        output.push(
            "      .lookup<NativeFunction<Void Function(Pointer<Utf8>)>>('keyrx_free_string')"
                .to_string(),
        );
        output.push("      .asFunction<void Function(Pointer<Utf8>)>();".to_string());
        output.push(String::new());

        // Generate function pointers (inside class)
        output.push(generate_function_pointers_block(&signatures));

        // Generate wrapper functions
        let wrappers = generate_wrapper_functions(contract)
            .map_err(|e| anyhow::anyhow!("Failed to generate wrapper functions: {e}"))?;
        output.push(generate_wrappers_block(&wrappers));

        output.push("}".to_string());

        Ok(output.join("\n"))
    }

    /// Generate model classes for a single contract
    fn generate_contract_models(&self, contract: &FfiContract) -> Result<String> {
        let models = generate_all_models(contract)
            .map_err(|e| anyhow::anyhow!("Failed to generate models: {e}"))?;

        if models.is_empty() {
            return Ok(String::new());
        }

        Ok(generate_models_block(&models))
    }

    /// Check if a file needs regeneration (for --check mode)
    ///
    /// This writes to a temp file, formats it, then compares with existing.
    fn check_needs_regeneration(&self, path: &PathBuf, content: &str) -> Result<bool> {
        if !path.exists() {
            return Ok(true);
        }

        // Write to temp file
        let temp_dir = tempfile::tempdir().with_context(|| "Failed to create temp directory")?;
        let temp_path = temp_dir.path().join("check.dart");
        std::fs::write(&temp_path, content).with_context(|| "Failed to write temp file")?;

        // Format the temp file
        if is_dart_available() {
            let _ = format_file(&temp_path);
        }

        // Read formatted content
        let formatted = std::fs::read_to_string(&temp_path)
            .with_context(|| "Failed to read formatted temp file")?;

        let existing = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read existing file: {}", path.display()))?;

        // Compare ignoring timestamp line (which changes on each generation)
        let existing_normalized = normalize_for_comparison(&existing);
        let new_normalized = normalize_for_comparison(&formatted);

        Ok(existing_normalized != new_normalized)
    }

    /// Write files and format them
    fn write_and_format(
        &self,
        bindings_path: &PathBuf,
        bindings_code: &str,
        models_path: &PathBuf,
        models_code: &str,
    ) -> Result<(usize, usize)> {
        let mut written = 0;
        let mut skipped = 0;

        // Write bindings file
        match write_if_changed(bindings_path, bindings_code)
            .with_context(|| format!("Failed to write bindings: {}", bindings_path.display()))?
        {
            WriteResult::Written => {
                written += 1;
                if self.cli.verbose {
                    eprintln!("Wrote: {}", bindings_path.display());
                }
            }
            WriteResult::Skipped => {
                skipped += 1;
                if self.cli.verbose {
                    eprintln!("Skipped (unchanged): {}", bindings_path.display());
                }
            }
        }

        // Write models file
        match write_if_changed(models_path, models_code)
            .with_context(|| format!("Failed to write models: {}", models_path.display()))?
        {
            WriteResult::Written => {
                written += 1;
                if self.cli.verbose {
                    eprintln!("Wrote: {}", models_path.display());
                }
            }
            WriteResult::Skipped => {
                skipped += 1;
                if self.cli.verbose {
                    eprintln!("Skipped (unchanged): {}", models_path.display());
                }
            }
        }

        // Format files if dart is available
        if is_dart_available() {
            self.format_generated_files(bindings_path, models_path);
        } else if self.cli.verbose {
            eprintln!("Warning: dart not found, skipping formatting");
        }

        Ok((written, skipped))
    }

    /// Format generated files with dart format
    fn format_generated_files(&self, bindings_path: &PathBuf, models_path: &PathBuf) {
        for path in [bindings_path, models_path] {
            if path.exists() {
                match format_file(path) {
                    FormatResult::Formatted => {
                        if self.cli.verbose {
                            eprintln!("Formatted: {}", path.display());
                        }
                    }
                    FormatResult::Skipped => {
                        if self.cli.verbose {
                            eprintln!("Format skipped: {}", path.display());
                        }
                    }
                    FormatResult::Failed(msg) => {
                        eprintln!("Warning: Failed to format {}: {msg}", path.display());
                    }
                }
            }
        }
    }
}

/// Convert snake_case to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Normalize content for comparison by removing timestamp lines and normalizing whitespace
fn normalize_for_comparison(content: &str) -> String {
    content
        .lines()
        .filter(|line| !line.contains("Generation time:"))
        // Normalize whitespace: trim lines and collapse multiple spaces
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        // Remove empty lines from comparison
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
