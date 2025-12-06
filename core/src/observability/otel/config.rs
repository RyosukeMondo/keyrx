//! Environment-driven OpenTelemetry configuration.
//!
//! This module centralizes OTEL configuration so tracing and metrics exporters
//! can be toggled at runtime without code changes.

use std::env;
use std::time::Duration;

/// Configuration for OTEL exporters loaded from environment variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtelConfig {
    /// Whether OTEL export is enabled.
    pub enabled: bool,
    /// OTLP endpoint, e.g. `http://localhost:4317`.
    pub endpoint: String,
    /// Logical service name for spans/metrics.
    pub service_name: String,
    /// Batch size for exporters.
    pub batch_size: usize,
    /// Export interval for batch processors.
    pub export_interval: Duration,
}

/// Errors encountered while parsing OTEL configuration.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum OtelConfigError {
    #[error("Environment variable {key} contained invalid UTF-8: {source}")]
    InvalidUnicode {
        key: String,
        #[source]
        source: env::VarError,
    },

    #[error("Environment variable {key} expected boolean (true/false/1/0/on/off/yes/no) but was '{value}'")]
    InvalidBoolean { key: String, value: String },

    #[error("Environment variable {key} expected integer but was '{value}'")]
    InvalidInteger { key: String, value: String },

    #[error("OTEL endpoint must start with http:// or https:// (got '{0}')")]
    InvalidEndpoint(String),

    #[error("Batch size must be greater than zero (got {0})")]
    InvalidBatchSize(usize),

    #[error("Export interval must be greater than zero seconds (got {0:?})")]
    InvalidExportInterval(Duration),
}

impl OtelConfig {
    pub const DEFAULT_ENDPOINT: &str = "http://localhost:4317";
    pub const DEFAULT_SERVICE_NAME: &str = "keyrx";
    pub const DEFAULT_BATCH_SIZE: usize = 512;
    pub const DEFAULT_EXPORT_INTERVAL_SECS: u64 = 5;

    /// Build configuration from environment variables with validation.
    ///
    /// Supported variables:
    /// - `OTEL_ENABLED`: boolean toggle (false if unset)
    /// - `OTEL_EXPORTER_OTLP_ENDPOINT`: OTLP endpoint (http/https)
    /// - `OTEL_SERVICE_NAME`: logical service name
    /// - `OTEL_EXPORTER_OTLP_BATCH_SIZE`: positive integer batch size
    /// - `OTEL_EXPORTER_OTLP_EXPORT_INTERVAL`: seconds between batch exports
    pub fn from_env() -> Result<Self, OtelConfigError> {
        let enabled = parse_enabled("OTEL_ENABLED")?;
        let endpoint =
            parse_string_or_default("OTEL_EXPORTER_OTLP_ENDPOINT", Self::DEFAULT_ENDPOINT)?;
        let service_name =
            parse_string_or_default("OTEL_SERVICE_NAME", Self::DEFAULT_SERVICE_NAME)?;
        let batch_size =
            parse_usize_or_default("OTEL_EXPORTER_OTLP_BATCH_SIZE", Self::DEFAULT_BATCH_SIZE)?;
        let export_interval = parse_duration_or_default(
            "OTEL_EXPORTER_OTLP_EXPORT_INTERVAL",
            Duration::from_secs(Self::DEFAULT_EXPORT_INTERVAL_SECS),
        )?;

        let config = Self {
            enabled,
            endpoint,
            service_name,
            batch_size,
            export_interval,
        };

        config.validate()?;
        Ok(config)
    }

    /// Validate that required fields are present and sensible.
    pub fn validate(&self) -> Result<(), OtelConfigError> {
        if self.enabled {
            validate_endpoint(&self.endpoint)?;
        }

        if self.batch_size == 0 {
            return Err(OtelConfigError::InvalidBatchSize(self.batch_size));
        }

        if self.export_interval.is_zero() {
            return Err(OtelConfigError::InvalidExportInterval(self.export_interval));
        }

        Ok(())
    }
}

fn parse_string_or_default(key: &str, default: &str) -> Result<String, OtelConfigError> {
    read_var(key).map(|value| value.unwrap_or_else(|| default.to_string()))
}

fn parse_enabled(key: &str) -> Result<bool, OtelConfigError> {
    let Some(value) = read_var(key)? else {
        return Ok(false);
    };

    match value.to_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(OtelConfigError::InvalidBoolean {
            key: key.to_string(),
            value,
        }),
    }
}

fn parse_usize_or_default(key: &str, default: usize) -> Result<usize, OtelConfigError> {
    let Some(value) = read_var(key)? else {
        return Ok(default);
    };

    value
        .parse::<usize>()
        .map_err(|_| OtelConfigError::InvalidInteger {
            key: key.to_string(),
            value,
        })
}

fn parse_duration_or_default(key: &str, default: Duration) -> Result<Duration, OtelConfigError> {
    let Some(value) = read_var(key)? else {
        return Ok(default);
    };

    let secs = value
        .parse::<u64>()
        .map_err(|_| OtelConfigError::InvalidInteger {
            key: key.to_string(),
            value,
        })?;

    Ok(Duration::from_secs(secs))
}

