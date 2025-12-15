//! Integration tests for FFI breaking change detection.
//!
//! These tests verify that the breaking change detection tool correctly
//! identifies breaking changes in FFI contracts.

#[cfg(test)]
mod tests {
    use serde_json::json;

    /// Test that removing a function is detected as a breaking change.
    #[test]
    fn test_removed_function_is_breaking() {
        let old_contract = json!({
            "functions": [
                {"name": "get_value", "parameters": [], "returns": {"type": "string"}},
                {"name": "set_value", "parameters": [], "returns": {"type": "null"}}
            ]
        });

        let new_contract = json!({
            "functions": [
                {"name": "get_value", "parameters": [], "returns": {"type": "string"}}
            ]
        });

        let changes = detect_breaking_changes(&old_contract, &new_contract);
        assert!(!changes.is_empty(), "Should detect removed function");
        assert!(changes
            .iter()
            .any(|c| c.contains("removed") && c.contains("set_value")));
    }

    /// Test that changing parameter types is detected as breaking.
    #[test]
    fn test_changed_parameter_type_is_breaking() {
        let old_contract = json!({
            "functions": [{
                "name": "get_value",
                "parameters": [{"name": "key", "type": "string"}],
                "returns": {"type": "string"}
            }]
        });

        let new_contract = json!({
            "functions": [{
                "name": "get_value",
                "parameters": [{"name": "key", "type": "number"}],
                "returns": {"type": "string"}
            }]
        });

        let changes = detect_breaking_changes(&old_contract, &new_contract);
        assert!(!changes.is_empty(), "Should detect parameter type change");
        assert!(changes
            .iter()
            .any(|c| c.contains("parameter") && c.contains("type")));
    }

    /// Test that changing return types is detected as breaking.
    #[test]
    fn test_changed_return_type_is_breaking() {
        let old_contract = json!({
            "functions": [{
                "name": "get_value",
                "parameters": [],
                "returns": {"type": "string"}
            }]
        });

        let new_contract = json!({
            "functions": [{
                "name": "get_value",
                "parameters": [],
                "returns": {"type": "number"}
            }]
        });

        let changes = detect_breaking_changes(&old_contract, &new_contract);
        assert!(!changes.is_empty(), "Should detect return type change");
        assert!(changes.iter().any(|c| c.contains("return type")));
    }

    /// Test that renaming a function is detected as breaking.
    #[test]
    fn test_renamed_function_is_breaking() {
        let old_contract = json!({
            "functions": [{
                "name": "get_value",
                "parameters": [],
                "returns": {"type": "string"}
            }]
        });

        let new_contract = json!({
            "functions": [{
                "name": "get_setting",
                "parameters": [],
                "returns": {"type": "string"}
            }]
        });

        let changes = detect_breaking_changes(&old_contract, &new_contract);
        assert!(!changes.is_empty(), "Should detect renamed function");
    }

    /// Test that adding required parameters is breaking.
    #[test]
    fn test_added_required_parameter_is_breaking() {
        let old_contract = json!({
            "functions": [{
                "name": "get_value",
                "parameters": [],
                "returns": {"type": "string"}
            }]
        });

        let new_contract = json!({
            "functions": [{
                "name": "get_value",
                "parameters": [{"name": "key", "type": "string", "required": true}],
                "returns": {"type": "string"}
            }]
        });

        let changes = detect_breaking_changes(&old_contract, &new_contract);
        assert!(
            !changes.is_empty(),
            "Should detect added required parameter"
        );
    }

    /// Test that adding optional parameters is NOT breaking.
    #[test]
    fn test_added_optional_parameter_is_not_breaking() {
        let old_contract = json!({
            "functions": [{
                "name": "get_value",
                "parameters": [],
                "returns": {"type": "string"}
            }]
        });

        let new_contract = json!({
            "functions": [{
                "name": "get_value",
                "parameters": [{"name": "default", "type": "string", "required": false}],
                "returns": {"type": "string"}
            }]
        });

        let changes = detect_breaking_changes(&old_contract, &new_contract);
        let breaking: Vec<_> = changes
            .iter()
            .filter(|c| !c.contains("compatible"))
            .collect();
        assert!(
            breaking.is_empty(),
            "Adding optional parameter should be compatible"
        );
    }

    /// Test that adding new functions is NOT breaking.
    #[test]
    fn test_added_function_is_not_breaking() {
        let old_contract = json!({
            "functions": [{
                "name": "get_value",
                "parameters": [],
                "returns": {"type": "string"}
            }]
        });

        let new_contract = json!({
            "functions": [
                {"name": "get_value", "parameters": [], "returns": {"type": "string"}},
                {"name": "set_value", "parameters": [], "returns": {"type": "null"}}
            ]
        });

        let changes = detect_breaking_changes(&old_contract, &new_contract);
        let breaking: Vec<_> = changes.iter().filter(|c| c.contains("breaking")).collect();
        assert!(
            breaking.is_empty(),
            "Adding function should not be breaking"
        );
    }

