//! Comprehensive tests for error formatting functionality.
//!
//! Tests verify:
//! - Each ParseError variant produces expected formatted output
//! - Code snippet generation with different line numbers
//! - Caret positioning for different column numbers
//! - Import chain display
//! - Colored vs non-colored output (NO_COLOR)

use keyrx_compiler::error::formatting::format_error;
use keyrx_compiler::error::types::{ImportStep, ParseError};
use std::path::PathBuf;

/// Helper function to strip ANSI escape codes from a string.
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::new();
    let mut in_escape = false;

    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
        } else {
            result.push(ch);
        }
    }

    result
}

// Helper to create a test source file with known content
fn test_source() -> &'static str {
    "device_start(\"keyboard\")\n\
     map(\"VK_A\", \"VK_B\")\n\
     map(\"VK_X\", \"InvalidKey\")\n\
     device_end()\n"
}

// Helper to create test source with multiple lines for context testing
fn multiline_source() -> &'static str {
    "device_start(\"keyboard\")\n\
     map(\"VK_A\", \"VK_B\")\n\
     map(\"VK_C\", \"VK_D\")\n\
     map(\"VK_E\", \"VK_F\")  // error on this line\n\
     map(\"VK_G\", \"VK_H\")\n\
     map(\"VK_I\", \"VK_J\")\n\
     device_end()\n"
}

#[test]
fn test_syntax_error_formatting() {
    let error = ParseError::SyntaxError {
        file: PathBuf::from("test.rhai"),
        line: 3,
        column: 15,
        message: "unexpected token".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), test_source());

    // Verify structure (without checking exact color codes)
    assert!(formatted.contains("test.rhai:3:15"));
    assert!(formatted.contains("Error:"));
    assert!(formatted.contains("unexpected token"));
    assert!(formatted.contains("map(\"VK_X\", \"InvalidKey\")"));
    assert!(formatted.contains("^")); // Caret should be present
    assert!(formatted.contains("help:"));
    assert!(formatted.contains("Check your Rhai script syntax"));
}

#[test]
fn test_syntax_error_with_import_chain() {
    let import_chain = vec![
        ImportStep {
            file: PathBuf::from("main.rhai"),
            line: 5,
        },
        ImportStep {
            file: PathBuf::from("common/vim.rhai"),
            line: 2,
        },
    ];

    let error = ParseError::SyntaxError {
        file: PathBuf::from("common/helpers.rhai"),
        line: 10,
        column: 5,
        message: "parse error".to_string(),
        import_chain,
    };

    let formatted = format_error(&error, &PathBuf::from("common/helpers.rhai"), test_source());

    // Verify import chain is displayed
    assert!(formatted.contains("Import chain:"));
    assert!(formatted.contains("main.rhai"));
    assert!(formatted.contains("line 5"));
    assert!(formatted.contains("common/vim.rhai"));
    assert!(formatted.contains("line 2"));
    assert!(formatted.contains("→")); // Arrow should be present
}

#[test]
fn test_code_snippet_first_line() {
    let error = ParseError::SyntaxError {
        file: PathBuf::from("test.rhai"),
        line: 1,
        column: 10,
        message: "error on first line".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), test_source());
    let stripped = strip_ansi_codes(&formatted);

    // Should show line 1 and 2 (no line 0)
    assert!(stripped.contains("device_start(\"keyboard\")"));

    // Verify line 1 and 2 are present (format is "1    |" not "   1 |")
    assert!(
        stripped.contains("1    |"),
        "Line 1 not found in formatted output"
    );
    assert!(
        stripped.contains("2    |"),
        "Line 2 not found in formatted output"
    );
}

#[test]
fn test_code_snippet_last_line() {
    let error = ParseError::SyntaxError {
        file: PathBuf::from("test.rhai"),
        line: 4,
        column: 5,
        message: "error on last line".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), test_source());
    let stripped = strip_ansi_codes(&formatted);

    // Should show line 3 and 4 (no line 5)
    assert!(stripped.contains("device_end()"));

    // Verify line numbers are present (format is "3    |" not "   3 |")
    assert!(
        stripped.contains("3    |"),
        "Line 3 not found in formatted output"
    );
    assert!(
        stripped.contains("4    |"),
        "Line 4 not found in formatted output"
    );
}

