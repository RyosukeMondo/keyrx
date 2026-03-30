//! MCP tool input types with JSON Schema derivations.
//!
//! Each struct defines the parameters for one MCP tool. The `schemars::JsonSchema`
//! derive generates the JSON Schema that rmcp returns in the `tools/list` response,
//! enabling AI agents to discover parameter types automatically.

use rmcp::schemars;

/// Input for `keyrx_list_profiles` — no parameters needed.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListProfilesInput {}

/// Input for `keyrx_get_profile_config`.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetProfileConfigInput {
    #[schemars(description = "Profile name")]
    pub name: String,
}

/// Input for `keyrx_set_profile_config`.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SetProfileConfigInput {
    #[schemars(description = "Profile name")]
    pub name: String,
    #[schemars(description = "Rhai DSL source code for the profile configuration")]
    pub source: String,
}

/// Input for `keyrx_activate_profile`.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ActivateProfileInput {
    #[schemars(description = "Profile name to activate")]
    pub name: String,
}

/// Input for `keyrx_create_profile`.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateProfileInput {
    #[schemars(description = "Name for the new profile")]
    pub name: String,
    #[schemars(
        description = "Template: blank, simple_remap, capslock_escape, vim_navigation, or gaming"
    )]
    pub template: Option<String>,
}

/// Input for `keyrx_validate_profile`.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ValidateProfileInput {
    #[schemars(description = "Profile name to validate")]
    pub name: String,
}

/// Input for `keyrx_simulate` — simulate key events through the remapping engine.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SimulateInput {
    #[schemars(description = "Built-in scenario name (e.g. 'capslock_remap')")]
    pub scenario: Option<String>,
}

/// Input for `keyrx_get_status` — no parameters needed.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetStatusInput {}

/// Input for `keyrx_get_state` — no parameters needed.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetStateInput {}

/// Input for `keyrx_list_devices` — no parameters needed.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListDevicesInput {}

/// Input for `keyrx_get_diagnostics` — no parameters needed.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetDiagnosticsInput {}

/// Input for `keyrx_get_latency` — no parameters needed.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetLatencyInput {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_profiles_input_deserializes() {
        let json = serde_json::json!({});
        let _input: ListProfilesInput = serde_json::from_value(json).unwrap();
    }

    #[test]
    fn test_get_profile_config_input_deserializes() {
        let json = serde_json::json!({"name": "default"});
        let input: GetProfileConfigInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.name, "default");
    }

    #[test]
    fn test_set_profile_config_input_deserializes() {
        let json = serde_json::json!({"name": "test", "source": "remap(\"A\", \"B\");"});
        let input: SetProfileConfigInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.name, "test");
        assert!(input.source.contains("remap"));
    }

    #[test]
    fn test_create_profile_input_with_optional_template() {
        let json = serde_json::json!({"name": "gaming"});
        let input: CreateProfileInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.name, "gaming");
        assert!(input.template.is_none());

        let json2 = serde_json::json!({"name": "gaming", "template": "vim_navigation"});
        let input2: CreateProfileInput = serde_json::from_value(json2).unwrap();
        assert_eq!(input2.template.as_deref(), Some("vim_navigation"));
    }

    #[test]
    fn test_simulate_input_optional_scenario() {
        let json = serde_json::json!({});
        let input: SimulateInput = serde_json::from_value(json).unwrap();
        assert!(input.scenario.is_none());
    }
}
