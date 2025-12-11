//! Migration from V1 (DeviceProfile) to V2 (revolutionary mapping Profile).
//!
//! This module handles the conversion of old device-specific profiles
//! to the new layout-aware profile system.

use super::{MigrationError, MigrationFailure, MigrationReport};
use crate::discovery::types::DeviceProfile;
use crate::registry::profile::{KeyAction, LayoutType, PhysicalPosition, Profile, ProfileRegistry};
use crate::registry::ProfileRegistryStorage;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Migrator for converting V1 profiles to V2
pub struct MigrationV1ToV2 {
    /// Directory containing old V1 profiles
    old_profiles_dir: PathBuf,

    /// ProfileRegistry for saving new V2 profiles
    profile_registry: ProfileRegistry,

    /// Whether to create backups before migration
    create_backup: bool,
}

impl MigrationV1ToV2 {
    /// Create a new migration instance
    pub fn new(
        old_profiles_dir: PathBuf,
        profile_registry: ProfileRegistry,
        create_backup: bool,
    ) -> Self {
        Self {
            old_profiles_dir,
            profile_registry,
            create_backup,
        }
    }

    /// Run the migration, converting all old profiles to new format
    pub async fn migrate(&self) -> Result<MigrationReport, MigrationError> {
        let mut report = MigrationReport::new();

        info!(
            service = "keyrx",
            event = "migration_started",
            component = "migration",
            old_dir = %self.old_profiles_dir.display(),
            "Starting migration from V1 to V2"
        );

        // Create backup if requested
        if self.create_backup {
            match self.create_backup_dir().await {
                Ok(backup_path) => {
                    report.backup_path = Some(backup_path.clone());
                    info!(
                        service = "keyrx",
                        event = "backup_created",
                        component = "migration",
                        path = %backup_path.display(),
                        "Backup created successfully"
                    );
                }
                Err(e) => {
                    return Err(MigrationError::BackupError(format!(
                        "Failed to create backup: {}",
                        e
                    )));
                }
            }
        }

        // Check if old profiles directory exists
        if !self.old_profiles_dir.exists() {
            warn!(
                service = "keyrx",
                event = "migration_skipped",
                component = "migration",
                reason = "old_profiles_dir_not_found",
                path = %self.old_profiles_dir.display(),
                "Old profiles directory not found, skipping migration"
            );
            return Ok(report);
        }

        // Scan for old profile files
        let old_profiles = self.scan_old_profiles()?;
        report.total_count = old_profiles.len();

        info!(
            service = "keyrx",
            event = "profiles_scanned",
            component = "migration",
            count = report.total_count,
            "Found {} old profiles to migrate",
            report.total_count
        );

        // Convert each old profile
        for (path, old_profile) in old_profiles {
            match self.convert_profile(&old_profile).await {
                Ok(new_profile) => {
                    // Save the new profile
                    if let Err(e) = self.profile_registry.save_profile(&new_profile).await {
                        report.failed_count += 1;
                        report.failures.push(MigrationFailure {
                            path: path.clone(),
                            error: format!("Failed to save new profile: {}", e),
                        });
                        warn!(
                            service = "keyrx",
                            event = "migration_save_failed",
                            component = "migration",
                            path = %path.display(),
                            error = %e,
                            "Failed to save migrated profile"
                        );
                    } else {
                        report.migrated_count += 1;
                        info!(
                            service = "keyrx",
                            event = "profile_migrated",
                            component = "migration",
                            path = %path.display(),
                            old_name = ?old_profile.name,
                            new_name = %new_profile.name,
                            "Profile migrated successfully"
                        );
                    }
                }
                Err(e) => {
                    report.failed_count += 1;
                    report.failures.push(MigrationFailure {
                        path: path.clone(),
                        error: format!("Conversion failed: {}", e),
                    });
                    warn!(
                        service = "keyrx",
                        event = "migration_conversion_failed",
                        component = "migration",
                        path = %path.display(),
                        error = %e,
                        "Failed to convert profile"
                    );
                }
            }
        }

        info!(
            service = "keyrx",
            event = "migration_completed",
            component = "migration",
            migrated = report.migrated_count,
            failed = report.failed_count,
            total = report.total_count,
            success_rate = report.success_rate(),
            "Migration completed: {} migrated, {} failed out of {} total",
            report.migrated_count,
            report.failed_count,
            report.total_count
        );

        Ok(report)
    }

