//! Search functionality for documentation.
//!
//! This module provides text search capabilities over the documentation registry,
//! with relevance scoring to rank results.

use super::registry;
use super::types::{FunctionDoc, SearchResult, SearchResultKind, TypeDoc};

/// Search configuration and options.
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Maximum number of results to return
    pub max_results: usize,

    /// Minimum relevance score to include (0.0 to 1.0)
    pub min_score: f64,

    /// Whether to search in function names
    pub search_functions: bool,

    /// Whether to search in type names
    pub search_types: bool,

    /// Whether to search in descriptions
    pub search_descriptions: bool,

    /// Whether to perform case-sensitive search
    pub case_sensitive: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            max_results: 50,
            min_score: 0.0,
            search_functions: true,
            search_types: true,
            search_descriptions: true,
            case_sensitive: false,
        }
    }
}

/// Search the documentation for items matching the query.
///
/// # Arguments
/// * `query` - The search term to look for
/// * `options` - Search configuration options
///
/// # Returns
/// A vector of search results sorted by relevance (highest first)
///
/// # Example
/// ```no_run
/// # use keyrx_core::scripting::docs::search::{search, SearchOptions};
/// let results = search("emit", SearchOptions::default());
/// for result in results {
///     println!("{}: {} (score: {})", result.name, result.description, result.score);
/// }
/// ```
pub fn search(query: &str, options: SearchOptions) -> Vec<SearchResult> {
    if query.is_empty() {
        return vec![];
    }

    let mut results = Vec::new();

    // Prepare query for comparison
    let normalized_query = if options.case_sensitive {
        query.to_string()
    } else {
        query.to_lowercase()
    };

    // Search functions
    if options.search_functions {
        for func in registry::all_functions() {
            if let Some(result) = search_function(&func, &normalized_query, &options) {
                if result.score >= options.min_score {
                    results.push(result);
                }
            }
        }
    }

    // Search types
    if options.search_types {
        for type_doc in registry::all_types() {
            if let Some(result) = search_type(&type_doc, &normalized_query, &options) {
                if result.score >= options.min_score {
                    results.push(result);
                }
            }
        }
    }

    // Sort by score (highest first)
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Limit results
    results.truncate(options.max_results);

    results
}

/// Search function documentation for a match.
fn search_function(
    func: &FunctionDoc,
    query: &str,
    options: &SearchOptions,
) -> Option<SearchResult> {
    let mut score: f64 = 0.0;
    let mut matched = false;

    // Normalize strings based on case sensitivity
    let name = if options.case_sensitive {
        func.name.clone()
    } else {
        func.name.to_lowercase()
    };

    let description = if options.case_sensitive {
        func.description.clone()
    } else {
        func.description.to_lowercase()
    };

    // Exact name match - highest score
    if name == query {
        score = 1.0;
        matched = true;
    }
    // Name starts with query - very high score
    else if name.starts_with(query) {
        score = 0.9;
        matched = true;
    }
    // Name contains query - high score
    else if name.contains(query) {
        score = 0.7;
        matched = true;
    }
    // Description contains query - medium score
    else if options.search_descriptions && description.contains(query) {
        score = 0.5;
        matched = true;
    }
    // Check parameters for matches - lower score
    else {
        for param in &func.parameters {
            let param_name = if options.case_sensitive {
                param.name.clone()
            } else {
                param.name.to_lowercase()
            };

            let param_desc = if options.case_sensitive {
                param.description.clone()
            } else {
                param.description.to_lowercase()
            };

            if param_name.contains(query) || param_desc.contains(query) {
                score = 0.3;
                matched = true;
                break;
            }
        }
    }

    if matched {
        // Boost score for non-deprecated items
        if func.deprecated.is_none() {
            score *= 1.1;
            score = score.min(1.0_f64); // Cap at 1.0
        } else {
            score *= 0.8;
        }

        Some(SearchResult {
            kind: SearchResultKind::Function,
            name: func.name.clone(),
            module: func.module.clone(),
            description: func.description.clone(),
            score,
        })
    } else {
        None
    }
}

/// Search type documentation for a match.
fn search_type(type_doc: &TypeDoc, query: &str, options: &SearchOptions) -> Option<SearchResult> {
    let mut score: f64 = 0.0;
    let mut matched = false;

    // Normalize strings based on case sensitivity
    let name = if options.case_sensitive {
        type_doc.name.clone()
    } else {
        type_doc.name.to_lowercase()
    };

    let description = if options.case_sensitive {
        type_doc.description.clone()
    } else {
        type_doc.description.to_lowercase()
    };

    // Exact name match - highest score
    if name == query {
        score = 1.0;
        matched = true;
    }
    // Name starts with query - very high score
    else if name.starts_with(query) {
        score = 0.9;
        matched = true;
    }
    // Name contains query - high score
    else if name.contains(query) {
        score = 0.7;
        matched = true;
    }
    // Description contains query - medium score
    else if options.search_descriptions && description.contains(query) {
        score = 0.5;
        matched = true;
    }
    // Check properties for matches - lower score
    else {
        for prop in &type_doc.properties {
            let prop_name = if options.case_sensitive {
                prop.name.clone()
            } else {
                prop.name.to_lowercase()
            };

            let prop_desc = if options.case_sensitive {
                prop.description.clone()
            } else {
                prop.description.to_lowercase()
            };

            if prop_name.contains(query) || prop_desc.contains(query) {
                score = 0.3;
                matched = true;
                break;
            }
        }

        // Check methods if properties didn't match
        if !matched {
            for method in &type_doc.methods {
                let method_name = if options.case_sensitive {
                    method.name.clone()
                } else {
                    method.name.to_lowercase()
                };

                if method_name.contains(query) {
                    score = 0.4;
                    matched = true;
                    break;
                }
            }
        }
    }

    if matched {
        Some(SearchResult {
            kind: SearchResultKind::Type,
            name: type_doc.name.clone(),
            module: type_doc.module.clone(),
            description: type_doc.description.clone(),
            score,
        })
    } else {
        None
    }
}

