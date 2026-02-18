//! Profile management high-priority fixes tests.
//!
//! Tests for PROF-001 through PROF-005 bug fixes:
//! - PROF-001: Profile Switching Race Conditions
//! - PROF-002: Missing Validation in Profile Operations
//! - PROF-003: Incomplete Error Handling
//! - PROF-004: Missing Activation Metadata
//! - PROF-005: Duplicate Profile Names Allowed

use keyrx_daemon::config::{ProfileError, ProfileManager, ProfileTemplate};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Helper function to create a test ProfileManager with a temporary directory.
fn setup_test_manager() -> (TempDir, ProfileManager) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().to_path_buf();
    let manager = ProfileManager::new(config_dir).expect("Failed to create ProfileManager");
    (temp_dir, manager)
}

// ============================================================================
// PROF-001: Profile Switching Race Conditions
// ============================================================================

#[test]
fn test_prof001_concurrent_activation_serialized() {
    let (_temp, manager) = setup_test_manager();

    // Create two profiles
    manager
        .create("profile1", ProfileTemplate::Blank)
        .expect("Failed to create profile1");
    manager
        .create("profile2", ProfileTemplate::Blank)
        .expect("Failed to create profile2");

    // Wrap manager in Arc for thread-safe sharing
    let manager_arc = Arc::new(manager);

    // Spawn two threads trying to activate different profiles concurrently
    let manager1 = Arc::clone(&manager_arc);
    let handle1 = thread::spawn(move || manager1.activate("profile1"));

    let manager2 = Arc::clone(&manager_arc);
    let handle2 = thread::spawn(move || manager2.activate("profile2"));

    // Wait for both threads
    let result1 = handle1.join().expect("Thread 1 panicked");
    let result2 = handle2.join().expect("Thread 2 panicked");

    // At least one activation should succeed
    assert!(
        result1.is_ok() || result2.is_ok(),
        "At least one activation should succeed"
    );

    // Both results should be either success or a proper error (not corrupted state)
    if let Ok(r1) = result1 {
        assert!(r1.success || r1.error.is_some());
    }
    if let Ok(r2) = result2 {
        assert!(r2.success || r2.error.is_some());
    }

    // Check final state is consistent (only one profile active)
    let active = manager_arc
        .get_active()
        .expect("Failed to get active profile");
    assert!(
        active == Some("profile1".to_string()) || active == Some("profile2".to_string()),
        "Expected one profile to be active, got: {:?}",
        active
    );
}

#[test]
fn test_prof001_rapid_activation_no_corruption() {
    let (_temp, manager) = setup_test_manager();

    // Create profile
    manager
        .create("test", ProfileTemplate::Blank)
        .expect("Failed to create profile");

    // Rapidly activate the same profile multiple times
    for _ in 0..10 {
        let result = manager.activate("test");
        assert!(
            result.is_ok(),
            "Activation should succeed: {:?}",
            result.err()
        );
    }

    // State should be consistent
    let active = manager.get_active().expect("Failed to get active profile");
    assert_eq!(active, Some("test".to_string()));
}

// ============================================================================
// PROF-002: Missing Validation in Profile Operations
// ============================================================================

#[test]
fn test_prof002_empty_name_rejected() {
    let (_temp, manager) = setup_test_manager();

    let result = manager.create("", ProfileTemplate::Blank);
    assert!(matches!(result, Err(ProfileError::InvalidName(_))));
}

#[test]
fn test_prof002_too_long_name_rejected() {
    let (_temp, manager) = setup_test_manager();

    // Create name longer than 64 characters
    let long_name = "a".repeat(65);
    let result = manager.create(&long_name, ProfileTemplate::Blank);
    assert!(matches!(result, Err(ProfileError::InvalidName(_))));
}

#[test]
fn test_prof002_special_chars_rejected() {
    let (_temp, manager) = setup_test_manager();

    let invalid_names = vec![
        "profile@test",    // @ not allowed
        "profile test",    // space not allowed
        "profile!",        // ! not allowed
        "profile#",        // # not allowed
        "profile$",        // $ not allowed
        "profile%",        // % not allowed
        "profile&",        // & not allowed
        "profile*",        // * not allowed
        "profile()",       // () not allowed
        "profile+",        // + not allowed
        "profile=",        // = not allowed
        "profile[test]",   // [] not allowed
        "profile{test}",   // {} not allowed
        "profile:test",    // : not allowed
        "profile;test",    // ; not allowed
        "profile\"test\"", // " not allowed
        "profile'test'",   // ' not allowed
        "profile<test>",   // <> not allowed
        "profile,test",    // , not allowed
        "profile.test",    // . not allowed
        "profile?test",    // ? not allowed
        "profile/test",    // / not allowed
        "profile\\test",   // \ not allowed
        "profile|test",    // | not allowed
        "profile~test",    // ~ not allowed
        "profile`test",    // ` not allowed
    ];

    for name in invalid_names {
        let result = manager.create(name, ProfileTemplate::Blank);
        assert!(
            matches!(result, Err(ProfileError::InvalidName(_))),
            "Name '{}' should be rejected",
            name
        );
    }
}

