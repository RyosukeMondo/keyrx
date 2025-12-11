//! Enhanced FFI Contract Adherence Validation
//!
//! This module provides comprehensive validation of FFI function signatures
//! against their JSON contract definitions using AST parsing.

pub mod parser;
pub mod type_mapper;
pub mod validator;

#[cfg(test)]
mod parser_tests;

#[cfg(test)]
mod type_mapper_tests;
