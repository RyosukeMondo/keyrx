//! JSON schema documentation generator.
//!
//! Generates JSON schemas for IDE integration and autocomplete support.
//! The output format is compatible with standard JSON schema specifications
//! and includes all type information needed for intelligent code completion.

use crate::scripting::docs::{registry, FunctionDoc, PropertyDoc, TypeDoc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// JSON schema output format for IDE integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchema {
    /// Schema version
    #[serde(rename = "$schema")]
    pub schema: String,

    /// API version
    pub version: String,

    /// All documented modules
    pub modules: BTreeMap<String, ModuleSchema>,

    /// Autocomplete definitions
    pub autocomplete: AutocompleteSchema,
}

/// Schema for a single module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSchema {
    /// Module name
    pub name: String,

    /// Module description
    pub description: Option<String>,

    /// Functions in this module
    pub functions: Vec<FunctionSchema>,

    /// Types in this module
    pub types: Vec<TypeSchema>,
}

/// Schema for a function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSchema {
    /// Function name
    pub name: String,

    /// Function description
    pub description: String,

    /// Parameters
    pub parameters: Vec<ParameterSchema>,

    /// Return type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returns: Option<ReturnTypeSchema>,

    /// Code examples
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,

    /// Deprecation warning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<String>,

    /// Version added
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,

    /// Additional notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// Completion snippet for IDE
    pub snippet: String,
}

/// Schema for a parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    /// Parameter name
    pub name: String,

    /// Parameter type
    #[serde(rename = "type")]
    pub type_name: String,

    /// Parameter description
    pub description: String,

    /// Whether optional
    pub optional: bool,

    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// Schema for return type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnTypeSchema {
    /// Return type name
    #[serde(rename = "type")]
    pub type_name: String,

    /// Return description
    pub description: String,
}

/// Schema for a type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeSchema {
    /// Type name
    pub name: String,

    /// Type description
    pub description: String,

    /// Properties
    pub properties: Vec<PropertySchema>,

    /// Methods
    pub methods: Vec<FunctionSchema>,

    /// Constructors
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub constructors: Vec<FunctionSchema>,

    /// Examples
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,

    /// Version added
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
}

/// Schema for a property.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    /// Property name
    pub name: String,

    /// Property type
    #[serde(rename = "type")]
    pub type_name: String,

    /// Property description
    pub description: String,

    /// Whether read-only
    pub readonly: bool,
}

/// Autocomplete schema for IDE support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteSchema {
    /// All function completions
    pub functions: Vec<CompletionItem>,

    /// All type completions
    pub types: Vec<CompletionItem>,

    /// All keyword completions
    pub keywords: Vec<String>,
}

/// Single completion item for autocomplete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    /// Display label
    pub label: String,

    /// Kind of completion (function, type, etc.)
    pub kind: String,

    /// Detail information
    pub detail: String,

    /// Documentation string
    pub documentation: String,

    /// Insert text (snippet with placeholders)
    pub insert_text: String,

    /// Insert text format (snippet or plaintext)
    pub insert_text_format: String,
}

