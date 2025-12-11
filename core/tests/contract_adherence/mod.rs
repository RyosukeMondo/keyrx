// Allow test-specific lints - tests need panic/unwrap/expect for failure assertions
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    clippy::print_stderr
)]

//! Enhanced FFI Contract Adherence Validation
//!
//! This module provides comprehensive validation of FFI function signatures
//! against their JSON contract definitions using AST parsing.
//!
//! # Architecture
//!
//! The validation system consists of four main components:
//!
//! - **Parser** (`parser.rs`): Uses `syn` to parse Rust source files and extract
//!   `extern "C"` functions with `#[no_mangle]` attribute
//! - **Type Mapper** (`type_mapper.rs`): Maps contract type strings (e.g., "string",
//!   "int32") to expected Rust FFI types (e.g., `*const c_char`, `i32`)
//! - **Validator** (`validator.rs`): Compares parsed functions against contracts,
//!   detecting mismatches in parameter counts, types, and return types
//! - **Reporter** (`reporter.rs`): Generates human-readable error reports with
//!   file locations and fix suggestions
//!
//! # Usage
//!
//! ```rust,ignore
//! use contract_adherence::parser::parse_ffi_exports;
//! use contract_adherence::validator::validate_all_functions;
//! use contract_adherence::reporter::generate_full_report;
//!
//! // Parse FFI exports from source file
//! let functions = parse_ffi_exports(Path::new("src/ffi/exports.rs"))?;
//!
//! // Load contracts and validate
//! let report = validate_all_functions(&contracts, &functions);
//!
//! if !report.is_success() {
//!     println!("{}", generate_full_report(&report));
//! }
//! ```
//!
//! # Type Mapping
//!
//! | Contract Type | Rust FFI Type |
//! |--------------|---------------|
//! | `void` | `()` |
//! | `bool` | `bool` |
//! | `int`, `int32` | `i32` |
//! | `string` | `*const c_char` or `*mut c_char` |
//! | `object`, `array` | `*const c_char` (JSON) |
//!
//! See `type_mapper.rs` for the complete mapping table.
//!
//! # Validation Errors
//!
//! The validator detects the following error types:
//!
//! - `MissingFunction`: Contract function not found in source
//! - `UncontractedFunction`: Source function not in any contract
//! - `ParameterCountMismatch`: Different number of parameters
//! - `ParameterTypeMismatch`: Parameter type doesn't match
//! - `ReturnTypeMismatch`: Return type doesn't match
//!
//! See `README.md` for detailed examples and fix instructions.

pub mod parser;
pub mod reporter;
pub mod type_mapper;
pub mod validator;

#[cfg(test)]
mod parser_tests;

#[cfg(test)]
mod type_mapper_tests;

#[cfg(test)]
mod validator_tests;
