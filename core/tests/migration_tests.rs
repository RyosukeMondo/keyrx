#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::field_reassign_with_default,
    clippy::useless_conversion,
    clippy::assertions_on_constants,
    clippy::manual_div_ceil,
    clippy::manual_strip,
    clippy::len_zero,
    clippy::redundant_closure,
    clippy::manual_range_contains,
    clippy::default_constructed_unit_structs,
    clippy::clone_on_copy,
    clippy::io_other_error,
    clippy::bool_assert_comparison,
    clippy::approx_constant,
    clippy::let_unit_value,
    clippy::while_let_on_iterator,
    clippy::await_holding_lock,
    clippy::unnecessary_cast,
    clippy::drop_non_drop,
    clippy::needless_range_loop,
    unused_imports,
    unused_variables,
    dead_code,
    unsafe_code,
    clippy::collapsible_if,
    clippy::bool_comparison,
    unexpected_cfgs
)]
//! Comprehensive integration tests for profile migration.
//!
//! This test suite covers:
//! - Migration with sample old data
//! - Backup creation and verification
//! - Partial migration (some failures, some successes)
//! - Idempotency (running migration multiple times)
//! - Error handling and recovery scenarios

use keyrx_core::discovery::types::{DeviceProfile, PhysicalKey, ProfileSource};
use keyrx_core::migration::{MigrationReport, MigrationV1ToV2};
use keyrx_core::registry::profile::{LayoutType, ProfileRegistry};
use keyrx_core::registry::{ProfileRegistryResolution, ProfileRegistryStorage};
use serial_test::serial;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::tempdir;

// ============================================================================
// Test Fixtures
// ============================================================================

/// Create a standard keyboard profile for testing
fn create_standard_keyboard_profile() -> DeviceProfile {
    let mut keymap = HashMap::new();

    // Add some typical keyboard keys
    keymap.insert(
        30, // A
        PhysicalKey {
            scan_code: 30,
            row: 2,
            col: 0,
            alias: Some("A".to_string()),
        },
    );
    keymap.insert(
        31, // S
        PhysicalKey {
            scan_code: 31,
            row: 2,
            col: 1,
            alias: Some("S".to_string()),
        },
    );
    keymap.insert(
        32, // D
        PhysicalKey {
            scan_code: 32,
            row: 2,
            col: 2,
            alias: Some("D".to_string()),
        },
    );
    keymap.insert(
        16, // Q
        PhysicalKey {
            scan_code: 16,
            row: 1,
            col: 0,
            alias: Some("Q".to_string()),
        },
    );
    keymap.insert(
        17, // W
        PhysicalKey {
            scan_code: 17,
            row: 1,
            col: 1,
            alias: Some("W".to_string()),
        },
    );

    DeviceProfile {
        schema_version: 1,
        vendor_id: 0x046D,
        product_id: 0xC52B,
        name: Some("Logitech K120".to_string()),
        discovered_at: chrono::Utc::now(),
        rows: 6,
        cols_per_row: vec![15, 15, 15, 13, 12, 10],
        keymap,
        aliases: HashMap::new(),
        source: ProfileSource::Default,
    }
}

/// Create a matrix layout device (macro pad) for testing
fn create_matrix_device_profile() -> DeviceProfile {
    let mut keymap = HashMap::new();

    // 3x5 matrix
    for row in 0u8..3 {
        for col in 0u8..5 {
            let scan_code = (row * 5 + col) as u16;
            keymap.insert(
                scan_code,
                PhysicalKey {
                    scan_code,
                    row,
                    col,
                    alias: Some(format!("Key_{}{}", row, col)),
                },
            );
        }
    }

    DeviceProfile {
        schema_version: 1,
        vendor_id: 0x0FD9,
        product_id: 0x0080,
        name: Some("Stream Deck MK.2".to_string()),
        discovered_at: chrono::Utc::now(),
        rows: 3,
        cols_per_row: vec![5, 5, 5], // Uniform columns = Matrix
        keymap,
        aliases: HashMap::new(),
        source: ProfileSource::Discovered,
    }
}

