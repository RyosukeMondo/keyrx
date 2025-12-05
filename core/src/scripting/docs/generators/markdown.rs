//! Markdown documentation generator.
//!
//! Generates clean, readable Markdown documentation from the DocRegistry,
//! organized by module with proper formatting and code blocks.

use crate::scripting::docs::{
    registry, FunctionDoc, FunctionSignature, ParamDoc, PropertyDoc, TypeDoc,
};
use std::collections::BTreeMap;

/// Generates Markdown documentation for all registered API documentation.
///
/// # Returns
/// A formatted Markdown string containing all documentation, organized by module.
///
/// # Example
/// ```no_run
/// use keyrx_core::scripting::docs::generators::markdown::generate_markdown;
/// let markdown = generate_markdown();
/// println!("{}", markdown);
/// ```
pub fn generate_markdown() -> String {
    let mut output = String::new();

    // Header
    output.push_str("# Rhai API Documentation\n\n");
    output.push_str("Complete API reference for the KeyRx scripting engine.\n\n");
    output.push_str("---\n\n");

    // Table of Contents
    output.push_str("## Table of Contents\n\n");

    let modules = collect_modules();

    for module_name in modules.keys() {
        output.push_str(&format!(
            "- [{}](#{})\n",
            module_name,
            module_name.to_lowercase()
        ));
    }
    output.push_str("\n---\n\n");

    // Generate documentation for each module
    for (module_name, module) in modules {
        generate_module_docs(&mut output, &module_name, &module);
    }

    output
}

/// Collects all documentation organized by module.
fn collect_modules() -> BTreeMap<String, ModuleData> {
    let mut modules = BTreeMap::new();

    // Collect functions by module
    for func in registry::all_functions() {
        let module_name = func.module.clone();
        let entry = modules
            .entry(module_name.clone())
            .or_insert_with(|| ModuleData::new(module_name));
        entry.functions.push(func);
    }

    // Collect types by module
    for type_doc in registry::all_types() {
        let module_name = type_doc.module.clone();
        let entry = modules
            .entry(module_name.clone())
            .or_insert_with(|| ModuleData::new(module_name));
        entry.types.push(type_doc);
    }

    // Add module-level documentation if available
    for module_doc in registry::all_modules() {
        if let Some(entry) = modules.get_mut(&module_doc.name) {
            entry.description = Some(module_doc.description);
        }
    }

    modules
}

/// Temporary structure to hold module data during generation.
struct ModuleData {
    description: Option<String>,
    functions: Vec<FunctionDoc>,
    types: Vec<TypeDoc>,
}

impl ModuleData {
    fn new(_name: String) -> Self {
        Self {
            description: None,
            functions: Vec::new(),
            types: Vec::new(),
        }
    }
}

/// Generates documentation for a single module.
fn generate_module_docs(output: &mut String, module_name: &str, module: &ModuleData) {
    // Module header
    output.push_str(&format!("## {}\n\n", module_name));

    // Module description
    if let Some(description) = &module.description {
        output.push_str(description);
        output.push_str("\n\n");
    }

    // Types section
    if !module.types.is_empty() {
        output.push_str("### Types\n\n");
        for type_doc in &module.types {
            generate_type_docs(output, type_doc);
        }
    }

    // Functions section
    if !module.functions.is_empty() {
        output.push_str("### Functions\n\n");
        for func in &module.functions {
            generate_function_docs(output, func);
        }
    }

    output.push_str("---\n\n");
}

/// Generates documentation for a single type.
fn generate_type_docs(output: &mut String, type_doc: &TypeDoc) {
    // Type name and description
    output.push_str(&format!("#### `{}`\n\n", type_doc.name));
    output.push_str(&type_doc.description);
    output.push_str("\n\n");

    // Version info
    if let Some(since) = &type_doc.since {
        output.push_str(&format!("*Since: {}*\n\n", since));
    }

    // Properties
    if !type_doc.properties.is_empty() {
        output.push_str("**Properties:**\n\n");
        for prop in &type_doc.properties {
            generate_property_docs(output, prop);
        }
        output.push('\n');
    }

    // Methods
    if !type_doc.methods.is_empty() {
        output.push_str("**Methods:**\n\n");
        for method in &type_doc.methods {
            generate_method_docs(output, method);
        }
        output.push('\n');
    }

    // Constructors
    if !type_doc.constructors.is_empty() {
        output.push_str("**Constructors:**\n\n");
        for constructor in &type_doc.constructors {
            generate_method_docs(output, constructor);
        }
        output.push('\n');
    }

    // Examples
    if !type_doc.examples.is_empty() {
        output.push_str("**Examples:**\n\n");
        for example in &type_doc.examples {
            output.push_str("```rhai\n");
            output.push_str(example);
            output.push_str("\n```\n\n");
        }
    }
}