/// Generates JSON schema documentation for all registered API documentation.
///
/// # Returns
/// A JSON string containing the complete schema suitable for IDE integration.
///
/// # Example
/// ```no_run
/// use keyrx_core::scripting::docs::generators::json::generate_json;
/// use std::fs;
/// let json = generate_json();
/// fs::write("api_schema.json", json).unwrap();
/// ```
pub fn generate_json() -> String {
    let schema = build_schema();
    serde_json::to_string_pretty(&schema)
        .unwrap_or_else(|e| format!(r#"{{"error": "Failed to serialize schema: {}"}}"#, e))
}

/// Builds the complete JSON schema from the registry.
fn build_schema() -> JsonSchema {
    let mut modules = BTreeMap::new();

    // Collect functions by module
    for func in registry::all_functions() {
        let module_name = func.module.clone();
        let entry = modules
            .entry(module_name.clone())
            .or_insert_with(|| ModuleSchema {
                name: module_name,
                description: None,
                functions: Vec::new(),
                types: Vec::new(),
            });
        entry.functions.push(convert_function(&func));
    }

    // Collect types by module
    for type_doc in registry::all_types() {
        let module_name = type_doc.module.clone();
        let entry = modules
            .entry(module_name.clone())
            .or_insert_with(|| ModuleSchema {
                name: module_name,
                description: None,
                functions: Vec::new(),
                types: Vec::new(),
            });
        entry.types.push(convert_type(&type_doc));
    }

    // Add module-level documentation
    for module_doc in registry::all_modules() {
        if let Some(entry) = modules.get_mut(&module_doc.name) {
            entry.description = Some(module_doc.description);
        }
    }

    JsonSchema {
        schema: "https://json-schema.org/draft/2020-12/schema".to_string(),
        version: "1.0.0".to_string(),
        modules,
        autocomplete: build_autocomplete_schema(),
    }
}

/// Converts a FunctionDoc to FunctionSchema.
fn convert_function(func: &FunctionDoc) -> FunctionSchema {
    let parameters: Vec<ParameterSchema> = func
        .parameters
        .iter()
        .map(|p| ParameterSchema {
            name: p.name.clone(),
            type_name: p.type_name.clone(),
            description: p.description.clone(),
            optional: p.optional,
            default: p.default.clone(),
        })
        .collect();

    let returns = func.returns.as_ref().map(|r| ReturnTypeSchema {
        type_name: r.type_name.clone(),
        description: r.description.clone(),
    });

    let snippet = generate_snippet(&func.name, &func.parameters);

    FunctionSchema {
        name: func.name.clone(),
        description: func.description.clone(),
        parameters,
        returns,
        examples: func.examples.clone(),
        deprecated: func.deprecated.clone(),
        since: func.since.clone(),
        notes: func.notes.clone(),
        snippet,
    }
}

/// Converts a TypeDoc to TypeSchema.
fn convert_type(type_doc: &TypeDoc) -> TypeSchema {
    let properties: Vec<PropertySchema> =
        type_doc.properties.iter().map(convert_property).collect();

    let methods: Vec<FunctionSchema> = type_doc.methods.iter().map(convert_function).collect();

    let constructors: Vec<FunctionSchema> =
        type_doc.constructors.iter().map(convert_function).collect();

    TypeSchema {
        name: type_doc.name.clone(),
        description: type_doc.description.clone(),
        properties,
        methods,
        constructors,
        examples: type_doc.examples.clone(),
        since: type_doc.since.clone(),
    }
}

/// Converts a PropertyDoc to PropertySchema.
fn convert_property(prop: &PropertyDoc) -> PropertySchema {
    PropertySchema {
        name: prop.name.clone(),
        type_name: prop.type_name.clone(),
        description: prop.description.clone(),
        readonly: prop.readonly,
    }
}

/// Generates a completion snippet for a function.
fn generate_snippet(name: &str, parameters: &[crate::scripting::docs::ParamDoc]) -> String {
    if parameters.is_empty() {
        return format!("{}()", name);
    }

    let params = parameters
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let placeholder = i + 1;
            if p.optional {
                if let Some(default) = &p.default {
                    format!("${{{}:{}}}", placeholder, default)
                } else {
                    format!("${{{}:{}}}", placeholder, p.name)
                }
            } else {
                format!("${{{}:{}}}", placeholder, p.name)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!("{}({})", name, params)
}

/// Builds the autocomplete schema with all completions.
fn build_autocomplete_schema() -> AutocompleteSchema {
    let mut functions = Vec::new();
    let mut types = Vec::new();

    // Function completions
    for func in registry::all_functions() {
        let signature = format_function_signature(&func);
        let documentation = build_function_documentation(&func);
        let snippet = generate_snippet(&func.name, &func.parameters);

        functions.push(CompletionItem {
            label: func.name.clone(),
            kind: "function".to_string(),
            detail: signature,
            documentation,
            insert_text: snippet,
            insert_text_format: "snippet".to_string(),
        });
    }

    // Type completions
    for type_doc in registry::all_types() {
        types.push(CompletionItem {
            label: type_doc.name.clone(),
            kind: "class".to_string(),
            detail: format!("type {}", type_doc.name),
            documentation: type_doc.description.clone(),
            insert_text: type_doc.name.clone(),
            insert_text_format: "plaintext".to_string(),
        });
    }

    // Rhai keywords
    let keywords = vec![
        "let", "const", "fn", "if", "else", "for", "while", "loop", "break", "continue", "return",
        "true", "false", "null", "import", "export", "as", "private",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    AutocompleteSchema {
        functions,
        types,
        keywords,
    }
}

/// Formats a function signature for display.
fn format_function_signature(func: &FunctionDoc) -> String {
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

    format!("{}({}){}", func.name, params, return_type)
}

/// Builds documentation string for a function.
fn build_function_documentation(func: &FunctionDoc) -> String {
    let mut docs = vec![func.description.clone()];

    if let Some(deprecated) = &func.deprecated {
        docs.push(format!("\n⚠️ DEPRECATED: {}", deprecated));
    }

    if !func.parameters.is_empty() {
        docs.push("\n\nParameters:".to_string());
        for param in &func.parameters {
            let optional = if param.optional { " (optional)" } else { "" };
            docs.push(format!(
                "\n- {}: {}{} - {}",
                param.name, param.type_name, optional, param.description
            ));
        }
    }

    if let Some(returns) = &func.returns {
        docs.push(format!(
            "\n\nReturns: {} - {}",
            returns.type_name, returns.description
        ));
    }

    if !func.examples.is_empty() {
        docs.push("\n\nExamples:".to_string());
        for example in &func.examples {
            docs.push(format!("\n```rhai\n{}\n```", example));
        }
    }

    docs.join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::docs::{FunctionSignature, ParamDoc, ReturnDoc};

    #[test]
    fn test_generate_json_structure() {
        registry::initialize();
        registry::clear();

        let json = generate_json();

        // Should be valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());

        // Should contain schema version
        assert!(json.contains("$schema"));
        assert!(json.contains("version"));
        assert!(json.contains("modules"));
        assert!(json.contains("autocomplete"));
    }

    #[test]
    fn test_generate_snippet_no_params() {
        let snippet = generate_snippet("test_func", &[]);
        assert_eq!(snippet, "test_func()");
    }

    #[test]
    fn test_generate_snippet_with_params() {
        let params = vec![
            ParamDoc {
                name: "key".to_string(),
                type_name: "KeyCode".to_string(),
                description: "The key".to_string(),
                optional: false,
                default: None,
            },
            ParamDoc {
                name: "count".to_string(),
                type_name: "int".to_string(),
                description: "The count".to_string(),
                optional: true,
                default: Some("1".to_string()),
            },
        ];

        let snippet = generate_snippet("emit_key", &params);
        assert_eq!(snippet, "emit_key(${1:key}, ${2:1})");
    }

    #[test]
    fn test_convert_function() {
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

        let schema = convert_function(&func);

        assert_eq!(schema.name, "emit_key");
        assert_eq!(schema.description, "Emits a key press event");
        assert_eq!(schema.parameters.len(), 1);
        assert_eq!(schema.parameters[0].name, "key");
        assert!(schema.returns.is_some());
        assert_eq!(schema.snippet, "emit_key(${1:key})");
    }

    #[test]
    fn test_convert_type() {
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

        let schema = convert_type(&type_doc);

        assert_eq!(schema.name, "KeyCode");
        assert_eq!(schema.description, "Represents a keyboard key");
        assert_eq!(schema.properties.len(), 1);
        assert_eq!(schema.properties[0].name, "value");
        assert!(schema.properties[0].readonly);
    }

    #[test]
    fn test_format_function_signature() {
        let func = FunctionDoc {
            name: "emit_key".to_string(),
            module: "keys".to_string(),
            signature: FunctionSignature {
                params: vec![
                    ("key".to_string(), "KeyCode".to_string()),
                    ("count".to_string(), "int".to_string()),
                ],
                return_type: Some("()".to_string()),
            },
            description: "Test".to_string(),
            parameters: vec![],
            returns: None,
            examples: vec![],
            since: None,
            deprecated: None,
            notes: None,
        };

        let sig = format_function_signature(&func);
        assert_eq!(sig, "emit_key(key: KeyCode, count: int) -> ()");
    }

    #[test]
    fn test_build_function_documentation() {
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

        let docs = build_function_documentation(&func);

        assert!(docs.contains("Emits a key press event"));
        assert!(docs.contains("Parameters:"));
        assert!(docs.contains("key: KeyCode"));
        assert!(docs.contains("Returns: ()"));
        assert!(docs.contains("Examples:"));
        assert!(docs.contains("emit_key(Key::A);"));
    }

    #[test]
    fn test_build_schema_empty_registry() {
        registry::initialize();
        registry::clear();

        let schema = build_schema();

        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(schema.version, "1.0.0");
        assert!(schema.modules.is_empty());
    }

    #[test]
    fn test_autocomplete_schema_keywords() {
        registry::initialize();
        registry::clear();

        let autocomplete = build_autocomplete_schema();

        assert!(autocomplete.keywords.contains(&"let".to_string()));
        assert!(autocomplete.keywords.contains(&"fn".to_string()));
        assert!(autocomplete.keywords.contains(&"if".to_string()));
        assert!(autocomplete.keywords.contains(&"return".to_string()));
    }

    #[test]
    fn test_completion_item_structure() {
        registry::initialize();
        registry::clear();

        let func = FunctionDoc {
            name: "test_func".to_string(),
            module: "test".to_string(),
            signature: FunctionSignature {
                params: vec![],
                return_type: None,
            },
            description: "Test function".to_string(),
            parameters: vec![],
            returns: None,
            examples: vec![],
            since: None,
            deprecated: None,
            notes: None,
        };

        registry::register_function(func);

        let autocomplete = build_autocomplete_schema();

        assert!(!autocomplete.functions.is_empty());
        let item = &autocomplete.functions[0];
        assert_eq!(item.kind, "function");
        assert_eq!(item.insert_text_format, "snippet");
    }

    #[test]
    fn test_deprecated_function_in_docs() {
        let func = FunctionDoc {
            name: "old_func".to_string(),
            module: "test".to_string(),
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

        let docs = build_function_documentation(&func);

        assert!(docs.contains("⚠️ DEPRECATED"));
        assert!(docs.contains("Use new_func instead"));
    }

    #[test]
    fn test_json_serialization_roundtrip() {
        let schema = JsonSchema {
            schema: "https://json-schema.org/draft/2020-12/schema".to_string(),
            version: "1.0.0".to_string(),
            modules: BTreeMap::new(),
            autocomplete: AutocompleteSchema {
                functions: vec![],
                types: vec![],
                keywords: vec!["let".to_string()],
            },
        };

        let json = serde_json::to_string(&schema).unwrap();
        let deserialized: JsonSchema = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, "1.0.0");
        assert_eq!(deserialized.autocomplete.keywords.len(), 1);
    }
}
