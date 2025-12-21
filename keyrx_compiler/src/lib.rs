//! KeyRx Compiler Library
//!
//! This library provides the compilation infrastructure for KeyRx configuration files.
//! It parses Rhai DSL scripts and compiles them to binary .krx format.

pub mod error;
pub mod import_resolver;
pub mod parser;
pub mod serialize;

// Modules that will be implemented later
// pub mod mphf_gen;
// pub mod dfa_gen;