/// Generates documentation for a property.
fn generate_property_docs(output: &mut String, prop: &PropertyDoc) {
    let readonly = if prop.readonly { " (read-only)" } else { "" };
    output.push_str(&format!(
        "- `{}`: `{}`{} - {}\n",
        prop.name, prop.type_name, readonly, prop.description
    ));
}

/// Generates documentation for a method (compact format for type methods).
fn generate_method_docs(output: &mut String, method: &FunctionDoc) {
    let signature = format_signature(&method.signature);
    output.push_str(&format!("- `{}` - {}\n", signature, method.description));
}

/// Generates documentation for a function.
fn generate_function_docs(output: &mut String, func: &FunctionDoc) {
    // Function header with name and signature
    let params = func
        .signature
        .params
        .iter()
        .map(|(name, type_name)| format!("{}: {}", name, type_name))
        .collect::<Vec<_>>()
        .join(", ");

    let return_type = func
        .signature
        .return_type
        .as_ref()
        .map(|t| format!(" -> {}", t))
        .unwrap_or_default();

    let signature = format!("{}({}){}", func.name, params, return_type);
    output.push_str(&format!("#### `{}`\n\n", signature));

    // Deprecation warning
    if let Some(deprecated) = &func.deprecated {
        output.push_str(&format!("**⚠️ DEPRECATED:** {}\n\n", deprecated));
    }

    // Description
    output.push_str(&func.description);
    output.push_str("\n\n");

    // Parameters
    if !func.parameters.is_empty() {
        output.push_str("**Parameters:**\n\n");
        for param in &func.parameters {
            generate_param_docs(output, param);
        }
        output.push('\n');
    }

    // Return value
    if let Some(returns) = &func.returns {
        output.push_str(&format!(
            "**Returns:** `{}` - {}\n\n",
            returns.type_name, returns.description
        ));
    }

    // Notes
    if let Some(notes) = &func.notes {
        output.push_str(&format!("**Notes:** {}\n\n", notes));
    }

    // Examples
    if !func.examples.is_empty() {
        output.push_str("**Examples:**\n\n");
        for example in &func.examples {
            output.push_str("```rhai\n");
            output.push_str(example);
            output.push_str("\n```\n\n");
        }
    }

    // Version info
    if let Some(since) = &func.since {
        output.push_str(&format!("*Since: {}*\n\n", since));
    }
}

/// Generates documentation for a parameter.
fn generate_param_docs(output: &mut String, param: &ParamDoc) {
    let optional = if param.optional { " (optional)" } else { "" };
    let default = if let Some(default) = &param.default {
        format!(" = `{}`", default)
    } else {
        String::new()
    };

    output.push_str(&format!(
        "- `{}`: `{}`{}{} - {}\n",
        param.name, param.type_name, optional, default, param.description
    ));
}