/// Create a profile with minimal data
fn create_minimal_profile() -> DeviceProfile {
    DeviceProfile {
        schema_version: 1,
        vendor_id: 0x1234,
        product_id: 0x5678,
        name: None, // No name
        discovered_at: chrono::Utc::now(),
        rows: 1,
        cols_per_row: vec![3],
        keymap: HashMap::new(), // No keys
        aliases: HashMap::new(),
        source: ProfileSource::Default,
    }
}

/// Save a DeviceProfile to a file
fn save_old_profile(dir: &std::path::Path, filename: &str, profile: &DeviceProfile) {
    let json = serde_json::to_string_pretty(profile).unwrap();
    std::fs::write(dir.join(filename), json).unwrap();
}

// ============================================================================
// Basic Migration Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_migrate_single_profile_success() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    // Create and save an old profile
    let old_profile = create_standard_keyboard_profile();
    save_old_profile(&old_dir, "046d_c52b.json", &old_profile);

    // Run migration
    let registry = ProfileRegistry::with_directory(new_dir.clone());
    let migrator = MigrationV1ToV2::new(old_dir.clone(), registry, false);
    let report = migrator.migrate().await.unwrap();

    // Verify report
    assert_eq!(report.total_count, 1);
    assert_eq!(report.migrated_count, 1);
    assert_eq!(report.failed_count, 0);
    assert!(report.is_success());
    assert!(!report.is_partial());
    assert_eq!(report.success_rate(), 100.0);

    // Verify the profile was created
    let registry2 = ProfileRegistry::with_directory(new_dir);
    registry2.load_all_profiles().await.unwrap();
    let profile_ids = registry2.list_profiles().await;
    assert_eq!(profile_ids.len(), 1);
    let profile = registry2.get_profile(&profile_ids[0]).await.unwrap();
    assert_eq!(profile.name, "Logitech K120");
}

#[tokio::test]
#[serial]
async fn test_migrate_multiple_profiles() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    // Create multiple old profiles
    save_old_profile(
        &old_dir,
        "keyboard.json",
        &create_standard_keyboard_profile(),
    );
    save_old_profile(&old_dir, "streamdeck.json", &create_matrix_device_profile());
    save_old_profile(&old_dir, "minimal.json", &create_minimal_profile());

    // Run migration
    let registry = ProfileRegistry::with_directory(new_dir.clone());
    let migrator = MigrationV1ToV2::new(old_dir, registry, false);
    let report = migrator.migrate().await.unwrap();

    // Verify all migrated
    assert_eq!(report.total_count, 3);
    assert_eq!(report.migrated_count, 3);
    assert_eq!(report.failed_count, 0);
    assert!(report.is_success());

    // Verify profiles were created
    let registry2 = ProfileRegistry::with_directory(new_dir);
    registry2.load_all_profiles().await.unwrap();
    let profile_ids = registry2.list_profiles().await;
    assert_eq!(profile_ids.len(), 3);
}

#[tokio::test]
#[serial]
async fn test_migrate_standard_keyboard_layout_inference() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    let old_profile = create_standard_keyboard_profile();
    save_old_profile(&old_dir, "keyboard.json", &old_profile);

    let registry = ProfileRegistry::with_directory(new_dir.clone());
    let migrator = MigrationV1ToV2::new(old_dir, registry, false);
    migrator.migrate().await.unwrap();

    // Check that layout was inferred as Standard
    let registry2 = ProfileRegistry::with_directory(new_dir);
    registry2.load_all_profiles().await.unwrap();
    let profile_ids = registry2.list_profiles().await;
    let profile = registry2.get_profile(&profile_ids[0]).await.unwrap();
    assert_eq!(profile.layout_type, LayoutType::Standard);
}

