//! Documentation system for Rhai API.
//!
//! This module provides automatic documentation extraction and generation
//! for Rhai functions and types exposed to scripts.

pub mod types;

pub use types::{
    FunctionDoc, FunctionSignature, ModuleDoc, ParamDoc, PropertyDoc, ReturnDoc, SearchResult,
    SearchResultKind, TypeDoc,
};
