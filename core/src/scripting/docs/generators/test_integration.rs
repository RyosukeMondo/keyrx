//! Integration test to verify markdown generation works with the actual registry.

#[cfg(test)]
mod tests {
    use crate::scripting::docs::{
        generators::markdown::generate_markdown, registry, FunctionDoc, FunctionSignature,
        ParamDoc, ReturnDoc, TypeDoc,
    };
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_generate_markdown_integration() {
        // Initialize and populate registry
        registry::initialize();
        registry::clear();

        // Add a function
        let func = FunctionDoc {
            name: "emit_key".to_string(),
            module: "keys".to_string(),
            signature: FunctionSignature {
                params: vec![("key".to_string(), "KeyCode".to_string())],
                return_type: Some("()".to_string()),
            },
            description: "Emits a key press event to the system".to_string(),
            parameters: vec![ParamDoc {
                name: "key".to_string(),
                type_name: "KeyCode".to_string(),
                description: "The key code to emit".to_string(),
                optional: false,
                default: None,
            }],
            returns: Some(ReturnDoc {
                type_name: "()".to_string(),
                description: "Nothing".to_string(),
            }),
            examples: vec!["emit_key(Key::A);".to_string()],
            since: Some("0.1.0".to_string()),
            deprecated: None,
            notes: Some(
                "This function requires elevated permissions on some platforms.".to_string(),
            ),
        };
        registry::register_function(func);

        // Add a type
        let type_doc = TypeDoc {
            name: "KeyCode".to_string(),
            description: "Represents a keyboard key code".to_string(),
            methods: vec![],
            properties: vec![],
            constructors: vec![],
            module: "keys".to_string(),
            since: Some("0.1.0".to_string()),
            examples: vec!["let key = Key::A;".to_string()],
        };
        registry::register_type(type_doc);

        // Generate markdown
        let markdown = generate_markdown();

        // Verify content
        assert!(markdown.contains("# Rhai API Documentation"));
        assert!(markdown.contains("## keys"));
        assert!(markdown.contains("### Types"));
        assert!(markdown.contains("#### `KeyCode`"));
        assert!(markdown.contains("Represents a keyboard key code"));
        assert!(markdown.contains("### Functions"));
        assert!(markdown.contains("emit_key"));
        assert!(markdown.contains("Emits a key press event to the system"));
        assert!(markdown.contains("**Parameters:**"));
        assert!(markdown.contains("**Returns:**"));
        assert!(markdown.contains("**Examples:**"));
        assert!(markdown.contains("```rhai"));
        assert!(markdown.contains("emit_key(Key::A);"));
        assert!(markdown.contains("*Since: 0.1.0*"));

        // Clean up
        registry::clear();
    }

    #[test]
    #[serial]
    fn test_generate_markdown_multiple_modules() {
        registry::initialize();
        registry::clear();

        // Add functions to different modules
        let keys_func = FunctionDoc {
            name: "emit_key".to_string(),
            module: "keys".to_string(),
            signature: FunctionSignature {
                params: vec![],
                return_type: None,
            },
            description: "Keys function".to_string(),
            parameters: vec![],
            returns: None,
            examples: vec![],
            since: None,
            deprecated: None,
            notes: None,
        };

        let layers_func = FunctionDoc {
            name: "switch_layer".to_string(),
            module: "layers".to_string(),
            signature: FunctionSignature {
                params: vec![],
                return_type: None,
            },
            description: "Layers function".to_string(),
            parameters: vec![],
            returns: None,
            examples: vec![],
            since: None,
            deprecated: None,
            notes: None,
        };

        registry::register_function(keys_func);
        registry::register_function(layers_func);

        let markdown = generate_markdown();

        // Verify both modules are present
        assert!(markdown.contains("## keys"));
        assert!(markdown.contains("## layers"));
        assert!(markdown.contains("emit_key"));
        assert!(markdown.contains("switch_layer"));

        registry::clear();
    }

    #[test]
    fn test_markdown_table_of_contents() {
        registry::initialize();
        registry::clear();

        let func = FunctionDoc {
            name: "test".to_string(),
            module: "alpha".to_string(),
            signature: FunctionSignature {
                params: vec![],
                return_type: None,
            },
            description: "Test".to_string(),
            parameters: vec![],
            returns: None,
            examples: vec![],
            since: None,
            deprecated: None,
            notes: None,
        };

        registry::register_function(func);

        let markdown = generate_markdown();

        // Verify TOC exists and contains the module
        assert!(markdown.contains("## Table of Contents"));
        assert!(markdown.contains("- [alpha](#alpha)"));

        registry::clear();
    }
}
