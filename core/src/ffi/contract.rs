// FFI Contract Schema Types and Validation
//
// This module provides runtime contract loading and validation for FFI functions.
// Contracts define function signatures, parameters, return types, and events.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// FFI Contract for a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiContract {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub version: String,
    pub domain: String,
    pub description: String,
    pub protocol_version: u32,
    pub functions: Vec<FunctionContract>,
    #[serde(default)]
    pub types: HashMap<String, TypeDefinition>,
    #[serde(default)]
    pub events: Vec<EventContract>,
}

/// Function contract definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionContract {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_name: Option<String>,
    pub parameters: Vec<ParameterContract>,
    pub returns: TypeDefinition,
    pub errors: Vec<ErrorContract>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events_emitted: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<ExampleContract>,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_version: Option<String>,
}

impl FunctionContract {
    /// Get the full Rust function name
    pub fn rust_function_name(&self) -> String {
        self.rust_name
            .clone()
            .unwrap_or_else(|| format!("keyrx_{}_{}", self.domain_from_context(), self.name))
    }

    fn domain_from_context(&self) -> &str {
        // This will be set by the parent contract
        "unknown"
    }
}

/// Parameter contract definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterContract {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<Constraints>,
}

/// Type definition for parameters and returns
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TypeDefinition {
    Primitive {
        #[serde(rename = "type")]
        type_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        constraints: Option<Constraints>,
    },
    Object {
        #[serde(rename = "type")]
        type_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        properties: HashMap<String, Box<TypeDefinition>>,
    },
    Array {
        #[serde(rename = "type")]
        type_name: String,
        items: Box<TypeDefinition>,
        #[serde(skip_serializing_if = "Option::is_none")]
        constraints: Option<Constraints>,
    },
    Enum {
        #[serde(rename = "type")]
        type_name: String,
        values: Vec<String>,
    },
}

impl TypeDefinition {
    pub fn type_name(&self) -> &str {
        match self {
            TypeDefinition::Primitive { type_name, .. } => type_name,
            TypeDefinition::Object { type_name, .. } => type_name,
            TypeDefinition::Array { type_name, .. } => type_name,
            TypeDefinition::Enum { type_name, .. } => type_name,
        }
    }

    pub fn is_primitive(&self) -> bool {
        matches!(self, TypeDefinition::Primitive { .. })
    }

    pub fn is_object(&self) -> bool {
        matches!(self, TypeDefinition::Object { .. })
    }

    pub fn is_array(&self) -> bool {
        matches!(self, TypeDefinition::Array { .. })
    }
}

/// Constraints for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    // Numeric constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<serde_json::Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<serde_json::Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<serde_json::Number>,

    // String constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "enum")]
    pub enum_values: Option<Vec<String>>,

    // Array constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,
}

/// Error contract definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContract {
    pub code: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details_schema: Option<TypeDefinition>,
}

/// Event contract definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContract {
    pub name: String,
    pub description: String,
    pub payload: TypeDefinition,
}

/// Example input/output for documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleContract {
    pub input: serde_json::Value,
    pub output: serde_json::Value,
}

