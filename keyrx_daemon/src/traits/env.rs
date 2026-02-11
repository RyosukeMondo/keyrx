//! Environment variable abstraction for dependency injection.
//!
//! This module provides traits and implementations for accessing environment
//! variables in a testable way. The `EnvProvider` trait abstracts `std::env::var`
//! allowing tests to use mock implementations without modifying global state.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Trait for accessing environment variables.
///
/// This trait abstracts environment variable access to enable testing
/// without modifying the actual process environment.
///
/// # Examples
///
/// ```
/// use keyrx_daemon::traits::env::{EnvProvider, RealEnvProvider};
///
/// let provider = RealEnvProvider::new();
/// if let Ok(home) = provider.var("HOME") {
///     println!("Home directory: {}", home);
/// }
/// ```
pub trait EnvProvider: Send + Sync {
    /// Get an environment variable.
    ///
    /// # Arguments
    ///
    /// * `key` - The environment variable name
    ///
    /// # Returns
    ///
    /// The value if the variable exists, or an error if not found.
    ///
    /// # Errors
    ///
    /// Returns `std::env::VarError::NotPresent` if the variable doesn't exist.
    fn var(&self, key: &str) -> Result<String, std::env::VarError>;
}

/// Production implementation that delegates to `std::env::var`.
///
/// This is the default implementation used in production code.
///
/// # Examples
///
/// ```
/// use keyrx_daemon::traits::env::{EnvProvider, RealEnvProvider};
///
/// let provider = RealEnvProvider::new();
/// let result = provider.var("PATH");
/// assert!(result.is_ok());
/// ```
#[derive(Debug, Clone, Default)]
pub struct RealEnvProvider;

impl RealEnvProvider {
    /// Creates a new real environment provider.
    pub fn new() -> Self {
        Self
    }
}

impl EnvProvider for RealEnvProvider {
    fn var(&self, key: &str) -> Result<String, std::env::VarError> {
        std::env::var(key)
    }
}

/// Mock implementation for testing.
///
/// Allows tests to control environment variables without modifying
/// the actual process environment.
///
/// # Thread Safety
///
/// MockEnvProvider is `Send + Sync` via internal `Arc<RwLock>`.
///
/// # Examples
///
/// ```
/// use keyrx_daemon::traits::env::{EnvProvider, MockEnvProvider};
///
/// let mut provider = MockEnvProvider::new();
/// provider.set("TEST_VAR", "test_value");
///
/// assert_eq!(provider.var("TEST_VAR").unwrap(), "test_value");
/// assert!(provider.var("NONEXISTENT").is_err());
/// ```
#[derive(Debug, Clone)]
pub struct MockEnvProvider {
    vars: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for MockEnvProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MockEnvProvider {
    /// Creates a new mock environment provider with no variables.
    pub fn new() -> Self {
        Self {
            vars: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Sets a mock environment variable.
    ///
    /// # Arguments
    ///
    /// * `key` - Variable name
    /// * `value` - Variable value
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_daemon::traits::env::MockEnvProvider;
    ///
    /// let mut provider = MockEnvProvider::new();
    /// provider.set("HOME", "/home/testuser");
    /// ```
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        if let Ok(mut vars) = self.vars.write() {
            vars.insert(key.into(), value.into());
        }
    }

    /// Removes a mock environment variable.
    ///
    /// # Arguments
    ///
    /// * `key` - Variable name to remove
    pub fn remove(&mut self, key: &str) {
        if let Ok(mut vars) = self.vars.write() {
            vars.remove(key);
        }
    }

    /// Clears all mock environment variables.
    pub fn clear(&mut self) {
        if let Ok(mut vars) = self.vars.write() {
            vars.clear();
        }
    }
}

impl EnvProvider for MockEnvProvider {
    fn var(&self, key: &str) -> Result<String, std::env::VarError> {
        self.vars
            .read()
            .ok()
            .and_then(|vars| vars.get(key).cloned())
            .ok_or(std::env::VarError::NotPresent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_real_env_provider() {
        let provider = RealEnvProvider::new();

        // PATH should always exist
        assert!(provider.var("PATH").is_ok());

        // Nonexistent variable should fail
        assert!(provider.var("KEYRX_NONEXISTENT_VAR_12345").is_err());
    }

    #[test]
    fn test_mock_env_provider_set_and_get() {
        let mut provider = MockEnvProvider::new();

        provider.set("TEST_VAR", "test_value");
        assert_eq!(provider.var("TEST_VAR").unwrap(), "test_value");
    }

    #[test]
    fn test_mock_env_provider_not_found() {
        let provider = MockEnvProvider::new();
        assert!(provider.var("NONEXISTENT").is_err());
    }

    #[test]
    fn test_mock_env_provider_remove() {
        let mut provider = MockEnvProvider::new();

        provider.set("TEST_VAR", "value");
        assert!(provider.var("TEST_VAR").is_ok());

        provider.remove("TEST_VAR");
        assert!(provider.var("TEST_VAR").is_err());
    }

    #[test]
    fn test_mock_env_provider_clear() {
        let mut provider = MockEnvProvider::new();

        provider.set("VAR1", "value1");
        provider.set("VAR2", "value2");

        provider.clear();

        assert!(provider.var("VAR1").is_err());
        assert!(provider.var("VAR2").is_err());
    }

    #[test]
    fn test_mock_env_provider_thread_safety() {
        use std::thread;

        let mut provider = MockEnvProvider::new();
        provider.set("SHARED_VAR", "initial");

        let provider_clone = provider.clone();

        let handle = thread::spawn(move || {
            assert_eq!(provider_clone.var("SHARED_VAR").unwrap(), "initial");
        });

        handle.join().unwrap();
    }

    #[test]
    fn test_mock_env_provider_default() {
        let provider = MockEnvProvider::default();
        assert!(provider.var("ANYTHING").is_err());
    }

    #[test]
    fn test_real_env_provider_default() {
        let provider = RealEnvProvider::default();
        // PATH should exist
        assert!(provider.var("PATH").is_ok());
    }
}