/// Search for functions by module.
///
/// # Arguments
/// * `module` - The module name to filter by
/// * `query` - The search term (optional)
///
/// # Returns
/// A vector of search results for functions in the specified module
pub fn search_in_module(module: &str, query: Option<&str>) -> Vec<SearchResult> {
    let functions = registry::functions_in_module(module);
    let types = registry::types_in_module(module);

    let mut results = Vec::new();

    if let Some(q) = query {
        // Search with query
        let options = SearchOptions::default();
        let normalized_query = if options.case_sensitive {
            q.to_string()
        } else {
            q.to_lowercase()
        };

        for func in functions {
            if let Some(result) = search_function(&func, &normalized_query, &options) {
                results.push(result);
            }
        }

        for type_doc in types {
            if let Some(result) = search_type(&type_doc, &normalized_query, &options) {
                results.push(result);
            }
        }

        // Sort by score
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        // Return all items in module
        for func in functions {
            results.push(SearchResult {
                kind: SearchResultKind::Function,
                name: func.name.clone(),
                module: func.module.clone(),
                description: func.description.clone(),
                score: 1.0,
            });
        }

        for type_doc in types {
            results.push(SearchResult {
                kind: SearchResultKind::Type,
                name: type_doc.name.clone(),
                module: type_doc.module.clone(),
                description: type_doc.description.clone(),
                score: 1.0,
            });
        }

        // Sort alphabetically
        results.sort_by(|a, b| a.name.cmp(&b.name));
    }

    results
}

/// Search for functions by name (convenience function).
///
/// # Arguments
/// * `query` - The search term
///
/// # Returns
/// A vector of function search results
pub fn search_functions(query: &str) -> Vec<SearchResult> {
    search(
        query,
        SearchOptions {
            search_types: false,
            ..Default::default()
        },
    )
}

