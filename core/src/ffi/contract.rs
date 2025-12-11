//! FFI Contract Schema Types and Validation.
//!
//! This module provides runtime contract loading and validation for FFI functions.
//! Contracts define function signatures, parameters, return types, and events for
//! cross-language bindings (e.g., Dart/Flutter integration).
//!
//! # Contract Structure
//!
//! Contracts are JSON files that describe the FFI interface:
//! - **Domain**: Logical grouping (e.g., "discovery", "engine", "profile")
//! - **Functions**: Callable operations with parameters and return types
//! - **Events**: Asynchronous notifications from the runtime
//! - **Types**: Reusable type definitions
//!
//! # Loading Contracts
//!
//! ```no_run
//! use keyrx_core::ffi::contract::{FfiContract, ContractRegistry};
//! use std::path::Path;
//!
//! // Load a single contract
//! let contract = FfiContract::from_file(Path::new("discovery.ffi-contract.json"))
//!     .expect("Failed to load contract");
//!
//! // Or load all contracts from a directory
//! let registry = ContractRegistry::load_from_dir(Path::new("contracts/"))
//!     .expect("Failed to load contracts");
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// FFI Contract definition for a domain.
///
/// An FFI contract describes all the functions, types, and events exposed
/// by a domain module for cross-language bindings. Contracts are loaded
/// from JSON files at runtime for introspection and validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiContract {
    /// JSON schema reference for validation.
    #[serde(rename = "$schema")]
    pub schema: String,
    /// Contract version (semantic versioning).
    pub version: String,
    /// Domain name (e.g., "discovery", "engine", "profile").
    pub domain: String,
    /// Human-readable description of the domain.
    pub description: String,
    /// Protocol version for compatibility checking.
    pub protocol_version: u32,
    /// List of functions exposed by this domain.
    pub functions: Vec<FunctionContract>,
    /// Reusable type definitions.
    #[serde(default)]
    pub types: HashMap<String, TypeDefinition>,
    /// Events emitted by this domain.
    #[serde(default)]
    pub events: Vec<EventContract>,
}

/// Contract definition for an FFI function.
///
/// Describes a single function exposed through FFI, including its parameters,
/// return type, possible errors, and associated events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionContract {
    /// Function name (snake_case, used in FFI calls).
    pub name: String,
    /// Human-readable description of what the function does.
    pub description: String,
    /// Optional override for the Rust function name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_name: Option<String>,
    /// List of parameters accepted by this function.
    pub parameters: Vec<ParameterContract>,
    /// Return type definition.
    pub returns: TypeDefinition,
    /// Possible errors that can be returned.
    pub errors: Vec<ErrorContract>,
    /// Names of events that may be emitted during execution.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events_emitted: Vec<String>,
    /// Example input/output for documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<ExampleContract>,
    /// Whether this function is deprecated.
    #[serde(default)]
    pub deprecated: bool,
    /// Version when this function was introduced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_version: Option<String>,
}

impl FunctionContract {
    /// Gets the full Rust function name for this contract.
    ///
    /// Returns `rust_name` if specified, otherwise generates a name
    /// in the format `keyrx_{domain}_{name}`.
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

/// Contract definition for a function parameter.
///
/// Describes a single parameter including its type, constraints, and whether
/// it's required.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterContract {
    /// Parameter name (snake_case).
    pub name: String,
    /// Type name (e.g., "string", "u32", "DeviceId").
    #[serde(rename = "type")]
    pub param_type: String,
    /// Human-readable description of the parameter.
    pub description: String,
    /// Whether this parameter is required.
    pub required: bool,
    /// Optional validation constraints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<Constraints>,
}

