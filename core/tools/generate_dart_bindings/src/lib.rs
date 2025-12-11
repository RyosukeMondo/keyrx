//! Dart Binding Code Generator Library
//!
//! This library provides functionality for generating type-safe Dart FFI bindings
//! from JSON contracts.

pub mod bindings_gen;
pub mod cli;
pub mod loader;
pub mod templates;
pub mod type_mapper;
pub mod types;

#[cfg(test)]
mod type_mapper_tests;