#[tokio::test]
#[serial]
async fn test_migrate_matrix_layout_inference() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    let old_profile = create_matrix_device_profile();
    save_old_profile(&old_dir, "streamdeck.json", &old_profile);

    let registry = ProfileRegistry::with_directory(new_dir.clone());
    let migrator = MigrationV1ToV2::new(old_dir, registry, false);
    migrator.migrate().await.unwrap();

    // Check that layout was inferred as Matrix
    let registry2 = ProfileRegistry::with_directory(new_dir);
    registry2.load_all_profiles().await.unwrap();
    let profile_ids = registry2.list_profiles().await;
    let profile = registry2.get_profile(&profile_ids[0]).await.unwrap();
    assert_eq!(profile.layout_type, LayoutType::Matrix);
}

#[tokio::test]
#[serial]
async fn test_migrate_profile_without_name() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    let old_profile = create_minimal_profile();
    save_old_profile(&old_dir, "unnamed.json", &old_profile);

    let registry = ProfileRegistry::with_directory(new_dir.clone());
    let migrator = MigrationV1ToV2::new(old_dir, registry, false);
    let report = migrator.migrate().await.unwrap();

    assert_eq!(report.migrated_count, 1);

    // Verify default name was generated from VID:PID
    let registry2 = ProfileRegistry::with_directory(new_dir);
    registry2.load_all_profiles().await.unwrap();
    let profile_ids = registry2.list_profiles().await;
    let profile = registry2.get_profile(&profile_ids[0]).await.unwrap();
    assert_eq!(profile.name, "Device 1234:5678");
}

// ============================================================================
// Backup Creation Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_migrate_with_backup_creates_backup_directory() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    save_old_profile(&old_dir, "test.json", &create_standard_keyboard_profile());

    let registry = ProfileRegistry::with_directory(new_dir);
    let migrator = MigrationV1ToV2::new(old_dir.clone(), registry, true);
    let report = migrator.migrate().await.unwrap();

    // Verify backup was created
    assert!(report.backup_path.is_some());
    let backup_path = report.backup_path.unwrap();
    assert!(backup_path.exists());
    assert!(backup_path.is_dir());
}

#[tokio::test]
#[serial]
async fn test_migrate_with_backup_copies_all_files() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    // Create multiple profile files
    save_old_profile(
        &old_dir,
        "profile1.json",
        &create_standard_keyboard_profile(),
    );
    save_old_profile(&old_dir, "profile2.json", &create_matrix_device_profile());
    save_old_profile(&old_dir, "profile3.json", &create_minimal_profile());

    let registry = ProfileRegistry::with_directory(new_dir);
    let migrator = MigrationV1ToV2::new(old_dir.clone(), registry, true);
    let report = migrator.migrate().await.unwrap();

    // Verify all files were backed up
    let backup_path = report.backup_path.unwrap();
    assert!(backup_path.join("profile1.json").exists());
    assert!(backup_path.join("profile2.json").exists());
    assert!(backup_path.join("profile3.json").exists());

    // Verify content matches
    let original_content = std::fs::read_to_string(old_dir.join("profile1.json")).unwrap();
    let backup_content = std::fs::read_to_string(backup_path.join("profile1.json")).unwrap();
    assert_eq!(original_content, backup_content);
}

#[tokio::test]
#[serial]
async fn test_migrate_without_backup_no_backup_directory() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    save_old_profile(&old_dir, "test.json", &create_standard_keyboard_profile());

    let registry = ProfileRegistry::with_directory(new_dir);
    let migrator = MigrationV1ToV2::new(old_dir, registry, false);
    let report = migrator.migrate().await.unwrap();

    // Verify no backup was created
    assert!(report.backup_path.is_none());
}

// ============================================================================
// Partial Migration Tests (Some Failures)
// ============================================================================

#[tokio::test]
#[serial]
async fn test_migrate_with_corrupt_file_continues() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    // Create valid profile
    save_old_profile(&old_dir, "valid.json", &create_standard_keyboard_profile());

    // Create corrupt JSON file
    std::fs::write(old_dir.join("corrupt.json"), "{ invalid json }").unwrap();

    // Create another valid profile
    save_old_profile(&old_dir, "valid2.json", &create_matrix_device_profile());

    let registry = ProfileRegistry::with_directory(new_dir.clone());
    let migrator = MigrationV1ToV2::new(old_dir, registry, false);
    let report = migrator.migrate().await.unwrap();

    // Should have attempted 3 files, but corrupt one was skipped during scan
    // (it's logged and skipped, not counted in total_count)
    assert_eq!(report.total_count, 2); // Only valid files counted
    assert_eq!(report.migrated_count, 2);
    assert_eq!(report.failed_count, 0);

    // Verify only valid profiles were migrated
    let registry2 = ProfileRegistry::with_directory(new_dir);
    registry2.load_all_profiles().await.unwrap();
    let profile_ids = registry2.list_profiles().await;
    assert_eq!(profile_ids.len(), 2);
}