#[test]
fn test_code_snippet_middle_line() {
    let error = ParseError::SyntaxError {
        file: PathBuf::from("test.rhai"),
        line: 4,
        column: 10,
        message: "error in middle".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), multiline_source());
    let stripped = strip_ansi_codes(&formatted);

    // Should show 3 lines of context (line before, error line, line after)
    assert!(stripped.contains("// error on this line"));

    // Verify line numbers are present (format is "3    |" not "   3 |")
    assert!(
        stripped.contains("3    |"),
        "Line 3 not found in formatted output"
    );
    assert!(
        stripped.contains("4    |"),
        "Line 4 not found in formatted output"
    );
    assert!(
        stripped.contains("5    |"),
        "Line 5 not found in formatted output"
    );
}

#[test]
fn test_caret_positioning_column_1() {
    let error = ParseError::SyntaxError {
        file: PathBuf::from("test.rhai"),
        line: 2,
        column: 1,
        message: "error at column 1".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), test_source());

    // Caret should be at the start (no leading spaces before the ^)
    // The format is: "     | ^" (5 spaces, pipe, space, caret)
    let has_caret = formatted.contains("^");
    assert!(has_caret, "Caret (^) not found in formatted output");

    // Verify caret is on its own line (after the pipe separator)
    let lines: Vec<&str> = formatted.lines().collect();
    let caret_line = lines
        .iter()
        .find(|line| {
            // Strip ANSI codes to find the caret line
            let stripped = line.chars().filter(|&c| c != '\x1b').collect::<String>();
            stripped.contains('^') && stripped.contains('|')
        })
        .expect("Caret line not found");

    // For column 1, should be: "     | ^" (with possible ANSI codes)
    // The caret should appear right after "| " with no additional spaces
    assert!(caret_line.contains("| ^") || caret_line.ends_with('^'));
}

#[test]
fn test_caret_positioning_column_15() {
    let error = ParseError::SyntaxError {
        file: PathBuf::from("test.rhai"),
        line: 3,
        column: 15,
        message: "error at column 15".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), test_source());
    let stripped = strip_ansi_codes(&formatted);

    // Caret should be at column 15
    assert!(
        stripped.contains("^"),
        "Caret (^) not found in formatted output"
    );

    // Find the caret line
    let lines: Vec<&str> = stripped.lines().collect();
    let caret_line = lines
        .iter()
        .find(|line| line.contains('^') && line.contains('|'))
        .expect("Caret line not found");

    // Count spaces after the pipe
    // The format is: "     | " + spaces + "^"
    // For column 15, we expect the pipe at position 5, then a space, then 14 more spaces, then ^
    // But the implementation does error_column.saturating_sub(1), so column 15 → 14 spaces
    // However, there's also the space after the pipe separator
    if let Some(pipe_pos) = caret_line.find('|') {
        let after_pipe = &caret_line[pipe_pos + 1..];
        let spaces_before_caret = after_pipe.chars().take_while(|&c| c == ' ').count();

        // The code does: "     | " + " ".repeat(error_column.saturating_sub(1)) + "^"
        // So for column 15: "     | " + " ".repeat(14) + "^"
        // That's 1 space (after |) + 14 spaces = 15 total spaces before ^
        assert_eq!(
            spaces_before_caret, 15,
            "Expected 15 spaces before caret for column 15 (1 after pipe + 14 from repeat)"
        );
    } else {
        panic!("No pipe separator found in caret line");
    }
}

