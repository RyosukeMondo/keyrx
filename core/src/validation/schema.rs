use crate::config::Config;
use crate::discovery::DeviceProfile;
use schemars::{schema::RootSchema, schema_for};
use std::collections::HashMap;
use std::sync::OnceLock;

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
}