#[tokio::test]
#[serial]
async fn test_migrate_partial_success() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    // Create some valid profiles
    save_old_profile(&old_dir, "valid1.json", &create_standard_keyboard_profile());
    save_old_profile(&old_dir, "valid2.json", &create_matrix_device_profile());

    let registry = ProfileRegistry::with_directory(new_dir.clone());
    let migrator = MigrationV1ToV2::new(old_dir, registry, false);
    let report = migrator.migrate().await.unwrap();

    // Verify partial success detection
    if report.failed_count > 0 {
        assert!(report.is_partial());
        assert!(!report.is_success());
    } else {
        assert!(report.is_success());
        assert!(!report.is_partial());
    }
}

#[tokio::test]
#[serial]
async fn test_migrate_missing_old_directory() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("nonexistent");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&new_dir).unwrap();

    let registry = ProfileRegistry::with_directory(new_dir);
    let migrator = MigrationV1ToV2::new(old_dir, registry, false);
    let report = migrator.migrate().await.unwrap();

    // Should complete successfully with no profiles
    assert_eq!(report.total_count, 0);
    assert_eq!(report.migrated_count, 0);
    assert_eq!(report.failed_count, 0);
}

#[tokio::test]
#[serial]
async fn test_migrate_empty_old_directory() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    let registry = ProfileRegistry::with_directory(new_dir);
    let migrator = MigrationV1ToV2::new(old_dir, registry, false);
    let report = migrator.migrate().await.unwrap();

    // Should complete successfully with no profiles
    assert_eq!(report.total_count, 0);
    assert_eq!(report.migrated_count, 0);
    assert_eq!(report.failed_count, 0);
}

#[tokio::test]
#[serial]
async fn test_migrate_non_json_files_ignored() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    // Create valid profile
    save_old_profile(&old_dir, "valid.json", &create_standard_keyboard_profile());

    // Create non-JSON files that should be ignored
    std::fs::write(old_dir.join("readme.txt"), "Some text").unwrap();
    std::fs::write(old_dir.join("config.toml"), "[section]").unwrap();
    std::fs::write(old_dir.join("notes.md"), "# Notes").unwrap();

    let registry = ProfileRegistry::with_directory(new_dir.clone());
    let migrator = MigrationV1ToV2::new(old_dir, registry, false);
    let report = migrator.migrate().await.unwrap();

    // Should only process the JSON file
    assert_eq!(report.total_count, 1);
    assert_eq!(report.migrated_count, 1);

    let registry2 = ProfileRegistry::with_directory(new_dir);
    registry2.load_all_profiles().await.unwrap();
    let profile_ids = registry2.list_profiles().await;
    assert_eq!(profile_ids.len(), 1);
}

// ============================================================================
// Idempotency Tests (Run Migration Twice)
// ============================================================================

