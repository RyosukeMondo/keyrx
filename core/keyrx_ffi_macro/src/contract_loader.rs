//! Contract loading for compile-time FFI generation.
//!
//! This module handles loading and parsing of `.ffi-contract.json` files
//! during macro expansion.

use proc_macro2::Span;
use serde::Deserialize;
use std::collections::HashMap;

/// FFI Contract loaded at compile time.
///
/// This is a subset of the runtime `FfiContract` optimized for code generation.
#[derive(Debug, Clone, Deserialize)]
pub struct FfiContract {
    pub version: String,
    pub domain: String,
    pub description: String,
    pub protocol_version: u32,
    pub functions: Vec<FunctionContract>,
    #[serde(default)]
    pub types: HashMap<String, TypeDefinition>,
}

/// Function definition from the contract.
#[derive(Debug, Clone, Deserialize)]
pub struct FunctionContract {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub rust_name: Option<String>,
    pub parameters: Vec<ParameterContract>,
    pub returns: TypeDefinition,
    #[serde(default)]
    pub errors: Vec<ErrorContract>,
}

impl FunctionContract {
    /// Get the FFI function name (rust_name or generated).
    pub fn ffi_name(&self, domain: &str) -> String {
        self.rust_name
            .clone()
            .unwrap_or_else(|| format!("keyrx_{}_{}", domain, self.name))
    }
}

/// Parameter definition from the contract.
#[derive(Debug, Clone, Deserialize)]
pub struct ParameterContract {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

/// Type definition for parameters and returns.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum TypeDefinition {
    Simple {
        #[serde(rename = "type")]
        type_name: String,
        #[serde(default)]
        description: Option<String>,
    },
    Object {
        #[serde(rename = "type")]
        type_name: String,
        #[serde(default)]
        description: Option<String>,
        properties: HashMap<String, Box<TypeDefinition>>,
    },
    Array {
        #[serde(rename = "type")]
        type_name: String,
        items: Box<TypeDefinition>,
    },
}

impl TypeDefinition {
    /// Get the type name.
    pub fn type_name(&self) -> &str {
        match self {
            TypeDefinition::Simple { type_name, .. } => type_name,
            TypeDefinition::Object { type_name, .. } => type_name,
            TypeDefinition::Array { type_name, .. } => type_name,
        }
    }
}

/// Error definition from the contract.
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorContract {
    pub code: String,
    pub description: String,
}

/// Load contract file for a domain at compile time.
///
/// Searches for the contract file in multiple possible locations relative to
/// `CARGO_MANIFEST_DIR`:
/// - Direct: `{MANIFEST_DIR}/src/ffi/contracts/{domain}.ffi-contract.json`
/// - Parent: `{MANIFEST_DIR}/../src/ffi/contracts/{domain}.ffi-contract.json`
///
/// This allows the macro to work when invoked from the `core` crate directly
/// or from the `keyrx_ffi_macro` subcrate.
///
/// # Arguments
///
/// * `domain` - The domain name (e.g., "config", "discovery")
/// * `span` - Span for error reporting
///
/// # Returns
///
/// * `Ok(FfiContract)` - Successfully loaded contract
/// * `Err(syn::Error)` - File not found or invalid JSON
pub fn load_contract_for_domain(domain: &str, span: Span) -> syn::Result<FfiContract> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        syn::Error::new(span, "CARGO_MANIFEST_DIR not set - cannot locate contracts")
    })?;

    let manifest_path = std::path::Path::new(&manifest_dir);
    let contract_filename = format!("{domain}.ffi-contract.json");

    // Try multiple possible locations for the contract file
    let candidate_paths = [
        // Direct: when called from core crate
        manifest_path
            .join("src/ffi/contracts")
            .join(&contract_filename),
        // Parent: when called from keyrx_ffi_macro subcrate
        manifest_path
            .parent()
            .map(|p| p.join("src/ffi/contracts").join(&contract_filename))
            .unwrap_or_default(),
    ];

    let (contract_path, content) = candidate_paths
        .iter()
        .filter(|p| !p.as_os_str().is_empty())
        .find_map(|path| {
            std::fs::read_to_string(path)
                .ok()
                .map(|c| (path.clone(), c))
        })
        .ok_or_else(|| {
            let paths: Vec<_> = candidate_paths
                .iter()
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| p.display().to_string())
                .collect();
            syn::Error::new(
                span,
                format!(
                    "failed to load contract for domain '{domain}'. Searched: {}",
                    paths.join(", ")
                ),
            )
        })?;

    serde_json::from_str(&content).map_err(|e| {
        syn::Error::new(
            span,
            format!(
                "failed to parse contract for domain '{domain}' at {}: {e}",
                contract_path.display()
            ),
        )
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_contract() {
        // Load the config contract directly for testing
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let contract_path = std::path::Path::new(manifest_dir)
            .parent()
            .expect("parent exists")
            .join("src/ffi/contracts/config.ffi-contract.json");

        let content = std::fs::read_to_string(&contract_path).expect("contract exists");
        let contract: FfiContract = serde_json::from_str(&content).expect("valid json");

        assert_eq!(contract.domain, "config");
        assert_eq!(contract.protocol_version, 1);
        assert!(!contract.functions.is_empty());
    }

    #[test]
    fn function_ffi_name_uses_rust_name() {
        let func = FunctionContract {
            name: "list_items".to_string(),
            description: "test".to_string(),
            rust_name: Some("keyrx_config_list_items".to_string()),
            parameters: vec![],
            returns: TypeDefinition::Simple {
                type_name: "string".to_string(),
                description: None,
            },
            errors: vec![],
        };

        assert_eq!(func.ffi_name("config"), "keyrx_config_list_items");
    }

    #[test]
    fn function_ffi_name_generates_default() {
        let func = FunctionContract {
            name: "list_items".to_string(),
            description: "test".to_string(),
            rust_name: None,
            parameters: vec![],
            returns: TypeDefinition::Simple {
                type_name: "string".to_string(),
                description: None,
            },
            errors: vec![],
        };

        assert_eq!(func.ffi_name("config"), "keyrx_config_list_items");
    }

    #[test]
    fn type_definition_type_name() {
        let simple = TypeDefinition::Simple {
            type_name: "string".to_string(),
            description: None,
        };
        assert_eq!(simple.type_name(), "string");

        let object = TypeDefinition::Object {
            type_name: "object".to_string(),
            description: None,
            properties: HashMap::new(),
        };
        assert_eq!(object.type_name(), "object");
    }

    #[test]
    fn load_contract_for_domain_success() {
        let span = Span::call_site();
        let contract = load_contract_for_domain("config", span).expect("should load");
        assert_eq!(contract.domain, "config");
    }

    #[test]
    fn load_contract_for_domain_not_found() {
        let span = Span::call_site();
        let result = load_contract_for_domain("nonexistent", span);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("failed to load contract"));
    }
}