/// Formats a function signature for display.
fn format_signature(sig: &FunctionSignature) -> String {
    let params = sig
        .params
        .iter()
        .map(|(name, type_name)| format!("{}: {}", name, type_name))
        .collect::<Vec<_>>()
        .join(", ");

    let return_type = sig
        .return_type
        .as_ref()
        .map(|t| format!(" -> {}", t))
        .unwrap_or_default();

    format!("fn({}){}", params, return_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::docs::{FunctionSignature, ReturnDoc};
    use serial_test::serial;

    #[test]
    fn test_format_signature_with_params_and_return() {
        let sig = FunctionSignature {
            params: vec![
                ("key".to_string(), "KeyCode".to_string()),
                ("count".to_string(), "int".to_string()),
            ],
            return_type: Some("()".to_string()),
        };

        let formatted = format_signature(&sig);
        assert_eq!(formatted, "fn(key: KeyCode, count: int) -> ()");
    }

    #[test]
    fn test_format_signature_no_params() {
        let sig = FunctionSignature {
            params: vec![],
            return_type: Some("bool".to_string()),
        };

        let formatted = format_signature(&sig);
        assert_eq!(formatted, "fn() -> bool");
    }

    #[test]
    fn test_format_signature_no_return() {
        let sig = FunctionSignature {
            params: vec![("x".to_string(), "int".to_string())],
            return_type: None,
        };

        let formatted = format_signature(&sig);
        assert_eq!(formatted, "fn(x: int)");
    }

    #[test]
    #[serial]
    fn test_generate_markdown_empty_registry() {
        registry::initialize();
        registry::clear();

        let markdown = generate_markdown();

        assert!(markdown.contains("# Rhai API Documentation"));
        assert!(markdown.contains("## Table of Contents"));
    }

    #[test]
    #[serial]
    fn test_generate_markdown_with_function() {
        registry::initialize();
        registry::clear();

        let func = FunctionDoc {
            name: "emit_key".to_string(),
            module: "keys".to_string(),
            signature: FunctionSignature {
                params: vec![("key".to_string(), "KeyCode".to_string())],
                return_type: Some("()".to_string()),
            },
            description: "Emits a key press event".to_string(),
            parameters: vec![ParamDoc {
                name: "key".to_string(),
                type_name: "KeyCode".to_string(),
                description: "The key to emit".to_string(),
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
            notes: None,
        };

        registry::register_function(func);

        let markdown = generate_markdown();

        assert!(markdown.contains("## keys"));
        assert!(markdown.contains("emit_key"));
        assert!(markdown.contains("Emits a key press event"));
        assert!(markdown.contains("```rhai"));
        assert!(markdown.contains("emit_key(Key::A);"));
    }

    #[test]
    #[serial]
    fn test_generate_markdown_with_type() {
        registry::initialize();
        registry::clear();

        let type_doc = TypeDoc {
            name: "KeyCode".to_string(),
            description: "Represents a keyboard key".to_string(),
            methods: vec![],
            properties: vec![PropertyDoc {
                name: "value".to_string(),
                type_name: "int".to_string(),
                description: "Numeric key code".to_string(),
                readonly: true,
            }],
            constructors: vec![],
            module: "keys".to_string(),
            since: Some("0.1.0".to_string()),
            examples: vec![],
        };

        registry::register_type(type_doc);

        let markdown = generate_markdown();

        assert!(markdown.contains("#### `KeyCode`"));
        assert!(markdown.contains("Represents a keyboard key"));
        assert!(markdown.contains("`value`"));
        assert!(markdown.contains("read-only"));
    }

    #[test]
    fn test_generate_param_docs_optional_with_default() {
        let mut output = String::new();
        let param = ParamDoc {
            name: "timeout".to_string(),
            type_name: "int".to_string(),
            description: "Timeout in milliseconds".to_string(),
            optional: true,
            default: Some("1000".to_string()),
        };

        generate_param_docs(&mut output, &param);

        assert!(output.contains("timeout"));
        assert!(output.contains("optional"));
        assert!(output.contains("= `1000`"));
    }

    #[test]
    fn test_generate_property_docs_readonly() {
        let mut output = String::new();
        let prop = PropertyDoc {
            name: "id".to_string(),
            type_name: "int".to_string(),
            description: "Unique identifier".to_string(),
            readonly: true,
        };

        generate_property_docs(&mut output, &prop);

        assert!(output.contains("`id`"));
        assert!(output.contains("read-only"));
        assert!(output.contains("Unique identifier"));
    }

    #[test]
    #[serial]
    fn test_deprecated_function_warning() {
        registry::initialize();
        registry::clear();

        let func = FunctionDoc {
            name: "old_func".to_string(),
            module: "deprecated".to_string(),
            signature: FunctionSignature {
                params: vec![],
                return_type: None,
            },
            description: "Old function".to_string(),
            parameters: vec![],
            returns: None,
            examples: vec![],
            since: Some("0.1.0".to_string()),
            deprecated: Some("Use new_func instead".to_string()),
            notes: None,
        };

        registry::register_function(func);

        let markdown = generate_markdown();

        assert!(markdown.contains("⚠️ DEPRECATED"));
        assert!(markdown.contains("Use new_func instead"));
    }
}
