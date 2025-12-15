//! Minimal FFI Function Example
//!
//! This example demonstrates the simplest possible FFI function using
//! the AI-native workflow with contract derivation.

use anyhow::Result;

/// A simple configuration value.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigValue {
    pub key: String,
    pub value: String,
}

/// Get a configuration value by key.
///
/// This is a minimal FFI function that demonstrates:
/// - The #[ffi_export] macro (commented out for standalone example)
/// - Result<T, E> return type for error handling
/// - Simple string parameters
/// - Documentation that becomes part of the contract
///
/// # Example
///
/// ```rust
/// use minimal_ffi_function::get_config_value;
///
/// let result = get_config_value("app.name");
/// assert!(result.is_ok());
/// ```
// In the real codebase, this would be:
// #[ffi_export(domain = "config", version = "1.0")]
pub fn get_config_value(key: &str) -> Result<ConfigValue> {
    // In a real implementation, this would:
    // 1. Validate the key
    // 2. Look up the value in a config store
    // 3. Return the value or an error

    if key.is_empty() {
        anyhow::bail!("Key cannot be empty");
    }

    Ok(ConfigValue {
        key: key.to_string(),
        value: format!("value_for_{}", key),
    })
}

/// Set a configuration value.
///
/// Demonstrates a function that modifies state and returns void (unit type).
// #[ffi_export(domain = "config", version = "1.0")]
pub fn set_config_value(key: &str, value: &str) -> Result<()> {
    if key.is_empty() {
        anyhow::bail!("Key cannot be empty");
    }

    if value.is_empty() {
        anyhow::bail!("Value cannot be empty");
    }

    // In a real implementation, persist the value
    println!("Setting {} = {}", key, value);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_value_success() {
        let result = get_config_value("test.key");
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.key, "test.key");
        assert_eq!(config.value, "value_for_test.key");
    }

    #[test]
    fn test_get_config_value_empty_key() {
        let result = get_config_value("");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_config_value_success() {
        let result = set_config_value("test.key", "test.value");
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_config_value_empty_key() {
        let result = set_config_value("", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_config_value_empty_value() {
        let result = set_config_value("key", "");
        assert!(result.is_err());
    }
}