    /// Scan old profiles directory and load all V1 profiles
    fn scan_old_profiles(&self) -> Result<Vec<(PathBuf, DeviceProfile)>, MigrationError> {
        let mut profiles = Vec::new();

        let entries =
            std::fs::read_dir(&self.old_profiles_dir).map_err(|source| MigrationError::Io {
                path: self.old_profiles_dir.clone(),
                source,
            })?;

        for entry in entries {
            let entry = entry.map_err(|source| MigrationError::Io {
                path: self.old_profiles_dir.clone(),
                source,
            })?;

            let path = entry.path();

            // Only process JSON files
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            // Try to load the old profile
            match self.load_old_profile(&path) {
                Ok(profile) => {
                    profiles.push((path, profile));
                }
                Err(e) => {
                    warn!(
                        service = "keyrx",
                        event = "old_profile_load_failed",
                        component = "migration",
                        path = %path.display(),
                        error = %e,
                        "Failed to load old profile, skipping"
                    );
                    // Don't fail the whole migration if one file is corrupt
                    continue;
                }
            }
        }

        Ok(profiles)
    }

    /// Load an old V1 profile from file
    fn load_old_profile(&self, path: &Path) -> Result<DeviceProfile, MigrationError> {
        let data = std::fs::read_to_string(path).map_err(|source| MigrationError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        let profile: DeviceProfile =
            serde_json::from_str(&data).map_err(|source| MigrationError::ParseError {
                path: path.to_path_buf(),
                source,
            })?;

        Ok(profile)
    }

    /// Convert an old V1 profile to new V2 profile
    async fn convert_profile(&self, old: &DeviceProfile) -> Result<Profile, MigrationError> {
        // Determine layout type based on device structure
        let layout_type = self.infer_layout_type(old);

        // Generate profile name from device info
        let profile_name = old
            .name
            .clone()
            .unwrap_or_else(|| format!("Device {:04x}:{:04x}", old.vendor_id, old.product_id));

        // Create new profile
        let mut new_profile = Profile::new(&profile_name, layout_type);

        // Convert keymap to new mappings format
        // Old format: scan_code -> PhysicalKey(scan_code, row, col, alias)
        // New format: PhysicalPosition(row, col) -> KeyAction
        for physical_key in old.keymap.values() {
            let position = PhysicalPosition::new(physical_key.row, physical_key.col);

            // For migration, we use Pass action by default since we don't have
            // the actual remap targets from the old system
            // Users will need to configure their remaps in the new system
            let action = KeyAction::Pass;

            new_profile.set_action(position, action);
        }

        // If there are no explicit mappings, create default passthrough mappings
        // based on the device layout
        if new_profile.mappings.is_empty() {
            self.create_default_mappings(&mut new_profile, old);
        }

        Ok(new_profile)
    }

    /// Infer the layout type from old device profile
    fn infer_layout_type(&self, old: &DeviceProfile) -> LayoutType {
        // Heuristic: if it's a standard keyboard size, use Standard
        // Otherwise, use Matrix

        // Standard keyboards typically have 5-7 rows
        let has_standard_rows = old.rows >= 5 && old.rows <= 7;

        // Standard keyboards typically have cols_per_row pattern like:
        // [15, 15, 15, 13, 12, 10] or similar
        // Matrix layouts have uniform columns
        let has_varying_columns = if old.cols_per_row.len() > 1 {
            let first_col = old.cols_per_row[0];
            old.cols_per_row.iter().any(|&c| c != first_col)
        } else {
            false
        };

        // If it has standard keyboard characteristics, use Standard
        if has_standard_rows && has_varying_columns {
            LayoutType::Standard
        } else {
            // Matrix layout for macro pads, Stream Decks, etc.
            LayoutType::Matrix
        }
    }

    /// Create default passthrough mappings for a profile based on old device layout
    fn create_default_mappings(&self, profile: &mut Profile, old: &DeviceProfile) {
        // Create a passthrough mapping for each key position in the old profile
        for row in 0..old.rows {
            let cols = old.cols_per_row.get(row as usize).copied().unwrap_or(0);
            for col in 0..cols {
                let position = PhysicalPosition::new(row, col);
                profile.set_action(position, KeyAction::Pass);
            }
        }
    }

    /// Create a backup of the old profiles directory
    async fn create_backup_dir(&self) -> Result<PathBuf, MigrationError> {
        let backup_dir = self.old_profiles_dir.with_extension("backup");

        // Add timestamp to make it unique
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_dir = PathBuf::from(format!("{}.{}", backup_dir.display(), timestamp));

        // Create backup directory
        std::fs::create_dir_all(&backup_dir).map_err(|source| MigrationError::Io {
            path: backup_dir.clone(),
            source,
        })?;

        // Copy all files from old directory to backup
        let entries =
            std::fs::read_dir(&self.old_profiles_dir).map_err(|source| MigrationError::Io {
                path: self.old_profiles_dir.clone(),
                source,
            })?;

        for entry in entries {
            let entry = entry.map_err(|source| MigrationError::Io {
                path: self.old_profiles_dir.clone(),
                source,
            })?;

            let src_path = entry.path();
            if src_path.is_file() {
                let file_name = src_path.file_name().ok_or_else(|| {
                    MigrationError::BackupError(format!(
                        "Failed to get filename from {}",
                        src_path.display()
                    ))
                })?;

                let dst_path = backup_dir.join(file_name);
                std::fs::copy(&src_path, &dst_path).map_err(|source| MigrationError::Io {
                    path: src_path.clone(),
                    source,
                })?;
            }
        }

        Ok(backup_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::types::PhysicalKey;
    use serial_test::serial;
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn create_test_device_profile() -> DeviceProfile {
        let mut keymap = HashMap::new();
        keymap.insert(
            30, // Scancode for 'A'
            PhysicalKey {
                scan_code: 30,
                row: 2,
                col: 0,
                alias: Some("A".to_string()),
            },
        );
        keymap.insert(
            31, // Scancode for 'B'
            PhysicalKey {
                scan_code: 31,
                row: 2,
                col: 1,
                alias: Some("B".to_string()),
            },
        );

        DeviceProfile {
            schema_version: 1,
            vendor_id: 0x1234,
            product_id: 0x5678,
            name: Some("Test Keyboard".to_string()),
            discovered_at: chrono::Utc::now(),
            rows: 6,
            cols_per_row: vec![15, 15, 15, 13, 12, 10],
            keymap,
            aliases: HashMap::new(),
            source: crate::discovery::types::ProfileSource::Default,
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_convert_profile() {
        let old_profile = create_test_device_profile();
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let migrator = MigrationV1ToV2::new(PathBuf::from("/tmp"), registry, false);

        let new_profile = migrator.convert_profile(&old_profile).await.unwrap();

        assert_eq!(new_profile.name, "Test Keyboard");
        assert_eq!(new_profile.layout_type, LayoutType::Standard);
        assert!(new_profile.mappings.len() >= 2); // At least the two keys we mapped
    }

    #[tokio::test]
    #[serial]
    async fn test_infer_layout_type_standard() {
        let old_profile = create_test_device_profile();
        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let migrator = MigrationV1ToV2::new(PathBuf::from("/tmp"), registry, false);

        // This should be inferred as Standard because it has 6 rows
        let layout_type = migrator.infer_layout_type(&old_profile);
        assert_eq!(layout_type, LayoutType::Standard);
    }

    #[tokio::test]
    #[serial]
    async fn test_infer_layout_type_matrix() {
        let mut old_profile = create_test_device_profile();
        old_profile.rows = 3;
        old_profile.cols_per_row = vec![5, 5, 5];

        let temp = tempdir().unwrap();
        let registry = ProfileRegistry::with_directory(temp.path().to_path_buf());

        let migrator = MigrationV1ToV2::new(PathBuf::from("/tmp"), registry, false);

        let layout_type = migrator.infer_layout_type(&old_profile);
        assert_eq!(layout_type, LayoutType::Matrix);
    }

    #[tokio::test]
    #[serial]
    async fn test_migration_with_backup() {
        let temp = tempdir().unwrap();
        let old_dir = temp.path().join("old_devices");
        let new_dir = temp.path().join("new_profiles");

        std::fs::create_dir_all(&old_dir).unwrap();
        std::fs::create_dir_all(&new_dir).unwrap();

        // Create a test old profile file
        let old_profile = create_test_device_profile();
        let profile_json = serde_json::to_string_pretty(&old_profile).unwrap();
        std::fs::write(old_dir.join("1234_5678.json"), profile_json).unwrap();

        let registry = ProfileRegistry::with_directory(new_dir);
        let migrator = MigrationV1ToV2::new(old_dir.clone(), registry, true);

        let report = migrator.migrate().await.unwrap();

        assert_eq!(report.total_count, 1);
        assert_eq!(report.migrated_count, 1);
        assert_eq!(report.failed_count, 0);
        assert!(report.backup_path.is_some());
        assert!(report.backup_path.unwrap().exists());
    }

    #[tokio::test]
    #[serial]
    async fn test_migration_without_backup() {
        let temp = tempdir().unwrap();
        let old_dir = temp.path().join("old_devices");
        let new_dir = temp.path().join("new_profiles");

        std::fs::create_dir_all(&old_dir).unwrap();
        std::fs::create_dir_all(&new_dir).unwrap();

        let old_profile = create_test_device_profile();
        let profile_json = serde_json::to_string_pretty(&old_profile).unwrap();
        std::fs::write(old_dir.join("1234_5678.json"), profile_json).unwrap();

        let registry = ProfileRegistry::with_directory(new_dir);
        let migrator = MigrationV1ToV2::new(old_dir, registry, false);

        let report = migrator.migrate().await.unwrap();

        assert_eq!(report.total_count, 1);
        assert_eq!(report.migrated_count, 1);
        assert!(report.backup_path.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_migration_report() {
        let mut report = MigrationReport::new();
        report.total_count = 10;
        report.migrated_count = 8;
        report.failed_count = 2;

        assert!(!report.is_success());
        assert!(report.is_partial());
        assert_eq!(report.success_rate(), 80.0);

        let summary = report.summary();
        assert!(summary.contains("Total profiles: 10"));
        assert!(summary.contains("Migrated: 8"));
        assert!(summary.contains("Failed: 2"));
    }
}
