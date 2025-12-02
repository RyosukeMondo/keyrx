//! Test discovery for Rhai scripts.
//!
//! This module provides test discovery for Rhai scripts:
//! - Discovers test functions by `test_` prefix (Rhai doesn't support attributes)
//! - Supports glob-style filtering of test names

use rhai::AST;

/// Discovered test function with metadata.
#[derive(Debug, Clone)]
pub struct DiscoveredTest {
    /// Function name.
    pub name: String,
    /// Line number where the function is defined.
    pub line_number: Option<u32>,
}

/// Discover test functions in a Rhai AST.
///
/// Finds all functions with names starting with `test_` prefix.
/// Rhai doesn't support attributes like Rust's `#[test]`, so we use
/// naming conventions instead.
///
/// # Arguments
/// * `ast` - The compiled Rhai AST to search
///
/// # Returns
/// A vector of discovered test names.
pub fn discover_tests(ast: &AST) -> Vec<DiscoveredTest> {
    let mut tests = Vec::new();

    for fn_def in ast.iter_functions() {
        if fn_def.name.starts_with("test_") {
            tests.push(DiscoveredTest {
                name: fn_def.name.to_string(),
                // ScriptFnMetadata doesn't expose line numbers, so we can't get them here.
                // Line numbers would require access to the internal ScriptFnDef structure.
                line_number: None,
            });
        }
    }

    // Sort by name for consistent ordering (line numbers not available from metadata)
    tests.sort_by(|a, b| a.name.cmp(&b.name));

    tracing::debug!(
        service = "keyrx",
        event = "tests_discovered",
        component = "test_discovery",
        count = tests.len(),
        "Discovered {} test functions",
        tests.len()
    );

    tests
}

/// Check if a test name matches a filter pattern.
///
/// Supports basic glob-style matching with `*` as wildcard.
pub fn matches_filter(name: &str, pattern: &str) -> bool {
    if pattern.is_empty() || pattern == "*" {
        return true;
    }

    // Handle patterns with wildcards
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();

        if parts.len() == 2 {
            // Simple prefix/suffix matching
            let prefix = parts[0];
            let suffix = parts[1];

            if prefix.is_empty() {
                // *suffix - match at end
                return name.ends_with(suffix);
            } else if suffix.is_empty() {
                // prefix* - match at start
                return name.starts_with(prefix);
            } else {
                // prefix*suffix - match both
                return name.starts_with(prefix) && name.ends_with(suffix);
            }
        }
        // For complex patterns, fall back to contains check on parts
        return parts.iter().all(|p| p.is_empty() || name.contains(p));
    }

    // Exact match
    name == pattern
}