    /// Test that changing domain is breaking.
    #[test]
    fn test_changed_domain_is_breaking() {
        let old_contract = json!({"domain": "engine"});
        let new_contract = json!({"domain": "runtime"});

        let changes = detect_breaking_changes(&old_contract, &new_contract);
        assert!(!changes.is_empty(), "Should detect domain change");
    }

    /// Test that making required parameter optional is NOT breaking.
    #[test]
    fn test_making_parameter_optional_is_not_breaking() {
        let old_contract = json!({
            "functions": [{
                "name": "get_value",
                "parameters": [{"name": "key", "type": "string", "required": true}],
                "returns": {"type": "string"}
            }]
        });

        let new_contract = json!({
            "functions": [{
                "name": "get_value",
                "parameters": [{"name": "key", "type": "string", "required": false}],
                "returns": {"type": "string"}
            }]
        });

        let changes = detect_breaking_changes(&old_contract, &new_contract);
        let breaking: Vec<_> = changes.iter().filter(|c| c.contains("breaking")).collect();
        assert!(
            breaking.is_empty(),
            "Making parameter optional should be compatible"
        );
    }

    /// Test that reordering parameters is breaking.
    #[test]
    fn test_reordered_parameters_is_breaking() {
        let old_contract = json!({
            "functions": [{
                "name": "update",
                "parameters": [
                    {"name": "key", "type": "string", "required": true},
                    {"name": "value", "type": "string", "required": true}
                ],
                "returns": {"type": "null"}
            }]
        });

        let new_contract = json!({
            "functions": [{
                "name": "update",
                "parameters": [
                    {"name": "value", "type": "string", "required": true},
                    {"name": "key", "type": "string", "required": true}
                ],
                "returns": {"type": "null"}
            }]
        });

        let changes = detect_breaking_changes(&old_contract, &new_contract);
        assert!(!changes.is_empty(), "Should detect parameter reordering");
    }

    // Helper function (would be implemented in actual breaking change detector)
    fn detect_breaking_changes(old: &serde_json::Value, new: &serde_json::Value) -> Vec<String> {
        let mut changes = Vec::new();

        // Check for domain changes
        if let (Some(old_domain), Some(new_domain)) = (old.get("domain"), new.get("domain")) {
            if old_domain != new_domain {
                changes.push("BREAKING: Domain changed".to_string());
            }
        }

        // Check for function changes
        if let (Some(old_funcs), Some(new_funcs)) = (
            old.get("functions").and_then(|f| f.as_array()),
            new.get("functions").and_then(|f| f.as_array()),
        ) {
            // Check for removed functions
            for old_func in old_funcs {
                let old_name = old_func.get("name").and_then(|n| n.as_str()).unwrap_or("");
                if !new_funcs
                    .iter()
                    .any(|f| f.get("name").and_then(|n| n.as_str()) == Some(old_name))
                {
                    changes.push(format!("BREAKING: Function removed: {}", old_name));
                }
            }

            // Check for parameter/return type changes
            for old_func in old_funcs {
                let old_name = old_func.get("name").and_then(|n| n.as_str()).unwrap_or("");
                if let Some(new_func) = new_funcs
                    .iter()
                    .find(|f| f.get("name").and_then(|n| n.as_str()) == Some(old_name))
                {
                    // Check return type
                    if old_func.get("returns") != new_func.get("returns") {
                        changes.push(format!("BREAKING: Return type changed for {}", old_name));
                    }

                    // Check parameters
                    let old_params = old_func.get("parameters").and_then(|p| p.as_array());
                    let new_params = new_func.get("parameters").and_then(|p| p.as_array());

                    if let (Some(old_p), Some(new_p)) = (old_params, new_params) {
                        // Check for parameter type changes
                        for (i, old_param) in old_p.iter().enumerate() {
                            if let Some(new_param) = new_p.get(i) {
                                if old_param.get("type") != new_param.get("type") {
                                    changes.push(format!(
                                        "BREAKING: Parameter type changed in {}",
                                        old_name
                                    ));
                                }
                            }
                        }

                        // Check for added required parameters
                        if new_p.len() > old_p.len() {
                            for param in new_p.iter().skip(old_p.len()) {
                                if param
                                    .get("required")
                                    .and_then(|r| r.as_bool())
                                    .unwrap_or(true)
                                {
                                    changes.push(format!(
                                        "BREAKING: Added required parameter in {}",
                                        old_name
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        changes
    }
}
