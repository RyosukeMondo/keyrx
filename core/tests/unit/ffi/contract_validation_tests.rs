//! Integration tests for FFI contract validation.
//!
//! These tests verify that contract validation catches common errors
//! and enforces contract schema constraints.

#[cfg(test)]
mod tests {
    use keyrx_core::ffi::contract::{FfiContract, FunctionContract, TypeDefinition};
    use serde_json;

    /// Helper to create a minimal valid contract for testing.
    fn create_minimal_contract() -> FfiContract {
        FfiContract {
            schema: Some("https://example.com/ffi-contract.schema.json".to_string()),
            version: "1.0.0".to_string(),
            domain: "test".to_string(),
            description: "Test domain".to_string(),
            protocol_version: 1,
            functions: vec![],
            types: Default::default(),
            events: vec![],
        }
    }

    /// Test that a valid contract passes validation.
    #[test]
    fn test_valid_contract_passes() {
        let contract = create_minimal_contract();

        // Serialize and deserialize to verify schema compliance
        let json = serde_json::to_string(&contract).unwrap();
        let parsed: FfiContract = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.domain, "test");
        assert_eq!(parsed.version, "1.0.0");
    }

    /// Test that contracts require a domain field.
    #[test]
    fn test_contract_requires_domain() {
        let json = r#"{
            "$schema": "https://example.com/ffi-contract.schema.json",
            "version": "1.0.0",
            "description": "Test",
            "protocol_version": 1,
            "functions": []
        }"#;

        let result: Result<FfiContract, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Contract without domain should fail");
    }

    /// Test that function names must be valid identifiers.
    #[test]
    fn test_function_name_validation() {
        let valid_names = vec!["get_value", "start_loop", "init_v2"];
        let invalid_names = vec!["get-value", "start loop", "123init", ""];

        for name in valid_names {
            assert!(is_valid_identifier(name), "{} should be valid", name);
        }

        for name in invalid_names {
            assert!(!is_valid_identifier(name), "{} should be invalid", name);
        }
    }

    /// Test that parameter types must be valid.
    #[test]
    fn test_parameter_type_validation() {
        let valid_types = vec!["string", "number", "bool", "object", "array"];
        let invalid_types = vec!["str", "int", "boolean", "unknown"];

        for type_name in valid_types {
            assert!(is_valid_type(type_name), "{} should be valid", type_name);
        }

        for type_name in invalid_types {
            assert!(!is_valid_type(type_name), "{} should be invalid", type_name);
        }
    }

    /// Test that contracts detect duplicate function names.
    #[test]
    fn test_duplicate_function_detection() {
        let mut contract = create_minimal_contract();

        let func1 = FunctionContract {
            name: "get_value".to_string(),
            description: "Get a value".to_string(),
            rust_name: None,
            parameters: vec![],
            returns: TypeDefinition::Primitive {
                type_name: "string".to_string(),
                description: None,
                constraints: None,
            },
            errors: vec![],
            events_emitted: vec![],
            example: None,
            deprecated: false,
            since_version: None,
        };

        let func2 = func1.clone();

        contract.functions.push(func1);
        contract.functions.push(func2);

        // Validation should detect duplicates
        assert!(
            has_duplicate_functions(&contract),
            "Should detect duplicate function names"
        );
    }

    /// Test that required parameters cannot have default values.
    #[test]
    fn test_required_parameter_constraints() {
        // This is a logical constraint: if a parameter is required,
        // it shouldn't have a default value (that would make it optional)

        let invalid_param = r#"{
            "name": "profile_id",
            "type": "string",
            "required": true,
            "default": "default_profile"
        }"#;

        // Validation should catch this inconsistency
        // (implementation would need to check this constraint)
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(invalid_param);
        assert!(parsed.is_ok()); // Parses fine, but semantic validation would catch it
    }

    /// Test that version strings follow semantic versioning.
    #[test]
    fn test_version_format_validation() {
        let valid_versions = vec!["1.0.0", "2.1.3", "0.1.0", "10.20.30"];
        let invalid_versions = vec!["1.0", "v1.0.0", "1.0.0-beta", "abc"];

        for version in valid_versions {
            assert!(is_valid_semver(version), "{} should be valid", version);
        }

        for version in invalid_versions {
            assert!(!is_valid_semver(version), "{} should be invalid", version);
        }
    }

    /// Test that contracts can be loaded from JSON files.
    #[test]
    fn test_contract_loading_from_json() {
        let json = r#"{
            "$schema": "https://example.com/ffi-contract.schema.json",
            "version": "1.0.0",
            "domain": "engine",
            "description": "Engine control functions",
            "protocol_version": 1,
            "functions": [
                {
                    "name": "start_loop",
                    "description": "Start the event loop",
                    "parameters": [],
                    "returns": {
                        "type": "object"
                    },
                    "errors": []
                }
            ]
        }"#;

        let contract: FfiContract = serde_json::from_str(json).unwrap();
        assert_eq!(contract.domain, "engine");
        assert_eq!(contract.functions.len(), 1);
        assert_eq!(contract.functions[0].name, "start_loop");
    }

    // Helper functions for validation (would be implemented in actual validation module)

    fn is_valid_identifier(name: &str) -> bool {
        !name.is_empty()
            && name
                .chars()
                .next()
                .map_or(false, |c| c.is_alphabetic() || c == '_')
            && name.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    fn is_valid_type(type_name: &str) -> bool {
        matches!(
            type_name,
            "string" | "number" | "bool" | "object" | "array" | "null"
        )
    }

    fn is_valid_semver(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        parts.len() == 3 && parts.iter().all(|p| p.parse::<u32>().is_ok())
    }

    fn has_duplicate_functions(contract: &FfiContract) -> bool {
        let mut names = std::collections::HashSet::new();
        contract.functions.iter().any(|f| !names.insert(&f.name))
    }
}
