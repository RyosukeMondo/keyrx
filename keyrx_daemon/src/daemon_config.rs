//! Central configuration for keyrx daemon
//!
//! This module provides a single source of truth for all configuration values
//! including ports, addresses, log levels, and other daemon settings.
//!
//! Configuration hierarchy (highest to lowest priority):
//! 1. Environment variables (KEYRX_*)
//! 2. settings.json (persisted user settings)
//! 3. Defaults defined in this file

use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

/// Default server bind address (localhost only)
pub const DEFAULT_BIND_HOST: &str = "127.0.0.1";

/// Default server port
pub const DEFAULT_PORT: u16 = 9867;

/// Default log level
pub const DEFAULT_LOG_LEVEL: &str = "info";

/// Minimum allowed port number
pub const MIN_PORT: u16 = 1024;

/// Maximum allowed port number
pub const MAX_PORT: u16 = 65535;

/// CORS allowed origins for development
pub const CORS_DEV_ORIGINS: &[&str] = &[
    "http://localhost:3000",
    "http://localhost:5173",
    "http://localhost:8080",
    "http://127.0.0.1:3000",
    "http://127.0.0.1:5173",
    "http://127.0.0.1:8080",
];

/// CORS allowed origins for production (empty - must be explicitly set)
pub const CORS_PROD_ORIGINS: &[&str] = &[];

/// Global daemon configuration
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// Server bind address (IP)
    pub bind_host: String,

    /// Server port
    pub port: u16,

    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,

    /// Enable debug mode (more verbose logging)
    pub debug: bool,

    /// Enable test mode (no keyboard capture, IPC only)
    pub test_mode: bool,

    /// Is production environment
    pub is_production: bool,

    /// CORS allowed origins (comma-separated or loaded from environment)
    pub cors_origins: Vec<String>,
}

impl DaemonConfig {
    /// Create a new configuration from environment variables and defaults
    ///
    /// Environment variables (with KEYRX_ prefix):
    /// - KEYRX_BIND_HOST: Server bind address (default: 127.0.0.1)
    /// - KEYRX_PORT: Server port (default: 9867)
    /// - KEYRX_LOG_LEVEL: Log level (default: info)
    /// - KEYRX_DEBUG: Enable debug mode (default: false)
    /// - KEYRX_TEST_MODE: Enable test mode (default: false)
    /// - KEYRX_ALLOWED_ORIGINS: Comma-separated CORS origins (required in production, defaults to dev origins)
    /// - RUST_ENV: Environment (development/production)
    pub fn from_env() -> Result<Self, String> {
        let bind_host =
            std::env::var("KEYRX_BIND_HOST").unwrap_or_else(|_| DEFAULT_BIND_HOST.to_string());

        let port_str = std::env::var("KEYRX_PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string());
        let port: u16 = port_str
            .parse()
            .map_err(|_| format!("Invalid port number: {}", port_str))?;

        if !(MIN_PORT..=MAX_PORT).contains(&port) {
            return Err(format!(
                "Port out of range: {} (must be between {} and {})",
                port, MIN_PORT, MAX_PORT
            ));
        }

        let log_level =
            std::env::var("KEYRX_LOG_LEVEL").unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string());

        let debug = std::env::var("KEYRX_DEBUG")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        let test_mode = std::env::var("KEYRX_TEST_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        let is_production = std::env::var("RUST_ENV")
            .map(|v| v.to_lowercase() == "production")
            .unwrap_or(false)
            || std::env::var("ENVIRONMENT")
                .map(|v| v.to_lowercase() == "production")
                .unwrap_or(false);

        // Parse CORS origins from environment or use defaults
        let cors_origins = Self::parse_cors_origins(is_production)?;

        Ok(Self {
            bind_host,
            port,
            log_level,
            debug,
            test_mode,
            is_production,
            cors_origins,
        })
    }

    /// Parse CORS origins from environment variable or use defaults
    ///
    /// In production, KEYRX_ALLOWED_ORIGINS must be explicitly set (non-empty).
    /// In development, defaults to development origins.
    fn parse_cors_origins(is_production: bool) -> Result<Vec<String>, String> {
        match std::env::var("KEYRX_ALLOWED_ORIGINS") {
            Ok(origins_str) => {
                if origins_str.trim().is_empty() {
                    if is_production {
                        return Err(
              "KEYRX_ALLOWED_ORIGINS is empty in production - must explicitly set CORS origins"
                .to_string(),
            );
                    }
                    // Use default dev origins if empty in development
                    Ok(CORS_DEV_ORIGINS.iter().map(|s| s.to_string()).collect())
                } else {
                    // Parse comma-separated origins and trim whitespace
                    Ok(origins_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect())
                }
            }
            Err(_) => {
                if is_production {
                    return Err(
                        "KEYRX_ALLOWED_ORIGINS environment variable not set in production. \
             Set it to a comma-separated list of allowed origins."
                            .to_string(),
                    );
                }
                // Use default dev origins
                Ok(CORS_DEV_ORIGINS.iter().map(|s| s.to_string()).collect())
            }
        }
    }

    /// Get the socket address for binding
    ///
    /// # Errors
    ///
    /// Returns an error if the bind address is invalid
    pub fn socket_addr(&self) -> Result<SocketAddr, String> {
        let ip = IpAddr::from_str(&self.bind_host)
            .map_err(|e| format!("Invalid bind address '{}': {}", self.bind_host, e))?;
        Ok(SocketAddr::new(ip, self.port))
    }

    /// Get the web server URL for logging/display
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = DaemonConfig::from_env()?;
    /// println!("Server running at: {}", config.web_url());
    /// // Output: "http://127.0.0.1:9867"
    /// ```
    pub fn web_url(&self) -> String {
        format!("http://{}:{}", self.bind_host, self.port)
    }

    /// Get the effective log level (debug overrides)
    pub fn effective_log_level(&self) -> String {
        if self.debug {
            "debug".to_string()
        } else {
            self.log_level.clone()
        }
    }

    /// Get CORS allowed origins
    pub fn cors_origins(&self) -> &[String] {
        &self.cors_origins
    }

    /// Validate configuration on startup
    ///
    /// # Errors
    ///
    /// Returns an error if configuration is invalid
    pub fn validate(&self) -> Result<(), String> {
        // Validate bind address
        IpAddr::from_str(&self.bind_host).map_err(|e| format!("Invalid bind address: {}", e))?;

        // Validate port range
        if !(MIN_PORT..=MAX_PORT).contains(&self.port) {
            return Err(format!(
                "Port out of range: {} (must be between {} and {})",
                self.port, MIN_PORT, MAX_PORT
            ));
        }

        // Validate log level
        match self.log_level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => {
                return Err(format!(
                    "Invalid log level: {} (must be: trace, debug, info, warn, error)",
                    self.log_level
                ))
            }
        }

        Ok(())
    }
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            bind_host: DEFAULT_BIND_HOST.to_string(),
            port: DEFAULT_PORT,
            log_level: DEFAULT_LOG_LEVEL.to_string(),
            debug: false,
            test_mode: false,
            is_production: false,
            cors_origins: CORS_DEV_ORIGINS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_default_config() {
        let config = DaemonConfig::default();
        assert_eq!(config.bind_host, DEFAULT_BIND_HOST);
        assert_eq!(config.port, DEFAULT_PORT);
        assert_eq!(config.log_level, DEFAULT_LOG_LEVEL);
        assert!(!config.debug);
        assert!(!config.test_mode);
    }

