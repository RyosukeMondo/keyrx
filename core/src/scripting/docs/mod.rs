//! Documentation system for Rhai API.
//!
//! This module provides automatic documentation extraction and generation
//! for Rhai functions and types exposed to scripts.

pub mod registry;
pub mod search;
pub mod types;

#[cfg(test)]
mod test_example;

pub use types::{
    FunctionDoc, FunctionSignature, ModuleDoc, ParamDoc, PropertyDoc, ReturnDoc, SearchResult,
    SearchResultKind, TypeDoc,
};

pub use search::{search, search_functions, search_in_module, search_types, SearchOptions};