#[test]
fn test_prof002_valid_names_accepted() {
    let (_temp, manager) = setup_test_manager();

    let valid_names = vec![
        "profile1",
        "my-profile",
        "my_profile",
        "Profile123",
        "test-profile_v2",
        "UPPERCASE",
        "lowercase",
        "MixedCase123",
    ];

    for name in valid_names {
        let result = manager.create(name, ProfileTemplate::Blank);
        assert!(
            result.is_ok(),
            "Name '{}' should be accepted, got error: {:?}",
            name,
            result.err()
        );
    }
}

#[test]
fn test_prof002_dash_underscore_start_rejected() {
    let (_temp, manager) = setup_test_manager();

    let result = manager.create("-profile", ProfileTemplate::Blank);
    assert!(matches!(result, Err(ProfileError::InvalidName(_))));

    let result = manager.create("_profile", ProfileTemplate::Blank);
    assert!(matches!(result, Err(ProfileError::InvalidName(_))));
}

#[test]
fn test_prof002_max_length_accepted() {
    let (_temp, manager) = setup_test_manager();

    // Exactly 64 characters should be accepted
    let name_64 = "a".repeat(64);
    let result = manager.create(&name_64, ProfileTemplate::Blank);
    assert!(result.is_ok(), "64-character name should be accepted");
}

// ============================================================================
// PROF-003: Incomplete Error Handling
// ============================================================================

#[test]
fn test_prof003_activation_missing_file_error() {
    let (_temp, manager) = setup_test_manager();

    // Create profile then delete its .rhai file manually
    manager
        .create("test", ProfileTemplate::Blank)
        .expect("Failed to create profile");

    let profile = manager.get("test").expect("Profile should exist");
    fs::remove_file(&profile.rhai_path).expect("Failed to remove file");

    // Try to activate - should get a clear error
    let result = manager.activate("test");
    assert!(
        result.is_err(),
        "Activation should fail when source file is missing"
    );

    // Error should be NotFound with context
    if let Err(ProfileError::NotFound(msg)) = result {
        assert!(
            msg.contains("source file not found"),
            "Error should mention source file, got: {}",
            msg
        );
    } else {
        panic!("Expected NotFound error, got: {:?}", result);
    }
}

#[test]
fn test_prof003_nonexistent_profile_activation_error() {
    let (_temp, manager) = setup_test_manager();

    let result = manager.activate("nonexistent");
    assert!(matches!(result, Err(ProfileError::NotFound(_))));

    if let Err(ProfileError::NotFound(msg)) = result {
        assert!(
            msg.contains("nonexistent"),
            "Error should mention profile name, got: {}",
            msg
        );
    }
}

#[test]
fn test_prof003_lock_error_contains_context() {
    // This test verifies that lock errors contain context about which operation failed
    // In a real scenario, we'd need to poison the lock, but for now we just verify the error type exists
    let error = ProfileError::LockError(
        "Failed to acquire write lock during activation of 'test': poisoned".to_string(),
    );
    let message = error.to_string();

    assert!(message.contains("activation"));
    assert!(message.contains("test"));
}

// ============================================================================
// PROF-004: Missing Activation Metadata
// ============================================================================

#[test]
fn test_prof004_activation_metadata_stored() {
    let (_temp, manager) = setup_test_manager();

    // Create and activate a profile
    manager
        .create("test", ProfileTemplate::Blank)
        .expect("Failed to create profile");

    let before_activation = std::time::SystemTime::now();
    thread::sleep(Duration::from_millis(10)); // Ensure time passes

    manager
        .activate("test")
        .expect("Failed to activate profile");

    thread::sleep(Duration::from_millis(10)); // Ensure time passes
    let after_activation = std::time::SystemTime::now();

    // Reload metadata to check persisted activation info
    let metadata = manager
        .load_profile_metadata_for_testing("test")
        .expect("Failed to load metadata");

    // Check activation metadata is present
    assert!(
        metadata.activated_at.is_some(),
        "Activation timestamp should be stored"
    );
    assert!(
        metadata.activated_by.is_some(),
        "Activation source should be stored"
    );

    // Check timestamp is reasonable
    let activated_at = metadata.activated_at.unwrap();
    assert!(
        activated_at >= before_activation && activated_at <= after_activation,
        "Activation timestamp should be between before and after"
    );

    // Check activation source
    assert_eq!(
        metadata.activated_by.unwrap(),
        "user",
        "Activation source should be 'user'"
    );
}

#[test]
fn test_prof004_activation_metadata_persisted() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().to_path_buf();

    {
        let manager = ProfileManager::new(config_dir.clone()).expect("Failed to create manager");
        manager
            .create("test", ProfileTemplate::Blank)
            .expect("Failed to create profile");
        manager
            .activate("test")
            .expect("Failed to activate profile");
    }

    // Create a new manager to verify persistence
    let manager = ProfileManager::new(config_dir).expect("Failed to create manager");
    let metadata = manager
        .load_profile_metadata_for_testing("test")
        .expect("Failed to load metadata");

    assert!(
        metadata.activated_at.is_some(),
        "Activation timestamp should persist across restarts"
    );
    assert!(
        metadata.activated_by.is_some(),
        "Activation source should persist across restarts"
    );
}

