//! Documentation data structures for Rhai API.
//!
//! This module defines the core types for storing and managing API documentation.
//! These types are serializable and can be used to generate documentation in
//! various formats (HTML, Markdown, JSON).

use serde::{Deserialize, Serialize};

/// Documentation for a single Rhai function.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionDoc {
    /// Function name as exposed to Rhai
    pub name: String,

    /// Module this function belongs to (e.g., "keys", "layers")
    pub module: String,

    /// Function signature with parameter and return types
    pub signature: FunctionSignature,

    /// Main description of what this function does
    pub description: String,

    /// Detailed parameter documentation
    pub parameters: Vec<ParamDoc>,

    /// Return value documentation
    pub returns: Option<ReturnDoc>,

    /// Example code snippets demonstrating usage
    pub examples: Vec<String>,

    /// Version when this function was added (e.g., "0.1.0")
    pub since: Option<String>,

    /// Deprecation message if this function is deprecated
    pub deprecated: Option<String>,

    /// Additional notes or warnings
    pub notes: Option<String>,
}

/// Function signature including parameters and return type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionSignature {
    /// Parameter list with (name, type) pairs
    pub params: Vec<(String, String)>,

    /// Return type, if any
    pub return_type: Option<String>,
}

/// Documentation for a function parameter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParamDoc {
    /// Parameter name
    pub name: String,

    /// Parameter type (e.g., "int", "KeyCode", "String")
    pub type_name: String,

    /// Description of what this parameter does
    pub description: String,

    /// Whether this parameter is optional
    pub optional: bool,

    /// Default value if parameter is optional
    pub default: Option<String>,
}

/// Documentation for a function's return value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReturnDoc {
    /// Return type name
    pub type_name: String,

    /// Description of what is returned
    pub description: String,
}

/// Documentation for a Rhai type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TypeDoc {
    /// Type name as exposed to Rhai
    pub name: String,

    /// Description of what this type represents
    pub description: String,

    /// Methods available on this type
    pub methods: Vec<FunctionDoc>,

    /// Properties accessible on this type
    pub properties: Vec<PropertyDoc>,

    /// Constructor functions for this type
    pub constructors: Vec<FunctionDoc>,

    /// Module this type belongs to
    pub module: String,

    /// Version when this type was added
    pub since: Option<String>,

    /// Example usage of this type
    pub examples: Vec<String>,
}

/// Documentation for a type property.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PropertyDoc {
    /// Property name
    pub name: String,

    /// Property type
    pub type_name: String,

    /// Description of what this property represents
    pub description: String,

    /// Whether this property is read-only
    pub readonly: bool,
}

/// Documentation for a module grouping related functions and types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleDoc {
    /// Module name (e.g., "keys", "layers", "timing")
    pub name: String,

    /// Description of what this module provides
    pub description: String,

    /// Functions in this module
    pub functions: Vec<FunctionDoc>,

    /// Types defined in this module
    pub types: Vec<TypeDoc>,
}

/// Search result from documentation search.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    /// Type of result found
    pub kind: SearchResultKind,

    /// Name of the item
    pub name: String,

    /// Module containing the item
    pub module: String,

    /// Brief description
    pub description: String,

    /// Relevance score (0.0 to 1.0)
    pub score: f64,
}

/// Type of search result.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SearchResultKind {
    /// A function
    Function,

    /// A type
    Type,

    /// A property
    Property,

    /// An example
    Example,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_doc_serialization() {
        let doc = FunctionDoc {
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

        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: FunctionDoc = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, deserialized);
    }

    #[test]
    fn test_type_doc_serialization() {
        let doc = TypeDoc {
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

        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: TypeDoc = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, deserialized);
    }

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            kind: SearchResultKind::Function,
            name: "emit_key".to_string(),
            module: "keys".to_string(),
            description: "Emits a key press event".to_string(),
            score: 0.95,
        };

        assert_eq!(result.kind, SearchResultKind::Function);
        assert!(result.score > 0.9);
    }

    #[test]
    fn test_module_doc_structure() {
        let module = ModuleDoc {
            name: "keys".to_string(),
            description: "Key emission functions".to_string(),
            functions: vec![],
            types: vec![],
        };

        assert_eq!(module.name, "keys");
        assert!(module.functions.is_empty());
    }
}