    #[test]
    fn test_socket_addr() {
        let config = DaemonConfig::default();
        let addr = config.socket_addr().unwrap();
        assert_eq!(addr.port(), DEFAULT_PORT);
    }

    #[test]
    fn test_web_url() {
        let config = DaemonConfig::default();
        assert_eq!(config.web_url(), "http://127.0.0.1:9867");
    }

    #[test]
    fn test_effective_log_level() {
        let mut config = DaemonConfig::default();
        config.log_level = "info".to_string();
        assert_eq!(config.effective_log_level(), "info");

        config.debug = true;
        assert_eq!(config.effective_log_level(), "debug");
    }

    #[test]
    fn test_validate_invalid_port() {
        let mut config = DaemonConfig::default();
        config.port = 100; // Below MIN_PORT
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_log_level() {
        let mut config = DaemonConfig::default();
        config.log_level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_valid_config() {
        let config = DaemonConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    #[serial]
    fn test_parse_cors_origins_dev() {
        // In dev mode, should return default origins even if env var not set
        std::env::remove_var("KEYRX_ALLOWED_ORIGINS");
        let origins = DaemonConfig::parse_cors_origins(false).unwrap();
        assert!(!origins.is_empty());
        assert!(origins.contains(&"http://localhost:3000".to_string()));
    }

    #[test]
    #[serial]
    fn test_parse_cors_origins_production_missing() {
        // In production, must be explicitly set
        std::env::remove_var("KEYRX_ALLOWED_ORIGINS");
        let result = DaemonConfig::parse_cors_origins(true);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("KEYRX_ALLOWED_ORIGINS environment variable not set in production"));
    }

    #[test]
    #[serial]
    fn test_parse_cors_origins_from_env() {
        std::env::set_var(
            "KEYRX_ALLOWED_ORIGINS",
            "https://example.com, https://app.example.com",
        );
        let origins = DaemonConfig::parse_cors_origins(true).unwrap();
        assert_eq!(origins.len(), 2);
        assert!(origins.contains(&"https://example.com".to_string()));
        assert!(origins.contains(&"https://app.example.com".to_string()));
        std::env::remove_var("KEYRX_ALLOWED_ORIGINS");
    }

    #[test]
    #[serial]
    fn test_parse_cors_origins_empty_in_production() {
        std::env::set_var("KEYRX_ALLOWED_ORIGINS", "");
        let result = DaemonConfig::parse_cors_origins(true);
        assert!(result.is_err());
        std::env::remove_var("KEYRX_ALLOWED_ORIGINS");
    }

    #[test]
    #[serial]
    fn test_parse_cors_origins_empty_in_dev() {
        std::env::set_var("KEYRX_ALLOWED_ORIGINS", "");
        let origins = DaemonConfig::parse_cors_origins(false).unwrap();
        // Should fall back to defaults in dev
        assert!(!origins.is_empty());
        std::env::remove_var("KEYRX_ALLOWED_ORIGINS");
    }
}
