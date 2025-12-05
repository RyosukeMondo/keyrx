use crate::config::Config;
use crate::discovery::DeviceProfile;
use jsonschema::{Draft, JSONSchema};
use schemars::{schema::RootSchema, schema_for};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::OnceLock;
use thiserror::Error;

/// Registry of JSON Schemas generated from KeyRx data models.
///
/// Schemas are generated once at startup and stored in-memory for fast lookup.
pub struct SchemaRegistry {
    schemas: HashMap<&'static str, RootSchema>,
}

/// Schema identifier for the root config file.
pub const CONFIG_SCHEMA_NAME: &str = "config";

/// Schema identifier for discovered device profiles.
pub const DEVICE_PROFILE_SCHEMA_NAME: &str = "device_profile";

/// Errors when accessing or compiling schemas.
#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("Schema '{name}' not found")]
    Missing { name: String },
    #[error("Failed to serialize schema '{name}': {source}")]
    Serialize {
        name: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("Failed to compile schema '{name}': {message}")]
    Compile { name: String, message: String },
}

/// Detailed information about a schema validation issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    /// JSON Pointer path to the offending instance location.
    pub instance_path: String,
    /// JSON Pointer path to the schema rule that failed.
    pub schema_path: String,
    /// Human-readable description of the issue.
    pub message: String,
}

impl From<jsonschema::ValidationError<'_>> for ValidationIssue {
    fn from(error: jsonschema::ValidationError<'_>) -> Self {
        ValidationIssue {
            instance_path: error.instance_path.to_string(),
            schema_path: error.schema_path.to_string(),
            message: error.to_string(),
        }
    }
}

/// Errors produced when validating JSON against a schema.
#[derive(Debug, Error)]
pub enum ValidationFailure {
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error("Schema '{name}' validation failed")]
    Invalid {
        name: String,
        issues: Vec<ValidationIssue>,
    },
}

impl SchemaRegistry {
    /// Global singleton containing all embedded schemas.
    pub fn global() -> &'static Self {
        static REGISTRY: OnceLock<SchemaRegistry> = OnceLock::new();
        REGISTRY.get_or_init(Self::build)
    }

    /// Return a schema by name if it exists.
    pub fn get(&self, name: &str) -> Option<&RootSchema> {
        self.schemas.get(name)
    }

    /// List all registered schema names.
    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.schemas.keys().copied()
    }

    /// Validate a JSON instance against a named schema.
    pub fn validate(&self, name: &str, instance: &JsonValue) -> Result<(), ValidationFailure> {
        let schema = self.schemas.get(name).ok_or_else(|| SchemaError::Missing {
            name: name.to_string(),
        })?;

        let schema_json =
            serde_json::to_value(schema).map_err(|source| SchemaError::Serialize {
                name: name.to_string(),
                source,
            })?;

        let compiled = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema_json)
            .map_err(|err| SchemaError::Compile {
                name: name.to_string(),
                message: err.to_string(),
            })?;

        if let Err(errors) = compiled.validate(instance) {
            let issues = errors.map(ValidationIssue::from).collect();
            return Err(ValidationFailure::Invalid {
                name: name.to_string(),
                issues,
            });
        }

        Ok(())
    }

    fn build() -> SchemaRegistry {
        let mut schemas = HashMap::new();
        schemas.insert(CONFIG_SCHEMA_NAME, schema_for!(Config));
        schemas.insert(DEVICE_PROFILE_SCHEMA_NAME, schema_for!(DeviceProfile));

        SchemaRegistry { schemas }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_contains_expected_schemas() {
        let registry = SchemaRegistry::global();

        assert!(registry.get(CONFIG_SCHEMA_NAME).is_some());
        assert!(registry.get(DEVICE_PROFILE_SCHEMA_NAME).is_some());
    }

    #[test]
    fn names_iterates_over_all_entries() {
        let registry = SchemaRegistry::global();
        let names: Vec<_> = registry.names().collect();

        assert!(names.contains(&CONFIG_SCHEMA_NAME));
        assert!(names.contains(&DEVICE_PROFILE_SCHEMA_NAME));
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn validate_accepts_valid_config() {
        let registry = SchemaRegistry::global();
        let json = serde_json::to_value(Config::default()).expect("serialize config");

        registry
            .validate(CONFIG_SCHEMA_NAME, &json)
            .expect("validation should pass");
    }

    #[test]
    fn validate_rejects_invalid_config() {
        let registry = SchemaRegistry::global();
        let json = serde_json::json!({
            "timing": { "tap_timeout_ms": "fast" },
            "ui": {},
            "performance": {},
            "paths": {},
            "scripting": {}
        });

        let err = registry
            .validate(CONFIG_SCHEMA_NAME, &json)
            .expect_err("validation should fail");

        match err {
            ValidationFailure::Invalid { issues, .. } => {
                assert!(!issues.is_empty());
            }
            _ => panic!("expected validation failure"),
        }
    }
}
