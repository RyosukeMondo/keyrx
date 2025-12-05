use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use thiserror::Error;

pub const CURRENT_CONFIG_VERSION: u32 = 1;
const VERSION_KEY: &str = "version";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationOutcome {
    pub from: u32,
    pub to: u32,
    pub backup_path: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("Config root must be a table for migration")]
    InvalidDocument,
    #[error("Config version must be a non-negative integer, found {value}")]
    InvalidVersionValue { value: i64 },
    #[error(
        "Config version {found} is newer than supported version {current}; please upgrade KeyRx"
    )]
    UnsupportedFutureVersion { found: u32, current: u32 },
    #[error("Failed to back up config before migration: {source}")]
    BackupFailed { source: std::io::Error },
    #[error("Failed to write migrated config: {source}")]
    WriteFailed { source: std::io::Error },
    #[error("No migration path from version {from} to {to}")]
    MissingMigration { from: u32, to: u32 },
}

pub fn migrate_config_file(
    path: &Path,
    config: &mut toml::Value,
) -> Result<Option<MigrationOutcome>, MigrationError> {
    let table = config
        .as_table_mut()
        .ok_or(MigrationError::InvalidDocument)?;

    let detected_version = detect_version(table)?;

    if detected_version > CURRENT_CONFIG_VERSION {
        return Err(MigrationError::UnsupportedFutureVersion {
            found: detected_version,
            current: CURRENT_CONFIG_VERSION,
        });
    }

    if detected_version == CURRENT_CONFIG_VERSION {
        return Ok(None);
    }

    let backup_path = backup_file(path)?;

    run_migrations(detected_version, table)?;

    table.insert(
        VERSION_KEY.to_string(),
        toml::Value::Integer(CURRENT_CONFIG_VERSION as i64),
    );

    write_config(path, config)?;

    Ok(Some(MigrationOutcome {
        from: detected_version,
        to: CURRENT_CONFIG_VERSION,
        backup_path,
    }))
}

fn detect_version(table: &toml::map::Map<String, toml::Value>) -> Result<u32, MigrationError> {
    match table.get(VERSION_KEY) {
        Some(toml::Value::Integer(value)) if *value >= 0 => {
            u32::try_from(*value).map_err(|_| MigrationError::InvalidVersionValue { value: *value })
        }
        Some(toml::Value::Integer(value)) => {
            Err(MigrationError::InvalidVersionValue { value: *value })
        }
        Some(_) => Err(MigrationError::InvalidVersionValue { value: -1 }),
        None => Ok(0),
    }
}

fn run_migrations(
    starting_version: u32,
    table: &mut toml::map::Map<String, toml::Value>,
) -> Result<(), MigrationError> {
    let mut version = starting_version;

    while version < CURRENT_CONFIG_VERSION {
        match version {
            0 => migrate_unversioned_to_v1(table),
            _ => {
                return Err(MigrationError::MissingMigration {
                    from: version,
                    to: version + 1,
                })
            }
        }

        version += 1;
    }

    Ok(())
}

fn migrate_unversioned_to_v1(table: &mut toml::map::Map<String, toml::Value>) {
    if !table.contains_key(VERSION_KEY) {
        table.insert(
            VERSION_KEY.to_string(),
            toml::Value::Integer(CURRENT_CONFIG_VERSION as i64),
        );
    }
}

fn backup_file(path: &Path) -> Result<Option<PathBuf>, MigrationError> {
    if !path.exists() {
        return Ok(None);
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let backup_path = path.with_extension(format!("bak.{timestamp}"));

    fs::copy(path, &backup_path).map_err(|source| MigrationError::BackupFailed { source })?;

    Ok(Some(backup_path))
}

fn write_config(path: &Path, config: &toml::Value) -> Result<(), MigrationError> {
    let content = toml::to_string_pretty(config).map_err(|err| MigrationError::WriteFailed {
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()),
    })?;
    fs::write(path, content).map_err(|source| MigrationError::WriteFailed { source })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use toml::toml;

    #[test]
    fn migrates_unversioned_file_and_creates_backup() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config.toml");
        fs::write(&config_path, "[timing]\ntap_timeout_ms = 150\n").unwrap();

        let mut value: toml::Value = toml! {
            timing = { tap_timeout_ms = 150 }
        }
        .into();

        let outcome = migrate_config_file(&config_path, &mut value)
            .expect("migration should succeed")
            .expect("migration should run");

        assert_eq!(outcome.from, 0);
        assert_eq!(outcome.to, CURRENT_CONFIG_VERSION);
        assert!(outcome.backup_path.is_some());
        assert!(value
            .get(VERSION_KEY)
            .and_then(|v| v.as_integer())
            .is_some());

        let rewritten = fs::read_to_string(config_path).unwrap();
        assert!(rewritten.contains("version"));
    }

    #[test]
    fn skips_when_version_is_current() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config.toml");
        fs::write(&config_path, "version = 1\n").unwrap();

        let mut value: toml::Value = toml! { version = CURRENT_CONFIG_VERSION }.into();
        let outcome =
            migrate_config_file(&config_path, &mut value).expect("migration call should not fail");

        assert!(outcome.is_none());
    }

    #[test]
    fn errors_on_future_version() {
        let mut value = toml::Value::Integer((CURRENT_CONFIG_VERSION + 1) as i64);
        let err = migrate_config_file(Path::new("config.toml"), &mut value).unwrap_err();

        matches!(err, MigrationError::UnsupportedFutureVersion { .. });
    }
}