fn validate_endpoint(endpoint: &str) -> Result<(), OtelConfigError> {
    if endpoint.trim().is_empty() {
        return Err(OtelConfigError::InvalidEndpoint(endpoint.to_string()));
    }

    let has_scheme = endpoint.starts_with("http://") || endpoint.starts_with("https://");
    if !has_scheme {
        return Err(OtelConfigError::InvalidEndpoint(endpoint.to_string()));
    }

    Ok(())
}

fn read_var(key: &str) -> Result<Option<String>, OtelConfigError> {
    match env::var(key) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(OtelConfigError::InvalidUnicode {
            key: key.to_string(),
            source: err,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn defaults_when_env_missing() {
        with_env(&[], || {
            let cfg = OtelConfig::from_env().expect("config should build");

            assert!(!cfg.enabled);
            assert_eq!(cfg.endpoint, OtelConfig::DEFAULT_ENDPOINT);
            assert_eq!(cfg.service_name, OtelConfig::DEFAULT_SERVICE_NAME);
            assert_eq!(cfg.batch_size, OtelConfig::DEFAULT_BATCH_SIZE);
            assert_eq!(
                cfg.export_interval,
                Duration::from_secs(OtelConfig::DEFAULT_EXPORT_INTERVAL_SECS)
            );
        });
    }

    #[test]
    #[serial]
    fn parses_overrides_from_env() {
        with_env(
            &[
                ("OTEL_ENABLED", Some("true")),
                (
                    "OTEL_EXPORTER_OTLP_ENDPOINT",
                    Some("https://collector:4318"),
                ),
                ("OTEL_SERVICE_NAME", Some("keyrx-core")),
                ("OTEL_EXPORTER_OTLP_BATCH_SIZE", Some("1024")),
                ("OTEL_EXPORTER_OTLP_EXPORT_INTERVAL", Some("7")),
            ],
            || {
                let cfg = OtelConfig::from_env().expect("config should build");

                assert!(cfg.enabled);
                assert_eq!(cfg.endpoint, "https://collector:4318");
                assert_eq!(cfg.service_name, "keyrx-core");
                assert_eq!(cfg.batch_size, 1024);
                assert_eq!(cfg.export_interval, Duration::from_secs(7));
            },
        );
    }

    #[test]
    #[serial]
    fn invalid_boolean_rejected() {
        with_env(&[("OTEL_ENABLED", Some("maybe"))], || {
            let err = OtelConfig::from_env().expect_err("should fail bool parsing");
            assert!(matches!(err, OtelConfigError::InvalidBoolean { .. }));
        });
    }

    #[test]
    #[serial]
    fn invalid_endpoint_rejected_when_enabled() {
        with_env(
            &[
                ("OTEL_ENABLED", Some("true")),
                ("OTEL_EXPORTER_OTLP_ENDPOINT", Some("localhost:4317")),
            ],
            || {
                let err = OtelConfig::from_env().expect_err("should reject missing scheme");
                assert!(matches!(err, OtelConfigError::InvalidEndpoint(_)));
            },
        );
    }

    #[test]
    #[serial]
    fn invalid_numbers_are_rejected() {
        with_env(
            &[
                ("OTEL_ENABLED", Some("true")),
                ("OTEL_EXPORTER_OTLP_BATCH_SIZE", Some("0")),
            ],
            || {
                let err = OtelConfig::from_env().expect_err("batch size zero not allowed");
                assert!(matches!(err, OtelConfigError::InvalidBatchSize(0)));
            },
        );

        with_env(
            &[
                ("OTEL_ENABLED", Some("true")),
                ("OTEL_EXPORTER_OTLP_EXPORT_INTERVAL", Some("0")),
            ],
            || {
                let err = OtelConfig::from_env().expect_err("export interval zero not allowed");
                assert!(matches!(
                    err,
                    OtelConfigError::InvalidExportInterval(duration) if duration.is_zero()
                ));
            },
        );
    }

    #[test]
    #[serial]
    fn invalid_integer_format_is_rejected() {
        with_env(
            &[
                ("OTEL_ENABLED", Some("true")),
                ("OTEL_EXPORTER_OTLP_BATCH_SIZE", Some("abc")),
            ],
            || {
                let err = OtelConfig::from_env().expect_err("should reject non-numeric");
                assert!(matches!(err, OtelConfigError::InvalidInteger { .. }));
            },
        );
    }

    fn with_env(vars: &[(&str, Option<&str>)], f: impl FnOnce()) {
        // Snapshot current values to restore after test.
        let originals: Vec<(String, Option<String>)> = vars
            .iter()
            .map(|(key, _)| (key.to_string(), env::var(key).ok()))
            .collect();

        // Apply test values.
        for (key, value) in vars {
            match value {
                Some(val) => env::set_var(key, val),
                None => env::remove_var(key),
            }
        }

        f();

        // Restore original environment.
        for (key, value) in originals {
            match value {
                Some(val) => env::set_var(&key, val),
                None => env::remove_var(&key),
            }
        }
    }
}
