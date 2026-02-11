//! Daemon factory for creating configured daemon instances.
//!
//! This module provides centralized daemon initialization with dependency injection,
//! following the Factory pattern and Dependency Inversion Principle.

use crate::container::ServiceContainer;
use crate::daemon::Daemon;
use crate::platform::Platform;
use std::path::Path;
use std::sync::Arc;

/// Factory for creating daemon instances with injected dependencies.
pub struct DaemonFactory {
    service_container: Option<Arc<ServiceContainer>>,
}

impl DaemonFactory {
    /// Create a new daemon factory.
    pub fn new() -> Self {
        Self {
            service_container: None,
        }
    }

    /// Set the service container for dependency injection.
    ///
    /// # Arguments
    ///
    /// * `container` - Service container with all dependencies
    pub fn with_services(mut self, container: Arc<ServiceContainer>) -> Self {
        self.service_container = Some(container);
        self
    }

    /// Build a daemon instance with the provided platform and configuration.
    ///
    /// # Arguments
    ///
    /// * `platform` - Platform implementation (injected)
    /// * `config_path` - Path to configuration file
    ///
    /// # Returns
    ///
    /// Returns a configured `Daemon` instance.
    ///
    /// # Errors
    ///
    /// Returns `DaemonError` if:
    /// - Configuration file cannot be loaded
    /// - Platform initialization fails
    /// - No devices are matched
    pub fn build(
        self,
        platform: Box<dyn Platform>,
        config_path: &Path,
    ) -> Result<Daemon, crate::daemon::DaemonError> {
        Daemon::new(platform, config_path)
    }
}

impl Default for DaemonFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_factory_new() {
        let factory = DaemonFactory::new();
        assert!(factory.service_container.is_none());
    }

    #[test]
    fn test_daemon_factory_default() {
        let factory = DaemonFactory::default();
        assert!(factory.service_container.is_none());
    }
}