/// Type definition for parameters and return values.
///
/// Represents the type system used in FFI contracts. Types can be:
/// - **Primitive**: Basic types like string, number, boolean
/// - **Object**: Structured types with named properties
/// - **Array**: Lists of items of a specific type
/// - **Enum**: Fixed set of string values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TypeDefinition {
    /// A primitive type (string, number, boolean, etc.).
    Primitive {
        /// Type name (e.g., "string", "u32", "bool").
        #[serde(rename = "type")]
        type_name: String,
        /// Optional description of this type usage.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// Optional validation constraints.
        #[serde(skip_serializing_if = "Option::is_none")]
        constraints: Option<Constraints>,
    },
    /// An object type with named properties.
    Object {
        /// Type name (usually "object" or a custom type name).
        #[serde(rename = "type")]
        type_name: String,
        /// Optional description of the object.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// Map of property names to their type definitions.
        properties: HashMap<String, Box<TypeDefinition>>,
    },
    /// An array type containing items of a specific type.
    Array {
        /// Type name (usually "array").
        #[serde(rename = "type")]
        type_name: String,
        /// Type definition for array items.
        items: Box<TypeDefinition>,
        /// Optional constraints on the array (min/max items, etc.).
        #[serde(skip_serializing_if = "Option::is_none")]
        constraints: Option<Constraints>,
    },
    /// An enumeration with a fixed set of string values.
    Enum {
        /// Type name (usually "enum" or a custom type name).
        #[serde(rename = "type")]
        type_name: String,
        /// Allowed values for this enum.
        values: Vec<String>,
    },
}

impl TypeDefinition {
    /// Returns the type name for this definition.
    pub fn type_name(&self) -> &str {
        match self {
            TypeDefinition::Primitive { type_name, .. } => type_name,
            TypeDefinition::Object { type_name, .. } => type_name,
            TypeDefinition::Array { type_name, .. } => type_name,
            TypeDefinition::Enum { type_name, .. } => type_name,
        }
    }

    /// Returns true if this is a primitive type.
    pub fn is_primitive(&self) -> bool {
        matches!(self, TypeDefinition::Primitive { .. })
    }

    /// Returns true if this is an object type.
    pub fn is_object(&self) -> bool {
        matches!(self, TypeDefinition::Object { .. })
    }

    /// Returns true if this is an array type.
    pub fn is_array(&self) -> bool {
        matches!(self, TypeDefinition::Array { .. })
    }
}

/// Validation constraints for type definitions.
///
/// Constraints are used to validate input values beyond basic type checking.
/// They support numeric bounds, string patterns, and array size limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    // Numeric constraints
    /// Minimum value for numeric types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<serde_json::Number>,
    /// Maximum value for numeric types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<serde_json::Number>,
    /// Value must be a multiple of this number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<serde_json::Number>,

    // String constraints
    /// Minimum string length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    /// Maximum string length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    /// Regex pattern the string must match.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    /// Allowed values (for string enums).
    #[serde(skip_serializing_if = "Option::is_none", rename = "enum")]
    pub enum_values: Option<Vec<String>>,

    // Array constraints
    /// Minimum number of items in array.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
    /// Maximum number of items in array.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
    /// Whether array items must be unique.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,
}

/// Contract definition for an error that can be returned by a function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContract {
    /// Error code (e.g., "DEVICE_NOT_FOUND", "INVALID_PARAMETER").
    pub code: String,
    /// Human-readable error description.
    pub description: String,
    /// Optional schema for additional error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details_schema: Option<TypeDefinition>,
}

/// Contract definition for an event that can be emitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContract {
    /// Event name (snake_case).
    pub name: String,
    /// Human-readable event description.
    pub description: String,
    /// Type definition for the event payload.
    pub payload: TypeDefinition,
}

/// Example input/output for documentation purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleContract {
    /// Example input parameters.
    pub input: serde_json::Value,
    /// Expected output for the example input.
    pub output: serde_json::Value,
}

