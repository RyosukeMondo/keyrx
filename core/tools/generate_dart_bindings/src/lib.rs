//! Dart Binding Code Generator Library
//!
//! This library provides functionality for generating type-safe Dart FFI bindings
//! from JSON contracts.

pub mod bindings_gen;
pub mod cli;
pub mod header;
pub mod loader;
pub mod models_gen;
pub mod templates;
pub mod type_mapper;
pub mod types;
pub mod writer;

#[cfg(test)]
mod type_mapper_tests;
