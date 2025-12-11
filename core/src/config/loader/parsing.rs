//! TOML parsing, file I/O, and schema validation for configuration.
//!
//! This module handles the low-level operations of reading configuration
//! files from disk, parsing TOML content, validating against JSON schemas,
//! and handling migrations.

use super::validation::validate_and_clamp;
use super::Config;
use crate::config::migration::migrate_config_file;
use crate::config::paths::config_dir;
use crate::validation::{SchemaError, SchemaRegistry, ValidationFailure, CONFIG_SCHEMA_NAME};
use std::fs;
use std::path::Path;

// =============================================================================
// Loading Functions
// =============================================================================

/// Load configuration from a TOML file.
///
/// If `path` is `Some`, loads from the specified path.
/// If `path` is `None`, attempts to load from the default path
/// (`~/.config/keyrx/config.toml` or `$XDG_CONFIG_HOME/keyrx/config.toml`).
///
/// If the file doesn't exist or fails to parse, returns default configuration
/// and logs a warning.
///
/// # Arguments
///
/// * `path` - Optional path to config file. If None, uses default location.
///
/// # Returns
///
/// The loaded configuration, falling back to defaults on error.
///
/// # Example
///
/// ```no_run
/// use keyrx_core::config::load_config;
///
/// // Load from default location
/// let config = load_config(None);
///
/// // Load from specific path
/// let config = load_config(Some(std::path::Path::new("/etc/keyrx/config.toml")));
/// ```
pub fn load_config(path: Option<&Path>) -> Config {
    let config_path = path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| config_dir().join("config.toml"));

    match fs::read_to_string(&config_path) {
        Ok(content) => load_from_content(&config_path, &content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!(
                service = "keyrx",
                event = "config_not_found",
                component = "config",
                path = %config_path.display(),
                "Config file not found, using defaults"
            );
            Config::default()
        }
        Err(e) => {
            tracing::warn!(
                service = "keyrx",
                event = "config_read_error",
                component = "config",
                path = %config_path.display(),
                error = %e,
                "Failed to read config file, using defaults"
            );
            Config::default()
        }
    }
}

/// Load configuration from TOML content string.
fn load_from_content(config_path: &Path, content: &str) -> Config {
    let mut toml_value = match toml::from_str::<toml::Value>(content) {
        Ok(value) => value,
        Err(e) => {
            tracing::warn!(
                service = "keyrx",
                event = "config_parse_error",
                component = "config",
                path = %config_path.display(),
                error = %e,
                "Failed to parse config file, using defaults"
            );
            return Config::default();
        }
    };

    let migration_result = match migrate_config_file(config_path, &mut toml_value) {
        Ok(result) => result,
        Err(e) => {
            tracing::error!(
                service = "keyrx",
                event = "config_migration_failed",
                component = "config",
                path = %config_path.display(),
                error = %e,
                "Failed to migrate config, using defaults"
            );
            return Config::default();
        }
    };

    if let Some(outcome) = migration_result {
        tracing::info!(
            service = "keyrx",
            event = "config_migrated",
            component = "config",
            path = %config_path.display(),
            from_version = outcome.from,
            to_version = outcome.to,
            backup = outcome
                .backup_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            "Config migrated to current version"
        );
    }

    if let Err(err) = validate_config_schema(config_path, &toml_value) {
        log_config_validation_error(config_path, &err);
        if should_abort_load(&err) {
            return Config::default();
        }
    }

    match toml_value.try_into::<Config>() {
        Ok(mut config) => {
            validate_and_clamp(&mut config);
            tracing::debug!(
                service = "keyrx",
                event = "config_loaded",
                component = "config",
                path = %config_path.display(),
                "Configuration loaded from file"
            );
            config
        }
        Err(e) => {
            tracing::warn!(
                service = "keyrx",
                event = "config_parse_error",
                component = "config",
                path = %config_path.display(),
                error = %e,
                "Failed to parse config file, using defaults"
            );
            Config::default()
        }
    }
}

// =============================================================================
// Schema Validation
// =============================================================================

/// Validate configuration against JSON schema.
fn validate_config_schema(
    config_path: &Path,
    toml_value: &toml::Value,
) -> Result<(), ValidationFailure> {
    let instance = serde_json::to_value(toml_value).map_err(|source| {
        ValidationFailure::Schema(SchemaError::Serialize {
            name: CONFIG_SCHEMA_NAME.to_string(),
            source,
        })
    })?;

    tracing::debug!(
        service = "keyrx",
        event = "config_schema_validation_started",
        component = "config",
        path = %config_path.display(),
        "Validating config against schema"
    );

    SchemaRegistry::global().validate(CONFIG_SCHEMA_NAME, &instance)
}

/// Log validation errors with appropriate severity.
fn log_config_validation_error(path: &Path, error: &ValidationFailure) {
    match error {
        ValidationFailure::Invalid { issues, .. } => {
            tracing::error!(
                service = "keyrx",
                event = "config_schema_invalid",
                component = "config",
                path = %path.display(),
                issue_count = issues.len(),
                "Config schema validation failed"
            );

            for issue in issues {
                tracing::error!(
                    service = "keyrx",
                    event = "config_schema_issue",
                    component = "config",
                    path = %path.display(),
                    instance_path = %issue.instance_path,
                    schema_path = %issue.schema_path,
                    message = %issue.message,
                    "Schema validation error"
                );
            }
        }
        ValidationFailure::Schema(SchemaError::Missing { .. }) => {
            tracing::warn!(
                service = "keyrx",
                event = "config_schema_missing",
                component = "config",
                path = %path.display(),
                "Config schema not found, skipping schema validation"
            );
        }
        ValidationFailure::Schema(SchemaError::Serialize { source, .. }) => {
            tracing::warn!(
                service = "keyrx",
                event = "config_schema_serialize_error",
                component = "config",
                path = %path.display(),
                error = %source,
                "Failed to serialize config schema for validation, continuing without schema check"
            );
        }
        ValidationFailure::Schema(SchemaError::Compile { message, .. }) => {
            tracing::error!(
                service = "keyrx",
                event = "config_schema_compile_error",
                component = "config",
                path = %path.display(),
                error = %message,
                "Failed to compile config schema, continuing without schema check"
            );
        }
    }
}

/// Determine if a validation error should abort config loading.
fn should_abort_load(error: &ValidationFailure) -> bool {
    matches!(error, ValidationFailure::Invalid { .. })
}
