//! Dart Binding Code Generator Library
//!
//! This library provides functionality for generating type-safe Dart FFI bindings
//! from JSON contracts.

pub mod cli;
pub mod loader;
pub mod type_mapper;
pub mod types;

#[cfg(test)]
mod type_mapper_tests;
