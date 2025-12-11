// FFI Introspection API
//
// Provides runtime metadata about available FFI functions, types, and events.
// This enables developer tools to dynamically discover and test FFI functions.

use crate::ffi::contract::{ContractRegistry, FfiContract};
use crate::ffi::error::{FfiError, FfiResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Global contract registry
static CONTRACT_REGISTRY: OnceLock<ContractRegistry> = OnceLock::new();

/// Initialize the contract registry
pub fn init_contracts() -> Result<(), FfiError> {
    let contracts_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ffi")
        .join("contracts");

    let registry = ContractRegistry::load_from_dir(&contracts_dir)
        .map_err(|e| FfiError::internal(format!("Failed to load contracts: {}", e)))?;

    CONTRACT_REGISTRY
        .set(registry)
        .map_err(|_| FfiError::internal("Contract registry already initialized"))?;

    Ok(())
}

/// Get the global contract registry
pub fn get_registry() -> Option<&'static ContractRegistry> {
    CONTRACT_REGISTRY.get()
}

/// Introspection metadata for all FFI functions
#[derive(Debug, Serialize, Deserialize)]
pub struct IntrospectionData {
    pub protocol_version: u32,
    pub domains: Vec<DomainMetadata>,
    pub total_functions: usize,
    pub total_events: usize,
}

/// Domain metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct DomainMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub functions: Vec<FunctionMetadata>,
    pub events: Vec<EventMetadata>,
}

/// Function metadata for introspection
#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionMetadata {
    pub name: String,
    pub rust_name: String,
    pub description: String,
    pub parameters: Vec<ParameterMetadata>,
    pub returns: TypeMetadata,
    pub errors: Vec<String>,
    pub events_emitted: Vec<String>,
    pub deprecated: bool,
    pub example: Option<ExampleMetadata>,
}

/// Parameter metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ParameterMetadata {
    pub name: String,
    pub type_name: String,
    pub description: String,
    pub required: bool,
    pub constraints: Option<serde_json::Value>,
}

/// Type metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct TypeMetadata {
    pub type_name: String,
    pub kind: String, // "primitive", "object", "array", "enum"
    pub description: Option<String>,
    pub properties: Option<HashMap<String, TypeMetadata>>,
    pub items: Option<Box<TypeMetadata>>,
}

/// Event metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct EventMetadata {
    pub name: String,
    pub description: String,
    pub payload: TypeMetadata,
}

/// Example metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ExampleMetadata {
    pub input: serde_json::Value,
    pub output: serde_json::Value,
}

/// Generate introspection data from contract registry
pub fn generate_introspection_data() -> FfiResult<IntrospectionData> {
    let registry =
        get_registry().ok_or_else(|| FfiError::internal("Contract registry not initialized"))?;

    let mut domains = Vec::new();
    let mut total_functions = 0;
    let mut total_events = 0;

    for contract in registry.all_contracts().values() {
        let domain_metadata = contract_to_domain_metadata(contract);
        total_functions += domain_metadata.functions.len();
        total_events += domain_metadata.events.len();
        domains.push(domain_metadata);
    }

    Ok(IntrospectionData {
        protocol_version: 1,
        domains,
        total_functions,
        total_events,
    })
}

fn contract_to_domain_metadata(contract: &FfiContract) -> DomainMetadata {
    DomainMetadata {
        name: contract.domain.clone(),
        description: contract.description.clone(),
        version: contract.version.clone(),
        functions: contract
            .functions
            .iter()
            .map(|f| FunctionMetadata {
                name: f.name.clone(),
                rust_name: f
                    .rust_name
                    .clone()
                    .unwrap_or_else(|| format!("keyrx_{}_{}", contract.domain, f.name)),
                description: f.description.clone(),
                parameters: f
                    .parameters
                    .iter()
                    .map(|p| ParameterMetadata {
                        name: p.name.clone(),
                        type_name: p.param_type.clone(),
                        description: p.description.clone(),
                        required: p.required,
                        constraints: p
                            .constraints
                            .as_ref()
                            .and_then(|c| serde_json::to_value(c).ok()),
                    })
                    .collect(),
                returns: type_def_to_metadata(&f.returns),
                errors: f.errors.iter().map(|e| e.code.clone()).collect(),
                events_emitted: f.events_emitted.clone(),
                deprecated: f.deprecated,
                example: f.example.as_ref().map(|ex| ExampleMetadata {
                    input: ex.input.clone(),
                    output: ex.output.clone(),
                }),
            })
            .collect(),
        events: contract
            .events
            .iter()
            .map(|e| EventMetadata {
                name: e.name.clone(),
                description: e.description.clone(),
                payload: type_def_to_metadata(&e.payload),
            })
            .collect(),
    }
}

fn type_def_to_metadata(type_def: &crate::ffi::contract::TypeDefinition) -> TypeMetadata {
    use crate::ffi::contract::TypeDefinition;

    match type_def {
        TypeDefinition::Primitive {
            type_name,
            description,
            ..
        } => TypeMetadata {
            type_name: type_name.clone(),
            kind: "primitive".to_string(),
            description: description.clone(),
            properties: None,
            items: None,
        },
        TypeDefinition::Object {
            type_name,
            description,
            properties,
        } => TypeMetadata {
            type_name: type_name.clone(),
            kind: "object".to_string(),
            description: description.clone(),
            properties: Some(
                properties
                    .iter()
                    .map(|(k, v)| (k.clone(), type_def_to_metadata(v)))
                    .collect(),
            ),
            items: None,
        },
        TypeDefinition::Array {
            type_name, items, ..
        } => TypeMetadata {
            type_name: type_name.clone(),
            kind: "array".to_string(),
            description: None,
            properties: None,
            items: Some(Box::new(type_def_to_metadata(items))),
        },
        TypeDefinition::Enum { type_name, .. } => TypeMetadata {
            type_name: type_name.clone(),
            kind: "enum".to_string(),
            description: None,
            properties: None,
            items: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Initialize contracts, ignoring "already initialized" error.
    /// This handles test ordering issues with the global OnceLock.
    fn ensure_contracts_initialized() {
        let _ = init_contracts(); // Ignore error if already initialized
    }

    #[test]
    fn test_init_contracts() {
        ensure_contracts_initialized();
        assert!(get_registry().is_some());
    }

    #[test]
    fn test_generate_introspection_data() {
        ensure_contracts_initialized();
        let data = generate_introspection_data().expect("Failed to generate introspection data");

        assert!(data.total_functions > 0);
        assert!(!data.domains.is_empty());
    }
}