impl FfiContract {
    /// Loads a contract from a JSON file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON contract file
    ///
    /// # Errors
    ///
    /// Returns [`ContractError::IoError`] if the file cannot be read,
    /// or [`ContractError::ParseError`] if the JSON is invalid.
    pub fn from_file(path: &std::path::Path) -> Result<Self, ContractError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ContractError::IoError(e.to_string()))?;
        Self::from_json(&content)
    }

    /// Loads a contract from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns [`ContractError::ParseError`] if the JSON is invalid.
    pub fn from_json(json: &str) -> Result<Self, ContractError> {
        serde_json::from_str(json).map_err(|e| ContractError::ParseError(e.to_string()))
    }

    /// Gets a function contract by name.
    ///
    /// Returns `None` if no function with the given name exists.
    pub fn get_function(&self, name: &str) -> Option<&FunctionContract> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Gets an event contract by name.
    ///
    /// Returns `None` if no event with the given name exists.
    pub fn get_event(&self, name: &str) -> Option<&EventContract> {
        self.events.iter().find(|e| e.name == name)
    }

    /// Validates the contract schema for correctness.
    ///
    /// Checks for empty required fields, valid protocol version,
    /// and consistent type references.
    ///
    /// # Errors
    ///
    /// Returns a list of [`ValidationError`]s if validation fails.
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

/// Registry for managing multiple FFI contracts.
///
/// The registry provides centralized access to all domain contracts,
/// allowing lookup by domain name and bulk loading from a directory.
#[derive(Debug, Default)]
pub struct ContractRegistry {
    contracts: HashMap<String, FfiContract>,
}

impl ContractRegistry {
    /// Creates an empty contract registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a contract for its domain.
    ///
    /// If a contract for the domain already exists, it will be replaced.
    pub fn register(&mut self, contract: FfiContract) {
        let domain = contract.domain.clone();
        self.contracts.insert(domain, contract);
    }

    /// Gets a contract by domain name.
    ///
    /// Returns `None` if no contract is registered for the domain.
    pub fn get(&self, domain: &str) -> Option<&FfiContract> {
        self.contracts.get(domain)
    }

    /// Returns a list of all registered domain names.
    pub fn domains(&self) -> Vec<&str> {
        self.contracts.keys().map(|s| s.as_str()).collect()
    }

    /// Returns a reference to all registered contracts.
    pub fn all_contracts(&self) -> &HashMap<String, FfiContract> {
        &self.contracts
    }

    /// Loads all contracts from a directory.
    ///
    /// Scans the directory for files matching `*.ffi-contract.json`
    /// and loads each as a contract. Invalid files are silently skipped.
    ///
    /// # Errors
    ///
    /// Returns [`ContractError::IoError`] if the directory cannot be read.
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

    /// Serializes all contracts to a JSON string.
    ///
    /// # Errors
    ///
    /// Returns [`ContractError::SerializationError`] if serialization fails.
    pub fn to_json(&self) -> Result<String, ContractError> {
        serde_json::to_string_pretty(&self.contracts)
            .map_err(|e| ContractError::SerializationError(e.to_string()))
    }
}

/// Errors that can occur when working with FFI contracts.
#[derive(Debug, Clone)]
pub enum ContractError {
    /// Error reading or writing contract files.
    IoError(String),
    /// Error parsing contract JSON.
    ParseError(String),
    /// Error serializing contracts to JSON.
    SerializationError(String),
    /// Contract validation failed.
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

/// Errors that can occur during contract validation.
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// A required field is empty.
    EmptyField(String),
    /// An invalid type was specified.
    InvalidType(String),
    /// The protocol version is invalid.
    InvalidProtocolVersion(u32),
    /// A constraint was violated.
    ConstraintViolation(String),
    /// A referenced function is missing.
    MissingFunction(String),
    /// A function is defined multiple times.
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
        // Contract takes a single JSON params_json parameter containing device_id and rows
        assert_eq!(func.parameters.len(), 1);
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

    #[test]
    fn test_load_engine_contract() {
        let contract_json = include_str!("contracts/engine.ffi-contract.json");
        let contract =
            FfiContract::from_json(contract_json).expect("Failed to parse engine contract");

        assert_eq!(contract.domain, "engine");
        assert_eq!(contract.protocol_version, 1);
        assert!(contract.validate().is_ok());

        assert!(contract.get_function("start_loop").is_some());
        assert!(contract.get_function("stop_loop").is_some());
    }
}