impl FfiContract {
    /// Load a contract from a JSON file
    pub fn from_file(path: &std::path::Path) -> Result<Self, ContractError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ContractError::IoError(e.to_string()))?;
        Self::from_json(&content)
    }

    /// Load a contract from JSON string
    pub fn from_json(json: &str) -> Result<Self, ContractError> {
        serde_json::from_str(json).map_err(|e| ContractError::ParseError(e.to_string()))
    }

    /// Get a function contract by name
    pub fn get_function(&self, name: &str) -> Option<&FunctionContract> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Get an event contract by name
    pub fn get_event(&self, name: &str) -> Option<&EventContract> {
        self.events.iter().find(|e| e.name == name)
    }

    /// Validate contract schema
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate domain name
        if self.domain.is_empty() {
            errors.push(ValidationError::EmptyField("domain".to_string()));
        }

        // Validate protocol version
        if self.protocol_version == 0 {
            errors.push(ValidationError::InvalidProtocolVersion(0));
        }

        // Validate functions
        for func in &self.functions {
            if func.name.is_empty() {
                errors.push(ValidationError::EmptyField(format!(
                    "function name in domain '{}'",
                    self.domain
                )));
            }

            // Validate parameters
            for param in &func.parameters {
                if param.name.is_empty() {
                    errors.push(ValidationError::EmptyField(format!(
                        "parameter name in function '{}'",
                        func.name
                    )));
                }

                if param.param_type.is_empty() {
                    errors.push(ValidationError::EmptyField(format!(
                        "parameter type in function '{}', param '{}'",
                        func.name, param.name
                    )));
                }
            }

            // Validate return type
            if func.returns.type_name().is_empty() {
                errors.push(ValidationError::EmptyField(format!(
                    "return type in function '{}'",
                    func.name
                )));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Contract registry for runtime access
#[derive(Debug, Default)]
pub struct ContractRegistry {
    contracts: HashMap<String, FfiContract>,
}

impl ContractRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a contract for a domain
    pub fn register(&mut self, contract: FfiContract) {
        let domain = contract.domain.clone();
        self.contracts.insert(domain, contract);
    }

    /// Get a contract by domain name
    pub fn get(&self, domain: &str) -> Option<&FfiContract> {
        self.contracts.get(domain)
    }

    /// Get all registered domains
    pub fn domains(&self) -> Vec<&str> {
        self.contracts.keys().map(|s| s.as_str()).collect()
    }

    /// Get all contracts
    pub fn all_contracts(&self) -> &HashMap<String, FfiContract> {
        &self.contracts
    }

    /// Load all contracts from a directory
    pub fn load_from_dir(dir: &std::path::Path) -> Result<Self, ContractError> {
        let mut registry = Self::new();

        if !dir.exists() {
            return Err(ContractError::IoError(format!(
                "Contract directory not found: {:?}",
                dir
            )));
        }

        for entry in std::fs::read_dir(dir).map_err(|e| ContractError::IoError(e.to_string()))? {
            let entry = entry.map_err(|e| ContractError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json")
                && path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.ends_with(".ffi-contract.json"))
                    .unwrap_or(false)
            {
                match FfiContract::from_file(&path) {
                    Ok(contract) => registry.register(contract),
                    Err(_e) => {
                        // Failed to load contract - skip this file
                        // Log error via tracing in production
                    }
                }
            }
        }

        Ok(registry)
    }

    /// Serialize all contracts to JSON
    pub fn to_json(&self) -> Result<String, ContractError> {
        serde_json::to_string_pretty(&self.contracts)
            .map_err(|e| ContractError::SerializationError(e.to_string()))
    }
}

/// Contract-related errors
#[derive(Debug, Clone)]
pub enum ContractError {
    IoError(String),
    ParseError(String),
    SerializationError(String),
    ValidationError(Vec<ValidationError>),
}

impl std::fmt::Display for ContractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractError::IoError(e) => write!(f, "IO error: {}", e),
            ContractError::ParseError(e) => write!(f, "Parse error: {}", e),
            ContractError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            ContractError::ValidationError(errors) => {
                write!(f, "Validation errors: ")?;
                for (i, err) in errors.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", err)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ContractError {}

/// Validation errors
#[derive(Debug, Clone)]
pub enum ValidationError {
    EmptyField(String),
    InvalidType(String),
    InvalidProtocolVersion(u32),
    ConstraintViolation(String),
    MissingFunction(String),
    DuplicateFunction(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyField(field) => write!(f, "Empty field: {}", field),
            ValidationError::InvalidType(msg) => write!(f, "Invalid type: {}", msg),
            ValidationError::InvalidProtocolVersion(v) => {
                write!(f, "Invalid protocol version: {}", v)
            }
            ValidationError::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
            ValidationError::MissingFunction(name) => write!(f, "Missing function: {}", name),
            ValidationError::DuplicateFunction(name) => write!(f, "Duplicate function: {}", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_discovery_contract() {
        let contract_json = include_str!("contracts/discovery.ffi-contract.json");
        let contract = FfiContract::from_json(contract_json).expect("Failed to parse contract");

        assert_eq!(contract.domain, "discovery");
        assert_eq!(contract.protocol_version, 1);
        assert!(!contract.functions.is_empty());
    }

    #[test]
    fn test_contract_validation() {
        let contract_json = include_str!("contracts/discovery.ffi-contract.json");
        let contract = FfiContract::from_json(contract_json).expect("Failed to parse contract");

        assert!(contract.validate().is_ok());
    }

    #[test]
    fn test_get_function() {
        let contract_json = include_str!("contracts/discovery.ffi-contract.json");
        let contract = FfiContract::from_json(contract_json).expect("Failed to parse contract");

        let func = contract
            .get_function("start_discovery")
            .expect("Function not found");
        assert_eq!(func.name, "start_discovery");
        assert_eq!(func.parameters.len(), 2);
    }

    #[test]
    fn test_contract_registry() {
        let mut registry = ContractRegistry::new();

        let contract_json = include_str!("contracts/discovery.ffi-contract.json");
        let contract = FfiContract::from_json(contract_json).expect("Failed to parse contract");

        registry.register(contract);

        assert_eq!(registry.domains().len(), 1);
        assert!(registry.get("discovery").is_some());
    }
}