/// Search for types by name (convenience function).
///
/// # Arguments
/// * `query` - The search term
///
/// # Returns
/// A vector of type search results
pub fn search_types(query: &str) -> Vec<SearchResult> {
    search(
        query,
        SearchOptions {
            search_functions: false,
            ..Default::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::docs::types::{FunctionSignature, ParamDoc, PropertyDoc, ReturnDoc};
    use serial_test::serial;

    fn create_test_function(name: &str, module: &str, description: &str) -> FunctionDoc {
        FunctionDoc {
            name: name.to_string(),
            module: module.to_string(),
            signature: FunctionSignature {
                params: vec![("key".to_string(), "KeyCode".to_string())],
                return_type: Some("()".to_string()),
            },
            description: description.to_string(),
            parameters: vec![ParamDoc {
                name: "key".to_string(),
                type_name: "KeyCode".to_string(),
                description: "The key to use".to_string(),
                optional: false,
                default: None,
            }],
            returns: Some(ReturnDoc {
                type_name: "()".to_string(),
                description: "Nothing".to_string(),
            }),
            examples: vec![],
            since: Some("0.1.0".to_string()),
            deprecated: None,
            notes: None,
        }
    }

    fn create_test_type(name: &str, module: &str, description: &str) -> TypeDoc {
        TypeDoc {
            name: name.to_string(),
            description: description.to_string(),
            methods: vec![],
            properties: vec![PropertyDoc {
                name: "value".to_string(),
                type_name: "int".to_string(),
                description: "Test property".to_string(),
                readonly: true,
            }],
            constructors: vec![],
            module: module.to_string(),
            since: Some("0.1.0".to_string()),
            examples: vec![],
        }
    }

    fn setup_test_registry() {
        registry::initialize();
        registry::clear();

        // Add test functions
        registry::register_function(create_test_function(
            "emit_key",
            "keys",
            "Emits a keyboard key press event",
        ));
        registry::register_function(create_test_function(
            "release_key",
            "keys",
            "Releases a keyboard key",
        ));
        registry::register_function(create_test_function(
            "switch_layer",
            "layers",
            "Switches to a different layer",
        ));

        // Add test types
        registry::register_type(create_test_type(
            "KeyCode",
            "keys",
            "Represents a keyboard key code",
        ));
        registry::register_type(create_test_type(
            "Layer",
            "layers",
            "Represents a keyboard layer",
        ));
    }

    #[test]
    #[serial]
    fn test_search_exact_match() {
        setup_test_registry();

        let results = search("emit_key", SearchOptions::default());
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "emit_key");
        assert_eq!(results[0].score, 1.0);
    }

    #[test]
    #[serial]
    fn test_search_partial_match() {
        setup_test_registry();

        let results = search("key", SearchOptions::default());
        assert!(results.len() >= 2); // Should match emit_key, release_key, and KeyCode

        // Should find items with "key" in name (highest scores)
        let emit_key_found = results.iter().any(|r| r.name == "emit_key");
        let release_key_found = results.iter().any(|r| r.name == "release_key");
        let keycode_found = results.iter().any(|r| r.name == "KeyCode");

        assert!(emit_key_found, "emit_key should be found");
        assert!(release_key_found, "release_key should be found");
        assert!(keycode_found, "KeyCode should be found");
    }

    #[test]
    #[serial]
    fn test_search_case_insensitive() {
        setup_test_registry();

        let results = search("EMIT", SearchOptions::default());
        assert!(!results.is_empty());

        let first_result = &results[0];
        assert_eq!(first_result.name, "emit_key");
    }

    #[test]
    #[serial]
    fn test_search_case_sensitive() {
        setup_test_registry();

        let options = SearchOptions {
            case_sensitive: true,
            ..Default::default()
        };

        let results = search("EMIT", options);
        assert!(results.is_empty()); // Should not match "emit_key"
    }

    #[test]
    #[serial]
    fn test_search_description() {
        setup_test_registry();

        let results = search("keyboard", SearchOptions::default());
        assert!(!results.is_empty());

        // Should match items with "keyboard" in description
        for result in &results {
            assert!(
                result.description.to_lowercase().contains("keyboard")
                    || result.name.to_lowercase().contains("key")
            );
        }
    }

    #[test]
    #[serial]
    fn test_search_functions_only() {
        setup_test_registry();

        let results = search_functions("key");
        assert!(!results.is_empty());

        // All results should be functions
        for result in &results {
            assert_eq!(result.kind, SearchResultKind::Function);
        }
    }

    #[test]
    #[serial]
    fn test_search_types_only() {
        setup_test_registry();

        let results = search_types("key");
        assert!(!results.is_empty());

        // All results should be types
        for result in &results {
            assert_eq!(result.kind, SearchResultKind::Type);
        }
    }

    #[test]
    #[serial]
    fn test_search_in_module() {
        setup_test_registry();

        let results = search_in_module("keys", None);
        assert!(!results.is_empty());

        // All results should be from "keys" module
        for result in &results {
            assert_eq!(result.module, "keys");
        }
    }

    #[test]
    #[serial]
    fn test_search_in_module_with_query() {
        setup_test_registry();

        let results = search_in_module("keys", Some("emit"));
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "emit_key");
        assert_eq!(results[0].module, "keys");
    }

    #[test]
    #[serial]
    fn test_search_max_results() {
        setup_test_registry();

        let options = SearchOptions {
            max_results: 1,
            ..Default::default()
        };

        let results = search("key", options);
        assert_eq!(results.len(), 1);
    }

    #[test]
    #[serial]
    fn test_search_min_score() {
        setup_test_registry();

        let options = SearchOptions {
            min_score: 0.8,
            ..Default::default()
        };

        let results = search("emit", options);
        assert!(!results.is_empty());

        // All results should have score >= 0.8
        for result in &results {
            assert!(result.score >= 0.8);
        }
    }

    #[test]
    #[serial]
    fn test_search_empty_query() {
        setup_test_registry();

        let results = search("", SearchOptions::default());
        assert!(results.is_empty());
    }

    #[test]
    #[serial]
    fn test_search_no_matches() {
        setup_test_registry();

        let results = search("nonexistent_function_xyz", SearchOptions::default());
        assert!(results.is_empty());
    }

    #[test]
    #[serial]
    fn test_search_scoring_order() {
        setup_test_registry();

        let results = search("key", SearchOptions::default());
        assert!(!results.is_empty());

        // Results should be sorted by score (highest first)
        for i in 1..results.len() {
            assert!(results[i - 1].score >= results[i].score);
        }
    }

    #[test]
    #[serial]
    fn test_search_with_deprecated() {
        registry::initialize();
        registry::clear();

        // Add a deprecated function
        let mut deprecated_func = create_test_function("old_emit", "keys", "Old emit function");
        deprecated_func.deprecated = Some("Use emit_key instead".to_string());
        registry::register_function(deprecated_func);

        // Add a non-deprecated function
        registry::register_function(create_test_function(
            "emit_key",
            "keys",
            "Emits a key press",
        ));

        let results = search("emit", SearchOptions::default());
        assert!(results.len() >= 2);

        // Non-deprecated should score higher
        let emit_key = results.iter().find(|r| r.name == "emit_key").unwrap();
        let old_emit = results.iter().find(|r| r.name == "old_emit").unwrap();
        assert!(emit_key.score > old_emit.score);
    }
}