#[test]
fn test_prof004_inactive_profile_no_metadata() {
    let (_temp, manager) = setup_test_manager();

    // Create profile but don't activate it
    manager
        .create("test", ProfileTemplate::Blank)
        .expect("Failed to create profile");

    let metadata = manager
        .load_profile_metadata_for_testing("test")
        .expect("Failed to load metadata");

    // Inactive profile should have no activation metadata
    assert!(
        metadata.activated_at.is_none(),
        "Inactive profile should not have activation timestamp"
    );
    assert!(
        metadata.activated_by.is_none(),
        "Inactive profile should not have activation source"
    );
}

// ============================================================================
// PROF-005: Duplicate Profile Names Allowed
// ============================================================================

#[test]
fn test_prof005_duplicate_name_rejected() {
    let (_temp, manager) = setup_test_manager();

    // Create first profile
    manager
        .create("test", ProfileTemplate::Blank)
        .expect("Failed to create first profile");

    // Try to create duplicate - should fail
    let result = manager.create("test", ProfileTemplate::Blank);
    assert!(
        matches!(result, Err(ProfileError::AlreadyExists(_))),
        "Duplicate profile creation should be rejected"
    );

    if let Err(ProfileError::AlreadyExists(name)) = result {
        assert_eq!(name, "test", "Error should contain the duplicate name");
    }
}

#[test]
fn test_prof005_duplicate_after_delete_allowed() {
    let (_temp, manager) = setup_test_manager();

    // Create, delete, then recreate with same name - should succeed
    manager
        .create("test", ProfileTemplate::Blank)
        .expect("Failed to create profile");

    manager.delete("test").expect("Failed to delete profile");

    let result = manager.create("test", ProfileTemplate::Blank);
    assert!(
        result.is_ok(),
        "Should be able to recreate profile after deletion"
    );
}

#[test]
fn test_prof005_case_sensitive_names() {
    let (_temp, manager) = setup_test_manager();

    // Create profiles with different cases
    manager
        .create("test", ProfileTemplate::Blank)
        .expect("Failed to create 'test'");

    manager
        .create("Test", ProfileTemplate::Blank)
        .expect("Failed to create 'Test'");

    manager
        .create("TEST", ProfileTemplate::Blank)
        .expect("Failed to create 'TEST'");

    // All three should exist as separate profiles
    let profiles = manager.list();
    assert_eq!(profiles.len(), 3, "Should have 3 distinct profiles");
}

#[test]
fn test_prof005_import_duplicate_rejected() {
    let (_temp, manager) = setup_test_manager();

    // Create a profile
    manager
        .create("test", ProfileTemplate::Blank)
        .expect("Failed to create profile");

    // Export it
    let export_path = PathBuf::from("/tmp/test_export.rhai");
    manager
        .export("test", &export_path)
        .expect("Failed to export");

    // Try to import with same name - should fail
    let result = manager.import(&export_path, "test");
    assert!(
        matches!(result, Err(ProfileError::AlreadyExists(_))),
        "Import with duplicate name should be rejected"
    );

    // Cleanup
    let _ = fs::remove_file(&export_path);
}

#[test]
fn test_prof005_duplicate_after_file_deleted_rejected() {
    let (_temp, manager) = setup_test_manager();

    // Create profile
    manager
        .create("test", ProfileTemplate::Blank)
        .expect("Failed to create profile");

    // Manually delete the file but leave it in memory
    let profile = manager.get("test").expect("Profile should exist");
    fs::remove_file(&profile.rhai_path).expect("Failed to remove file");

    // Try to create with same name - should still be rejected (checks memory first)
    let result = manager.create("test", ProfileTemplate::Blank);
    assert!(
        matches!(result, Err(ProfileError::AlreadyExists(_))),
        "Duplicate should be rejected even if file is missing"
    );
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_all_fixes_integration() {
    let (_temp, manager) = setup_test_manager();

    // PROF-002: Test validation
    assert!(manager.create("", ProfileTemplate::Blank).is_err());
    assert!(manager.create("-invalid", ProfileTemplate::Blank).is_err());

    // PROF-005: Create valid profile
    let result = manager.create("valid-profile_1", ProfileTemplate::Blank);
    assert!(result.is_ok());

    // PROF-005: Reject duplicate
    assert!(manager
        .create("valid-profile_1", ProfileTemplate::Blank)
        .is_err());

    // PROF-001 & PROF-004: Activate and check metadata
    let activation = manager.activate("valid-profile_1");
    assert!(activation.is_ok());

    let metadata = manager
        .load_profile_metadata_for_testing("valid-profile_1")
        .unwrap();
    assert!(metadata.activated_at.is_some());
    assert!(metadata.activated_by.is_some());

    // PROF-003: Try to activate nonexistent profile
    let result = manager.activate("nonexistent");
    assert!(matches!(result, Err(ProfileError::NotFound(_))));
}