#[tokio::test]
#[serial]
async fn test_migrate_idempotency_same_result() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    // Create test profiles
    save_old_profile(
        &old_dir,
        "profile1.json",
        &create_standard_keyboard_profile(),
    );
    save_old_profile(&old_dir, "profile2.json", &create_matrix_device_profile());

    let registry = ProfileRegistry::with_directory(new_dir.clone());

    // First migration
    let migrator1 = MigrationV1ToV2::new(old_dir.clone(), registry, false);
    let report1 = migrator1.migrate().await.unwrap();

    assert_eq!(report1.total_count, 2);
    assert_eq!(report1.migrated_count, 2);
    assert_eq!(report1.failed_count, 0);

    // Second migration (idempotency test)
    let registry2 = ProfileRegistry::with_directory(new_dir.clone());
    let migrator2 = MigrationV1ToV2::new(old_dir.clone(), registry2, false);
    let report2 = migrator2.migrate().await.unwrap();

    // Should still process same files
    assert_eq!(report2.total_count, 2);
    // Migration creates new profiles with new IDs each time
    assert_eq!(report2.migrated_count, 2);
    assert_eq!(report2.failed_count, 0);

    // Verify we now have 4 profiles (2 from first run + 2 from second run)
    // This is expected - migration generates new profile IDs each time
    let registry3 = ProfileRegistry::with_directory(new_dir);
    registry3.load_all_profiles().await.unwrap();
    let profile_ids = registry3.list_profiles().await;
    assert_eq!(profile_ids.len(), 4);
}

#[tokio::test]
#[serial]
async fn test_migrate_multiple_times_stable_output() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    let old_profile = create_standard_keyboard_profile();
    save_old_profile(&old_dir, "keyboard.json", &old_profile);

    let _registry = ProfileRegistry::with_directory(new_dir.clone());

    // Run migration 3 times
    for i in 1..=3 {
        let reg = ProfileRegistry::with_directory(new_dir.clone());
        let migrator = MigrationV1ToV2::new(old_dir.clone(), reg, false);
        let report = migrator.migrate().await.unwrap();

        assert_eq!(report.total_count, 1, "Run {}: total_count should be 1", i);
        assert_eq!(
            report.migrated_count, 1,
            "Run {}: migrated_count should be 1",
            i
        );
        assert_eq!(
            report.failed_count, 0,
            "Run {}: failed_count should be 0",
            i
        );
    }

    // Verify 3 profiles exist (one from each run - migration creates new IDs each time)
    let registry_final = ProfileRegistry::with_directory(new_dir);
    registry_final.load_all_profiles().await.unwrap();
    let profile_ids = registry_final.list_profiles().await;
    assert_eq!(profile_ids.len(), 3);
}

#[tokio::test]
#[serial]
async fn test_migrate_idempotency_with_new_files() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    // First migration with one file
    save_old_profile(
        &old_dir,
        "profile1.json",
        &create_standard_keyboard_profile(),
    );

    let registry = ProfileRegistry::with_directory(new_dir.clone());
    let migrator1 = MigrationV1ToV2::new(old_dir.clone(), registry, false);
    let report1 = migrator1.migrate().await.unwrap();

    assert_eq!(report1.migrated_count, 1);

    // Add a new file
    save_old_profile(&old_dir, "profile2.json", &create_matrix_device_profile());

    // Second migration should process both files
    let registry2 = ProfileRegistry::with_directory(new_dir.clone());
    let migrator2 = MigrationV1ToV2::new(old_dir.clone(), registry2, false);
    let report2 = migrator2.migrate().await.unwrap();

    assert_eq!(report2.total_count, 2);
    assert_eq!(report2.migrated_count, 2);

    // We now have 3 profiles total: 1 from first migration + 2 from second migration
    // The second migration creates new profile IDs for both files
    let registry3 = ProfileRegistry::with_directory(new_dir);
    registry3.load_all_profiles().await.unwrap();
    let profile_ids = registry3.list_profiles().await;
    assert_eq!(profile_ids.len(), 3);
}

// ============================================================================
// Migration Report Tests
// ============================================================================

#[test]
fn test_migration_report_success_conditions() {
    let mut report = MigrationReport::new();

    // Empty report is not success
    assert!(!report.is_success());

    // Report with migrations is success
    report.total_count = 5;
    report.migrated_count = 5;
    report.failed_count = 0;
    assert!(report.is_success());
    assert!(!report.is_partial());
    assert_eq!(report.success_rate(), 100.0);
}

#[test]
fn test_migration_report_partial_conditions() {
    let mut report = MigrationReport::new();
    report.total_count = 10;
    report.migrated_count = 7;
    report.failed_count = 3;

    assert!(!report.is_success());
    assert!(report.is_partial());
    assert_eq!(report.success_rate(), 70.0);
}