/// Filter discovered tests by pattern.
///
/// The filter supports basic glob-style matching:
/// - `*` matches any sequence of characters
/// - `test_capslock*` matches all tests starting with `test_capslock`
///
/// # Arguments
/// * `tests` - List of discovered tests
/// * `filter` - Pattern to filter test names
///
/// # Returns
/// A vector of tests matching the filter.
pub fn filter_tests(tests: &[DiscoveredTest], filter: &str) -> Vec<DiscoveredTest> {
    let filtered: Vec<_> = tests
        .iter()
        .filter(|t| matches_filter(&t.name, filter))
        .cloned()
        .collect();

    tracing::debug!(
        service = "keyrx",
        event = "tests_filtered",
        component = "test_discovery",
        filter = filter,
        matched = filtered.len(),
        total = tests.len(),
        "Filtered tests: {}/{} match pattern",
        filtered.len(),
        tests.len()
    );

    filtered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_filter_exact() {
        assert!(matches_filter("test_foo", "test_foo"));
        assert!(!matches_filter("test_foo", "test_bar"));
    }

    #[test]
    fn test_matches_filter_wildcard_suffix() {
        assert!(matches_filter("test_capslock_remap", "test_capslock*"));
        assert!(matches_filter("test_capslock", "test_capslock*"));
        assert!(!matches_filter("test_layer", "test_capslock*"));
    }

    #[test]
    fn test_matches_filter_wildcard_prefix() {
        assert!(matches_filter("test_something_remap", "*remap"));
        assert!(matches_filter("remap", "*remap"));
        assert!(!matches_filter("test_layer", "*remap"));
    }

    #[test]
    fn test_matches_filter_wildcard_both() {
        assert!(matches_filter("test_capslock_remap", "*capslock*"));
        assert!(matches_filter("capslock", "*capslock*"));
        assert!(!matches_filter("test_layer", "*capslock*"));
    }

    #[test]
    fn test_matches_filter_empty_or_star() {
        assert!(matches_filter("anything", ""));
        assert!(matches_filter("anything", "*"));
    }

    #[test]
    fn discover_tests_finds_test_prefix() {
        // Load a script with test functions
        let script = r#"
            fn test_alpha() { }
            fn test_beta() { }
            fn helper_function() { }
            fn test_gamma() { }
        "#;

        // Get AST for discovery - need to compile it
        let ast = rhai::Engine::new().compile(script).expect("compilation");
        let tests = discover_tests(&ast);

        assert_eq!(tests.len(), 3);
        assert!(tests.iter().any(|t| t.name == "test_alpha"));
        assert!(tests.iter().any(|t| t.name == "test_beta"));
        assert!(tests.iter().any(|t| t.name == "test_gamma"));
        assert!(!tests.iter().any(|t| t.name == "helper_function"));
    }

    #[test]
    fn discover_tests_empty_script() {
        let script = r#"
            fn helper() { }
            let x = 42;
        "#;
        let ast = rhai::Engine::new().compile(script).expect("compilation");
        let tests = discover_tests(&ast);

        assert!(tests.is_empty());
    }

    #[test]
    fn discovered_test_struct_clone() {
        let test = DiscoveredTest {
            name: "test_clone".to_string(),
            line_number: Some(42),
        };
        let cloned = test.clone();
        assert_eq!(cloned.name, "test_clone");
        assert_eq!(cloned.line_number, Some(42));
    }

    #[test]
    fn discover_tests_sorted_by_name() {
        let script = r#"
            fn test_zebra() { }
            fn test_alpha() { }
            fn test_middle() { }
        "#;
        let ast = rhai::Engine::new().compile(script).expect("compilation");
        let tests = discover_tests(&ast);

        assert_eq!(tests.len(), 3);
        // Should be sorted alphabetically
        assert_eq!(tests[0].name, "test_alpha");
        assert_eq!(tests[1].name, "test_middle");
        assert_eq!(tests[2].name, "test_zebra");
    }

    #[test]
    fn filter_tests_matches_pattern() {
        let tests = vec![
            DiscoveredTest {
                name: "test_capslock_remap".to_string(),
                line_number: Some(1),
            },
            DiscoveredTest {
                name: "test_capslock_block".to_string(),
                line_number: Some(2),
            },
            DiscoveredTest {
                name: "test_layer_push".to_string(),
                line_number: Some(3),
            },
        ];

        let filtered = filter_tests(&tests, "test_capslock*");

        assert_eq!(filtered.len(), 2);
        // Order is preserved from input
        assert_eq!(filtered[0].name, "test_capslock_remap");
        assert_eq!(filtered[1].name, "test_capslock_block");
    }

    #[test]
    fn filter_tests_with_no_matches() {
        let tests = vec![
            DiscoveredTest {
                name: "test_foo".to_string(),
                line_number: None,
            },
            DiscoveredTest {
                name: "test_bar".to_string(),
                line_number: None,
            },
        ];

        let filtered = filter_tests(&tests, "test_nonexistent*");

        assert!(filtered.is_empty());
    }

    #[test]
    fn matches_filter_complex_pattern() {
        // Multiple wildcards (simplified handling)
        assert!(matches_filter("test_foo_bar_baz", "*foo*baz*"));
        assert!(matches_filter("test_foo_bar_baz", "*bar*"));
    }
}
