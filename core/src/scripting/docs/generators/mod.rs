//! Documentation generators for different output formats.
//!
//! This module provides generators that convert the documentation registry
//! into various output formats like Markdown, HTML, and JSON.

pub mod markdown;

#[cfg(test)]
mod test_integration;

pub use markdown::generate_markdown;