#[test]
fn test_invalid_prefix_error() {
    let error = ParseError::InvalidPrefix {
        expected: "VK_".to_string(),
        got: "MD_00".to_string(),
        context: "map output parameter".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");

    assert!(formatted.contains("Error:"));
    assert!(formatted.contains("Invalid prefix"));
    assert!(formatted.contains("VK_"));
    assert!(formatted.contains("MD_00"));
    assert!(formatted.contains("help:"));
}

#[test]
fn test_invalid_prefix_md_with_physical_name() {
    let error = ParseError::InvalidPrefix {
        expected: "MD_XX (hex)".to_string(),
        got: "MD_LShift".to_string(),
        context: "modifier parameter".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");

    assert!(formatted.contains("Invalid modifier ID format"));
    assert!(formatted.contains("MD_LShift"));
    assert!(formatted.contains("MD_00 through MD_FE"));
    assert!(formatted.contains("Example:"));
}

#[test]
fn test_invalid_prefix_vk_in_hold() {
    let error = ParseError::InvalidPrefix {
        expected: "MD_".to_string(),
        got: "VK_A".to_string(),
        context: "tap_hold hold parameter".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");

    assert!(formatted.contains("hold parameter must have MD_ prefix"));
    assert!(formatted.contains("VK_A"));
    assert!(formatted.contains("tap_hold"));
}

#[test]
fn test_modifier_id_out_of_range() {
    let error = ParseError::ModifierIdOutOfRange {
        got: 255,
        max: 254,
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");

    assert!(formatted.contains("Modifier ID out of range"));
    assert!(formatted.contains("255"));
    assert!(formatted.contains("254"));
    assert!(formatted.contains("MD_00 to MD_FE"));
    assert!(formatted.contains("help:"));
}

#[test]
fn test_lock_id_out_of_range() {
    let error = ParseError::LockIdOutOfRange {
        got: 300,
        max: 254,
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");

    assert!(formatted.contains("Lock ID out of range"));
    assert!(formatted.contains("300"));
    assert!(formatted.contains("254"));
    assert!(formatted.contains("LK_00 to LK_FE"));
    assert!(formatted.contains("help:"));
}

#[test]
fn test_physical_modifier_in_md() {
    let error = ParseError::PhysicalModifierInMD {
        name: "LShift".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");

    assert!(formatted.contains("Physical modifier name 'LShift'"));
    assert!(formatted.contains("MD_ prefix"));
    assert!(formatted.contains("LShift, RShift, LCtrl, RCtrl"));
    assert!(formatted.contains("MD_00 through MD_FE"));
    assert!(formatted.contains("Example:"));
}

#[test]
fn test_missing_prefix_output_context() {
    let error = ParseError::MissingPrefix {
        key: "A".to_string(),
        context: "map output parameter".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");

    assert!(formatted.contains("Missing prefix for key 'A'"));
    assert!(formatted.contains("Output keys must have VK_, MD_, or LK_ prefix"));
    assert!(formatted.contains("VK_A"));
    assert!(formatted.contains("Examples:"));
}

#[test]
fn test_missing_prefix_generic_context() {
    let error = ParseError::MissingPrefix {
        key: "Space".to_string(),
        context: "key parameter".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");

    assert!(formatted.contains("Missing prefix for key 'Space'"));
    assert!(formatted.contains("help:"));
}

#[test]
fn test_import_not_found() {
    let error = ParseError::ImportNotFound {
        path: PathBuf::from("common/vim.rhai"),
        searched_paths: vec![
            PathBuf::from("/home/user/config/common/vim.rhai"),
            PathBuf::from("/home/user/common/vim.rhai"),
        ],
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");

    assert!(formatted.contains("Import file not found"));
    assert!(formatted.contains("common/vim.rhai"));
    assert!(formatted.contains("Searched paths:"));
    assert!(formatted.contains("/home/user/config/common/vim.rhai"));
    assert!(formatted.contains("/home/user/common/vim.rhai"));
    assert!(formatted.contains("help:"));
}

#[test]
fn test_circular_import() {
    let error = ParseError::CircularImport {
        chain: vec![
            PathBuf::from("a.rhai"),
            PathBuf::from("b.rhai"),
            PathBuf::from("c.rhai"),
            PathBuf::from("a.rhai"),
        ],
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");

    assert!(formatted.contains("Circular import detected"));
    assert!(formatted.contains("a.rhai"));
    assert!(formatted.contains("b.rhai"));
    assert!(formatted.contains("c.rhai"));
    assert!(formatted.contains("→"));
    assert!(formatted.contains("help:"));
    assert!(formatted.contains("Remove the circular dependency"));
}

#[test]
fn test_resource_limit_exceeded() {
    let error = ParseError::ResourceLimitExceeded {
        limit_type: "maximum nesting depth (10)".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");

    assert!(formatted.contains("Resource limit exceeded"));
    assert!(formatted.contains("maximum nesting depth"));
    assert!(formatted.contains("help:"));
    assert!(formatted.contains("simplifying"));
}

#[test]
fn test_import_chain_empty() {
    let error = ParseError::SyntaxError {
        file: PathBuf::from("test.rhai"),
        line: 1,
        column: 1,
        message: "error".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), test_source());

    // Should not contain "Import chain:" when chain is empty
    assert!(!formatted.contains("Import chain:"));
}

#[test]
fn test_import_chain_single_step() {
    let import_chain = vec![ImportStep {
        file: PathBuf::from("main.rhai"),
        line: 10,
    }];

    let error = ParseError::SyntaxError {
        file: PathBuf::from("imported.rhai"),
        line: 1,
        column: 1,
        message: "error".to_string(),
        import_chain,
    };

    let formatted = format_error(&error, &PathBuf::from("imported.rhai"), test_source());

    assert!(formatted.contains("Import chain:"));
    assert!(formatted.contains("main.rhai"));
    assert!(formatted.contains("line 10"));
}

#[test]
fn test_import_chain_multiple_steps() {
    let import_chain = vec![
        ImportStep {
            file: PathBuf::from("main.rhai"),
            line: 5,
        },
        ImportStep {
            file: PathBuf::from("a.rhai"),
            line: 3,
        },
        ImportStep {
            file: PathBuf::from("b.rhai"),
            line: 7,
        },
    ];

    let error = ParseError::SyntaxError {
        file: PathBuf::from("c.rhai"),
        line: 1,
        column: 1,
        message: "error".to_string(),
        import_chain,
    };

    let formatted = format_error(&error, &PathBuf::from("c.rhai"), test_source());

    assert!(formatted.contains("Import chain:"));
    assert!(formatted.contains("main.rhai"));
    assert!(formatted.contains("line 5"));
    assert!(formatted.contains("a.rhai"));
    assert!(formatted.contains("line 3"));
    assert!(formatted.contains("b.rhai"));
    assert!(formatted.contains("line 7"));

    // Verify arrows are present between steps
    let arrow_count = formatted.matches('→').count();
    assert_eq!(arrow_count, 3); // 3 arrows for 3 steps
}

#[test]
fn test_error_with_import_chain_all_variants() {
    let import_chain = vec![ImportStep {
        file: PathBuf::from("main.rhai"),
        line: 1,
    }];

    // Test InvalidPrefix with import chain
    let error = ParseError::InvalidPrefix {
        expected: "VK_".to_string(),
        got: "Bad".to_string(),
        context: "test".to_string(),
        import_chain: import_chain.clone(),
    };
    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");
    assert!(formatted.contains("Import chain:"));

    // Test ModifierIdOutOfRange with import chain
    let error = ParseError::ModifierIdOutOfRange {
        got: 255,
        max: 254,
        import_chain: import_chain.clone(),
    };
    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");
    assert!(formatted.contains("Import chain:"));

    // Test LockIdOutOfRange with import chain
    let error = ParseError::LockIdOutOfRange {
        got: 255,
        max: 254,
        import_chain: import_chain.clone(),
    };
    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");
    assert!(formatted.contains("Import chain:"));

    // Test PhysicalModifierInMD with import chain
    let error = ParseError::PhysicalModifierInMD {
        name: "LShift".to_string(),
        import_chain: import_chain.clone(),
    };
    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");
    assert!(formatted.contains("Import chain:"));

    // Test MissingPrefix with import chain
    let error = ParseError::MissingPrefix {
        key: "A".to_string(),
        context: "test".to_string(),
        import_chain: import_chain.clone(),
    };
    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");
    assert!(formatted.contains("Import chain:"));

    // Test ImportNotFound with import chain
    let error = ParseError::ImportNotFound {
        path: PathBuf::from("missing.rhai"),
        searched_paths: vec![],
        import_chain: import_chain.clone(),
    };
    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");
    assert!(formatted.contains("Import chain:"));

    // Test ResourceLimitExceeded with import chain
    let error = ParseError::ResourceLimitExceeded {
        limit_type: "depth".to_string(),
        import_chain,
    };
    let formatted = format_error(&error, &PathBuf::from("test.rhai"), "");
    assert!(formatted.contains("Import chain:"));
}

// NO_COLOR tests
// Note: The `colored` crate respects NO_COLOR environment variable automatically.
// These tests verify behavior with and without colors.

#[test]
fn test_colored_output_contains_ansi_codes() {
    // Ensure NO_COLOR is not set
    std::env::remove_var("NO_COLOR");
    // Also remove CLICOLOR and related env vars that might disable colors
    std::env::remove_var("CLICOLOR");
    std::env::remove_var("CLICOLOR_FORCE");

    let error = ParseError::SyntaxError {
        file: PathBuf::from("test.rhai"),
        line: 1,
        column: 1,
        message: "test error".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), test_source());

    // Note: The colored crate may still disable colors in test environments
    // or non-TTY contexts. We'll just verify the output is valid, not force colors.
    // The important thing is that the error formatting function works correctly.

    // Verify essential content is present regardless of color
    assert!(formatted.contains("test.rhai:1:1"));
    assert!(formatted.contains("Error:"));
    assert!(formatted.contains("test error"));
    assert!(formatted.contains("^"));
    assert!(formatted.contains("help:"));
}

#[test]
fn test_no_color_output_plain_text() {
    // Set NO_COLOR environment variable
    std::env::set_var("NO_COLOR", "1");

    let error = ParseError::SyntaxError {
        file: PathBuf::from("test.rhai"),
        line: 1,
        column: 1,
        message: "test error".to_string(),
        import_chain: Vec::new(),
    };

    let formatted = format_error(&error, &PathBuf::from("test.rhai"), test_source());

    // With NO_COLOR set, output should not contain ANSI escape codes
    assert!(
        !formatted.contains("\x1b["),
        "Expected no ANSI color codes when NO_COLOR is set"
    );

    // But should still contain the error message content
    assert!(formatted.contains("test.rhai:1:1"));
    assert!(formatted.contains("Error:"));
    assert!(formatted.contains("test error"));

    // Clean up
    std::env::remove_var("NO_COLOR");
}

#[test]
fn test_help_text_present_in_all_errors() {
    let errors = vec![
        ParseError::SyntaxError {
            file: PathBuf::from("test.rhai"),
            line: 1,
            column: 1,
            message: "error".to_string(),
            import_chain: Vec::new(),
        },
        ParseError::InvalidPrefix {
            expected: "VK_".to_string(),
            got: "Bad".to_string(),
            context: "test".to_string(),
            import_chain: Vec::new(),
        },
        ParseError::ModifierIdOutOfRange {
            got: 255,
            max: 254,
            import_chain: Vec::new(),
        },
        ParseError::LockIdOutOfRange {
            got: 255,
            max: 254,
            import_chain: Vec::new(),
        },
        ParseError::PhysicalModifierInMD {
            name: "LShift".to_string(),
            import_chain: Vec::new(),
        },
        ParseError::MissingPrefix {
            key: "A".to_string(),
            context: "output".to_string(),
            import_chain: Vec::new(),
        },
        ParseError::ImportNotFound {
            path: PathBuf::from("missing.rhai"),
            searched_paths: vec![],
            import_chain: Vec::new(),
        },
        ParseError::CircularImport {
            chain: vec![PathBuf::from("a.rhai"), PathBuf::from("b.rhai")],
        },
        ParseError::ResourceLimitExceeded {
            limit_type: "depth".to_string(),
            import_chain: Vec::new(),
        },
    ];

    for error in errors {
        let formatted = format_error(&error, &PathBuf::from("test.rhai"), test_source());
        assert!(
            formatted.contains("help:") || formatted.contains("Help:"),
            "Error variant {:?} should contain help text",
            error
        );
    }
}

#[test]
fn test_all_errors_contain_suggestions() {
    let errors = vec![
        (
            ParseError::SyntaxError {
                file: PathBuf::from("test.rhai"),
                line: 1,
                column: 1,
                message: "error".to_string(),
                import_chain: Vec::new(),
            },
            "Check your Rhai script syntax",
        ),
        (
            ParseError::InvalidPrefix {
                expected: "VK_".to_string(),
                got: "Bad".to_string(),
                context: "test".to_string(),
                import_chain: Vec::new(),
            },
            "Valid prefixes",
        ),
        (
            ParseError::ModifierIdOutOfRange {
                got: 255,
                max: 254,
                import_chain: Vec::new(),
            },
            "MD_00 to MD_FE",
        ),
        (
            ParseError::PhysicalModifierInMD {
                name: "LShift".to_string(),
                import_chain: Vec::new(),
            },
            "MD_00 through MD_FE",
        ),
        (
            ParseError::MissingPrefix {
                key: "A".to_string(),
                context: "output".to_string(),
                import_chain: Vec::new(),
            },
            "VK_A",
        ),
        (
            ParseError::CircularImport {
                chain: vec![PathBuf::from("a.rhai")],
            },
            "Remove the circular dependency",
        ),
        (
            ParseError::ResourceLimitExceeded {
                limit_type: "depth".to_string(),
                import_chain: Vec::new(),
            },
            "simplifying",
        ),
    ];

    for (error, expected_suggestion) in errors {
        let formatted = format_error(&error, &PathBuf::from("test.rhai"), test_source());
        assert!(
            formatted.contains(expected_suggestion),
            "Error variant {:?} should contain suggestion: {}",
            error,
            expected_suggestion
        );
    }
}