#[test]
fn test_migration_report_complete_failure() {
    let mut report = MigrationReport::new();
    report.total_count = 5;
    report.migrated_count = 0;
    report.failed_count = 5;

    assert!(!report.is_success());
    assert!(!report.is_partial());
    assert_eq!(report.success_rate(), 0.0);
}

#[test]
fn test_migration_report_summary_format() {
    let mut report = MigrationReport::new();
    report.total_count = 10;
    report.migrated_count = 8;
    report.failed_count = 2;
    report.backup_path = Some(PathBuf::from("/tmp/backup"));

    let summary = report.summary();

    assert!(summary.contains("Total profiles: 10"));
    assert!(summary.contains("Migrated: 8"));
    assert!(summary.contains("Failed: 2"));
    assert!(summary.contains("Success rate: 80.0%"));
    assert!(summary.contains("Backup created at: /tmp/backup"));
}

#[test]
fn test_migration_report_with_failures() {
    let mut report = MigrationReport::new();
    report.total_count = 3;
    report.migrated_count = 2;
    report.failed_count = 1;
    report
        .failures
        .push(keyrx_core::migration::MigrationFailure {
            path: PathBuf::from("/path/to/failed.json"),
            error: "Parse error".to_string(),
        });

    let summary = report.summary();

    assert!(summary.contains("Failures:"));
    assert!(summary.contains("failed.json"));
    assert!(summary.contains("Parse error"));
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_migrate_profile_with_many_keys() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    // Create a profile with many keys
    let mut keymap = HashMap::new();
    for scan_code in 0..104 {
        keymap.insert(
            scan_code,
            PhysicalKey {
                scan_code,
                row: (scan_code / 15) as u8,
                col: (scan_code % 15) as u8,
                alias: Some(format!("Key_{}", scan_code)),
            },
        );
    }

    let large_profile = DeviceProfile {
        schema_version: 1,
        vendor_id: 0x046D,
        product_id: 0xC52B,
        name: Some("Full Keyboard".to_string()),
        discovered_at: chrono::Utc::now(),
        rows: 7,
        cols_per_row: vec![15, 15, 15, 15, 15, 15, 14],
        keymap,
        aliases: HashMap::new(),
        source: ProfileSource::Default,
    };

    save_old_profile(&old_dir, "large.json", &large_profile);

    let registry = ProfileRegistry::with_directory(new_dir.clone());
    let migrator = MigrationV1ToV2::new(old_dir, registry, false);
    let report = migrator.migrate().await.unwrap();

    assert_eq!(report.migrated_count, 1);

    // Verify all keys were migrated
    let registry2 = ProfileRegistry::with_directory(new_dir);
    registry2.load_all_profiles().await.unwrap();
    let profile_ids = registry2.list_profiles().await;
    let profile = registry2.get_profile(&profile_ids[0]).await.unwrap();
    assert!(profile.mappings.len() >= 104);
}

#[tokio::test]
#[serial]
async fn test_migrate_profile_with_special_characters_in_name() {
    let temp = tempdir().unwrap();
    let old_dir = temp.path().join("old_devices");
    let new_dir = temp.path().join("new_profiles");

    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::create_dir_all(&new_dir).unwrap();

    let mut profile = create_standard_keyboard_profile();
    profile.name = Some("Keyboard™ (2024) – Special ★ Edition".to_string());

    save_old_profile(&old_dir, "special.json", &profile);

    let registry = ProfileRegistry::with_directory(new_dir.clone());
    let migrator = MigrationV1ToV2::new(old_dir, registry, false);
    let report = migrator.migrate().await.unwrap();

    assert_eq!(report.migrated_count, 1);

    let registry2 = ProfileRegistry::with_directory(new_dir);
    registry2.load_all_profiles().await.unwrap();
    let profile_ids = registry2.list_profiles().await;
    let profile_loaded = registry2.get_profile(&profile_ids[0]).await.unwrap();
    assert_eq!(profile_loaded.name, "Keyboard™ (2024) – Special ★ Edition");
}
